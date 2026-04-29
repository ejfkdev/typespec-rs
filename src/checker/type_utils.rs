//! Type utility functions for TypeSpec-Rust
//! Ported from TypeSpec compiler/src/core/type-utils.ts

use crate::checker::types::{
    Entity, IntrinsicTypeName, ModelType, NamespaceType, Type, TypeId, TypeStore,
};

/// Check if a type is the ErrorType intrinsic
pub fn is_error_type(t: &Type) -> bool {
    matches!(t, Type::Intrinsic(intr) if intr.name == IntrinsicTypeName::ErrorType)
}

/// Check if a type is the void intrinsic
pub fn is_void_type(t: &Type) -> bool {
    matches!(t, Type::Intrinsic(intr) if intr.name == IntrinsicTypeName::Void)
}

/// Check if a type is the never intrinsic
pub fn is_never_type(t: &Type) -> bool {
    matches!(t, Type::Intrinsic(intr) if intr.name == IntrinsicTypeName::Never)
}

/// Check if a type is the unknown intrinsic
pub fn is_unknown_type(t: &Type) -> bool {
    matches!(t, Type::Intrinsic(intr) if intr.name == IntrinsicTypeName::Unknown)
}

/// Check if a type is the null intrinsic
pub fn is_null_type(t: &Type) -> bool {
    matches!(t, Type::Intrinsic(intr) if intr.name == IntrinsicTypeName::Null)
}

/// Check if a model is an array type (has an integer-keyed indexer)
/// Ported from TS isArrayModelType() — checks that the indexer key type is "integer".
pub fn is_array_model_type(store: &TypeStore, t: &ModelType) -> bool {
    t.indexer
        .is_some_and(|(key_id, _)| match store.get(key_id) {
            Some(Type::Scalar(s)) => s.name == "integer",
            _ => false,
        })
}

/// Check if a model is a record type (has a string-keyed indexer)
/// Ported from TS isRecordModelType() — checks that the indexer key type is "string".
pub fn is_record_model_type(store: &TypeStore, t: &ModelType) -> bool {
    t.indexer
        .is_some_and(|(key_id, _)| match store.get(key_id) {
            Some(Type::Scalar(s)) => s.name == "string",
            _ => false,
        })
}

/// Check if a namespace is the global namespace
/// Ported from TS isGlobalNamespace()
pub fn is_global_namespace(ns: &NamespaceType) -> bool {
    ns.name.is_empty()
}

/// Get the fully qualified name of a type
/// Ported from TS getFullyQualifiedSymbolName / getTypeName
pub fn get_fully_qualified_name(store: &TypeStore, type_id: TypeId) -> String {
    match store.get(type_id) {
        Some(t) => {
            // For named types, use qualify_name (includes namespace prefix if any)
            if let Some(name) = t.name() {
                // These types have namespace and should use qualify_name
                if matches!(
                    t,
                    Type::Model(_)
                        | Type::Scalar(_)
                        | Type::Interface(_)
                        | Type::Enum(_)
                        | Type::Union(_)
                        | Type::Namespace(_)
                        | Type::Operation(_)
                ) {
                    return qualify_name(store, name, t.namespace());
                }
                // Other named types (ModelProperty, EnumMember, etc.) return name directly
                return name.to_string();
            }
            // Fallback for other types
            match t {
                Type::Intrinsic(intr) => format!("{:?}", intr.name).to_lowercase(),
                Type::String(s) => format!("\"{}\"", s.value),
                Type::Number(n) => n.value_as_string.clone(),
                Type::Boolean(b) => if b.value { "true" } else { "false" }.to_string(),
                Type::Tuple(_) => "[...]".to_string(),
                _ => format!("Type#{}", type_id),
            }
        }
        None => format!("Type#{}", type_id),
    }
}

/// Get the namespace string for a namespace TypeId
/// Ported from TS getNamespaceString()
pub fn get_namespace_string(store: &TypeStore, ns_id: Option<TypeId>) -> String {
    match ns_id {
        None => String::new(),
        Some(id) => match store.get(id) {
            Some(Type::Namespace(ns)) => {
                if is_global_namespace(ns) {
                    String::new()
                } else {
                    qualify_name(store, &ns.name, ns.namespace)
                }
            }
            _ => String::new(),
        },
    }
}

