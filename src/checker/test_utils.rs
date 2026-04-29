//! Shared test utilities for checker tests
//!
//! This module provides common helper functions used across all checker test
//! files, eliminating the previous pattern of duplicating identical `check()`,
//! `has_diagnostic()`, etc. functions in every test file.

use crate::checker::Checker;
use crate::diagnostics::Diagnostic;
use crate::parser::parse;

/// Parse source code, run the checker, and return the Checker instance.
///
/// This is the most common helper across all checker test files.
pub fn check(source: &str) -> Checker {
    let result = parse(source);
    let mut checker = Checker::new();
    checker.set_parse_result(result.root_id, result.builder);
    checker.check_program();
    checker
}

/// Parse source code, run the checker, and return both the Checker
/// and any parse diagnostics converted to `Diagnostic` instances.
pub fn check_with_parse_diagnostics(source: &str) -> (Checker, Vec<Diagnostic>) {
    let result = parse(source);
    let parse_diags: Vec<Diagnostic> = result
        .diagnostics
        .iter()
        .map(|pd| Diagnostic::error(pd.code, &pd.message))
        .collect();
    let mut checker = Checker::new();
    checker.set_parse_result(result.root_id, result.builder);
    checker.check_program();
    (checker, parse_diags)
}

/// Get all diagnostics (parse + checker) as a single Vec.
pub fn all_diagnostics(source: &str) -> Vec<Diagnostic> {
    let (checker, mut parse_diags) = check_with_parse_diagnostics(source);
    parse_diags.extend(checker.diagnostics().to_vec());
    parse_diags
}

/// Check if the checker produced any diagnostic with the given code.
pub fn has_diagnostic(checker: &Checker, code: &str) -> bool {
    checker.diagnostics().iter().any(|d| d.code == code)
}

/// Count how many diagnostics with the given code the checker produced.
pub fn count_diagnostics(checker: &Checker, code: &str) -> usize {
    checker
        .diagnostics()
        .iter()
        .filter(|d| d.code == code)
        .count()
}

/// Assert that the checker has at least `count` diagnostics with the given code.
pub fn assert_diag_at_least(checker: &Checker, code: &str, count: usize) {
    let actual = count_diagnostics(checker, code);
    assert!(
        actual >= count,
        "Expected at least {} diagnostic(s) with code '{}', but got {}. Actual diagnostics: {:?}",
        count,
        code,
        actual,
        checker.diagnostics()
    );
}

/// Assert that the checker has no diagnostics with the given code.
pub fn assert_no_diag(checker: &Checker, code: &str) {
    let actual = count_diagnostics(checker, code);
    assert_eq!(
        actual,
        0,
        "Expected no diagnostic(s) with code '{}', but got {}. Actual diagnostics: {:?}",
        code,
        actual,
        checker.diagnostics()
    );
}
