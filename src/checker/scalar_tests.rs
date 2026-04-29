//! Checker Scalar Tests
//!
//! Ported from TypeSpec compiler/test/checker/scalar.test.ts
//!
//! Skipped (needs decorator execution):
//! - Template parameters with @doc decorator
//!
//! Skipped (needs diagnostics system):
//! - Default value type mismatch

use crate::checker::Type;
use crate::checker::test_utils::check;

#[test]
fn test_declare_simple_scalar() {
    // Ported from: "declare simple scalar"
    let checker = check("scalar A;");
    let a_type = checker.declared_types.get("A").copied().unwrap();
    let t = checker.get_type(a_type).cloned().unwrap();
    match t {
        Type::Scalar(s) => {
            assert_eq!(s.name, "A");
            assert!(s.base_scalar.is_none(), "Simple scalar should have no base");
        }
        _ => panic!("Expected Scalar type, got {:?}", t.kind_name()),
    }
}

#[test]
fn test_declare_scalar_extending_another() {
    // Ported from: "declare simple scalar extending another"
    let checker = check("scalar A extends numeric;");
    let a_type = checker.declared_types.get("A").copied().unwrap();
    let t = checker.get_type(a_type).cloned().unwrap();
    match t {
        Type::Scalar(s) => {
            assert_eq!(s.name, "A");
            assert!(s.base_scalar.is_some(), "A should have base_scalar");

            // Verify base_scalar resolves to the std "numeric" type
            let base_id = s.base_scalar.unwrap();
            let base = checker.get_type(base_id).cloned().unwrap();
            assert!(
                matches!(base, Type::Scalar(ref bs) if bs.name == "numeric"),
                "Base scalar should be 'numeric', got {:?}",
                base.kind_name()
            );
        }
        _ => panic!("Expected Scalar type, got {:?}", t.kind_name()),
    }
}

#[test]
fn test_declare_scalar_extending_string() {
    let checker = check("scalar email extends string;");
    let email_type = checker.declared_types.get("email").copied().unwrap();
    let t = checker.get_type(email_type).cloned().unwrap();
    match t {
        Type::Scalar(s) => {
            assert_eq!(s.name, "email");
            let base_id = s.base_scalar.unwrap();
            let base = checker.get_type(base_id).cloned().unwrap();
            assert!(matches!(base, Type::Scalar(ref bs) if bs.name == "string"));
        }
        _ => panic!("Expected Scalar type"),
    }
}

#[test]
fn test_declare_scalar_extending_int32() {
    let checker = check("scalar positive_int extends int32;");
    let s_type = checker.declared_types.get("positive_int").copied().unwrap();
    let t = checker.get_type(s_type).cloned().unwrap();
    match t {
        Type::Scalar(s) => {
            assert_eq!(s.name, "positive_int");
            let base_id = s.base_scalar.unwrap();
            let base = checker.get_type(base_id).cloned().unwrap();
            assert!(matches!(base, Type::Scalar(ref bs) if bs.name == "int32"));
        }
        _ => panic!("Expected Scalar type"),
    }
}

#[test]
fn test_scalar_with_decorator() {
    let checker = check("@tag scalar myScalar extends string;");
    let s_type = checker.declared_types.get("myScalar").copied().unwrap();
    let t = checker.get_type(s_type).cloned().unwrap();
    match t {
        Type::Scalar(s) => {
            assert_eq!(s.decorators.len(), 1);
        }
        _ => panic!("Expected Scalar type"),
    }
}

#[test]
fn test_scalar_is_finished() {
    let checker = check("scalar A extends string;");
    let a_type = checker.declared_types.get("A").copied().unwrap();
    let t = checker.get_type(a_type).cloned().unwrap();
    assert!(
        t.is_finished(),
        "Scalar type should be finished after checking"
    );
}

#[test]
fn test_multiple_scalars() {
    let checker = check(
        "
        scalar email extends string;
        scalar uuid extends string;
        scalar positive_int extends int32;
    ",
    );
    assert!(checker.declared_types.contains_key("email"));
    assert!(checker.declared_types.contains_key("uuid"));
    assert!(checker.declared_types.contains_key("positive_int"));
}

// ============================================================================
// Scalar Diagnostic Tests
// ============================================================================

