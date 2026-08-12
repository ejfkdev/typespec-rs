//! @encode decorator tests
//!
//! Ported from TypeSpec compiler/test/decorators/decorators.test.ts
//! ("@encode" describe block), including the microsoft/typespec#10875
//! changes (`@encode(string)` on boolean).

use crate::checker::test_utils::{check, has_diagnostic};
use crate::libs::compiler::get_encode_data;

/// Get a type's simple (unqualified) name.
fn simple_name(
    checker: &crate::checker::Checker,
    type_id: crate::checker::types::TypeId,
) -> String {
    checker
        .get_type(type_id)
        .and_then(|t| t.name().map(|s| s.to_string()))
        .unwrap_or_default()
}

/// Get the encode data stored for a named type.
fn encode_data_for(
    checker: &crate::checker::Checker,
    type_name: &str,
) -> Option<crate::libs::compiler::EncodeData> {
    let type_id = checker.get_type_by_name(type_name)?;
    get_encode_data(&checker.state_accessors, type_id)
}

/// Get the encode data stored for a property of a model.
fn encode_data_for_prop(
    checker: &crate::checker::Checker,
    model_name: &str,
    prop_name: &str,
) -> Option<crate::libs::compiler::EncodeData> {
    let model_id = checker.get_type_by_name(model_name)?;
    let prop_id = match checker.get_type(model_id) {
        Some(crate::checker::Type::Model(m)) => *m.properties.get(prop_name)?,
        _ => return None,
    };
    get_encode_data(&checker.state_accessors, prop_id)
}

// ============================================================================
// Basic encoding storage
// ============================================================================

/// Ported from TS: "set encoding on scalar"
#[test]
fn test_encode_on_scalar() {
    let checker = check(
        r#"
        @encode("rfc3339")
        scalar s extends utcDateTime;
    "#,
    );
    let data = encode_data_for(&checker, "s").expect("encode data should be stored");
    assert_eq!(data.encoding.as_deref(), Some("rfc3339"));
}

/// Ported from TS: "set encoding on model property"
#[test]
fn test_encode_on_model_property() {
    let checker = check(
        r#"
        model Foo {
            @encode("rfc3339")
            prop: utcDateTime;
        }
    "#,
    );
    let data = encode_data_for_prop(&checker, "Foo", "prop").expect("encode data should be stored");
    assert_eq!(data.encoding.as_deref(), Some("rfc3339"));
}

/// Ported from TS: "encode type default to string"
#[test]
fn test_encode_type_defaults_to_string() {
    let checker = check(
        r#"
        @encode("rfc3339")
        scalar s extends utcDateTime;
    "#,
    );
    let data = encode_data_for(&checker, "s").expect("encode data should be stored");
    let encode_as = data
        .encode_as_type
        .expect("encode-as type should be stored");
    let name = simple_name(&checker, encode_as);
    assert_eq!(name, "string");
}

/// Ported from TS: "change encode type"
#[test]
fn test_encode_change_encode_type() {
    let checker = check(
        r#"
        @encode("unixTimestamp", int32)
        scalar s extends utcDateTime;
    "#,
    );
    let data = encode_data_for(&checker, "s").expect("encode data should be stored");
    let encode_as = data
        .encode_as_type
        .expect("encode-as type should be stored");
    let name = simple_name(&checker, encode_as);
    assert_eq!(name, "int32");
}

// ============================================================================
// @encode(string) — numeric and boolean (microsoft/typespec#10875)
// ============================================================================

/// Ported from TS: "@encode(string) on numeric scalar"
#[test]
fn test_encode_string_on_numeric_scalar() {
    let checker = check(
        r#"
        @encode(string)
        scalar s extends int64;
    "#,
    );
    let data = encode_data_for(&checker, "s").expect("encode data should be stored");
    assert_eq!(data.encoding, None, "encoding should be undefined");
    let encode_as = data
        .encode_as_type
        .expect("encode-as type should be stored");
    assert_eq!(simple_name(&checker, encode_as), "string");
}

