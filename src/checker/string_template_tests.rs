//! Checker String Template Tests
//!
//! Ported from TypeSpec compiler/test/checker/string-template.test.ts
//!
//! Categories:
//! - Simple string template with numeric interpolation
//! - String interpolation with isInterpolated check
//! - Model type interpolation
//! - Mixed value/type interpolation diagnostic (mixed-string-template)
//! - Value in type context diagnostic (value-in-type)
//!
//! Skipped (needs valueof constraint resolution):
//! - Empty string interpolation in templates with valueof constraint
//! - Interpolating template parameter that can be a type or value
//! - Interpolating template parameter that is a value but used as type
//!
use crate::checker::Type;
use crate::checker::test_utils::check;

// ============================================================================
// Basic String Template Tests
// ============================================================================

#[test]
fn test_simple_string_template_with_numeric_interpolation() {
    // Ported from: "simple"
    // `"Start ${123} end"` → StringTemplate with 3 spans:
    //   span[0]: literal, isInterpolated=false (StringTemplateSpan with expression=None)
    //   span[1]: interpolated Number(123), isInterpolated=true (StringTemplateSpan with expression=Some)
    //   span[2]: literal, isInterpolated=false
    let checker = check(r#"model Test { test: "Start ${123} end"; }"#);
    let test_type = checker.declared_types.get("Test").copied().unwrap();
    let t = checker.get_type(test_type).cloned().unwrap();
    match t {
        Type::Model(m) => {
            let prop_id = m.properties.get("test").copied().unwrap();
            let prop = checker.get_type(prop_id).cloned().unwrap();
            match prop {
                Type::ModelProperty(p) => {
                    let value_type = checker.get_type(p.r#type).cloned().unwrap();
                    match value_type {
                        Type::StringTemplate(st) => {
                            assert_eq!(st.spans.len(), 3, "Should have 3 spans");

                            // Span 0: literal (not interpolated) → String("Start ")
                            let span0 = checker.get_type(st.spans[0]).cloned().unwrap();
                            match &span0 {
                                Type::StringTemplateSpan(s) => {
                                    assert!(
                                        s.expression.is_none(),
                                        "Span 0 should not be interpolated"
                                    );
                                    if let Some(tid) = s.r#type {
                                        let t = checker.get_type(tid).cloned().unwrap();
                                        match t {
                                            Type::String(str_type) => assert_eq!(
                                                str_type.value, "Start ",
                                                "Span 0 literal value should be 'Start '"
                                            ),
                                            other => panic!(
                                                "Expected String type for span 0, got {:?}",
                                                other.kind_name()
                                            ),
                                        }
                                    }
                                }
                                other => panic!(
                                    "Expected StringTemplateSpan for span 0, got {:?}",
                                    other.kind_name()
                                ),
                            }

                            // Span 1: interpolated Number(123)
                            let span1 = checker.get_type(st.spans[1]).cloned().unwrap();
                            match &span1 {
                                Type::StringTemplateSpan(s) => {
                                    assert!(
                                        s.expression.is_some(),
                                        "Span 1 should be interpolated"
                                    );
                                    if let Some(tid) = s.r#type {
                                        let t = checker.get_type(tid).cloned().unwrap();
                                        match t {
                                            Type::Number(n) => assert_eq!(n.value, 123.0),
                                            other => panic!(
                                                "Expected Number type for span 1, got {:?}",
                                                other.kind_name()
                                            ),
                                        }
                                    }
                                }
                                other => panic!(
                                    "Expected StringTemplateSpan for span 1, got {:?}",
                                    other.kind_name()
                                ),
                            }

                            // Span 2: literal (not interpolated) → String(" end")
                            let span2 = checker.get_type(st.spans[2]).cloned().unwrap();
                            match &span2 {
                                Type::StringTemplateSpan(s) => {
                                    assert!(
                                        s.expression.is_none(),
                                        "Span 2 should not be interpolated"
                                    );
                                    if let Some(tid) = s.r#type {
                                        let t = checker.get_type(tid).cloned().unwrap();
                                        match t {
                                            Type::String(str_type) => assert_eq!(
                                                str_type.value, " end",
                                                "Span 2 literal value should be ' end'"
                                            ),
                                            other => panic!(
                                                "Expected String type for span 2, got {:?}",
                                                other.kind_name()
                                            ),
                                        }
                                    }
                                }
                                other => panic!(
                                    "Expected StringTemplateSpan for span 2, got {:?}",
                                    other.kind_name()
                                ),
                            }
                        }
                        other => {
                            panic!("Expected StringTemplate type, got {:?}", other.kind_name())
                        }
                    }
                }
                other => panic!("Expected ModelProperty, got {:?}", other.kind_name()),
            }
        }
        other => panic!("Expected Model type, got {:?}", other.kind_name()),
    }
}

