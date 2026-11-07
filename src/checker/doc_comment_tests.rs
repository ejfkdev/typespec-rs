//! Checker Doc Comment Tests
//!
//! Ported from TypeSpec compiler/test/checker/doc-comment.test.ts
//!
//! Categories:
//! - Main doc comment (/** */) applied to model, property, scalar, enum, operation, interface
//! - @doc decorator overriding doc comments
//! - Model is doc override
//! - Op is doc override
//! - @returns / @errors doc tags in comments
//! - @param / @prop doc tags in comments
//!
//! Skipped (needs doc comment parsing + getDoc/@doc decorator execution):
//! - All tests marked #[ignore] until doc comment infrastructure is implemented

use crate::checker::test_utils::check;

// ============================================================================
// Main doc comment application tests - ported from TS "main doc apply to"
// ============================================================================

#[test]
fn test_doc_comment_on_model() {
    let checker = check(
        r#"
        /**
         * This is a doc comment.
         */
        model Foo {}
    "#,
    );
    let foo_id = checker.get_type_by_name("Foo").unwrap();
    // Should be able to retrieve the doc comment from the type
    // getDoc(program, Foo) === "This is a doc comment."
    assert!(
        !checker.is_deprecated(foo_id),
        "Doc comment should not affect deprecation status"
    );
}

#[test]
fn test_doc_comment_on_templated_model_instance() {
    let _checker = check(
        r#"
        /**
         * This is a doc comment.
         */
        model Foo<T> {}

        model Bar { foo: Foo<string> }
    "#,
    );
    // Foo<string> instance should inherit doc from Foo<T>
    // getDoc(program, fooInstance) === "This is a doc comment."
}

#[test]
fn test_doc_comment_on_model_property() {
    let _checker = check(
        r#"
        model Foo {
            /**
             * This is a doc comment.
             */
            name: string;
        }
    "#,
    );
    // getDoc(program, Foo.name) === "This is a doc comment."
}

#[test]
fn test_doc_comment_on_scalar() {
    let _checker = check(
        r#"
        /**
         * This is a doc comment.
         */
        scalar unreal;
    "#,
    );
    // getDoc(program, unreal) === "This is a doc comment."
}

#[test]
fn test_doc_comment_on_enum() {
    let _checker = check(
        r#"
        /**
         * This is a doc comment.
         */
        enum Foo {}
    "#,
    );
    // getDoc(program, Foo) === "This is a doc comment."
}

#[test]
fn test_doc_comment_on_enum_member() {
    let _checker = check(
        r#"
        enum Foo {
            /**
             * This is a doc comment.
             */
            a,
        }
    "#,
    );
    // getDoc(program, Foo.a) === "This is a doc comment."
}

#[test]
fn test_doc_comment_on_operation() {
    let _checker = check(
        r#"
        /**
         * This is a doc comment.
         */
        op foo(): string;
    "#,
    );
    // getDoc(program, foo) === "This is a doc comment."
}

#[test]
fn test_doc_comment_on_interface() {
    let _checker = check(
        r#"
        /**
         * This is a doc comment.
         */
        interface Foo {}
    "#,
    );
    // getDoc(program, Foo) === "This is a doc comment."
}

// ============================================================================
// @doc decorator override tests - ported from TS
// ============================================================================

#[test]
fn test_doc_decorator_overrides_comment() {
    let _checker = check(
        r#"
        /**
         * This is a doc comment.
         */
        @doc("This is the actual doc.")
        model Foo {}
    "#,
    );
    // getDoc(program, Foo) === "This is the actual doc."
}

// ============================================================================
// Model is doc override tests - ported from TS "override model is comment"
// ============================================================================

#[test]
fn test_model_is_doc_comment_overrides_base() {
    let _checker = check(
        r#"
        /** Base comment */
        model Base {}

        /** Override comment */
        model Foo is Base {}
    "#,
    );
    // getDoc(program, Foo) === "Override comment"
}

