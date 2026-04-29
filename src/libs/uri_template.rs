//! URI Template Parser
//!
//! Ported from TypeSpec packages/http/src/uri-template.ts
//!
//! Parses URI templates according to [RFC 6570](https://datatracker.ietf.org/doc/html/rfc6570).
//!
//! Supports:
//! - Simple string expansion: `{var}`
//! - Reserved expansion: `{+var}`
//! - Fragment expansion: `{#var}`
//! - Label expansion: `{.var}`
//! - Path segment expansion: `{/var}`
//! - Path-style parameter expansion: `{;var}`
//! - Form-style query expansion: `{?var}`
//! - Form-style query continuation: `{&var}`
//! - Modifiers: explode (`{*`) and prefix (`{:n}`)

/// URI template operators per RFC 6570 Section 3.2
const OPERATORS: &[&str] = &["+", "#", ".", "/", ";", "?", "&"];

/// Modifier for a URI template parameter
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UriTemplateModifier {
    /// Explode modifier (trailing asterisk)
    Explode,
    /// Prefix modifier (trailing colon + max length)
    Prefix(usize),
}

/// A single URI template parameter
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UriTemplateParameter {
    /// Parameter name
    pub name: String,
    /// Optional operator (+, #, ., /, ;, ?, &)
    pub operator: Option<String>,
    /// Optional modifier (explode or prefix)
    pub modifier: Option<UriTemplateModifier>,
}

/// Parsed URI template
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UriTemplate {
    /// Segments of the template (either literal strings or parameters)
    pub segments: Vec<UriTemplateSegment>,
    /// All parameters extracted from the template
    pub parameters: Vec<UriTemplateParameter>,
}

/// A segment of a URI template
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UriTemplateSegment {
    /// Literal string segment
    Literal(String),
    /// Parameter segment
    Parameter(UriTemplateParameter),
}

/// Regex for matching URI template expressions
const URI_TEMPLATE_REGEX: &str = r"\{([^{}]+)\}|([^{}]+)";

/// Regex for matching parameter name and modifier within an expression
const EXPRESSION_REGEX: &str = r"^([^:*]*)(?::(\d+)|(\*))?$";

/// Lazily compiled regex for URI template expressions
static URI_TEMPLATE_RE: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    regex::Regex::new(URI_TEMPLATE_REGEX).expect("valid URI template regex")
});

/// Lazily compiled regex for parameter name/modifier parsing
static EXPRESSION_RE: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    regex::Regex::new(EXPRESSION_REGEX).expect("valid expression regex")
});

/// Lazily compiled regex for extracting params from path
static PARAMS_FROM_PATH_RE: std::sync::LazyLock<regex::Regex> =
    std::sync::LazyLock::new(|| regex::Regex::new(r"\{[^}]+\}").expect("valid params regex"));

/// Parse a URI template string according to RFC 6570
///
/// # Arguments
/// * `template` - A URI template string (e.g., "/pets/{id}", "/search{?q,limit}")
///
/// # Returns
/// A parsed `UriTemplate` with segments and parameters
pub fn parse_uri_template(template: &str) -> UriTemplate {
    let mut parameters = Vec::new();
    let mut segments = Vec::new();

    for cap in URI_TEMPLATE_RE.captures_iter(template) {
        if let Some(expression) = cap.get(1) {
            let mut expression = expression.as_str();

            // Check for operator
            let operator = if OPERATORS.contains(&&expression[..1]) {
                let op = expression[..1].to_string();
                expression = &expression[1..];
                Some(op)
            } else {
                None
            };

            // Split by comma for multiple items
            for item in expression.split(',') {
                if let Some(expr_cap) = EXPRESSION_RE.captures(item) {
                    let name = expr_cap.get(1).map(|m| m.as_str().trim()).unwrap_or("");
                    if name.is_empty() {
                        continue;
                    }

                    let modifier = if expr_cap.get(3).is_some() {
                        Some(UriTemplateModifier::Explode)
                    } else if let Some(prefix_val) = expr_cap.get(2) {
                        prefix_val
                            .as_str()
                            .parse::<usize>()
                            .ok()
                            .map(UriTemplateModifier::Prefix)
                    } else {
                        None
                    };

                    let parameter = UriTemplateParameter {
                        name: name.to_string(),
                        operator: operator.clone(),
                        modifier,
                    };
                    parameters.push(parameter.clone());
                    segments.push(UriTemplateSegment::Parameter(parameter));
                }
            }
        } else if let Some(literal) = cap.get(2) {
            let literal_str = literal.as_str().to_string();
            if !literal_str.is_empty() {
                segments.push(UriTemplateSegment::Literal(literal_str));
            }
        }
    }

    UriTemplate {
        segments,
        parameters,
    }
}