#[test]
fn test_string_template_with_string_interpolation() {
    // Ported from: "string interpolated are marked with isInterpolated"
    // `"Start ${"interpolate"} end"` → span[1] is String("interpolate")
    let checker = check(r#"model Test { test: "Start ${"interpolate"} end"; }"#);
    let test_type = checker.declared_types.get("Test").copied().unwrap();
    let t = checker.get_type(test_type).cloned().unwrap();
    match t {
        Type::Model(m) => {
            let prop_id = m.properties.get("test").copied().unwrap();
            let prop = checker.get_type(prop_id).cloned().unwrap();
            match prop {
                Type::ModelProperty(p) => {
                    let value_type = checker.get_type(p.r#type).cloned().unwrap();
                    match value_type {
                        Type::StringTemplate(st) => {
                            assert_eq!(st.spans.len(), 3);

                            // Span 0: literal → String("Start ")
                            let span0 = checker.get_type(st.spans[0]).cloned().unwrap();
                            match &span0 {
                                Type::StringTemplateSpan(s) => {
                                    assert!(
                                        s.expression.is_none(),
                                        "Span 0 should not be interpolated"
                                    );
                                    if let Some(tid) = s.r#type {
                                        let t = checker.get_type(tid).cloned().unwrap();
                                        match t {
                                            Type::String(str_type) => {
                                                assert_eq!(str_type.value, "Start ")
                                            }
                                            other => panic!(
                                                "Expected String type for span 0, got {:?}",
                                                other.kind_name()
                                            ),
                                        }
                                    }
                                }
                                other => panic!(
                                    "Expected StringTemplateSpan, got {:?}",
                                    other.kind_name()
                                ),
                            }

                            // Span 1: interpolated String("interpolate")
                            let span1 = checker.get_type(st.spans[1]).cloned().unwrap();
                            match &span1 {
                                Type::StringTemplateSpan(s) => {
                                    assert!(
                                        s.expression.is_some(),
                                        "Span 1 should be interpolated"
                                    );
                                    if let Some(tid) = s.r#type {
                                        let t = checker.get_type(tid).cloned().unwrap();
                                        match t {
                                            Type::String(str_type) => assert_eq!(
                                                str_type.value, "interpolate",
                                                "Interpolated string value should be 'interpolate'"
                                            ),
                                            other => panic!(
                                                "Expected String type for span 1, got {:?}",
                                                other.kind_name()
                                            ),
                                        }
                                    }
                                }
                                other => panic!(
                                    "Expected StringTemplateSpan, got {:?}",
                                    other.kind_name()
                                ),
                            }

                            // Span 2: literal → String(" end")
                            let span2 = checker.get_type(st.spans[2]).cloned().unwrap();
                            match &span2 {
                                Type::StringTemplateSpan(s) => {
                                    assert!(
                                        s.expression.is_none(),
                                        "Span 2 should not be interpolated"
                                    );
                                    if let Some(tid) = s.r#type {
                                        let t = checker.get_type(tid).cloned().unwrap();
                                        match t {
                                            Type::String(str_type) => {
                                                assert_eq!(str_type.value, " end")
                                            }
                                            other => panic!(
                                                "Expected String type for span 2, got {:?}",
                                                other.kind_name()
                                            ),
                                        }
                                    }
                                }
                                other => panic!(
                                    "Expected StringTemplateSpan, got {:?}",
                                    other.kind_name()
                                ),
                            }
                        }
                        other => panic!("Expected StringTemplate, got {:?}", other.kind_name()),
                    }
                }
                other => panic!("Expected ModelProperty, got {:?}", other.kind_name()),
            }
        }
        other => panic!("Expected Model, got {:?}", other.kind_name()),
    }
}

