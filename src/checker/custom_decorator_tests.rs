//! Tests for custom decorator P0 features:
//! - Decorator definition + args marshalling
//! - using support for custom namespaces created by register_decorator

use crate::checker::test_utils::{check_with_decorators, has_diagnostic};
use crate::checker::types::{DecoratorMarshalledValue, Type};

// ============================================================================
// P0-1: Custom decorator definition resolution
// ============================================================================

#[test]
fn test_registered_decorator_definition_resolved() {
    // When a decorator is registered via register_decorator and used with a
    // namespace prefix, the DecoratorApplication.definition should point to
    // the registered DecoratorType.
    let checker = check_with_decorators(
        r#"
        @HTTP.route("/api/pets")
        op listPets(): void;
    "#,
        vec![("route", "HTTP", "Operation")],
    );

    // Find the operation type
    let op_type = checker
        .declared_types
        .get("listPets")
        .copied()
        .expect("operation should exist");

    let decs = match checker.get_type(op_type) {
        Some(Type::Operation(op)) => &op.decorators,
        _ => panic!("expected Operation type"),
    };

    assert_eq!(decs.len(), 1, "should have exactly 1 decorator");
    let dec = &decs[0];

    // definition should be Some — the decorator was resolved
    assert!(
        dec.definition.is_some(),
        "decorator definition should be resolved (not None)"
    );

    // Verify it's a Decorator type with name "route"
    if let Some(def_id) = dec.definition {
        if let Some(Type::Decorator(dt)) = checker.get_type(def_id) {
            assert_eq!(dt.name, "route");
        } else {
            panic!("definition should point to a DecoratorType");
        }
    }
}

#[test]
fn test_registered_decorator_args_populated() {
    // Decorator arguments should be parsed and stored in DecoratorApplication.args
    let checker = check_with_decorators(
        r#"
        @HTTP.route("/api/pets")
        op listPets(): void;
    "#,
        vec![("route", "HTTP", "Operation")],
    );

    let op_type = checker
        .declared_types
        .get("listPets")
        .copied()
        .expect("operation should exist");

    let decs = match checker.get_type(op_type) {
        Some(Type::Operation(op)) => &op.decorators,
        _ => panic!("expected Operation type"),
    };

    assert_eq!(decs.len(), 1);
    let dec = &decs[0];
    assert_eq!(dec.args.len(), 1, "should have 1 argument");

    // The argument should have a marshalled js_value
    let arg = &dec.args[0];
    assert!(
        arg.js_value.is_some(),
        "argument should have a marshalled js_value"
    );

    // The value should be the string "/api/pets"
    if let Some(DecoratorMarshalledValue::String(s)) = &arg.js_value {
        assert_eq!(s, "/api/pets");
    } else {
        panic!(
            "expected String marshalled value, got {:?}",
            arg.js_value
        );
    }
}

#[test]
fn test_decorator_arg_numeric_marshall() {
    let checker = check_with_decorators(
        r#"
        @priority(3)
        op doWork(): void;
    "#,
        vec![("priority", "MyApp", "Operation")],
    );

    let op_type = checker
        .declared_types
        .get("doWork")
        .copied()
        .expect("operation should exist");

    let decs = match checker.get_type(op_type) {
        Some(Type::Operation(op)) => &op.decorators,
        _ => panic!("expected Operation type"),
    };

    assert_eq!(decs.len(), 1);
    let arg = &decs[0].args[0];
    if let Some(DecoratorMarshalledValue::Number(n)) = &arg.js_value {
        assert_eq!(*n, 3.0);
    } else {
        panic!("expected Number marshalled value, got {:?}", arg.js_value);
    }
}

