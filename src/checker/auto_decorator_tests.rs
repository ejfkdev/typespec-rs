//! Auto decorator tests
//!
//! Ported from TypeSpec compiler/test/checker/decorators.test.ts
//! ("auto decorators" describe block, microsoft/typespec#10197).

use crate::checker::auto_decorator::{AutoDecoratorValue, get_auto_decorator_state_key};
use crate::checker::test_utils::{check, check_with_feature, has_diagnostic};
use crate::checker::{Checker, DecoratorDeclarationKind, Type};

/// Check source with the `auto-decorators` compiler feature enabled.
fn check_with_auto_feature(source: &str) -> Checker {
    check_with_feature(source, "auto-decorators")
}

/// Get the TypeId of a top-level model by name.
fn get_model(checker: &Checker, name: &str) -> Option<crate::checker::types::TypeId> {
    checker.get_type_by_name(name)
}

// ============================================================================
// Declaration
// ============================================================================

/// Ported from TS: "auto decorator does not require an implementation"
#[test]
fn test_auto_decorator_does_not_require_implementation() {
    let checker = check_with_auto_feature(
        r#"
        auto dec myFlag(target: unknown);
    "#,
    );
    assert!(
        !has_diagnostic(&checker, "missing-implementation"),
        "auto dec should not require an implementation: {:?}",
        checker.diagnostics()
    );
    // The declaration should be registered with declaration kind Auto.
    let dec = checker
        .get_declared_type_in_scope("myFlag")
        .expect("myFlag should be declared");
    match checker.get_type(dec) {
        Some(Type::Decorator(d)) => {
            assert_eq!(d.declaration_kind, DecoratorDeclarationKind::Auto);
        }
        other => panic!("expected Decorator type, got {:?}", other),
    }
}

/// Ported from TS: "emits error without feature flag"
#[test]
fn test_auto_decorator_error_without_feature_flag() {
    let checker = check(
        r#"
        auto dec myFlag(target: unknown);
    "#,
    );
    let diag = checker
        .diagnostics()
        .iter()
        .find(|d| d.code == "auto-decorator-disabled");
    assert!(
        diag.is_some(),
        "should report auto-decorator-disabled: {:?}",
        checker.diagnostics()
    );
    assert!(
        diag.unwrap()
            .message
            .contains("Auto decorator declarations require the 'auto-decorators' feature")
    );
}

/// Ported from TS: "internal auto dec is valid"
#[test]
fn test_internal_auto_dec_is_valid() {
    let checker = check_with_auto_feature(
        r#"
        internal auto dec myDec(target: unknown);
    "#,
    );
    assert!(
        checker.diagnostics().is_empty(),
        "internal auto dec should not diagnose: {:?}",
        checker.diagnostics()
    );
}

// ============================================================================
// State storage
// ============================================================================

/// Ported from TS: "auto decorator with no args stores empty record in stateMap"
#[test]
fn test_auto_decorator_no_args_stores_empty_record() {
    let checker = check_with_auto_feature(
        r#"
        auto dec myFlag(target: unknown);

        @myFlag
        model Foo {}
    "#,
    );
    let foo = get_model(&checker, "Foo").expect("Foo should exist");
    let value = checker.get_auto_decorator_value("myFlag", foo);
    assert!(
        matches!(value, Some(record) if record.is_empty()),
        "expected empty record, got {:?}",
        value
    );
}

/// Ported from TS: "auto decorator with single arg stores as key-value record in stateMap"
#[test]
fn test_auto_decorator_single_arg_record() {
    let checker = check_with_auto_feature(
        r#"
        auto dec myLabel(target: unknown, label: valueof string);

        @myLabel("hello")
        model Foo {}
    "#,
    );
    let foo = get_model(&checker, "Foo").expect("Foo should exist");
    let record = checker
        .get_auto_decorator_value("myLabel", foo)
        .expect("myLabel should be applied");
    assert_eq!(record.len(), 1);
    assert_eq!(record[0].0, "label");
    match &record[0].1 {
        AutoDecoratorValue::Value(crate::checker::DecoratorMarshalledValue::String(s)) => {
            assert_eq!(s, "hello")
        }
        other => panic!("expected String(hello), got {:?}", other),
    }
}