#[test]
fn test_string_template_with_model_interpolation() {
    // Ported from: "can interpolate a model"
    // `"Start ${TestModel} end"` → span[1] is Model named "TestModel"
    let checker = check(
        r#"
        model TestModel {}
        model Test { test: "Start ${TestModel} end"; }
    "#,
    );
    let test_type = checker.declared_types.get("Test").copied().unwrap();
    let t = checker.get_type(test_type).cloned().unwrap();
    match t {
        Type::Model(m) => {
            let prop_id = m.properties.get("test").copied().unwrap();
            let prop = checker.get_type(prop_id).cloned().unwrap();
            match prop {
                Type::ModelProperty(p) => {
                    let value_type = checker.get_type(p.r#type).cloned().unwrap();
                    match value_type {
                        Type::StringTemplate(st) => {
                            assert_eq!(st.spans.len(), 3);

                            // Span 0: literal → String("Start ")
                            let span0 = checker.get_type(st.spans[0]).cloned().unwrap();
                            match &span0 {
                                Type::StringTemplateSpan(s) => {
                                    assert!(
                                        s.expression.is_none(),
                                        "Span 0 should not be interpolated"
                                    );
                                    if let Some(tid) = s.r#type {
                                        let t = checker.get_type(tid).cloned().unwrap();
                                        match t {
                                            Type::String(str_type) => {
                                                assert_eq!(str_type.value, "Start ")
                                            }
                                            other => panic!(
                                                "Expected String type for span 0, got {:?}",
                                                other.kind_name()
                                            ),
                                        }
                                    }
                                }
                                other => panic!(
                                    "Expected StringTemplateSpan, got {:?}",
                                    other.kind_name()
                                ),
                            }

                            // Span 1: interpolated Model("TestModel")
                            let span1 = checker.get_type(st.spans[1]).cloned().unwrap();
                            match &span1 {
                                Type::StringTemplateSpan(s) => {
                                    assert!(
                                        s.expression.is_some(),
                                        "Span 1 should be interpolated"
                                    );
                                    if let Some(tid) = s.r#type {
                                        let t = checker.get_type(tid).cloned().unwrap();
                                        match t {
                                            Type::Model(m) => assert_eq!(m.name, "TestModel"),
                                            other => panic!(
                                                "Expected Model type for interpolated span, got {:?}",
                                                other.kind_name()
                                            ),
                                        }
                                    }
                                }
                                other => panic!(
                                    "Expected StringTemplateSpan, got {:?}",
                                    other.kind_name()
                                ),
                            }

                            // Span 2: literal → String(" end")
                            let span2 = checker.get_type(st.spans[2]).cloned().unwrap();
                            match &span2 {
                                Type::StringTemplateSpan(s) => {
                                    assert!(
                                        s.expression.is_none(),
                                        "Span 2 should not be interpolated"
                                    );
                                    if let Some(tid) = s.r#type {
                                        let t = checker.get_type(tid).cloned().unwrap();
                                        match t {
                                            Type::String(str_type) => {
                                                assert_eq!(str_type.value, " end")
                                            }
                                            other => panic!(
                                                "Expected String type for span 2, got {:?}",
                                                other.kind_name()
                                            ),
                                        }
                                    }
                                }
                                other => panic!(
                                    "Expected StringTemplateSpan, got {:?}",
                                    other.kind_name()
                                ),
                            }
                        }
                        other => panic!("Expected StringTemplate, got {:?}", other.kind_name()),
                    }
                }
                other => panic!("Expected ModelProperty, got {:?}", other.kind_name()),
            }
        }
        other => panic!("Expected Model, got {:?}", other.kind_name()),
    }
}

// ============================================================================
// String template as model property type (no interpolation)
// ============================================================================