/// Helper: qualify a name with its namespace
fn qualify_name(store: &TypeStore, name: &str, namespace: Option<TypeId>) -> String {
    let ns_str = get_namespace_string(store, namespace);
    if ns_str.is_empty() {
        name.to_string()
    } else {
        format!("{}.{}", ns_str, name)
    }
}

/// Check if a scalar type extends another scalar (directly or transitively)
/// Ported from TS scalarExtends()
pub fn scalar_extends(store: &TypeStore, source_id: TypeId, target_name: &str) -> bool {
    let mut current_id = Some(source_id);
    while let Some(id) = current_id {
        match store.get(id) {
            Some(Type::Scalar(s)) => {
                if s.name == target_name {
                    return true;
                }
                current_id = s.base_scalar;
            }
            _ => break,
        }
    }
    false
}

/// Get the index type (element type) of a Model type.
/// For array/record models, returns the indexer value type.
/// Ported from TS getIndexType().
pub fn get_index_type(store: &TypeStore, type_id: TypeId) -> Option<TypeId> {
    match store.get(type_id) {
        Some(Type::Model(m)) => m.indexer.map(|(_, value_id)| value_id),
        _ => None,
    }
}

/// Check if a type is an anonymous model (empty name).
/// Ported from TS helper used in various places.
pub fn is_anonymous_model(t: &Type) -> bool {
    matches!(t, Type::Model(m) if m.name.is_empty())
}

/// Get the type from a Type or Indeterminate entity.
/// Ported from TS checker.ts getTypeForTypeOrIndeterminate()
///
/// For a Type entity, returns the type itself.
/// For an Indeterminate entity, returns the inner type.
pub fn get_type_for_type_or_indeterminate(entity: &Entity) -> Option<TypeId> {
    match entity {
        Entity::Type(type_id) => Some(*type_id),
        Entity::Indeterminate(type_id) => Some(*type_id),
        Entity::Value(_) => None,
        Entity::MixedConstraint(_) => None,
    }
}

/// Check if a type is a numeric literal type
pub fn is_numeric_literal_type(t: &Type) -> bool {
    matches!(t, Type::Number(_))
}

/// Check if a type is a string literal type
pub fn is_string_literal_type(t: &Type) -> bool {
    matches!(t, Type::String(_))
}

/// Check if a type is a boolean literal type
pub fn is_boolean_literal_type(t: &Type) -> bool {
    matches!(t, Type::Boolean(_))
}

