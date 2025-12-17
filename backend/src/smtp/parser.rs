use chrono::{DateTime, Utc};
use mail_parser::{MessageParser, MimeHeaders, PartType};
use std::collections::HashMap;
use tracing::warn;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmailAttachment {
    pub filename: Option<String>,
    pub mime_type: String,
    pub data: Vec<u8>,
    pub content_id: Option<String>,
    pub headers: Option<serde_json::Value>,
}

pub(super) struct ParsedEmailDetails {
    pub attachments: Vec<EmailAttachment>,
    pub message_id: Option<String>,
    pub subject: Option<String>,
    pub date: Option<DateTime<Utc>>,
    pub headers: Option<serde_json::Value>,
    pub body_text: String,
    pub body_html: String,
    pub to_addresses: Vec<String>,
}

pub(super) fn parse_email_details(raw: &str) -> ParsedEmailDetails {
    let parser = MessageParser::default();
    match parser.parse(raw.as_bytes()) {
        Some(message) => {
            // Extract all headers as JSON
            let headers = extract_all_headers(&message);

            // Extract To addresses
            let to_addresses = extract_to_addresses(&message);

            // Extract body text and HTML
            // Note: mail-parser auto-generates HTML from text if no HTML part exists
            // We only want actual HTML content, so check if there's a real text/html part
            let body_text = message
                .body_text(0)
                .map(|s| s.to_string())
                .unwrap_or_default();
            let body_html = message
                .html_part(0)
                .filter(|part| {
                    // Only use if it's actually text/html content type
                    part.content_type()
                        .map(|ct| ct.ctype() == "text" && ct.subtype() == Some("html"))
                        .unwrap_or(false)
                })
                .and_then(|_| message.body_html(0))
                .map(|s| s.to_string())
                .unwrap_or_default();

            // Extract attachments
            let attachments = collect_attachments(&message);

            // Convert mail-parser DateTime to chrono DateTime<Utc>
            let date = message
                .date()
                .and_then(|dt| DateTime::from_timestamp(dt.to_timestamp(), 0));

            ParsedEmailDetails {
                attachments,
                message_id: message.message_id().map(|s| s.to_string()),
                subject: message.subject().map(|s| s.to_string()),
                date,
                headers: Some(headers),
                body_text,
                body_html,
                to_addresses,
            }
        }
        None => {
            warn!(component = "smtp", "Failed to parse mail for metadata");
            ParsedEmailDetails {
                attachments: Vec::new(),
                message_id: None,
                subject: None,
                date: None,
                headers: None,
                body_text: String::new(),
                body_html: String::new(),
                to_addresses: Vec::new(),
            }
        }
    }
}

fn extract_all_headers(message: &mail_parser::Message<'_>) -> serde_json::Value {
    let mut header_map: HashMap<String, Vec<String>> = HashMap::new();

    for header in message.headers() {
        let key = header.name().to_string();
        let value = header.value().as_text().unwrap_or_default().to_string();

        header_map.entry(key).or_default().push(value);
    }

    serde_json::to_value(header_map).unwrap_or(serde_json::Value::Object(serde_json::Map::new()))
}

fn extract_to_addresses(message: &mail_parser::Message<'_>) -> Vec<String> {
    message
        .to()
        .map(|addr| {
            addr.iter()
                .filter_map(|a| a.address.as_ref().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

fn collect_attachments(message: &mail_parser::Message<'_>) -> Vec<EmailAttachment> {
    let mut attachments = Vec::new();

    for attachment in message.attachments() {
        // Get filename from Content-Disposition or Content-Type name parameter
        let filename = attachment.attachment_name().map(|s| s.to_string());

        // Get Content-ID, removing angle brackets if present
        let content_id = attachment.content_id().map(|cid| {
            cid.trim_start_matches('<')
                .trim_end_matches('>')
                .to_string()
        });

        // Get MIME type
        let mime_type = attachment
            .content_type()
            .map(|ct| {
                if let Some(subtype) = ct.subtype() {
                    format!("{}/{}", ct.ctype(), subtype)
                } else {
                    ct.ctype().to_string()
                }
            })
            .unwrap_or_else(|| "application/octet-stream".to_string());

        // Get attachment body data
        let data = match &attachment.body {
            PartType::Binary(data) | PartType::InlineBinary(data) => data.to_vec(),
            PartType::Text(text) => text.as_bytes().to_vec(),
            PartType::Html(html) => html.as_bytes().to_vec(),
            PartType::Message(msg) => msg.raw_message().to_vec(),
            PartType::Multipart(_) => Vec::new(),
        };

        // Extract headers for this attachment part
        let headers = extract_part_headers(attachment);

        attachments.push(EmailAttachment {
            filename,
            mime_type,
            data,
            content_id,
            headers: Some(headers),
        });
    }

    attachments
}

fn extract_part_headers(part: &mail_parser::MessagePart<'_>) -> serde_json::Value {
    let mut header_map: HashMap<String, Vec<String>> = HashMap::new();

    for header in part.headers() {
        let key = header.name().to_string();
        let value = header.value().as_text().unwrap_or_default().to_string();

        header_map.entry(key).or_default().push(value);
    }

    serde_json::to_value(header_map).unwrap_or(serde_json::Value::Object(serde_json::Map::new()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_multipart_attachment() {
        let raw = "\
Content-Type: multipart/mixed; boundary=\"BOUNDARY\"\r\n\
\r\n\
--BOUNDARY\r\n\
Content-Type: text/plain\r\n\
\r\n\
Hello\r\n\
--BOUNDARY\r\n\
Content-Type: text/plain\r\n\
Content-Disposition: attachment; filename=\"note.txt\"\r\n\
\r\n\
Attachment body\r\n\
--BOUNDARY--\r\n";
        let details = parse_email_details(raw);
        assert_eq!(details.attachments.len(), 1);
        let attachment = &details.attachments[0];
        assert_eq!(attachment.filename.as_deref(), Some("note.txt"));
        assert_eq!(attachment.mime_type, "text/plain");
        assert_eq!(attachment.data, b"Attachment body");
        assert!(details.message_id.is_none());
    }

    #[test]
    fn extracts_message_id_header() {
        let raw = "\
Message-ID: <1234@example.com>\r\n\
Content-Type: text/plain\r\n\
\r\n\
Body\r\n";
        let details = parse_email_details(raw);
        // mail-parser returns message-id without angle brackets
        assert_eq!(details.message_id.as_deref(), Some("1234@example.com"));
        assert!(details.attachments.is_empty());
    }
}