/// Ported from TS: "auto decorator with multiple args stores named record in stateMap"
#[test]
fn test_auto_decorator_multiple_args_record() {
    let checker = check_with_auto_feature(
        r#"
        auto dec myMeta(target: unknown, name: valueof string, version: valueof int32);

        @myMeta("test", 42)
        model Foo {}
    "#,
    );
    let foo = get_model(&checker, "Foo").expect("Foo should exist");
    let record = checker
        .get_auto_decorator_value("myMeta", foo)
        .expect("myMeta should be applied");
    assert_eq!(record.len(), 2);
    assert_eq!(record[0].0, "name");
    assert_eq!(record[1].0, "version");
    match &record[0].1 {
        AutoDecoratorValue::Value(crate::checker::DecoratorMarshalledValue::String(s)) => {
            assert_eq!(s, "test")
        }
        other => panic!("expected String(test), got {:?}", other),
    }
    match &record[1].1 {
        AutoDecoratorValue::Value(crate::checker::DecoratorMarshalledValue::Number(n)) => {
            assert_eq!(*n, 42.0)
        }
        other => panic!("expected Number(42), got {:?}", other),
    }
}

/// Ported from TS: "auto decorator in namespace uses FQN for state key"
#[test]
fn test_auto_decorator_namespace_fqn_key() {
    let checker = check_with_auto_feature(
        r#"
        namespace MyLib {
          auto dec myLabel(target: unknown, label: valueof string);
        }

        @MyLib.myLabel("world")
        model Foo {}
    "#,
    );
    let foo = get_model(&checker, "Foo").expect("Foo should exist");
    let record = checker
        .get_auto_decorator_value("MyLib.myLabel", foo)
        .expect("MyLib.myLabel should be applied");
    assert_eq!(record.len(), 1);
    assert_eq!(record[0].0, "label");
}

/// Ported from TS: "auto decorator with rest params stores as array in record"
#[test]
fn test_auto_decorator_rest_params_array() {
    let checker = check_with_auto_feature(
        r#"
        auto dec tags(target: unknown, ...tags: valueof string[]);

        @tags("a", "b", "c")
        model Foo {}
    "#,
    );
    let foo = get_model(&checker, "Foo").expect("Foo should exist");
    let record = checker
        .get_auto_decorator_value("tags", foo)
        .expect("tags should be applied");
    assert_eq!(record.len(), 1);
    assert_eq!(record[0].0, "tags");
    match &record[0].1 {
        AutoDecoratorValue::Array(values) => {
            let strs: Vec<&str> = values
                .iter()
                .map(|v| match v {
                    crate::checker::DecoratorMarshalledValue::String(s) => s.as_str(),
                    _ => panic!("expected string values, got {:?}", v),
                })
                .collect();
            assert_eq!(strs, vec!["a", "b", "c"]);
        }
        other => panic!("expected Array, got {:?}", other),
    }
}

/// Ported from TS: "auto decorator with mixed params and rest stores correctly"
#[test]
fn test_auto_decorator_mixed_params_and_rest() {
    let checker = check_with_auto_feature(
        r#"
        auto dec route(target: unknown, path: valueof string, ...tags: valueof string[]);

        @route("/foo", "tag1", "tag2")
        model Foo {}
    "#,
    );
    let foo = get_model(&checker, "Foo").expect("Foo should exist");
    let record = checker
        .get_auto_decorator_value("route", foo)
        .expect("route should be applied");
    assert_eq!(record.len(), 2);
    assert_eq!(record[0].0, "path");
    assert_eq!(record[1].0, "tags");
    match &record[1].1 {
        AutoDecoratorValue::Array(values) => assert_eq!(values.len(), 2),
        other => panic!("expected Array, got {:?}", other),
    }
}

