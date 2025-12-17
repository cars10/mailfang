use mailfang_backend::{db, logging, migration, smtp, web};
use sea_orm_migration::prelude::*;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::{fs, io};
use tokio::sync::broadcast;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    logging::init();
    let smtp_port = std::env::var("SMTP_PORT")
        .unwrap_or_else(|_| "2525".to_string())
        .parse::<u16>()?;
    let web_port = std::env::var("WEB_PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()?;

    let smtp_addr: SocketAddr = format!("0.0.0.0:{}", smtp_port)
        .parse()
        .expect("valid SMTP listen addr");
    let web_addr: SocketAddr = format!("0.0.0.0:{}", web_port)
        .parse()
        .expect("valid web listen addr");

    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:///app/mailfang.db".to_string());
    ensure_sqlite_db_file(&database_url)?;
    let db = Arc::new(
        sea_orm::Database::connect(&database_url)
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
                    info!(component = "smtp", email_id = %email_id, "Email saved to database");
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

    let smtp_server = smtp::SmtpServer::new(smtp_addr).on_receive(save_callback);

    let static_dir = std::env::var("STATIC_DIR").ok();

    tokio::select! {
        smtp_result = smtp_server.run() => {
            smtp_result?;
        }
        web_result = web::run_web_server(web_addr, db, broadcast_tx, static_dir.as_deref()) => {
            web_result?;
        }
    }

    Ok(())
}

fn ensure_sqlite_db_file(database_url: &str) -> io::Result<()> {
    if !database_url.starts_with("sqlite:") {
        return Ok(());
    }

    if database_url.contains(":memory:") {
        return Ok(());
    }

    // Handle both sqlite:// and sqlite:/ formats
    let path_part = if database_url.starts_with("sqlite://") {
        database_url.trim_start_matches("sqlite://")
    } else if database_url.starts_with("sqlite:/") {
        database_url.trim_start_matches("sqlite:/")
    } else {
        database_url.trim_start_matches("sqlite:")
    };

    let path: PathBuf = if path_part.starts_with('/') {
        PathBuf::from(path_part)
    } else {
        std::env::current_dir()?.join(path_part)
    };

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    if !path.exists() {
        fs::File::create(path)?;
    }

    Ok(())
}
