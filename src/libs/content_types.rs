//! HTTP Content Type Utilities
//!
//! Ported from TypeSpec packages/http/src/content-types.ts
//!
//! Provides content type resolution from model properties.

/// Resolve content types from a type representation.
///
/// In the TS implementation, this resolves content types from a ModelProperty
/// by examining its type. In this Rust port, we provide the core logic
/// without the Type-system dependency.
///
/// # Arguments
/// * `type_kind` - The kind of type ("string", "union", "scalar", etc.)
/// * `string_value` - For string types, the literal value
/// * `union_values` - For union types, the variant values (each is (kind, value))
///
/// # Returns
/// A tuple of (content_types, error_message)
pub fn resolve_content_types(
    type_kind: &str,
    string_value: Option<&str>,
    union_values: Option<&[(String, Option<String>)]>,
) -> (Vec<String>, Option<String>) {
    match type_kind {
        "String" => {
            if let Some(value) = string_value {
                (vec![value.to_string()], None)
            } else {
                (vec![], Some("String type has no value".to_string()))
            }
        }
        "Union" => {
            if let Some(variants) = union_values {
                let mut content_types = Vec::new();
                for (kind, value) in variants {
                    if kind == "String" {
                        if let Some(v) = value {
                            content_types.push(v.clone());
                        }
                    } else {
                        return (
                            vec![],
                            Some(
                                "Union variant for content type must be a string literal"
                                    .to_string(),
                            ),
                        );
                    }
                }
                (content_types, None)
            } else {
                (vec![], Some("Union type has no variants".to_string()))
            }
        }
        "Scalar" => {
            // Scalar types map to wildcard content type
            (vec!["*/*".to_string()], None)
        }
        _ => (
            vec![],
            Some(
                "Content type property must be a string, union of strings, or the string scalar"
                    .to_string(),
            ),
        ),
    }
}

/// Check if a content type indicates JSON
pub fn is_json_content_type(content_type: &str) -> bool {
    let ct = content_type.to_lowercase();
    if ct == "application/json" {
        return true;
    }
    if ct.starts_with("application/") && ct.ends_with("+json") {
        // Must have a subtype before the +json suffix (e.g., "vnd.api+json")
        let subtype = &ct["application/".len()..ct.len() - "+json".len()];
        return !subtype.is_empty();
    }
    false
}

/// Check if a content type indicates XML
pub fn is_xml_content_type(content_type: &str) -> bool {
    let ct = content_type.to_lowercase();
    if ct == "application/xml" || ct == "text/xml" {
        return true;
    }
    if ct.starts_with("application/") && ct.ends_with("+xml") {
        // Must have a subtype before the +xml suffix (e.g., "soap+xml")
        let subtype = &ct["application/".len()..ct.len() - "+xml".len()];
        return !subtype.is_empty();
    }
    false
}

/// Check if a content type indicates multipart form data
pub fn is_multipart_content_type(content_type: &str) -> bool {
    content_type.to_lowercase().starts_with("multipart/")
}

/// Check if a content type indicates SSE
pub fn is_sse_content_type(content_type: &str) -> bool {
    content_type.to_lowercase() == "text/event-stream"
}

/// Check if a content type indicates plain text
pub fn is_plain_text_content_type(content_type: &str) -> bool {
    content_type.to_lowercase().starts_with("text/plain")
}

/// Check if a content type indicates octet stream
pub fn is_octet_stream_content_type(content_type: &str) -> bool {
    content_type.to_lowercase() == "application/octet-stream"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_string_content_type() {
        let (cts, err) = resolve_content_types("String", Some("application/json"), None);
        assert_eq!(cts, vec!["application/json"]);
        assert!(err.is_none());
    }

    #[test]
    fn test_resolve_union_content_types() {
        let variants = vec![
            ("String".to_string(), Some("application/json".to_string())),
            ("String".to_string(), Some("application/xml".to_string())),
        ];
        let (cts, err) = resolve_content_types("Union", None, Some(&variants));
        assert_eq!(cts, vec!["application/json", "application/xml"]);
        assert!(err.is_none());
    }

    #[test]
    fn test_resolve_scalar_string_content_type() {
        let (cts, err) = resolve_content_types("Scalar", Some("string"), None);
        assert_eq!(cts, vec!["*/*"]);
        assert!(err.is_none());
    }

    #[test]
    fn test_resolve_invalid_content_type() {
        let (cts, err) = resolve_content_types("Model", None, None);
        assert!(cts.is_empty());
        assert!(err.is_some());
    }

    #[test]
    fn test_is_json_content_type() {
        assert!(is_json_content_type("application/json"));
        assert!(is_json_content_type("application/merge-patch+json"));
        assert!(is_json_content_type("application/vnd.api+json"));
        assert!(!is_json_content_type("application/xml"));
    }

    #[test]
    fn test_is_xml_content_type() {
        assert!(is_xml_content_type("application/xml"));
        assert!(is_xml_content_type("text/xml"));
        assert!(is_xml_content_type("application/soap+xml"));
        assert!(!is_xml_content_type("application/json"));
    }

    #[test]
    fn test_is_multipart_content_type() {
        assert!(is_multipart_content_type("multipart/form-data"));
        assert!(is_multipart_content_type("multipart/mixed"));
        assert!(!is_multipart_content_type("application/json"));
    }

    #[test]
    fn test_is_sse_content_type() {
        assert!(is_sse_content_type("text/event-stream"));
        assert!(!is_sse_content_type("text/plain"));
    }

    #[test]
    fn test_is_plain_text_content_type() {
        assert!(is_plain_text_content_type("text/plain"));
        assert!(is_plain_text_content_type("text/plain; charset=utf-8"));
        assert!(!is_plain_text_content_type("text/html"));
    }

    #[test]
    fn test_is_octet_stream_content_type() {
        assert!(is_octet_stream_content_type("application/octet-stream"));
        assert!(!is_octet_stream_content_type("application/json"));
    }

    #[test]
    fn test_is_json_content_type_edge_cases() {
        // Not JSON: no subtype before +json
        assert!(!is_json_content_type("application/+json"));
        // Not JSON: text type
        assert!(!is_json_content_type("text/json"));
        // Not JSON: completely different
        assert!(!is_json_content_type("multipart/form-data"));
        // JSON: case insensitive
        assert!(is_json_content_type("Application/JSON"));
    }

    #[test]
    fn test_is_xml_content_type_edge_cases() {
        // Not XML: no subtype before +xml
        assert!(!is_xml_content_type("application/+xml"));
        // XML: case insensitive
        assert!(is_xml_content_type("Application/XML"));
    }

    #[test]
    fn test_resolve_content_types_union_with_non_string() {
        let variants = vec![
            ("String".to_string(), Some("application/json".to_string())),
            ("Model".to_string(), None),
        ];
        let (cts, err) = resolve_content_types("Union", None, Some(&variants));
        assert!(cts.is_empty());
        assert!(err.is_some());
    }

    #[test]
    fn test_resolve_content_types_scalar_wildcard() {
        let (cts, err) = resolve_content_types("Scalar", None, None);
        assert_eq!(cts, vec!["*/*"]);
        assert!(err.is_none());
    }
}
