use regex::Regex;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SearchField {
    Subject,
    From,
    Recipient,
    To,
    Text,
    Html,
    Attachment,
}

impl SearchField {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "subject" => Some(SearchField::Subject),
            "from" => Some(SearchField::From),
            "recipient" => Some(SearchField::Recipient),
            "to" => Some(SearchField::To),
            "text" => Some(SearchField::Text),
            "html" => Some(SearchField::Html),
            "attachment" => Some(SearchField::Attachment),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchTerm {
    pub field: SearchField,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedSearchQuery {
    pub field_terms: Vec<SearchTerm>,
    pub default_terms: Vec<String>,
}

pub fn parse_search_query(query: &str) -> ParsedSearchQuery {
    if query.trim().is_empty() {
        return ParsedSearchQuery {
            field_terms: Vec::new(),
            default_terms: Vec::new(),
        };
    }

    let field_pattern = Regex::new(r"(\w+):(\S+)").unwrap();
    let mut field_terms = Vec::new();
    let mut matched_ranges = Vec::new();

    for cap in field_pattern.captures_iter(query) {
        let field_name = cap.get(1).unwrap().as_str();
        let value = cap.get(2).unwrap().as_str();
        let full_match = cap.get(0).unwrap();

        if let Some(field) = SearchField::from_str(field_name) {
            let normalized_field = match field {
                SearchField::To => SearchField::Recipient,
                _ => field,
            };
            field_terms.push(SearchTerm {
                field: normalized_field,
                value: value.to_string(),
            });
            matched_ranges.push(full_match.range());
        }
    }

    let mut default_terms = Vec::new();
    let mut last_end = 0;

    for range in matched_ranges {
        if range.start > last_end {
            let text_between = &query[last_end..range.start];
            default_terms.extend(
                text_between
                    .split_whitespace()
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string()),
            );
        }
        last_end = range.end;
    }

    if last_end < query.len() {
        let remaining = &query[last_end..];
        default_terms.extend(
            remaining
                .split_whitespace()
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string()),
        );
    }

    ParsedSearchQuery {
        field_terms,
        default_terms,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_field_parsing() {
        let result = parse_search_query("subject:foo");
        assert_eq!(result.field_terms.len(), 1);
        assert_eq!(result.field_terms[0].field, SearchField::Subject);
        assert_eq!(result.field_terms[0].value, "foo");
        assert_eq!(result.default_terms.len(), 0);
    }

    #[test]
    fn test_multiple_fields() {
        let result = parse_search_query("subject:paypal recipient:john");
        assert_eq!(result.field_terms.len(), 2);
        assert_eq!(result.field_terms[0].field, SearchField::Subject);
        assert_eq!(result.field_terms[0].value, "paypal");
        assert_eq!(result.field_terms[1].field, SearchField::Recipient);
        assert_eq!(result.field_terms[1].value, "john");
    }

    #[test]
    fn test_default_search() {
        let result = parse_search_query("simple search");
        assert_eq!(result.field_terms.len(), 0);
        assert_eq!(result.default_terms.len(), 2);
        assert_eq!(result.default_terms[0], "simple");
        assert_eq!(result.default_terms[1], "search");
    }

    #[test]
    fn test_mixed_search() {
        let result = parse_search_query("subject:paypal test");
        assert_eq!(result.field_terms.len(), 1);
        assert_eq!(result.field_terms[0].field, SearchField::Subject);
        assert_eq!(result.field_terms[0].value, "paypal");
        assert_eq!(result.default_terms.len(), 1);
        assert_eq!(result.default_terms[0], "test");
    }

    #[test]
    fn test_mixed_search_with_from() {
        let result = parse_search_query("from:paypal birthdayparty");
        assert_eq!(result.field_terms.len(), 1);
        assert_eq!(result.field_terms[0].field, SearchField::From);
        assert_eq!(result.field_terms[0].value, "paypal");
        assert_eq!(result.default_terms.len(), 1);
        assert_eq!(result.default_terms[0], "birthdayparty");
    }

    #[test]
    fn test_field_aliases() {
        let result1 = parse_search_query("to:john");
        let result2 = parse_search_query("recipient:john");
        assert_eq!(result1.field_terms[0].field, SearchField::Recipient);
        assert_eq!(result2.field_terms[0].field, SearchField::Recipient);
    }

    #[test]
    fn test_empty_string() {
        let result = parse_search_query("");
        assert_eq!(result.field_terms.len(), 0);
        assert_eq!(result.default_terms.len(), 0);
    }

    #[test]
    fn test_whitespace_only() {
        let result = parse_search_query("   ");
        assert_eq!(result.field_terms.len(), 0);
        assert_eq!(result.default_terms.len(), 0);
    }

    #[test]
    fn test_malformed_field() {
        let result = parse_search_query("invalid:field test");
        assert_eq!(result.field_terms.len(), 0);
        assert_eq!(result.default_terms.len(), 2);
    }

    #[test]
    fn test_multiple_default_terms() {
        let result = parse_search_query("test search query");
        assert_eq!(result.default_terms.len(), 3);
        assert_eq!(result.default_terms[0], "test");
        assert_eq!(result.default_terms[1], "search");
        assert_eq!(result.default_terms[2], "query");
    }

    #[test]
    fn test_all_field_types() {
        let result = parse_search_query(
            "subject:test from:sender recipient:recv text:body html:content attachment:file",
        );
        assert_eq!(result.field_terms.len(), 6);
        assert_eq!(result.field_terms[0].field, SearchField::Subject);
        assert_eq!(result.field_terms[1].field, SearchField::From);
        assert_eq!(result.field_terms[2].field, SearchField::Recipient);
        assert_eq!(result.field_terms[3].field, SearchField::Text);
        assert_eq!(result.field_terms[4].field, SearchField::Html);
        assert_eq!(result.field_terms[5].field, SearchField::Attachment);
    }

    #[test]
    fn test_complex_mixed() {
        let result = parse_search_query("from:paypal subject:invoice payment");
        assert_eq!(result.field_terms.len(), 2);
        assert_eq!(result.field_terms[0].field, SearchField::From);
        assert_eq!(result.field_terms[1].field, SearchField::Subject);
        assert_eq!(result.default_terms.len(), 1);
        assert_eq!(result.default_terms[0], "payment");
    }

    #[test]
    fn test_value_with_underscore() {
        let result = parse_search_query("subject:test_value");
        assert_eq!(result.field_terms.len(), 1);
        assert_eq!(result.field_terms[0].value, "test_value");
    }

    #[test]
    fn test_value_with_dash() {
        let result = parse_search_query("subject:test-value");
        assert_eq!(result.field_terms.len(), 1);
        assert_eq!(result.field_terms[0].value, "test-value");
    }
}