#[test]
fn test_decorator_arg_boolean_marshall() {
    let checker = check_with_decorators(
        r#"
        @enabled(true)
        op doWork(): void;
    "#,
        vec![("enabled", "MyApp", "Operation")],
    );

    let op_type = checker
        .declared_types
        .get("doWork")
        .copied()
        .expect("operation should exist");

    let decs = match checker.get_type(op_type) {
        Some(Type::Operation(op)) => &op.decorators,
        _ => panic!("expected Operation type"),
    };

    assert_eq!(decs.len(), 1);
    let arg = &decs[0].args[0];
    if let Some(DecoratorMarshalledValue::Boolean(b)) = &arg.js_value {
        assert!(*b);
    } else {
        panic!(
            "expected Boolean marshalled value, got {:?}",
            arg.js_value
        );
    }
}

#[test]
fn test_decorator_on_model_with_definition() {
    let checker = check_with_decorators(
        r#"
        @API.tag("pet")
        model Pet { name: string }
    "#,
        vec![("tag", "API", "Model")],
    );

    let model_type = checker
        .declared_types
        .get("Pet")
        .copied()
        .expect("model should exist");

    let decs = match checker.get_type(model_type) {
        Some(Type::Model(m)) => &m.decorators,
        _ => panic!("expected Model type"),
    };

    assert_eq!(decs.len(), 1);
    assert!(
        decs[0].definition.is_some(),
        "decorator definition should be resolved on model"
    );

    let arg = &decs[0].args[0];
    if let Some(DecoratorMarshalledValue::String(s)) = &arg.js_value {
        assert_eq!(s, "pet");
    } else {
        panic!("expected String marshalled value");
    }
}

#[test]
fn test_multiple_decorator_args() {
    let checker = check_with_decorators(
        r#"
        @route("/api/pets", "GET")
        op listPets(): void;
    "#,
        vec![("route", "HTTP", "Operation")],
    );

    let op_type = checker
        .declared_types
        .get("listPets")
        .copied()
        .expect("operation should exist");

    let decs = match checker.get_type(op_type) {
        Some(Type::Operation(op)) => &op.decorators,
        _ => panic!("expected Operation type"),
    };

    assert_eq!(decs[0].args.len(), 2, "should have 2 arguments");
}

#[test]
fn test_unregistered_decorator_definition_is_none() {
    // Decorators NOT registered should still have definition=None
    let checker = check_with_decorators(
        r#"
        @unknownDecorator("test")
        op doWork(): void;
    "#,
        vec![], // no decorators registered
    );

    let op_type = checker
        .declared_types
        .get("doWork")
        .copied()
        .expect("operation should exist");

    let decs = match checker.get_type(op_type) {
        Some(Type::Operation(op)) => &op.decorators,
        _ => panic!("expected Operation type"),
    };

    // The decorator should exist on the type but definition should be None
    // (because "unknownDecorator" was not registered)
    assert_eq!(decs.len(), 1);
    assert!(
        decs[0].definition.is_none(),
        "unregistered decorator should have definition=None"
    );
}

// ============================================================================
// P0-2: using support for custom namespaces
// ============================================================================

#[test]
fn test_using_custom_namespace_no_error() {
    // A namespace created by register_decorator should be usable with `using`
    let checker = check_with_decorators(
        r#"
        using HTTP;
    "#,
        vec![("route", "HTTP", "Operation")],
    );

    assert!(
        !has_diagnostic(&checker, "using-invalid-ref"),
        "Should NOT report using-invalid-ref for custom namespace: {:?}",
        checker.diagnostics()
    );
}

#[test]
fn test_using_custom_namespace_decorator_resolved() {
    // After `using HTTP;`, `@route(...)` should resolve the decorator
    let checker = check_with_decorators(
        r#"
        using HTTP;
        @route("/api/pets")
        op listPets(): void;
    "#,
        vec![("route", "HTTP", "Operation")],
    );

    let op_type = checker
        .declared_types
        .get("listPets")
        .copied()
        .expect("operation should exist");

    let decs = match checker.get_type(op_type) {
        Some(Type::Operation(op)) => &op.decorators,
        _ => panic!("expected Operation type"),
    };

    assert_eq!(decs.len(), 1);
    assert!(
        decs[0].definition.is_some(),
        "decorator should be resolved via using namespace"
    );
}