/// Ported from TS: "@encode(string) on boolean model property"
/// (microsoft/typespec#10875)
#[test]
fn test_encode_string_on_boolean_property() {
    let checker = check(
        r#"
        model Foo {
            @encode(string)
            prop: boolean;
        }
    "#,
    );
    let data = encode_data_for_prop(&checker, "Foo", "prop").expect("encode data should be stored");
    assert_eq!(data.encoding, None, "encoding should be undefined");
    let encode_as = data
        .encode_as_type
        .expect("encode-as type should be stored");
    assert_eq!(simple_name(&checker, encode_as), "string");
    assert!(
        !has_diagnostic(&checker, "invalid-encode"),
        "boolean is a valid target for @encode(string): {:?}",
        checker.diagnostics()
    );
}

// ============================================================================
// Known encoding validation
// ============================================================================

/// Ported from TS valid cases (representative subset)
#[test]
fn test_encode_valid_cases() {
    let cases = [
        ("utcDateTime", "rfc3339"),
        ("utcDateTime", "rfc7231"),
        ("offsetDateTime", "rfc3339"),
        ("offsetDateTime", "rfc7231"),
        ("bytes", "base64"),
        ("bytes", "base64url"),
    ];
    for (target, encoding) in cases {
        let checker = check(&format!(
            r#"
            @encode("{}")
            scalar s extends {};
        "#,
            encoding, target
        ));
        assert!(
            !has_diagnostic(&checker, "invalid-encode"),
            "encoding '{}' on {} should be valid: {:?}",
            encoding,
            target,
            checker.diagnostics()
        );
    }
}

/// Ported from TS: unknown encodings are not blocked
#[test]
fn test_encode_unknown_encoding_not_blocked() {
    let checker = check(
        r#"
        @encode("custom-encoding")
        scalar s extends utcDateTime;
    "#,
    );
    assert!(
        !has_diagnostic(&checker, "invalid-encode"),
        "unknown encoding should not be validated: {:?}",
        checker.diagnostics()
    );
}

/// Ported from TS invalid cases (representative subset)
#[test]
fn test_encode_invalid_cases() {
    // rfc3339 with wrong encode-as type
    let checker = check(
        r#"
        @encode("rfc3339", int32)
        scalar s extends utcDateTime;
    "#,
    );
    assert!(
        checker.diagnostics().iter().any(|d| d.code == "invalid-encode"
            && d.message == "Encoding 'rfc3339' on type 's' is expected to be serialized as 'string' but got 'int32'."),
        "diagnostics: {:?}",
        checker.diagnostics()
    );

    // unixTimestamp on offsetDateTime — wrong target
    let checker = check(
        r#"
        @encode("unixTimestamp", int32)
        scalar s extends offsetDateTime;
    "#,
    );
    assert!(
        checker.diagnostics().iter().any(|d| d.code == "invalid-encode"
            && d.message == "Encoding 'unixTimestamp' cannot be used on type 's'. Expected: utcDateTime."),
        "diagnostics: {:?}",
        checker.diagnostics()
    );

    // rfc3339 on duration — wrong target
    let checker = check(
        r#"
        @encode("rfc3339")
        scalar s extends duration;
    "#,
    );
    assert!(
        checker.diagnostics().iter().any(|d| d.code == "invalid-encode"
            && d.message == "Encoding 'rfc3339' cannot be used on type 's'. Expected: utcDateTime, offsetDateTime."),
        "diagnostics: {:?}",
        checker.diagnostics()
    );

    // seconds on duration without numeric encode-as
    let checker = check(
        r#"
        @encode("seconds")
        scalar s extends duration;
    "#,
    );
    assert!(
        checker.diagnostics().iter().any(|d| d.code == "invalid-encode"
            && d.message == "Encoding 'seconds' on type 's' is expected to be serialized as 'numeric' but got 'string'. Set '@encode' 2nd parameter to be of type numeric. e.g. '@encode(\"seconds\", int32)'"),
        "diagnostics: {:?}",
        checker.diagnostics()
    );
}

/// Ported from TS (updated in microsoft/typespec#10875):
/// "@encode(string) on non-numeric/non-boolean scalar" — the expected list
/// now includes boolean.
#[test]
fn test_encode_string_on_invalid_scalar() {
    let checker = check(
        r#"
        scalar s;

        model Foo {
            @encode(string)
            prop: s;
        }
    "#,
    );
    assert!(
        checker
            .diagnostics()
            .iter()
            .any(|d| d.code == "invalid-encode"
                && d.message
                    == "Encoding 'string' cannot be used on type 's'. Expected: numeric, boolean."),
        "diagnostics: {:?}",
        checker.diagnostics()
    );
}
