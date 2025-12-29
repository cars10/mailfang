use clap::Parser;
use std::net::SocketAddr;
use tracing::info;

#[derive(Parser, Debug, Clone)]
#[command(
    name = "mailfang",
    about = "Email testing server that receives emails via SMTP and provides a web interface to view them.",
    long_about = "Email testing server that receives emails via SMTP and provides a web interface to view them.

The SMTP server supports the following authentication methods:
  - PLAIN: Simple username/password authentication
  - LOGIN: Base64-encoded username/password authentication
  - CRAM-MD5: Challenge-response authentication using HMAC-MD5

If SMTP authentication credentials are not configured, all authentication attempts will be accepted.",
    author,
    version
)]
pub struct Config {
    #[arg(long, env = "SMTP_HOST", default_value = "0.0.0.0:2525")]
    pub smtp_host: String,

    #[arg(long, env = "SMTP_USERNAME", help = "SMTP authentication username")]
    pub smtp_username: Option<String>,

    #[arg(long, env = "SMTP_PASSWORD", help = "SMTP authentication password")]
    pub smtp_password: Option<String>,

    #[arg(
        long,
        env = "SMTP_MAX_CONNECTIONS",
        default_value = "4",
        help = "Maximum number of concurrent SMTP connections"
    )]
    pub smtp_max_connections: usize,

    #[arg(long, env = "WEB_HOST", default_value = "0.0.0.0:3000")]
    pub web_host: String,

    #[arg(
        long,
        env = "DATABASE_URL",
        default_value = "sqlite::memory:",
        help = "SQLite database URL. Defaults to in-memory database."
    )]
    pub database_url: String,
}

impl Config {
    pub fn smtp_socket_addr(&self) -> SocketAddr {
        self.smtp_host.parse().expect("valid SMTP listen addr")
    }

    pub fn web_socket_addr(&self) -> SocketAddr {
        self.web_host.parse().expect("valid web listen addr")
    }

    pub fn print(&self) {
        info!(component = "config", "SMTP host: {}", self.smtp_host);
        info!(
            component = "config",
            "SMTP username: {}",
            self.smtp_username.as_deref().unwrap_or("")
        );
        info!(
            component = "config",
            "SMTP password: {}",
            if self.smtp_password.is_some() {
                "****"
            } else {
                ""
            }
        );

        info!(
            component = "config",
            "SMTP max connections: {}", self.smtp_max_connections
        );
        info!(component = "config", "Web host: {}", self.web_host);
        info!(component = "config", "Database URL: {}", self.database_url);
    }
}
