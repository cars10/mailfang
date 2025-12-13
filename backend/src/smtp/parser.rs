use mailparse::{DispositionType, MailAddr, MailHeaderMap, ParsedMail, addrparse, parse_mail};
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
    pub date: Option<String>,
    pub headers: Option<serde_json::Value>,
    pub body_text: String,
    pub body_html: String,
    pub to_addresses: Vec<String>,
}

fn extract_all_headers(headers: &[mailparse::MailHeader<'_>]) -> serde_json::Value {
    let mut header_map: HashMap<String, Vec<String>> = HashMap::new();

    for header in headers {
        let key = header.get_key().to_string();
        let value = header.get_value();

        header_map.entry(key).or_insert_with(Vec::new).push(value);
    }

    serde_json::to_value(header_map).unwrap_or(serde_json::Value::Object(serde_json::Map::new()))
}

fn parse_to_addresses(headers: &[mailparse::MailHeader<'_>]) -> Vec<String> {
    if let Some(to_header) = headers
        .iter()
        .find(|h| h.get_key().eq_ignore_ascii_case("To"))
    {
        let to_value = to_header.get_value();
        match addrparse(&to_value) {
            Ok(addr_list) => {
                addr_list
                    .iter()
                    .filter_map(|addr| {
                        match addr {
                            MailAddr::Single(info) => Some(info.addr.clone()),
                            MailAddr::Group(_) => None, // Groups don't have direct addresses
                        }
                    })
                    .collect()
            }
            Err(_) => Vec::new(),
        }
    } else {
        Vec::new()
    }
}

pub(super) fn parse_email_details(raw: &str) -> ParsedEmailDetails {
    match parse_mail(raw.as_bytes()) {
        Ok(mail) => {
            let mut attachments = Vec::new();
            collect_attachments(&mail, &mut attachments);
            let (body_text, body_html) = extract_bodies(&mail);
            let headers = extract_all_headers(mail.headers.as_slice());
            let to_addresses = parse_to_addresses(mail.headers.as_slice());
            ParsedEmailDetails {
                attachments,
                message_id: mail.headers.get_first_value("Message-ID"),
                subject: mail.headers.get_first_value("Subject"),
                date: mail.headers.get_first_value("Date"),
                headers: Some(headers),
                body_text,
                body_html,
                to_addresses,
            }
        }
        Err(err) => {
            warn!(
                component = "smtp",
                "Failed to parse mail for metadata: {}", err
            );
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

fn collect_attachments(node: &ParsedMail<'_>, out: &mut Vec<EmailAttachment>) {
    let disposition = node.get_content_disposition();
    let is_attachment = matches!(disposition.disposition, DispositionType::Attachment)
        || disposition
            .params
            .iter()
            .any(|(k, _)| k.eq_ignore_ascii_case("filename"));

    // Also collect inline parts with Content-ID (for CID images)
    let has_content_id = node.headers.get_first_value("Content-ID").is_some();
    let is_inline = matches!(disposition.disposition, DispositionType::Inline);

    if is_attachment || (is_inline && has_content_id) {
        if let Ok(body) = node.get_body_raw() {
            let filename = disposition
                .params
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("filename"))
                .map(|(_, v)| v.clone());

            // Extract Content-Description for filename fallback
            let content_description = node.headers.get_first_value("Content-Description");

            // Extract Content-ID, removing angle brackets if present
            let content_id = node.headers.get_first_value("Content-ID").map(|cid| {
                // Remove angle brackets if present: <cid> -> cid
                cid.trim_start_matches('<')
                    .trim_end_matches('>')
                    .to_string()
            });

            // Extract all headers for this attachment
            let headers = extract_all_headers(node.headers.as_slice());

            out.push(EmailAttachment {
                filename: filename.or_else(|| content_description.clone()),
                mime_type: node.ctype.mimetype.clone(),
                data: body,
                content_id,
                headers: Some(headers),
            });
        }
    }

    for child in &node.subparts {
        collect_attachments(child, out);
    }
}

fn extract_bodies(mail: &ParsedMail<'_>) -> (String, String) {
    let mut text_body: Option<String> = None;
    let mut html_body: Option<String> = None;

    extract_body_recursive(mail, &mut text_body, &mut html_body);

    (
        text_body.unwrap_or_else(|| String::new()),
        html_body.unwrap_or_else(|| String::new()),
    )
}

fn extract_body_recursive(
    node: &ParsedMail<'_>,
    text_body: &mut Option<String>,
    html_body: &mut Option<String>,
) {
    let disposition = node.get_content_disposition();
    let is_attachment = matches!(disposition.disposition, DispositionType::Attachment)
        || disposition
            .params
            .iter()
            .any(|(k, _)| k.eq_ignore_ascii_case("filename"));

    // Skip attachments
    if !is_attachment {
        let mimetype = &node.ctype.mimetype;
        if mimetype == "text/plain" && text_body.is_none() {
            if let Ok(body) = node.get_body() {
                *text_body = Some(body);
            }
        } else if mimetype == "text/html" && html_body.is_none() {
            if let Ok(body) = node.get_body() {
                *html_body = Some(body);
            }
        }
    }

    for child in &node.subparts {
        extract_body_recursive(child, text_body, html_body);
    }
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
        assert_eq!(details.message_id.as_deref(), Some("<1234@example.com>"));
        assert!(details.attachments.is_empty());
    }
}
