//! Auto decorator library feature tests
//!
//! Ported from TypeSpec compiler/test/checker/auto-decorator-library.test.ts
//! (microsoft/typespec#11235): a library enables compiler features for its
//! own code independently of the consuming project.

use crate::checker::Checker;
use crate::diagnostics::CompilerOptions;
use crate::parser::ParseOptions;

const LIB_SOURCE: &str = r#"namespace MyLib;
auto dec myFlag(target: unknown);"#;

/// Check the consumer source with the auto-decorator library injected,
/// optionally with features enabled on the library and/or the project.
fn check_with_library(lib_features: Vec<String>, project_features: Vec<String>) -> Checker {
    let options = ParseOptions::new_with_features(vec![(LIB_SOURCE.to_string(), lib_features)]);
    let result = crate::parser::parse_with_options(
        r#"using MyLib;
@myFlag model Foo {}"#,
        options,
    );
    let compiler_options = CompilerOptions {
        features: project_features,
        ..CompilerOptions::default()
    };
    let mut checker = Checker::with_options(compiler_options);
    checker.set_parse_result(result.root_id, result.builder);
    checker.check_program();
    checker
}

fn has_code(checker: &Checker, code: &str) -> bool {
    checker.diagnostics().iter().any(|d| d.code == code)
}

/// Ported from TS: "library can declare an auto decorator by enabling the
/// feature in its own tspconfig.yaml"
#[test]
fn test_library_feature_enables_auto_decorator() {
    let checker = check_with_library(vec!["auto-decorators".to_string()], Vec::new());
    assert!(
        !has_code(&checker, "auto-decorator-disabled"),
        "library with its own feature enabled should compile: {:?}",
        checker.diagnostics()
    );
}

/// Ported from TS: "library auto decorator still errors when the library
/// does not enable the feature"
#[test]
fn test_library_without_feature_errors() {
    let checker = check_with_library(Vec::new(), Vec::new());
    assert!(
        has_code(&checker, "auto-decorator-disabled"),
        "library without the feature should error: {:?}",
        checker.diagnostics()
    );
}

/// Ported from TS: "feature is scoped per package: enabling it in the
/// consumer project does not enable it for library code"
#[test]
fn test_feature_scoped_per_package() {
    let checker = check_with_library(Vec::new(), vec!["auto-decorators".to_string()]);
    assert!(
        has_code(&checker, "auto-decorator-disabled"),
        "project feature must not enable the feature for library code: {:?}",
        checker.diagnostics()
    );
}
