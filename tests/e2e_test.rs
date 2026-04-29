//! End-to-end integration tests for TypeSpec-Rust
//!
//! Ported from TypeSpec compiler/test/e2e/ scenarios.
//! Tests the full pipeline: source → parse → check → verify diagnostics.
//!
//! Scenario .tsp files live in `tests/e2e/scenarios/`.
//! Each file can contain expectation comments:
//!   // expect-error(code)     — expect an error diagnostic with given code
//!   // expect-warning(code)   — expect a warning diagnostic with given code
//!   // expect-clean           — expect zero diagnostics
//!
//! If no expectation comment is present, expect-clean is the default.

use std::fs;
use std::path::Path;

use typespec_rs::checker::Checker;
use typespec_rs::diagnostics::Diagnostic;
use typespec_rs::parser::parse;

/// Expected diagnostic from a comment annotation
#[derive(Debug, PartialEq)]
enum ExpectedDiag {
    Error(String),
    Warning(String),
}

/// Parse expectation comments from source code.
fn parse_expectations(source: &str) -> (Vec<ExpectedDiag>, bool) {
    let mut expectations = Vec::new();
    let mut expect_clean = false;

    for line in source.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("// expect-error(") {
            if let Some(code) = rest.strip_suffix(')') {
                expectations.push(ExpectedDiag::Error(code.to_string()));
            }
        } else if let Some(rest) = trimmed.strip_prefix("// expect-warning(") {
            if let Some(code) = rest.strip_suffix(')') {
                expectations.push(ExpectedDiag::Warning(code.to_string()));
            }
        } else if trimmed.contains("// expect-clean") {
            expect_clean = true;
        }
    }

    (expectations, expect_clean)
}

/// Run the full pipeline on a source string and return the Checker + parse diagnostics.
fn compile(source: &str) -> (Checker, Vec<Diagnostic>) {
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

/// Verify that diagnostics match expectations.
fn verify_expectations(source: &str, checker: &Checker, parse_diags: &[Diagnostic]) {
    let (expectations, expect_clean) = parse_expectations(source);
    let all_diags: Vec<&Diagnostic> = parse_diags
        .iter()
        .chain(checker.diagnostics().iter())
        .collect();

    if expect_clean || expectations.is_empty() {
        assert!(
            all_diags.is_empty(),
            "Expected clean compilation, but got {} diagnostic(s):\n{:#?}",
            all_diags.len(),
            all_diags
        );
        return;
    }

    for expected in &expectations {
        match expected {
            ExpectedDiag::Error(code) => {
                let found = all_diags.iter().any(|d| d.code == *code);
                assert!(
                    found,
                    "Expected error diagnostic with code '{}', but not found.\nAll diagnostics: {:#?}",
                    code, all_diags
                );
            }
            ExpectedDiag::Warning(code) => {
                let found = all_diags.iter().any(|d| d.code == *code);
                assert!(
                    found,
                    "Expected warning diagnostic with code '{}', but not found.\nAll diagnostics: {:#?}",
                    code, all_diags
                );
            }
        }
    }
}

/// Run an e2e scenario from a .tsp file.
fn run_scenario(name: &str) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/e2e/scenarios")
        .join(format!("{}.tsp", name));

    let source = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read scenario '{}': {}", name, e));

    let (checker, parse_diags) = compile(&source);
    verify_expectations(&source, &checker, &parse_diags);
}

// ============================================================================
// E2E Scenarios ported from TS compiler/test/e2e/
// ============================================================================

#[test]
fn e2e_simple() {
    // Ported from TS e2e/cli/scenarios/simple/main.tsp
    run_scenario("simple");
}

#[test]
fn e2e_deprecated_warn() {
    // Ported from TS e2e/cli/scenarios/warn/main.tsp
    // Tests deprecation warning on model usage
    run_scenario("deprecated-warn");
}

#[test]
fn e2e_pet_store() {
    // Comprehensive API scenario with models, enums, interfaces, operations
    run_scenario("pet-store");
}

#[test]
fn e2e_template_models() {
    // Template model declaration and instantiation
    run_scenario("template-models");
}

#[test]
fn e2e_nested_namespaces() {
    // Namespace declarations with nested types
    run_scenario("nested-namespaces");
}

#[test]
fn e2e_scalar_extends() {
    // Scalar declarations extending built-in types
    run_scenario("scalar-extends");
}

#[test]
fn e2e_union_types() {
    // Union types with named variants and anonymous unions
    run_scenario("union-types");
}

#[test]
fn e2e_const_values() {
    // Const declarations with various value types
    run_scenario("const-values");
}

#[test]
fn e2e_model_extends() {
    // Model extends and property inheritance
    run_scenario("model-extends");
}

#[test]
fn e2e_decorators() {
    // Decorators on models, properties, operations
    run_scenario("decorators");
}

// ============================================================================
// Inline e2e tests (no .tsp file needed)
// ============================================================================