/// Ported from TS: "auto decorator with optional param stores undefined for missing arg"
#[test]
fn test_auto_decorator_optional_missing_arg_stores_null() {
    let checker = check_with_auto_feature(
        r#"
        auto dec myDec(target: unknown, name?: valueof string);

        @myDec
        model Foo {}
    "#,
    );
    let foo = get_model(&checker, "Foo").expect("Foo should exist");
    let record = checker
        .get_auto_decorator_value("myDec", foo)
        .expect("myDec should be applied");
    assert_eq!(record.len(), 1);
    assert_eq!(record[0].0, "name");
    match &record[0].1 {
        AutoDecoratorValue::Value(crate::checker::DecoratorMarshalledValue::Null) => {}
        other => panic!("expected Null for missing optional arg, got {:?}", other),
    }
}

// ============================================================================
// Duplicates
// ============================================================================

/// Ported from TS: "emits duplicate-decorator warning when applied twice on same node"
#[test]
fn test_auto_decorator_duplicate_warns() {
    let checker = check_with_auto_feature(
        r#"
        auto dec myFlag(target: unknown);

        @myFlag
        @myFlag
        model Foo {}
    "#,
    );
    let count = checker
        .diagnostics()
        .iter()
        .filter(|d| d.code == "duplicate-decorator")
        .count();
    assert_eq!(
        count,
        2,
        "expected two duplicate-decorator warnings, got {:?} in {:?}",
        count,
        checker.diagnostics()
    );
}

/// Ported from TS: "duplicate auto decorators on same node are last-write-wins"
#[test]
fn test_auto_decorator_duplicate_last_write_wins() {
    let checker = check_with_auto_feature(
        r#"
        auto dec myLabel(target: unknown, label: valueof string);

        @myLabel("first")
        @myLabel("second")
        model Foo {}
    "#,
    );
    let foo = get_model(&checker, "Foo").expect("Foo should exist");
    let record = checker
        .get_auto_decorator_value("myLabel", foo)
        .expect("myLabel should be applied");
    match &record[0].1 {
        AutoDecoratorValue::Value(crate::checker::DecoratorMarshalledValue::String(s)) => {
            assert_eq!(s, "second", "last application should win")
        }
        other => panic!("expected String(second), got {:?}", other),
    }
}

// ============================================================================
// Read APIs
// ============================================================================

/// Ported from TS: "getAutoDecoratorTargets returns all targets"
#[test]
fn test_get_auto_decorator_targets() {
    let checker = check_with_auto_feature(
        r#"
        auto dec myFlag(target: unknown);

        @myFlag
        model Foo {}

        @myFlag
        model Bar {}
    "#,
    );
    let targets = checker
        .get_auto_decorator_targets("myFlag")
        .expect("myFlag should have targets");
    assert_eq!(targets.len(), 2);
}

/// Ported from TS: "hasAutoDecorator reflects whether the decorator was applied"
#[test]
fn test_has_auto_decorator() {
    let checker = check_with_auto_feature(
        r#"
        auto dec myFlag(target: unknown);

        @myFlag
        model Foo {}

        model Bar {}
    "#,
    );
    let foo = get_model(&checker, "Foo").expect("Foo should exist");
    let bar = get_model(&checker, "Bar").expect("Bar should exist");
    assert!(checker.has_auto_decorator("myFlag", foo));
    assert!(!checker.has_auto_decorator("myFlag", bar));
}

/// Ported from TS: "getAutoDecoratorValue returns undefined when not applied"
#[test]
fn test_get_auto_decorator_value_not_applied() {
    let checker = check_with_auto_feature(
        r#"
        auto dec myFlag(target: unknown);

        model Foo {}
    "#,
    );
    let foo = get_model(&checker, "Foo").expect("Foo should exist");
    assert!(checker.get_auto_decorator_value("myFlag", foo).is_none());
}

