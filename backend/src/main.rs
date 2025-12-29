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
    logging::init();
    let config = config::Config::parse();
    config.print();

    ensure_sqlite_db_file(&config.database_url)?;
    let db = Arc::new(
        sea_orm::Database::connect(&config.database_url)
            .await
            .expect("Failed to connect to database"),
    );

    migration::Migrator::up(&*db, None).await?;
    info!(component = "main", "Database migrations completed");

    let (broadcast_tx, _) = broadcast::channel::<web::WebSocketMessage>(100);

    let db_for_smtp = db.clone();
    let broadcast_for_smtp = broadcast_tx.clone();
    let save_callback = move |message: &smtp::Email| {
        let db = db_for_smtp.clone();
        let broadcast = broadcast_for_smtp.clone();
        let message = message.clone();
        tokio::spawn(async move {
            match db::save_email(&db, &message).await {
                Ok(email_id) => {
                    let email_result = db::get_email_by_id(&db, &email_id).await;

                    if let Ok(Some(email_record)) = email_result {
                        let email_list_record = db::EmailListRecord {
                            id: email_record.id,
                            subject: email_record.subject,
                            date: email_record.date,
                            created_at: email_record.created_at,
                            from: email_record.from,
                            to: email_record.to,
                            read: email_record.read,
                            has_attachments: !email_record.attachments.is_empty(),
                        };
                        broadcast
                            .send(web::WebSocketMessage {
                                event: web::WebSocketEvent::NewMail,
                                email: Some(email_list_record),
                                email_id: None,
                                recipients: Some(email_record.recipients),
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
    };

    let smtp_server = smtp::SmtpServer::new(config.smtp_socket_addr())
        .max_connections(config.smtp_max_connections)
        .auth(config.smtp_username.clone(), config.smtp_password.clone())
        .on_receive(save_callback);

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

fn ensure_sqlite_db_file(database_url: &str) -> io::Result<()> {
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
