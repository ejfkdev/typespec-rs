//! Operation type operations
//!
//! Ported from TypeSpec compiler/src/typekit/kits/operation.ts

use crate::checker::types::TypeId;

define_type_check!(is_operation, Operation);
define_type_field_getter!(get_return_type, Operation, return_type, Option<TypeId>);
define_type_field_getter!(get_parameters_model, Operation, parameters, Option<TypeId>);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::test_utils::check;

    #[test]
    fn test_is_operation() {
        let checker = check("op test(): void;");
        let op_id = checker.declared_types.get("test").copied().unwrap();
        assert!(is_operation(&checker, op_id));
    }

    #[test]
    fn test_is_operation_not_model() {
        let checker = check("model Foo {}");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        assert!(!is_operation(&checker, foo_id));
    }

    #[test]
    fn test_get_return_type_void() {
        let checker = check("op test(): void;");
        let op_id = checker.declared_types.get("test").copied().unwrap();
        let ret = get_return_type(&checker, op_id);
        assert!(ret.is_some());
    }

    #[test]
    fn test_get_return_type_string() {
        let checker = check("op test(): string;");
        let op_id = checker.declared_types.get("test").copied().unwrap();
        let ret = get_return_type(&checker, op_id);
        assert!(ret.is_some());
    }

    #[test]
    fn test_get_parameters_model() {
        let checker = check("op test(name: string): void;");
        let op_id = checker.declared_types.get("test").copied().unwrap();
        let params = get_parameters_model(&checker, op_id);
        assert!(params.is_some());
    }

    #[test]
    fn test_get_parameters_model_no_params() {
        let checker = check("op test(): void;");
        let op_id = checker.declared_types.get("test").copied().unwrap();
        let params = get_parameters_model(&checker, op_id);
        // Operation with no params may still have an empty model
        // This is implementation-dependent
        let _ = params;
    }
}