#[test]
fn test_string_template_no_interpolation() {
    // A simple string literal (not a template) used as model property
    let checker = check(r#"model Foo { x: "hello"; }"#);
    let foo_type = checker.declared_types.get("Foo").copied().unwrap();
    let t = checker.get_type(foo_type).cloned().unwrap();
    match t {
        Type::Model(m) => {
            let prop_id = m.properties.get("x").copied().unwrap();
            let prop = checker.get_type(prop_id).cloned().unwrap();
            match prop {
                Type::ModelProperty(p) => {
                    let value_type = checker.get_type(p.r#type).cloned().unwrap();
                    // Simple string literal → String type, not StringTemplate
                    assert!(
                        matches!(value_type, Type::String(_)),
                        "Expected String type for simple string literal, got {:?}",
                        value_type.kind_name()
                    );
                }
                other => panic!("Expected ModelProperty, got {:?}", other.kind_name()),
            }
        }
        other => panic!("Expected Model, got {:?}", other.kind_name()),
    }
}

#[test]
fn test_string_template_multiple_interpolations() {
    // String template with multiple interpolations: "${A} and ${B}"
    let checker = check(
        r#"
        model A {}
        model B {}
        model Test { test: "${A} and ${B}"; }
    "#,
    );
    let test_type = checker.declared_types.get("Test").copied().unwrap();
    let t = checker.get_type(test_type).cloned().unwrap();
    match t {
        Type::Model(m) => {
            let prop_id = m.properties.get("test").copied().unwrap();
            let prop = checker.get_type(prop_id).cloned().unwrap();
            match prop {
                Type::ModelProperty(p) => {
                    let value_type = checker.get_type(p.r#type).cloned().unwrap();
                    match value_type {
                        Type::StringTemplate(st) => {
                            // Pattern: head(""), interp(A), mid(" and "), interp(B), tail("")
                            // = 5 spans
                            assert_eq!(
                                st.spans.len(),
                                5,
                                "Should have 5 spans for 2 interpolations, got {}",
                                st.spans.len()
                            );

                            // Span 0: literal → String("")
                            let span0 = checker.get_type(st.spans[0]).cloned().unwrap();
                            match &span0 {
                                Type::StringTemplateSpan(s) => {
                                    assert!(
                                        s.expression.is_none(),
                                        "Span 0 should not be interpolated"
                                    );
                                    if let Some(tid) = s.r#type {
                                        let t = checker.get_type(tid).cloned().unwrap();
                                        match t {
                                            Type::String(str_type) => assert_eq!(
                                                str_type.value, "",
                                                "Span 0 literal value should be empty string"
                                            ),
                                            other => panic!(
                                                "Expected String type for span 0, got {:?}",
                                                other.kind_name()
                                            ),
                                        }
                                    }
                                }
                                other => panic!(
                                    "Expected StringTemplateSpan, got {:?}",
                                    other.kind_name()
                                ),
                            }

                            // Span 1: interpolated Model A
                            let span1 = checker.get_type(st.spans[1]).cloned().unwrap();
                            match &span1 {
                                Type::StringTemplateSpan(s) => {
                                    assert!(
                                        s.expression.is_some(),
                                        "Span 1 should be interpolated"
                                    );
                                    if let Some(tid) = s.r#type {
                                        let t = checker.get_type(tid).cloned().unwrap();
                                        match t {
                                            Type::Model(m) => assert_eq!(m.name, "A"),
                                            other => panic!(
                                                "Expected Model type for span 1, got {:?}",
                                                other.kind_name()
                                            ),
                                        }
                                    }
                                }
                                other => panic!(
                                    "Expected StringTemplateSpan, got {:?}",
                                    other.kind_name()
                                ),
                            }

                            // Span 2: literal → String(" and ")
                            let span2 = checker.get_type(st.spans[2]).cloned().unwrap();
                            match &span2 {
                                Type::StringTemplateSpan(s) => {
                                    assert!(
                                        s.expression.is_none(),
                                        "Span 2 should not be interpolated"
                                    );
                                    if let Some(tid) = s.r#type {
                                        let t = checker.get_type(tid).cloned().unwrap();
                                        match t {
                                            Type::String(str_type) => assert_eq!(
                                                str_type.value, " and ",
                                                "Span 2 literal value should be ' and '"
                                            ),
                                            other => panic!(
                                                "Expected String type for span 2, got {:?}",
                                                other.kind_name()
                                            ),
                                        }
                                    }
                                }
                                other => panic!(
                                    "Expected StringTemplateSpan, got {:?}",
                                    other.kind_name()
                                ),
                            }

                            // Span 3: interpolated Model B
                            let span3 = checker.get_type(st.spans[3]).cloned().unwrap();
                            match &span3 {
                                Type::StringTemplateSpan(s) => {
                                    assert!(
                                        s.expression.is_some(),
                                        "Span 3 should be interpolated"
                                    );
                                    if let Some(tid) = s.r#type {
                                        let t = checker.get_type(tid).cloned().unwrap();
                                        match t {
                                            Type::Model(m) => assert_eq!(m.name, "B"),
                                            other => panic!(
                                                "Expected Model type for span 3, got {:?}",
                                                other.kind_name()
                                            ),
                                        }
                                    }
                                }
                                other => panic!(
                                    "Expected StringTemplateSpan, got {:?}",
                                    other.kind_name()
                                ),
                            }

                            // Span 4: literal → String("")
                            let span4 = checker.get_type(st.spans[4]).cloned().unwrap();
                            match &span4 {
                                Type::StringTemplateSpan(s) => {
                                    assert!(
                                        s.expression.is_none(),
                                        "Span 4 should not be interpolated"
                                    );
                                    if let Some(tid) = s.r#type {
                                        let t = checker.get_type(tid).cloned().unwrap();
                                        match t {
                                            Type::String(str_type) => assert_eq!(
                                                str_type.value, "",
                                                "Span 4 literal value should be empty string"
                                            ),
                                            other => panic!(
                                                "Expected String type for span 4, got {:?}",
                                                other.kind_name()
                                            ),
                                        }
                                    }
                                }
                                other => panic!(
                                    "Expected StringTemplateSpan, got {:?}",
                                    other.kind_name()
                                ),
                            }
                        }
                        other => panic!("Expected StringTemplate, got {:?}", other.kind_name()),
                    }
                }
                other => panic!("Expected ModelProperty, got {:?}", other.kind_name()),
            }
        }
        other => panic!("Expected Model, got {:?}", other.kind_name()),
    }
}