#[test]
fn test_dotted_decorator_name_resolved() {
    // @HTTP.route(...) should also resolve correctly (dotted name)
    let checker = check_with_decorators(
        r#"
        @HTTP.route("/api/pets")
        op listPets(): void;
    "#,
        vec![("route", "HTTP", "Operation")],
    );

    let op_type = checker
        .declared_types
        .get("listPets")
        .copied()
        .expect("operation should exist");

    let decs = match checker.get_type(op_type) {
        Some(Type::Operation(op)) => &op.decorators,
        _ => panic!("expected Operation type"),
    };

    assert_eq!(decs.len(), 1);
    assert!(
        decs[0].definition.is_some(),
        "decorator should be resolved via dotted name (HTTP.route)"
    );
}

#[test]
fn test_decorator_target_type_validation() {
    // A decorator registered for "Operation" should warn when applied to Model
    let checker = check_with_decorators(
        r#"
        @HTTP.route("/api/pets")
        model Pet { name: string }
    "#,
        vec![("route", "HTTP", "Operation")],
    );

    assert!(
        has_diagnostic(&checker, "decorator-wrong-target"),
        "Should report decorator-wrong-target when applying Operation decorator to Model: {:?}",
        checker.diagnostics()
    );
}

#[test]
fn test_multiple_namespaces_using() {
    // Multiple custom namespaces should all work with using
    let checker = check_with_decorators(
        r#"
        using HTTP;
        using API;
        @route("/api/pets")
        @tag("pet")
        op listPets(): void;
    "#,
        vec![
            ("route", "HTTP", "Operation"),
            ("tag", "API", "Operation"),
        ],
    );

    let op_type = checker
        .declared_types
        .get("listPets")
        .copied()
        .expect("operation should exist");

    let decs = match checker.get_type(op_type) {
        Some(Type::Operation(op)) => &op.decorators,
        _ => panic!("expected Operation type"),
    };

    assert_eq!(decs.len(), 2, "should have 2 decorators");

    // Both should have resolved definitions
    for (i, dec) in decs.iter().enumerate() {
        assert!(
            dec.definition.is_some(),
            "decorator {} should have resolved definition",
            i
        );
    }
}

// ============================================================================
// P1-1: Namespace hierarchy population
// ============================================================================

#[test]
fn test_custom_namespace_has_decorator_declarations() {
    // Custom namespace created by register_decorator should have
    // decorator_declarations populated
    let checker = check_with_decorators(
        "",
        vec![("route", "HTTP", "Operation")],
    );

    let http_id = checker
        .declared_types
        .get("HTTP")
        .copied()
        .expect("HTTP namespace should exist");

    match checker.get_type(http_id) {
        Some(Type::Namespace(ns)) => {
            assert!(
                ns.decorator_declarations.contains_key("route"),
                "HTTP namespace should contain 'route' decorator"
            );
            assert!(
                ns.decorator_declaration_names.contains(&"route".to_string()),
                "HTTP namespace decorator_declaration_names should contain 'route'"
            );
        }
        _ => panic!("expected Namespace type"),
    }
}

#[test]
fn test_user_namespace_has_model_and_operation_names() {
    // User-defined namespace should have model_names and operation_names populated
    let checker = check_with_decorators(
        r#"
        namespace MyApp {
            model Pet { name: string }
            op list(): void;
        }
    "#,
        vec![],
    );

    let myapp_id = checker
        .declared_types
        .get("MyApp")
        .copied()
        .expect("MyApp namespace should exist");

    match checker.get_type(myapp_id) {
        Some(Type::Namespace(ns)) => {
            assert!(
                ns.models.contains_key("Pet"),
                "MyApp should contain Pet model"
            );
            assert!(
                ns.model_names.contains(&"Pet".to_string()),
                "MyApp model_names should contain 'Pet'"
            );
            assert!(
                ns.operations.contains_key("list"),
                "MyApp should contain list operation"
            );
            assert!(
                ns.operation_names.contains(&"list".to_string()),
                "MyApp operation_names should contain 'list'"
            );
        }
        _ => panic!("expected Namespace type"),
    }
}

