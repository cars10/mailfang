use super::parser::{EmailAttachment, parse_email_details};
use chrono::Utc;
use futures::{SinkExt, StreamExt};
use r2d2::Pool;
use rand::Rng;
use smtp_proto::Request;
use std::fmt;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::{Framed, LinesCodec, LinesCodecError};
use tracing::{error, info};
use uuid::Uuid;

/// Decode base64 string, handling padding issues
fn base64_decode(input: &str) -> std::result::Result<String, String> {
    use base64::Engine;
    let engine = base64::engine::general_purpose::STANDARD;
    let decoded = engine
        .decode(input.trim())
        .map_err(|e| format!("Base64 decode error: {}", e))?;
    String::from_utf8(decoded).map_err(|e| format!("UTF-8 decode error: {}", e))
}

pub type Result<T> = std::result::Result<T, SmtpError>;

/// Callback function type for handling received emails
pub type OnReceiveCallback = Arc<dyn Fn(&Email) + Send + Sync>;

#[derive(Debug)]
struct ConnectionSlot;

#[derive(Debug)]
struct ConnectionManager;

impl r2d2::ManageConnection for ConnectionManager {
    type Connection = ConnectionSlot;
    type Error = std::io::Error;

    fn connect(&self) -> std::result::Result<Self::Connection, Self::Error> {
        Ok(ConnectionSlot)
    }

    fn is_valid(&self, _conn: &mut Self::Connection) -> std::result::Result<(), Self::Error> {
        Ok(())
    }

    fn has_broken(&self, _conn: &mut Self::Connection) -> bool {
        false
    }
}

pub struct SmtpServer {
    addr: SocketAddr,
    on_receive: Option<OnReceiveCallback>,
    max_connections: usize,
    auth_username: Option<String>,
    auth_password: Option<String>,
}

impl SmtpServer {
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            addr,
            on_receive: None,
            max_connections: 0,
            auth_username: None,
            auth_password: None,
        }
    }

    pub fn on_receive<F>(mut self, callback: F) -> Self
    where
        F: Fn(&Email) + Send + Sync + 'static,
    {
        self.on_receive = Some(Arc::new(callback));
        self
    }

    pub fn auth(mut self, username: Option<String>, password: Option<String>) -> Self {
        self.auth_username = username;
        self.auth_password = password;
        self
    }

    pub fn max_connections(mut self, max: usize) -> Self {
        self.max_connections = max.max(1);
        self
    }

    pub fn address(&self) -> SocketAddr {
        self.addr
    }

    pub async fn run(&self) -> Result<()> {
        let listener = TcpListener::bind(self.addr).await?;
        let on_receive = self.on_receive.clone();

        let manager = ConnectionManager;
        let pool = Arc::new(
            Pool::builder()
                .max_size(self.max_connections as u32)
                .build(manager)
                .map_err(|e| {
                    SmtpError::Io(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Failed to create connection pool: {}", e),
                    ))
                })?,
        );

        info!(
            component = "smtp",
            "SMTP server listening on {} (max connections: {})", self.addr, self.max_connections
        );

        loop {
            let (stream, peer) = listener.accept().await?;
            info!(component = "smtp", peer = %peer, "Connection accepted");
            let on_receive = on_receive.clone();
            let pool = pool.clone();

            let auth_username = self.auth_username.clone();
            let auth_password = self.auth_password.clone();
            tokio::spawn(async move {
                let connection_result = tokio::task::spawn_blocking({
                    let pool = pool.clone();
                    move || pool.get()
                })
                .await;

                let _ = match connection_result {
                    Ok(Ok(conn)) => conn,
                    Ok(Err(e)) => {
                        error!(component = "smtp", peer = %peer, "Failed to get connection from pool: {}", e);
                        return;
                    }
                    Err(e) => {
                        error!(component = "smtp", peer = %peer, "Failed to spawn blocking task: {}", e);
                        return;
                    }
                };

                if let Err(err) =
                    handle_connection(stream, on_receive, auth_username, auth_password, peer).await
                {
                    error!(component = "smtp", peer = %peer, "SMTP session failed: {}", err);
                }
            });
        }
    }
}

impl Clone for SmtpServer {
    fn clone(&self) -> Self {
        Self {
            addr: self.addr,
            on_receive: self.on_receive.clone(),
            max_connections: self.max_connections,
            auth_username: self.auth_username.clone(),
            auth_password: self.auth_password.clone(),
        }
    }
}

async fn handle_connection(
    stream: TcpStream,
    on_receive: Option<OnReceiveCallback>,
    auth_username: Option<String>,
    auth_password: Option<String>,
    peer: SocketAddr,
) -> Result<()> {
    let mut framed: Framed<TcpStream, LinesCodec> =
        Framed::new(stream, LinesCodec::new_with_max_length(26_214_400));
    framed.send("220 mailfang SMTP ready".to_string()).await?;

    let mut session = Session::new(on_receive, auth_username, auth_password, peer);

    while let Some(line_result) = framed.next().await {
        match line_result {
            Ok(line) => {
                match session.process_line(&line) {
                    Ok(responses) => {
                        for response in responses {
                            if let Err(e) = framed.send(response).await {
                                return Err(e.into());
                            }
                        }
                        if session.should_close() {
                            break;
                        }
                    }
                    Err(e) => {
                        let _ = framed.send(format!("500 {}", e)).await;
                        // Don't break on error, continue processing
                    }
                }
            }
            Err(e) => {
                return Err(e.into());
            }
        }
    }

    Ok(())
}