// ============================================================================
// Diagnostic Tests
// ============================================================================

#[test]
fn test_mixed_string_template_diagnostic() {
    // Ported from: "emit error if interpolating value and types"
    let checker = check(
        r#"
        const str1 = "hi";
        alias str2 = "${str1} and ${string}";
    "#,
    );
    let diags = checker.diagnostics();
    assert!(
        diags.iter().any(|d| d.code == "mixed-string-template"),
        "Should report mixed-string-template when interpolating values and types: {:?}",
        diags
    );
}

// ============================================================================
// String template with scalar type interpolation
// ============================================================================

#[test]
fn test_string_template_with_scalar_interpolation() {
    // Interpolating a scalar type (string) in a template
    let checker = check(r#"model Test { test: "prefix_${string}"; }"#);
    let test_type = checker.declared_types.get("Test").copied().unwrap();
    let t = checker.get_type(test_type).cloned().unwrap();
    match t {
        Type::Model(m) => {
            let prop_id = m.properties.get("test").copied().unwrap();
            let prop = checker.get_type(prop_id).cloned().unwrap();
            match prop {
                Type::ModelProperty(p) => {
                    let value_type = checker.get_type(p.r#type).cloned().unwrap();
                    match value_type {
                        Type::StringTemplate(st) => {
                            // May have 2 or 3 spans depending on whether trailing empty string is included
                            assert!(
                                st.spans.len() >= 2,
                                "Should have at least 2 spans, got {}",
                                st.spans.len()
                            );
                            // Span 0: literal "prefix_"
                            let span0 = checker.get_type(st.spans[0]).cloned().unwrap();
                            match &span0 {
                                Type::StringTemplateSpan(s) => {
                                    assert!(s.expression.is_none());
                                    if let Some(tid) = s.r#type {
                                        let t = checker.get_type(tid).cloned().unwrap();
                                        match t {
                                            Type::String(str_type) => {
                                                assert_eq!(str_type.value, "prefix_")
                                            }
                                            other => panic!(
                                                "Expected String, got {:?}",
                                                other.kind_name()
                                            ),
                                        }
                                    }
                                }
                                other => panic!(
                                    "Expected StringTemplateSpan, got {:?}",
                                    other.kind_name()
                                ),
                            }
                            // Span 1: interpolated scalar "string"
                            let span1 = checker.get_type(st.spans[1]).cloned().unwrap();
                            match &span1 {
                                Type::StringTemplateSpan(s) => {
                                    assert!(s.expression.is_some());
                                    if let Some(tid) = s.r#type {
                                        let t = checker.get_type(tid).cloned().unwrap();
                                        match t {
                                            Type::Scalar(sc) => assert_eq!(sc.name, "string"),
                                            other => panic!(
                                                "Expected Scalar, got {:?}",
                                                other.kind_name()
                                            ),
                                        }
                                    }
                                }
                                other => panic!(
                                    "Expected StringTemplateSpan, got {:?}",
                                    other.kind_name()
                                ),
                            }
                        }
                        other => panic!("Expected StringTemplate, got {:?}", other.kind_name()),
                    }
                }
                other => panic!("Expected ModelProperty, got {:?}", other.kind_name()),
            }
        }
        other => panic!("Expected Model, got {:?}", other.kind_name()),
    }
}

