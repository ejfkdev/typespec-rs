//! Integration tests for real-world scenarios from the feature request.
//! Tests all 7 scenarios (A-L) to verify current implementation status.

use crate::checker::test_utils::{check_with_decorators, has_diagnostic};
use crate::checker::types::{DecoratorMarshalledValue, Type};
use crate::checker::type_utils::get_fully_qualified_name;
use crate::checker::{Checker, DecoratorParamDef};
use crate::parser;

// ============================================================================
// Scenario A: Extract @route argument on interface
// ============================================================================

#[test]
fn test_scenario_a_route_on_interface() {
    let checker = check_with_decorators(
        r#"
        using HTTP;
        @route("/chat/completions")
        interface Chat {
            op create(): void;
        }
    "#,
        vec![("route", "HTTP", "Interface")],
    );

    let chat_id = checker.declared_types.get("Chat").copied().expect("Chat should exist");
    let iface = match checker.get_type(chat_id) {
        Some(Type::Interface(i)) => i,
        _ => panic!("expected Interface"),
    };

    assert_eq!(iface.decorators.len(), 1, "should have 1 decorator");
    let dec = &iface.decorators[0];
    assert!(dec.definition.is_some(), "definition should be resolved");
    assert_eq!(dec.args.len(), 1, "should have 1 argument");

    match &dec.args[0].js_value {
        Some(DecoratorMarshalledValue::String(s)) => {
            assert_eq!(s, "/chat/completions");
        }
        other => panic!("expected String js_value, got {:?}", other),
    }
}

// ============================================================================
// Scenario B: Extract @command argument on operation
// ============================================================================

#[test]
fn test_scenario_b_command_on_operation() {
    let checker = check_with_decorators(
        r#"
        using CLI;
        @command("status")
        op gitStatus(): string;
    "#,
        vec![("command", "CLI", "Operation")],
    );

    let op_id = checker.declared_types.get("gitStatus").copied().expect("gitStatus should exist");
    let op = match checker.get_type(op_id) {
        Some(Type::Operation(o)) => o,
        _ => panic!("expected Operation"),
    };

    assert_eq!(op.decorators.len(), 1);
    let dec = &op.decorators[0];
    assert!(dec.definition.is_some(), "definition should be resolved");
    assert_eq!(dec.args.len(), 1);

    match &dec.args[0].js_value {
        Some(DecoratorMarshalledValue::String(s)) => {
            assert_eq!(s, "status");
        }
        other => panic!("expected String js_value, got {:?}", other),
    }
}

// ============================================================================
// Scenario C: @header with object param on ModelProperty
// ============================================================================

#[test]
fn test_scenario_c_header_with_object_param() {
    let checker = check_with_decorators(
        r#"
        using HTTP;
        model Request {
            @header({name: "X-Request-Id"})
            requestId: string;
        }
    "#,
        vec![("header", "HTTP", "unknown")],
    );

    let model_id = checker.declared_types.get("Request").copied().expect("Request should exist");
    let model = match checker.get_type(model_id) {
        Some(Type::Model(m)) => m,
        _ => panic!("expected Model"),
    };

    let prop_id = model.properties.get("requestId").expect("requestId property should exist");
    let prop = match checker.get_type(*prop_id) {
        Some(Type::ModelProperty(p)) => p,
        _ => panic!("expected ModelProperty"),
    };

    assert_eq!(prop.decorators.len(), 1, "property should have 1 decorator");
    let dec = &prop.decorators[0];

    // definition should be resolved
    assert!(dec.definition.is_some(), "property decorator definition should be resolved");

    // args should be populated
    assert_eq!(dec.args.len(), 1, "should have 1 argument");

    // js_value should be a Record
    match &dec.args[0].js_value {
        Some(DecoratorMarshalledValue::Record(map)) => {
            assert!(map.contains_key("name"), "record should contain 'name' key");
        }
        other => panic!("expected Record js_value, got {:?}", other),
    }
}