struct Session {
    state: SessionState,
    greeted: bool,
    authenticated: bool,
    mail_from: Option<String>,
    rcpt_to: Vec<String>,
    buffer: Vec<String>,
    messages: Vec<Email>,
    quit: bool,
    on_receive: Option<OnReceiveCallback>,
    auth_state: AuthState,
    auth_username: Option<String>,
    auth_password: Option<String>,
    peer: SocketAddr,
}

impl Session {
    fn new(
        on_receive: Option<OnReceiveCallback>,
        auth_username: Option<String>,
        auth_password: Option<String>,
        peer: SocketAddr,
    ) -> Self {
        // If no credentials are set, authentication is not required
        let authenticated = auth_username.is_none() || auth_password.is_none();
        Self {
            state: SessionState::Command,
            greeted: false,
            authenticated,
            mail_from: None,
            rcpt_to: Vec::new(),
            buffer: Vec::new(),
            messages: Vec::new(),
            quit: false,
            on_receive,
            auth_state: AuthState::None,
            auth_username,
            auth_password,
            peer,
        }
    }

    fn process_line(&mut self, line: &str) -> Result<Vec<String>> {
        match self.state {
            SessionState::Command => self.handle_command(line),
            SessionState::Data => self.handle_data(line),
            SessionState::Auth => self.handle_auth(line),
        }
    }

    fn handle_command(&mut self, line: &str) -> Result<Vec<String>> {
        // smtp-proto expects CRLF-terminated lines, so we append \r\n
        let line_with_crlf = format!("{}\r\n", line);
        let request = Request::parse(&mut line_with_crlf.as_bytes().iter())
            .map_err(|_| SmtpError::Protocol("Invalid command syntax"))?;

        match request {
            Request::Helo { host } => {
                self.greeted = true;
                Ok(vec![format!("250 Hello {}", host)])
            }
            Request::Ehlo { host } => {
                self.greeted = true;
                // Advertise AUTH PLAIN, LOGIN, and CRAM-MD5 capabilities
                Ok(vec![
                    format!("250-Hello {}", host),
                    "250-AUTH PLAIN LOGIN CRAM-MD5".into(),
                    "250 SIZE 26214400".into(),
                ])
            }
            Request::Mail { from } => {
                ensure(self.greeted, "503 Send HELO/EHLO first")?;
                ensure(self.authenticated, "530 Authentication required")?;
                self.mail_from = Some(from.address.to_string());
                self.rcpt_to.clear();
                Ok(vec!["250 Sender OK".into()])
            }
            Request::Rcpt { to } => {
                ensure(self.mail_from.is_some(), "503 Need MAIL FROM first")?;
                self.rcpt_to.push(to.address.to_string());
                Ok(vec!["250 Recipient OK".into()])
            }
            Request::Data => {
                ensure(!self.rcpt_to.is_empty(), "503 Need RCPT TO first")?;
                self.state = SessionState::Data;
                self.buffer.clear();
                Ok(vec!["354 End data with <CR><LF>.<CR><LF>".into()])
            }
            Request::Rset => {
                self.reset_transaction();
                Ok(vec!["250 Reset state".into()])
            }
            Request::Noop { .. } => Ok(vec!["250 OK".into()]),
            Request::Auth {
                mechanism,
                initial_response,
            } => {
                ensure(self.greeted, "503 Send HELO/EHLO first")?;

                if mechanism == smtp_proto::AUTH_PLAIN {
                    if !initial_response.is_empty() {
                        // AUTH PLAIN <base64> - credentials provided in same line
                        if self.validate_plain_auth(&initial_response) {
                            self.state = SessionState::Command;
                            self.auth_state = AuthState::None;
                            self.authenticated = true;
                            Ok(vec!["235 Authentication successful".into()])
                        } else {
                            Ok(vec!["535 Authentication failed".into()])
                        }
                    } else {
                        // AUTH PLAIN - wait for credentials on next line
                        self.state = SessionState::Auth;
                        self.auth_state = AuthState::WaitingForPlainCredentials;
                        Ok(vec!["334 ".into()]) // Base64 prompt (empty means just send credentials)
                    }
                } else if mechanism == smtp_proto::AUTH_LOGIN {
                    self.state = SessionState::Auth;
                    self.auth_state = AuthState::WaitingForLoginUsername;
                    Ok(vec!["334 VXNlcm5hbWU6".into()]) // "Username:" in base64
                } else if mechanism == smtp_proto::AUTH_CRAM_MD5 {
                    // Generate a challenge (typically timestamp-based or random)
                    let challenge = self.generate_cram_md5_challenge();
                    self.state = SessionState::Auth;
                    self.auth_state = AuthState::WaitingForCramMd5Response {
                        challenge: challenge.clone(),
                    };
                    // Send base64-encoded challenge
                    use base64::Engine;
                    let encoded =
                        base64::engine::general_purpose::STANDARD.encode(challenge.as_bytes());
                    Ok(vec![format!("334 {}", encoded)])
                } else {
                    // Unknown auth type
                    Ok(vec!["504 Unrecognized authentication type".into()])
                }
            }
            Request::Quit => {
                self.quit = true;
                Ok(vec!["221 Bye".into()])
            }
            _ => Ok(vec!["502 Command not implemented".into()]),
        }
    }

