//! Decorator utility functions for TypeSpec-Rust
//! Ported from TypeSpec compiler/src/core/decorator-utils.ts
//!
//! Provides helper functions for decorator validation and type conversion.

use crate::checker::Diagnostic;
use crate::checker::types::{Type, TypeId, TypeStore};

/// Type kind string for decorator target validation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeKind {
    Model,
    ModelProperty,
    Scalar,
    Interface,
    Operation,
    Enum,
    EnumMember,
    Union,
    UnionVariant,
    Namespace,
    Tuple,
    String,
    Number,
    Boolean,
    Intrinsic,
    TemplateParameter,
    Any,
}

impl TypeKind {
    /// Get the display name for this type kind
    pub fn as_str(&self) -> &'static str {
        match self {
            TypeKind::Model => "Model",
            TypeKind::ModelProperty => "ModelProperty",
            TypeKind::Scalar => "Scalar",
            TypeKind::Interface => "Interface",
            TypeKind::Operation => "Operation",
            TypeKind::Enum => "Enum",
            TypeKind::EnumMember => "EnumMember",
            TypeKind::Union => "Union",
            TypeKind::UnionVariant => "UnionVariant",
            TypeKind::Namespace => "Namespace",
            TypeKind::Tuple => "Tuple",
            TypeKind::String => "String",
            TypeKind::Number => "Number",
            TypeKind::Boolean => "Boolean",
            TypeKind::Intrinsic => "Intrinsic",
            TypeKind::TemplateParameter => "TemplateParameter",
            TypeKind::Any => "Any",
        }
    }
}

impl std::fmt::Display for TypeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Get the type kind of a Type
pub fn get_type_kind(t: &Type) -> TypeKind {
    match t {
        Type::Model(_) => TypeKind::Model,
        Type::ModelProperty(_) => TypeKind::ModelProperty,
        Type::Scalar(_) => TypeKind::Scalar,
        Type::Interface(_) => TypeKind::Interface,
        Type::Operation(_) => TypeKind::Operation,
        Type::Enum(_) => TypeKind::Enum,
        Type::EnumMember(_) => TypeKind::EnumMember,
        Type::Union(_) => TypeKind::Union,
        Type::UnionVariant(_) => TypeKind::UnionVariant,
        Type::Namespace(_) => TypeKind::Namespace,
        Type::Tuple(_) => TypeKind::Tuple,
        Type::String(_) => TypeKind::String,
        Type::Number(_) => TypeKind::Number,
        Type::Boolean(_) => TypeKind::Boolean,
        Type::Intrinsic(_) => TypeKind::Intrinsic,
        Type::TemplateParameter(_) => TypeKind::TemplateParameter,
        Type::ScalarConstructor(_) => TypeKind::Scalar,
        Type::Decorator(_) => TypeKind::Intrinsic,
        Type::FunctionType(_) => TypeKind::Intrinsic,
        Type::FunctionParameter(_) => TypeKind::Intrinsic,
        Type::StringTemplate(_) => TypeKind::String,
        Type::StringTemplateSpan(_) => TypeKind::String,
    }
}

/// Validate that a decorator's target type matches the expected type(s).
/// Ported from TS validateDecoratorTarget()
pub fn validate_decorator_target(
    target_type: &Type,
    decorator_name: &str,
    expected_kinds: &[TypeKind],
) -> Result<(), Box<Diagnostic>> {
    let target_kind = get_type_kind(target_type);

    if expected_kinds.contains(&TypeKind::Any) || expected_kinds.contains(&target_kind) {
        return Ok(());
    }

    Err(Box::new(Diagnostic::error(
        "decorator-wrong-target",
        &format!(
            "Decorator '{}' cannot be applied to '{}'. Expected: {}",
            decorator_name,
            target_kind,
            expected_kinds
                .iter()
                .map(|k| k.to_string())
                .collect::<Vec<_>>()
                .join(" | ")
        ),
    )))
}

