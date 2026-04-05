//! Interface types for TypeSpec-Rust
//! Ported from TypeSpec compiler/src/core/types.ts

use super::decorator::DecoratorApplication;
use super::primitive::TypeKind;
use crate::ast::node::NodeId;
use std::collections::{HashMap, HashSet};

/// Interface - represents a TypeSpec interface type
/// Contains operations and can extend other interfaces
#[derive(Debug, Clone)]
pub struct Interface {
    /// Node ID for this interface
    pub id: NodeId,
    /// Name of the interface
    pub name: String,
    /// Source node ID
    pub node: Option<NodeId>,
    /// Namespace containing this interface
    pub namespace: Option<NodeId>,
    /// Source interfaces extended by this interface
    pub source_interfaces: Vec<NodeId>,
    /// Operations defined in this interface (ordered as they appear in source)
    pub operations: HashMap<String, NodeId>,
    /// Template mapper if this is a template instantiation
    pub template_mapper: Option<NodeId>,
    /// Template node if this is a template declaration
    pub template_node: Option<NodeId>,
    /// Decorators applied to this interface
    pub decorators: Vec<DecoratorApplication>,
    /// Whether this type has been finished (decorators called)
    pub is_finished: bool,
    /// Symbol ID for late-bound symbols
    pub symbol: Option<NodeId>,
}

impl Interface {
    pub fn new(id: NodeId, name: String) -> Self {
        Self {
            id,
            name,
            node: None,
            namespace: None,
            source_interfaces: Vec::new(),
            operations: HashMap::new(),
            template_mapper: None,
            template_node: None,
            decorators: Vec::new(),
            is_finished: false,
            symbol: None,
        }
    }

    pub fn kind(&self) -> TypeKind {
        TypeKind::Interface
    }

    pub fn add_operation(&mut self, name: String, operation_id: NodeId) {
        self.operations.insert(name, operation_id);
    }

    pub fn add_source_interface(&mut self, interface_id: NodeId) {
        self.source_interfaces.push(interface_id);
    }
}
