use clap::Parser;
use std::io;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;
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
    #[arg(
        long,
        env = "SMTP_HOST",
        help = "SMTP server listen address",
        default_value = "127.0.0.1:2525"
    )]
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

    #[arg(
        long,
        env = "WEB_HOST",
        help = "Web server listen address",
        default_value = "127.0.0.1:3000"
    )]
    pub web_host: String,

    #[arg(
        long,
        env = "DATABASE_URL",
        default_value = "sqlite://./mailfang.db",
        help = "SQLite database URL."
    )]
    pub database_url: String,
}

impl Config {
    pub fn smtp_socket_addr(&self) -> io::Result<SocketAddr> {
        resolve_socket_addr("SMTP", &self.smtp_host)
    }

    pub fn web_socket_addr(&self) -> io::Result<SocketAddr> {
        resolve_socket_addr("web", &self.web_host)
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

fn resolve_socket_addr(kind: &str, raw_addr: &str) -> io::Result<SocketAddr> {
    let mut resolved = raw_addr.to_socket_addrs().map_err(|err| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("invalid {} listen address '{}': {}", kind, raw_addr, err),
        )
    })?;

    resolved.next().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "{} listen address '{}' did not resolve to any socket addresses",
                kind, raw_addr
            ),
        )
    })
}