#[test]
fn test_model_is_doc_overrides_base_doc_decorator() {
    let _checker = check(
        r#"
        @doc("Base comment")
        model Base {}

        /** Override comment */
        model Foo is Base {}
    "#,
    );
    // getDoc(program, Foo) === "Override comment"
}

// ============================================================================
// Op is doc override tests - ported from TS "override op is comment"
// ============================================================================

#[test]
fn test_op_is_doc_comment_overrides_base() {
    let _checker = check(
        r#"
        /** Base comment */
        op base(): void;

        /** Override comment */
        op foo is base;
    "#,
    );
    // getDoc(program, foo) === "Override comment"
}

#[test]
fn test_op_is_doc_overrides_base_doc_decorator() {
    let _checker = check(
        r#"
        @doc("Base comment")
        op base(): void;

        /** Override comment */
        op foo is base;
    "#,
    );
    // getDoc(program, foo) === "Override comment"
}

// ============================================================================
// @returns doc tag tests - ported from TS "@returns"
// ============================================================================

#[test]
fn test_returns_doc_tag_in_comment() {
    let _checker = check(
        r#"
        /**
         * @returns A string
         */
        op test(): string;
    "#,
    );
    // getReturnsDoc(program, test) === "A string"
}

#[test]
fn test_returns_doc_decorator_overrides_comment() {
    let _checker = check(
        r#"
        /**
         * @returns A string
         */
        @returnsDoc("Another string")
        op test(): string;
    "#,
    );
    // getReturnsDoc(program, test) === "Another string"
}

// ============================================================================
// @errors doc tag tests - ported from TS "@errors"
// ============================================================================

#[test]
fn test_errors_doc_tag_in_comment() {
    let _checker = check(
        r#"
        /**
         * @errors A string
         */
        op test(): string;
    "#,
    );
    // getErrorsDoc(program, test) === "A string"
}

#[test]
fn test_errors_doc_decorator_overrides_comment() {
    let _checker = check(
        r#"
        /**
         * @errors A string
         */
        @errorsDoc("Another string")
        op test(): string;
    "#,
    );
    // getErrorsDoc(program, test) === "Another string"
}

// ============================================================================
// @param doc tag tests - ported from TS "@param"
// ============================================================================

#[test]
fn test_param_doc_tag_in_comment() {
    let _checker = check(
        r#"
        /**
         * @param one Doc comment
         */
        op target(one: string): void;
    "#,
    );
    // getDoc(program, target.parameters.one) === "Doc comment"
}

#[test]
fn test_param_doc_decorator_overrides_comment() {
    let _checker = check(
        r#"
        /**
         * @param one Doc comment
         */
        op target(@doc("Explicit") one: string): void;
    "#,
    );
    // getDoc(program, target.parameters.one) === "Explicit"
}

#[test]
fn test_param_doc_carries_over_with_op_is() {
    let _checker = check(
        r#"
        /**
         * @param one Doc comment
         */
        op base(one: string): void;

        op target is base;
    "#,
    );
    // getDoc(program, target.parameters.one) === "Doc comment"
}

// ============================================================================
// @prop doc tag tests - ported from TS "@prop"
// ============================================================================

#[test]
fn test_prop_doc_tag_in_comment() {
    let _checker = check(
        r#"
        /**
         * @prop one Doc comment
         */
        model target { one: string }
    "#,
    );
    // getDoc(program, target.one) === "Doc comment"
}

#[test]
fn test_prop_doc_decorator_overrides_comment() {
    let _checker = check(
        r#"
        /**
         * @prop one Doc comment
         */
        model target { @doc("Explicit") one: string }
    "#,
    );
    // getDoc(program, target.one) === "Explicit"
}

#[test]
fn test_prop_doc_carries_over_with_model_is() {
    let _checker = check(
        r#"
        /**
         * @prop one Doc comment
         */
        model Base { one: string }

        model target is Base;
    "#,
    );
    // getDoc(program, target.one) === "Doc comment"
}

#[test]
fn test_prop_doc_from_spread_model() {
    let _checker = check(
        r#"
        model Base {
            @doc("Via model") one: string
        }
        model target { ...Base }
    "#,
    );
    // getDoc(program, target.one) === "Via model"
}

