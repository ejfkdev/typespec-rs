//! Suppression tests
//!
//! Ported from TypeSpec `compiler/test/suppression.test.ts`.
//! The upstream suite drives custom diagnostics through `navigateProgram`;
//! the Rust port exercises the same semantics with real checker diagnostics
//! (`deprecated` as the suppressible warning, `invalid-ref` as an error).

use crate::checker::test_utils::{assert_no_diag, check, count_diagnostics, has_diagnostic};

// ============================================================================
// Basic suppression behavior (ported from TS suppression.test.ts)
// ============================================================================

#[test]
fn test_emit_warning_when_no_suppression() {
    // Ported from TS: "emit warning diagnostics when there is no suppression"
    let checker = check(
        r#"
        #deprecated "Foo is deprecated"
        model Foo {}

        model Bar {
            name: Foo;
        }
        "#,
    );
    assert!(
        has_diagnostic(&checker, "deprecated"),
        "expected deprecated warning, got: {:?}",
        checker.diagnostics()
    );
}

#[test]
fn test_suppress_warning_on_item_itself() {
    // Ported from TS: "suppress warning diagnostic on item itself"
    let checker = check(
        r#"
        #deprecated "Foo is deprecated"
        model Foo {}

        model Bar {
            #suppress "deprecated" "This is needed"
            name: Foo;
        }
        "#,
    );
    assert_no_diag(&checker, "deprecated");
}

#[test]
fn test_suppress_warning_on_parent_node() {
    // Ported from TS: "suppress warning diagnostic on parent node"
    let checker = check(
        r#"
        #deprecated "Foo is deprecated"
        model Foo {}

        #suppress "deprecated" "This is needed"
        model Bar {
            name: Foo;
        }
        "#,
    );
    assert_no_diag(&checker, "deprecated");
}

#[test]
fn test_error_cannot_be_suppressed() {
    // Ported from TS: "error diagnostics cannot be suppressed and emit another error"
    let checker = check(
        r#"
        model Foo {
            #suppress "invalid-ref" "This is needed"
            id: Unknown;
        }
        "#,
    );
    assert!(
        has_diagnostic(&checker, "suppress-error"),
        "expected suppress-error, got: {:?}",
        checker.diagnostics()
    );
    assert!(
        has_diagnostic(&checker, "invalid-ref"),
        "expected the original error to remain, got: {:?}",
        checker.diagnostics()
    );
}

// ============================================================================
// Unused suppression tracking (ported from TS suppression.test.ts)
// ============================================================================

#[test]
fn test_reports_unused_suppression_via_tracker() {
    // Ported from TS: "reports unused suppression via tracker"
    let checker = check(
        r#"
        #suppress "deprecated" "not needed anymore"
        model Foo {}
        "#,
    );
    let unused = checker.unused_suppressions();
    assert_eq!(unused.len(), 1, "unused: {:?}", unused);
    assert_eq!(unused[0].directive.code, "deprecated");
}

#[test]
fn test_no_unused_suppression_when_diagnostic_was_suppressed() {
    // Ported from TS: "does not report unused suppression when diagnostic was suppressed"
    let checker = check(
        r#"
        #deprecated "Old is deprecated"
        model Old {}

        model Foo {
            #suppress "deprecated" "intentional"
            prop: Old;
        }
        "#,
    );
    assert_no_diag(&checker, "deprecated");
    let unused = checker.unused_suppressions();
    assert!(
        unused.is_empty(),
        "suppression was used, should not be reported unused: {:?}",
        unused
    );
}

#[test]
fn test_no_unused_suppression_for_unavailable_diagnostic_source() {
    // Ported from TS: "does not report unused suppression for unavailable diagnostic source"
    let checker = check(
        r#"
        #suppress "test-emitter/not-run" "only emitted by another tool"
        model Foo {}
        "#,
    );
    let unused = checker.unused_suppressions();
    assert!(
        unused.is_empty(),
        "unknown diagnostic source should not be reported unused: {:?}",
        unused
    );
}

#[test]
fn test_no_unused_suppression_for_errors_replacement_of_suppress_error() {
    // Ported from TS: "does not report unused suppression for errors as
    // replacement for suppress-error"
    let checker = check(
        r#"
        model Foo {
            #suppress "invalid-ref" "errors cannot be suppressed"
            prop: Unknown;
        }
        "#,
    );
    assert!(
        has_diagnostic(&checker, "suppress-error"),
        "expected suppress-error, got: {:?}",
        checker.diagnostics()
    );
    assert!(
        has_diagnostic(&checker, "invalid-ref"),
        "expected invalid-ref, got: {:?}",
        checker.diagnostics()
    );
    // The suppression for an error should not appear as "unused" since it
    // was explicitly rejected (marked used).
    let unused = checker.unused_suppressions();
    assert!(
        unused.is_empty(),
        "error suppression is marked used via suppress-error: {:?}",
        unused
    );
}

