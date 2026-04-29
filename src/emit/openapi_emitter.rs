//! OpenAPI Emitter
//!
//! Emits TypeSpec to OpenAPI 3.x format based on the Checker's type graph.

use super::{EmitResult, Emitter};
use crate::checker::Checker;
use crate::checker::types::*;
use crate::parser::ParseResult;

/// OpenAPI Emitter implementation
pub struct OpenAPIEmitter {
    version: String,
}

impl OpenAPIEmitter {
    pub fn new() -> Self {
        Self {
            version: "3.0.0".to_string(),
        }
    }

    pub fn version(mut self, version: &str) -> Self {
        self.version = version.to_string();
        self
    }

    /// Emit OpenAPI from the Checker's type graph (preferred path)
    pub fn emit_from_checker(&self, checker: &Checker) -> Result<EmitResult, String> {
        let mut out = String::new();
        let i = Indent::new();

        // Header
        out.push_str(&format!("{}openapi: {}\n", i, self.version));
        out.push_str(&format!("{}info:\n", i));
        out.push_str(&format!("{}  title: TypeSpec API\n", i));
        out.push_str(&format!("{}  version: 0.1.0\n", i));

        // Collect schemas, paths, and parameters from the type graph
        let mut schemas = Vec::new();
        let mut paths = Vec::new();

        // Walk the global namespace
        if let Some(gns_id) = checker.global_namespace_type
            && let Some(Type::Namespace(ns)) = checker.get_type(gns_id)
        {
            // Models -> schemas
            for name in &ns.model_names {
                if let Some(&model_id) = ns.models.get(name)
                    && let Some(Type::Model(m)) = checker.get_type(model_id)
                {
                    // Skip template instances (they are inline)
                    if m.template_node.is_some() && m.template_mapper.is_some() {
                        continue;
                    }
                    // Skip array-like models (have indexer), they are rendered inline
                    if m.indexer.is_some() {
                        continue;
                    }
                    schemas.push(self.emit_model_schema(checker, m));
                }
            }

            // Enums -> schemas
            for name in &ns.enum_names {
                if let Some(&enum_id) = ns.enums.get(name)
                    && let Some(Type::Enum(e)) = checker.get_type(enum_id)
                {
                    schemas.push(self.emit_enum_schema(e));
                }
            }

            // Unions -> schemas
            for name in &ns.union_names {
                if let Some(&union_id) = ns.unions.get(name)
                    && let Some(Type::Union(u)) = checker.get_type(union_id)
                    && !u.expression
                {
                    schemas.push(self.emit_union_schema(checker, u));
                }
            }

            // Interfaces -> paths
            for name in &ns.interface_names {
                if let Some(&iface_id) = ns.interfaces.get(name)
                    && let Some(Type::Interface(iface)) = checker.get_type(iface_id)
                {
                    paths.push(self.emit_interface_paths(checker, iface));
                }
            }

            // Standalone operations -> paths
            for name in &ns.operation_names {
                if let Some(&op_id) = ns.operations.get(name)
                    && let Some(Type::Operation(op)) = checker.get_type(op_id)
                    && op.interface_.is_none()
                {
                    paths.push(self.emit_operation_path(checker, op));
                }
            }
        }

        // Paths section
        out.push_str(&format!("{}paths:\n", i));
        if paths.is_empty() {
            out.push_str(&format!("{}  {}\n", i, "{}"));
        } else {
            for path in &paths {
                out.push_str(path);
            }
        }

        // Components/schemas section
        out.push_str(&format!("{}components:\n", i));
        out.push_str(&format!("{}  schemas:\n", i));
        if schemas.is_empty() {
            out.push_str(&format!("{}    {}\n", i, "{}"));
        } else {
            for schema in &schemas {
                out.push_str(schema);
            }
        }

        Ok(EmitResult::new(out, "openapi", checker.diagnostics().len()))
    }

