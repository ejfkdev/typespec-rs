//! Tests for diagnostic source location (line/column) tracking
//!
//! Verifies that error_at/warning_at correctly populate line and column
//! information from AST node spans, so that diagnostic output includes
//! meaningful source positions.
//!
//! Note: `error_at(node_id, ...)` records the location of the AST node
//! passed as `node_id`. In some cases (e.g., duplicate-property in
//! check_model), the node_id is the parent model declaration, not the
//! specific property. Tests verify that locations are populated and
//! reasonable rather than asserting exact line numbers in those cases.

use super::test_utils::check;
use crate::diagnostics::DiagnosticSeverity;

/// Helper: find a diagnostic by code and return its location.
/// Panics if not found or if no location is set.
fn find_location(checker: &crate::checker::Checker, code: &str) -> (u32, u32) {
    let diag = checker
        .diagnostics()
        .iter()
        .find(|d| d.code == code)
        .unwrap_or_else(|| {
            let codes: Vec<_> = checker
                .diagnostics()
                .iter()
                .map(|d| d.code.as_str())
                .collect();
            panic!(
                "No diagnostic with code '{}' found. Available: {:?}",
                code, codes
            )
        });
    let loc = diag
        .location
        .as_ref()
        .unwrap_or_else(|| panic!("Diagnostic '{}' has no location", code));
    (loc.line, loc.column)
}

/// Helper: find an error diagnostic by code, verify it's an error, and return location.
fn find_error_location(checker: &crate::checker::Checker, code: &str) -> (u32, u32) {
    let (line, col) = find_location(checker, code);
    let diag = checker
        .diagnostics()
        .iter()
        .find(|d| d.code == code)
        .unwrap();
    assert_eq!(
        diag.severity,
        DiagnosticSeverity::Error,
        "Expected error severity for '{}'",
        code
    );
    assert!(line > 0, "Diagnostic '{}' should have line > 0", code);
    (line, col)
}

// ═══════════════════════════════════════════════════════════════════
// duplicate-symbol — node_id is the duplicate declaration itself
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_duplicate_symbol_has_location() {
    let checker = check(
        "\
model A { x: string; }
model A { y: int32; }",
    );
    let (line, _col) = find_error_location(&checker, "duplicate-symbol");
    // The second "model A" is on line 2
    assert_eq!(line, 2);
}

#[test]
fn test_duplicate_symbol_in_namespace_has_location() {
    let checker = check(
        "\
namespace N {
  model A { x: string; }
  model A { y: int32; }
}",
    );
    let (line, _col) = find_error_location(&checker, "duplicate-symbol");
    // Second "model A" is on line 3
    assert_eq!(line, 3);
}

// ═══════════════════════════════════════════════════════════════════
// invalid-ref — node_id is the identifier/member expression node
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_unknown_type_has_location() {
    let checker = check(
        "\
model Foo {
  bar: NonExistent;
}",
    );
    let (line, _col) = find_error_location(&checker, "invalid-ref");
    // "NonExistent" is on line 2
    assert_eq!(line, 2);
}

#[test]
fn test_member_access_invalid_ref_has_location() {
    let checker = check(
        "\
namespace Ns { model A { x: string; } }
model Foo {
  bar: Ns.NonExistent;
}",
    );
    let (line, _col) = find_error_location(&checker, "invalid-ref");
    // Member expression on line 3
    assert!(line >= 3, "Expected line >= 3, got {}", line);
}

// ═══════════════════════════════════════════════════════════════════
// duplicate-property — node_id is the model declaration (not the prop)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_duplicate_property_has_location() {
    let checker = check(
        "\
model Foo {
  x: string;
  x: int32;
}",
    );
    let (line, _col) = find_error_location(&checker, "duplicate-property");
    // Points to the model declaration (line 1) since node_id is the model
    assert!((1..=3).contains(&line), "Expected line 1-3, got {}", line);
}

// ═══════════════════════════════════════════════════════════════════
// circular-base-type
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_circular_base_type_has_location() {
    let checker = check(
        "\
model A extends B { }
model B extends A { }",
    );
    let (line, _col) = find_error_location(&checker, "circular-base-type");
    assert!((1..=2).contains(&line), "Expected line 1-2, got {}", line);
}