#[test]
fn test_string_template_only_interpolation() {
    // Template that is just an interpolation: "${SomeModel}"
    let checker = check(
        r#"
        model M { x: int32 }
        model Test { test: "${M}"; }
    "#,
    );
    let test_type = checker.declared_types.get("Test").copied().unwrap();
    let t = checker.get_type(test_type).cloned().unwrap();
    match t {
        Type::Model(m) => {
            let prop_id = m.properties.get("test").copied().unwrap();
            let prop = checker.get_type(prop_id).cloned().unwrap();
            match prop {
                Type::ModelProperty(p) => {
                    let value_type = checker.get_type(p.r#type).cloned().unwrap();
                    match value_type {
                        Type::StringTemplate(st) => {
                            assert!(
                                !st.spans.is_empty(),
                                "Should have at least 1 span, got {}",
                                st.spans.len()
                            );
                            // Find the interpolated span (one with expression = Some)
                            let mut found_model = false;
                            for &span_id in &st.spans {
                                let span = checker.get_type(span_id).cloned().unwrap();
                                match &span {
                                    Type::StringTemplateSpan(s) => {
                                        if s.expression.is_some()
                                            && let Some(tid) = s.r#type
                                        {
                                            let t = checker.get_type(tid).cloned().unwrap();
                                            match t {
                                                Type::Model(m) => {
                                                    assert_eq!(m.name, "M");
                                                    found_model = true;
                                                }
                                                other => panic!(
                                                    "Expected Model, got {:?}",
                                                    other.kind_name()
                                                ),
                                            }
                                        }
                                    }
                                    other => panic!(
                                        "Expected StringTemplateSpan, got {:?}",
                                        other.kind_name()
                                    ),
                                }
                            }
                            assert!(found_model, "Should find interpolated Model span");
                        }
                        other => panic!("Expected StringTemplate, got {:?}", other.kind_name()),
                    }
                }
                other => panic!("Expected ModelProperty, got {:?}", other.kind_name()),
            }
        }
        other => panic!("Expected Model, got {:?}", other.kind_name()),
    }
}

#[test]
fn test_string_template_with_enum_interpolation() {
    // Interpolating an enum type in a template
    let checker = check(
        r#"
        enum Color { red, green, blue }
        model Test { test: "color: ${Color}"; }
    "#,
    );
    let test_type = checker.declared_types.get("Test").copied().unwrap();
    let t = checker.get_type(test_type).cloned().unwrap();
    match t {
        Type::Model(m) => {
            let prop_id = m.properties.get("test").copied().unwrap();
            let prop = checker.get_type(prop_id).cloned().unwrap();
            match prop {
                Type::ModelProperty(p) => {
                    let value_type = checker.get_type(p.r#type).cloned().unwrap();
                    match value_type {
                        Type::StringTemplate(st) => {
                            assert!(
                                st.spans.len() >= 2,
                                "Should have at least 2 spans, got {}",
                                st.spans.len()
                            );
                            // Check that the interpolated span references the enum
                            let span1 = checker.get_type(st.spans[1]).cloned().unwrap();
                            match &span1 {
                                Type::StringTemplateSpan(s) => {
                                    assert!(s.expression.is_some());
                                    if let Some(tid) = s.r#type {
                                        let t = checker.get_type(tid).cloned().unwrap();
                                        match t {
                                            Type::Enum(e) => assert_eq!(e.name, "Color"),
                                            other => {
                                                panic!("Expected Enum, got {:?}", other.kind_name())
                                            }
                                        }
                                    }
                                }
                                other => panic!(
                                    "Expected StringTemplateSpan, got {:?}",
                                    other.kind_name()
                                ),
                            }
                        }
                        other => panic!("Expected StringTemplate, got {:?}", other.kind_name()),
                    }
                }
                other => panic!("Expected ModelProperty, got {:?}", other.kind_name()),
            }
        }
        other => panic!("Expected Model, got {:?}", other.kind_name()),
    }
}

// ============================================================================
// value-in-type diagnostic tests
// Ported from TS: "emit error if interpolating value in a context where
// template is used as a type"
// ============================================================================

/// Ported from TS: value-in-type in alias context
#[test]
fn test_value_in_type_alias_context() {
    let checker = check(
        r#"
        const str1 = "hi";
        alias str2 = "with value ${str1}";
    "#,
    );
    let diags = checker.diagnostics();
    assert!(
        diags.iter().any(|d| d.code == "value-in-type"),
        "Should report value-in-type when interpolating value in alias type context: {:?}",
        diags
    );
}

/// Ported from TS: value-in-type in model property context
#[test]
fn test_value_in_type_model_prop_context() {
    let checker = check(
        r#"
        const str1 = "hi";
        model Foo { a: "with value ${str1}"; }
    "#,
    );
    let diags = checker.diagnostics();
    assert!(
        diags.iter().any(|d| d.code == "value-in-type"),
        "Should report value-in-type when interpolating value in model property type context: {:?}",
        diags
    );
}

/// No diagnostic when interpolating types only (no values)
#[test]
fn test_no_diagnostic_type_only_interpolation() {
    let checker = check(
        r#"
        alias Foo = "prefix_${string}";
    "#,
    );
    let diags = checker.diagnostics();
    assert!(
        !diags
            .iter()
            .any(|d| d.code == "mixed-string-template" || d.code == "value-in-type"),
        "Should have no mixed-string-template or value-in-type for type-only interpolation: {:?}",
        diags
    );
}

// ============================================================================
// Non-literal string template diagnostic tests
// Ported from TS: test/checker/values/string-values.test.ts
// ============================================================================

/// Ported from TS: "emit error if string template is not serializable to string"
/// Interpolating `boolean` (a type, not a literal value) should emit non-literal-string-template
#[test]
fn test_non_literal_string_template_with_boolean_type() {
    let checker = check(r#"alias Foo = "one ${boolean} def";"#);
    let diags = checker.diagnostics();
    assert!(
        diags
            .iter()
            .any(|d| d.code == "non-literal-string-template"),
        "Should report non-literal-string-template when interpolating non-literal type: {:?}",
        diags
    );
}

/// Ported from TS: "emit error if string template if interpolating non serializable value"
/// Interpolating a model type in a string template should emit non-literal-string-template
#[test]
fn test_non_literal_string_template_with_model_type() {
    let checker = check(
        r#"
        model Bar { x: int32 }
        alias Foo = "value: ${Bar}";
    "#,
    );
    let diags = checker.diagnostics();
    assert!(
        diags
            .iter()
            .any(|d| d.code == "non-literal-string-template"),
        "Should report non-literal-string-template when interpolating model type: {:?}",
        diags
    );
}

/// Interpolating a scalar type in a string template should emit non-literal-string-template
#[test]
fn test_non_literal_string_template_with_scalar_type() {
    let checker = check(r#"alias Foo = "value: ${int32}";"#);
    let diags = checker.diagnostics();
    assert!(
        diags
            .iter()
            .any(|d| d.code == "non-literal-string-template"),
        "Should report non-literal-string-template when interpolating scalar type: {:?}",
        diags
    );
}

/// Interpolating a string literal should NOT emit non-literal-string-template
#[test]
fn test_literal_string_template_no_error() {
    let checker = check(r#"alias Foo = "hello ${"world"}";"#);
    let diags = checker.diagnostics();
    assert!(
        !diags
            .iter()
            .any(|d| d.code == "non-literal-string-template"),
        "Should NOT report non-literal-string-template for string literal: {:?}",
        diags
    );
}

/// Interpolating a numeric literal should NOT emit non-literal-string-template
#[test]
fn test_numeric_literal_string_template_no_error() {
    let checker = check(r#"alias Foo = "count: ${123}";"#);
    let diags = checker.diagnostics();
    assert!(
        !diags
            .iter()
            .any(|d| d.code == "non-literal-string-template"),
        "Should NOT report non-literal-string-template for numeric literal: {:?}",
        diags
    );
}

/// Ported from TS: "only emit invalid-ref error when interpolating an invalid reference, not non-literal-string-template"
/// When interpolating an unknown identifier, should report invalid-ref, NOT non-literal-string-template
#[test]
fn test_invalid_ref_not_non_literal_string_template() {
    let checker = check(r#"alias Foo = "Some ${bad}";"#);
    let diags = checker.diagnostics();
    assert!(
        diags.iter().any(|d| d.code == "invalid-ref"),
        "Should report invalid-ref for unknown identifier: {:?}",
        diags
    );
    assert!(
        !diags
            .iter()
            .any(|d| d.code == "non-literal-string-template"),
        "Should NOT report non-literal-string-template when invalid-ref already reported: {:?}",
        diags
    );
}
