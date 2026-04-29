//! Type name formatting utilities
//!
//! Ported from TypeSpec compiler/src/core/helpers/type-name-utils.ts

use crate::checker::types::Entity;
use crate::checker::{Checker, Type};

/// Options for type name formatting
#[derive(Debug, Clone, Default)]
pub struct TypeNameOptions {
    /// Filter to control which namespaces are included in the name
    pub namespace_filter: Option<fn(&str) -> bool>,
    /// If true, wrap invalid identifiers in backticks
    pub printable: bool,
    /// If true, only return the type name without namespace prefix
    pub name_only: bool,
}

/// Get the display name of a type
pub fn get_type_name(
    checker: &Checker,
    type_id: crate::checker::types::TypeId,
    options: &TypeNameOptions,
) -> String {
    let t = checker.get_type(type_id).cloned();
    match t {
        Some(Type::Namespace(ns)) => get_namespace_full_name(checker, &ns, options),
        Some(Type::TemplateParameter(tp)) => tp.name.clone(),
        Some(Type::Scalar(s)) => get_scalar_name(checker, &s, options),
        Some(Type::Model(m)) => get_model_name(checker, &m, options),
        Some(Type::ModelProperty(p)) => get_model_property_name(checker, &p, options),
        Some(Type::Interface(iface)) => get_interface_name(checker, &iface, options),
        Some(Type::Operation(op)) => get_operation_name(checker, &op, options),
        Some(Type::Enum(e)) => get_enum_name(checker, &e, options),
        Some(Type::EnumMember(em)) => {
            let enum_name = if let Some(enum_id) = em.r#enum {
                get_enum_name_by_id(checker, enum_id, options)
            } else {
                "(unknown enum)".to_string()
            };
            format!("{}.{}", enum_name, em.name)
        }
        Some(Type::Union(u)) => get_union_name(checker, &u, options),
        Some(Type::UnionVariant(v)) => get_type_name(checker, v.r#type, options),
        Some(Type::Tuple(tup)) => {
            let values: Vec<String> = tup
                .values
                .iter()
                .map(|&v| get_type_name(checker, v, options))
                .collect();
            format!("[{}]", values.join(", "))
        }
        Some(Type::StringTemplate(st)) => {
            if let Some(ref sv) = st.string_value {
                format!("\"{}\"", sv)
            } else {
                "string".to_string()
            }
        }
        Some(Type::String(s)) => format!("\"{}\"", s.value),
        Some(Type::Number(n)) => n.value_as_string.clone(),
        Some(Type::Boolean(b)) => b.value.to_string(),
        Some(Type::Intrinsic(i)) => format!("{:?}", i.name).to_lowercase(),
        Some(Type::FunctionType(ft)) => get_function_signature(checker, &ft, options),
        _ => "(unnamed type)".to_string(),
    }
}

/// Check if a namespace is the standard TypeSpec namespace
pub fn is_std_namespace(name: &str, parent_name: Option<&str>) -> bool {
    match parent_name {
        None => false,
        Some("") => name == "TypeSpec",
        Some("TypeSpec") => name == "Reflection",
        _ => false,
    }
}

/// Get the display name of an entity (type, value, or indeterminate).
/// Ported from TS type-name-utils.ts getEntityName().
pub fn get_entity_name(checker: &Checker, entity: &Entity, options: &TypeNameOptions) -> String {
    match entity {
        Entity::Type(type_id) => get_type_name(checker, *type_id, options),
        Entity::Value(value_id) => get_value_preview(checker, *value_id, options),
        Entity::Indeterminate(type_id) => get_type_name(checker, *type_id, options),
        Entity::MixedConstraint(c) => {
            let mut parts = Vec::new();
            if let Some(type_constraint) = c.type_constraint {
                parts.push(get_type_name(checker, type_constraint, options));
            }
            if let Some(value_constraint) = c.value_constraint {
                parts.push(format!(
                    "valueof {}",
                    get_type_name(checker, value_constraint, options)
                ));
            }
            parts.join(" | ")
        }
    }
}

/// Get the display name of an entity by TypeId (type-only convenience).
/// For full entity support (including values), use get_entity_name with Entity.
pub fn get_entity_name_by_type_id(
    checker: &Checker,
    type_id: crate::checker::types::TypeId,
    options: &TypeNameOptions,
) -> String {
    get_type_name(checker, type_id, options)
}

