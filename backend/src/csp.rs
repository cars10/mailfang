use regex::{Captures, Regex};

/// Injects a Content Security Policy meta tag into an HTML string.
///
/// This CSP blocks all external resources but allows inline styles and data URIs,
/// which is necessary for email rendering while still blocking remote content.
pub fn inject_csp_meta_tag(html: String) -> String {
    // CSP that blocks all external resources but allows inline styles and data URIs
    // This is necessary for email rendering which often uses inline CSS
    // 'self' allows same-origin requests (e.g., CID images served via /api/attachments/{id})
    // while still blocking remote content (http://, https://, etc.)
    const CSP: &str = "default-src 'none'; img-src 'self' data:; script-src 'none'; style-src 'unsafe-inline'; font-src data:; connect-src 'none'; frame-src 'none'; object-src 'none'; media-src data:; base-uri 'none';";
    let csp_meta = format!(
        "<meta http-equiv=\"Content-Security-Policy\" content=\"{}\">",
        CSP
    );

    // Check if HTML already has a head tag (handle attributes)
    let head_regex = Regex::new(r"(?i)<head[^>]*>").unwrap();
    let html_regex = Regex::new(r"(?i)<html[^>]*>").unwrap();

    if head_regex.is_match(&html) {
        // Insert CSP meta tag right after <head> (or <head ...>)
        head_regex
            .replace(&html, |caps: &Captures| format!("{}{}", &caps[0], csp_meta))
            .to_string()
    } else if html_regex.is_match(&html) {
        // Insert head with CSP if html tag exists but no head
        html_regex
            .replace(&html, |caps: &Captures| {
                format!("{}<head>{}</head>", &caps[0], csp_meta)
            })
            .to_string()
    } else {
        // Wrap in html/head if neither exists
        format!(
            "<html><head>{}</head><body>{}</body></html>",
            csp_meta, html
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inject_csp_with_existing_head() {
        let html = "<html><head><title>Test</title></head><body>Content</body></html>";
        let result = inject_csp_meta_tag(html.to_string());

        assert!(result.contains("Content-Security-Policy"));
        assert!(result.contains("<head>"));
        assert!(result.contains("<title>Test</title>"));
        assert!(result.contains("Content"));
    }

    #[test]
    fn test_inject_csp_with_head_attributes() {
        let html = "<html><head lang=\"en\"><title>Test</title></head><body>Content</body></html>";
        let result = inject_csp_meta_tag(html.to_string());

        assert!(result.contains("Content-Security-Policy"));
        assert!(result.contains("lang=\"en\""));
        assert!(result.contains("<title>Test</title>"));
    }

    #[test]
    fn test_inject_csp_with_html_but_no_head() {
        let html = "<html><body>Content</body></html>";
        let result = inject_csp_meta_tag(html.to_string());

        assert!(result.contains("Content-Security-Policy"));
        assert!(result.contains("<head>"));
        assert!(result.contains("Content"));
    }

    #[test]
    fn test_inject_csp_with_html_attributes_but_no_head() {
        let html = "<html lang=\"en\"><body>Content</body></html>";
        let result = inject_csp_meta_tag(html.to_string());

        assert!(result.contains("Content-Security-Policy"));
        assert!(result.contains("lang=\"en\""));
        assert!(result.contains("<head>"));
        assert!(result.contains("Content"));
    }

    #[test]
    fn test_inject_csp_with_fragment() {
        let html = "<div>Just some content</div>";
        let result = inject_csp_meta_tag(html.to_string());

        assert!(result.contains("Content-Security-Policy"));
        assert!(result.starts_with("<html>"));
        assert!(result.contains("<head>"));
        assert!(result.contains("<body>"));
        assert!(result.contains("Just some content"));
        assert!(result.ends_with("</body></html>"));
    }

    #[test]
    fn test_inject_csp_case_insensitive() {
        let html = "<HTML><HEAD><title>Test</title></HEAD><BODY>Content</BODY></HTML>";
        let result = inject_csp_meta_tag(html.to_string());

        assert!(result.contains("Content-Security-Policy"));
        assert!(result.contains("<title>Test</title>"));
    }

    #[test]
    fn test_inject_csp_contains_correct_policy() {
        let html = "<html><head></head><body>Test</body></html>";
        let result = inject_csp_meta_tag(html.to_string());

        assert!(result.contains("default-src 'none'"));
        assert!(result.contains("img-src 'self' data:"));
        assert!(result.contains("script-src 'none'"));
        assert!(result.contains("style-src 'unsafe-inline'"));
        assert!(result.contains("font-src data:"));
        assert!(result.contains("connect-src 'none'"));
        assert!(result.contains("frame-src 'none'"));
        assert!(result.contains("object-src 'none'"));
        assert!(result.contains("media-src data:"));
        assert!(result.contains("base-uri 'none'"));
    }
}