#[test]
fn test_scalar_extends_non_scalar_detected() {
    // Ported from: "Scalar must extend other scalars"
    let checker = check("scalar A extends SomeModel;");
    let diags = checker.diagnostics();
    // SomeModel doesn't exist, so we get invalid-ref. But if it were a model,
    // we'd get extend-scalar. Test with a model that exists:
    assert!(
        diags
            .iter()
            .any(|d| d.code == "extend-scalar" || d.code == "invalid-ref"),
        "Should report extend-scalar or invalid-ref: {:?}",
        diags
    );
}

#[test]
fn test_scalar_extends_model_detected() {
    // A scalar extending a model should report extend-scalar
    let checker = check("model Bar {} scalar A extends Bar;");
    let diags = checker.diagnostics();
    assert!(
        diags.iter().any(|d| d.code == "extend-scalar"),
        "Should report extend-scalar when scalar extends model: {:?}",
        diags
    );
}

#[test]
fn test_scalar_circular_extends_self() {
    // Ported from: "reference itself"
    let checker = check("scalar a extends a;");
    let diags = checker.diagnostics();
    assert!(
        diags.iter().any(|d| d.code == "circular-base-type"),
        "Should report circular-base-type: {:?}",
        diags
    );
}

#[test]
fn test_scalar_circular_extends_via_another() {
    // Ported from: "reference itself via another scalar"
    let checker = check(
        "
        scalar a extends b;
        scalar b extends a;
    ",
    );
    let diags = checker.diagnostics();
    assert!(
        diags.iter().any(|d| d.code == "circular-base-type"),
        "Should report circular-base-type: {:?}",
        diags
    );
}

#[test]
fn test_no_error_scalar_extends_valid_scalar() {
    // No diagnostic when extending a valid scalar
    let checker = check("scalar A extends string;");
    let diags = checker.diagnostics();
    assert!(diags.is_empty(), "Should have no diagnostics: {:?}", diags);
}

#[test]
fn test_scalar_circular_extends_via_alias_message() {
    // Verify message content for circular via alias
    let checker = check(
        "
        scalar a extends b;
        alias b = a;
    ",
    );
    let diags = checker.diagnostics();
    let circ_diag = diags
        .iter()
        .find(|d| d.code == "circular-base-type")
        .unwrap();
    assert!(
        circ_diag.message.contains("a"),
        "Message should mention type 'a': {}",
        circ_diag.message
    );
}

#[test]
fn test_scalar_circular_extends_via_alias() {
    // Ported from: "reference itself via an alias"
    let checker = check(
        "
        scalar a extends b;
        alias b = a;
    ",
    );
    let diags = checker.diagnostics();
    assert!(
        diags.iter().any(|d| d.code == "circular-base-type"),
        "Should report circular-base-type: {:?}",
        diags
    );
}

#[test]
fn test_scalar_circular_self_extends_message() {
    // Verify message content for circular self-extends
    let checker = check("scalar a extends a;");
    let diags = checker.diagnostics();
    let circ_diag = diags
        .iter()
        .find(|d| d.code == "circular-base-type")
        .unwrap();
    assert!(
        circ_diag.message.contains("a"),
        "Message should mention type 'a': {}",
        circ_diag.message
    );
}

#[test]
fn test_scalar_simple_declaration_no_base() {
    // A scalar with no extends clause
    let checker = check("scalar MyScalar;");
    let s_type = checker.declared_types.get("MyScalar").copied().unwrap();
    let t = checker.get_type(s_type).cloned().unwrap();
    match t {
        Type::Scalar(s) => {
            assert_eq!(s.name, "MyScalar");
            assert!(s.base_scalar.is_none(), "Simple scalar should have no base");
        }
        _ => panic!("Expected Scalar type"),
    }
}

#[test]
fn test_scalar_extends_numeric() {
    // Ported from: "declare simple scalar extending another" with numeric
    let checker = check("scalar MyNum extends numeric;");
    let s_type = checker.declared_types.get("MyNum").copied().unwrap();
    let t = checker.get_type(s_type).cloned().unwrap();
    match t {
        Type::Scalar(s) => {
            assert_eq!(s.name, "MyNum");
            assert!(s.base_scalar.is_some());
            let base_id = s.base_scalar.unwrap();
            let base = checker.get_type(base_id).cloned().unwrap();
            assert!(
                matches!(base, Type::Scalar(ref bs) if bs.name == "numeric"),
                "Base scalar should be 'numeric', got {:?}",
                base.kind_name()
            );
        }
        _ => panic!("Expected Scalar type"),
    }
}