    fn handle_data(&mut self, line: &str) -> Result<Vec<String>> {
        if line == "." {
            let data = self.buffer.join("\r\n");
            let parsed_details = parse_email_details(&data);

            let message = Email {
                id: Uuid::new_v4(),
                message_id: parsed_details.message_id.clone(),
                subject: parsed_details.subject.clone(),
                date: parsed_details.date,
                headers: parsed_details.headers.clone(),
                from: self.mail_from.clone().unwrap_or_default(),
                to: self.rcpt_to.clone(),
                recipients: self.rcpt_to.clone(),
                size: data.as_bytes().len() as u64,
                data: data.clone(),
                body_text: parsed_details.body_text.clone(),
                body_html: parsed_details.body_html.clone(),
                attachments: parsed_details.attachments.clone(),
            };
            self.messages.push(message.clone());

            // Log email acceptance
            info!(
                component = "smtp",
                peer = %self.peer,
                from = %message.from,
                to = ?message.to,
                subject = ?message.subject,
                size = message.size,
                "Email accepted"
            );

            if let Some(ref callback) = self.on_receive {
                callback(&message);
            }

            self.state = SessionState::Command;
            self.buffer.clear();
            Ok(vec!["250 Message received".into()])
        } else {
            let processed_line = if line.starts_with("..") {
                &line[1..]
            } else {
                line
            };
            self.buffer.push(processed_line.to_string());
            Ok(vec![])
        }
    }

    fn should_close(&self) -> bool {
        self.quit
    }

    fn handle_auth(&mut self, line: &str) -> Result<Vec<String>> {
        match &self.auth_state {
            AuthState::WaitingForPlainCredentials => {
                if self.validate_plain_auth(line) {
                    self.state = SessionState::Command;
                    self.auth_state = AuthState::None;
                    self.authenticated = true;
                    Ok(vec!["235 Authentication successful".into()])
                } else {
                    self.state = SessionState::Command;
                    self.auth_state = AuthState::None;
                    Ok(vec!["535 Authentication failed".into()])
                }
            }
            AuthState::WaitingForLoginUsername => {
                let username = match base64_decode(line) {
                    Ok(u) => u,
                    Err(_) => {
                        self.state = SessionState::Command;
                        self.auth_state = AuthState::None;
                        return Ok(vec!["535 Authentication failed".into()]);
                    }
                };
                self.auth_state = AuthState::WaitingForLoginPassword { username };
                // Base64 for "Password:"
                Ok(vec!["334 UGFzc3dvcmQ6".into()])
            }
            AuthState::WaitingForLoginPassword { username } => {
                let password = match base64_decode(line) {
                    Ok(p) => p,
                    Err(_) => {
                        self.state = SessionState::Command;
                        self.auth_state = AuthState::None;
                        return Ok(vec!["535 Authentication failed".into()]);
                    }
                };
                if self.validate_login_auth(&username, &password) {
                    self.state = SessionState::Command;
                    self.auth_state = AuthState::None;
                    self.authenticated = true;
                    Ok(vec!["235 Authentication successful".into()])
                } else {
                    self.state = SessionState::Command;
                    self.auth_state = AuthState::None;
                    Ok(vec!["535 Authentication failed".into()])
                }
            }
            AuthState::WaitingForCramMd5Response { challenge } => {
                if self.validate_cram_md5_auth(line, &challenge) {
                    self.state = SessionState::Command;
                    self.auth_state = AuthState::None;
                    self.authenticated = true;
                    Ok(vec!["235 Authentication successful".into()])
                } else {
                    self.state = SessionState::Command;
                    self.auth_state = AuthState::None;
                    Ok(vec!["535 Authentication failed".into()])
                }
            }
            _ => {
                self.state = SessionState::Command;
                self.auth_state = AuthState::None;
                Ok(vec!["503 Bad sequence of commands".into()])
            }
        }
    }

