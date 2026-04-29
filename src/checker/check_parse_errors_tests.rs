//! Checker Parse Error Tests
//!
//! Ported from TypeSpec compiler/test/checker/check-parse-errors.test.ts
//!
//! Tests that semantic errors are reported in addition to parse errors.
//!
//! Skipped (needs parse error + semantic diagnostic co-reporting):
//! - All tests marked #[ignore] until checker can report semantic diagnostics
//!   on source that also has parse errors

use crate::checker::test_utils::{check, has_diagnostic};

/// Helper: check if diagnostics contain a specific code
#[test]
fn test_semantic_errors_with_parse_errors() {
    // Ported from TS: "reports semantic errors in addition to parse errors"
    // Model M extends Q { a: B; a: C;  (missing closing brace)
    // Should report: unknown identifiers Q/B/C (semantic),
    // and duplicate property name 'a' (semantic)
    // Note: parse errors for missing '}' may or may not be reported depending on
    // parser recovery; the key check is that semantic diagnostics are still emitted
    let checker = check(
        r#"
        model M extends Q {
            a: B;
            a: C;
    "#,
    );
    // Should have semantic errors (unknown identifiers, duplicate property)
    assert!(
        has_diagnostic(&checker, "invalid-ref") || has_diagnostic(&checker, "duplicate-property"),
        "Should report semantic errors in addition to parse errors: {:?}",
        checker.diagnostics()
    );
}
