//! Type graph JSON serialization for WASM extension consumption
#![allow(dead_code)]
//!
//! Serializes the Checker's type graph and StateAccessors to JSON,
//! so WASM extensions can walk the type graph and read decorator state.

use typespec_rs::checker::types::DecoratorArgument;
use typespec_rs::checker::{Checker, Type, TypeId};

/// Serializes the type graph from a Checker into a JSON string
pub struct TypeGraphSerializer;

impl TypeGraphSerializer {
    /// Serialize the entire type graph and decorator state as JSON
    pub fn serialize(checker: &Checker) -> String {
        let mut json = String::new();
        json.push('{');

        // Global namespace ID
        if let Some(ns_id) = checker.global_namespace_type {
            json.push_str(&format!("\"global_namespace_id\":{},", ns_id));
        }

        // Types map
        json.push_str("\"types\":{");
        let type_store = &checker.type_store;
        let mut first = true;
        for id in 0..type_store.len() {
            let type_id = id as TypeId;
            if let Some(t) = type_store.get(type_id) {
                if !first {
                    json.push(',');
                }
                json.push_str(&format!("\"{}\":", id));
                Self::serialize_type(&mut json, t);
                first = false;
            }
        }
        json.push('}');

        // State map
        json.push_str(",\"state\":");
        Self::serialize_state(&mut json, checker);

        json.push('}');
        json
    }