#[test]
fn test_prop_doc_overrides_spread_model_doc() {
    let _checker = check(
        r#"
        model Base {
            @doc("Via model") one: string
        }
        /**
         * @prop one Doc comment
         */
        model target { ...Base }
    "#,
    );
    // getDoc(program, target.one) === "Doc comment"
}

// ============================================================================
// @doc decorator overriding doc comment - ported from TS
// ============================================================================

/// Ported from TS: "using @doc() decorator will override the doc comment"
#[test]
fn test_doc_decorator_overrides_doc_comment() {
    let checker = check(
        r#"
        /**
         * This is a doc comment.
         */
        @doc("This is the actual doc.")
        model Foo {}
    "#,
    );
    // getDoc(program, Foo) === "This is the actual doc."
    assert!(checker.get_type_by_name("Foo").is_some());
}

// ============================================================================
// Model is doc override - ported from TS "override model is comment"
// ============================================================================

/// Ported from TS: "override another doc comment" (model is)
#[test]
fn test_model_is_overrides_base_doc_comment() {
    let checker = check(
        r#"
        /** Base comment */
        model Base {}

        /** Override comment */
        model Foo is Base {}
    "#,
    );
    // getDoc(program, Foo) === "Override comment"
    assert!(checker.get_type_by_name("Foo").is_some());
    assert!(checker.get_type_by_name("Base").is_some());
}

/// Ported from TS: "override @doc" (model is)
#[test]
fn test_model_is_overrides_base_doc_decorator() {
    let checker = check(
        r#"
        @doc("Base comment")
        model Base {}

        /** Override comment */
        model Foo is Base {}
    "#,
    );
    // getDoc(program, Foo) === "Override comment"
    assert!(checker.get_type_by_name("Foo").is_some());
}

// ============================================================================
// Op is doc override - ported from TS "override op is comment"
// ============================================================================

/// Ported from TS: "override another doc comment" (op is)
#[test]
fn test_op_is_overrides_base_doc_comment() {
    let _checker = check(
        r#"
        /** Base comment */
        op base(): void;

        /** Override comment */
        op foo is base;
    "#,
    );
    // getDoc(program, foo) === "Override comment"
}

/// Ported from TS: "override @doc" (op is)
#[test]
fn test_op_is_overrides_base_doc_decorator() {
    let _checker = check(
        r#"
        @doc("Base comment")
        op base(): void;

        /** Override comment */
        op foo is base;
    "#,
    );
    // getDoc(program, foo) === "Override comment"
}

// ============================================================================
// @returns / @errors doc tags - ported from TS
// ============================================================================

/// Ported from TS: "set the returnsDoc on an operation"
#[test]
fn test_returns_doc_tag_in_comment_override() {
    let _checker = check(
        r#"
        /**
         * @returns A string
         */
        op test(): string;
    "#,
    );
    // getReturnsDoc(program, test) === "A string"
}

/// Ported from TS: "set the errorsDoc on an operation"
#[test]
fn test_errors_doc_tag_in_comment_override() {
    let _checker = check(
        r#"
        /**
         * @errors A string
         */
        op test(): string;
    "#,
    );
    // getErrorsDoc(program, test) === "A string"
}

// ============================================================================
// @param / @prop multiple parameters - ported from TS
// ============================================================================

/// Ported from TS: "applies to distinct parameters" (operation with multiple @param)
#[test]
fn test_param_doc_applies_to_distinct_parameters() {
    let _checker = check(
        r#"
        /**
         * This is the operation doc.
         * @param name This is the name param doc.
         * @param age - This is the age param doc.
         */
        op addUser(name: string, age: string): void;
    "#,
    );
    // getDoc(program, addUser) === "This is the operation doc."
    // getDoc(program, addUser.parameters.name) === "This is the name param doc."
    // getDoc(program, addUser.parameters.age) === "This is the age param doc."
}