    fn emit_model_schema(&self, checker: &Checker, m: &ModelType) -> String {
        let mut s = String::new();
        s.push_str(&format!("    {}:\n", m.name));
        s.push_str("      type: object\n");

        // Required properties
        let required: Vec<&str> = m
            .property_names
            .iter()
            .filter(|name| {
                m.properties
                    .get(*name)
                    .and_then(|&pid| checker.get_type(pid))
                    .map(|t| {
                        if let Type::ModelProperty(p) = t {
                            !p.optional
                        } else {
                            true
                        }
                    })
                    .unwrap_or(true)
            })
            .map(|s| s.as_str())
            .collect();

        if !required.is_empty() {
            s.push_str("      required:\n");
            for r in &required {
                s.push_str(&format!("        - {}\n", r));
            }
        }

        s.push_str("      properties:\n");

        for prop_name in &m.property_names {
            if let Some(&prop_id) = m.properties.get(prop_name)
                && let Some(Type::ModelProperty(prop)) = checker.get_type(prop_id)
            {
                s.push_str(&format!("        {}:\n", prop_name));
                s.push_str(&self.type_to_schema(checker, prop.r#type, 10));
            }
        }

        s
    }

    fn emit_enum_schema(&self, e: &EnumType) -> String {
        let mut s = String::new();
        s.push_str(&format!("    {}:\n", e.name));
        s.push_str("      type: string\n");
        s.push_str("      enum:\n");
        for member_name in &e.member_names {
            s.push_str(&format!("        - {}\n", member_name));
        }
        s
    }

    fn emit_union_schema(&self, checker: &Checker, u: &UnionType) -> String {
        let mut s = String::new();
        s.push_str(&format!("    {}:\n", u.name));
        s.push_str("      oneOf:\n");
        for variant_name in &u.variant_names {
            if let Some(&variant_id) = u.variants.get(variant_name)
                && let Some(Type::UnionVariant(v)) = checker.get_type(variant_id)
            {
                s.push_str(&self.type_to_schema(checker, v.r#type, 8));
            }
        }
        s
    }

    fn emit_interface_paths(&self, checker: &Checker, iface: &InterfaceType) -> String {
        let mut s = String::new();
        for op_name in &iface.operation_names {
            if let Some(&op_id) = iface.operations.get(op_name)
                && let Some(Type::Operation(op)) = checker.get_type(op_id)
            {
                s.push_str(&self.emit_operation_path(checker, op));
            }
        }
        s
    }

    fn emit_operation_path(&self, checker: &Checker, op: &OperationType) -> String {
        let mut s = String::new();
        // Generate a path from the operation name
        let path = format!("/{}", to_kebab_case(&op.name));

        s.push_str(&format!("    {}:\n", path));
        s.push_str("      get:\n");
        s.push_str(&format!("        operationId: {}\n", op.name));
        s.push_str(&format!("        summary: {}\n", op.name));

        // Parameters from operation parameters model
        if let Some(params_id) = op.parameters
            && let Some(Type::Model(params_model)) = checker.get_type(params_id)
        {
            for param_name in &params_model.property_names {
                if let Some(&prop_id) = params_model.properties.get(param_name)
                    && let Some(Type::ModelProperty(prop)) = checker.get_type(prop_id)
                {
                    s.push_str("        parameters:\n");
                    s.push_str("          - name: ");
                    s.push_str(param_name);
                    s.push('\n');
                    s.push_str("            in: query\n");
                    if !prop.optional {
                        s.push_str("            required: true\n");
                    }
                    s.push_str("            schema:\n");
                    s.push_str(&self.type_to_schema(checker, prop.r#type, 14));
                }
            }
        }

        // Responses
        s.push_str("        responses:\n");
        s.push_str("          '200':\n");
        s.push_str("            description: Success\n");
        if let Some(ret_id) = op.return_type {
            s.push_str("            content:\n");
            s.push_str("              application/json:\n");
            s.push_str("                schema:\n");
            s.push_str(&self.type_to_schema(checker, ret_id, 18));
        }

        s
    }

    /// Convert a TypeId to an OpenAPI schema fragment
    fn type_to_schema(&self, checker: &Checker, type_id: TypeId, indent: usize) -> String {
        let pad = " ".repeat(indent);
        match checker.get_type(type_id) {
            None => format!("{}type: object\n", pad),
            Some(t) => match t {
                Type::Intrinsic(i) => match i.name {
                    IntrinsicTypeName::Void => format!("{}type: object\n", pad),
                    IntrinsicTypeName::Unknown => format!("{}type: object\n", pad),
                    IntrinsicTypeName::Null => format!("{}nullable: true\n", pad),
                    _ => format!("{}type: object\n", pad),
                },
                Type::String(_) => format!("{}type: string\n", pad),
                Type::Boolean(_) => format!("{}type: boolean\n", pad),
                Type::Number(n) => {
                    // Check if it's an integer
                    if n.value.fract() == 0.0 {
                        format!("{}type: integer\n", pad)
                    } else {
                        format!("{}type: number\n", pad)
                    }
                }
                Type::Model(m) => {
                    // Array-like model (has indexer) → array schema
                    if m.indexer.is_some() {
                        if let Some((_key, val)) = &m.indexer {
                            let mut s = format!("{}type: array\n", pad);
                            s.push_str(&format!("{}items:\n", pad));
                            s.push_str(&self.type_to_schema(checker, *val, indent + 2));
                            s
                        } else {
                            format!("{}type: object\n", pad)
                        }
                    } else if !m.name.is_empty() {
                        // Named model → $ref
                        format!("{}$ref: '#/components/schemas/{}'\n", pad, m.name)
                    } else {
                        // Anonymous model → inline object
                        let mut s = format!("{}type: object\n", pad);
                        s.push_str(&format!("{}properties:\n", pad));
                        for prop_name in &m.property_names {
                            if let Some(&prop_id) = m.properties.get(prop_name)
                                && let Some(Type::ModelProperty(prop)) = checker.get_type(prop_id)
                            {
                                s.push_str(&format!("{}  {}:\n", pad, prop_name));
                                s.push_str(&self.type_to_schema(checker, prop.r#type, indent + 4));
                            }
                        }
                        s
                    }
                }
                Type::ModelProperty(p) => self.type_to_schema(checker, p.r#type, indent),
                Type::Enum(e) => {
                    format!("{}$ref: '#/components/schemas/{}'\n", pad, e.name)
                }
                Type::Union(u) => {
                    if u.expression {
                        // Anonymous union expression (e.g., `Pet | Error`)
                        // Check if it's a nullable union (X | null)
                        let variant_types: Vec<TypeId> = u
                            .variant_names
                            .iter()
                            .filter_map(|name| u.variants.get(name).copied())
                            .filter_map(|vid| {
                                checker.get_type(vid).and_then(|t| {
                                    if let Type::UnionVariant(v) = t {
                                        Some(v.r#type)
                                    } else {
                                        None
                                    }
                                })
                            })
                            .collect();

                        let has_null = variant_types.iter().any(|&tid| {
                            checker.get_type(tid).is_some_and(|t| {
                                matches!(t, Type::Intrinsic(i) if i.name == IntrinsicTypeName::Null)
                            })
                        });

                        let non_null: Vec<TypeId> = variant_types
                            .iter()
                            .copied()
                            .filter(|&tid| {
                                !checker.get_type(tid).is_some_and(|t| {
                                    matches!(t, Type::Intrinsic(i) if i.name == IntrinsicTypeName::Null)
                                })
                            })
                            .collect();

                        if has_null && non_null.len() == 1 {
                            // Nullable type
                            let mut s = self.type_to_schema(checker, non_null[0], indent);
                            s.push_str(&format!("{}nullable: true\n", pad));
                            s
                        } else if non_null.len() > 1 {
                            // oneOf
                            let mut s = format!("{}oneOf:\n", pad);
                            for &tid in &non_null {
                                s.push_str(&self.type_to_schema(checker, tid, indent + 2));
                            }
                            if has_null {
                                s.push_str(&format!("{}nullable: true\n", pad));
                            }
                            s
                        } else {
                            format!("{}nullable: true\n", pad)
                        }
                    } else {
                        // Named union → $ref
                        format!("{}$ref: '#/components/schemas/{}'\n", pad, u.name)
                    }
                }
                Type::Scalar(s) => {
                    // Map well-known TypeSpec scalars to OpenAPI types
                    match s.name.as_str() {
                        "string" => format!("{}type: string\n", pad),
                        "int8" | "int16" | "int32" | "int64" => {
                            format!("{}type: integer\n{}format: {}\n", pad, pad, s.name)
                        }
                        "uint8" | "uint16" | "uint32" | "uint64" => {
                            format!("{}type: integer\n{}format: {}\n", pad, pad, s.name)
                        }
                        "float32" | "float64" | "float" => {
                            format!("{}type: number\n{}format: {}\n", pad, pad, s.name)
                        }
                        "boolean" => format!("{}type: boolean\n", pad),
                        "url" | "uri" => {
                            format!("{}type: string\n{}format: uri\n", pad, pad)
                        }
                        "bytes" => {
                            format!("{}type: string\n{}format: byte\n", pad, pad)
                        }
                        "unixTimestamp32" => {
                            format!("{}type: integer\n{}format: int64\n", pad, pad)
                        }
                        // Custom scalars → $ref if named, otherwise string
                        _ => {
                            if s.base_scalar.is_some() {
                                format!("{}$ref: '#/components/schemas/{}'\n", pad, s.name)
                            } else {
                                format!("{}type: string\n", pad)
                            }
                        }
                    }
                }
                Type::Tuple(t) => {
                    let mut s = format!("{}type: array\n", pad);
                    if t.values.len() == 1 {
                        s.push_str(&format!("{}items:\n", pad));
                        s.push_str(&self.type_to_schema(checker, t.values[0], indent + 2));
                    } else {
                        // prefixItems for OpenAPI 3.1, fallback to items for 3.0
                        s.push_str(&format!("{}items:\n", pad));
                        s.push_str(&format!("{}  oneOf:\n", pad));
                        for &val_id in &t.values {
                            s.push_str(&self.type_to_schema(checker, val_id, indent + 4));
                        }
                    }
                    s
                }
                Type::Interface(iface) => {
                    format!("{}$ref: '#/components/schemas/{}'\n", pad, iface.name)
                }
                Type::Operation(op) => {
                    format!("{}$ref: '#/components/schemas/{}'\n", pad, op.name)
                }
                Type::EnumMember(em) => {
                    if let Some(eid) = em.r#enum
                        && let Some(Type::Enum(e)) = checker.get_type(eid)
                    {
                        format!("{}$ref: '#/components/schemas/{}'\n", pad, e.name)
                    } else {
                        format!("{}type: string\n", pad)
                    }
                }
                Type::UnionVariant(v) => self.type_to_schema(checker, v.r#type, indent),
                _ => format!("{}type: object\n", pad),
            },
        }
    }
}

impl Default for OpenAPIEmitter {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple indent helper
struct Indent;

impl Indent {
    fn new() -> Self {
        Self
    }
}

impl std::fmt::Display for Indent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}

/// Convert CamelCase/PascalCase to kebab-case
fn to_kebab_case(s: &str) -> String {
    let mut result = String::new();
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() && i > 0 {
            result.push('-');
        }
        result.push(ch.to_ascii_lowercase());
    }
    result
}

/// Fallback: emit from AST (placeholder, delegates to checker-based path when possible)
impl Emitter for OpenAPIEmitter {
    fn emit(&self, result: &ParseResult) -> Result<EmitResult, String> {
        // AST-based fallback — run checker and use the type graph
        let mut checker = Checker::new();
        checker.set_parse_result(result.root_id, result.builder.clone());
        checker.check_program();
        self.emit_from_checker(&checker)
    }

    fn format(&self) -> &str {
        "openapi"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    fn check(source: &str) -> Checker {
        let parse_result = parse(source);
        let mut checker = Checker::new();
        checker.set_parse_result(parse_result.root_id, parse_result.builder.clone());
        checker.check_program();
        checker
    }

    #[test]
    fn test_openapi_emitter_model() {
        let checker = check("model Pet { name: string; id: int32; }");
        let emitter = OpenAPIEmitter::new();
        let output = emitter.emit_from_checker(&checker).unwrap();
        assert!(output.output.contains("Pet"));
        assert!(output.output.contains("name"));
        assert!(output.output.contains("type: string"));
        assert!(output.output.contains("type: integer"));
        assert!(output.output.contains("required"));
        assert_eq!(output.format, "openapi");
    }

    #[test]
    fn test_openapi_emitter_enum() {
        let checker = check("enum Status { available, pending, sold }");
        let emitter = OpenAPIEmitter::new();
        let output = emitter.emit_from_checker(&checker).unwrap();
        assert!(output.output.contains("Status"));
        assert!(output.output.contains("available"));
        assert!(output.output.contains("type: string"));
    }

    #[test]
    fn test_openapi_emitter_interface() {
        let checker =
            check("interface PetStore { listPets(): Pet[]; } model Pet { name: string; }");
        let emitter = OpenAPIEmitter::new();
        let output = emitter.emit_from_checker(&checker).unwrap();
        assert!(output.output.contains("paths"));
        assert!(output.output.contains("list-pets"));
        assert!(output.output.contains("operationId: listPets"));
    }

    #[test]
    fn test_openapi_emitter_optional() {
        let checker = check("model Pet { name: string; tag?: string; }");
        let emitter = OpenAPIEmitter::new();
        let output = emitter.emit_from_checker(&checker).unwrap();
        // name should be required, tag should not
        assert!(output.output.contains("required"));
        assert!(output.output.contains("name"));
        assert!(output.output.contains("tag"));
    }

    #[test]
    fn test_openapi_emitter_default() {
        let emitter = OpenAPIEmitter::default();
        let src = "model X {}";
        let result = parse(src);
        let output = emitter.emit(&result).unwrap();
        assert_eq!(output.format, "openapi");
    }

    #[test]
    fn test_openapi_emitter_custom_version() {
        let checker = check("model Foo {}");
        let emitter = OpenAPIEmitter::new().version("3.1.0");
        let output = emitter.emit_from_checker(&checker).unwrap();
        assert!(output.output.contains("3.1.0"));
    }

    #[test]
    fn test_openapi_emitter_union() {
        let checker = check(
            "union Shape { circle: Circle, rect: Rect } model Circle { r: float64; } model Rect { w: float64; h: float64; }",
        );
        let emitter = OpenAPIEmitter::new();
        let output = emitter.emit_from_checker(&checker).unwrap();
        assert!(output.output.contains("Shape"));
        assert!(output.output.contains("oneOf"));
    }

    #[test]
    fn test_openapi_pet_store() {
        let src = r#"
            model Pet {
              id: int32;
              name: string;
              tag?: string;
            }
            enum Status {
              available,
              pending,
              sold,
            }
            model Error {
              code: int32;
              message: string;
            }
            interface PetStore {
              listPets(limit?: int32): Pet[];
              getPet(petId: int32): Pet | Error;
            }
        "#;
        let checker = check(src);
        let emitter = OpenAPIEmitter::new();
        let output = emitter.emit_from_checker(&checker).unwrap();
        let yaml = &output.output;

        // Check schemas
        assert!(yaml.contains("Pet:"));
        assert!(yaml.contains("Status:"));
        assert!(yaml.contains("Error:"));
        assert!(yaml.contains("type: object"));
        assert!(yaml.contains("type: integer"));
        assert!(yaml.contains("type: string"));
        assert!(yaml.contains("format: int32"));

        // Check paths
        assert!(yaml.contains("paths:"));
        assert!(yaml.contains("operationId: listPets"));
        assert!(yaml.contains("operationId: getPet"));

        // Check array return type
        assert!(yaml.contains("type: array"));
        assert!(yaml.contains("$ref"));
    }
}