    fn serialize_type(json: &mut String, t: &Type) {
        json.push('{');
        json.push_str(&format!("\"kind\":\"{}\"", t.kind_name()));

        match t {
            Type::Namespace(ns) => {
                if !ns.name.is_empty() {
                    json.push_str(&format!(",\"name\":\"{}\"", escape_json(&ns.name)));
                }
                if !ns.models.is_empty() || !ns.enums.is_empty() || !ns.interfaces.is_empty() {
                    json.push_str(",\"members\":{");
                    let mut first = true;
                    for (name, &id) in &ns.models {
                        if !first {
                            json.push(',');
                        }
                        json.push_str(&format!("\"{}\":{}", escape_json(name), id));
                        first = false;
                    }
                    for (name, &id) in &ns.enums {
                        if !first {
                            json.push(',');
                        }
                        json.push_str(&format!("\"{}\":{}", escape_json(name), id));
                        first = false;
                    }
                    for (name, &id) in &ns.interfaces {
                        if !first {
                            json.push(',');
                        }
                        json.push_str(&format!("\"{}\":{}", escape_json(name), id));
                        first = false;
                    }
                    for (name, &id) in &ns.scalars {
                        if !first {
                            json.push(',');
                        }
                        json.push_str(&format!("\"{}\":{}", escape_json(name), id));
                        first = false;
                    }
                    for (name, &id) in &ns.unions {
                        if !first {
                            json.push(',');
                        }
                        json.push_str(&format!("\"{}\":{}", escape_json(name), id));
                        first = false;
                    }
                    for (name, &id) in &ns.operations {
                        if !first {
                            json.push(',');
                        }
                        json.push_str(&format!("\"{}\":{}", escape_json(name), id));
                        first = false;
                    }
                    for (name, &id) in &ns.namespaces {
                        if !first {
                            json.push(',');
                        }
                        json.push_str(&format!("\"{}\":{}", escape_json(name), id));
                        first = false;
                    }
                    json.push('}');
                }
            }
            Type::Model(m) => {
                if !m.name.is_empty() {
                    json.push_str(&format!(",\"name\":\"{}\"", escape_json(&m.name)));
                }
                if let Some(base) = m.base_model {
                    json.push_str(&format!(",\"baseModel\":{}", base));
                }
                if !m.properties.is_empty() {
                    json.push_str(",\"properties\":{");
                    let mut first = true;
                    for name in &m.property_names {
                        if let Some(&id) = m.properties.get(name) {
                            if !first {
                                json.push(',');
                            }
                            json.push_str(&format!("\"{}\":{}", escape_json(name), id));
                            first = false;
                        }
                    }
                    json.push('}');
                }
                if let Some((key, val)) = &m.indexer {
                    json.push_str(&format!(
                        ",\"indexer\":{{\"key\":{},\"value\":{}}}",
                        key, val
                    ));
                }
            }
            Type::ModelProperty(p) => {
                json.push_str(&format!(",\"name\":\"{}\"", escape_json(&p.name)));
                json.push_str(&format!(",\"type\":{}", p.r#type));
                json.push_str(&format!(",\"optional\":{}", p.optional));
            }
            Type::Interface(i) => {
                if !i.name.is_empty() {
                    json.push_str(&format!(",\"name\":\"{}\"", escape_json(&i.name)));
                }
                if !i.operations.is_empty() {
                    json.push_str(",\"operations\":{");
                    let mut first = true;
                    for name in &i.operation_names {
                        if let Some(&id) = i.operations.get(name) {
                            if !first {
                                json.push(',');
                            }
                            json.push_str(&format!("\"{}\":{}", escape_json(name), id));
                            first = false;
                        }
                    }
                    json.push('}');
                }
            }
            Type::Operation(op) => {
                if !op.name.is_empty() {
                    json.push_str(&format!(",\"name\":\"{}\"", escape_json(&op.name)));
                }
                if let Some(rt) = op.return_type {
                    json.push_str(&format!(",\"returnType\":{}", rt));
                }
            }
            Type::Enum(e) => {
                if !e.name.is_empty() {
                    json.push_str(&format!(",\"name\":\"{}\"", escape_json(&e.name)));
                }
                if !e.members.is_empty() {
                    json.push_str(",\"members\":{");
                    let mut first = true;
                    for name in &e.member_names {
                        if let Some(&id) = e.members.get(name) {
                            if !first {
                                json.push(',');
                            }
                            json.push_str(&format!("\"{}\":{}", escape_json(name), id));
                            first = false;
                        }
                    }
                    json.push('}');
                }
            }
            Type::EnumMember(m) => {
                json.push_str(&format!(",\"name\":\"{}\"", escape_json(&m.name)));
                if let Some(e) = m.r#enum {
                    json.push_str(&format!(",\"enum\":{}", e));
                }
                if let Some(v) = m.value {
                    json.push_str(&format!(",\"value\":{}", v));
                }
            }
            Type::Union(u) => {
                if !u.name.is_empty() {
                    json.push_str(&format!(",\"name\":\"{}\"", escape_json(&u.name)));
                }
                if !u.variants.is_empty() {
                    json.push_str(",\"variants\":{");
                    let mut first = true;
                    for name in &u.variant_names {
                        if let Some(&id) = u.variants.get(name) {
                            if !first {
                                json.push(',');
                            }
                            json.push_str(&format!("\"{}\":{}", escape_json(name), id));
                            first = false;
                        }
                    }
                    json.push('}');
                }
            }
            Type::UnionVariant(v) => {
                json.push_str(&format!(",\"name\":\"{}\"", escape_json(&v.name)));
                json.push_str(&format!(",\"type\":{}", v.r#type));
            }
            Type::Scalar(s) => {
                if !s.name.is_empty() {
                    json.push_str(&format!(",\"name\":\"{}\"", escape_json(&s.name)));
                }
                if let Some(base) = s.base_scalar {
                    json.push_str(&format!(",\"baseScalar\":{}", base));
                }
            }
            Type::Intrinsic(i) => {
                let name_str = match i.name {
                    typespec_rs::checker::types::IntrinsicTypeName::ErrorType => "ErrorType",
                    typespec_rs::checker::types::IntrinsicTypeName::Void => "void",
                    typespec_rs::checker::types::IntrinsicTypeName::Never => "never",
                    typespec_rs::checker::types::IntrinsicTypeName::Unknown => "unknown",
                    typespec_rs::checker::types::IntrinsicTypeName::Null => "null",
                };
                json.push_str(&format!(",\"name\":\"{}\"", name_str));
            }
            Type::String(s) => {
                json.push_str(&format!(",\"value\":\"{}\"", escape_json(&s.value)));
            }
            Type::Number(n) => {
                json.push_str(&format!(",\"value\":{}", n.value));
            }
            Type::Boolean(b) => {
                json.push_str(&format!(",\"value\":{}", b.value));
            }
            Type::Tuple(t) => {
                if !t.values.is_empty() {
                    json.push_str(",\"values\":[");
                    let mut first = true;
                    for &v in &t.values {
                        if !first {
                            json.push(',');
                        }
                        json.push_str(&format!("{}", v));
                        first = false;
                    }
                    json.push(']');
                }
            }
            Type::TemplateParameter(tp) => {
                json.push_str(&format!(",\"name\":\"{}\"", escape_json(&tp.name)));
                if let Some(c) = tp.constraint {
                    json.push_str(&format!(",\"constraint\":{}", c));
                }
            }
            Type::Decorator(d) => {
                json.push_str(&format!(",\"name\":\"{}\"", escape_json(&d.name)));
            }
            Type::ScalarConstructor(sc) => {
                json.push_str(&format!(",\"name\":\"{}\"", escape_json(&sc.name)));
            }
            Type::FunctionType(ft) => {
                json.push_str(&format!(",\"name\":\"{}\"", escape_json(&ft.name)));
            }
            Type::FunctionParameter(fp) => {
                json.push_str(&format!(",\"name\":\"{}\"", escape_json(&fp.name)));
            }
            Type::StringTemplate(st) => {
                if !st.spans.is_empty() {
                    json.push_str(&format!(",\"spansCount\":{}", st.spans.len()));
                }
            }
            Type::StringTemplateSpan(_) => {}
        }

        // Decorators (attached to the type)
        if let Some(decorators) = t.decorators()
            && !decorators.is_empty()
        {
                json.push_str(",\"decorators\":[");
                let mut first = true;
                for dec in decorators {
                    if !first {
                        json.push(',');
                    }
                    json.push('{');
                    if let Some(def) = dec.definition {
                        json.push_str(&format!("\"definition\":{}", def));
                    }
                    json.push_str(",\"args\":[");
                    let mut af = true;
                    for arg in &dec.args {
                        if !af {
                            json.push(',');
                        }
                        Self::serialize_decorator_arg(json, arg);
                        af = false;
                    }
                    json.push_str("]}");
                    first = false;
                }
                json.push(']');
        }

        json.push('}');
    }

    fn serialize_decorator_arg(json: &mut String, arg: &DecoratorArgument) {
        json.push('{');
        json.push_str(&format!("\"value\":{}", arg.value));
        if let Some(ref js_val) = arg.js_value {
            json.push_str(",\"jsValue\":");
            match js_val {
                typespec_rs::checker::types::DecoratorMarshalledValue::String(s) => {
                    json.push_str(&format!("\"{}\"", escape_json(s)));
                }
                typespec_rs::checker::types::DecoratorMarshalledValue::Number(n) => {
                    json.push_str(&format!("{}", n));
                }
                typespec_rs::checker::types::DecoratorMarshalledValue::Boolean(b) => {
                    json.push_str(&format!("{}", b));
                }
                typespec_rs::checker::types::DecoratorMarshalledValue::Null => {
                    json.push_str("null");
                }
                typespec_rs::checker::types::DecoratorMarshalledValue::Type(id) => {
                    json.push_str(&format!("{{\"type\":{}}}", id));
                }
                typespec_rs::checker::types::DecoratorMarshalledValue::Value(id) => {
                    json.push_str(&format!("{{\"value\":{}}}", id));
                }
                typespec_rs::checker::types::DecoratorMarshalledValue::Record(map) => {
                    json.push('{');
                    let mut first = true;
                    for (k, &v) in map {
                        if !first {
                            json.push(',');
                        }
                        json.push_str(&format!("\"{}\":{}", escape_json(k), v));
                        first = false;
                    }
                    json.push('}');
                }
                typespec_rs::checker::types::DecoratorMarshalledValue::Array(arr) => {
                    json.push('[');
                    let mut first = true;
                    for &v in arr {
                        if !first {
                            json.push(',');
                        }
                        json.push_str(&format!("{}", v));
                        first = false;
                    }
                    json.push(']');
                }
            }
        }
        json.push('}');
    }

    fn serialize_state(json: &mut String, checker: &Checker) {
        json.push('{');
        let state = &checker.state_accessors;
        let mut first = true;
        for (key, map) in state.iter_state_maps() {
            if map.is_empty() {
                continue;
            }
            if !first {
                json.push(',');
            }
            json.push_str(&format!("\"{}\":{{", escape_json(key)));
            let mut mf = true;
            for (&type_id, value) in map {
                if !mf {
                    json.push(',');
                }
                json.push_str(&format!("\"{}\":\"{}\"", type_id, escape_json(value)));
                mf = false;
            }
            json.push('}');
            first = false;
        }
        for (key, set) in state.iter_state_sets() {
            if set.is_empty() {
                continue;
            }
            if !first {
                json.push(',');
            }
            json.push_str(&format!("\"{}_set\":{{", escape_json(key)));
            let mut sf = true;
            for &type_id in set {
                if !sf {
                    json.push(',');
                }
                json.push_str(&format!("\"{}\":true", type_id));
                sf = false;
            }
            json.push('}');
            first = false;
        }
        json.push('}');
    }
}

/// Escape a string for JSON output
fn escape_json(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            c if c.is_control() => result.push_str(&format!("\\u{:04x}", c as u32)),
            c => result.push(c),
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(source: &str) -> typespec_rs::checker::Checker {
        let parse_result = typespec_rs::parser::parse(source);
        let mut checker = typespec_rs::checker::Checker::new();
        checker.set_parse_result(parse_result.root_id, parse_result.builder.clone());
        checker.check_program();
        checker
    }

    fn parse_json(json: &str) -> serde_json::Value {
        serde_json::from_str(json).expect("serialized output should be valid JSON")
    }

    #[test]
    fn test_serialize_simple_model() {
        let checker = check("model Pet { name: string; id: int32; }");
        let json = TypeGraphSerializer::serialize(&checker);
        let v = parse_json(&json);
        assert!(v["types"].is_object());
        assert!(v["global_namespace_id"].is_number());
    }

    #[test]
    fn test_serialize_enum() {
        let checker = check("enum Color { red, green, blue }");
        let json = TypeGraphSerializer::serialize(&checker);
        assert!(json.contains("\"Color\""));
        assert!(json.contains("\"red\""));
    }

    #[test]
    fn test_escape_json() {
        assert_eq!(escape_json("hello"), "hello");
        assert_eq!(escape_json("say \"hi\""), "say \\\"hi\\\"");
        assert_eq!(escape_json("a\nb"), "a\\nb");
        assert_eq!(escape_json("a\tb"), "a\\tb");
        assert_eq!(escape_json("a\rb"), "a\\rb");
        assert_eq!(escape_json("a\\b"), "a\\\\b");
    }

    #[test]
    fn test_serialize_model_with_optional() {
        let checker = check("model Pet { name: string; age?: int32; }");
        let json = TypeGraphSerializer::serialize(&checker);
        let v = parse_json(&json);
        // Find the ModelProperty type for "age" and check optional
        let types = &v["types"];
        for (_, t) in types.as_object().unwrap() {
            if t["kind"] == "ModelProperty" && t["name"] == "age" {
                assert_eq!(t["optional"], true);
            }
        }
    }

    #[test]
    fn test_serialize_union() {
        let checker = check("union Shape { circle: string, square: int32 }");
        let json = TypeGraphSerializer::serialize(&checker);
        assert!(json.contains("\"Shape\""));
        assert!(json.contains("\"circle\""));
    }

    #[test]
    fn test_serialize_scalar() {
        let checker = check("scalar myString extends string;");
        let json = TypeGraphSerializer::serialize(&checker);
        let v = parse_json(&json);
        let types = &v["types"];
        let found = types.as_object().unwrap().values().any(|t| {
            t["kind"] == "Scalar" && t["name"] == "myString"
        });
        assert!(found, "should find myString scalar");
    }

    #[test]
    fn test_serialize_namespace_with_members() {
        let checker = check("namespace MyNs { model Foo { x: int32; } }");
        let json = TypeGraphSerializer::serialize(&checker);
        let v = parse_json(&json);
        let types = &v["types"];
        let found = types.as_object().unwrap().values().any(|t| {
            t["kind"] == "Namespace" && t["name"] == "MyNs"
        });
        assert!(found, "should find MyNs namespace");
    }

    #[test]
    fn test_serialize_interface() {
        let checker = check("interface MyApi { op get(): string; }");
        let json = TypeGraphSerializer::serialize(&checker);
        assert!(json.contains("\"MyApi\""));
    }

    #[test]
    fn test_serialize_tuple() {
        let checker = check("model M { val: [string, int32]; }");
        let json = TypeGraphSerializer::serialize(&checker);
        let v = parse_json(&json);
        let types = &v["types"];
        let found = types.as_object().unwrap().values().any(|t| {
            t["kind"] == "Tuple"
        });
        assert!(found, "should find a Tuple type");
    }

    #[test]
    fn test_serialize_model_inheritance() {
        let checker = check("model Base { id: string; } model Derived extends Base { name: string; }");
        let json = TypeGraphSerializer::serialize(&checker);
        let v = parse_json(&json);
        let types = &v["types"];
        let found = types.as_object().unwrap().values().any(|t| {
            t["kind"] == "Model" && t["name"] == "Derived" && t["baseModel"].is_number()
        });
        assert!(found, "Derived should have baseModel");
    }

    #[test]
    fn test_serialize_state() {
        let checker = check("model Pet { name: string; }");
        let json = TypeGraphSerializer::serialize(&checker);
        let v = parse_json(&json);
        assert!(v["state"].is_object());
    }

    #[test]
    fn test_serialize_with_custom_decorator() {
        let parse_result = typespec_rs::parser::parse("model Config { name: string; }");
        let mut checker = typespec_rs::checker::Checker::new();
        checker.set_parse_result(parse_result.root_id, parse_result.builder.clone());
        checker.register_decorator("command", "CLI", "unknown");
        checker.register_decorator("flag", "CLI", "unknown");
        checker.check_program();

        let json = TypeGraphSerializer::serialize(&checker);
        let v = parse_json(&json);

        // Verify CLI namespace and its decorators are in the type graph
        let types = &v["types"];
        let cli_ns = types.as_object().unwrap().values().find(|t| {
            t["kind"] == "Namespace" && t["name"] == "CLI"
        });
        assert!(cli_ns.is_some(), "should find CLI namespace");

        let cmd_dec = types.as_object().unwrap().values().find(|t| {
            t["kind"] == "Decorator" && t["name"] == "command"
        });
        assert!(cmd_dec.is_some(), "should find command decorator");

        let flag_dec = types.as_object().unwrap().values().find(|t| {
            t["kind"] == "Decorator" && t["name"] == "flag"
        });
        assert!(flag_dec.is_some(), "should find flag decorator");
    }

    #[test]
    fn test_serialize_intrinsic_types() {
        let checker = check("model M { a: string; b: int32; }");
        let json = TypeGraphSerializer::serialize(&checker);
        let v = parse_json(&json);
        let types = &v["types"];
        let intrinsic_count = types.as_object().unwrap().values()
            .filter(|t| t["kind"] == "Intrinsic")
            .count();
        assert!(intrinsic_count > 0, "should have intrinsic types");
    }
}