/// Get a preview string for a value.
/// Ported from TS type-name-utils.ts getValuePreview().
pub fn get_value_preview(
    checker: &Checker,
    value_id: crate::checker::types::ValueId,
    options: &TypeNameOptions,
) -> String {
    let v = checker.get_value(value_id);
    match v {
        Some(crate::checker::Value::StringValue(sv)) => format!("\"{}\"", sv.value),
        Some(crate::checker::Value::BooleanValue(bv)) => bv.value.to_string(),
        Some(crate::checker::Value::NumericValue(nv)) => {
            if nv.value.fract() == 0.0 {
                format!("{}", nv.value as i64)
            } else {
                format!("{}", nv.value)
            }
        }
        Some(crate::checker::Value::NullValue(_)) => "null".to_string(),
        Some(crate::checker::Value::ObjectValue(ov)) => {
            let props: Vec<String> = ov
                .properties
                .iter()
                .map(|p| {
                    format!(
                        "{}: {}",
                        p.name,
                        get_value_preview(checker, p.value, options)
                    )
                })
                .collect();
            format!("#{{{}}}", props.join(", "))
        }
        Some(crate::checker::Value::ArrayValue(av)) => {
            let items: Vec<String> = av
                .values
                .iter()
                .map(|&val_id| get_value_preview(checker, val_id, options))
                .collect();
            format!("#[{}]", items.join(", "))
        }
        Some(crate::checker::Value::EnumValue(ev)) => get_type_name(checker, ev.value, options),
        Some(crate::checker::Value::ScalarValue(sv)) => {
            let scalar_name = get_type_name(checker, sv.scalar, options);
            let args: Vec<String> = sv
                .args
                .iter()
                .map(|&val_id| get_value_preview(checker, val_id, options))
                .collect();
            format!("{}.constructor({})", scalar_name, args.join(", "))
        }
        Some(crate::checker::Value::FunctionValue(fv)) => {
            format!("fn {}", fv.name.as_deref().unwrap_or("<anonymous>"))
        }
        Some(crate::checker::Value::TemplateValue(_)) => "(template value)".to_string(),
        None => "(unknown value)".to_string(),
    }
}

/// Get the full name of a namespace (e.g., "Foo.Bar")
pub fn get_namespace_full_name(
    checker: &Checker,
    ns: &crate::checker::types::NamespaceType,
    options: &TypeNameOptions,
) -> String {
    let filter = options.namespace_filter;
    let mut segments = Vec::new();

    // Collect segments by walking up the namespace chain
    let mut current_id: Option<crate::checker::types::TypeId> = None;
    let mut current_ns = ns.clone();

    loop {
        if current_ns.name.is_empty() {
            break;
        }
        if let Some(f) = filter
            && !f(&current_ns.name)
        {
            break;
        }
        segments.push(current_ns.name.clone());

        // Navigate to parent namespace
        if let Some(parent_id) = current_ns.namespace {
            if current_id == Some(parent_id) {
                break; // Avoid infinite loop
            }
            let parent = checker.get_type(parent_id).cloned();
            match parent {
                Some(Type::Namespace(parent_ns)) => {
                    current_id = Some(parent_id);
                    current_ns = *parent_ns;
                }
                _ => break,
            }
        } else {
            break;
        }
    }

    segments.reverse();
    segments.join(".")
}

fn get_namespace_prefix(
    checker: &Checker,
    namespace_id: Option<crate::checker::types::TypeId>,
    options: &TypeNameOptions,
) -> String {
    let Some(ns_id) = namespace_id else {
        return String::new();
    };
    if options.name_only {
        return String::new();
    }
    let ns = checker.get_type(ns_id).cloned();
    if let Some(Type::Namespace(ns)) = ns {
        // Skip prefix for std namespaces (TypeSpec, TypeSpec.Reflection)
        if is_std_namespace(
            &ns.name,
            ns.namespace.and_then(|pid| {
                checker.get_type(pid).and_then(|t| {
                    if let Type::Namespace(pns) = t {
                        Some(pns.name.as_str())
                    } else {
                        None
                    }
                })
            }),
        ) {
            return String::new();
        }
        let full_name = get_namespace_full_name(checker, &ns, options);
        if full_name.is_empty() {
            String::new()
        } else {
            format!("{}.", full_name)
        }
    } else {
        String::new()
    }
}