// ============================================================================
// Scenario D: Namespace hierarchy — interface in namespace
// ============================================================================

#[test]
fn test_scenario_d_namespace_hierarchy() {
    let checker = check_with_decorators(
        r#"
        namespace Llm {
            interface Chat { op create(): void; }
        }
    "#,
        vec![],
    );

    let llm_id = checker.declared_types.get("Llm").copied().expect("Llm namespace should exist");
    let llm_ns = match checker.get_type(llm_id) {
        Some(Type::Namespace(ns)) => ns,
        _ => panic!("expected Namespace"),
    };

    // Llm namespace should have Chat in interface_names
    assert!(
        llm_ns.interface_names.contains(&"Chat".to_string()),
        "Llm.interface_names should contain 'Chat', got {:?}",
        llm_ns.interface_names
    );

    // Chat interface's namespace should point to Llm, not global
    let chat_id = checker.declared_types.get("Chat").copied().expect("Chat should exist");
    let chat_iface = match checker.get_type(chat_id) {
        Some(Type::Interface(i)) => i,
        _ => panic!("expected Interface"),
    };

    assert_eq!(
        chat_iface.namespace, Some(llm_id),
        "Chat.namespace should point to Llm namespace, not global"
    );
}

// ============================================================================
// Scenario E: Decorator inheritance — effective route
// ============================================================================

#[test]
fn test_scenario_e_decorator_inheritance() {
    let checker = check_with_decorators(
        r#"
        using HTTP;
        @route("/chat")
        interface Chat {
            @route("/completions")
            op create(): void;
        }
    "#,
        vec![("route", "HTTP", "unknown")],
    );

    let chat_id = checker.declared_types.get("Chat").copied().expect("Chat should exist");
    let iface = match checker.get_type(chat_id) {
        Some(Type::Interface(i)) => i,
        _ => panic!("expected Interface"),
    };

    // Operation create should have its own @route("/completions")
    if let Some(op_id) = iface.operations.get("create").copied() {
        let op = match checker.get_type(op_id) {
            Some(Type::Operation(o)) => o,
            _ => panic!("expected Operation"),
        };

        // Operation has its own @route decorator
        assert!(!op.decorators.is_empty(), "operation should have at least 1 decorator");

        // Check if get_effective_route exists and works
        let effective_route = checker.get_effective_route(op_id);
        assert_eq!(
            effective_route,
            Some("/chat/completions".to_string()),
            "effective route should combine interface + operation routes"
        );

        // Check get_effective_decorators — includes interface decorators
        let effective_decs = checker.get_effective_decorators(op_id);
        // Should include interface's @route("/chat") + operation's @route("/completions")
        assert!(
            effective_decs.len() >= 2,
            "effective decorators should include interface + operation decorators, got {}",
            effective_decs.len()
        );
    }
}

// ============================================================================
// Scenario G: using dotted namespace from register_decorator
// ============================================================================

#[test]
fn test_scenario_g_using_dotted_namespace() {
    let checker = check_with_decorators(
        r#"
        using AnyUse.CLI;
        model Options {
            @cliFlag verbose?: boolean;
        }
    "#,
        vec![("cliFlag", "AnyUse.CLI", "unknown")],
    );

    // Should not have using-invalid-ref
    assert!(
        !has_diagnostic(&checker, "using-invalid-ref"),
        "Should NOT report using-invalid-ref: {:?}",
        checker.diagnostics()
    );

    // Check if the property decorator was resolved
    let opts_id = checker.declared_types.get("Options").copied().expect("Options should exist");
    if let Some(Type::Model(m)) = checker.get_type(opts_id)
        && let Some(prop_id) = m.properties.get("verbose")
        && let Some(Type::ModelProperty(prop)) = checker.get_type(*prop_id)
    {
        assert_eq!(prop.decorators.len(), 1, "verbose should have @cliFlag");
        let dec = &prop.decorators[0];
        assert!(dec.definition.is_some(), "cliFlag definition should be resolved");
    }
}