#[test]
fn test_global_namespace_has_top_level_types() {
    // Global namespace should have top-level types populated
    let checker = check_with_decorators(
        r#"
        model Pet { name: string }
        op list(): void;
        enum Status { Active Inactive }
    "#,
        vec![],
    );

    let global_id = checker
        .global_namespace_type
        .expect("global namespace should exist");

    match checker.get_type(global_id) {
        Some(Type::Namespace(ns)) => {
            assert!(
                ns.models.contains_key("Pet"),
                "Global namespace should contain Pet model"
            );
            assert!(
                ns.operations.contains_key("list"),
                "Global namespace should contain list operation"
            );
            assert!(
                ns.enums.contains_key("Status"),
                "Global namespace should contain Status enum"
            );
        }
        _ => panic!("expected Namespace type"),
    }
}

// ============================================================================
// P1-2: Type fully qualified names
// ============================================================================

#[test]
fn test_fully_qualified_name_for_namespace_member() {
    // get_fully_qualified_name should return "MyApp.Pet" for a model in MyApp namespace
    use crate::checker::type_utils::get_fully_qualified_name;

    let checker = check_with_decorators(
        r#"
        namespace MyApp {
            model Pet { name: string }
        }
    "#,
        vec![],
    );

    let pet_id = checker
        .declared_types
        .get("Pet")
        .copied()
        .expect("Pet should exist");

    let fqn = get_fully_qualified_name(&checker.type_store, pet_id);
    assert_eq!(
        fqn, "MyApp.Pet",
        "FQN for model in namespace should be 'MyApp.Pet'"
    );
}

#[test]
fn test_fully_qualified_name_for_global_type() {
    // get_fully_qualified_name should return just the name for global scope
    use crate::checker::type_utils::get_fully_qualified_name;

    let checker = check_with_decorators(
        r#"
        model Pet { name: string }
    "#,
        vec![],
    );

    let pet_id = checker
        .declared_types
        .get("Pet")
        .copied()
        .expect("Pet should exist");

    let fqn = get_fully_qualified_name(&checker.type_store, pet_id);
    assert_eq!(
        fqn, "Pet",
        "FQN for global model should just be 'Pet'"
    );
}

#[test]
fn test_fully_qualified_name_for_nested_namespace() {
    // get_fully_qualified_name should return "A.B.Type" for deeply nested types
    use crate::checker::type_utils::get_fully_qualified_name;

    let checker = check_with_decorators(
        r#"
        namespace A {
            namespace B {
                model Inner { name: string }
            }
        }
    "#,
        vec![],
    );

    let inner_id = checker
        .declared_types
        .get("Inner")
        .copied()
        .expect("Inner should exist");

    let fqn = get_fully_qualified_name(&checker.type_store, inner_id);
    assert_eq!(
        fqn, "A.B.Inner",
        "FQN for deeply nested type should be 'A.B.Inner'"
    );
}

// ============================================================================
// Regression: Bug 1 — Decorator resolution without explicit `using`
// ============================================================================

#[test]
fn test_decorator_resolved_without_using_in_sub_namespace() {
    // A decorator registered in a sub-namespace (e.g., "Llm") should be
    // resolvable even without an explicit `using Llm;` declaration,
    // thanks to recursive namespace tree search.
    let checker = check_with_decorators(
        r#"
        @command("status")
        op gitStatus(): string;
    "#,
        vec![("command", "Llm", "Operation")],
    );

    let op_id = checker
        .declared_types
        .get("gitStatus")
        .copied()
        .expect("gitStatus should exist");

    let op = match checker.get_type(op_id) {
        Some(Type::Operation(o)) => o,
        _ => panic!("expected Operation"),
    };

    assert_eq!(op.decorators.len(), 1, "should have 1 decorator");
    assert!(
        op.decorators[0].definition.is_some(),
        "decorator should be resolved without explicit using"
    );
}

