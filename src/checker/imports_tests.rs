//! Checker Import Tests
//!
//! Ported from TypeSpec compiler/test/checker/imports.test.ts
//!
//! Categories:
//! - import relative TypeSpec file
//! - import relative JS file
//! - import directory with main.tsp
//! - import library with typespec exports
//! - import-not-found diagnostic
//! - import scopes (project vs library)
//!
//! Skipped (needs multi-file compilation support - Tester.files()):
//! - All import tests require multi-file import resolution which is not
//!   yet implemented in the Rust checker. These tests are provided as
//!   a skeleton for future implementation.

use crate::checker::test_utils::{check, has_diagnostic};

// ============================================================================
// import-not-found diagnostic tests - single file, no multi-file needed
// ============================================================================
#[test]
fn test_import_not_found_relative() {
    // Ported from TS: "emit diagnostic when trying to load invalid relative file"
    let checker = check(r#"import "./doesnotexists";"#);
    assert!(
        has_diagnostic(&checker, "import-not-found"),
        "Should report import-not-found for missing relative import: {:?}",
        checker.diagnostics()
    );
}

#[test]
fn test_import_not_found_library() {
    // Ported from TS: "emit diagnostic when trying to load invalid library"
    let checker = check(r#"import "@typespec/doesnotexists";"#);
    assert!(
        has_diagnostic(&checker, "import-not-found"),
        "Should report import-not-found for missing library import: {:?}",
        checker.diagnostics()
    );
}

// ============================================================================
// Multi-file import tests - need Tester.files() equivalent
// ============================================================================

#[test]
fn test_import_relative_typespec_file() {
    // Ported from TS: "import relative TypeSpec file"
    // Needs: b.tsp containing "model B { }"
    // Main file: import "./b.tsp"; model A extends B { }
    // Expected: both files loaded, B available
}

#[test]
fn test_import_relative_js_file() {
    // Ported from TS: "import relative JS file"
    // Needs: blue.js with $blue decorator
    // Main file: import "./blue.js"; @blue model A {}
}

#[test]
fn test_import_directory_with_main() {
    // Ported from TS: "import directory with main.tsp"
    // Needs: test/main.tsp containing "model C { }"
    // Main file: import "./test"; model A { x: C }
}

#[test]
fn test_import_library_with_typespec_exports() {
    // Ported from TS: "import library with typespec exports"
    // Needs: node_modules/my-lib/package.json + main.tsp
}

#[test]
fn test_import_library_with_tspmain() {
    // Ported from TS: "import library(with tspmain)"
    // Needs: node_modules/my-lib/package.json with tspMain field
}

#[test]
fn test_import_scopes_project_files() {
    // Ported from TS: "relative files are stays in project"
    // Needs: location context tracking per source file
}

#[test]
fn test_import_scopes_library_files() {
    // Ported from TS: "importing a library resolve is as its library"
    // Needs: library scope tracking in program
}

// ============================================================================
// Additional import diagnostic tests
// ============================================================================

/// Multiple imports that don't resolve
#[test]
fn test_multiple_import_not_found() {
    let checker = check(
        r#"
        import "./missing1";
        import "./missing2";
    "#,
    );
    let count = checker
        .diagnostics()
        .iter()
        .filter(|d| d.code == "import-not-found")
        .count();
    assert!(
        count >= 1,
        "Should report at least 1 import-not-found for missing imports: {:?}",
        checker.diagnostics()
    );
}

/// Import with valid syntax but missing target
#[test]
fn test_import_not_found_nested_path() {
    let checker = check(r#"import "./foo/bar/baz";"#);
    assert!(
        has_diagnostic(&checker, "import-not-found"),
        "Should report import-not-found for nested path: {:?}",
        checker.diagnostics()
    );
}

/// Import statement without semicolons (TypeSpec allows this)
#[test]
fn test_import_without_semicolon() {
    let checker = check(
        r#"
        import "./missing"
    "#,
    );
    // Should still parse the import and report import-not-found
    assert!(
        has_diagnostic(&checker, "import-not-found"),
        "Should report import-not-found even without semicolon: {:?}",
        checker.diagnostics()
    );
}