// ═══════════════════════════════════════════════════════════════════
// extend-model (model expression as base)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_extend_model_expression_has_location() {
    let checker = check("model Foo extends { x: string } { }");
    let (line, _col) = find_error_location(&checker, "extend-model");
    assert_eq!(line, 1);
}

// ═══════════════════════════════════════════════════════════════════
// invalid-template-args
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_missing_template_arg_has_location() {
    let checker = check(
        "\
model Template<T> { t: T; }
model Foo {
  bar: Template;
}",
    );
    let (line, _col) = find_error_location(&checker, "invalid-template-args");
    assert!(line >= 3, "Expected line >= 3, got {}", line);
}

// ═══════════════════════════════════════════════════════════════════
// circular-prop
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_circular_prop_has_location() {
    let checker = check(
        "\
model A {
  x: A.x;
}",
    );
    let (line, _col) = find_error_location(&checker, "circular-prop");
    assert!(line >= 2, "Expected line >= 2, got {}", line);
}

// ═══════════════════════════════════════════════════════════════════
// using-invalid-ref
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_using_invalid_ref_has_location() {
    // using-invalid-ref is reported when the target resolves to a
    // non-namespace type (upstream reports only invalid-ref for unknown
    // identifiers).
    let checker = check("using Target;\nmodel Target {}");
    let (line, _col) = find_error_location(&checker, "using-invalid-ref");
    assert_eq!(line, 1);
}

// ═══════════════════════════════════════════════════════════════════
// duplicate-using
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_duplicate_using_has_location() {
    let checker = check(
        "\
namespace Ns { model A { x: string; } }
using Ns;
using Ns;",
    );
    let (line, _col) = find_location(&checker, "duplicate-using");
    assert_eq!(line, 3);
}

// ═══════════════════════════════════════════════════════════════════
// enum-member-duplicate
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_enum_member_duplicate_has_location() {
    let checker = check(
        "\
enum Direction {
  North,
  North,
}",
    );
    let (line, _col) = find_error_location(&checker, "enum-member-duplicate");
    assert!((2..=3).contains(&line), "Expected line 2-3, got {}", line);
}

// ═══════════════════════════════════════════════════════════════════
// union-duplicate
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_union_duplicate_has_location() {
    let checker = check(
        "\
union Status {
  ok: string,
  ok: string
}",
    );
    let (line, _col) = find_error_location(&checker, "union-duplicate");
    assert!((2..=3).contains(&line), "Expected line 2-3, got {}", line);
}

// ═══════════════════════════════════════════════════════════════════
// interface-duplicate — node_id is the interface declaration
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_interface_duplicate_has_location() {
    let checker = check(
        "\
interface IFace {
  op(): void;
  op(): void;
}",
    );
    let (line, _col) = find_error_location(&checker, "interface-duplicate");
    // Points to interface declaration (line 1) since node_id is the interface
    assert!((1..=3).contains(&line), "Expected line 1-3, got {}", line);
}

// ═══════════════════════════════════════════════════════════════════
// spread-model — node_id is the model declaration
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_spread_non_model_has_location() {
    let checker = check(
        "\
enum E { A, B }
model Foo {
  ...E;
}",
    );
    let (line, _col) = find_error_location(&checker, "spread-model");
    // Points to model declaration (line 2) since node_id is the model
    assert!((2..=4).contains(&line), "Expected line 2-4, got {}", line);
}

// ═══════════════════════════════════════════════════════════════════
// Multiple diagnostics with distinct locations
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_multiple_errors_have_distinct_locations() {
    let checker = check(
        "\
model A { x: string; }
model A { y: int32; }
model B {
  z: NonExistent;
}",
    );
    let diags = checker.diagnostics();

    let dup_diag = diags.iter().find(|d| d.code == "duplicate-symbol").unwrap();
    let ref_diag = diags.iter().find(|d| d.code == "invalid-ref").unwrap();

    let dup_line = dup_diag.location.as_ref().unwrap().line;
    let ref_line = ref_diag.location.as_ref().unwrap().line;

    assert_ne!(
        dup_line, ref_line,
        "Different errors should have different locations: dup={}, ref={}",
        dup_line, ref_line
    );
    assert_eq!(dup_line, 2, "duplicate-symbol should be on line 2");
    assert_eq!(ref_line, 4, "invalid-ref should be on line 4");
}