// ============================================================================
// Scenario H: Multiple custom namespaces
// ============================================================================

#[test]
fn test_scenario_h_multiple_custom_namespaces() {
    let checker = check_with_decorators(
        r#"
        using MyCorp.Auth;
        using MyCorp.RateLimit;
        @requireAuth @rateLimit(100)
        op getData(): string;
    "#,
        vec![
            ("requireAuth", "MyCorp.Auth", "Operation"),
            ("rateLimit", "MyCorp.RateLimit", "Operation"),
        ],
    );

    // No using-invalid-ref
    assert!(
        !has_diagnostic(&checker, "using-invalid-ref"),
        "Should NOT report using-invalid-ref: {:?}",
        checker.diagnostics()
    );

    let op_id = checker.declared_types.get("getData").copied().expect("getData should exist");
    let op = match checker.get_type(op_id) {
        Some(Type::Operation(o)) => o,
        _ => panic!("expected Operation"),
    };

    assert_eq!(op.decorators.len(), 2, "should have 2 decorators");
    for (i, dec) in op.decorators.iter().enumerate() {
        assert!(
            dec.definition.is_some(),
            "decorator {} definition should be resolved",
            i
        );
    }

    // rateLimit(100) — check numeric arg
    let rate_limit_dec = op.decorators.iter().find(|d| {
        d.definition
            .and_then(|id| checker.get_type(id).and_then(|t| match t {
                Type::Decorator(dt) => Some(dt.name.clone()),
                _ => None,
            }))
            .is_some_and(|n| n == "rateLimit")
    });
    if let Some(dec) = rate_limit_dec {
        assert_eq!(dec.args.len(), 1);
        match &dec.args[0].js_value {
            Some(DecoratorMarshalledValue::Number(n)) => assert_eq!(*n, 100.0),
            other => panic!("expected Number, got {:?}", other),
        }
    }
}

// ============================================================================
// Scenario I: Same-name types in different namespaces
// ============================================================================

#[test]
fn test_scenario_i_same_name_types_different_namespaces() {
    let checker = check_with_decorators(
        r#"
        namespace A { model Request { id: string } }
        namespace B { model Request { name: string } }
    "#,
        vec![],
    );

    // FQN should differentiate them
    let a_ns_id = checker.declared_types.get("A").copied().expect("A should exist");
    let b_ns_id = checker.declared_types.get("B").copied().expect("B should exist");

    // Find Request in each namespace
    let a_req = {
        let ns = match checker.get_type(a_ns_id) {
            Some(Type::Namespace(ns)) => ns,
            _ => panic!("expected Namespace"),
        };
        ns.models.get("Request").copied().expect("A.Request should exist")
    };

    let b_req = {
        let ns = match checker.get_type(b_ns_id) {
            Some(Type::Namespace(ns)) => ns,
            _ => panic!("expected Namespace"),
        };
        ns.models.get("Request").copied().expect("B.Request should exist")
    };

    let fqn_a = get_fully_qualified_name(&checker.type_store, a_req);
    let fqn_b = get_fully_qualified_name(&checker.type_store, b_req);

    assert_eq!(fqn_a, "A.Request", "FQN for A.Request should be 'A.Request'");
    assert_eq!(fqn_b, "B.Request", "FQN for B.Request should be 'B.Request'");
}

// ============================================================================
// Scenario K: ModelProperty structured decorator args
// ============================================================================

