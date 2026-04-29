//! Generate plausible names for anonymous types
//!
//! Ported from TypeSpec compiler/src/typekit/utils/get-plausible-name.ts

use crate::checker::types::TypeId;
use crate::checker::{Checker, Type};

/// Get a plausible name for a type, useful for anonymous types
/// that don't have explicit names
pub fn get_plausible_name(checker: &Checker, id: TypeId) -> String {
    match checker.get_type(id) {
        Some(Type::Model(m)) => {
            if !m.name.is_empty() {
                m.name.clone()
            } else {
                // Anonymous model: derive name from properties
                let prop_names: Vec<&str> = m.property_names.iter().map(|s| s.as_str()).collect();
                if prop_names.is_empty() {
                    "AnonymousModel".to_string()
                } else {
                    // Capitalize first letter of each property name
                    let parts: Vec<String> = prop_names
                        .iter()
                        .map(|p| {
                            let mut s = p.to_string();
                            if let Some(c) = s.chars().next() {
                                s = c.to_uppercase().to_string() + &s[c.len_utf8()..];
                            }
                            s
                        })
                        .collect();
                    format!("Anonymous_{}", parts.join("_"))
                }
            }
        }
        Some(Type::Union(u)) => {
            if !u.name.is_empty() {
                u.name.clone()
            } else {
                "AnonymousUnion".to_string()
            }
        }
        Some(Type::Enum(e)) => e.name.clone(),
        Some(Type::Scalar(s)) => s.name.clone(),
        Some(Type::Interface(i)) => i.name.clone(),
        Some(Type::Operation(op)) => op.name.clone(),
        Some(Type::Namespace(ns)) => ns.name.clone(),
        Some(Type::String(s)) => format!("String_{}", s.value),
        Some(Type::Number(n)) => format!("Number_{}", n.value_as_string),
        Some(Type::Boolean(b)) => format!("Boolean_{}", b.value),
        Some(Type::Tuple(t)) => {
            let value_names: Vec<String> = t
                .values
                .iter()
                .map(|&v| get_plausible_name(checker, v))
                .collect();
            format!("Tuple_{}", value_names.join("_"))
        }
        Some(Type::Intrinsic(i)) => format!("{:?}", i.name).to_lowercase(),
        _ => "Unknown".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::test_utils::check;

    #[test]
    fn test_named_model() {
        let checker = check("model Foo {}");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        assert_eq!(get_plausible_name(&checker, foo_id), "Foo");
    }

    #[test]
    fn test_scalar() {
        let checker = check("scalar MyS extends string;");
        let s_id = checker.declared_types.get("MyS").copied().unwrap();
        assert_eq!(get_plausible_name(&checker, s_id), "MyS");
    }

    #[test]
    fn test_enum() {
        let checker = check("enum Color { red }");
        let e_id = checker.declared_types.get("Color").copied().unwrap();
        assert_eq!(get_plausible_name(&checker, e_id), "Color");
    }
}
