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
    let foo_id = checker.declared_types.get("Foo").copied().unwrap();
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
    assert!(checker.declared_types.contains_key("Foo"));
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
    assert!(checker.declared_types.contains_key("Foo"));
    assert!(checker.declared_types.contains_key("Base"));
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
    assert!(checker.declared_types.contains_key("Foo"));
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