    fn validate_plain_auth(&self, base64_credentials: &str) -> bool {
        if self.auth_username.is_none() || self.auth_password.is_none() {
            return true;
        }

        let expected_username = self.auth_username.as_ref().unwrap();
        let expected_password = self.auth_password.as_ref().unwrap();

        let decoded = match base64_decode(base64_credentials) {
            Ok(d) => d,
            Err(_) => {
                return false;
            }
        };

        let parts: Vec<&str> = decoded.split('\0').collect();
        if parts.len() >= 3 {
            let username = parts[1];
            let password = parts[2];
            username == expected_username && password == expected_password
        } else {
            false
        }
    }

    fn validate_login_auth(&self, username: &str, password: &str) -> bool {
        if self.auth_username.is_none() || self.auth_password.is_none() {
            return true;
        }

        let expected_username = self.auth_username.as_ref().unwrap();
        let expected_password = self.auth_password.as_ref().unwrap();
        username == expected_username && password == expected_password
    }

    fn generate_cram_md5_challenge(&self) -> String {
        use base64::Engine;
        let mut rng = rand::thread_rng();
        let random_bytes: Vec<u8> = (0..16).map(|_| rng.r#gen::<u8>()).collect();
        let challenge = format!(
            "<{}@mailfang.com>",
            base64::engine::general_purpose::STANDARD.encode(&random_bytes)
        );
        challenge
    }

    fn validate_cram_md5_auth(&self, response: &str, challenge: &str) -> bool {
        if self.auth_username.is_none() || self.auth_password.is_none() {
            return true;
        }

        let expected_username = self.auth_username.as_ref().unwrap();
        let expected_password = self.auth_password.as_ref().unwrap();

        let decoded = match base64_decode(response) {
            Ok(d) => d,
            Err(_) => return false,
        };

        let parts: Vec<&str> = decoded.splitn(2, ' ').collect();
        if parts.len() != 2 {
            return false;
        }

        let username = parts[0];
        let received_hmac_hex = parts[1];

        if username != expected_username {
            return false;
        }

        // HMAC-MD5(K, m) = MD5((K' ⊕ opad) || MD5((K' ⊕ ipad) || m))
        let key = expected_password.as_bytes();
        let mut key_padded = [0u8; 64];
        if key.len() > 64 {
            // If key is longer than 64 bytes, hash it first
            let key_hash = md5::compute(key);
            key_padded[..16].copy_from_slice(&key_hash.0);
        } else {
            key_padded[..key.len()].copy_from_slice(key);
        }

        // Inner pad: key ⊕ 0x36
        let mut ipad = key_padded;
        for byte in &mut ipad {
            *byte ^= 0x36;
        }

        // Inner hash: MD5((K' ⊕ ipad) || challenge)
        let mut inner_data = Vec::with_capacity(64 + challenge.len());
        inner_data.extend_from_slice(&ipad);
        inner_data.extend_from_slice(challenge.as_bytes());
        let inner_hash = md5::compute(&inner_data);

        // Outer pad: key ⊕ 0x5c
        let mut opad = key_padded;
        for byte in &mut opad {
            *byte ^= 0x5c;
        }

        // Outer hash: MD5((K' ⊕ opad) || inner_hash)
        let mut outer_data = Vec::with_capacity(64 + 16);
        outer_data.extend_from_slice(&opad);
        outer_data.extend_from_slice(&inner_hash.0);
        let computed_hmac = md5::compute(&outer_data);
        let computed_hex = hex::encode(computed_hmac.0);

        // Compare (case-insensitive)
        received_hmac_hex.to_lowercase() == computed_hex.to_lowercase()
    }

    fn reset_transaction(&mut self) {
        self.mail_from = None;
        self.rcpt_to.clear();
        self.buffer.clear();
    }

    #[cfg(test)]
    fn last_message(&self) -> Option<&Email> {
        self.messages.last()
    }
}

#[derive(Debug, Copy, Clone)]
enum SessionState {
    Command,
    Data,
    Auth,
}

#[derive(Debug, Clone)]
enum AuthState {
    None,
    WaitingForPlainCredentials,
    WaitingForLoginUsername,
    WaitingForLoginPassword { username: String },
    WaitingForCramMd5Response { challenge: String },
}

impl Default for SessionState {
    fn default() -> Self {
        SessionState::Command
    }
}

#[derive(Debug, Clone)]
pub struct Email {
    pub id: Uuid,
    pub message_id: Option<String>,
    pub subject: Option<String>,
    pub date: Option<chrono::DateTime<Utc>>,
    pub headers: Option<serde_json::Value>,
    pub from: String,            // SMTP envelope sender (MAIL FROM)
    pub to: Vec<String>,         // SMTP envelope recipients (RCPT TO)
    pub recipients: Vec<String>, // Same as `to` - all SMTP envelope recipients
    pub size: u64,
    pub data: String,
    pub body_text: String,
    pub body_html: String,
    pub attachments: Vec<EmailAttachment>,
}

fn ensure(condition: bool, err: &'static str) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(SmtpError::Protocol(err))
    }
}