#[test]
fn test_decorator_resolved_without_using_dotted_namespace() {
    // A decorator registered in a dotted namespace (e.g., "AnyUse.CLI")
    // should be resolvable without `using AnyUse.CLI;`.
    let checker = check_with_decorators(
        r#"
        @cliFlag verbose?: boolean;
    "#,
        vec![("cliFlag", "AnyUse.CLI", "unknown")],
    );

    // Should not have invalid-ref diagnostic
    assert!(
        !has_diagnostic(&checker, "invalid-ref"),
        "Should NOT report invalid-ref for decorator in sub-namespace: {:?}",
        checker.diagnostics()
    );
}

// ============================================================================
// Regression: Bug 2 — Pre-registered types namespace field
// ============================================================================

#[test]
fn test_interface_namespace_correct_in_user_namespace() {
    // Interface declared inside a user namespace should have its namespace
    // field pointing to the parent namespace, not the global namespace.
    let checker = check_with_decorators(
        r#"
        namespace Llm {
            interface Chat { op create(): void; }
        }
    "#,
        vec![],
    );

    let llm_id = checker
        .declared_types
        .get("Llm")
        .copied()
        .expect("Llm should exist");

    let chat_id = checker
        .declared_types
        .get("Chat")
        .copied()
        .expect("Chat should exist");

    let chat_iface = match checker.get_type(chat_id) {
        Some(Type::Interface(i)) => i,
        _ => panic!("expected Interface"),
    };

    assert_eq!(
        chat_iface.namespace,
        Some(llm_id),
        "Chat.namespace should point to Llm, not global"
    );
}

#[test]
fn test_model_namespace_correct_in_user_namespace() {
    // Model declared inside a user namespace should have correct namespace.
    let checker = check_with_decorators(
        r#"
        namespace Llm {
            model Request { id: string }
        }
    "#,
        vec![],
    );

    let llm_id = checker
        .declared_types
        .get("Llm")
        .copied()
        .expect("Llm should exist");

    let req_id = checker
        .declared_types
        .get("Request")
        .copied()
        .expect("Request should exist");

    let model = match checker.get_type(req_id) {
        Some(Type::Model(m)) => m,
        _ => panic!("expected Model"),
    };

    assert_eq!(
        model.namespace,
        Some(llm_id),
        "Request.namespace should point to Llm"
    );
}

#[test]
fn test_enum_namespace_correct_in_user_namespace() {
    // Enum declared inside a user namespace should have correct namespace.
    let checker = check_with_decorators(
        r#"
        namespace Llm {
            enum Status { Active Inactive }
        }
    "#,
        vec![],
    );

    let llm_id = checker
        .declared_types
        .get("Llm")
        .copied()
        .expect("Llm should exist");

    let status_id = checker
        .declared_types
        .get("Status")
        .copied()
        .expect("Status should exist");

    let enum_type = match checker.get_type(status_id) {
        Some(Type::Enum(e)) => e,
        _ => panic!("expected Enum"),
    };

    assert_eq!(
        enum_type.namespace,
        Some(llm_id),
        "Status.namespace should point to Llm"
    );
}

