use regex::{Captures, Regex};

pub fn normalize_html_document(html: &str) -> String {
    let lower = html.to_lowercase();
    let start_idx = lower.find("<!doctype html").or_else(|| lower.find("<html"));

    let Some(start_idx) = start_idx else {
        return html.to_string();
    };

    let trimmed_start = &html[start_idx..];
    let trimmed_start_lower = &lower[start_idx..];

    if let Some(end_idx) = trimmed_start_lower.rfind("</html>") {
        let html_end = end_idx + "</html>".len();
        trimmed_start[..html_end].to_string()
    } else {
        trimmed_start.to_string()
    }
}

pub fn insert_into_head(html: &str, content: &str) -> String {
    let head_regex = Regex::new(r"(?i)<head[^>]*>").expect("valid head regex");
    let html_regex = Regex::new(r"(?i)<html[^>]*>").expect("valid html regex");

    if head_regex.is_match(html) {
        head_regex
            .replace(html, |caps: &Captures| format!("{}{}", &caps[0], content))
            .to_string()
    } else if html_regex.is_match(html) {
        html_regex
            .replace(html, |caps: &Captures| {
                format!("{}<head>{}</head>", &caps[0], content)
            })
            .to_string()
    } else {
        format!("<html><head>{}</head><body>{}</body></html>", content, html)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_into_head_with_existing_head() {
        let html = "<html><head><title>Test</title></head><body>Content</body></html>";
        let result = insert_into_head(html, "<meta test>");

        assert!(result.contains("<meta test>"));
        assert!(result.contains("<head>"));
        assert!(result.contains("<title>Test</title>"));
        assert!(result.contains("Content"));
        assert!(result.contains("<head><meta test>"));
    }

    #[test]
    fn test_insert_into_head_with_head_attributes() {
        let html = "<html><head lang=\"en\"><title>Test</title></head><body>Content</body></html>";
        let result = insert_into_head(html, "<base target=\"_blank\">");

        assert!(result.contains("<base target=\"_blank\">"));
        assert!(result.contains("lang=\"en\""));
        assert!(result.contains("<title>Test</title>"));
        assert!(result.contains("<head lang=\"en\"><base target=\"_blank\">"));
    }

    #[test]
    fn test_insert_into_head_with_html_but_no_head() {
        let html = "<html><body>Content</body></html>";
        let result = insert_into_head(html, "<meta test>");

        assert!(result.contains("<meta test>"));
        assert!(result.contains("<head>"));
        assert!(result.contains("Content"));
        assert!(result.contains("<html><head><meta test></head>"));
    }

    #[test]
    fn test_insert_into_head_with_html_attributes_but_no_head() {
        let html = "<html lang=\"en\"><body>Content</body></html>";
        let result = insert_into_head(html, "<base target=\"_blank\">");

        assert!(result.contains("<base target=\"_blank\">"));
        assert!(result.contains("lang=\"en\""));
        assert!(result.contains("<head>"));
        assert!(result.contains("Content"));
        assert!(result.contains("<html lang=\"en\"><head><base target=\"_blank\"></head>"));
    }

    #[test]
    fn test_insert_into_head_with_fragment() {
        let html = "<div>Just some content</div>";
        let result = insert_into_head(html, "<meta test>");

        assert!(result.contains("<meta test>"));
        assert!(result.starts_with("<html>"));
        assert!(result.contains("<head>"));
        assert!(result.contains("<body>"));
        assert!(result.contains("Just some content"));
        assert!(result.ends_with("</body></html>"));
        assert_eq!(
            result,
            "<html><head><meta test></head><body><div>Just some content</div></body></html>"
        );
    }

    #[test]
    fn test_insert_into_head_case_insensitive() {
        let html = "<HTML><HEAD><title>Test</title></HEAD><BODY>Content</BODY></HTML>";
        let result = insert_into_head(html, "<meta test>");

        assert!(result.contains("<meta test>"));
        assert!(result.contains("<title>Test</title>"));
    }

    #[test]
    fn test_normalize_html_document_strips_wrappers() {
        let html = "<p><div>tracking</div><!doctype html><html><head><meta name=\"x\"></head><body>Hi</body></html></p>";
        let result = normalize_html_document(html);

        assert!(result.starts_with("<!doctype html>"));
        assert!(result.ends_with("</html>"));
        assert!(!result.starts_with("<p>"));
        assert!(!result.ends_with("</p>"));
        assert!(result.contains("<head><meta name=\"x\"></head>"));
    }

    #[test]
    fn test_normalize_html_document_returns_original_without_html_tag() {
        let html = "<div>fragment</div>";
        let result = normalize_html_document(html);
        assert_eq!(result, html);
    }
}