/// Validate the number of parameters passed to a decorator.
/// Ported from TS validateDecoratorParamCount()
pub fn validate_decorator_param_count(
    decorator_name: &str,
    min: usize,
    max: Option<usize>,
    param_count: usize,
) -> Result<(), Box<Diagnostic>> {
    if param_count < min || max.is_some_and(|m| param_count > m) {
        let expected = match max {
            Some(m) if m == min => min.to_string(),
            Some(m) => format!("{}-{}", min, m),
            None => format!("{}-infinity", min),
        };
        Err(Box::new(Diagnostic::error(
            "invalid-argument-count",
            &format!(
                "Decorator '{}' expects {} argument(s) but got {}",
                decorator_name, expected, param_count
            ),
        )))
    } else {
        Ok(())
    }
}

/// Check if a scalar type is an intrinsic type of the given kind.
/// Ported from TS isIntrinsicType()
pub fn is_intrinsic_type(store: &TypeStore, scalar_id: TypeId, kind: &str) -> bool {
    super::type_utils::scalar_extends(store, scalar_id, kind)
}

/// Check if a TypeSpecValue is of any of the expected type kinds.
/// Ported from TS isTypeSpecValueTypeOf()
pub fn is_type_spec_value_type_of(value: &TypeSpecValue, expected_kinds: &[TypeKind]) -> bool {
    let kind = match value {
        TypeSpecValue::Null => return expected_kinds.contains(&TypeKind::Intrinsic),
        TypeSpecValue::Bool(_) => TypeKind::Boolean,
        TypeSpecValue::Number(_) => TypeKind::Number,
        TypeSpecValue::String(_) => TypeKind::String,
        TypeSpecValue::Array(_) => TypeKind::Tuple,
        TypeSpecValue::Object(_) => TypeKind::Model,
    };
    expected_kinds.contains(&TypeKind::Any) || expected_kinds.contains(&kind)
}

/// Return the type of a ModelProperty, or the Scalar itself.
/// Ported from TS getPropertyType()
pub fn get_property_type(store: &TypeStore, target_id: TypeId) -> TypeId {
    match store.get(target_id) {
        Some(Type::ModelProperty(prop)) => prop.r#type,
        _ => target_id,
    }
}

/// Validate that a decorator is not applied more than once on the same node.
/// Ported from TS validateDecoratorUniqueOnNode()
///
/// Returns Ok(()) if unique, Err with diagnostic if duplicate found.
pub fn validate_decorator_unique_on_node(
    store: &TypeStore,
    type_id: TypeId,
    decorator_name: &str,
) -> Result<(), Box<Diagnostic>> {
    let decorators = match store.get(type_id).and_then(|t| t.decorators()) {
        Some(d) => d,
        None => return Ok(()),
    };

    let count = decorators
        .iter()
        .filter(|d| {
            if let Some(def_id) = d.definition
                && let Some(Type::Decorator(dec)) = store.get(def_id)
            {
                return dec.name == decorator_name;
            }
            false
        })
        .count();

    if count > 1 {
        Err(Box::new(Diagnostic::error(
            "duplicate-decorator",
            &format!(
                "Decorator '@{}' cannot be used twice on the same declaration",
                decorator_name
            ),
        )))
    } else {
        Ok(())
    }
}

/// Validate that a decorator is not on a type or any of its base types.
/// Useful to check for decorator usage that conflicts with another decorator.
/// Ported from TS validateDecoratorNotOnType()
pub fn validate_decorator_not_on_type(
    store: &TypeStore,
    type_id: TypeId,
    bad_decorator_name: &str,
    given_decorator_name: &str,
) -> Result<(), Box<Diagnostic>> {
    let mut current_id = Some(type_id);
    while let Some(id) = current_id {
        let decorators = match store.get(id) {
            Some(Type::Model(m)) => {
                current_id = m.base_model;
                &m.decorators
            }
            Some(Type::Scalar(s)) => {
                current_id = s.base_scalar;
                &s.decorators
            }
            _ => break,
        };

        for dec_app in decorators {
            if let Some(def_id) = dec_app.definition
                && let Some(Type::Decorator(dec)) = store.get(def_id)
                && dec.name == bad_decorator_name
            {
                return Err(Box::new(Diagnostic::error(
                    "decorator-conflict",
                    &format!(
                        "Decorator '@{}' cannot be used with '@{}'",
                        bad_decorator_name, given_decorator_name
                    ),
                )));
            }
        }
    }
    Ok(())
}