#[test]
fn test_scalar_in_namespace() {
    let checker = check("namespace MyNs { scalar S extends string; }");
    let ns_type = checker.declared_types.get("MyNs").copied().unwrap();
    let t = checker.get_type(ns_type).cloned().unwrap();
    match t {
        Type::Namespace(ns) => {
            assert!(
                ns.scalars.contains_key("S"),
                "Namespace should contain S scalar"
            );
        }
        _ => panic!("Expected Namespace type"),
    }
}

#[test]
fn test_scalar_simple_no_base_is_finished() {
    let checker = check("scalar A;");
    let a_type = checker.declared_types.get("A").copied().unwrap();
    let t = checker.get_type(a_type).cloned().unwrap();
    assert!(t.is_finished(), "Simple scalar type should be finished");
}

#[test]
fn test_scalar_with_decorator_in_namespace() {
    let checker = check("namespace N { @tag scalar S extends string; }");
    let ns_type = checker.declared_types.get("N").copied().unwrap();
    let t = checker.get_type(ns_type).cloned().unwrap();
    match t {
        Type::Namespace(ns) => {
            let s_id = ns.scalars.get("S").copied().unwrap();
            let s = checker.get_type(s_id).cloned().unwrap();
            match s {
                Type::Scalar(scalar) => {
                    assert_eq!(scalar.decorators.len(), 1, "S should have 1 decorator");
                }
                _ => panic!("Expected Scalar"),
            }
        }
        _ => panic!("Expected Namespace type"),
    }
}

#[test]
fn test_scalar_extends_boolean() {
    let checker = check("scalar MyBool extends boolean;");
    let s_type = checker.declared_types.get("MyBool").copied().unwrap();
    let t = checker.get_type(s_type).cloned().unwrap();
    match t {
        Type::Scalar(s) => {
            assert_eq!(s.name, "MyBool");
            let base_id = s.base_scalar.unwrap();
            let base = checker.get_type(base_id).cloned().unwrap();
            assert!(matches!(base, Type::Scalar(ref bs) if bs.name == "boolean"));
        }
        _ => panic!("Expected Scalar type"),
    }
}

// ============================================================================
// Additional Scalar Tests
// ============================================================================

#[test]
fn test_scalar_extends_float64() {
    let checker = check("scalar Price extends float64;");
    let s_type = checker.declared_types.get("Price").copied().unwrap();
    let t = checker.get_type(s_type).cloned().unwrap();
    match t {
        Type::Scalar(s) => {
            assert_eq!(s.name, "Price");
            let base_id = s.base_scalar.unwrap();
            let base = checker.get_type(base_id).cloned().unwrap();
            assert!(matches!(base, Type::Scalar(ref bs) if bs.name == "float64"));
        }
        _ => panic!("Expected Scalar type"),
    }
}

#[test]
fn test_scalar_extends_int64() {
    let checker = check("scalar BigId extends int64;");
    let s_type = checker.declared_types.get("BigId").copied().unwrap();
    let t = checker.get_type(s_type).cloned().unwrap();
    match t {
        Type::Scalar(s) => {
            assert_eq!(s.name, "BigId");
            let base_id = s.base_scalar.unwrap();
            let base = checker.get_type(base_id).cloned().unwrap();
            assert!(matches!(base, Type::Scalar(ref bs) if bs.name == "int64"));
        }
        _ => panic!("Expected Scalar type"),
    }
}

#[test]
fn test_scalar_extends_url() {
    let checker = check("scalar MyUrl extends url;");
    let s_type = checker.declared_types.get("MyUrl").copied().unwrap();
    let t = checker.get_type(s_type).cloned().unwrap();
    match t {
        Type::Scalar(s) => {
            assert_eq!(s.name, "MyUrl");
            let base_id = s.base_scalar.unwrap();
            let base = checker.get_type(base_id).cloned().unwrap();
            assert!(matches!(base, Type::Scalar(ref bs) if bs.name == "url"));
        }
        _ => panic!("Expected Scalar type"),
    }
}

