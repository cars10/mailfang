use mailswallow_backend::{db, migration, smtp, web};
use sea_orm_migration::prelude::*;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::broadcast;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read ports from environment variables
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

    // Initialize database and run migrations
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://mailswallow.db".to_string());
    let db = Arc::new(
        sea_orm::Database::connect(&database_url)
            .await
            .expect("Failed to connect to database"),
    );

    // Run migrations using SeaORM Migrator
    migration::Migrator::up(&*db, None).await?;
    println!("Database migrations completed");

    // Create broadcast channel for WebSocket notifications
    let (broadcast_tx, _) = broadcast::channel::<web::WebSocketMessage>(100);

    // Create callback that saves to database and sends WebSocket notification
    let db_for_smtp = db.clone();
    let broadcast_for_smtp = broadcast_tx.clone();
    let save_callback = move |message: &smtp::Email| {
        let db = db_for_smtp.clone();
        let broadcast = broadcast_for_smtp.clone();
        let message = message.clone();
        tokio::spawn(async move {
            match db::save_email(&db, &message).await {
                Ok(email_id) => {
                    // Fetch the email list record and counts to send in the websocket message
                    let email_result = db::get_email_by_id(&db, &email_id).await;
                    let counts_result = db::get_email_stats(&db).await;

                    let email_list_record = if let Ok(Some(email_record)) = email_result {
                        // Convert to EmailListRecord
                        Some(db::EmailListRecord {
                            id: email_record.id,
                            subject: email_record.subject,
                            date: email_record.date,
                            created_at: email_record.created_at,
                            from: email_record.from,
                            to: email_record.to,
                            read: email_record.read,
                            has_attachments: !email_record.attachments.is_empty(),
                        })
                    } else {
                        None
                    };

                    let counts = counts_result.ok();

                    // Send WebSocket notification with the email data and counts
                    let _ = broadcast.send(web::WebSocketMessage {
                        event: "new_email".to_string(),
                        email: email_list_record,
                        counts,
                    });
                }
                Err(e) => {
                    eprintln!("Failed to save email to database: {}", e);
                }
            }
        });
    };

    // Run SMTP and web servers concurrently
    let smtp_server = smtp::SmtpServer::new(smtp_addr).on_receive(save_callback);

    // Only set static_dir if STATIC_DIR env var is explicitly set (production mode)
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
