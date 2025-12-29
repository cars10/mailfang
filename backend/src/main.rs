use clap::Parser;
use mailfang::{config, db, logging, migration, smtp, web};
use sea_orm_migration::prelude::*;
use std::path::Path;
use std::sync::Arc;
use std::{fs, io};
use tokio::sync::broadcast;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ctrlc::set_handler(|| {
        info!("Stopping mailfang...");
        std::process::exit(0);
    })?;

    logging::init();
    let config = setup_config();

    let db = setup_database(&config).await?;

    let (broadcast_tx, _) = broadcast::channel::<web::WebSocketMessage>(100);

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

    let static_dir = std::env::var("STATIC_DIR").ok();

    tokio::select! {
        smtp_result = smtp_server.run() => {
            smtp_result?;
        }
        web_result = web::run(config.web_socket_addr(), db, broadcast_tx, static_dir.as_deref()) => {
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

async fn setup_database(
    config: &config::Config,
) -> Result<Arc<sea_orm::DatabaseConnection>, io::Error> {
    create_sqlite_db_file(&config.database_url)?;

    let db = Arc::new(
        sea_orm::Database::connect(&config.database_url)
            .await
            .expect("Failed to connect to database"),
    );

    info!(component = "main", "Running database migrations...");
    migration::Migrator::up(&*db, None)
        .await
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    info!(component = "main", "Database migrations completed");

    Ok(db)
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
    db: Arc<sea_orm::DatabaseConnection>,
    broadcast: broadcast::Sender<web::WebSocketMessage>,
    message: smtp::Email,
) {
    tokio::spawn(async move {
        match db::save_email(&db, &message).await {
            Ok(email_id) => {
                let email_result = db::get_email_by_id(&db, &email_id).await;

                if let Ok(Some(email_record)) = email_result {
                    let recipients = email_record.recipients.clone();
                    let email_list_record: db::EmailListRecord = email_record.into();
                    broadcast
                        .send(web::WebSocketMessage {
                            event: web::WebSocketEvent::NewMail,
                            email: Some(email_list_record),
                            email_id: None,
                            recipients: Some(recipients),
                        })
                        .ok();
                }
            }
            Err(e) => {
                error!(
                    component = "smtp",
                    "Failed to save email to database: {}", e
                );
            }
        }
    });
}