/// Get the string value of a type for display purposes.
/// Handles string literals, numeric literals, boolean literals, and enum members.
/// Ported from TS typeReferenceToString() / getEntityName()
pub fn type_to_string(store: &TypeStore, type_id: TypeId) -> String {
    match store.get(type_id) {
        Some(Type::String(s)) => format!("\"{}\"", s.value),
        Some(Type::Number(n)) => n.value_as_string.clone(),
        Some(Type::Boolean(b)) => if b.value { "true" } else { "false" }.to_string(),
        Some(Type::Intrinsic(intr)) => format!("{:?}", intr.name).to_lowercase(),
        Some(Type::Model(m)) => {
            if m.name.is_empty() {
                "{ ... }".to_string()
            } else {
                get_fully_qualified_name(store, type_id)
            }
        }
        Some(Type::Scalar(_s)) => get_fully_qualified_name(store, type_id),
        Some(Type::Enum(_e)) => get_fully_qualified_name(store, type_id),
        Some(Type::Union(u)) => {
            let variant_names: Vec<String> = u
                .variant_names
                .iter()
                .filter_map(|name| u.variants.get(name))
                .filter_map(|&vid| match store.get(vid) {
                    Some(Type::UnionVariant(v)) => Some(type_to_string(store, v.r#type)),
                    _ => None,
                })
                .collect();
            variant_names.join(" | ")
        }
        Some(Type::Tuple(t)) => {
            let items: Vec<String> = t.values.iter().map(|&v| type_to_string(store, v)).collect();
            format!("[{}]", items.join(", "))
        }
        Some(Type::EnumMember(m)) => m.name.clone(),
        Some(Type::TemplateParameter(p)) => p.name.clone(),
        _ => format!("Type#{}", type_id),
    }
}

/// Check if a type is a template instance (created by instantiating a template).
/// A template instance has a template_mapper set, or has a template_node that differs
/// from its own node (pointing to the original template declaration).
/// Note: Our checker currently sets template_node to the model's own node for template
/// declarations, so we check that template_node != node to distinguish instances.
/// Ported from TS isTemplateInstance().
pub fn is_template_instance(t: &Type) -> bool {
    match super::types::get_template_info(t) {
        Some(info) => {
            if info.has_template_mapper() {
                return true;
            }
            info.template_node()
                .is_some_and(|tn| info.node().is_some_and(|n| tn != n))
        }
        None => false,
    }
}

/// Check if a type is a template declaration (has template parameters but is not an instance).
/// A template declaration is the base template (e.g., `model Foo<T> {}`), not an instance
/// (e.g., `Foo<string>`). We detect this by checking if:
/// - It's not a template instance (no template_mapper, template_node == own node or None)
/// - Its source node has template instantiations in the checker's symbol_links
///
/// Ported from TS isTemplateDeclaration().
pub fn is_template_declaration(checker: &crate::checker::Checker, type_id: TypeId) -> bool {
    let Some(t) = checker.get_type(type_id) else {
        return false;
    };
    if is_template_instance(t) {
        return false;
    }

    let node = match super::types::get_template_info(t) {
        Some(info) => info.node(),
        None => None,
    };

    if let Some(node_id) = node
        && let Some(links) = checker.symbol_links.get(&node_id)
    {
        return links.instantiations.is_some();
    }
    false
}

/// Check if a type is a template declaration OR a template instance.
/// Ported from TS isTemplateDeclarationOrInstance().
pub fn is_template_declaration_or_instance(
    checker: &crate::checker::Checker,
    type_id: TypeId,
) -> bool {
    let Some(t) = checker.get_type(type_id) else {
        return false;
    };
    is_template_instance(t) || is_template_declaration(checker, type_id)
}

/// Check if a type is defined in a given namespace (optionally recursively).
/// Ported from TS isDeclaredInNamespace().
pub fn is_declared_in_namespace(
    store: &TypeStore,
    type_id: TypeId,
    namespace_id: TypeId,
    recursive: bool,
) -> bool {
    let Some(t) = store.get(type_id) else {
        return false;
    };
    let ns_id = t.namespace();

    match ns_id {
        None => false,
        Some(id) if id == namespace_id => true,
        Some(id) if recursive => {
            let mut current = Some(id);
            while let Some(cid) = current {
                if cid == namespace_id {
                    return true;
                }
                current = match store.get(cid) {
                    Some(Type::Namespace(n)) => n.namespace,
                    _ => None,
                };
            }
            false
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::types::*;
    use std::collections::HashMap;

    /// Helper to create a simple NamespaceType for testing
    fn make_namespace(name: &str, namespace: Option<TypeId>) -> NamespaceType {
        NamespaceType {
            id: 0,
            name: name.to_string(),
            node: None,
            namespace,
            models: HashMap::new(),
            model_names: vec![],
            scalars: HashMap::new(),
            scalar_names: vec![],
            operations: HashMap::new(),
            operation_names: vec![],
            interfaces: HashMap::new(),
            interface_names: vec![],
            enums: HashMap::new(),
            enum_names: vec![],
            unions: HashMap::new(),
            union_names: vec![],
            namespaces: HashMap::new(),
            namespace_names: vec![],
            decorator_declarations: HashMap::new(),
            decorator_declaration_names: vec![],
            function_declarations: HashMap::new(),
            function_declaration_names: vec![],
            decorators: vec![],
            doc: None,
            summary: None,
            is_finished: true,
        }
    }

    #[test]
    fn test_intrinsic_type_checks() {
        let mut store = TypeStore::new();
        let error_type = store.add(Type::Intrinsic(IntrinsicType {
            id: store.next_type_id(),
            name: IntrinsicTypeName::ErrorType,
            node: None,
            is_finished: true,
        }));
        let void_type = store.add(Type::Intrinsic(IntrinsicType {
            id: store.next_type_id(),
            name: IntrinsicTypeName::Void,
            node: None,
            is_finished: true,
        }));
        let never_type = store.add(Type::Intrinsic(IntrinsicType {
            id: store.next_type_id(),
            name: IntrinsicTypeName::Never,
            node: None,
            is_finished: true,
        }));
        let unknown_type = store.add(Type::Intrinsic(IntrinsicType {
            id: store.next_type_id(),
            name: IntrinsicTypeName::Unknown,
            node: None,
            is_finished: true,
        }));
        let null_type = store.add(Type::Intrinsic(IntrinsicType {
            id: store.next_type_id(),
            name: IntrinsicTypeName::Null,
            node: None,
            is_finished: true,
        }));

        assert!(is_error_type(store.get(error_type).unwrap()));
        assert!(is_void_type(store.get(void_type).unwrap()));
        assert!(is_never_type(store.get(never_type).unwrap()));
        assert!(is_unknown_type(store.get(unknown_type).unwrap()));
        assert!(is_null_type(store.get(null_type).unwrap()));

        // Cross-checks
        assert!(!is_error_type(store.get(void_type).unwrap()));
        assert!(!is_void_type(store.get(never_type).unwrap()));
    }

    #[test]
    fn test_get_type_for_type_or_indeterminate() {
        let entity_type = Entity::Type(42);
        let entity_value = Entity::Value(1);
        let entity_indeterminate = Entity::Indeterminate(99);
        let entity_mixed = Entity::MixedConstraint(MixedParameterConstraint {
            node: None,
            type_constraint: None,
            value_constraint: None,
        });

        assert_eq!(get_type_for_type_or_indeterminate(&entity_type), Some(42));
        assert_eq!(get_type_for_type_or_indeterminate(&entity_value), None);
        assert_eq!(
            get_type_for_type_or_indeterminate(&entity_indeterminate),
            Some(99)
        );
        assert_eq!(get_type_for_type_or_indeterminate(&entity_mixed), None);
    }

    #[test]
    fn test_literal_type_checks() {
        let mut store = TypeStore::new();
        let string_lit = store.add(Type::String(StringType {
            id: store.next_type_id(),
            value: "hello".to_string(),
            node: None,
            is_finished: true,
        }));
        let num_lit = store.add(Type::Number(NumericType {
            id: store.next_type_id(),
            value: 42.0,
            value_as_string: "42".to_string(),
            node: None,
            is_finished: true,
        }));
        let bool_lit = store.add(Type::Boolean(BooleanType {
            id: store.next_type_id(),
            value: true,
            node: None,
            is_finished: true,
        }));
        let model_type = store.add(Type::Model({
            let mut m = ModelType::new(store.next_type_id(), "Foo".to_string(), None, None);
            m.is_finished = true;
            m
        }));

        assert!(is_string_literal_type(store.get(string_lit).unwrap()));
        assert!(!is_string_literal_type(store.get(num_lit).unwrap()));
        assert!(is_numeric_literal_type(store.get(num_lit).unwrap()));
        assert!(!is_numeric_literal_type(store.get(model_type).unwrap()));
        assert!(is_boolean_literal_type(store.get(bool_lit).unwrap()));
        assert!(!is_boolean_literal_type(store.get(string_lit).unwrap()));
    }

    #[test]
    fn test_type_to_string() {
        let mut store = TypeStore::new();
        let string_lit = store.add(Type::String(StringType {
            id: store.next_type_id(),
            value: "hello".to_string(),
            node: None,
            is_finished: true,
        }));
        let num_lit = store.add(Type::Number(NumericType {
            id: store.next_type_id(),
            value: 42.0,
            value_as_string: "42".to_string(),
            node: None,
            is_finished: true,
        }));
        let bool_lit = store.add(Type::Boolean(BooleanType {
            id: store.next_type_id(),
            value: true,
            node: None,
            is_finished: true,
        }));
        let void_type = store.add(Type::Intrinsic(IntrinsicType {
            id: store.next_type_id(),
            name: IntrinsicTypeName::Void,
            node: None,
            is_finished: true,
        }));
        let model = store.add(Type::Model({
            let mut m = ModelType::new(store.next_type_id(), "Foo".to_string(), None, None);
            m.is_finished = true;
            m
        }));
        let anon_model = store.add(Type::Model({
            let mut m = ModelType::new(store.next_type_id(), String::new(), None, None);
            m.is_finished = true;
            m
        }));

        assert_eq!(type_to_string(&store, string_lit), "\"hello\"");
        assert_eq!(type_to_string(&store, num_lit), "42");
        assert_eq!(type_to_string(&store, bool_lit), "true");
        assert_eq!(type_to_string(&store, void_type), "void");
        assert_eq!(type_to_string(&store, model), "Foo");
        assert_eq!(type_to_string(&store, anon_model), "{ ... }");
    }

    #[test]
    fn test_is_global_namespace() {
        let global_ns = make_namespace("", None);
        assert!(is_global_namespace(&global_ns));

        // "TypeSpec" is a named namespace, NOT the global namespace
        let typespec_ns = make_namespace("TypeSpec", None);
        assert!(!is_global_namespace(&typespec_ns));

        let named_ns = make_namespace("MyApp", None);
        assert!(!is_global_namespace(&named_ns));
    }

    #[test]
    fn test_scalar_extends() {
        let mut store = TypeStore::new();
        // Create string base scalar
        let string_id = store.add(Type::Scalar({
            let mut s =
                ScalarType::new(store.next_type_id(), "string".to_string(), None, None, None);
            s.is_finished = true;
            s
        }));
        // Create custom scalar extending string
        let custom_id = store.add(Type::Scalar({
            let mut s = ScalarType::new(
                store.next_type_id(),
                "myString".to_string(),
                None,
                None,
                Some(string_id),
            );
            s.is_finished = true;
            s
        }));

        assert!(scalar_extends(&store, custom_id, "myString"));
        assert!(scalar_extends(&store, custom_id, "string"));
        assert!(!scalar_extends(&store, custom_id, "int32"));
        assert!(scalar_extends(&store, string_id, "string"));
        assert!(!scalar_extends(&store, string_id, "myString"));
    }

    #[test]
    fn test_is_anonymous_model() {
        let named_model = Type::Model({
            let mut m = ModelType::new(0, "Foo".to_string(), None, None);
            m.is_finished = true;
            m
        });
        assert!(!is_anonymous_model(&named_model));

        let anon_model = Type::Model({
            let mut m = ModelType::new(1, String::new(), None, None);
            m.is_finished = true;
            m
        });
        assert!(is_anonymous_model(&anon_model));
    }

    #[test]
    fn test_get_fully_qualified_name_no_namespace() {
        let mut store = TypeStore::new();
        let model_id = store.add(Type::Model({
            let mut m = ModelType::new(store.next_type_id(), "Foo".to_string(), None, None);
            m.is_finished = true;
            m
        }));
        assert_eq!(get_fully_qualified_name(&store, model_id), "Foo");
    }

    #[test]
    fn test_get_fully_qualified_name_with_namespace() {
        let mut store = TypeStore::new();
        let ns_id = store.add(Type::Namespace(Box::new(make_namespace("MyApp", None))));
        let model_id = store.add(Type::Model({
            let mut m = ModelType::new(store.next_type_id(), "Foo".to_string(), None, Some(ns_id));
            m.is_finished = true;
            m
        }));
        assert_eq!(get_fully_qualified_name(&store, model_id), "MyApp.Foo");
    }

    #[test]
    fn test_get_namespace_string() {
        let mut store = TypeStore::new();
        assert_eq!(get_namespace_string(&store, None), "");

        let ns_id = store.add(Type::Namespace(Box::new(make_namespace("MyApp", None))));
        assert_eq!(get_namespace_string(&store, Some(ns_id)), "MyApp");
    }

    #[test]
    fn test_is_array_and_record_model_type() {
        let mut store = TypeStore::new();
        let int_scalar = store.add(Type::Scalar({
            let mut s = ScalarType::new(
                store.next_type_id(),
                "integer".to_string(),
                None,
                None,
                None,
            );
            s.is_finished = true;
            s
        }));
        let string_scalar = store.add(Type::Scalar({
            let mut s =
                ScalarType::new(store.next_type_id(), "string".to_string(), None, None, None);
            s.is_finished = true;
            s
        }));

        // Array model: indexer key is integer
        let array_model_id = store.add(Type::Model({
            let mut m = ModelType::new(store.next_type_id(), "Array".to_string(), None, None);
            m.indexer = Some((int_scalar, 0));
            m.is_finished = true;
            m
        }));
        if let Type::Model(m) = store.get(array_model_id).unwrap() {
            assert!(is_array_model_type(&store, m));
            assert!(!is_record_model_type(&store, m));
        }

        // Record model: indexer key is string
        let record_model_id = store.add(Type::Model({
            let mut m = ModelType::new(store.next_type_id(), "Record".to_string(), None, None);
            m.indexer = Some((string_scalar, 0));
            m.is_finished = true;
            m
        }));
        if let Type::Model(m) = store.get(record_model_id).unwrap() {
            assert!(!is_array_model_type(&store, m));
            assert!(is_record_model_type(&store, m));
        }
    }
}
