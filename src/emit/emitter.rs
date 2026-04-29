//! Emitter trait definition
//!
//! Defines the interface for emitting TypeSpec to various formats.

use crate::emit::EmitResult;
use crate::parser::{AstBuilder, AstNode, ParseResult};

/// Emitter trait for converting parsed TypeSpec to output formats
pub trait Emitter: Send + Sync {
    /// Emit the parsed TypeSpec to the target format
    fn emit(&self, result: &ParseResult) -> Result<EmitResult, String>;

    /// Get the format name (e.g., "yaml", "json", "markdown")
    fn format(&self) -> &str;
}

/// Get the identifier name from an AST node ID
pub(crate) fn get_name(builder: &AstBuilder, node_id: u32) -> String {
    if let Some(AstNode::Identifier(id)) = builder.nodes.get(&node_id) {
        id.value.clone()
    } else {
        "<unknown>".to_string()
    }
}

/// Get a type string representation from an AST node ID
pub(crate) fn get_type_string(builder: &AstBuilder, node_id: u32) -> String {
    match builder.nodes.get(&node_id) {
        Some(AstNode::Identifier(id)) => id.value.clone(),
        Some(AstNode::ArrayExpression(arr)) => {
            format!("{}[]", get_type_string(builder, arr.element_type))
        }
        Some(AstNode::TypeReference(tr)) => get_name(builder, tr.name),
        Some(AstNode::UnionExpression(u)) => {
            let opts: Vec<String> = u
                .options
                .iter()
                .map(|&opt| get_type_string(builder, opt))
                .collect();
            opts.join(" | ")
        }
        Some(AstNode::StringLiteral(s)) => format!("\"{}\"", s.value),
        Some(AstNode::NumericLiteral(n)) => n.value_as_string.clone(),
        Some(AstNode::BooleanLiteral(b)) => b.value.to_string(),
        _ => "<type>".to_string(),
    }
}