#[test]
fn test_interface_namespace_correct_with_decorator_overlap() {
    // When a decorator is registered in "Llm" namespace AND user code
    // also declares "Llm" namespace with an interface, the interface's
    // namespace should still point to the correct Llm namespace.
    let checker = check_with_decorators(
        r#"
        namespace Llm {
            @command("status")
            op gitStatus(): string;
            interface Chat { op create(): void; }
        }
    "#,
        vec![("command", "Llm", "Operation")],
    );

    let llm_id = checker
        .declared_types
        .get("Llm")
        .copied()
        .expect("Llm should exist");

    // Operation should be in Llm namespace
    let op_id = checker
        .declared_types
        .get("gitStatus")
        .copied()
        .expect("gitStatus should exist");

    let op = match checker.get_type(op_id) {
        Some(Type::Operation(o)) => o,
        _ => panic!("expected Operation"),
    };

    assert_eq!(
        op.namespace,
        Some(llm_id),
        "gitStatus.namespace should point to Llm"
    );

    // Interface should be in Llm namespace
    let chat_id = checker
        .declared_types
        .get("Chat")
        .copied()
        .expect("Chat should exist");

    let chat = match checker.get_type(chat_id) {
        Some(Type::Interface(i)) => i,
        _ => panic!("expected Interface"),
    };

    assert_eq!(
        chat.namespace,
        Some(llm_id),
        "Chat.namespace should point to Llm"
    );
}

// ============================================================================
// Regression: Bug 3 — FQN calculation (consequence of Bug 2)
// ============================================================================

#[test]
fn test_fqn_for_interface_in_namespace() {
    // FQN for interface in namespace should include namespace prefix.
    use crate::checker::type_utils::get_fully_qualified_name;

    let checker = check_with_decorators(
        r#"
        namespace Llm {
            interface Chat { op create(): void; }
        }
    "#,
        vec![],
    );

    let chat_id = checker
        .declared_types
        .get("Chat")
        .copied()
        .expect("Chat should exist");

    let fqn = get_fully_qualified_name(&checker.type_store, chat_id);
    assert_eq!(
        fqn, "Llm.Chat",
        "FQN for interface in namespace should be 'Llm.Chat'"
    );
}

#[test]
fn test_fqn_for_operation_in_namespace() {
    // FQN for operation in namespace should include namespace prefix.
    use crate::checker::type_utils::get_fully_qualified_name;

    let checker = check_with_decorators(
        r#"
        namespace Llm {
            op create(): void;
        }
    "#,
        vec![],
    );

    let op_id = checker
        .declared_types
        .get("create")
        .copied()
        .expect("create should exist");

    let fqn = get_fully_qualified_name(&checker.type_store, op_id);
    assert_eq!(
        fqn, "Llm.create",
        "FQN for operation in namespace should be 'Llm.create'"
    );
}

#[test]
fn test_fqn_for_enum_in_namespace() {
    // FQN for enum in namespace should include namespace prefix.
    use crate::checker::type_utils::get_fully_qualified_name;

    let checker = check_with_decorators(
        r#"
        namespace Llm {
            enum Status { Active Inactive }
        }
    "#,
        vec![],
    );

    let status_id = checker
        .declared_types
        .get("Status")
        .copied()
        .expect("Status should exist");

    let fqn = get_fully_qualified_name(&checker.type_store, status_id);
    assert_eq!(
        fqn, "Llm.Status",
        "FQN for enum in namespace should be 'Llm.Status'"
    );
}

#[test]
fn test_fqn_for_interface_with_decorator_overlap() {
    // FQN should work correctly when decorator namespace overlaps with user namespace.
    use crate::checker::type_utils::get_fully_qualified_name;

    let checker = check_with_decorators(
        r#"
        namespace Llm {
            @command("status")
            op gitStatus(): string;
            interface Chat { op create(): void; }
        }
    "#,
        vec![("command", "Llm", "Operation")],
    );

    let chat_id = checker
        .declared_types
        .get("Chat")
        .copied()
        .expect("Chat should exist");

    let fqn = get_fully_qualified_name(&checker.type_store, chat_id);
    assert_eq!(
        fqn, "Llm.Chat",
        "FQN should be 'Llm.Chat' even with decorator overlap"
    );
}