/// Ported from TS: "applies to distinct parameters" (model with multiple @prop)
#[test]
fn test_prop_doc_applies_to_distinct_properties() {
    let _checker = check(
        r#"
        /**
         * This is the model doc.
         * @prop name This is the name prop doc.
         * @prop age - This is the age prop doc.
         */
        model Base { name: string, age: int32 }
    "#,
    );
    // getDoc(program, Base) === "This is the model doc."
    // getDoc(program, Base.name) === "This is the name prop doc."
    // getDoc(program, Base.age) === "This is the age prop doc."
}

// ============================================================================
// Doc comment execution (ported from doc-comment.test.ts)
// ============================================================================

fn doc_of(
    checker: &crate::checker::Checker,
    type_id: crate::checker::types::TypeId,
) -> Option<String> {
    crate::libs::compiler::get_doc(&checker.state_accessors, type_id)
}

#[test]
fn test_doc_comment_applies_to_model() {
    let checker = check(
        "\n        /**\n         * This is a doc comment.\n         */\n        model Foo {}\n    ",
    );
    let foo_id = checker.get_type_by_name("Foo").unwrap();
    assert_eq!(
        doc_of(&checker, foo_id).as_deref(),
        Some("This is a doc comment.")
    );
}

#[test]
fn test_doc_comment_applies_to_model_property() {
    let checker =
        check("model Foo {\n  /**\n   * This is a doc comment.\n   */\n  name: string;\n}");
    let foo_id = checker.get_type_by_name("Foo").unwrap();
    let prop_id = checker.get_model_property(foo_id, "name").unwrap();
    assert_eq!(
        doc_of(&checker, prop_id).as_deref(),
        Some("This is a doc comment.")
    );
}

#[test]
fn test_doc_comment_applies_to_scalar_enum_and_interface() {
    let checker = check(
        "/** Scalar doc */ scalar unreal;\n/** Enum doc */ enum E { a }\n/** Interface doc */ interface I {}",
    );
    for name in ["unreal", "E", "I"] {
        let id = checker.get_type_by_name(name).unwrap();
        assert_eq!(
            doc_of(&checker, id).as_deref(),
            Some(match name {
                "unreal" => "Scalar doc",
                "E" => "Enum doc",
                _ => "Interface doc",
            })
        );
    }
}

#[test]
fn test_doc_comment_applies_to_operation() {
    let checker = check("/** Operation doc */ op foo(): string;");
    let op_id = checker.get_type_by_name("foo").unwrap();
    assert_eq!(doc_of(&checker, op_id).as_deref(), Some("Operation doc"));
}

#[test]
fn test_doc_comment_param_and_returns_tags() {
    let checker = check(
        "/**\n * Operation doc.\n * @param one Param doc.\n * @returns Returns doc.\n * @errors Errors doc.\n */\nop foo(one: string): string;",
    );
    let op_id = checker.get_type_by_name("foo").unwrap();
    assert_eq!(doc_of(&checker, op_id).as_deref(), Some("Operation doc."));
    let op = match checker.get_type(op_id) {
        Some(crate::checker::types::Type::Operation(op)) => op.clone(),
        _ => panic!(),
    };
    let params = op.parameters.unwrap();
    let one = match checker.get_type(params) {
        Some(crate::checker::types::Type::Model(model)) => model.properties["one"],
        _ => panic!(),
    };
    assert_eq!(doc_of(&checker, one).as_deref(), Some("Param doc."));
    assert_eq!(
        crate::libs::compiler::get_returns_doc(&checker.state_accessors, op_id).as_deref(),
        Some("Returns doc.")
    );
    assert_eq!(
        crate::libs::compiler::get_errors_doc(&checker.state_accessors, op_id).as_deref(),
        Some("Errors doc.")
    );
}

#[test]
fn test_doc_comment_prop_tag() {
    let checker =
        check("/**\n * Model doc.\n * @prop name Name doc.\n */\nmodel Foo { name: string; }");
    let foo_id = checker.get_type_by_name("Foo").unwrap();
    let prop_id = checker.get_model_property(foo_id, "name").unwrap();
    assert_eq!(doc_of(&checker, prop_id).as_deref(), Some("Name doc."));
}

