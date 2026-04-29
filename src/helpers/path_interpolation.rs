//! Path template variable interpolation
//!
//! Ported from TypeSpec compiler/src/core/helpers/path-interpolation.ts

use regex::Regex;

/// Lazily compiled regex for path template variable extraction
static PATH_TEMPLATE_RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r"\{([a-zA-Z\-_.]+)\}(/|\.)?").expect("valid path template regex")
});

/// Interpolate a path template by replacing `{variable}` patterns with values
/// from the predefined variables map.
///
/// Supports dot-separated nested access (e.g., `{foo.bar}`).
/// When a variable is undefined and followed by `/` or `.`, the suffix is also removed.
///
/// # Examples
/// ```
/// use std::collections::HashMap;
/// use typespec_rs::helpers::path_interpolation::interpolate_path;
///
/// assert_eq!(interpolate_path("output.json", &HashMap::new()), "output.json");
/// let mut vars = HashMap::new();
/// vars.insert("version".to_string(), "v1".to_string());
/// assert_eq!(
///     interpolate_path("{version}/output.json", &vars),
///     "v1/output.json"
/// );
/// ```
pub fn interpolate_path(
    path_template: &str,
    predefined_variables: &std::collections::HashMap<String, String>,
) -> String {
    let mut result = String::new();
    let mut last_end = 0;

    for cap in PATH_TEMPLATE_RE.captures_iter(path_template) {
        let m = cap.get(0).expect("capture group 0 always exists");
        let expression = &cap[1];
        let suffix = cap.get(2).map(|s| s.as_str()).unwrap_or("");

        result.push_str(&path_template[last_end..m.start()]);

        if let Some(resolved) = resolve_expression(predefined_variables, expression) {
            if !suffix.is_empty() {
                result.push_str(&resolved);
                result.push_str(suffix);
            } else {
                result.push_str(&resolved);
            }
        }
        // If unresolved, omit the variable and suffix (if path segment separator)

        last_end = m.end();
    }

    result.push_str(&path_template[last_end..]);
    result
}

fn resolve_expression(
    predefined_variables: &std::collections::HashMap<String, String>,
    expression: &str,
) -> Option<String> {
    let segments: Vec<&str> = expression.split('.').collect();
    // For simple single-segment expressions, just look up directly
    if segments.len() == 1 {
        return predefined_variables.get(segments[0]).cloned();
    }
    // For dot-separated access, try looking up the full key first,
    // then try nested access
    let full_key = expression;
    if let Some(v) = predefined_variables.get(full_key) {
        return Some(v.clone());
    }
    // Try progressively shorter prefixes
    for i in 1..segments.len() {
        let prefix = &segments[..i].join(".");
        if let Some(v) = predefined_variables.get(prefix) {
            // The remaining segments would need to access nested fields
            // but since our values are just strings, we can't do nested access
            // Return the value only if all remaining segments are consumed
            if i == segments.len() {
                return Some(v.clone());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn vars(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn test_noop_if_nothing_to_interpolate() {
        assert_eq!(
            interpolate_path("output.json", &HashMap::new()),
            "output.json"
        );
    }

    #[test]
    fn test_interpolate_variable_in_path() {
        assert_eq!(
            interpolate_path("{version}/output.json", &vars(&[("version", "v1")])),
            "v1/output.json"
        );
    }

    #[test]
    fn test_interpolate_variable_in_filename() {
        assert_eq!(
            interpolate_path("output.{version}.json", &vars(&[("version", "v1")])),
            "output.v1.json"
        );
    }

    #[test]
    fn test_omit_path_segment_if_undefined_followed_by_slash() {
        assert_eq!(
            interpolate_path(
                "dist/{version}/output.json",
                &vars(&[("serviceName", "PetStore")])
            ),
            "dist/output.json"
        );
    }

    #[test]
    fn test_omit_segment_if_undefined_followed_by_dot() {
        assert_eq!(
            interpolate_path(
                "dist/{version}.output.json",
                &vars(&[("serviceName", "PetStore")])
            ),
            "dist/output.json"
        );
    }

    #[test]
    fn test_does_not_omit_if_in_middle_of_path_segment() {
        // {version}-suffix: the suffix after variable is "-", not "/" or ".",
        // so only the variable is removed, leaving "-suffix"
        assert_eq!(
            interpolate_path(
                "dist/{version}-suffix/output.json",
                &vars(&[("serviceName", "PetStore")])
            ),
            "dist/-suffix/output.json"
        );
    }
}