#[derive(Debug)]
pub enum SmtpError {
    Io(std::io::Error),
    Codec(LinesCodecError),
    Protocol(&'static str),
    InvalidAddress,
}

impl fmt::Display for SmtpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SmtpError::Io(err) => write!(f, "io error: {err}"),
            SmtpError::Codec(err) => write!(f, "codec error: {err}"),
            SmtpError::Protocol(msg) => write!(f, "protocol error: {msg}"),
            SmtpError::InvalidAddress => write!(f, "invalid address syntax"),
        }
    }
}

impl std::error::Error for SmtpError {}

impl From<std::io::Error> for SmtpError {
    fn from(value: std::io::Error) -> Self {
        SmtpError::Io(value)
    }
}

impl From<LinesCodecError> for SmtpError {
    fn from(value: LinesCodecError) -> Self {
        SmtpError::Codec(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine;

    #[test]
    fn handles_basic_flow() {
        let peer: SocketAddr = "127.0.0.1:12345".parse().unwrap();
        let mut session = Session::new(None, None, None, peer);
        assert_eq!(
            session.process_line("EHLO localhost").unwrap(),
            vec![
                "250-Hello localhost",
                "250-AUTH PLAIN LOGIN CRAM-MD5",
                "250 SIZE 26214400"
            ]
        );
        assert_eq!(
            session
                .process_line("MAIL FROM:<sender@example.com>")
                .unwrap(),
            vec!["250 Sender OK"]
        );
        assert_eq!(
            session
                .process_line("RCPT TO:<recipient@example.com>")
                .unwrap(),
            vec!["250 Recipient OK"]
        );
        assert_eq!(
            session.process_line("DATA").unwrap(),
            vec!["354 End data with <CR><LF>.<CR><LF>"]
        );

        assert!(session.process_line("Subject: Hi").unwrap().is_empty());
        assert!(session.process_line("").unwrap().is_empty()); // Blank line between headers and body
        assert!(session.process_line("Body line").unwrap().is_empty());
        assert_eq!(
            session.process_line(".").unwrap(),
            vec!["250 Message received"]
        );

        let stored = session.last_message().unwrap();
        assert_eq!(stored.from, "sender@example.com");
        assert_eq!(stored.to, vec!["recipient@example.com"]);
        assert_eq!(stored.recipients, vec!["recipient@example.com"]);
        assert_eq!(stored.data, "Subject: Hi\r\n\r\nBody line");
        assert_eq!(stored.size, stored.data.as_bytes().len() as u64);
        assert_eq!(stored.body_text, "Body line");
        assert_eq!(stored.body_html, "");
        // Date might be None if not in email, so we just check it's not in the future if present
        if let Some(date) = stored.date {
            assert!(Utc::now() >= date);
        }
        assert_eq!(stored.attachments.len(), 0);
        assert_eq!(stored.subject.as_deref(), Some("Hi"));
        assert!(stored.message_id.is_none());
    }

    #[test]
    fn handles_cc_and_bcc_recipients() {
        let peer: SocketAddr = "127.0.0.1:12345".parse().unwrap();
        let mut session = Session::new(None, None, None, peer);
        assert_eq!(
            session.process_line("EHLO localhost").unwrap(),
            vec![
                "250-Hello localhost",
                "250-AUTH PLAIN LOGIN CRAM-MD5",
                "250 SIZE 26214400"
            ]
        );
        assert_eq!(
            session
                .process_line("MAIL FROM:<sender@example.com>")
                .unwrap(),
            vec!["250 Sender OK"]
        );
        // Add multiple RCPT TO commands (To, CC, BCC all use RCPT TO in SMTP)
        assert_eq!(
            session.process_line("RCPT TO:<to@example.com>").unwrap(),
            vec!["250 Recipient OK"]
        );
        assert_eq!(
            session.process_line("RCPT TO:<cc@example.com>").unwrap(),
            vec!["250 Recipient OK"]
        );
        assert_eq!(
            session.process_line("RCPT TO:<bcc@example.com>").unwrap(),
            vec!["250 Recipient OK"]
        );
        assert_eq!(
            session.process_line("DATA").unwrap(),
            vec!["354 End data with <CR><LF>.<CR><LF>"]
        );

        // Include To, Cc, and Bcc headers in the email
        assert!(
            session
                .process_line("From: sender@example.com")
                .unwrap()
                .is_empty()
        );
        assert!(
            session
                .process_line("To: to@example.com")
                .unwrap()
                .is_empty()
        );
        assert!(
            session
                .process_line("Cc: cc@example.com")
                .unwrap()
                .is_empty()
        );
        assert!(
            session
                .process_line("Bcc: bcc@example.com")
                .unwrap()
                .is_empty()
        );
        assert!(
            session
                .process_line("Subject: Test with CC and BCC")
                .unwrap()
                .is_empty()
        );
        assert!(session.process_line("").unwrap().is_empty()); // Blank line between headers and body
        assert!(session.process_line("Body text").unwrap().is_empty());
        assert_eq!(
            session.process_line(".").unwrap(),
            vec!["250 Message received"]
        );

        let stored = session.last_message().unwrap();
        assert_eq!(stored.from, "sender@example.com");
        // Both "to" and "recipients" fields should contain all SMTP envelope recipients
        // Header data is only used when actually requesting headers
        assert_eq!(
            stored.to,
            vec!["to@example.com", "cc@example.com", "bcc@example.com"]
        );
        assert_eq!(
            stored.recipients,
            vec!["to@example.com", "cc@example.com", "bcc@example.com"]
        );
        assert!(stored.data.contains("To: to@example.com"));
        assert!(stored.data.contains("Cc: cc@example.com"));
        assert!(stored.data.contains("Bcc: bcc@example.com"));
        assert_eq!(stored.body_text, "Body text");
    }

    #[test]
    fn rejects_out_of_order_commands() {
        let peer: SocketAddr = "127.0.0.1:12345".parse().unwrap();
        let mut session = Session::new(None, None, None, peer);
        assert!(
            session
                .process_line("MAIL FROM:<sender@example.com>")
                .is_err()
        );
        assert!(
            session
                .process_line("RCPT TO:<recipient@example.com>")
                .is_err()
        );

        session.process_line("EHLO example").unwrap();
        assert!(
            session
                .process_line("RCPT TO:<recipient@example.com>")
                .is_err()
        );
    }

    #[test]
    fn handles_quit() {
        let peer: SocketAddr = "127.0.0.1:12345".parse().unwrap();
        let mut session = Session::new(None, None, None, peer);
        session.process_line("EHLO localhost").unwrap();
        let responses = session.process_line("QUIT").unwrap();
        assert_eq!(responses, vec!["221 Bye"]);
        assert!(session.should_close());
    }

    #[test]
    fn accepts_all_when_no_credentials() {
        let peer: SocketAddr = "127.0.0.1:12345".parse().unwrap();
        let mut session = Session::new(None, None, None, peer);
        assert!(session.authenticated); // Should be pre-authenticated when no creds

        session.process_line("EHLO localhost").unwrap();
        // Should be able to send mail without auth
        assert!(
            session
                .process_line("MAIL FROM:<sender@example.com>")
                .is_ok()
        );
    }

    #[test]
    fn requires_auth_when_credentials_set() {
        let peer: SocketAddr = "127.0.0.1:12345".parse().unwrap();
        let mut session = Session::new(
            None,
            Some("user".to_string()),
            Some("pass".to_string()),
            peer,
        );
        assert!(!session.authenticated); // Should require auth

        session.process_line("EHLO localhost").unwrap();
        // Should reject MAIL FROM without auth
        assert!(
            session
                .process_line("MAIL FROM:<sender@example.com>")
                .is_err()
        );
    }

    #[test]
    fn auth_plain_single_line() {
        let engine = base64::engine::general_purpose::STANDARD;
        let peer: SocketAddr = "127.0.0.1:12345".parse().unwrap();

        let mut session = Session::new(
            None,
            Some("user".to_string()),
            Some("pass".to_string()),
            peer,
        );
        session.process_line("EHLO localhost").unwrap();

        // Create base64 encoded credentials: \0user\0pass
        let credentials = format!("\0user\0pass");
        let encoded = engine.encode(credentials.as_bytes());

        let response = session
            .process_line(&format!("AUTH PLAIN {}", encoded))
            .unwrap();
        assert_eq!(response, vec!["235 Authentication successful"]);
        assert!(session.authenticated);

        // Now should be able to send mail
        assert!(
            session
                .process_line("MAIL FROM:<sender@example.com>")
                .is_ok()
        );
    }

    #[test]
    fn auth_plain_two_line() {
        let engine = base64::engine::general_purpose::STANDARD;
        let peer: SocketAddr = "127.0.0.1:12345".parse().unwrap();

        let mut session = Session::new(
            None,
            Some("user".to_string()),
            Some("pass".to_string()),
            peer,
        );
        session.process_line("EHLO localhost").unwrap();

        // AUTH PLAIN without credentials
        let response = session.process_line("AUTH PLAIN").unwrap();
        assert_eq!(response, vec!["334 "]);

        // Send credentials
        let credentials = format!("\0user\0pass");
        let encoded = engine.encode(credentials.as_bytes());
        let response = session.process_line(&encoded).unwrap();
        assert_eq!(response, vec!["235 Authentication successful"]);
        assert!(session.authenticated);
    }

    #[test]
    fn auth_plain_rejects_wrong_credentials() {
        let engine = base64::engine::general_purpose::STANDARD;
        let peer: SocketAddr = "127.0.0.1:12345".parse().unwrap();

        let mut session = Session::new(
            None,
            Some("user".to_string()),
            Some("pass".to_string()),
            peer,
        );
        session.process_line("EHLO localhost").unwrap();

        // Wrong password
        let credentials = format!("\0user\0wrong");
        let encoded = engine.encode(credentials.as_bytes());
        let response = session
            .process_line(&format!("AUTH PLAIN {}", encoded))
            .unwrap();
        assert_eq!(response, vec!["535 Authentication failed"]);
        assert!(!session.authenticated);
    }

    #[test]
    fn auth_login() {
        let engine = base64::engine::general_purpose::STANDARD;
        let peer: SocketAddr = "127.0.0.1:12345".parse().unwrap();

        let mut session = Session::new(
            None,
            Some("user".to_string()),
            Some("pass".to_string()),
            peer,
        );
        session.process_line("EHLO localhost").unwrap();

        // Start LOGIN auth
        let response = session.process_line("AUTH LOGIN").unwrap();
        assert_eq!(response, vec!["334 VXNlcm5hbWU6"]); // "Username:" in base64

        // Send username
        let username_encoded = engine.encode("user");
        let response = session.process_line(&username_encoded).unwrap();
        assert_eq!(response, vec!["334 UGFzc3dvcmQ6"]); // "Password:" in base64

        // Send password
        let password_encoded = engine.encode("pass");
        let response = session.process_line(&password_encoded).unwrap();
        assert_eq!(response, vec!["235 Authentication successful"]);
        assert!(session.authenticated);

        // Now should be able to send mail
        assert!(
            session
                .process_line("MAIL FROM:<sender@example.com>")
                .is_ok()
        );
    }

    #[test]
    fn auth_login_rejects_wrong_credentials() {
        let engine = base64::engine::general_purpose::STANDARD;
        let peer: SocketAddr = "127.0.0.1:12345".parse().unwrap();

        let mut session = Session::new(
            None,
            Some("user".to_string()),
            Some("pass".to_string()),
            peer,
        );
        session.process_line("EHLO localhost").unwrap();

        // Start LOGIN auth
        session.process_line("AUTH LOGIN").unwrap();

        // Send username
        let username_encoded = engine.encode("user");
        session.process_line(&username_encoded).unwrap();

        // Send wrong password
        let password_encoded = engine.encode("wrong");
        let response = session.process_line(&password_encoded).unwrap();
        assert_eq!(response, vec!["535 Authentication failed"]);
        assert!(!session.authenticated);
    }

    #[test]
    fn auth_plain_accepts_all_when_no_credentials() {
        let engine = base64::engine::general_purpose::STANDARD;
        let peer: SocketAddr = "127.0.0.1:12345".parse().unwrap();

        let mut session = Session::new(None, None, None, peer);
        session.process_line("EHLO localhost").unwrap();

        // Any credentials should be accepted
        let credentials = format!("\0anyuser\0anypass");
        let encoded = engine.encode(credentials.as_bytes());
        let response = session
            .process_line(&format!("AUTH PLAIN {}", encoded))
            .unwrap();
        assert_eq!(response, vec!["235 Authentication successful"]);
        assert!(session.authenticated);
    }

    #[test]
    fn auth_login_accepts_all_when_no_credentials() {
        let engine = base64::engine::general_purpose::STANDARD;
        let peer: SocketAddr = "127.0.0.1:12345".parse().unwrap();

        let mut session = Session::new(None, None, None, peer);
        session.process_line("EHLO localhost").unwrap();

        // Start LOGIN auth
        session.process_line("AUTH LOGIN").unwrap();

        // Any username/password should be accepted
        let username_encoded = engine.encode("anyuser");
        session.process_line(&username_encoded).unwrap();

        let password_encoded = engine.encode("anypass");
        let response = session.process_line(&password_encoded).unwrap();
        assert_eq!(response, vec!["235 Authentication successful"]);
        assert!(session.authenticated);
    }

    // Helper function to compute HMAC-MD5 for testing
    fn compute_hmac_md5(key: &[u8], message: &[u8]) -> String {
        let mut key_padded = [0u8; 64];
        if key.len() > 64 {
            let key_hash = md5::compute(key);
            key_padded[..16].copy_from_slice(&key_hash.0);
        } else {
            key_padded[..key.len()].copy_from_slice(key);
        }

        // Inner pad: key ⊕ 0x36
        let mut ipad = key_padded;
        for byte in &mut ipad {
            *byte ^= 0x36;
        }

        // Inner hash: MD5((K' ⊕ ipad) || message)
        let mut inner_data = Vec::with_capacity(64 + message.len());
        inner_data.extend_from_slice(&ipad);
        inner_data.extend_from_slice(message);
        let inner_hash = md5::compute(&inner_data);

        // Outer pad: key ⊕ 0x5c
        let mut opad = key_padded;
        for byte in &mut opad {
            *byte ^= 0x5c;
        }

        // Outer hash: MD5((K' ⊕ opad) || inner_hash)
        let mut outer_data = Vec::with_capacity(64 + 16);
        outer_data.extend_from_slice(&opad);
        outer_data.extend_from_slice(&inner_hash.0);
        let computed_hmac = md5::compute(&outer_data);
        hex::encode(computed_hmac.0)
    }

    #[test]
    fn auth_cram_md5() {
        let engine = base64::engine::general_purpose::STANDARD;
        let peer: SocketAddr = "127.0.0.1:12345".parse().unwrap();

        let mut session = Session::new(
            None,
            Some("user".to_string()),
            Some("pass".to_string()),
            peer,
        );
        session.process_line("EHLO localhost").unwrap();

        // Start CRAM-MD5 auth
        let response = session.process_line("AUTH CRAM-MD5").unwrap();
        assert_eq!(response.len(), 1);
        assert!(response[0].starts_with("334 "));

        // Extract challenge from response (base64 encoded)
        let challenge_encoded = &response[0][4..];
        let challenge = String::from_utf8(engine.decode(challenge_encoded).unwrap()).unwrap();

        // Compute HMAC-MD5(challenge, password)
        let hmac_hex = compute_hmac_md5(b"pass", challenge.as_bytes());

        // Create response: username<space>HMAC-MD5-hex
        let response_text = format!("user {}", hmac_hex);
        let response_encoded = engine.encode(response_text.as_bytes());

        // Send response
        let auth_response = session.process_line(&response_encoded).unwrap();
        assert_eq!(auth_response, vec!["235 Authentication successful"]);
        assert!(session.authenticated);

        // Now should be able to send mail
        assert!(
            session
                .process_line("MAIL FROM:<sender@example.com>")
                .is_ok()
        );
    }

    #[test]
    fn auth_cram_md5_rejects_wrong_credentials() {
        let engine = base64::engine::general_purpose::STANDARD;
        let peer: SocketAddr = "127.0.0.1:12345".parse().unwrap();

        let mut session = Session::new(
            None,
            Some("user".to_string()),
            Some("pass".to_string()),
            peer,
        );
        session.process_line("EHLO localhost").unwrap();

        // Start CRAM-MD5 auth
        let response = session.process_line("AUTH CRAM-MD5").unwrap();
        assert_eq!(response.len(), 1);
        assert!(response[0].starts_with("334 "));

        // Extract challenge from response
        let challenge_encoded = &response[0][4..];
        let challenge = String::from_utf8(engine.decode(challenge_encoded).unwrap()).unwrap();

        // Compute HMAC-MD5 with wrong password
        let hmac_hex = compute_hmac_md5(b"wrongpass", challenge.as_bytes());

        // Create response with wrong password
        let response_text = format!("user {}", hmac_hex);
        let response_encoded = engine.encode(response_text.as_bytes());

        // Send response
        let auth_response = session.process_line(&response_encoded).unwrap();
        assert_eq!(auth_response, vec!["535 Authentication failed"]);
        assert!(!session.authenticated);
    }

    #[test]
    fn auth_cram_md5_rejects_wrong_username() {
        let engine = base64::engine::general_purpose::STANDARD;
        let peer: SocketAddr = "127.0.0.1:12345".parse().unwrap();

        let mut session = Session::new(
            None,
            Some("user".to_string()),
            Some("pass".to_string()),
            peer,
        );
        session.process_line("EHLO localhost").unwrap();

        // Start CRAM-MD5 auth
        let response = session.process_line("AUTH CRAM-MD5").unwrap();
        assert_eq!(response.len(), 1);
        assert!(response[0].starts_with("334 "));

        // Extract challenge from response
        let challenge_encoded = &response[0][4..];
        let challenge = String::from_utf8(engine.decode(challenge_encoded).unwrap()).unwrap();

        // Compute HMAC-MD5 with correct password but wrong username
        let hmac_hex = compute_hmac_md5(b"pass", challenge.as_bytes());

        // Create response with wrong username
        let response_text = format!("wronguser {}", hmac_hex);
        let response_encoded = engine.encode(response_text.as_bytes());

        // Send response
        let auth_response = session.process_line(&response_encoded).unwrap();
        assert_eq!(auth_response, vec!["535 Authentication failed"]);
        assert!(!session.authenticated);
    }

    #[test]
    fn auth_cram_md5_accepts_all_when_no_credentials() {
        let engine = base64::engine::general_purpose::STANDARD;
        let peer: SocketAddr = "127.0.0.1:12345".parse().unwrap();

        let mut session = Session::new(None, None, None, peer);
        session.process_line("EHLO localhost").unwrap();

        // Start CRAM-MD5 auth
        let response = session.process_line("AUTH CRAM-MD5").unwrap();
        assert_eq!(response.len(), 1);
        assert!(response[0].starts_with("334 "));

        // Extract challenge from response
        let challenge_encoded = &response[0][4..];
        let challenge = String::from_utf8(engine.decode(challenge_encoded).unwrap()).unwrap();

        // Compute HMAC-MD5 with any password
        let hmac_hex = compute_hmac_md5(b"anypass", challenge.as_bytes());

        // Create response with any credentials
        let response_text = format!("anyuser {}", hmac_hex);
        let response_encoded = engine.encode(response_text.as_bytes());

        // Send response - should be accepted when no credentials configured
        let auth_response = session.process_line(&response_encoded).unwrap();
        assert_eq!(auth_response, vec!["235 Authentication successful"]);
        assert!(session.authenticated);
    }
}
