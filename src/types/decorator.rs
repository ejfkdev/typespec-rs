//! Decorator types for TypeSpec-Rust
//! Ported from TypeSpec compiler/src/core/types.ts

use super::primitive::TypeKind;
use crate::ast::node::NodeId;
use std::collections::HashMap;

/// Decorator - represents a TypeSpec decorator declaration
/// Decorators are applied to types and other declarations using `@`
#[derive(Debug, Clone)]
pub struct Decorator {
    /// Node ID for this decorator
    pub id: NodeId,
    /// Source node ID
    pub node: Option<NodeId>,
    /// Name of the decorator (with @ prefix)
    pub name: String,
    /// Namespace containing this decorator
    pub namespace: NodeId,
    /// Target parameter (first parameter of decorator function)
    pub target: NodeId,
    /// Additional parameters
    pub parameters: Vec<NodeId>,
    /// Whether this type has been finished (decorators called)
    pub is_finished: bool,
}

impl Decorator {
    pub fn new(id: NodeId, name: String, namespace: NodeId) -> Self {
        Self {
            id,
            node: None,
            name,
            namespace,
            target: NodeId::MAX, // Placeholder until AST is available
            parameters: Vec::new(),
            is_finished: false,
        }
    }

    pub fn kind(&self) -> TypeKind {
        TypeKind::Decorator
    }
}

/// DecoratorArgumentValue - possible values for decorator arguments
#[derive(Debug, Clone)]
pub enum DecoratorArgumentValue {
    Type(NodeId),
    Number(f64),
    String(String),
    Boolean(bool),
}

/// DecoratorArgument - a single argument passed to a decorator
#[derive(Debug, Clone)]
pub struct DecoratorArgument {
    /// The value (as a type or literal)
    pub value: NodeId,
    /// The JS-marshalled value for use in JavaScript interop
    pub js_value: Option<DecoratorMarshalledValue>,
    /// Node where this argument appears
    pub node: Option<NodeId>,
}

/// DecoratorMarshalledValue - JS-marshalled values
#[derive(Debug, Clone)]
pub enum DecoratorMarshalledValue {
    Type(NodeId),
    Value(NodeId),
    Record(HashMap<String, NodeId>),
    Array(Vec<NodeId>),
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
}

/// DecoratorApplication - a decorator applied to a declaration
#[derive(Debug, Clone)]
pub struct DecoratorApplication {
    /// The decorator definition
    pub definition: Option<NodeId>,
    /// The decorator node ID
    pub decorator: NodeId,
    /// Arguments to the decorator
    pub args: Vec<DecoratorArgument>,
    /// The node where this decorator was applied
    pub node: Option<NodeId>,
}

impl DecoratorApplication {
    pub fn new(decorator: NodeId) -> Self {
        Self {
            definition: None,
            decorator,
            args: Vec::new(),
            node: None,
        }
    }

    pub fn with_args(mut self, args: Vec<DecoratorArgument>) -> Self {
        self.args = args;
        self
    }

    pub fn with_definition(mut self, definition: NodeId) -> Self {
        self.definition = Some(definition);
        self
    }

    pub fn with_node(mut self, node: NodeId) -> Self {
        self.node = Some(node);
        self
    }
}