#[test]
fn test_scenario_k_model_property_structured_decorator_args() {
    let checker = check_with_decorators(
        r#"
        using CLI;
        model Options {
            @cliFlag({short: "v", description: "Verbose output"})
            verbose?: boolean;
        }
    "#,
        vec![("cliFlag", "CLI", "unknown")],
    );

    let opts_id = checker.declared_types.get("Options").copied().expect("Options should exist");
    let model = match checker.get_type(opts_id) {
        Some(Type::Model(m)) => m,
        _ => panic!("expected Model"),
    };

    let prop_id = model.properties.get("verbose").expect("verbose property should exist");
    let prop = match checker.get_type(*prop_id) {
        Some(Type::ModelProperty(p)) => p,
        _ => panic!("expected ModelProperty"),
    };

    assert_eq!(prop.decorators.len(), 1);
    let dec = &prop.decorators[0];
    assert!(dec.definition.is_some(), "cliFlag definition should be resolved");
    assert_eq!(dec.args.len(), 1);

    // The arg should be a Record with "short" and "description" keys
    match &dec.args[0].js_value {
        Some(DecoratorMarshalledValue::Record(map)) => {
            assert!(map.contains_key("short"), "record should contain 'short'");
            assert!(map.contains_key("description"), "record should contain 'description'");
        }
        other => panic!("expected Record js_value, got {:?}", other),
    }
}

// ============================================================================
// Scenario L: Multiple decorators on ModelProperty
// ============================================================================

#[test]
fn test_scenario_l_multiple_decorators_on_property() {
    let checker = check_with_decorators(
        r#"
        using HTTP;
        model Request {
            @HTTP.header("X-Api-Key")
            @HTTP.query("page")
            page?: int32;
        }
    "#,
        vec![("header", "HTTP", "unknown"), ("query", "HTTP", "unknown")],
    );

    let model_id = checker.declared_types.get("Request").copied().expect("Request should exist");
    let model = match checker.get_type(model_id) {
        Some(Type::Model(m)) => m,
        _ => panic!("expected Model"),
    };

    let prop_id = model.properties.get("page").expect("page property should exist");
    let prop = match checker.get_type(*prop_id) {
        Some(Type::ModelProperty(p)) => p,
        _ => panic!("expected ModelProperty"),
    };

    assert_eq!(prop.decorators.len(), 2, "should have 2 decorators");

    // Both should have resolved definitions
    for (i, dec) in prop.decorators.iter().enumerate() {
        assert!(
            dec.definition.is_some(),
            "decorator {} definition should be resolved",
            i
        );
    }

    // Check string args
    for dec in &prop.decorators {
        if let Some(arg) = dec.args.first() {
            match &arg.js_value {
                Some(DecoratorMarshalledValue::String(s)) => {
                    assert!(
                        s == "X-Api-Key" || s == "page",
                        "decorator arg should be 'X-Api-Key' or 'page', got '{}'",
                        s
                    );
                }
                other => panic!("expected String js_value, got {:?}", other),
            }
        }
    }
}

// ============================================================================
// Bug regression tests
// ============================================================================

/// Regression test for Bug 2: DecoratorType should not reject arguments
/// when parameters is empty (registered without params → skip count validation).
#[test]
fn test_bug2_no_invalid_argument_count_for_paramless_decorators() {
    let checker = check_with_decorators(
        r#"
        using CLI;
        @command("status")
        op gitStatus(): string;
    "#,
        vec![("command", "CLI", "Operation")],
    );

    // Should NOT have invalid-argument-count diagnostic
    assert!(
        !has_diagnostic(&checker, "invalid-argument-count"),
        "Should NOT report invalid-argument-count for paramless decorator with args: {:?}",
        checker.diagnostics()
    );

    // Decorator should still be applied with correct args
    let op_id = checker.declared_types.get("gitStatus").copied().expect("gitStatus should exist");
    let op = match checker.get_type(op_id) {
        Some(Type::Operation(o)) => o,
        _ => panic!("expected Operation"),
    };
    assert_eq!(op.decorators.len(), 1);
    assert_eq!(op.decorators[0].args.len(), 1);
    match &op.decorators[0].args[0].js_value {
        Some(DecoratorMarshalledValue::String(s)) => assert_eq!(s, "status"),
        other => panic!("expected String, got {:?}", other),
    }
}