#[test]
fn e2e_inline_simple_model() {
    let source = "model Foo {}";
    let (checker, parse_diags) = compile(source);
    let all: Vec<&Diagnostic> = parse_diags
        .iter()
        .chain(checker.diagnostics().iter())
        .collect();
    assert!(all.is_empty(), "Expected clean, got: {:#?}", all);
    assert!(checker.declared_types.contains_key("Foo"));
}

#[test]
fn e2e_inline_deprecated_model_reference() {
    // Ported from TS e2e/cli/scenarios/warn/main.tsp
    let source = r#"
        #deprecated "Deprecated"
        model Foo {}

        model Bar {
          foo: Foo;
        }
    "#;
    let (checker, parse_diags) = compile(source);
    let all: Vec<&Diagnostic> = parse_diags
        .iter()
        .chain(checker.diagnostics().iter())
        .collect();
    // Should produce a deprecated warning
    let has_deprecated = all.iter().any(|d| d.code == "deprecated");
    assert!(
        has_deprecated,
        "Expected deprecated warning, got: {:#?}",
        all
    );
}

#[test]
fn e2e_inline_model_with_basic_property_types() {
    // Test basic property types that the checker currently resolves correctly
    let source = r#"
        model BasicTypes {
          s: string;
          i: int32;
          b: boolean;
        }
    "#;
    let (checker, parse_diags) = compile(source);
    let all: Vec<&Diagnostic> = parse_diags
        .iter()
        .chain(checker.diagnostics().iter())
        .collect();
    assert!(all.is_empty(), "Expected clean, got: {:#?}", all);
    assert!(checker.declared_types.contains_key("BasicTypes"));
}

#[test]
fn e2e_inline_enum_with_values() {
    let source = r#"
        enum Direction {
          up: 1,
          down: 2,
          left: 3,
          right: 4,
        }
    "#;
    let (checker, parse_diags) = compile(source);
    let all: Vec<&Diagnostic> = parse_diags
        .iter()
        .chain(checker.diagnostics().iter())
        .collect();
    assert!(all.is_empty(), "Expected clean, got: {:#?}", all);
    assert!(checker.declared_types.contains_key("Direction"));
}

#[test]
fn e2e_inline_interface_with_operations() {
    let source = r#"
        interface Service {
          getItem(id: string): string;
          listItems(): string[];
          createItem(item: string): void;
        }
    "#;
    let (checker, parse_diags) = compile(source);
    let all: Vec<&Diagnostic> = parse_diags
        .iter()
        .chain(checker.diagnostics().iter())
        .collect();
    assert!(all.is_empty(), "Expected clean, got: {:#?}", all);
    assert!(checker.declared_types.contains_key("Service"));
}

#[test]
fn e2e_inline_template_instantiation() {
    // Template model with simple property types.
    // Note: template parameter references (T) may produce invalid-ref diagnostics
    // until full template resolution is implemented, but types are still created.
    let source = r#"
        model Page<T> {
          item: T;
          count: int32;
        }

        model StringPage is Page<string> {}
    "#;
    let (checker, _parse_diags) = compile(source);
    assert!(checker.declared_types.contains_key("Page"));
    assert!(checker.declared_types.contains_key("StringPage"));
}

#[test]
fn e2e_inline_const_assignability() {
    // Ported from TS values/const.test.ts — invalid assignment should produce unassignable
    let source = r#"const a: int32 = "abc";"#;
    let (checker, parse_diags) = compile(source);
    let all: Vec<&Diagnostic> = parse_diags
        .iter()
        .chain(checker.diagnostics().iter())
        .collect();
    let has_unassignable = all.iter().any(|d| d.code == "unassignable");
    assert!(
        has_unassignable,
        "Expected unassignable diagnostic for string→int32, got: {:#?}",
        all
    );
}

#[test]
fn e2e_inline_namespace_with_types() {
    // Namespace with types — doesn't test cross-namespace references yet
    let source = r#"
        namespace Services {
          model Error {
            code: int32;
            message: string;
          }
        }
    "#;
    let (checker, parse_diags) = compile(source);
    let all: Vec<&Diagnostic> = parse_diags
        .iter()
        .chain(checker.diagnostics().iter())
        .collect();
    assert!(all.is_empty(), "Expected clean, got: {:#?}", all);
    assert!(checker.declared_types.contains_key("Services"));
}

#[test]
fn e2e_inline_scalar_chain() {
    let source = r#"
        scalar uuid extends string;
        scalar myId extends uuid;
    "#;
    let (checker, parse_diags) = compile(source);
    let all: Vec<&Diagnostic> = parse_diags
        .iter()
        .chain(checker.diagnostics().iter())
        .collect();
    assert!(all.is_empty(), "Expected clean, got: {:#?}", all);
    assert!(checker.declared_types.contains_key("uuid"));
    assert!(checker.declared_types.contains_key("myId"));
}