/// Extract parameter names (wrapped in `{` and `}`) from a path/URL string.
/// Ported from TS `extractParamsFromPath()`.
///
/// # Example
/// `"foo/{name}/bar"` -> `["name"]`
pub fn extract_params_from_path(path: &str) -> Vec<String> {
    PARAMS_FROM_PATH_RE
        .find_iter(path)
        .map(|m| m.as_str()[1..m.as_str().len() - 1].to_string())
        .collect()
}

// ============================================================================
// Path joining utilities
// Ported from TS http/src/route.ts
// ============================================================================

/// Allowed segment separator characters for path normalization.
const ALLOWED_SEGMENT_SEPARATORS: &[char] = &['/', ':', '?'];

/// Check if a path fragment needs a slash prefix.
fn needs_slash_prefix(fragment: &str) -> bool {
    if fragment.is_empty() {
        return false;
    }
    let first = fragment.chars().next().expect("fragment is non-empty");
    if ALLOWED_SEGMENT_SEPARATORS.contains(&first) {
        return false;
    }
    if first == '{' {
        let mut chars = fragment.chars();
        chars.next();
        if chars.next() == Some('/') {
            return false;
        }
    }
    true
}

/// Normalize a path fragment by adding a slash prefix if needed.
fn normalize_fragment(fragment: &str, trim_last: bool) -> String {
    let mut result = if needs_slash_prefix(fragment) {
        format!("/{}", fragment)
    } else {
        fragment.to_string()
    };
    if trim_last && result.ends_with('/') {
        result.pop();
    }
    result
}

