//! Operation types for TypeSpec-Rust
//! Ported from TypeSpec compiler/src/core/types.ts

use super::decorator::DecoratorApplication;
use super::primitive::TypeKind;
use crate::ast::node::NodeId;

/// Operation - represents a TypeSpec operation
/// Operations are the functional equivalent of methods in interfaces
#[derive(Debug, Clone)]
pub struct Operation {
    /// Node ID for this operation
    pub id: NodeId,
    /// Source node ID
    pub node: Option<NodeId>,
    /// Name of the operation
    pub name: String,
    /// Namespace containing this operation
    pub namespace: Option<NodeId>,
    /// Interface containing this operation (if part of an interface)
    pub interface: Option<NodeId>,
    /// Parameters of the operation (as a Model)
    pub parameters: NodeId,
    /// Return type of the operation
    pub return_type: NodeId,
    /// Template mapper if this is a template instantiation
    pub template_mapper: Option<NodeId>,
    /// Template node if this is a template declaration
    pub template_node: Option<NodeId>,
    /// Source operation if this is `op is` reference
    pub source_operation: Option<NodeId>,
    /// Decorators applied to this operation
    pub decorators: Vec<DecoratorApplication>,
    /// Whether this type has been finished (decorators called)
    pub is_finished: bool,
}

impl Operation {
    pub fn new(id: NodeId, name: String, parameters: NodeId, return_type: NodeId) -> Self {
        Self {
            id,
            node: None,
            name,
            namespace: None,
            interface: None,
            parameters,
            return_type,
            template_mapper: None,
            template_node: None,
            source_operation: None,
            decorators: Vec::new(),
            is_finished: false,
        }
    }

    pub fn kind(&self) -> TypeKind {
        TypeKind::Operation
    }
}

/// OperationSignature - represents the signature of an operation
#[derive(Debug, Clone)]
pub enum OperationSignature {
    /// Inline signature declaration
    Declaration {
        /// Parameters as a Model node
        parameters: NodeId,
        /// Return type node
        return_type: NodeId,
    },
    /// Reference to another operation via `op is`
    Reference {
        /// The base operation being referenced
        base_operation: NodeId,
    },
}
