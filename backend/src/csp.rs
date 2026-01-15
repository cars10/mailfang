pub fn inject_csp_meta_tag(html: String) -> String {
    const CSP: &str = "default-src 'none'; img-src 'self' data:; script-src 'none'; style-src 'unsafe-inline'; font-src data:; connect-src 'none'; frame-src 'none'; object-src 'none'; media-src data:; base-uri 'none';";
    let csp_meta = format!(
        "<meta http-equiv=\"Content-Security-Policy\" content=\"{}\">",
        CSP
    );

    crate::html::insert_into_head(&html, &csp_meta)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inject_csp_contains_correct_policy() {
        let html = "<html><head></head><body>Test</body></html>";
        let result = inject_csp_meta_tag(html.to_string());

        assert!(result.contains("Content-Security-Policy"));
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
