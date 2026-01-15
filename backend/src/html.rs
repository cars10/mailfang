use regex::{Captures, Regex};

pub fn insert_into_head(html: &str, content: &str) -> String {
    let head_regex = Regex::new(r"(?i)<head[^>]*>").unwrap();
    let html_regex = Regex::new(r"(?i)<html[^>]*>").unwrap();

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
}
