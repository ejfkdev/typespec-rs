#[cfg(test)]
#[allow(clippy::module_inception)]
mod template_tests {
    use crate::checker::Type;
    use crate::checker::test_utils::{all_diagnostics, check};
    use crate::parser::parse;

    // ==================== Template Tests from TypeSpec templates.test.ts ====================

    #[test]
    fn test_parse_template_model_simple() {
        // Model with template parameter
        let result = parse("model A<T> {}");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_template_model_multiple_params() {
        // Model with multiple template parameters
        let result = parse("model A<T, U, V> {}");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_template_model_with_constraint() {
        // Model with template parameter constraint
        let result = parse("model A<T extends object> { a: T }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_template_model_with_default() {
        // Model with template parameter default
        let result = parse(r#"model A<T = string> { a: T }"#);
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_template_model_with_constraint_and_default() {
        // Model with template parameter constraint and default
        let result = parse(r#"model A<T extends string = "hi"> { a: T }"#);
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_template_model_with_multiple_constraints() {
        // Model with multiple template parameters with constraints
        let result = parse("model A<T extends string, U extends object> {}");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_template_reference_simple() {
        // Template instantiation
        let result = parse("model B { foo: A<string> }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_template_reference_multiple_args() {
        // Template instantiation with multiple arguments
        let result = parse("model B { foo: A<string, int32> }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_template_reference_named_arg() {
        // Template instantiation with named argument
        let result = parse("model B { foo: A<T = string> }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_template_reference_named_args_out_of_order() {
        // Template instantiation with named arguments out of order
        let result = parse("model B { foo: A<U = int32, T = string> }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_template_reference_with_default() {
        // Template with default template args
        let result = parse("model B { foo: A<\"bye\"> }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_template_reference_constraint() {
        // Template with constraint
        let result = parse("model A<T extends string> { b: B<T> }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_template_reference_default_reference() {
        // Template default referencing another parameter
        let result = parse("model A<T, X = T> { a: T, b: X }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_template_reference_default_with_constraint() {
        // Template with constraint on default reference
        let result = parse("model Foo<A extends string, B extends string = A> { b: B }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_template_model_with_valueof_constraint() {
        // Template parameter with valueof constraint
        let result = parse("model A<T extends valueof string> {}");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_template_model_with_nested_constraint() {
        // Template parameter with nested constraint
        let result = parse("model Test<A extends {name: A}> {}");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_interface_template() {
        // Interface with template parameter
        let result = parse("interface A<T> {}");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_interface_template_with_constraint() {
        // Interface with template parameter constraint
        let result = parse("interface A<T extends object> { op foo(): T; }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_interface_template_with_default() {
        // Interface with template parameter default
        let result = parse("interface A<T = string> { op foo(): T; }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_interface_template_with_operation() {
        // Interface with template parameter used in operation
        let result = parse("interface A<T> { op foo(): T; }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_interface_template_operation_with_default() {
        // Interface with template parameter with default in operation
        let result = parse("interface A<T> { op foo<R = T, P = R>(params: P): R; }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_union_template() {
        // Union with template parameter
        let result = parse("union A<T> { value: T }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_union_template_multiple() {
        // Union with multiple template parameters
        let result = parse("union Result<T, E> { value: T; error: E; }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_alias_template() {
        // Alias with template parameter
        let result = parse("alias Optional<T> = T | null;");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_alias_template_with_default() {
        // Alias with template parameter default
        let result = parse("alias MyAlias<T = string> = T;");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_alias_template_reference() {
        // Alias with template reference
        let result = parse("alias Bar<T> = Foo<T>;");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_operation_template() {
        // Operation with template parameter
        let result = parse("op foo<T>(value: T): T;");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_operation_template_with_constraint() {
        // Operation with template parameter constraint
        let result = parse("op Action<T extends {}>(): T;");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_operation_template_multiple() {
        // Operation with multiple template parameters
        let result = parse("op convert<T, V>(value: T): V;");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_model_template_instantiation_in_property() {
        // Model using template instantiation in property
        let result = parse("model B { foo: A<string> }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_model_template_named_in_property() {
        // Model using named template arguments
        let result = parse("model B { foo: A<T = string> }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_model_template_multiple_named_args() {
        // Model with multiple named template arguments
        let result = parse("model B { foo: A<T = int32, U = string> }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_model_template_with_model_default() {
        // Template with model default
        let result = parse("model A<T = string> { a: T }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_template_default_with_object() {
        // Template default with object type
        let result = parse("model A<T, X = {t: T}> { b: X }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_template_default_reference_chain() {
        // Template default referencing another template
        let result = parse("model A<T, X = Foo<T>> { b: X }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_model_extends_with_template() {
        // Model extends with template
        let result = parse("model foo<T> extends bar<T> { }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_model_is_with_template() {
        // Model is with template
        let result = parse("model Car<T> is Vehicle<T>;");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_template_with_decorator() {
        // Template with decorator
        let result = parse("@format(\"json\") model Foo<T> {}");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_template_with_decorator_on_property() {
        // Template with decorator on property
        let result = parse("model Foo<T> { @format prop: T }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_template_with_multiple_decorators() {
        // Template with multiple decorators
        let result = parse("@foo @bar model Foo<T> {}");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_interface_extends_with_template() {
        // Interface extends with template
        let result = parse("interface Foo<T> extends Bar<T> {}");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_interface_extends_multiple_with_template() {
        // Interface extends multiple with template
        let result = parse("interface Foo<T> extends Bar<T>, Baz<T> {}");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_template_array_type() {
        // Template with array type
        let result = parse("model B { x: Foo<number, string>[]; }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_template_with_union_type() {
        // Template with union type
        let result = parse("model A { x: B | C }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_template_with_intersection_type() {
        // Template with intersection type
        let result = parse("model A { x: B & C }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_template_empty_args() {
        // Template with empty args
        let result = parse("model A<T> { foo: A }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_template_with_nested_template() {
        // Template with nested template
        let result = parse("model A { x: B<C<D>> }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_template_with_function_type() {
        // Template with function type
        let result = parse("alias Test = Foo<fn()>;");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_template_with_function_return_type() {
        // Template with function return type
        let result = parse("alias Test = Foo<fn() => string>;");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_template_with_function_params() {
        // Template with function parameters
        let result = parse("alias Test = Foo<fn(a: string, b: int) => string>;");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_template_with_valueof_constraint() {
        // Template with valueof constraint
        let result = parse("model A<T extends valueof {a: string, b: int32}> {}");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_template_with_valueof_array_constraint() {
        // Template with valueof array constraint
        let result = parse("model A<T extends valueof int8[]> {}");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_template_with_tuple_default() {
        // Template with tuple default
        let result = parse("model Template<T = []> { }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_interface_template_with_default_in_operation() {
        // Interface with template default in operation
        let result =
            parse("interface MyInterface<A, B = string> { op foo<R = B, P = R>(params: P): R; }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_interface_template_with_override_default() {
        // Interface with template default override
        let result = parse("interface A<T> { op foo<U = T>(): U; }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_template_reference_with_model_expression_default() {
        // Template reference with model expression default
        let result = parse("model A<T, X = {t: T}> { b: X }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_template_with_parent_reference() {
        // Template with parent parameter reference
        let result = parse("interface A<T> { op foo<R = T, P = R>(params: P): R; }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_model_template_with_template_on_property() {
        // Model template with decorator on property using template param
        let result = parse("model Foo<T> { @mark(T) prop: T }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_interface_template_with_decorator_on_operation() {
        // Interface template with decorator on operation parameter
        let result = parse("interface Test<T> { foo(@mark(T) prop: string;): void; }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_template_with_nested_model_decorator() {
        // Template with nested model decorator
        let result = parse("model Foo<T> { nested: { @mark(T) prop: string; } }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_template_with_union_decorator() {
        // Template with decorator in union
        let result = parse("model Foo<T> { nested: string | { @mark(T) prop: string; } }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_template_augment_decorator() {
        // Template with augment decorator
        let result = parse("@@format(Foo);");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_template_augment_decorator_with_args() {
        // Template with augment decorator and args
        let result = parse(r#"@@doc(Foo, "description");"#);
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_template_with_initializer() {
        // Template with property initializer
        let result = parse("model Config { name: string = \"default\"; }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_template_with_optional_property() {
        // Template with optional property
        let result = parse("model Person { name?: string; age: int32; }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_template_with_spread() {
        // Template with spread property
        let result = parse("model Car { ...BaseCar; make: string; }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_template_reference_with_object_literal() {
        // Template reference with object literal default
        let result = parse(r#"const obj = #{name: "test", value: 42};"#);
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_template_reference_with_array_literal() {
        // Template reference with array literal default
        let result = parse(r#"const arr = #["a", "b", "c"];"#);
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_model_template_with_inline_model() {
        // Model template with inline model
        let result = parse(r#"model Car { engine: { type: "v8" } }"#);
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_template_with_type_reference() {
        // Template with type reference
        let result = parse("model A { x: Foo.Bar<T>; }");
        assert!(result.diagnostics.is_empty());
    }

    // NOTE: Foo.Bar<T>.baz and foo.bar<T>() are NOT valid TypeSpec syntax.
    // Template arguments always end the expression chain - you cannot add
    // member access or call after template args. These tests were based on
    // incorrect assumptions and have been removed per T2.4.

    #[test]
    fn test_parse_template_with_valueof_in_default() {
        // Template with valueof in default
        let result = parse("model A<T extends valueof string = valueof string> {}");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_template_special_keywords_as_identifiers() {
        // Template with special keywords as identifiers
        let result = parse("model A<T> { model: T, enum: T }");
        assert!(result.diagnostics.is_empty());
    }

    // ============================================================================
    // Checker-level template tests
    // ============================================================================

    /// Get all diagnostics (parse + checker) as a single Vec

    #[test]
    fn test_template_model_is_declared() {
        let checker = check("model Pair<K, V> { key: K; value: V; }");
        assert!(
            checker.declared_types.contains_key("Pair"),
            "Pair should be in declared_types"
        );
    }

    #[test]
    fn test_template_model_instantiation_via_extends() {
        let checker = check(
            "model Pair<K, V> { key: K; value: V; } model StringPair extends Pair<string, int32> {}",
        );
        assert!(checker.declared_types.contains_key("Pair"));
        assert!(checker.declared_types.contains_key("StringPair"));
    }

    #[test]
    fn test_template_model_instantiation_via_is() {
        let checker = check("model Box<T> { content: T; } model MyBox is Box<string> {}");
        let my_box_type = checker.declared_types.get("MyBox").copied().unwrap();
        let t = checker.get_type(my_box_type).cloned().unwrap();
        match t {
            Type::Model(m) => {
                assert!(
                    m.source_model.is_some(),
                    "MyBox should have source_model from 'is'"
                );
            }
            _ => panic!("Expected Model type"),
        }
    }

    #[test]
    fn test_template_interface_is_declared() {
        let checker = check("interface Container<T> { get(): T; }");
        assert!(
            checker.declared_types.contains_key("Container"),
            "Container should be in declared_types"
        );
    }

    #[test]
    fn test_template_union_is_declared() {
        let checker = check("union Result<T, E> { ok: T; err: E; }");
        assert!(
            checker.declared_types.contains_key("Result"),
            "Result should be in declared_types"
        );
    }

    #[test]
    fn test_template_alias_is_declared() {
        let checker = check("alias Optional<T> = T | null;");
        assert!(
            checker.declared_types.contains_key("Optional"),
            "Optional should be in declared_types"
        );
    }

    #[test]
    fn test_template_scalar_is_declared() {
        let checker = check("scalar Id<T extends valueof string> extends string;");
        assert!(
            checker.declared_types.contains_key("Id"),
            "Id should be in declared_types"
        );
    }

    #[test]
    fn test_template_operation_is_declared() {
        let checker = check("op create<T>(value: T): T;");
        // Operations with templates should still be declared
        assert!(
            checker.declared_types.contains_key("create"),
            "create operation should be in declared_types"
        );
    }

    #[test]
    fn test_template_model_with_decorator() {
        let checker = check("@doc model Foo<T> { x: T; }");
        let foo_type = checker.declared_types.get("Foo").copied().unwrap();
        let t = checker.get_type(foo_type).cloned().unwrap();
        match t {
            Type::Model(m) => {
                assert_eq!(m.decorators.len(), 1, "Foo should have 1 decorator");
            }
            _ => panic!("Expected Model type"),
        }
    }

    #[test]
    fn test_template_model_in_namespace() {
        let checker = check("namespace N { model Foo<T> { x: T; } }");
        let ns_type = checker.declared_types.get("N").copied().unwrap();
        let t = checker.get_type(ns_type).cloned().unwrap();
        match t {
            Type::Namespace(ns) => {
                assert!(
                    ns.models.contains_key("Foo"),
                    "Namespace should contain Foo model"
                );
            }
            _ => panic!("Expected Namespace type"),
        }
    }

    // ==================== Template Diagnostic Tests (ported from TS templates.test.ts) ====================

    /// Ported from TS: "emit diagnostics when using template params on non templated model"
    #[test]
    fn test_diagnostic_template_args_on_non_templated_model() {
        let checker = check(
            "
            model A {}
            model B {
              foo: A<string>
            };
        ",
        );
        let diags = checker.diagnostics();
        let matching: Vec<_> = diags
            .iter()
            .filter(|d| d.code == "invalid-template-args")
            .collect();
        assert!(
            !matching.is_empty(),
            "Should emit invalid-template-args for template args on non-templated model: {:?}",
            diags
        );
        // Check the message content
        let msg = &matching[0].message;
        assert!(
            msg.contains("non-templated") || msg.contains("Can't pass"),
            "Message should mention non-templated type: got '{}'",
            msg
        );
    }

    /// Ported from TS: "emit diagnostics when using template without passing any arguments"
    #[test]
    fn test_diagnostic_template_missing_required_arg() {
        let checker = check(
            "
            model A<T> {}
            model B {
              foo: A
            };
        ",
        );
        let diags = checker.diagnostics();
        let matching: Vec<_> = diags
            .iter()
            .filter(|d| d.code == "invalid-template-args")
            .collect();
        assert!(
            !matching.is_empty(),
            "Should emit invalid-template-args for missing required arg T: {:?}",
            diags
        );
        let msg = &matching[0].message;
        assert!(
            msg.contains("T") && msg.contains("required"),
            "Message should mention T is required: got '{}'",
            msg
        );
    }

    /// Ported from TS: "emit diagnostics when using template with too many arguments"
    #[test]
    fn test_diagnostic_template_too_many_args() {
        let checker = check(
            "
            model A<T> {}
            model B {
              foo: A<string, string>
            };
        ",
        );
        let diags = checker.diagnostics();
        let matching: Vec<_> = diags
            .iter()
            .filter(|d| d.code == "invalid-template-args")
            .collect();
        assert!(
            !matching.is_empty(),
            "Should emit invalid-template-args for too many template args: {:?}",
            diags
        );
        let msg = &matching[0].message;
        assert!(
            msg.contains("Too many") || msg.contains("too many"),
            "Message should mention too many arguments: got '{}'",
            msg
        );
    }

    /// Ported from TS: "emits diagnostics when using too few template parameters"
    #[test]
    fn test_diagnostic_template_too_few_args_with_default() {
        let checker = check(
            "
            model A<T, U, V = \"hi\"> { a: T, b: U, c: V }
            model B {
              foo: A<\"bye\">
            };
        ",
        );
        let diags = checker.diagnostics();
        let matching: Vec<_> = diags
            .iter()
            .filter(|d| d.code == "invalid-template-args")
            .collect();
        assert!(
            !matching.is_empty(),
            "Should emit invalid-template-args for missing required arg U: {:?}",
            diags
        );
        let msg = &matching[0].message;
        assert!(
            msg.contains("U") && msg.contains("required"),
            "Message should mention U is required: got '{}'",
            msg
        );
    }

    /// Ported from TS: "emits diagnostics when non-defaulted template parameter comes after defaulted one"
    #[test]
    fn test_diagnostic_default_required() {
        let diags = all_diagnostics("model A<T = \"hi\", U> { a: T, b: U }");
        assert!(
            diags.iter().any(|d| d.code == "default-required"),
            "Should emit default-required for non-defaulted param after defaulted: {:?}",
            diags
        );
    }

    /// Ported from TS: "emits diagnostics when defaulted template use later template parameter"
    #[test]
    fn test_diagnostic_invalid_template_default() {
        let checker = check("model A<A = B, B = \"hi\"> { a: A, b: B }");
        let diags = checker.diagnostics();
        assert!(
            diags.iter().any(|d| d.code == "invalid-template-default"),
            "Should emit invalid-template-default for default referencing later param: {:?}",
            diags
        );
    }

    /// Ported from TS: "emits diagnostics when defaulted template use later template parameter in complex type"
    #[test]
    fn test_diagnostic_invalid_template_default_in_union() {
        let checker = check("model A<A = \"one\" | B, B = \"hi\"> { a: A, b: B }");
        let diags = checker.diagnostics();
        assert!(
            diags.iter().any(|d| d.code == "invalid-template-default"),
            "Should emit invalid-template-default for default referencing later param in union: {:?}",
            diags
        );
    }

    /// Ported from TS: "emits diagnostic when constraint reference itself"
    #[test]
    fn test_diagnostic_circular_constraint_self() {
        let checker = check("model Test<A extends A> {}");
        let diags = checker.diagnostics();
        assert!(
            diags.iter().any(|d| d.code == "circular-constraint"),
            "Should emit circular-constraint for self-referencing constraint: {:?}",
            diags
        );
    }

    /// Ported from TS: "emits diagnostic when constraint reference other parameter in circular constraint"
    #[test]
    fn test_diagnostic_circular_constraint_mutual() {
        let checker = check("model Test<A extends B, B extends A> {}");
        let diags = checker.diagnostics();
        assert!(
            diags.iter().any(|d| d.code == "circular-constraint"),
            "Should emit circular-constraint for mutually circular constraints: {:?}",
            diags
        );
    }

    /// Ported from TS: "emits diagnostic when constraint reference itself inside an expression"
    #[test]
    fn test_diagnostic_circular_constraint_in_expression() {
        let checker = check("model Test<A extends {name: A}> {}");
        let diags = checker.diagnostics();
        assert!(
            diags.iter().any(|d| d.code == "circular-constraint"),
            "Should emit circular-constraint for self-referencing constraint in expression: {:?}",
            diags
        );
    }

    // Invalid template default on interface
    #[test]
    fn test_diagnostic_invalid_template_default_interface() {
        let checker = check("interface A<A = B, B = string> { op foo(): void; }");
        let diags = checker.diagnostics();
        assert!(
            diags.iter().any(|d| d.code == "invalid-template-default"),
            "Should emit invalid-template-default for interface default referencing later param: {:?}",
            diags
        );
    }

    // Invalid template default on scalar
    #[test]
    fn test_diagnostic_invalid_template_default_scalar() {
        let checker = check("scalar A<A = B, B = string> extends string;");
        let diags = checker.diagnostics();
        assert!(
            diags.iter().any(|d| d.code == "invalid-template-default"),
            "Should emit invalid-template-default for scalar default referencing later param: {:?}",
            diags
        );
    }

    // Invalid template default on alias
    #[test]
    fn test_diagnostic_invalid_template_default_alias() {
        let checker = check("alias A<A = B, B = string> = B;");
        let diags = checker.diagnostics();
        assert!(
            diags.iter().any(|d| d.code == "invalid-template-default"),
            "Should emit invalid-template-default for alias default referencing later param: {:?}",
            diags
        );
    }

    // Circular constraint on interface template parameters
    // TS: same pattern as model, but for interface declarations
    #[test]
    fn test_diagnostic_circular_constraint_interface_self() {
        let checker = check("interface Test<A extends A> { op foo(): void; }");
        let diags = checker.diagnostics();
        assert!(
            diags.iter().any(|d| d.code == "circular-constraint"),
            "Should emit circular-constraint for self-referencing interface constraint: {:?}",
            diags
        );
    }

    #[test]
    fn test_diagnostic_circular_constraint_interface_mutual() {
        let checker = check("interface Test<A extends B, B extends A> { op foo(): void; }");
        let diags = checker.diagnostics();
        assert!(
            diags.iter().any(|d| d.code == "circular-constraint"),
            "Should emit circular-constraint for mutual interface constraint: {:?}",
            diags
        );
    }

    // Circular constraint on scalar template parameters
    #[test]
    fn test_diagnostic_circular_constraint_scalar_self() {
        let checker = check("scalar Test<A extends A> extends string;");
        let diags = checker.diagnostics();
        assert!(
            diags.iter().any(|d| d.code == "circular-constraint"),
            "Should emit circular-constraint for self-referencing scalar constraint: {:?}",
            diags
        );
    }

    #[test]
    fn test_diagnostic_circular_constraint_scalar_mutual() {
        let checker = check("scalar Test<A extends B, B extends A> extends string;");
        let diags = checker.diagnostics();
        assert!(
            diags.iter().any(|d| d.code == "circular-constraint"),
            "Should emit circular-constraint for mutual scalar constraint: {:?}",
            diags
        );
    }

    // Circular constraint on alias template parameters
    #[test]
    fn test_diagnostic_circular_constraint_alias_self() {
        let checker = check("alias Test<A extends A> = A;");
        let diags = checker.diagnostics();
        assert!(
            diags.iter().any(|d| d.code == "circular-constraint"),
            "Should emit circular-constraint for self-referencing alias constraint: {:?}",
            diags
        );
    }

    #[test]
    fn test_diagnostic_circular_constraint_alias_mutual() {
        let checker = check("alias Test<A extends B, B extends A> = A;");
        let diags = checker.diagnostics();
        assert!(
            diags.iter().any(|d| d.code == "circular-constraint"),
            "Should emit circular-constraint for mutual alias constraint: {:?}",
            diags
        );
    }

    // Circular constraint on union template parameters
    #[test]
    fn test_diagnostic_circular_constraint_union_self() {
        let checker = check("union Test<A extends A> { a: A; }");
        let diags = checker.diagnostics();
        assert!(
            diags.iter().any(|d| d.code == "circular-constraint"),
            "Should emit circular-constraint for self-referencing union constraint: {:?}",
            diags
        );
    }

    // Circular constraint on operation template parameters
    #[test]
    fn test_diagnostic_circular_constraint_operation_self() {
        let checker = check("op test<A extends A>(): void;");
        let diags = checker.diagnostics();
        assert!(
            diags.iter().any(|d| d.code == "circular-constraint"),
            "Should emit circular-constraint for self-referencing operation constraint: {:?}",
            diags
        );
    }

    /// Ported from TS: "allows default template parameters"
    #[test]
    fn test_default_template_parameters() {
        let checker = check(
            "
            model A<T, U = \"hi\"> { a: T, b: U }
            model B {
              foo: A<\"bye\">
            };
        ",
        );
        let b_type = checker.declared_types.get("B").copied().unwrap();
        let t = checker.get_type(b_type).cloned().unwrap();
        match t {
            Type::Model(m) => {
                assert!(
                    m.properties.contains_key("foo"),
                    "B should have foo property"
                );
                let foo_prop_id = m.properties.get("foo").copied().unwrap();
                let foo_prop = checker.get_type(foo_prop_id).cloned().unwrap();
                match foo_prop {
                    Type::ModelProperty(p) => {
                        // The instantiated A<"bye"> should be a model
                        let foo_type = checker.get_type(p.r#type).cloned().unwrap();
                        assert!(
                            matches!(foo_type, Type::Model(_)),
                            "foo type should be Model, got {:?}",
                            foo_type.kind_name()
                        );
                    }
                    _ => panic!("Expected ModelProperty"),
                }
            }
            _ => panic!("Expected Model type"),
        }
    }

    /// Ported from TS: "allows default template parameters that are models"
    #[test]
    fn test_default_template_parameter_model_type() {
        let checker = check(
            "
            model A<T = string> { a: T }
            model B {
              foo: A
            };
        ",
        );
        let b_type = checker.declared_types.get("B").copied().unwrap();
        let t = checker.get_type(b_type).cloned().unwrap();
        match t {
            Type::Model(m) => {
                assert!(
                    m.properties.contains_key("foo"),
                    "B should have foo property"
                );
            }
            _ => panic!("Expected Model type"),
        }
    }

    /// Ported from TS: "cache indeterminate types"
    /// Same template with same args should return the same type instance
    #[test]
    fn test_cache_template_instances() {
        let checker = check(
            "
            model Template<T> {t: T}
            model Test {
              a: Template<\"a\">;
              b: Template<\"a\">;
            }
        ",
        );
        let test_type = checker.declared_types.get("Test").copied().unwrap();
        let t = checker.get_type(test_type).cloned().unwrap();
        match t {
            Type::Model(m) => {
                let a_prop_id = m.properties.get("a").copied().unwrap();
                let b_prop_id = m.properties.get("b").copied().unwrap();
                let a_prop = checker.get_type(a_prop_id).cloned().unwrap();
                let b_prop = checker.get_type(b_prop_id).cloned().unwrap();
                match (a_prop, b_prop) {
                    (Type::ModelProperty(a), Type::ModelProperty(b)) => {
                        assert_eq!(
                            a.r#type, b.r#type,
                            "Template instances with same args should be the same type"
                        );
                    }
                    _ => panic!("Expected ModelProperty"),
                }
            }
            _ => panic!("Expected Model type"),
        }
    }

    /// TS: template instance should be the exact same when passing value that is the same as the default
    #[test]
    fn test_template_instance_same_with_default_value() {
        let checker = check(
            "
            model Foo<A = string, B = string> { a: A, b: B }
            model Test {
              a: Foo;
              b: Foo<string>;
              c: Foo<string, string>;
            };
        ",
        );
        let test_type = checker.declared_types.get("Test").copied().unwrap();
        let t = checker.get_type(test_type).cloned().unwrap();
        match t {
            Type::Model(m) => {
                let a_prop = checker
                    .get_type(m.properties.get("a").copied().unwrap())
                    .cloned()
                    .unwrap();
                let b_prop = checker
                    .get_type(m.properties.get("b").copied().unwrap())
                    .cloned()
                    .unwrap();
                let c_prop = checker
                    .get_type(m.properties.get("c").copied().unwrap())
                    .cloned()
                    .unwrap();
                match (a_prop, b_prop, c_prop) {
                    (Type::ModelProperty(a), Type::ModelProperty(b), Type::ModelProperty(c)) => {
                        assert_eq!(
                            a.r#type, b.r#type,
                            "Foo and Foo<string> should be same instance"
                        );
                        assert_eq!(
                            a.r#type, c.r#type,
                            "Foo and Foo<string, string> should be same instance"
                        );
                    }
                    _ => panic!("Expected ModelProperty"),
                }
            }
            _ => panic!("Expected Model type"),
        }
    }

    /// Template auto-instantiate when all params have defaults
    #[test]
    fn test_template_auto_instantiate_with_defaults() {
        // When referencing a template without args and all params have defaults,
        // it should auto-instantiate with the defaults.
        let checker = check(
            "
            model Foo<A = string, B = string> { a: A, b: B }
            model Test {
                x: Foo;
            }
        ",
        );
        let diags = checker.diagnostics();
        // Should not report duplicate-property or other errors
        assert!(
            !diags.iter().any(|d| d.code == "duplicate-property"),
            "Should have no duplicate-property: {:?}",
            diags
        );
        // Check that Test has property x
        assert!(
            checker.declared_types.contains_key("Test"),
            "Test should be in declared_types"
        );
    }

    // ========================================================================
    // Additional tests ported from TS templates.test.ts
    // ========================================================================

    /// Ported from TS: "emits diagnostics for template parameter defaults that are incorrect"
    #[test]
    fn test_diagnostic_template_default_incorrect() {
        let checker = check("model A<T = Record> { a: T }");
        let diags = checker.diagnostics();
        assert!(
            diags.iter().any(|d| d.code == "invalid-template-args"),
            "Should report invalid-template-args for incorrect default: {:?}",
            diags
        );
    }

    /// Ported from TS: "emits diagnostics when passing value to template parameter without constraint"
    #[test]
    fn test_diagnostic_value_in_type() {
        let checker = check(
            r#"
            model A<T> { }
            const a = "abc";
            alias B = A<a>;
        "#,
        );
        let diags = checker.diagnostics();
        assert!(
            diags.iter().any(|d| d.code == "value-in-type"),
            "Should report value-in-type when passing value to unconstrained param: {:?}",
            diags
        );
    }

    /// Ported from TS: "emit diagnostics if template default is not assignable to constraint"
    #[test]
    fn test_diagnostic_template_default_not_assignable() {
        let checker = check("model A<T extends string = 123> { a: T }");
        let diags = checker.diagnostics();
        assert!(
            diags.iter().any(|d| d.code == "unassignable"),
            "Should report unassignable when default doesn't match constraint: {:?}",
            diags
        );
    }

    /// Ported from TS: "emit diagnostics if template reference arg is not assignable to constraint"
    #[test]
    fn test_diagnostic_template_arg_not_assignable() {
        let checker = check(
            r#"
            model A<T extends string> { a: T }
            model B {
              a: A<456>
            }
        "#,
        );
        let diags = checker.diagnostics();
        assert!(
            diags.iter().any(|d| d.code == "invalid-argument"),
            "Should report invalid-argument when arg doesn't match constraint: {:?}",
            diags
        );
    }

    /// Ported from TS: "emit diagnostics if using another template with a constraint but template parameter constraint is not compatible"
    #[test]
    fn test_diagnostic_incompatible_constraint_via_template() {
        let checker = check(
            r#"
            model A<T> { b: B<T> }
            model B<T extends string> {}
        "#,
        );
        let diags = checker.diagnostics();
        assert!(
            diags.iter().any(|d| d.code == "invalid-argument"),
            "Should report invalid-argument for incompatible constraint: {:?}",
            diags
        );
    }

    /// Ported from TS: "emit diagnostics if referencing itself"
    #[test]
    fn test_diagnostic_template_default_self_reference() {
        let checker = check(
            r#"
            model A<T = T> { a: T }
            model B {
              foo: A
            };
        "#,
        );
        let diags = checker.diagnostics();
        assert!(
            diags.iter().any(|d| d.code == "invalid-template-default"),
            "Should report invalid-template-default for self-referencing default: {:?}",
            diags
        );
    }

    /// Ported from TS: "emit diagnostics if args reference each other"
    #[test]
    fn test_diagnostic_template_default_mutual_reference() {
        let checker = check(
            r#"
            model A<T = K, K = T> { a: T }
            model B {
              foo: A
            };
        "#,
        );
        let diags = checker.diagnostics();
        assert!(
            diags.iter().any(|d| d.code == "invalid-template-default"),
            "Should report invalid-template-default for mutual reference: {:?}",
            diags
        );
    }

    /// Ported from TS: "emit diagnostics if referencing itself nested"
    #[test]
    fn test_diagnostic_template_default_nested_self_reference() {
        let checker = check(
            r#"
            model A<T = {foo: T}> { a: T }
            model B {
              foo: A
            };
        "#,
        );
        let diags = checker.diagnostics();
        assert!(
            diags.iter().any(|d| d.code == "invalid-template-default"),
            "Should report invalid-template-default for nested self-reference: {:?}",
            diags
        );
    }

    /// Ported from TS: "compile when the constrain is satisfied in the default value"
    #[test]
    fn test_template_constraint_satisfied_in_default() {
        let checker = check(r#"model A<T extends string = "abc"> { a: T }"#);
        let diags = checker.diagnostics();
        assert!(
            !diags
                .iter()
                .any(|d| d.code == "unassignable" || d.code == "invalid-template-args"),
            "Should NOT report error when default satisfies constraint: {:?}",
            diags
        );
    }

    /// Ported from TS: "compile when the constrain is satisfied in template arg"
    #[test]
    fn test_template_constraint_satisfied_in_arg() {
        let checker = check(
            r#"
            model A<T extends string> { a: T }
            model B {
              a: A<"def">
            }
        "#,
        );
        let diags = checker.diagnostics();
        assert!(
            !diags
                .iter()
                .any(|d| d.code == "invalid-argument" || d.code == "unassignable"),
            "Should NOT report error when arg satisfies constraint: {:?}",
            diags
        );
    }

    /// Ported from TS: "use constrain as type when referencing another template"
    #[test]
    fn test_template_constraint_compatible_with_another_template() {
        let checker = check(
            r#"
            model A<T extends string> { b: B<T> }
            model B<T extends string> {}
        "#,
        );
        let diags = checker.diagnostics();
        assert!(
            !diags.iter().any(|d| d.code == "invalid-argument"),
            "Should NOT report error when compatible constraints: {:?}",
            diags
        );
    }

    /// Ported from TS: "use constrain as type when referencing another template parameter"
    #[test]
    fn test_template_constraint_from_another_param() {
        let checker = check("model Foo<A extends string, B extends string = A> { b: B }");
        let diags = checker.diagnostics();
        assert!(
            !diags
                .iter()
                .any(|d| d.code == "invalid-template-default" || d.code == "unassignable"),
            "Should NOT report error when using another param as default: {:?}",
            diags
        );
    }

    /// Ported from TS: "cannot specify a typereference with args as a parameter name"
    #[test]
    fn test_diagnostic_template_arg_name_with_args() {
        let diags = all_diagnostics(
            r#"
            model A<T> { a: T }
            model B {
              foo: A<T<string> = string>
            }
        "#,
        );
        assert!(
            diags
                .iter()
                .any(|d| d.code == "invalid-template-argument-name"),
            "Should report invalid-template-argument-name for T<string> as param name: {:?}",
            diags
        );
    }
}