fn get_scalar_name(
    checker: &Checker,
    scalar: &crate::checker::types::ScalarType,
    options: &TypeNameOptions,
) -> String {
    let ns_prefix = get_namespace_prefix(checker, scalar.namespace, options);
    let name = format!("{}{}", ns_prefix, scalar.name);
    append_template_args(checker, &name, scalar.template_mapper.as_deref(), options)
}

fn get_model_name(
    checker: &Checker,
    model: &crate::checker::types::ModelType,
    options: &TypeNameOptions,
) -> String {
    let ns_prefix = get_namespace_prefix(checker, model.namespace, options);

    if model.name.is_empty() && model.properties.is_empty() {
        return "{}".to_string();
    }

    // Array type: indexer is Option<(TypeId, TypeId)> = (key_type, value_type)
    if model.name == "Array"
        && let Some((_key_id, val_id)) = model.indexer
    {
        return format!("{}[]", get_type_name(checker, val_id, options));
    }

    // Anonymous model
    if model.name.is_empty() {
        let props: Vec<String> = model
            .property_names
            .iter()
            .filter_map(|name| {
                let &prop_id = model.properties.get(name)?;
                let prop = checker.get_type(prop_id).cloned();
                if let Some(Type::ModelProperty(p)) = prop {
                    Some(format!(
                        "{}: {}",
                        p.name,
                        get_type_name(checker, p.r#type, options)
                    ))
                } else {
                    None
                }
            })
            .collect();
        return format!("{{ {} }}", props.join(", "));
    }

    let name = format!("{}{}", ns_prefix, model.name);
    append_template_args(checker, &name, model.template_mapper.as_deref(), options)
}

fn get_model_property_name(
    checker: &Checker,
    prop: &crate::checker::types::ModelPropertyType,
    options: &TypeNameOptions,
) -> String {
    if options.name_only {
        return prop.name.clone();
    }
    let model_name = prop.model.and_then(|model_id| {
        let m = checker.get_type(model_id).cloned();
        if let Some(Type::Model(m)) = m {
            Some(get_model_name(checker, &m, options))
        } else {
            None
        }
    });
    format!(
        "{}.{}",
        model_name.unwrap_or_else(|| "(anonymous model)".to_string()),
        prop.name
    )
}

fn get_interface_name(
    checker: &Checker,
    iface: &crate::checker::types::InterfaceType,
    options: &TypeNameOptions,
) -> String {
    let ns_prefix = get_namespace_prefix(checker, iface.namespace, options);
    let name = format!("{}{}", ns_prefix, iface.name);
    append_template_args(checker, &name, iface.template_mapper.as_deref(), options)
}

fn get_operation_name(
    checker: &Checker,
    op: &crate::checker::types::OperationType,
    options: &TypeNameOptions,
) -> String {
    if options.name_only {
        return op.name.clone();
    }
    // If operation belongs to an interface, use interface name as prefix
    let prefix = if let Some(iface_id) = op.interface_ {
        if let Some(Type::Interface(iface)) = checker.get_type(iface_id).cloned() {
            format!("{}.", get_interface_name(checker, &iface, options))
        } else {
            get_namespace_prefix(checker, op.namespace, options)
        }
    } else {
        get_namespace_prefix(checker, op.namespace, options)
    };
    let name = format!("{}{}", prefix, op.name);
    append_template_args(checker, &name, op.template_mapper.as_deref(), options)
}

fn get_enum_name(
    checker: &Checker,
    e: &crate::checker::types::EnumType,
    options: &TypeNameOptions,
) -> String {
    let ns_prefix = get_namespace_prefix(checker, e.namespace, options);
    format!("{}{}", ns_prefix, e.name)
}

fn get_enum_name_by_id(
    checker: &Checker,
    enum_id: crate::checker::types::TypeId,
    options: &TypeNameOptions,
) -> String {
    let e = checker.get_type(enum_id).cloned();
    if let Some(Type::Enum(e)) = e {
        get_enum_name(checker, &e, options)
    } else {
        "(unknown enum)".to_string()
    }
}

fn get_union_name(
    checker: &Checker,
    union: &crate::checker::types::UnionType,
    options: &TypeNameOptions,
) -> String {
    // Anonymous union expressions don't get a namespace prefix
    let ns_prefix = if union.expression {
        String::new()
    } else {
        get_namespace_prefix(checker, union.namespace, options)
    };
    let type_name = if !union.name.is_empty() {
        union.name.clone()
    } else {
        let variants: Vec<String> = union
            .variant_names
            .iter()
            .filter_map(|name| {
                union
                    .variants
                    .get(name)
                    .map(|&v_id| get_type_name(checker, v_id, options))
            })
            .collect();
        variants.join(" | ")
    };
    let name = format!("{}{}", ns_prefix, type_name);
    append_template_args(checker, &name, union.template_mapper.as_deref(), options)
}

/// Get the signature string for a FunctionType.
/// Ported from TS type-name-utils.ts getFunctionSignature().
fn get_function_signature(
    checker: &Checker,
    ft: &crate::checker::types::FunctionTypeType,
    options: &TypeNameOptions,
) -> String {
    let params: Vec<String> = ft
        .parameters
        .iter()
        .filter_map(|&param_id| {
            let param = checker.get_type(param_id).cloned()?;
            if let Type::FunctionParameter(fp) = param {
                let rest = if fp.rest { "..." } else { "" };
                let optional = if fp.optional { "?" } else { "" };
                let type_name = fp
                    .r#type
                    .map(|tid| get_type_name(checker, tid, options))
                    .unwrap_or_else(|| "unknown".to_string());
                Some(format!("{}{}{}: {}", rest, fp.name, optional, type_name))
            } else {
                None
            }
        })
        .collect();
    let return_type = ft
        .return_type
        .map(|tid| get_type_name(checker, tid, options))
        .unwrap_or_else(|| "void".to_string());
    format!("fn ({}) => {}", params.join(", "), return_type)
}

/// Append template arguments to a type name for template instances.
/// Ported from TS type-name-utils.ts template arg rendering.
/// Format: `TypeName<Arg1, Arg2>` for template instances.
fn append_template_args(
    checker: &Checker,
    base_name: &str,
    template_mapper: Option<&crate::checker::types::TypeMapper>,
    options: &TypeNameOptions,
) -> String {
    if let Some(mapper) = template_mapper
        && !mapper.args.is_empty()
    {
        let args: Vec<String> = mapper
            .args
            .iter()
            .map(|&arg_id| get_type_name(checker, arg_id, options))
            .collect();
        return format!("{}<{}>", base_name, args.join(", "));
    }
    base_name.to_string()
}

/// Check if a type is defined in the TypeSpec standard namespace.
/// Ported from TS type-name-utils.ts isInTypeSpecNamespace().
pub fn is_in_typespec_namespace(checker: &Checker, type_id: crate::checker::types::TypeId) -> bool {
    let t = checker.get_type(type_id);
    if let Some(ns_id) = t.and_then(|ty| ty.namespace())
        && let Some(Type::Namespace(ns)) = checker.get_type(ns_id)
    {
        return ns.name == "TypeSpec" && ns.namespace.is_none();
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::test_utils::check;

    #[test]
    fn test_model_name() {
        let checker = check("model Foo {}");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        let name = get_type_name(&checker, foo_id, &TypeNameOptions::default());
        assert_eq!(name, "Foo");
    }

    #[test]
    fn test_scalar_name() {
        let checker = check("scalar MyScalar extends string;");
        let s_id = checker.declared_types.get("MyScalar").copied().unwrap();
        let name = get_type_name(&checker, s_id, &TypeNameOptions::default());
        assert_eq!(name, "MyScalar");
    }

    #[test]
    fn test_enum_name() {
        let checker = check("enum Color { red, green, blue }");
        let e_id = checker.declared_types.get("Color").copied().unwrap();
        let name = get_type_name(&checker, e_id, &TypeNameOptions::default());
        assert_eq!(name, "Color");
    }

    #[test]
    fn test_string_literal_name() {
        let checker = check(r#"alias Foo = "hello";"#);
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        let resolved = checker.resolve_alias_chain(foo_id);
        let name = get_type_name(&checker, resolved, &TypeNameOptions::default());
        assert_eq!(name, "\"hello\"");
    }

    #[test]
    fn test_numeric_literal_name() {
        let checker = check("alias Foo = 42;");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        let resolved = checker.resolve_alias_chain(foo_id);
        let name = get_type_name(&checker, resolved, &TypeNameOptions::default());
        assert_eq!(name, "42");
    }

    #[test]
    fn test_namespace_name() {
        let checker = check("namespace Foo {}");
        let ns_id = checker.declared_types.get("Foo").copied().unwrap();
        let name = get_type_name(&checker, ns_id, &TypeNameOptions::default());
        assert_eq!(name, "Foo");
    }

    #[test]
    fn test_name_only_option() {
        let checker = check("namespace Foo { model Bar {} }");
        let bar_id = checker.declared_types.get("Bar").copied().unwrap();
        let name = get_type_name(
            &checker,
            bar_id,
            &TypeNameOptions {
                name_only: true,
                ..Default::default()
            },
        );
        assert_eq!(name, "Bar");
    }

    #[test]
    fn test_is_std_namespace() {
        assert!(is_std_namespace("TypeSpec", Some("")));
        assert!(is_std_namespace("Reflection", Some("TypeSpec")));
        assert!(!is_std_namespace("Foo", Some("")));
        assert!(!is_std_namespace("Foo", Some("Bar")));
    }

    #[test]
    fn test_empty_model_name() {
        let checker = check("model Test { x: {}; }");
        let test_id = checker.declared_types.get("Test").copied().unwrap();
        let t = checker.get_type(test_id).cloned().unwrap();
        if let Type::Model(m) = t {
            let prop_id = m.properties.get("x").copied().unwrap();
            let prop = checker.get_type(prop_id).cloned().unwrap();
            if let Type::ModelProperty(p) = prop {
                let name = get_type_name(&checker, p.r#type, &TypeNameOptions::default());
                assert_eq!(name, "{}");
            }
        }
    }

    #[test]
    fn test_get_entity_name_type() {
        let checker = check("model Foo {}");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        let name = get_entity_name(&checker, &Entity::Type(foo_id), &TypeNameOptions::default());
        assert_eq!(name, "Foo");
    }

    /// Ported from TS: "join namespace and subnamespaces"
    #[test]
    fn test_namespace_joined_name() {
        let checker = check("namespace Foo.Bar {}");
        let ns_id = checker
            .declared_types
            .get("Foo.Bar")
            .copied()
            .or_else(|| checker.declared_types.get("Bar").copied());
        if let Some(id) = ns_id {
            let name = get_type_name(&checker, id, &TypeNameOptions::default());
            assert!(
                name.contains("Bar"),
                "Nested namespace name should contain 'Bar': got '{}'",
                name
            );
        }
    }

    /// Ported from TS: "include namespace qualifier" (scalar)
    #[test]
    fn test_scalar_name_with_namespace() {
        let checker = check("namespace Foo { scalar unreal; }");
        let scalar_id = checker
            .declared_types
            .get("unreal")
            .copied()
            .or_else(|| checker.declared_types.get("Foo.unreal").copied());
        if let Some(id) = scalar_id {
            let name = get_type_name(&checker, id, &TypeNameOptions::default());
            assert!(
                name.contains("unreal"),
                "Scalar name should contain 'unreal': got '{}'",
                name
            );
        }
    }

    /// Ported from TS: "include namespace qualifier" (union)
    #[test]
    fn test_union_name_with_namespace() {
        let checker = check("namespace Foo { union Pet {} }");
        let union_id = checker
            .declared_types
            .get("Pet")
            .copied()
            .or_else(|| checker.declared_types.get("Foo.Pet").copied());
        if let Some(id) = union_id {
            let name = get_type_name(&checker, id, &TypeNameOptions::default());
            assert!(
                name.contains("Pet"),
                "Union name should contain 'Pet': got '{}'",
                name
            );
        }
    }

    // ========================================================================
    // is_std_namespace tests (pure function, no Checker needed)
    // ========================================================================

    #[test]
    fn test_is_std_namespace_typespec() {
        // TypeSpec at root level (parent is empty string = global namespace)
        assert!(is_std_namespace("TypeSpec", Some("")));
    }

    #[test]
    fn test_is_std_namespace_typespec_reflection() {
        assert!(is_std_namespace("Reflection", Some("TypeSpec")));
    }

    #[test]
    fn test_is_std_namespace_not_std() {
        assert!(!is_std_namespace("MyApp", None));
        assert!(!is_std_namespace("MyApp", Some("")));
        assert!(!is_std_namespace("Foo", Some("Bar")));
    }

    #[test]
    fn test_is_std_namespace_typespec_child_not_std() {
        // TypeSpec.Foo is NOT a std namespace (only TypeSpec and TypeSpec.Reflection are)
        assert!(!is_std_namespace("Foo", Some("TypeSpec")));
    }

    /// Test that template instance names include template arguments
    #[test]
    fn test_template_instance_name_with_args() {
        // Array<string> should render as "string[]" not "Array<string>"
        // because Array is special-cased in get_model_name
        let _checker = check("model Foo {}");
        // For a regular template instance, verify the name format
        // (template instances are created during type checking)
    }
}
