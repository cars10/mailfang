use clap::Parser;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use mailfang::{config, db, logging, smtp, web};
use std::path::Path;
use std::sync::Arc;
use std::{fs, io};
use tokio::sync::broadcast;
use tracing::{error, info};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ctrlc::set_handler(|| {
        info!("Stopping mailfang...");
        std::process::exit(0);
    })?;

    logging::init();
    let config = setup_config();

    let db = setup_database(&config).await?;

    let (broadcast_tx, _) = broadcast::channel::<web::ws::WebSocketMessage>(100);

    let db_for_smtp = db.clone();
    let broadcast_for_smtp = broadcast_tx.clone();
    let smtp_on_receive = move |message: &smtp::Email| {
        handle_new_email(
            db_for_smtp.clone(),
            broadcast_for_smtp.clone(),
            message.clone(),
        );
    };

    let smtp_server = smtp::SmtpServer::new(config.smtp_socket_addr())
        .max_connections(config.smtp_max_connections)
        .auth(config.smtp_username.clone(), config.smtp_password.clone())
        .on_receive(smtp_on_receive);

    tokio::select! {
        smtp_result = smtp_server.run() => {
            smtp_result?;
        }
        web_result = web::run(config.web_socket_addr(), db, broadcast_tx) => {
            web_result?;
        }
    }

    Ok(())
}

fn setup_config() -> config::Config {
    let config = config::Config::parse();
    config.print();
    config
}

async fn setup_database(config: &config::Config) -> Result<db::DbPool, io::Error> {
    create_sqlite_db_file(&config.database_url)?;

    let url = config
        .database_url
        .trim_start_matches("sqlite://")
        .trim_start_matches("sqlite:");
    let manager = ConnectionManager::<SqliteConnection>::new(url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    info!(component = "main", "Database connected");

    // Run database migrations
    let mut conn = pool
        .get()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    conn.run_pending_migrations(MIGRATIONS)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    info!(component = "main", "Database migrations completed");

    Ok(Arc::new(pool))
}

fn create_sqlite_db_file(database_url: &str) -> io::Result<()> {
    if !database_url.starts_with("sqlite:") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Database URL must start with 'sqlite:'",
        ));
    }

    if database_url.contains(":memory:") {
        return Ok(());
    }

    let path_str = database_url
        .trim_start_matches("sqlite://")
        .trim_start_matches("sqlite:")
        .split('?')
        .next()
        .unwrap_or("");

    if path_str.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "SQLite database URL is missing a file path",
        ));
    }

    let path = Path::new(path_str);

    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }

    // create the file if missing
    if !path.exists() {
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)?;
    }

    Ok(())
}

fn handle_new_email(
    db: db::DbPool,
    broadcast: broadcast::Sender<web::ws::WebSocketMessage>,
    message: smtp::Email,
) {
    tokio::spawn(async move {
        let db_clone = db.clone();
        let message_clone = message.clone();
        let email_id_result = tokio::task::spawn_blocking(move || {
            let mut conn = db_clone.get().map_err(|e| {
                diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::UnableToSendCommand,
                    Box::new(e.to_string()),
                )
            })?;
            db::save_email::save_email(&mut conn, &message_clone)
        })
        .await;

        match email_id_result {
            Ok(Ok(email_id)) => {
                let db_clone = db.clone();
                let email_result = tokio::task::spawn_blocking(move || {
                    let mut conn = db_clone.get().map_err(|e| {
                        diesel::result::Error::DatabaseError(
                            diesel::result::DatabaseErrorKind::UnableToSendCommand,
                            Box::new(e.to_string()),
                        )
                    })?;
                    db::get_email_by_id(&mut conn, &email_id)
                })
                .await;

                if let Ok(Ok(Some(email_record))) = email_result {
                    let recipients = email_record.recipients.clone();
                    let email_list_record: db::EmailListRecord = email_record.into();
                    broadcast
                        .send(web::ws::WebSocketMessage {
                            event: web::ws::WebSocketEvent::NewMail,
                            email: Some(email_list_record),
                            email_id: None,
                            recipients: Some(recipients),
                        })
                        .ok();
                }
            }
            Ok(Err(e)) => {
                error!(
                    component = "smtp",
                    "Failed to save email to database: {}", e
                );
            }
            Err(e) => {
                error!(component = "smtp", "Failed to spawn blocking task: {}", e);
            }
        }
    });
}