/// A simple JSON-like value type for decorator argument conversion
#[derive(Debug, Clone, PartialEq)]
pub enum TypeSpecValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<TypeSpecValue>),
    Object(Vec<(String, TypeSpecValue)>),
}

/// Convert a TypeSpec type to a JSON-like value.
/// Ported from TS typespecTypeToJson()
pub fn typespec_type_to_value(store: &TypeStore, type_id: TypeId) -> Result<TypeSpecValue, String> {
    match store.get(type_id) {
        Some(Type::String(s)) => Ok(TypeSpecValue::String(s.value.clone())),
        Some(Type::Number(n)) => Ok(TypeSpecValue::Number(n.value)),
        Some(Type::Boolean(b)) => Ok(TypeSpecValue::Bool(b.value)),
        Some(Type::EnumMember(m)) => {
            // Ported from TS: typespecType.value ?? typespecType.name
            if let Some(value_id) = m.value {
                match store.get(value_id) {
                    Some(Type::String(s)) => Ok(TypeSpecValue::String(s.value.clone())),
                    Some(Type::Number(n)) => Ok(TypeSpecValue::Number(n.value)),
                    _ => Ok(TypeSpecValue::String(m.name.clone())),
                }
            } else {
                Ok(TypeSpecValue::String(m.name.clone()))
            }
        }
        Some(Type::Tuple(t)) => {
            let mut result = Vec::new();
            for &val_id in &t.values {
                result.push(typespec_type_to_value(store, val_id)?);
            }
            Ok(TypeSpecValue::Array(result))
        }
        Some(Type::Model(m)) => {
            let mut result = Vec::new();
            for name in &m.property_names {
                if let Some(&prop_id) = m.properties.get(name)
                    && let Some(Type::ModelProperty(prop)) = store.get(prop_id)
                {
                    let value = typespec_type_to_value(store, prop.r#type)?;
                    result.push((name.clone(), value));
                }
            }
            Ok(TypeSpecValue::Object(result))
        }
        Some(Type::StringTemplate(st)) => {
            // Ported from TS: if stringValue exists, return it; otherwise fall through to error
            if let Some(ref sv) = st.string_value {
                Ok(TypeSpecValue::String(sv.clone()))
            } else {
                Err("Non-literal string template cannot be converted to value".to_string())
            }
        }
        Some(Type::Union(_)) => Err("Union types cannot be converted to value".to_string()),
        Some(Type::Enum(_)) => Err("Enum types cannot be directly converted to value".to_string()),
        _ => Err("Cannot convert type to value".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::types::*;
    use std::collections::HashMap;

    #[test]
    fn test_get_type_kind() {
        let mut store = TypeStore::new();
        let model = store.add(Type::Model({
            let mut m = ModelType::new(store.next_type_id(), "Foo".to_string(), None, None);
            m.is_finished = true;
            m
        }));
        assert_eq!(get_type_kind(store.get(model).unwrap()), TypeKind::Model);
    }

    #[test]
    fn test_validate_decorator_target_success() {
        let mut store = TypeStore::new();
        let model = store.add(Type::Model({
            let mut m = ModelType::new(store.next_type_id(), "Foo".to_string(), None, None);
            m.is_finished = true;
            m
        }));
        let result =
            validate_decorator_target(store.get(model).unwrap(), "test", &[TypeKind::Model]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_decorator_target_failure() {
        let mut store = TypeStore::new();
        let model = store.add(Type::Model({
            let mut m = ModelType::new(store.next_type_id(), "Foo".to_string(), None, None);
            m.is_finished = true;
            m
        }));
        let result =
            validate_decorator_target(store.get(model).unwrap(), "test", &[TypeKind::Scalar]);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_decorator_target_any() {
        let mut store = TypeStore::new();
        let model = store.add(Type::Model({
            let mut m = ModelType::new(store.next_type_id(), "Foo".to_string(), None, None);
            m.is_finished = true;
            m
        }));
        let result = validate_decorator_target(store.get(model).unwrap(), "test", &[TypeKind::Any]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_decorator_param_count() {
        assert!(validate_decorator_param_count("test", 1, Some(3), 2).is_ok());
        assert!(validate_decorator_param_count("test", 1, Some(3), 0).is_err());
        assert!(validate_decorator_param_count("test", 1, Some(3), 4).is_err());
        assert!(validate_decorator_param_count("test", 2, None, 5).is_ok());
    }

    #[test]
    fn test_typespec_type_to_value_string() {
        let mut store = TypeStore::new();
        let s = store.add(Type::String(StringType {
            id: store.next_type_id(),
            value: "hello".to_string(),
            node: None,
            is_finished: true,
        }));
        let result = typespec_type_to_value(&store, s).unwrap();
        assert_eq!(result, TypeSpecValue::String("hello".to_string()));
    }

    #[test]
    fn test_typespec_type_to_value_number() {
        let mut store = TypeStore::new();
        let n = store.add(Type::Number(NumericType {
            id: store.next_type_id(),
            value: 42.0,
            value_as_string: "42".to_string(),
            node: None,
            is_finished: true,
        }));
        let result = typespec_type_to_value(&store, n).unwrap();
        assert_eq!(result, TypeSpecValue::Number(42.0));
    }

    #[test]
    fn test_typespec_type_to_value_bool() {
        let mut store = TypeStore::new();
        let b = store.add(Type::Boolean(BooleanType {
            id: store.next_type_id(),
            value: true,
            node: None,
            is_finished: true,
        }));
        let result = typespec_type_to_value(&store, b).unwrap();
        assert_eq!(result, TypeSpecValue::Bool(true));
    }

    // ============================================================================
    // isTypeSpecValueTypeOf tests
    // ============================================================================

    #[test]
    fn test_is_type_spec_value_type_of_string() {
        let value = TypeSpecValue::String("hello".to_string());
        assert!(is_type_spec_value_type_of(&value, &[TypeKind::String]));
        assert!(!is_type_spec_value_type_of(&value, &[TypeKind::Number]));
    }

    #[test]
    fn test_is_type_spec_value_type_of_number() {
        let value = TypeSpecValue::Number(42.0);
        assert!(is_type_spec_value_type_of(&value, &[TypeKind::Number]));
        assert!(!is_type_spec_value_type_of(&value, &[TypeKind::String]));
    }

    #[test]
    fn test_is_type_spec_value_type_of_bool() {
        let value = TypeSpecValue::Bool(true);
        assert!(is_type_spec_value_type_of(&value, &[TypeKind::Boolean]));
        assert!(!is_type_spec_value_type_of(&value, &[TypeKind::String]));
    }

    #[test]
    fn test_is_type_spec_value_type_of_any() {
        let value = TypeSpecValue::String("hello".to_string());
        assert!(is_type_spec_value_type_of(&value, &[TypeKind::Any]));
    }

    #[test]
    fn test_is_type_spec_value_type_of_multiple() {
        let value = TypeSpecValue::String("hello".to_string());
        assert!(is_type_spec_value_type_of(
            &value,
            &[TypeKind::Number, TypeKind::String]
        ));
        assert!(!is_type_spec_value_type_of(
            &value,
            &[TypeKind::Number, TypeKind::Boolean]
        ));
    }

    #[test]
    fn test_is_type_spec_value_type_of_null() {
        let value = TypeSpecValue::Null;
        assert!(is_type_spec_value_type_of(&value, &[TypeKind::Intrinsic]));
        assert!(!is_type_spec_value_type_of(&value, &[TypeKind::String]));
    }

    // ============================================================================
    // getPropertyType tests
    // ============================================================================

    #[test]
    fn test_get_property_type_model_property() {
        let mut store = TypeStore::new();
        let string_type = store.add(Type::Scalar({
            let mut s =
                ScalarType::new(store.next_type_id(), "string".to_string(), None, None, None);
            s.is_finished = true;
            s
        }));
        let prop = store.add(Type::ModelProperty(ModelPropertyType {
            id: store.next_type_id(),
            name: "x".to_string(),
            node: None,
            r#type: string_type,
            optional: false,
            default_value: None,
            model: None,
            source_property: None,
            decorators: vec![],
            is_finished: true,
        }));
        assert_eq!(get_property_type(&store, prop), string_type);
    }

    #[test]
    fn test_get_property_type_scalar_returns_self() {
        let mut store = TypeStore::new();
        let scalar = store.add(Type::Scalar({
            let mut s = ScalarType::new(
                store.next_type_id(),
                "MyScalar".to_string(),
                None,
                None,
                None,
            );
            s.is_finished = true;
            s
        }));
        assert_eq!(get_property_type(&store, scalar), scalar);
    }

    // ============================================================================
    // validateDecoratorUniqueOnNode tests
    // ============================================================================

    #[test]
    fn test_validate_decorator_unique_no_decorators() {
        let mut store = TypeStore::new();
        let model = store.add(Type::Model({
            let mut m = ModelType::new(store.next_type_id(), "Foo".to_string(), None, None);
            m.is_finished = true;
            m
        }));
        assert!(validate_decorator_unique_on_node(&store, model, "doc").is_ok());
    }

    #[test]
    fn test_validate_decorator_unique_single_decorator() {
        let mut store = TypeStore::new();
        let dec_id = store.add(Type::Decorator(DecoratorType {
            id: store.next_type_id(),
            name: "doc".to_string(),
            node: None,
            namespace: None,
            target: None,
            target_type: "unknown".to_string(),
            parameters: vec![],
            is_finished: true,
        }));
        let model = store.add(Type::Model({
            let mut m = ModelType::new(store.next_type_id(), "Foo".to_string(), None, None);
            m.decorators = vec![DecoratorApplication {
                definition: Some(dec_id),
                decorator: 0,
                args: vec![],
                node: None,
            }];
            m.is_finished = true;
            m
        }));
        assert!(validate_decorator_unique_on_node(&store, model, "doc").is_ok());
    }

    #[test]
    fn test_validate_decorator_unique_duplicate() {
        let mut store = TypeStore::new();
        let dec_id = store.add(Type::Decorator(DecoratorType {
            id: store.next_type_id(),
            name: "doc".to_string(),
            node: None,
            namespace: None,
            target: None,
            target_type: "unknown".to_string(),
            parameters: vec![],
            is_finished: true,
        }));
        let model = store.add(Type::Model({
            let mut m = ModelType::new(store.next_type_id(), "Foo".to_string(), None, None);
            m.decorators = vec![
                DecoratorApplication {
                    definition: Some(dec_id),
                    decorator: 0,
                    args: vec![],
                    node: None,
                },
                DecoratorApplication {
                    definition: Some(dec_id),
                    decorator: 1,
                    args: vec![],
                    node: None,
                },
            ];
            m.is_finished = true;
            m
        }));
        let result = validate_decorator_unique_on_node(&store, model, "doc");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, "duplicate-decorator");
    }

    // ============================================================================
    // validateDecoratorNotOnType tests
    // ============================================================================

    #[test]
    fn test_validate_decorator_not_on_type_ok() {
        let mut store = TypeStore::new();
        let _dec_id = store.add(Type::Decorator(DecoratorType {
            id: store.next_type_id(),
            name: "sensitive".to_string(),
            node: None,
            namespace: None,
            target: None,
            target_type: "unknown".to_string(),
            parameters: vec![],
            is_finished: true,
        }));
        // Model without the bad decorator
        let model = store.add(Type::Model({
            let mut m = ModelType::new(store.next_type_id(), "Foo".to_string(), None, None);
            m.is_finished = true;
            m
        }));
        assert!(validate_decorator_not_on_type(&store, model, "sensitive", "secret").is_ok());
    }

    #[test]
    fn test_validate_decorator_not_on_type_found() {
        let mut store = TypeStore::new();
        let dec_id = store.add(Type::Decorator(DecoratorType {
            id: store.next_type_id(),
            name: "sensitive".to_string(),
            node: None,
            namespace: None,
            target: None,
            target_type: "unknown".to_string(),
            parameters: vec![],
            is_finished: true,
        }));
        let model = store.add(Type::Model({
            let mut m = ModelType::new(store.next_type_id(), "Foo".to_string(), None, None);
            m.decorators = vec![DecoratorApplication {
                definition: Some(dec_id),
                decorator: 0,
                args: vec![],
                node: None,
            }];
            m.is_finished = true;
            m
        }));
        let result = validate_decorator_not_on_type(&store, model, "sensitive", "secret");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, "decorator-conflict");
    }

    #[test]
    fn test_validate_decorator_not_on_type_in_base_model() {
        let mut store = TypeStore::new();
        let dec_id = store.add(Type::Decorator(DecoratorType {
            id: store.next_type_id(),
            name: "sensitive".to_string(),
            node: None,
            namespace: None,
            target: None,
            target_type: "unknown".to_string(),
            parameters: vec![],
            is_finished: true,
        }));
        // Base model has the decorator
        let base = store.add(Type::Model({
            let mut m = ModelType::new(store.next_type_id(), "Base".to_string(), None, None);
            m.decorators = vec![DecoratorApplication {
                definition: Some(dec_id),
                decorator: 0,
                args: vec![],
                node: None,
            }];
            m.is_finished = true;
            m
        }));
        // Derived model extends base
        let derived = store.add(Type::Model({
            let mut m = ModelType::new(store.next_type_id(), "Derived".to_string(), None, None);
            m.base_model = Some(base);
            m.is_finished = true;
            m
        }));
        let result = validate_decorator_not_on_type(&store, derived, "sensitive", "secret");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, "decorator-conflict");
    }

    #[test]
    fn test_validate_decorator_not_on_type_scalar_base() {
        let mut store = TypeStore::new();
        let dec_id = store.add(Type::Decorator(DecoratorType {
            id: store.next_type_id(),
            name: "sensitive".to_string(),
            node: None,
            namespace: None,
            target: None,
            target_type: "unknown".to_string(),
            parameters: vec![],
            is_finished: true,
        }));
        // Base scalar has the decorator
        let base = store.add(Type::Scalar({
            let mut s = ScalarType::new(
                store.next_type_id(),
                "BaseScalar".to_string(),
                None,
                None,
                None,
            );
            s.decorators = vec![DecoratorApplication {
                definition: Some(dec_id),
                decorator: 0,
                args: vec![],
                node: None,
            }];
            s.is_finished = true;
            s
        }));
        // Derived scalar extends base
        let derived = store.add(Type::Scalar({
            let mut s = ScalarType::new(
                store.next_type_id(),
                "DerivedScalar".to_string(),
                None,
                None,
                Some(base),
            );
            s.is_finished = true;
            s
        }));
        let result = validate_decorator_not_on_type(&store, derived, "sensitive", "secret");
        assert!(result.is_err());
    }

    // ============================================================================
    // isIntrinsicType tests
    // ============================================================================

    #[test]
    fn test_is_intrinsic_type_string() {
        let mut store = TypeStore::new();
        let string_scalar = store.add(Type::Scalar({
            let mut s =
                ScalarType::new(store.next_type_id(), "string".to_string(), None, None, None);
            s.is_finished = true;
            s
        }));
        assert!(is_intrinsic_type(&store, string_scalar, "string"));
        assert!(!is_intrinsic_type(&store, string_scalar, "int32"));
    }

    #[test]
    fn test_is_intrinsic_type_derived() {
        let mut store = TypeStore::new();
        let string_scalar = store.add(Type::Scalar({
            let mut s =
                ScalarType::new(store.next_type_id(), "string".to_string(), None, None, None);
            s.is_finished = true;
            s
        }));
        let derived = store.add(Type::Scalar({
            let mut s = ScalarType::new(
                store.next_type_id(),
                "MyString".to_string(),
                None,
                None,
                Some(string_scalar),
            );
            s.is_finished = true;
            s
        }));
        assert!(is_intrinsic_type(&store, derived, "string"));
    }

    // ============================================================================
    // typespecTypeToJson - additional tests
    // ============================================================================

    #[test]
    fn test_typespec_type_to_value_enum_member() {
        let mut store = TypeStore::new();
        let member = store.add(Type::EnumMember(EnumMemberType {
            id: store.next_type_id(),
            name: "red".to_string(),
            value: None,
            r#enum: None,
            source_member: None,
            decorators: vec![],
            node: None,
            is_finished: true,
        }));
        let result = typespec_type_to_value(&store, member).unwrap();
        assert_eq!(result, TypeSpecValue::String("red".to_string()));
    }

    #[test]
    fn test_typespec_type_to_value_tuple() {
        let mut store = TypeStore::new();
        let s1 = store.add(Type::String(StringType {
            id: store.next_type_id(),
            value: "a".to_string(),
            node: None,
            is_finished: true,
        }));
        let s2 = store.add(Type::String(StringType {
            id: store.next_type_id(),
            value: "b".to_string(),
            node: None,
            is_finished: true,
        }));
        let tuple = store.add(Type::Tuple(TupleType {
            id: store.next_type_id(),
            values: vec![s1, s2],
            node: None,
            is_finished: true,
        }));
        let result = typespec_type_to_value(&store, tuple).unwrap();
        assert_eq!(
            result,
            TypeSpecValue::Array(vec![
                TypeSpecValue::String("a".to_string()),
                TypeSpecValue::String("b".to_string()),
            ])
        );
    }

    #[test]
    fn test_typespec_type_to_value_model() {
        let mut store = TypeStore::new();
        let string_val = store.add(Type::String(StringType {
            id: store.next_type_id(),
            value: "hello".to_string(),
            node: None,
            is_finished: true,
        }));
        let prop = store.add(Type::ModelProperty(ModelPropertyType {
            id: store.next_type_id(),
            name: "name".to_string(),
            node: None,
            r#type: string_val,
            optional: false,
            default_value: None,
            model: None,
            source_property: None,
            decorators: vec![],
            is_finished: true,
        }));
        let mut properties = HashMap::new();
        properties.insert("name".to_string(), prop);
        let model = store.add(Type::Model({
            let mut m = ModelType::new(store.next_type_id(), "Foo".to_string(), None, None);
            m.properties = properties;
            m.property_names = vec!["name".to_string()];
            m.is_finished = true;
            m
        }));
        let result = typespec_type_to_value(&store, model).unwrap();
        match result {
            TypeSpecValue::Object(pairs) => {
                assert_eq!(pairs.len(), 1);
                assert_eq!(pairs[0].0, "name");
                assert_eq!(pairs[0].1, TypeSpecValue::String("hello".to_string()));
            }
            _ => panic!("Expected Object"),
        }
    }
}