/// Join path segments into a single path string.
/// Ported from TS `joinPathSegments()`.
///
/// Normalizes each segment by adding slash prefixes where needed
/// and optionally trimming trailing slashes from non-final segments.
///
/// # Example
/// `join_path_segments(&["pets", "{id}"])` -> `"/pets/{id}"`
/// `join_path_segments(&["/pets", "/{id}"])` -> `"/pets/{id}"`
pub fn join_path_segments(segments: &[&str]) -> String {
    let mut current = String::new();
    for (index, segment) in segments.iter().enumerate() {
        let trim_last = index < segments.len() - 1;
        current.push_str(&normalize_fragment(segment, trim_last));
    }
    current
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_template() {
        let result = parse_uri_template("/pets/{id}");
        assert_eq!(result.parameters.len(), 1);
        assert_eq!(result.parameters[0].name, "id");
        assert_eq!(result.parameters[0].operator, None);
        assert_eq!(result.parameters[0].modifier, None);
    }

    #[test]
    fn test_multiple_parameters() {
        let result = parse_uri_template("/pets/{petId}/toys/{toyId}");
        assert_eq!(result.parameters.len(), 2);
        assert_eq!(result.parameters[0].name, "petId");
        assert_eq!(result.parameters[1].name, "toyId");
    }

    #[test]
    fn test_query_parameters() {
        let result = parse_uri_template("/search{?q,limit}");
        assert_eq!(result.parameters.len(), 2);
        assert_eq!(result.parameters[0].name, "q");
        assert_eq!(result.parameters[0].operator, Some("?".to_string()));
        assert_eq!(result.parameters[1].name, "limit");
    }

    #[test]
    fn test_explode_modifier() {
        let result = parse_uri_template("/search{?q*}");
        assert_eq!(result.parameters.len(), 1);
        assert_eq!(
            result.parameters[0].modifier,
            Some(UriTemplateModifier::Explode)
        );
    }

    #[test]
    fn test_prefix_modifier() {
        let result = parse_uri_template("/search{?q:3}");
        assert_eq!(result.parameters.len(), 1);
        assert_eq!(
            result.parameters[0].modifier,
            Some(UriTemplateModifier::Prefix(3))
        );
    }

    #[test]
    fn test_reserved_expansion() {
        let result = parse_uri_template("/path/{+path}");
        assert_eq!(result.parameters.len(), 1);
        assert_eq!(result.parameters[0].operator, Some("+".to_string()));
        assert_eq!(result.parameters[0].name, "path");
    }

    #[test]
    fn test_fragment_expansion() {
        let result = parse_uri_template("/search{#query}");
        assert_eq!(result.parameters.len(), 1);
        assert_eq!(result.parameters[0].operator, Some("#".to_string()));
    }

    #[test]
    fn test_label_expansion() {
        let result = parse_uri_template("/search{.format}");
        assert_eq!(result.parameters.len(), 1);
        assert_eq!(result.parameters[0].operator, Some(".".to_string()));
    }

    #[test]
    fn test_path_segment_expansion() {
        let result = parse_uri_template("{/id}");
        assert_eq!(result.parameters.len(), 1);
        assert_eq!(result.parameters[0].operator, Some("/".to_string()));
    }

    #[test]
    fn test_path_style_parameter() {
        let result = parse_uri_template("{;params}");
        assert_eq!(result.parameters.len(), 1);
        assert_eq!(result.parameters[0].operator, Some(";".to_string()));
    }

    #[test]
    fn test_form_query_continuation() {
        let result = parse_uri_template("/search{&q,limit}");
        assert_eq!(result.parameters.len(), 2);
        assert_eq!(result.parameters[0].operator, Some("&".to_string()));
    }

    #[test]
    fn test_segments() {
        let result = parse_uri_template("/pets/{id}/toys/{toyId}");
        assert_eq!(result.segments.len(), 4);
        assert!(matches!(&result.segments[0], UriTemplateSegment::Literal(s) if s == "/pets/"));
        assert!(matches!(&result.segments[1], UriTemplateSegment::Parameter(p) if p.name == "id"));
        assert!(matches!(&result.segments[2], UriTemplateSegment::Literal(s) if s == "/toys/"));
        assert!(
            matches!(&result.segments[3], UriTemplateSegment::Parameter(p) if p.name == "toyId")
        );
    }

    #[test]
    fn test_no_parameters() {
        let result = parse_uri_template("/pets");
        assert_eq!(result.parameters.len(), 0);
        assert_eq!(result.segments.len(), 1);
    }

    #[test]
    fn test_empty_template() {
        let result = parse_uri_template("");
        assert_eq!(result.parameters.len(), 0);
        assert_eq!(result.segments.len(), 0);
    }

    #[test]
    fn test_literal_only_segments() {
        let result = parse_uri_template("/api/v1/resources");
        assert_eq!(result.segments.len(), 1);
        assert!(
            matches!(&result.segments[0], UriTemplateSegment::Literal(s) if s == "/api/v1/resources")
        );
    }

    #[test]
    fn test_extract_params_from_path() {
        let params = extract_params_from_path("foo/{name}/bar");
        assert_eq!(params, vec!["name"]);
    }

    #[test]
    fn test_extract_params_multiple() {
        let params = extract_params_from_path("/pets/{petId}/toys/{toyId}");
        assert_eq!(params, vec!["petId", "toyId"]);
    }

    #[test]
    fn test_extract_params_none() {
        let params = extract_params_from_path("/static/path");
        assert!(params.is_empty());
    }

    #[test]
    fn test_extract_params_empty() {
        let params = extract_params_from_path("");
        assert!(params.is_empty());
    }

    // ========================================================================
    // Path joining tests
    // ========================================================================

    #[test]
    fn test_join_path_segments_basic() {
        let path = join_path_segments(&["pets", "{id}"]);
        assert_eq!(path, "/pets/{id}");
    }

    #[test]
    fn test_join_path_segments_with_slashes() {
        let path = join_path_segments(&["/pets", "/{id}"]);
        assert_eq!(path, "/pets/{id}");
    }

    #[test]
    fn test_join_path_segments_single() {
        let path = join_path_segments(&["pets"]);
        assert_eq!(path, "/pets");
    }

    #[test]
    fn test_join_path_segments_empty() {
        let path = join_path_segments(&[]);
        assert_eq!(path, "");
    }

    #[test]
    fn test_join_path_segments_trailing_slash() {
        let path = join_path_segments(&["pets/", "items"]);
        assert_eq!(path, "/pets/items");
    }

    #[test]
    fn test_join_path_segments_question_mark() {
        let path = join_path_segments(&["search", "?q={query}"]);
        assert_eq!(path, "/search?q={query}");
    }

    #[test]
    fn test_join_path_segments_path_expansion() {
        let path = join_path_segments(&["/{+path}"]);
        assert_eq!(path, "/{+path}");
    }
}