/// Regression test for Bug 2: register_decorator_with_params enforces arg count.
#[test]
fn test_bug2_decorator_with_params_enforces_count() {
    let mut checker = {
        let result = parser::parse(r#"
            using CLI;
            @command("status")
            op gitStatus(): string;
        "#);
        let mut c = Checker::new();
        c.register_decorator_with_params(
            "command",
            "CLI",
            "Operation",
            vec![DecoratorParamDef {
                name: "name".to_string(),
                type_name: "string".to_string(),
                optional: false,
                rest: false,
            }],
        );
        c.set_parse_result(result.root_id, result.builder);
        c
    };
    checker.check_program();

    // With explicit params, the 1-arg call should be valid
    assert!(
        !has_diagnostic(&checker, "invalid-argument-count"),
        "Should NOT report invalid-argument-count: {:?}",
        checker.diagnostics()
    );
}

/// Regression test for Bug 2: decorator with params rejects wrong arg count.
#[test]
fn test_bug2_decorator_with_params_rejects_wrong_count() {
    let mut checker = {
        let result = parser::parse(r#"
            using CLI;
            @command("status", "extra")
            op gitStatus(): string;
        "#);
        let mut c = Checker::new();
        c.register_decorator_with_params(
            "command",
            "CLI",
            "Operation",
            vec![DecoratorParamDef {
                name: "name".to_string(),
                type_name: "string".to_string(),
                optional: false,
                rest: false,
            }],
        );
        c.set_parse_result(result.root_id, result.builder);
        c
    };
    checker.check_program();

    // With 1 declared param, 2 args should fail
    assert!(
        has_diagnostic(&checker, "invalid-argument-count"),
        "Should report invalid-argument-count for 2 args when 1 declared: {:?}",
        checker.diagnostics()
    );
}

/// Regression test for Bug 3: using a dotted namespace should NOT report unused-using.
#[test]
fn test_bug3_using_dotted_namespace_not_unused() {
    let checker = check_with_decorators(
        r#"
        using AnyUse.CLI;
        model Options {
            @cliFlag verbose?: boolean;
        }
    "#,
        vec![("cliFlag", "AnyUse.CLI", "unknown")],
    );

    // Should NOT report unused-using
    assert!(
        !has_diagnostic(&checker, "unused-using"),
        "Should NOT report unused-using for used dotted namespace: {:?}",
        checker.diagnostics()
    );
}

/// Regression test for Bug 3: using with simple namespace not reported as unused.
#[test]
fn test_bug3_using_simple_namespace_not_unused() {
    let checker = check_with_decorators(
        r#"
        using HTTP;
        @route("/api")
        interface Api {
            op list(): void;
        }
    "#,
        vec![("route", "HTTP", "Interface")],
    );

    // Should NOT report unused-using
    assert!(
        !has_diagnostic(&checker, "unused-using"),
        "Should NOT report unused-using for used simple namespace: {:?}",
        checker.diagnostics()
    );
}

/// Regression test for Bug 1: namespace created by ensure_decorator_namespace
/// with overlapping user-code namespace declaration.
#[test]
fn test_bug1_overlapping_namespace_decorator_and_user_code() {
    // Register a decorator in "Llm" namespace, then also declare Llm in user code
    let checker = check_with_decorators(
        r#"
        namespace Llm {
            @command("status")
            op gitStatus(): string;
        }
    "#,
        vec![("command", "Llm", "Operation")],
    );

    // Llm namespace should have the operation
    let llm_id = checker.declared_types.get("Llm").copied().expect("Llm should exist");
    let llm_ns = match checker.get_type(llm_id) {
        Some(Type::Namespace(ns)) => ns,
        _ => panic!("expected Namespace"),
    };

    // The operation should be in the namespace's operations
    assert!(
        llm_ns.operations.contains_key("gitStatus"),
        "Llm.operations should contain 'gitStatus', got {:?}",
        llm_ns.operation_names
    );

    // The decorator should be in the namespace's decorator_declarations
    assert!(
        llm_ns.decorator_declarations.contains_key("command"),
        "Llm.decorator_declarations should contain 'command'"
    );
}