#[test]
fn test_scalar_multiple_decorators() {
    let checker = check("@doc @tag scalar MyScalar extends string;");
    let s_type = checker.declared_types.get("MyScalar").copied().unwrap();
    let t = checker.get_type(s_type).cloned().unwrap();
    match t {
        Type::Scalar(s) => {
            assert_eq!(s.decorators.len(), 2, "Scalar should have 2 decorators");
        }
        _ => panic!("Expected Scalar type"),
    }
}

#[test]
fn test_scalar_extends_scalar_chain() {
    // Scalar extending another user-defined scalar
    let checker = check(
        "
        scalar Base extends string;
        scalar Derived extends Base;
    ",
    );
    let derived_type = checker.declared_types.get("Derived").copied().unwrap();
    let t = checker.get_type(derived_type).cloned().unwrap();
    match t {
        Type::Scalar(s) => {
            assert_eq!(s.name, "Derived");
            let base_id = s.base_scalar.unwrap();
            let base = checker.get_type(base_id).cloned().unwrap();
            // Base scalar of Derived should be Base (user-defined)
            assert!(
                matches!(base, Type::Scalar(ref bs) if bs.name == "Base"),
                "Expected Base as base_scalar, got {:?}",
                base.kind_name()
            );
        }
        _ => panic!("Expected Scalar type"),
    }
}

#[test]
fn test_scalar_extends_model_detected_alt() {
    // Extending a model should report extend-scalar (alternate test case)
    let checker = check("model Foo {} scalar Bar extends Foo;");
    let diags = checker.diagnostics();
    assert!(
        diags.iter().any(|d| d.code == "extend-scalar"),
        "Should report extend-scalar when scalar extends model: {:?}",
        diags
    );
}

#[test]
fn test_scalar_in_global_namespace() {
    let checker = check("scalar MyScalar extends string;");
    let global_ns_id = checker.get_global_namespace_type().unwrap();
    let t = checker.get_type(global_ns_id).cloned().unwrap();
    match t {
        Type::Namespace(ns) => {
            assert!(
                ns.scalars.contains_key("MyScalar"),
                "Global namespace should contain MyScalar"
            );
        }
        _ => panic!("Expected Namespace type"),
    }
}

// ============================================================================
// Template parameter tests for scalars
// ============================================================================

/// Ported from TS: "declare scalar with template parameters"
#[test]
fn test_scalar_template_declaration() {
    let checker = check("scalar Id<T extends valueof string> extends string;");
    assert!(
        checker.declared_types.contains_key("Id"),
        "Template scalar Id should be declared"
    );
    let id_type = checker.declared_types.get("Id").copied().unwrap();
    match checker.get_type(id_type).cloned().unwrap() {
        Type::Scalar(s) => {
            assert!(
                s.template_node.is_some(),
                "Template scalar should have template_node set"
            );
            assert!(
                !s.is_finished,
                "Template declaration should not be finished"
            );
        }
        _ => panic!("Expected Scalar type"),
    }
}

/// Scalar with circular constraint
#[test]
fn test_scalar_circular_constraint_self() {
    let checker = check("scalar Test<A extends A> extends string;");
    let diags = checker.diagnostics();
    assert!(
        diags.iter().any(|d| d.code == "circular-constraint"),
        "Should emit circular-constraint for self-referencing scalar constraint: {:?}",
        diags
    );
}

/// Scalar with invalid template default
#[test]
fn test_scalar_invalid_template_default() {
    let checker = check("scalar A<A = B, B = string> extends string;");
    let diags = checker.diagnostics();
    assert!(
        diags.iter().any(|d| d.code == "invalid-template-default"),
        "Should emit invalid-template-default for scalar default referencing later param: {:?}",
        diags
    );
}

// ============================================================================
// Scalar hierarchy chain verification (TypeSpec intrinsics.tsp)
// ============================================================================

#[test]
fn test_int8_extends_int16() {
    let checker = check("");
    let int8 = checker.std_types.get("int8").copied().unwrap();
    let int16 = checker.std_types.get("int16").copied().unwrap();
    let s = checker.get_type(int8).cloned().unwrap();
    match s {
        Type::Scalar(s) => assert_eq!(
            s.base_scalar,
            Some(int16),
            "int8.base_scalar should be int16"
        ),
        _ => panic!("Expected Scalar"),
    }
}

