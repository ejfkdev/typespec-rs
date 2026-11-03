//! JSON Schema Emitter
//!
//! Emits a JSON Schema (draft 2020-12) bundle from the Checker's type graph,
//! matching the output of the official `@typespec/json-schema` emitter run with
//! `emitAllModels: true` and a `bundleId`, for the subset of TypeSpec it covers:
//!
//! - models and their properties (optional properties are left out of
//!   `required`), `extends` as `allOf`, arrays and tuples;
//! - enums, named unions, union expressions (including `T | null`) and literal
//!   types;
//! - standard scalars (sized integers carry their `minimum`/`maximum`; `int64`
//!   and `uint64` are strings) and custom scalars;
//! - `@minValue`, `@maxValue`, `@minValueExclusive`, `@maxValueExclusive`,
//!   `@minLength`, `@maxLength`, `@minItems`, `@maxItems`, `@pattern` and
//!   `@format`;
//! - `@extension` from `TypeSpec.JsonSchema` with a string, number, boolean,
//!   array or type value.
//!
//! Every named model, enum, union and custom scalar declared in a project
//! namespace becomes a `$defs` entry with its own `$id` (`<Name>.json`), and
//! references to it are `{"$ref": "<Name>.json"}`.
//!
//! The emitter reads only the public `Checker` API.

use std::collections::HashSet;

use serde_json::{Map, Value, json};

use super::{EmitResult, Emitter};
use crate::checker::Checker;
use crate::checker::type_utils::is_array_model_type;
use crate::checker::types::*;
use crate::libs::json_schema::JSON_SCHEMA_NAMESPACE;
use crate::parser::ParseResult;

/// The `$schema` dialect written on the bundle and on every `$defs` entry.
const DIALECT: &str = "https://json-schema.org/draft/2020-12/schema";

/// Scalars whose name is a fixed numeric width, written as `format` when
/// [`JsonSchemaEmitter::width_format`] is on.
const WIDTH_SCALARS: &[&str] = &[
    "int8", "int16", "int32", "int64", "uint8", "uint16", "uint32", "uint64", "float32", "float64",
];

/// JSON Schema emitter.
///
/// # Example
/// ```ignore
/// use typespec_rs::emit::JsonSchemaEmitter;
///
/// let schema = JsonSchemaEmitter::new().bundle_id("pets.json").emit_value(&checker);
/// ```
#[derive(Debug, Clone, Default)]
pub struct JsonSchemaEmitter {
    bundle_id: Option<String>,
    width_format: bool,
}

impl JsonSchemaEmitter {
    /// Create an emitter with the official emitter's defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the bundle's root `$id`. Without one, the root has no `$id`.
    pub fn bundle_id(mut self, id: &str) -> Self {
        self.bundle_id = Some(id.to_string());
        self
    }

    /// Also write each fixed-width numeric scalar's name as `format`
    /// (`"int8"`, `"uint16"`, `"float32"`, ...). Off by default; the official
    /// emitter does not do this.
    pub fn width_format(mut self, on: bool) -> Self {
        self.width_format = on;
        self
    }

    /// Build the JSON Schema bundle for every project type in the checked program.
    pub fn emit_value(&self, checker: &Checker) -> Value {
        let mut walk = Walk {
            checker,
            width_format: self.width_format,
            defs: Map::new(),
            started: HashSet::new(),
        };
        if let Some(global) = checker.global_namespace_type {
            walk.namespace(global);
        }
        let mut root = Map::new();
        root.insert("$schema".into(), DIALECT.into());
        if let Some(id) = &self.bundle_id {
            root.insert("$id".into(), id.as_str().into());
        }
        root.insert("$defs".into(), Value::Object(walk.defs));
        Value::Object(root)
    }

