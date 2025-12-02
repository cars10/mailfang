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
    let (broadcast_tx, _) = broadcast::channel::<String>(100);

    // Create callback that saves to database and sends WebSocket notification
    let db_for_smtp = db.clone();
    let broadcast_for_smtp = broadcast_tx.clone();
    let save_callback = move |message: &smtp::Email| {
        let db = db_for_smtp.clone();
        let broadcast = broadcast_for_smtp.clone();
        let message = message.clone();
        tokio::spawn(async move {
            if let Err(e) = db::save_email(db.as_ref(), &message).await {
                eprintln!("Failed to save email to database: {}", e);
            } else {
                // Send WebSocket notification that a new email was saved
                let _ = broadcast.send("new_email".to_string());
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
