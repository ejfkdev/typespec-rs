//! JSON Emitter
//!
//! Emits TypeSpec to OpenAPI JSON format.

use super::emitter::{get_name, get_type_string};
use super::{EmitResult, Emitter};
use crate::ast::types::*;
use crate::parser::{AstBuilder, AstNode, ParseResult};

/// JSON Emitter implementation
pub struct JsonEmitter {
    compact: bool,
}

impl JsonEmitter {
    pub fn new() -> Self {
        Self { compact: false }
    }

    pub fn compact(mut self) -> Self {
        self.compact = true;
        self
    }
}

impl Default for JsonEmitter {
    fn default() -> Self {
        Self::new()
    }
}

impl Emitter for JsonEmitter {
    fn emit(&self, result: &ParseResult) -> Result<EmitResult, String> {
        let mut json = String::new();
        let indent = if self.compact { 0 } else { 2 };
        let sep = if self.compact { "" } else { "\n" };
        let indent_str = "  ".repeat(indent);

        // Start JSON object
        json.push('{');
        json.push_str(sep);

        // Source info
        json.push_str(&format!("{}  \"source\": \"TypeSpec\",{}", indent_str, sep));
        json.push_str(&format!("{}  \"format\": \"json\",{}", indent_str, sep));
        json.push_str(&format!("{}  \"types\": {{{}", indent_str, sep));

        // Collect and emit declarations
        let mut first_type = true;

        type ModelInfo = (String, Vec<(String, String, bool)>);
        type InterfaceInfo = (String, Vec<(String, String)>);
        type EnumInfo = (String, Vec<String>);

        let mut models: Vec<ModelInfo> = Vec::new();
        let mut interfaces: Vec<InterfaceInfo> = Vec::new();
        let mut enums: Vec<EnumInfo> = Vec::new();
        let mut aliases: Vec<String> = Vec::new();

        for node in result.builder.nodes.values() {
            match node {
                AstNode::ModelDeclaration(m) => {
                    let name = get_name(&result.builder, m.name);
                    let props = get_model_properties(&result.builder, m);
                    models.push((name, props));
                }
                AstNode::InterfaceDeclaration(i) => {
                    let name = get_name(&result.builder, i.name);
                    let ops = get_interface_operations(&result.builder, i);
                    interfaces.push((name, ops));
                }
                AstNode::EnumDeclaration(e) => {
                    let name = get_name(&result.builder, e.name);
                    let members = get_enum_members(&result.builder, e);
                    enums.push((name, members));
                }
                AstNode::AliasStatement(a) => {
                    aliases.push(get_name(&result.builder, a.name));
                }
                _ => {}
            }
        }

        // Emit models
        let mut first_model = true;
        for (name, props) in &models {
            if !first_model {
                json.push(',');
                json.push_str(sep);
            }
            json.push_str(&format!("{}    \"{}\": {{{}", indent_str, name, sep));
            json.push_str(&format!("{}      \"type\": \"object\",{}", indent_str, sep));
            json.push_str(&format!("{}      \"properties\": {{{}", indent_str, sep));

            let mut first_prop = true;
            for (prop_name, prop_type, optional) in props {
                if !first_prop {
                    json.push(',');
                }
                let opt_marker = if *optional { "?" } else { "" };
                json.push_str(&format!(
                    "{}        \"{}{}\": \"{}\"",
                    indent_str, prop_name, opt_marker, prop_type
                ));
                first_prop = false;
            }

            json.push_str(sep);
            json.push_str(&format!("{}      }}", indent_str));
            json.push_str(sep);
            json.push_str(&format!("{}    }}", indent_str));
            first_model = false;
            first_type = false;
        }

        // Emit interfaces
        for (name, ops) in &interfaces {
            if !first_type {
                json.push(',');
                json.push_str(sep);
            }
            json.push_str(&format!("{}    \"{}\": {{{}", indent_str, name, sep));
            json.push_str(&format!(
                "{}      \"type\": \"interface\",{}",
                indent_str, sep
            ));
            json.push_str(&format!("{}      \"operations\": {{{}", indent_str, sep));

            let mut first_op = true;
            for (op_name, return_type) in ops {
                if !first_op {
                    json.push(',');
                }
                json.push_str(&format!(
                    "{}        \"{}\": \"{}\"",
                    indent_str, op_name, return_type
                ));
                first_op = false;
            }

            json.push_str(sep);
            json.push_str(&format!("{}      }}", indent_str));
            json.push_str(sep);
            json.push_str(&format!("{}    }}", indent_str));
            first_type = false;
        }

        // Emit enums
        for (name, members) in &enums {
            if !first_type {
                json.push(',');
                json.push_str(sep);
            }
            json.push_str(&format!("{}    \"{}\": {{{}", indent_str, name, sep));
            json.push_str(&format!("{}      \"type\": \"enum\",{}", indent_str, sep));
            json.push_str(&format!(
                "{}      \"values\": [{}]",
                indent_str,
                members.join(", ")
            ));
            json.push_str(sep);
            json.push_str(&format!("{}    }}", indent_str));
            first_type = false;
        }

        // Emit aliases
        if !aliases.is_empty() {
            if !first_type {
                json.push(',');
                json.push_str(sep);
            }
            json.push_str(&format!(
                "{}    \"_aliases\": [{}]",
                indent_str,
                aliases
                    .iter()
                    .map(|s| format!("\"{}\"", s))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        json.push_str(sep);
        json.push_str(&format!("{}  }}", indent_str));
        json.push_str(sep);
        json.push('}');

        Ok(EmitResult::new(json, "json", result.diagnostics.len()))
    }

    fn format(&self) -> &str {
        "json"
    }
}

fn get_model_properties(
    builder: &AstBuilder,
    model: &ModelDeclaration,
) -> Vec<(String, String, bool)> {
    model
        .properties
        .iter()
        .filter_map(|&prop_id| {
            if let Some(AstNode::ModelProperty(prop)) = builder.nodes.get(&prop_id) {
                let name = get_name(builder, prop.name);
                let type_str = get_type_string(builder, prop.value);
                Some((name, type_str, prop.optional))
            } else {
                None
            }
        })
        .collect()
}

fn get_interface_operations(
    builder: &AstBuilder,
    interface: &InterfaceDeclaration,
) -> Vec<(String, String)> {
    interface
        .operations
        .iter()
        .filter_map(|&op_id| {
            if let Some(AstNode::OperationDeclaration(op)) = builder.nodes.get(&op_id) {
                let name = get_name(builder, op.name);
                let return_type = if let Some(AstNode::OperationSignatureDeclaration(sig)) =
                    builder.nodes.get(&op.signature)
                {
                    get_type_string(builder, sig.return_type)
                } else {
                    "void".to_string()
                };
                Some((name, return_type))
            } else {
                None
            }
        })
        .collect()
}

fn get_enum_members(builder: &AstBuilder, enum_decl: &EnumDeclaration) -> Vec<String> {
    enum_decl
        .members
        .iter()
        .filter_map(|&member_id| {
            if let Some(AstNode::EnumMember(member)) = builder.nodes.get(&member_id) {
                Some(get_name(builder, member.name))
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    #[test]
    fn test_json_emitter_model() {
        let src = "model Pet { name: string; }";
        let result = parse(src);
        let emitter = JsonEmitter::new();
        let output = emitter.emit(&result).unwrap();
        assert!(output.output.contains("Pet"));
        assert!(output.output.contains("name"));
        assert_eq!(output.format, "json");
    }

    #[test]
    fn test_json_emitter_enum() {
        let src = "enum Color { red, green, blue }";
        let result = parse(src);
        let emitter = JsonEmitter::new();
        let output = emitter.emit(&result).unwrap();
        assert!(output.output.contains("Color"));
        assert_eq!(output.format, "json");
    }

    #[test]
    fn test_json_emitter_compact() {
        let src = "model Foo { x: string; }";
        let result = parse(src);
        let emitter = JsonEmitter::new().compact();
        let output = emitter.emit(&result).unwrap();
        assert!(!output.output.contains("\n"));
    }

    #[test]
    fn test_json_emitter_default() {
        let emitter = JsonEmitter::default();
        let src = "model X {}";
        let result = parse(src);
        let output = emitter.emit(&result).unwrap();
        assert_eq!(output.format, "json");
    }
}