#[test]
fn test_int16_extends_int32() {
    let checker = check("");
    let int16 = checker.std_types.get("int16").copied().unwrap();
    let int32 = checker.std_types.get("int32").copied().unwrap();
    let s = checker.get_type(int16).cloned().unwrap();
    match s {
        Type::Scalar(s) => assert_eq!(
            s.base_scalar,
            Some(int32),
            "int16.base_scalar should be int32"
        ),
        _ => panic!("Expected Scalar"),
    }
}

#[test]
fn test_int32_extends_int64() {
    let checker = check("");
    let int32 = checker.std_types.get("int32").copied().unwrap();
    let int64 = checker.std_types.get("int64").copied().unwrap();
    let s = checker.get_type(int32).cloned().unwrap();
    match s {
        Type::Scalar(s) => assert_eq!(
            s.base_scalar,
            Some(int64),
            "int32.base_scalar should be int64"
        ),
        _ => panic!("Expected Scalar"),
    }
}

#[test]
fn test_int64_extends_integer() {
    let checker = check("");
    let int64 = checker.std_types.get("int64").copied().unwrap();
    let integer = checker.std_types.get("integer").copied().unwrap();
    let s = checker.get_type(int64).cloned().unwrap();
    match s {
        Type::Scalar(s) => assert_eq!(
            s.base_scalar,
            Some(integer),
            "int64.base_scalar should be integer"
        ),
        _ => panic!("Expected Scalar"),
    }
}

#[test]
fn test_uint8_extends_uint16() {
    let checker = check("");
    let uint8 = checker.std_types.get("uint8").copied().unwrap();
    let uint16 = checker.std_types.get("uint16").copied().unwrap();
    let s = checker.get_type(uint8).cloned().unwrap();
    match s {
        Type::Scalar(s) => assert_eq!(
            s.base_scalar,
            Some(uint16),
            "uint8.base_scalar should be uint16"
        ),
        _ => panic!("Expected Scalar"),
    }
}

#[test]
fn test_uint16_extends_uint32() {
    let checker = check("");
    let uint16 = checker.std_types.get("uint16").copied().unwrap();
    let uint32 = checker.std_types.get("uint32").copied().unwrap();
    let s = checker.get_type(uint16).cloned().unwrap();
    match s {
        Type::Scalar(s) => assert_eq!(
            s.base_scalar,
            Some(uint32),
            "uint16.base_scalar should be uint32"
        ),
        _ => panic!("Expected Scalar"),
    }
}

#[test]
fn test_uint32_extends_uint64() {
    let checker = check("");
    let uint32 = checker.std_types.get("uint32").copied().unwrap();
    let uint64 = checker.std_types.get("uint64").copied().unwrap();
    let s = checker.get_type(uint32).cloned().unwrap();
    match s {
        Type::Scalar(s) => assert_eq!(
            s.base_scalar,
            Some(uint64),
            "uint32.base_scalar should be uint64"
        ),
        _ => panic!("Expected Scalar"),
    }
}

#[test]
fn test_uint64_extends_integer() {
    let checker = check("");
    let uint64 = checker.std_types.get("uint64").copied().unwrap();
    let integer = checker.std_types.get("integer").copied().unwrap();
    let s = checker.get_type(uint64).cloned().unwrap();
    match s {
        Type::Scalar(s) => assert_eq!(
            s.base_scalar,
            Some(integer),
            "uint64.base_scalar should be integer"
        ),
        _ => panic!("Expected Scalar"),
    }
}

#[test]
fn test_float32_extends_float64() {
    let checker = check("");
    let float32 = checker.std_types.get("float32").copied().unwrap();
    let float64 = checker.std_types.get("float64").copied().unwrap();
    let s = checker.get_type(float32).cloned().unwrap();
    match s {
        Type::Scalar(s) => assert_eq!(
            s.base_scalar,
            Some(float64),
            "float32.base_scalar should be float64"
        ),
        _ => panic!("Expected Scalar"),
    }
}

#[test]
fn test_float64_extends_float() {
    let checker = check("");
    let float64 = checker.std_types.get("float64").copied().unwrap();
    let float = checker.std_types.get("float").copied().unwrap();
    let s = checker.get_type(float64).cloned().unwrap();
    match s {
        Type::Scalar(s) => assert_eq!(
            s.base_scalar,
            Some(float),
            "float64.base_scalar should be float"
        ),
        _ => panic!("Expected Scalar"),
    }
}

