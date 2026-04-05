//! Namespace types for TypeSpec-Rust
//! Ported from TypeSpec compiler/src/core/types.ts

use super::decorator::DecoratorApplication;
use super::primitive::TypeKind;
use crate::ast::node::NodeId;
use std::collections::HashMap;

/// Namespace - represents a TypeSpec namespace
/// Namespaces contain declarations and can be nested
#[derive(Debug, Clone)]
pub struct Namespace {
    /// Node ID for this namespace
    pub id: NodeId,
    /// Name of the namespace
    pub name: String,
    /// Parent namespace (if nested)
    pub namespace: Option<NodeId>,
    /// Source node ID
    pub node: Option<NodeId>,
    /// Models defined in this namespace
    pub models: HashMap<String, NodeId>,
    /// Scalars defined in this namespace
    pub scalars: HashMap<String, NodeId>,
    /// Operations defined in this namespace
    pub operations: HashMap<String, NodeId>,
    /// Sub-namespaces
    pub namespaces: HashMap<String, NodeId>,
    /// Interfaces defined in this namespace
    pub interfaces: HashMap<String, NodeId>,
    /// Enums defined in this namespace
    pub enums: HashMap<String, NodeId>,
    /// Unions defined in this namespace
    pub unions: HashMap<String, NodeId>,
    /// Decorator declarations in this namespace
    pub decorator_declarations: HashMap<String, NodeId>,
    /// Function declarations in this namespace
    pub function_declarations: HashMap<String, NodeId>,
    /// Decorators applied to this namespace
    pub decorators: Vec<DecoratorApplication>,
    /// Whether this type has been finished (decorators called)
    pub is_finished: bool,
}

impl Namespace {
    pub fn new(id: NodeId, name: String) -> Self {
        Self {
            id,
            name,
            namespace: None,
            node: None,
            models: HashMap::new(),
            scalars: HashMap::new(),
            operations: HashMap::new(),
            namespaces: HashMap::new(),
            interfaces: HashMap::new(),
            enums: HashMap::new(),
            unions: HashMap::new(),
            decorator_declarations: HashMap::new(),
            function_declarations: HashMap::new(),
            decorators: Vec::new(),
            is_finished: false,
        }
    }

    pub fn kind(&self) -> TypeKind {
        TypeKind::Namespace
    }

    // Helper methods to add various types to the namespace
    pub fn add_model(&mut self, name: String, model_id: NodeId) {
        self.models.insert(name, model_id);
    }

    pub fn add_scalar(&mut self, name: String, scalar_id: NodeId) {
        self.scalars.insert(name, scalar_id);
    }

    pub fn add_operation(&mut self, name: String, operation_id: NodeId) {
        self.operations.insert(name, operation_id);
    }

    pub fn add_namespace(&mut self, name: String, namespace_id: NodeId) {
        self.namespaces.insert(name, namespace_id);
    }

    pub fn add_interface(&mut self, name: String, interface_id: NodeId) {
        self.interfaces.insert(name, interface_id);
    }

    pub fn add_enum(&mut self, name: String, enum_id: NodeId) {
        self.enums.insert(name, enum_id);
    }

    pub fn add_union(&mut self, name: String, union_id: NodeId) {
        self.unions.insert(name, union_id);
    }

    pub fn add_decorator_declaration(&mut self, name: String, decorator_id: NodeId) {
        self.decorator_declarations.insert(name, decorator_id);
    }

    pub fn add_function_declaration(&mut self, name: String, function_id: NodeId) {
        self.function_declarations.insert(name, function_id);
    }
}