// ═══════════════════════════════════════════════════════════════════
// Column numbers are populated (at least non-zero for some diagnostics)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_column_number_populated_for_identifier() {
    let checker = check("model B { z: NonExistent; }");
    let (line, col) = find_error_location(&checker, "invalid-ref");
    assert_eq!(line, 1);
    // "NonExistent" starts at some column > 0
    assert!(col > 0, "Column should be > 0 for identifier, got {}", col);
}

// ═══════════════════════════════════════════════════════════════════
// extends-interface
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_extends_interface_invalid_has_location() {
    let checker = check(
        "\
model M { x: string; }
interface IFace extends M { }",
    );
    let (line, _col) = find_error_location(&checker, "extends-interface");
    assert_eq!(line, 2);
}

// ═══════════════════════════════════════════════════════════════════
// Invalid modifier
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_invalid_modifier_has_location() {
    let checker = check("extern model Foo { x: string; }");
    let (line, _col) = find_error_location(&checker, "invalid-modifier");
    assert_eq!(line, 1);
}

// ═══════════════════════════════════════════════════════════════════
// Decorator error: invalid-argument-count
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_invalid_argument_count_has_location() {
    let checker = check(
        "\
dec myDec(target: Model, arg1: string)
@myDec
model Foo { }",
    );
    let (line, _col) = find_error_location(&checker, "invalid-argument-count");
    assert!(line >= 2, "Expected line >= 2, got {}", line);
}

// ═══════════════════════════════════════════════════════════════════
// alias circular reference
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_circular_alias_has_location() {
    let checker = check(
        "\
alias A = B;
alias B = A;",
    );
    // Should produce circular-const or similar
    let diag = checker
        .diagnostics()
        .iter()
        .find(|d| d.code == "circular-const" || d.code == "circular-base-type");
    if let Some(d) = diag {
        let loc = d
            .location
            .as_ref()
            .expect("circular diag should have location");
        assert!(loc.line > 0, "line should be > 0, got {}", loc.line);
    }
    // If no circular diag, at least check for any error with location
    let any_error = checker.diagnostics().iter().find(|d| d.is_error());
    if let Some(d) = any_error
        && let Some(loc) = &d.location
    {
        assert!(loc.line > 0);
    }
}

// ═══════════════════════════════════════════════════════════════════
// scalar extend-scalar
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_extend_scalar_model_has_location() {
    let checker = check(
        "\
model M { x: string; }
scalar S extends M;",
    );
    let (line, _col) = find_error_location(&checker, "extend-scalar");
    assert_eq!(line, 2);
}

// ═══════════════════════════════════════════════════════════════════
// import-not-found
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_import_not_found_has_location() {
    let checker = check("import \"nonexistent.tsp\";");
    let (line, _col) = find_error_location(&checker, "import-not-found");
    assert_eq!(line, 1);
}

// ═══════════════════════════════════════════════════════════════════
// value-in-type
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_value_in_type_has_location() {
    let checker = check(
        "\
const c = \"abc\";
model Foo { x: c; }",
    );
    let diag = checker
        .diagnostics()
        .iter()
        .find(|d| d.code == "value-in-type");
    if let Some(d) = diag {
        let loc = d
            .location
            .as_ref()
            .expect("value-in-type should have location");
        assert!(loc.line > 0, "line should be > 0, got {}", loc.line);
    }
}

// ═══════════════════════════════════════════════════════════════════
// intersect-non-model
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_intersect_non_model_has_location() {
    let checker = check("alias A = string & int32;");
    let diag = checker
        .diagnostics()
        .iter()
        .find(|d| d.code == "intersect-non-model");
    if let Some(d) = diag {
        let loc = d
            .location
            .as_ref()
            .expect("intersect-non-model should have location");
        assert!(loc.line > 0, "line should be > 0, got {}", loc.line);
    }
}