/// The state key uses the `dec:<fqn>` scheme (identity-based, not
/// declaration-style-based).
#[test]
fn test_auto_decorator_state_key() {
    assert_eq!(
        get_auto_decorator_state_key("MyLib.label"),
        "dec:MyLib.label"
    );
}

// ============================================================================
// Modifier validation (ported from TS decorator declaration tests updated in
// microsoft/typespec#10197)
// ============================================================================

/// Ported from TS: "errors if decorator is missing extern or auto modifier"
#[test]
fn test_dec_missing_extern_or_auto() {
    let checker = check(
        r#"
        dec testDec(target: unknown);
    "#,
    );
    let diag = checker
        .diagnostics()
        .iter()
        .find(|d| d.code == "invalid-modifier");
    assert!(diag.is_some(), "diagnostics: {:?}", checker.diagnostics());
    assert_eq!(
        diag.unwrap().message,
        "Declaration of type 'dec' is missing one of the required modifiers: 'extern' or 'auto'."
    );
}

/// Ported from TS: "errors if both extern and auto modifiers are used"
#[test]
fn test_dec_extern_and_auto_conflict() {
    let checker = check(
        r#"
        auto extern dec testDec(target: unknown);
    "#,
    );
    assert!(
        checker
            .diagnostics()
            .iter()
            .any(|d| d.code == "invalid-modifier"
                && d.message == "Modifiers 'extern' and 'auto' cannot be used together."),
        "diagnostics: {:?}",
        checker.diagnostics()
    );
    assert!(
        has_diagnostic(&checker, "auto-decorator-disabled"),
        "diagnostics: {:?}",
        checker.diagnostics()
    );
}

/// Ported from TS: "errors if auto modifier is used on a model declaration"
#[test]
fn test_auto_on_model_invalid() {
    let checker = check(
        r#"
        auto model Foo {}
    "#,
    );
    let diag = checker
        .diagnostics()
        .iter()
        .find(|d| d.code == "invalid-modifier");
    assert!(diag.is_some(), "diagnostics: {:?}", checker.diagnostics());
    assert_eq!(
        diag.unwrap().message,
        "Modifier 'auto' cannot be used on declarations of type 'model'."
    );
}

/// Ported from TS: "errors if auto modifier is used on a function declaration"
#[test]
fn test_auto_on_function_invalid() {
    let checker = check(
        r#"
        auto extern fn foo(): void;
    "#,
    );
    assert!(
        checker
            .diagnostics()
            .iter()
            .any(|d| d.code == "invalid-modifier"
                && d.message
                    == "Modifier 'auto' cannot be used on declarations of type 'function'."),
        "diagnostics: {:?}",
        checker.diagnostics()
    );
}

/// Ported from TS: "setAutoDecorator programmatically marks a target read
/// back by the accessors" (microsoft/typespec#11247)
#[test]
fn test_set_auto_decorator_programmatic() {
    let mut checker = check("model Foo {}");
    let foo = get_model(&checker, "Foo").expect("Foo should exist");

    // No decorator written in source yet.
    assert!(!checker.has_auto_decorator("MyLib.myLabel", foo));

    checker.set_auto_decorator(
        "MyLib.myLabel",
        foo,
        vec![(
            "label".to_string(),
            AutoDecoratorValue::Value(crate::checker::DecoratorMarshalledValue::String(
                "world".to_string(),
            )),
        )],
    );
    assert!(checker.has_auto_decorator("MyLib.myLabel", foo));
    let record = checker
        .get_auto_decorator_value("MyLib.myLabel", foo)
        .expect("setAutoDecorator should store a record");
    assert_eq!(record.len(), 1);
    assert_eq!(record[0].0, "label");

    // Empty record for a no-arg mark.
    checker.set_auto_decorator("MyLib.myFlag", foo, Vec::new());
    let flag_record = checker
        .get_auto_decorator_value("MyLib.myFlag", foo)
        .expect("no-arg mark should store an empty record");
    assert!(flag_record.is_empty());
}