    /// Emit the bundle as pretty-printed JSON.
    pub fn emit_from_checker(&self, checker: &Checker) -> Result<EmitResult, String> {
        let output = serde_json::to_string_pretty(&self.emit_value(checker))
            .map_err(|e| format!("cannot serialize JSON Schema: {e}"))?;
        Ok(EmitResult {
            output,
            format: "json-schema".to_string(),
            diagnostics_count: checker.diagnostics().len(),
        })
    }
}

impl Emitter for JsonSchemaEmitter {
    fn emit(&self, result: &ParseResult) -> Result<EmitResult, String> {
        let mut checker = Checker::new();
        checker.set_parse_result(result.root_id, result.builder.clone());
        checker.check_program();
        self.emit_from_checker(&checker)
    }

    fn format(&self) -> &str {
        "json-schema"
    }
}

/// One pass over the type graph, collecting `$defs` in the order the official
/// emitter writes them: a declaration takes its slot when first reached,
/// either by the namespace walk or by a reference.
struct Walk<'a> {
    checker: &'a Checker,
    width_format: bool,
    defs: Map<String, Value>,
    started: HashSet<TypeId>,
}

impl Walk<'_> {
    /// Declare a namespace's types: child namespaces, models, enums, unions,
    /// then scalars, skipping library code and templates.
    fn namespace(&mut self, ns_id: TypeId) {
        let Some(Type::Namespace(ns)) = self.checker.get_type(ns_id) else {
            return;
        };
        for name in &ns.namespace_names {
            if let Some(&id) = ns.namespaces.get(name)
                && self.is_project(id)
            {
                self.namespace(id);
            }
        }
        let declared = [
            (&ns.model_names, &ns.models),
            (&ns.enum_names, &ns.enums),
            (&ns.union_names, &ns.unions),
            (&ns.scalar_names, &ns.scalars),
        ];
        for (names, ids) in declared {
            for id in names.iter().filter_map(|n| ids.get(n)) {
                if self.is_project(*id) && self.declared_name(*id).is_some() {
                    self.reference(*id);
                }
            }
        }
    }

    /// Whether a type was written in project source, not a library.
    fn is_project(&self, id: TypeId) -> bool {
        let node = match self.checker.get_type(id) {
            Some(Type::Namespace(t)) => t.node,
            Some(Type::Model(t)) => t.node,
            Some(Type::Enum(t)) => t.node,
            Some(Type::Union(t)) => t.node,
            Some(Type::Scalar(t)) => t.node,
            _ => None,
        };
        node.is_some_and(|node| !self.checker.is_library_node(node))
    }

    /// The `$defs` key of a type that is hoisted into its own declaration.
    fn declared_name(&self, id: TypeId) -> Option<&str> {
        let name = match self.checker.get_type(id)? {
            Type::Model(m) if !self.is_template(m) => &m.name,
            Type::Enum(e) => &e.name,
            Type::Union(u)
                if !u.expression && u.template_node.is_none() && u.template_mapper.is_none() =>
            {
                &u.name
            }
            Type::Scalar(s) if !self.checker.is_std_type(id, None) => &s.name,
            _ => return None,
        };
        (!name.is_empty()).then_some(name.as_str())
    }

    /// Whether a model is a template declaration or instance, including the
    /// standard `Array` and `Record`; these are written in place.
    fn is_template(&self, m: &ModelType) -> bool {
        let std_template = (m.name == "Array" || m.name == "Record")
            && (m.namespace.is_none()
                || m.namespace == self.checker.lookup_type_by_fqn("TypeSpec"));
        std_template || m.template_node.is_some() || m.template_mapper.is_some()
    }

    /// The schema that stands for a type where it is used.
    fn reference(&mut self, id: TypeId) -> Value {
        if let Some(name) = self.declared_name(id).map(str::to_string) {
            if self.started.insert(id) {
                self.defs.insert(name.clone(), Value::Null);
                let schema = self.declaration(id, &name);
                self.defs.insert(name.clone(), Value::Object(schema));
            }
            return json!({ "$ref": format!("{name}.json") });
        }
        Value::Object(self.inline(id))
    }

    /// The `$defs` entry of a hoisted type.
    fn declaration(&mut self, id: TypeId, name: &str) -> Map<String, Value> {
        let mut schema = Map::new();
        schema.insert("$schema".into(), DIALECT.into());
        schema.insert("$id".into(), format!("{name}.json").into());
        let body = match self.checker.get_type(id) {
            Some(Type::Enum(e)) => self.enum_schema(e),
            Some(Type::Scalar(s)) => self.scalar(s),
            _ => self.inline(id),
        };
        schema.extend(body);
        if !matches!(self.checker.get_type(id), Some(Type::Scalar(_))) {
            self.constraints(id, &mut schema);
        }
        schema
    }

    /// A type's schema written in place (no `$ref`).
    fn inline(&mut self, id: TypeId) -> Map<String, Value> {
        let checker = self.checker;
        match checker.get_type(id) {
            Some(Type::Intrinsic(i)) => match i.name {
                IntrinsicTypeName::Null => object([("type", "null".into())]),
                IntrinsicTypeName::Never | IntrinsicTypeName::Void => object([("not", json!({}))]),
                _ => Map::new(),
            },
            Some(Type::String(s)) => literal("string", s.value.as_str().into()),
            Some(Type::Number(n)) => literal("number", number(n.value)),
            Some(Type::Boolean(b)) => literal("boolean", b.value.into()),
            Some(Type::Model(m)) => self.model(m),
            Some(Type::ModelProperty(p)) => {
                let mut schema = self.reference_map(p.r#type);
                self.constraints(id, &mut schema);
                schema
            }
            Some(Type::Union(u)) => {
                let variants = u
                    .variant_names
                    .iter()
                    .filter_map(|n| u.variants.get(n))
                    .map(|&v| Value::Object(self.inline(v)))
                    .collect();
                object([("anyOf", Value::Array(variants))])
            }
            Some(Type::UnionVariant(v)) => {
                let mut schema = self.reference_map(v.r#type);
                self.constraints(id, &mut schema);
                schema
            }
            Some(Type::EnumMember(m)) => match m.value.and_then(|v| checker.get_type(v)) {
                Some(Type::Number(n)) => literal("number", number(n.value)),
                Some(Type::String(s)) => literal("string", s.value.as_str().into()),
                _ => literal("string", m.name.as_str().into()),
            },
            Some(Type::Scalar(s)) => self.scalar(s),
            Some(Type::Tuple(t)) => {
                let items = t.values.iter().map(|&v| self.reference(v)).collect();
                object([
                    ("type", "array".into()),
                    ("prefixItems", Value::Array(items)),
                    ("minItems", t.values.len().into()),
                    ("maxItems", t.values.len().into()),
                ])
            }
            _ => Map::new(),
        }
    }

    /// [`Self::reference`] as an object, so constraints can be added to it.
    fn reference_map(&mut self, id: TypeId) -> Map<String, Value> {
        match self.reference(id) {
            Value::Object(map) => map,
            _ => Map::new(),
        }
    }

    fn model(&mut self, m: &ModelType) -> Map<String, Value> {
        if is_array_model_type(&self.checker.type_store, m)
            && let Some((_, item)) = m.indexer
        {
            return object([("type", "array".into()), ("items", self.reference(item))]);
        }
        let mut properties = Map::new();
        let mut required = Vec::new();
        for name in &m.property_names {
            let Some(&prop_id) = m.properties.get(name) else {
                continue;
            };
            if let Some(Type::ModelProperty(p)) = self.checker.get_type(prop_id)
                && !p.optional
            {
                required.push(Value::from(name.as_str()));
            }
            properties.insert(name.clone(), Value::Object(self.inline(prop_id)));
        }
        let mut schema = object([
            ("type", "object".into()),
            ("properties", Value::Object(properties)),
        ]);
        if !required.is_empty() {
            schema.insert("required".into(), Value::Array(required));
        }
        if let Some(base) = m.base_model {
            schema.insert("allOf".into(), Value::Array(vec![self.reference(base)]));
        }
        if let Some((_, value)) = m.indexer {
            schema.insert("unevaluatedProperties".into(), self.reference(value));
        }
        schema
    }

    fn enum_schema(&self, e: &EnumType) -> Map<String, Value> {
        let mut kinds: Vec<&str> = Vec::new();
        let mut values = Vec::new();
        for member in e.member_names.iter().filter_map(|n| e.members.get(n)) {
            let Some(Type::EnumMember(m)) = self.checker.get_type(*member) else {
                continue;
            };
            let (kind, value) = match m.value.and_then(|v| self.checker.get_type(v)) {
                Some(Type::Number(n)) => ("number", number(n.value)),
                Some(Type::String(s)) => ("string", s.value.as_str().into()),
                _ => ("string", m.name.as_str().into()),
            };
            if !kinds.contains(&kind) {
                kinds.push(kind);
            }
            if !values.contains(&value) {
                values.push(value);
            }
        }
        let kind = match kinds.as_slice() {
            [one] => Value::from(*one),
            many => many.iter().copied().map(Value::from).collect(),
        };
        object([("type", kind), ("enum", Value::Array(values))])
    }

    /// A scalar's schema: its standard base's schema, then the constraints of
    /// each scalar on the way down to this one.
    fn scalar(&mut self, s: &ScalarType) -> Map<String, Value> {
        let mut schema = if self.checker.is_std_type(s.id, None) {
            self.std_scalar(&s.name)
        } else {
            match s.base_scalar.and_then(|b| self.checker.get_type(b)) {
                Some(Type::Scalar(base)) => self.scalar(base),
                _ => Map::new(),
            }
        };
        self.constraints(s.id, &mut schema);
        schema
    }

    fn std_scalar(&self, name: &str) -> Map<String, Value> {
        let integer = |min: i64, max: i64| {
            object([
                ("type", "integer".into()),
                ("minimum", min.into()),
                ("maximum", max.into()),
            ])
        };
        let typed = |t: &str| object([("type", t.into())]);
        let formatted = |f: &str| object([("type", "string".into()), ("format", f.into())]);
        let mut schema = match name {
            "int8" => integer(i8::MIN.into(), i8::MAX.into()),
            "int16" => integer(i16::MIN.into(), i16::MAX.into()),
            "int32" | "unixTimestamp32" => integer(i32::MIN.into(), i32::MAX.into()),
            "uint8" => integer(0, u8::MAX.into()),
            "uint16" => integer(0, u16::MAX.into()),
            "uint32" => integer(0, u32::MAX.into()),
            "int64" | "uint64" | "decimal" | "decimal128" | "string" => typed("string"),
            "integer" | "safeint" => typed("integer"),
            "float" | "float32" | "float64" | "numeric" => typed("number"),
            "boolean" => typed("boolean"),
            "plainDate" => formatted("date"),
            "plainTime" => formatted("time"),
            "utcDateTime" | "offsetDateTime" => formatted("date-time"),
            "duration" => formatted("duration"),
            "url" => formatted("uri"),
            "bytes" => object([
                ("type", "string".into()),
                ("contentEncoding", "base64".into()),
            ]),
            _ => Map::new(),
        };
        if self.width_format && WIDTH_SCALARS.contains(&name) {
            schema.insert("format".into(), name.into());
        }
        schema
    }

    /// Add the constraint keywords and `@extension` entries applied to a type.
    fn constraints(&mut self, id: TypeId, schema: &mut Map<String, Value>) {
        let c = self.checker;
        let numeric = [
            ("minLength", c.get_type_min_length(id)),
            ("maxLength", c.get_type_max_length(id)),
            ("minimum", c.get_type_min_value(id)),
            ("exclusiveMinimum", c.get_type_min_value_exclusive(id)),
            ("maximum", c.get_type_max_value(id)),
            ("exclusiveMaximum", c.get_type_max_value_exclusive(id)),
        ];
        for (key, value) in numeric {
            if let Some(v) = value {
                schema.insert(key.into(), number(v));
            }
        }
        if let Some(pattern) = c.get_type_pattern(id) {
            schema.insert("pattern".into(), pattern.into());
        }
        for (key, value) in [
            ("minItems", c.get_type_min_items(id)),
            ("maxItems", c.get_type_max_items(id)),
        ] {
            if let Some(v) = value {
                schema.insert(key.into(), number(v));
            }
        }
        // The standard `url` scalar carries `@format("url")`; JSON Schema says "uri".
        if !c.is_std_type(id, Some("url"))
            && let Some(format) = c.get_type_format(id)
        {
            schema.insert("format".into(), format.into());
        }
        for (key, value) in self.extensions(id) {
            schema.insert(key, value);
        }
    }

    /// The `@extension(key, value)` entries from `TypeSpec.JsonSchema` on a type,
    /// in the order the decorators apply (the one nearest the declaration first).
    fn extensions(&mut self, id: TypeId) -> Vec<(String, Value)> {
        let c = self.checker;
        let Some(library) = c.lookup_type_by_fqn(JSON_SCHEMA_NAMESPACE) else {
            return Vec::new();
        };
        let mut found = Vec::new();
        for dec in c.find_decorators(id, "extension").into_iter().rev() {
            let in_library = matches!(
                dec.definition.and_then(|d| c.get_type(d)),
                Some(Type::Decorator(d)) if d.namespace == Some(library)
            );
            let (true, Some(key), Some(value)) = (in_library, dec.args.first(), dec.args.get(1))
            else {
                continue;
            };
            let Some(DecoratorMarshalledValue::String(key)) = &key.js_value else {
                continue;
            };
            let value = match &value.js_value {
                Some(DecoratorMarshalledValue::Type(t)) if !self.is_raw_json(*t, library) => {
                    Some(self.reference(*t))
                }
                Some(DecoratorMarshalledValue::Type(_)) | None => None,
                Some(other) => self.raw_value(other),
            };
            if let Some(value) = value {
                found.push((key.clone(), value));
            }
        }
        found
    }

    /// Whether a type is an instance of `TypeSpec.JsonSchema.Json<T>`.
    fn is_raw_json(&self, id: TypeId, library: TypeId) -> bool {
        matches!(
            self.checker.get_type(id),
            Some(Type::Model(m)) if m.name == "Json" && m.namespace == Some(library)
        )
    }

    /// A decorator argument written as raw JSON.
    fn raw_value(&self, value: &DecoratorMarshalledValue) -> Option<Value> {
        match value {
            DecoratorMarshalledValue::String(s) => Some(s.as_str().into()),
            DecoratorMarshalledValue::Number(n) => Some(number(*n)),
            DecoratorMarshalledValue::Boolean(b) => Some((*b).into()),
            DecoratorMarshalledValue::Null => Some(Value::Null),
            DecoratorMarshalledValue::Array(items) => items
                .iter()
                .map(|&t| self.literal_value(t))
                .collect::<Option<Vec<_>>>()
                .map(Value::Array),
            _ => None,
        }
    }

    /// A literal type's value as raw JSON.
    fn literal_value(&self, id: TypeId) -> Option<Value> {
        match self.checker.get_type(id)? {
            Type::String(s) => Some(s.value.as_str().into()),
            Type::Number(n) => Some(number(n.value)),
            Type::Boolean(b) => Some(b.value.into()),
            Type::Intrinsic(i) if i.name == IntrinsicTypeName::Null => Some(Value::Null),
            _ => None,
        }
    }
}

/// `{"type": kind, "const": value}`.
fn literal(kind: &str, value: Value) -> Map<String, Value> {
    object([("type", kind.into()), ("const", value)])
}

/// An object from ordered entries.
fn object<const N: usize>(entries: [(&str, Value); N]) -> Map<String, Value> {
    entries
        .into_iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect()
}

/// A number written the way JavaScript prints it: whole numbers without `.0`.
fn number(n: f64) -> Value {
    if n.fract() == 0.0 && n.abs() < 9_007_199_254_740_992.0 {
        Value::from(n as i64)
    } else {
        Value::from(n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::libs::json_schema::JSON_SCHEMA_DECORATORS_TSP;
    use crate::parser::parse_with_libraries;

    fn check(source: &str) -> Checker {
        let parse_result = parse_with_libraries(source, vec![JSON_SCHEMA_DECORATORS_TSP.into()]);
        let mut checker = Checker::new();
        checker.set_parse_result(parse_result.root_id, parse_result.builder.clone());
        checker.check_program();
        let diagnostics: Vec<_> = checker.diagnostics().iter().map(|d| &d.message).collect();
        assert!(diagnostics.is_empty(), "diagnostics: {diagnostics:?}");
        checker
    }

    fn defs(source: &str) -> Value {
        JsonSchemaEmitter::new().emit_value(&check(source))["$defs"].clone()
    }

    /// Compare with the official emitter's output, keyword order included.
    fn assert_matches_reference(name: &str, source: &str, reference: &str) {
        let expected: Value = serde_json::from_str(reference).expect("reference is JSON");
        let actual = JsonSchemaEmitter::new()
            .bundle_id(&format!("{name}.json"))
            .emit_value(&check(source));
        let pretty = |v: &Value| serde_json::to_string_pretty(v).expect("serializable");
        assert_eq!(pretty(&actual), pretty(&expected), "fixture {name}");
    }

    macro_rules! reference_test {
        ($test:ident, $name:literal) => {
            #[test]
            fn $test() {
                assert_matches_reference(
                    $name,
                    include_str!(concat!("../../tests/json-schema/", $name, ".tsp")),
                    include_str!(concat!("../../tests/json-schema/", $name, ".json")),
                );
            }
        };
    }

    reference_test!(test_reference_models, "models");
    reference_test!(test_reference_scalars, "scalars");
    reference_test!(test_reference_constraints, "constraints");
    reference_test!(test_reference_unions, "unions");
    reference_test!(test_reference_extensions, "extensions");

    #[test]
    fn test_optional_property_not_required() {
        let d = defs("model M { a: string; b?: string; }");
        assert_eq!(d["M"]["required"], json!(["a"]));
        assert_eq!(d["M"]["properties"]["b"], json!({ "type": "string" }));
    }

    #[test]
    fn test_all_optional_has_no_required() {
        let d = defs("model M { a?: string; }");
        assert!(d["M"].get("required").is_none());
    }

    #[test]
    fn test_named_model_is_ref() {
        let d = defs("model A { b: B; } model B { x: string; }");
        assert_eq!(d["A"]["properties"]["b"], json!({ "$ref": "B.json" }));
        assert_eq!(d["B"]["$id"], json!("B.json"));
    }

    #[test]
    fn test_nullable() {
        let d = defs("model M { a: string | null; }");
        assert_eq!(
            d["M"]["properties"]["a"],
            json!({ "anyOf": [{ "type": "string" }, { "type": "null" }] })
        );
    }

    #[test]
    fn test_self_reference_terminates() {
        let d = defs("model Node { next: Node | null; children: Node[]; }");
        assert_eq!(
            d["Node"]["properties"]["children"]["items"],
            json!({ "$ref": "Node.json" })
        );
    }

    #[test]
    fn test_int_widths_have_bounds() {
        let d = defs("model M { a: int8; b: uint32; }");
        assert_eq!(
            d["M"]["properties"]["a"],
            json!({ "type": "integer", "minimum": -128, "maximum": 127 })
        );
        assert_eq!(d["M"]["properties"]["b"]["maximum"], json!(4294967295u32));
    }

    #[test]
    fn test_int64_is_string() {
        let d = defs("model M { a: int64; b: uint64; }");
        assert_eq!(d["M"]["properties"]["a"], json!({ "type": "string" }));
        assert_eq!(d["M"]["properties"]["b"], json!({ "type": "string" }));
    }

    #[test]
    fn test_width_format_off_by_default() {
        let d = defs("model M { a: int8; }");
        assert!(d["M"]["properties"]["a"].get("format").is_none());
    }

    #[test]
    fn test_width_format_on() {
        let checker = check("model M { a: int8; b: float32; c: int64; d: integer; }");
        let v = JsonSchemaEmitter::new()
            .width_format(true)
            .emit_value(&checker);
        let props = &v["$defs"]["M"]["properties"];
        assert_eq!(
            props["a"],
            json!({ "type": "integer", "minimum": -128, "maximum": 127, "format": "int8" })
        );
        assert_eq!(props["b"], json!({ "type": "number", "format": "float32" }));
        assert_eq!(props["c"], json!({ "type": "string", "format": "int64" }));
        assert_eq!(props["d"], json!({ "type": "integer" }));
    }

    #[test]
    fn test_width_format_yields_to_format_decorator() {
        let checker = check(r#"model M { @format("x") a: int16; }"#);
        let v = JsonSchemaEmitter::new()
            .width_format(true)
            .emit_value(&checker);
        assert_eq!(v["$defs"]["M"]["properties"]["a"]["format"], json!("x"));
    }

    #[test]
    fn test_decorator_bounds_replace_width_bounds() {
        let d = defs("model M { @minValue(1) @maxValue(5) a: int8; }");
        assert_eq!(
            d["M"]["properties"]["a"],
            json!({ "type": "integer", "minimum": 1, "maximum": 5 })
        );
    }

    #[test]
    fn test_extension_on_model() {
        let d = defs(
            r#"using TypeSpec.JsonSchema;
            @extension("x-a", "b") model M { }"#,
        );
        assert_eq!(d["M"]["x-a"], json!("b"));
    }

    #[test]
    fn test_extension_type_value_is_schema() {
        let d = defs(
            r#"using TypeSpec.JsonSchema;
            model M { @extension("x-of", Other) a: string; }
            model Other { }"#,
        );
        assert_eq!(
            d["M"]["properties"]["a"]["x-of"],
            json!({ "$ref": "Other.json" })
        );
    }

    #[test]
    fn test_template_declaration_skipped() {
        let d = defs("model Page<T> { items: T[]; }");
        assert!(d.get("Page").is_none());
    }

    #[test]
    fn test_namespaced_types() {
        let d = defs("namespace Demo { namespace Inner { model B {} } model A { b: Inner.B; } }");
        assert_eq!(d["A"]["properties"]["b"], json!({ "$ref": "B.json" }));
        let keys: Vec<_> = d.as_object().expect("object").keys().collect();
        assert_eq!(keys, ["B", "A"]);
    }

    #[test]
    fn test_bundle_root() {
        let v = JsonSchemaEmitter::new()
            .bundle_id("x.json")
            .emit_value(&check("model M {}"));
        assert_eq!(v["$schema"], json!(DIALECT));
        assert_eq!(v["$id"], json!("x.json"));
        let v = JsonSchemaEmitter::new().emit_value(&check("model M {}"));
        assert!(v.get("$id").is_none());
    }

    #[test]
    fn test_emit_from_checker() {
        let result = JsonSchemaEmitter::new()
            .emit_from_checker(&check("model M { a: string; }"))
            .expect("emits");
        assert_eq!(result.format, "json-schema");
        assert!(!result.has_errors());
        let v: Value = serde_json::from_str(&result.output).expect("JSON");
        assert_eq!(v["$defs"]["M"]["type"], json!("object"));
    }
}