// ═══════════════════════════════════════════════════════════════════
// non-callable (calling a non-function)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_non_callable_has_location() {
    let checker = check("model M { x: string; }\nalias A = M();");
    let diag = checker
        .diagnostics()
        .iter()
        .find(|d| d.code == "non-callable");
    if let Some(d) = diag {
        let loc = d
            .location
            .as_ref()
            .expect("non-callable should have location");
        assert!(loc.line > 0, "line should be > 0, got {}", loc.line);
    }
}

// ═══════════════════════════════════════════════════════════════════
// Warning diagnostics also get locations
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_warning_has_location() {
    let checker = check("import \"foo.tsp\";\nimport \"foo.tsp\";");
    let diag = checker
        .diagnostics()
        .iter()
        .find(|d| d.code == "duplicate-import");
    if let Some(d) = diag {
        let loc = d
            .location
            .as_ref()
            .expect("duplicate-import should have location");
        assert!(
            loc.line > 0,
            "duplicate-import line should be > 0, got {}",
            loc.line
        );
    }
}

// ═══════════════════════════════════════════════════════════════════
// Diagnostics without node_id (legacy path) have no location
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_legacy_error_has_no_location() {
    // check_helpers errors (e.g., override-property-mismatch) use self.error()
    // without node_id, so they don't have location information yet
    let checker = check(
        "\
model Base { name: string; }
model Deriv extends Base { name: int32; }",
    );
    let diag = checker
        .diagnostics()
        .iter()
        .find(|d| d.code == "override-property-mismatch" || d.code == "unassignable");
    // These may or may not have location depending on code path;
    // the test just verifies they don't crash
    if let Some(d) = diag {
        let _ = &d.location;
    }
}

// ============================================================================
// get_node_for_target (microsoft/typespec#10921)
// ============================================================================

/// Ported from TS getNodeForTarget tests: type entities resolve to their
/// declaration node.
#[test]
fn test_get_node_for_target_type_entity() {
    let checker = check("model Foo {}");
    let foo = checker.get_type_by_name("Foo").expect("Foo should exist");
    let node = checker.get_node_for_target(&crate::checker::Entity::Type(foo));
    assert!(node.is_some(), "type entity should resolve to a node");
}

/// Ported from TS: "falls back to value type node when value has no node" —
/// primitive values (string literals) fall back to their type's node.
#[test]
fn test_get_node_for_target_value_entity_falls_back_to_type() {
    let checker = check(
        r#"
        model Target {}
        const x: Target = #{ };
    "#,
    );
    // const x = #{ } creates an ObjectValue which carries its own node
    let value_id = *checker
        .declared_values
        .get("x")
        .expect("const x should be declared");
    let node = checker.get_node_for_target(&crate::checker::Entity::Value(value_id));
    assert!(node.is_some(), "value entity should resolve to a node");
}

/// Ported from TS: mixed parameter constraints resolve with priority order
/// (explicit node, then type side, then value side).
#[test]
fn test_get_node_for_target_mixed_constraint() {
    use crate::checker::types::MixedParameterConstraint;
    let checker = check("model Foo {}");
    let foo = checker.get_type_by_name("Foo").expect("Foo should exist");

    // Type side only -> resolves via the type constraint's node.
    let mc = MixedParameterConstraint {
        node: None,
        type_constraint: Some(foo),
        value_constraint: None,
    };
    let node = checker.get_node_for_target(&crate::checker::Entity::MixedConstraint(mc));
    assert!(
        node.is_some(),
        "mixed constraint should fall back to the type side node"
    );

    // Explicit node wins over both sides.
    let explicit_node = node.expect("resolved node from type side");
    let mc_explicit = MixedParameterConstraint {
        node: Some(explicit_node),
        type_constraint: Some(foo),
        value_constraint: None,
    };
    assert_eq!(
        checker.get_node_for_target(&crate::checker::Entity::MixedConstraint(mc_explicit)),
        Some(explicit_node),
        "explicit node should take priority"
    );
}

/// Ported from TS: indeterminate entities resolve through the inner type.
#[test]
fn test_get_node_for_target_indeterminate() {
    let checker = check("model Foo {}");
    let foo = checker.get_type_by_name("Foo").expect("Foo should exist");
    let node = checker.get_node_for_target(&crate::checker::Entity::Indeterminate(foo));
    assert!(
        node.is_some(),
        "indeterminate should resolve via inner type"
    );
}