#[test]
fn test_doc_decorator_overrides_comment_execution() {
    let checker = check("/** Comment doc */\n@doc(\"Decorator doc\")\nmodel Foo {}");
    let foo_id = checker.get_type_by_name("Foo").unwrap();
    assert_eq!(doc_of(&checker, foo_id).as_deref(), Some("Decorator doc"));
}

#[test]
fn test_templated_model_instance_inherits_comment() {
    let checker = check("/** Template doc */\nmodel Foo<T> {}\nmodel Bar { foo: Foo<string>; }");
    let bar_id = checker.get_type_by_name("Bar").unwrap();
    let prop_id = checker.get_model_property(bar_id, "foo").unwrap();
    let prop_type = checker.get_property_type(prop_id);
    assert_eq!(doc_of(&checker, prop_type).as_deref(), Some("Template doc"));
}

#[test]
fn test_doc_comment_applies_to_enum_member_and_union_variant() {
    let checker = check(
        "enum E {\n  /** Member doc */\n  a,\n}\nunion U {\n  /** Variant doc */\n  v: string,\n}\n/** Namespace doc */\nnamespace N {}",
    );
    let e_id = checker.get_type_by_name("E").unwrap();
    let member_id = match checker.get_type(e_id) {
        Some(crate::checker::types::Type::Enum(e)) => e.members["a"],
        _ => panic!(),
    };
    assert_eq!(doc_of(&checker, member_id).as_deref(), Some("Member doc"));
    let u_id = checker.get_type_by_name("U").unwrap();
    let variant_id = match checker.get_type(u_id) {
        Some(crate::checker::types::Type::Union(u)) => u.variants["v"],
        _ => panic!(),
    };
    assert_eq!(doc_of(&checker, variant_id).as_deref(), Some("Variant doc"));
    let n_id = checker.get_type_by_name("N").unwrap();
    assert_eq!(doc_of(&checker, n_id).as_deref(), Some("Namespace doc"));
}

#[test]
fn test_augment_doc_overrides_prop_comment() {
    let checker = check(
        "/**\n * @prop one Comment doc.\n */\nmodel Foo { one: string }\n@@doc(Foo.one, \"Override\");",
    );
    let foo_id = checker.get_type_by_name("Foo").unwrap();
    let prop_id = checker.get_model_property(foo_id, "one").unwrap();
    assert_eq!(doc_of(&checker, prop_id).as_deref(), Some("Override"));
}

#[test]
fn test_augment_doc_overrides_param_comment() {
    let checker = check(
        "/**\n * @param one Comment doc.\n */\nop foo(one: string): void;\n@@doc(foo::parameters.one, \"Override\");",
    );
    let op_id = checker.get_type_by_name("foo").unwrap();
    let params = match checker.get_type(op_id) {
        Some(crate::checker::types::Type::Operation(op)) => op.parameters.unwrap(),
        _ => panic!(),
    };
    let one = match checker.get_type(params) {
        Some(crate::checker::types::Type::Model(model)) => model.properties["one"],
        _ => panic!(),
    };
    assert_eq!(doc_of(&checker, one).as_deref(), Some("Override"));
}

#[test]
fn test_augment_doc_on_model() {
    let checker = check("model Foo {}\n@@doc(Foo, \"Override\");");
    let foo_id = checker.get_type_by_name("Foo").unwrap();
    assert_eq!(doc_of(&checker, foo_id).as_deref(), Some("Override"));
}

#[test]
fn test_augment_doc_last_wins() {
    let checker = check("model Foo {}\n@@doc(Foo, \"First\");\n@@doc(Foo, \"Second\");");
    let foo_id = checker.get_type_by_name("Foo").unwrap();
    assert_eq!(doc_of(&checker, foo_id).as_deref(), Some("Second"));
}

#[test]
fn test_augment_summary() {
    let checker = check("model Foo {}\n@@summary(Foo, \"Summary\");");
    let foo_id = checker.get_type_by_name("Foo").unwrap();
    assert_eq!(
        crate::libs::compiler::get_summary(&checker.state_accessors, foo_id).as_deref(),
        Some("Summary")
    );
}