#[test]
fn test_decimal128_extends_decimal() {
    let checker = check("");
    let decimal128 = checker.std_types.get("decimal128").copied().unwrap();
    let decimal = checker.std_types.get("decimal").copied().unwrap();
    let s = checker.get_type(decimal128).cloned().unwrap();
    match s {
        Type::Scalar(s) => assert_eq!(
            s.base_scalar,
            Some(decimal),
            "decimal128.base_scalar should be decimal"
        ),
        _ => panic!("Expected Scalar"),
    }
}

#[test]
fn test_safeint_extends_int64() {
    let checker = check("");
    let safeint = checker.std_types.get("safeint").copied().unwrap();
    let int64 = checker.std_types.get("int64").copied().unwrap();
    let s = checker.get_type(safeint).cloned().unwrap();
    match s {
        Type::Scalar(s) => assert_eq!(
            s.base_scalar,
            Some(int64),
            "safeint.base_scalar should be int64"
        ),
        _ => panic!("Expected Scalar"),
    }
}

// ============================================================================
// Scalar Default Value Tests
// Ported from TS scalar.test.ts
// ============================================================================

#[test]
fn test_scalar_default_outside_range() {
    // Ported from TS: "does not allow custom numeric scalar to have a default outside range"
    let checker = check(
        r#"
        namespace SomeNamespace;
        scalar S extends int8;
        model M { p?: S = 9999; }
    "#,
    );
    let diags = checker.diagnostics();
    assert!(
        diags.iter().any(|d| d.code == "unassignable"),
        "Should report unassignable when default 9999 is outside int8 range: {:?}",
        diags
    );
}

#[test]
fn test_scalar_default_non_numeric_scalar() {
    // Ported from TS: "does not allow non-numeric/boolean/string custom scalar to have a default"
    let checker = check(
        r#"
        scalar S;
        model M { p?: S = 42; }
    "#,
    );
    let diags = checker.diagnostics();
    assert!(
        diags.iter().any(|d| d.code == "unassignable"),
        "Should report unassignable when assigning default to plain scalar: {:?}",
        diags
    );
}

// ============================================================================
// constructor-duplicate diagnostic tests
// ============================================================================

/// When a scalar declares two init constructors with the same name, emit constructor-duplicate
#[test]
fn test_constructor_duplicate_same_scalar() {
    let checker = check(
        r#"
        scalar MyScalar extends string {
            init create(value: string);
            init create(value: string);
        }
    "#,
    );
    let diags = checker.diagnostics();
    assert!(
        diags.iter().any(|d| d.code == "constructor-duplicate"),
        "Should report constructor-duplicate for duplicate init names: {:?}",
        diags
    );
}

/// Verify message content for constructor-duplicate
#[test]
fn test_constructor_duplicate_message() {
    let checker = check(
        r#"
        scalar MyScalar extends string {
            init create(value: string);
            init create(value: string);
        }
    "#,
    );
    let diags = checker.diagnostics();
    let diag = diags
        .iter()
        .find(|d| d.code == "constructor-duplicate")
        .unwrap();
    assert!(
        diag.message.contains("create"),
        "Message should mention the duplicate constructor name 'create': {}",
        diag.message
    );
}

/// No constructor-duplicate when constructors have different names
#[test]
fn test_constructor_no_duplicate_different_names() {
    let checker = check(
        r#"
        scalar MyScalar extends string {
            init fromString(value: string);
            init fromNumber(value: int32);
        }
    "#,
    );
    let diags = checker.diagnostics();
    assert!(
        !diags.iter().any(|d| d.code == "constructor-duplicate"),
        "Should NOT report constructor-duplicate for different init names: {:?}",
        diags
    );
}

/// When a scalar inherits a constructor from base and declares same name, emit constructor-duplicate
#[test]
fn test_constructor_duplicate_inherited_from_base() {
    let checker = check(
        r#"
        scalar Base extends string {
            init create(value: string);
        }
        scalar Child extends Base {
            init create(value: string);
        }
    "#,
    );
    let diags = checker.diagnostics();
    assert!(
        diags.iter().any(|d| d.code == "constructor-duplicate"),
        "Should report constructor-duplicate for init inherited from base: {:?}",
        diags
    );
}
