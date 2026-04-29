//! MIME type utilities for TypeSpec-Rust
//!
//! Ported from TypeSpec compiler/src/core/mime-type.ts
//!
//! This module provides MIME type parsing utilities.

/// MIME type structure
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MimeType {
    /// The type (e.g., "application", "text", "image", etc.)
    pub mime_type: String,
    /// The subtype (e.g., "json", "plain", "png", etc.)
    pub subtype: String,
    /// Optional suffix (e.g., "xml" in "application/vnd.api+xml")
    pub suffix: Option<String>,
}

/// Valid top-level MIME type names
const VALID_TYPES: &[&str] = &[
    "application",
    "audio",
    "font",
    "image",
    "message",
    "model",
    "multipart",
    "text",
    "video",
    "example",
];

/// Check if a character is valid in a MIME subtype
fn is_valid_subtype_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || "!#$%&'*+.^_`|~-".contains(c)
}

/// Check if a character is valid in a MIME type
fn is_valid_type_char(c: char) -> bool {
    c.is_ascii_lowercase() || c == 'x' || c == '-'
}

/// Parse a MIME type string into its components
///
/// # Arguments
/// * `mime_type_str` - A MIME type string (e.g., "application/json", "text/plain; charset=utf-8")
///
/// # Returns
/// * `Some(MimeType)` if parsing succeeds, `None` otherwise
pub fn parse_mime_type(mime_type_str: &str) -> Option<MimeType> {
    let s = mime_type_str.trim();

    // Must contain a slash
    let slash_pos = s.find('/')?;
    let (mime_type_part, subtype_part) = s.split_at(slash_pos);
    let subtype_part = &subtype_part[1..]; // Skip the slash

    // Validate and parse type
    let mime_type = mime_type_part.to_lowercase();
    if mime_type.is_empty() || !mime_type.chars().all(is_valid_type_char) {
        return None;
    }

    // Check if it's a known type or x- prefix
    let valid = VALID_TYPES.contains(&mime_type.as_str()) || mime_type.starts_with("x-");
    if !valid {
        return None;
    }

    // Validate subtype (before any parameters)
    let semicolon_pos = subtype_part.find(';');
    let subtype_raw = if let Some(pos) = semicolon_pos {
        &subtype_part[..pos]
    } else {
        subtype_part
    };

    if subtype_raw.is_empty() || !subtype_raw.chars().all(is_valid_subtype_char) {
        return None;
    }

    // Parse subtype for suffix
    let (subtype, suffix) = parse_sub_type(subtype_raw);

    Some(MimeType {
        mime_type,
        subtype,
        suffix,
    })
}

/// Parse the subtype to extract suffix if present
fn parse_sub_type(value: &str) -> (String, Option<String>) {
    let mut parts = value.splitn(2, '+');
    let subtype = parts.next().unwrap_or(value).to_string();
    let suffix = parts.next().map(|s| s.to_string());
    (subtype, suffix)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_mime_type() {
        let result = parse_mime_type("application/json").unwrap();
        assert_eq!(result.mime_type, "application");
        assert_eq!(result.subtype, "json");
        assert_eq!(result.suffix, None);
    }

    #[test]
    fn test_parse_text_plain() {
        let result = parse_mime_type("text/plain").unwrap();
        assert_eq!(result.mime_type, "text");
        assert_eq!(result.subtype, "plain");
        assert_eq!(result.suffix, None);
    }

    #[test]
    fn test_parse_image_png() {
        let result = parse_mime_type("image/png").unwrap();
        assert_eq!(result.mime_type, "image");
        assert_eq!(result.subtype, "png");
        assert_eq!(result.suffix, None);
    }

    #[test]
    fn test_parse_mime_type_with_suffix() {
        let result = parse_mime_type("application/vnd.api+json").unwrap();
        assert_eq!(result.mime_type, "application");
        assert_eq!(result.subtype, "vnd.api");
        assert_eq!(result.suffix, Some("json".to_string()));
    }

    #[test]
    fn test_parse_mime_type_with_charset() {
        // Note: charset is not parsed by this function
        let result = parse_mime_type("text/plain; charset=utf-8").unwrap();
        assert_eq!(result.mime_type, "text");
        // Subtype is truncated at semicolon in our implementation
        assert_eq!(result.subtype, "plain");
        assert_eq!(result.suffix, None);
    }

    #[test]
    fn test_parse_invalid_mime_type() {
        assert!(parse_mime_type("invalid").is_none());
        assert!(parse_mime_type("").is_none());
        assert!(parse_mime_type("text/").is_none());
        assert!(parse_mime_type("/json").is_none());
    }

    #[test]
    fn test_parse_custom_mime_type() {
        let result = parse_mime_type("application/x-custom-type").unwrap();
        assert_eq!(result.mime_type, "application");
        assert_eq!(result.subtype, "x-custom-type");
        assert_eq!(result.suffix, None);
    }

    #[test]
    fn test_parse_mime_type_with_plus_suffix() {
        let result = parse_mime_type("application/soap+xml").unwrap();
        assert_eq!(result.mime_type, "application");
        assert_eq!(result.subtype, "soap");
        assert_eq!(result.suffix, Some("xml".to_string()));
    }

    #[test]
    fn test_parse_case_insensitive_type() {
        // Type is lowercased
        let result = parse_mime_type("APPLICATION/JSON").unwrap();
        assert_eq!(result.mime_type, "application");
        // Subtype preserves original case
        assert_eq!(result.subtype, "JSON");
    }
}