#[test]
fn test_unused_suppression_for_linter_code() {
    // Suppressions using the built-in linter library prefix are "available"
    // and reported unused (mirrors TS availability via builtInLinterLibraryName).
    let checker = check(
        r#"
        #suppress "@typespec/compiler/unused-using" "we will use it later"
        model Foo {}
        "#,
    );
    let unused = checker.unused_suppressions();
    assert_eq!(unused.len(), 1, "unused: {:?}", unused);
    assert_eq!(unused[0].directive.code, "@typespec/compiler/unused-using");
}

#[test]
fn test_unused_suppression_for_loaded_library_code() {
    // Suppressions with a loaded-library prefix are available once the
    // library name is registered on the checker.
    let checker_src = r#"
        #suppress "MyLib/some-rule" "handled elsewhere"
        model Foo {}
        "#;
    let checker_without = check(checker_src);
    assert!(
        checker_without.unused_suppressions().is_empty(),
        "unknown library prefix should not be reported unused"
    );

    let result = crate::parser::parse(checker_src);
    let mut checker = crate::checker::Checker::new();
    checker.set_loaded_library_names(vec!["MyLib".to_string()]);
    checker.set_parse_result(result.root_id, result.builder);
    checker.check_program();
    let unused = checker.unused_suppressions();
    assert_eq!(unused.len(), 1, "unused: {:?}", unused);
    assert_eq!(unused[0].directive.code, "MyLib/some-rule");
}

#[test]
fn test_multiple_suppressions_partial_use() {
    // One of two suppressions is used; only the other is reported unused.
    let checker = check(
        r#"
        #deprecated "Old is deprecated"
        model Old {}

        model Foo {
            #suppress "deprecated" "intentional"
            prop: Old;
        }

        #suppress "deprecated" "never triggered"
        model Bar {}
        "#,
    );
    assert_eq!(count_diagnostics(&checker, "deprecated"), 0);
    let unused = checker.unused_suppressions();
    assert_eq!(unused.len(), 1, "unused: {:?}", unused);
    assert_eq!(unused[0].directive.message, "never triggered");
}

// ============================================================================
// Duplicate suppressions (microsoft/typespec#11113)
// ============================================================================

/// Ported from TS: "warns on duplicate suppressions with a message"
#[test]
fn test_warns_on_duplicate_suppressions_with_message() {
    let checker = check(
        r#"
        #deprecated "Old is deprecated"
        model Old {}

        model Foo {
            #suppress "deprecated" "intentional"
            #suppress "deprecated" "duplicate"
            prop: Old;
        }
        "#,
    );
    let diag = checker
        .diagnostics()
        .iter()
        .find(|d| d.code == "duplicate-suppression");
    assert!(
        diag.is_some(),
        "expected duplicate-suppression warning, got: {:?}",
        checker.diagnostics()
    );
    assert_eq!(
        diag.unwrap().message,
        "Diagnostic \"deprecated\" is already suppressed on this node."
    );
    // The deprecated warning itself is suppressed by the first directive.
    assert!(
        !checker.diagnostics().iter().any(|d| d.code == "deprecated"),
        "deprecated should be suppressed: {:?}",
        checker.diagnostics()
    );
}

/// Ported from TS: "warns on duplicate suppressions without a message"
#[test]
fn test_warns_on_duplicate_suppressions_without_message() {
    let checker = check(
        r#"
        #deprecated "Old is deprecated"
        model Old {}

        model Foo {
            #suppress "deprecated"
            #suppress "deprecated"
            prop: Old;
        }
        "#,
    );
    let count = checker
        .diagnostics()
        .iter()
        .filter(|d| d.code == "duplicate-suppression")
        .count();
    assert_eq!(
        count,
        1,
        "expected exactly one duplicate-suppression warning, got: {:?}",
        checker.diagnostics()
    );
}

/// Different codes on the same node are not duplicates.
#[test]
fn test_no_duplicate_warning_for_different_codes() {
    let checker = check(
        r#"
        model Foo {
            #suppress "some-code" "reason"
            #suppress "other-code" "reason"
            prop: string;
        }
        "#,
    );
    assert!(
        !checker
            .diagnostics()
            .iter()
            .any(|d| d.code == "duplicate-suppression"),
        "different codes should not be duplicates: {:?}",
        checker.diagnostics()
    );
}
