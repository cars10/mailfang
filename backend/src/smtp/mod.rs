mod parser;
mod server;

pub use parser::EmailAttachment;
pub use server::{Email, Result, SmtpError, SmtpServer};
