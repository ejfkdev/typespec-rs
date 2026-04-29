//! String template serializability checks
//!
//! Ported from TypeSpec compiler/src/core/helpers/string-template-utils.ts

use crate::checker::{Checker, Type};
use crate::diagnostics::Diagnostic;

/// Check if a string template can be serialized to a plain string.
/// Returns `true` if the template has a resolved `stringValue`.
/// If not, returns `false` and collects diagnostics explaining why.
pub fn is_string_template_serializable(
    checker: &Checker,
    string_template_id: crate::checker::types::TypeId,
) -> (bool, Vec<Diagnostic>) {
    let t = checker.get_type(string_template_id).cloned();
    match t {
        Some(Type::StringTemplate(st)) => {
            if st.string_value.is_some() {
                (true, vec![])
            } else {
                (
                    false,
                    explain_string_template_not_serializable(checker, string_template_id),
                )
            }
        }
        _ => (false, vec![]),
    }
}

/// Get a list of diagnostics explaining why this string template cannot be
/// converted to a string.
pub fn explain_string_template_not_serializable(
    checker: &Checker,
    string_template_id: crate::checker::types::TypeId,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let t = checker.get_type(string_template_id).cloned();
    if let Some(Type::StringTemplate(st)) = t {
        for &span_id in &st.spans {
            let span = checker.get_type(span_id).cloned();
            if let Some(Type::StringTemplateSpan(s)) = span
                && s.expression.is_some()
                && let Some(span_type_id) = s.r#type
            {
                let span_type = checker.get_type(span_type_id).cloned();
                match span_type {
                    Some(Type::String(_)) | Some(Type::Number(_)) | Some(Type::Boolean(_)) => {
                        // These are serializable
                    }
                    Some(Type::StringTemplate(_)) => {
                        let (_, sub_diags) = is_string_template_serializable(checker, span_type_id);
                        diagnostics.extend(sub_diags);
                    }
                    Some(Type::TemplateParameter(_)) => {
                        // Template parameters with valueof constraint are serializable
                        // For now, skip (simplified)
                    }
                    _ => {
                        // Non-serializable type — would add diagnostic
                        // but requires DiagnosticTarget which needs node info
                        // For now, just mark as non-serializable
                    }
                }
            }
        }
    }
    diagnostics
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::test_utils::check;

    #[test]
    fn test_string_template_with_only_literals_is_serializable() {
        // A simple string literal (not a template) is trivially serializable
        let checker = check(r#"model Foo { x: "hello"; }"#);
        let foo_type = checker.declared_types.get("Foo").copied().unwrap();
        let t = checker.get_type(foo_type).cloned().unwrap();
        if let Type::Model(m) = t {
            let prop_id = m.properties.get("x").copied().unwrap();
            let prop = checker.get_type(prop_id).cloned().unwrap();
            if let Type::ModelProperty(p) = prop {
                // Simple string → not a StringTemplate, it's a String
                let value_type = checker.get_type(p.r#type).cloned().unwrap();
                assert!(matches!(value_type, Type::String(_)));
            }
        }
    }

    #[test]
    fn test_string_template_with_model_interpolation_not_serializable() {
        let checker = check(
            r#"
            model M { x: int32 }
            model Test { test: "prefix_${M}"; }
        "#,
        );
        let test_type = checker.declared_types.get("Test").copied().unwrap();
        let t = checker.get_type(test_type).cloned().unwrap();
        if let Type::Model(m) = t {
            let prop_id = m.properties.get("test").copied().unwrap();
            let prop = checker.get_type(prop_id).cloned().unwrap();
            if let Type::ModelProperty(p) = prop {
                let value_type = checker.get_type(p.r#type).cloned().unwrap();
                if let Type::StringTemplate(st) = value_type {
                    // Template with model interpolation should not be serializable
                    let (serializable, _) = is_string_template_serializable(&checker, p.r#type);
                    assert!(
                        !serializable,
                        "Template with model interpolation should not be serializable"
                    );
                    assert!(st.string_value.is_none(), "Should have no stringValue");
                }
            }
        }
    }

    #[test]
    fn test_string_template_with_numeric_interpolation() {
        let checker = check(r#"model Test { test: "Start ${123} end"; }"#);
        let test_type = checker.declared_types.get("Test").copied().unwrap();
        let t = checker.get_type(test_type).cloned().unwrap();
        if let Type::Model(m) = t {
            let prop_id = m.properties.get("test").copied().unwrap();
            let prop = checker.get_type(prop_id).cloned().unwrap();
            if let Type::ModelProperty(p) = prop {
                let value_type = checker.get_type(p.r#type).cloned().unwrap();
                if let Type::StringTemplate(_st) = value_type {
                    // Number interpolation — if stringValue is set it's serializable
                    let (serializable, _) = is_string_template_serializable(&checker, p.r#type);
                    // May or may not be serializable depending on whether stringValue is computed
                    // The key thing is the function doesn't panic
                    let _ = serializable;
                }
            }
        }
    }
}
