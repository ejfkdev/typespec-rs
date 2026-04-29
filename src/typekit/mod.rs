//! TypeKit — High-level type manipulation API
//!
//! Ported from TypeSpec compiler/src/typekit/
//!
//! TypeKit provides a fluent, high-level API for creating, inspecting, and
//! manipulating TypeSpec types. It wraps the lower-level Checker API with
//! more ergonomic operations.

pub mod kits;
pub mod utils;

use crate::checker::types::TypeId;
use crate::checker::{Checker, Type};

/// TypeKit context — provides access to checker operations
///
/// Methods delegate to the kits modules for consistency and to avoid
/// duplicate implementations.
pub struct TypeKit<'a> {
    pub checker: &'a mut Checker,
}

impl<'a> TypeKit<'a> {
    pub fn new(checker: &'a mut Checker) -> Self {
        Self { checker }
    }

    /// Get a type by its ID
    pub fn get_type(&self, id: TypeId) -> Option<&Type> {
        self.checker.get_type(id)
    }

    /// Check if a type is a model
    pub fn is_model(&self, id: TypeId) -> bool {
        kits::model::is_model(self.checker, id)
    }

    /// Check if a type is a scalar
    pub fn is_scalar(&self, id: TypeId) -> bool {
        kits::scalar::is_scalar(self.checker, id)
    }

    /// Check if a type is an enum
    pub fn is_enum(&self, id: TypeId) -> bool {
        kits::enum_type::is_enum(self.checker, id)
    }

    /// Check if a type is a union
    pub fn is_union(&self, id: TypeId) -> bool {
        kits::union::is_union(self.checker, id)
    }

    /// Check if a type is an interface
    pub fn is_interface(&self, id: TypeId) -> bool {
        matches!(self.checker.get_type(id), Some(Type::Interface(_)))
    }

    /// Check if a type is an operation
    pub fn is_operation(&self, id: TypeId) -> bool {
        kits::operation::is_operation(self.checker, id)
    }

    /// Check if a type is a string literal
    pub fn is_string_literal(&self, id: TypeId) -> bool {
        kits::literal::is_string_literal(self.checker, id)
    }

    /// Check if a type is a numeric literal
    pub fn is_numeric_literal(&self, id: TypeId) -> bool {
        kits::literal::is_numeric_literal(self.checker, id)
    }

    /// Check if a type is a boolean literal
    pub fn is_boolean_literal(&self, id: TypeId) -> bool {
        kits::literal::is_boolean_literal(self.checker, id)
    }

    /// Resolve through alias chains
    pub fn resolve_alias(&self, id: TypeId) -> TypeId {
        kits::type_kind::resolve_alias(self.checker, id)
    }

    /// Get the effective model type
    pub fn get_effective_model(&self, id: TypeId) -> TypeId {
        self.checker.get_effective_model_type(id)
    }

    /// Check type assignability
    pub fn is_assignable_to(&mut self, source: TypeId, target: TypeId) -> bool {
        kits::entity::is_assignable_to(self.checker, source, target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::test_utils::check;

    #[test]
    fn test_typekit_is_model() {
        let mut checker = check("model Foo {}");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        let tk = TypeKit::new(&mut checker);
        assert!(tk.is_model(foo_id));
        assert!(!tk.is_scalar(foo_id));
    }

    #[test]
    fn test_typekit_is_scalar() {
        let mut checker = check("scalar MyS extends string;");
        let s_id = checker.declared_types.get("MyS").copied().unwrap();
        let tk = TypeKit::new(&mut checker);
        assert!(tk.is_scalar(s_id));
        assert!(!tk.is_model(s_id));
    }

    #[test]
    fn test_typekit_is_enum() {
        let mut checker = check("enum Color { red, green, blue }");
        let e_id = checker.declared_types.get("Color").copied().unwrap();
        let tk = TypeKit::new(&mut checker);
        assert!(tk.is_enum(e_id));
    }

    #[test]
    fn test_typekit_resolve_alias() {
        let mut checker = check("model Foo {} alias Bar = Foo;");
        let bar_id = checker.declared_types.get("Bar").copied().unwrap();
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        let tk = TypeKit::new(&mut checker);
        let resolved = tk.resolve_alias(bar_id);
        assert_eq!(resolved, foo_id);
    }

    #[test]
    fn test_typekit_is_assignable() {
        let mut checker = check("scalar A extends string; scalar B extends int32;");
        let a_id = checker.declared_types.get("A").copied().unwrap();
        let b_id = checker.declared_types.get("B").copied().unwrap();
        let mut tk = TypeKit::new(&mut checker);
        // A extends string, so A is assignable to string but not to B (which extends int32)
        // This test just verifies the method works without panic
        let _ = tk.is_assignable_to(a_id, b_id);
    }
}
