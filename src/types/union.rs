//! Union types for TypeSpec-Rust
//! Ported from TypeSpec compiler/src/core/types.ts

use super::decorator::DecoratorApplication;
use super::primitive::TypeKind;
use crate::ast::node::NodeId;
use std::collections::HashMap;

/// Union - represents a TypeSpec union type
#[derive(Debug, Clone)]
pub struct Union {
    /// Node ID for this union
    pub id: NodeId,
    /// Name (optional - anonymous unions may not have names)
    pub name: Option<String>,
    /// Source node ID
    pub node: Option<NodeId>,
    /// Namespace containing this union
    pub namespace: Option<NodeId>,
    /// Variants of the union (ordered as they appear in source)
    pub variants: HashMap<String, NodeId>,
    /// Whether this is an expression (inline union) vs statement
    pub expression: bool,
    /// Template mapper if this is a template instantiation
    pub template_mapper: Option<NodeId>,
    /// Template node if this is a template declaration
    pub template_node: Option<NodeId>,
    /// Decorators applied to this union
    pub decorators: Vec<DecoratorApplication>,
    /// Whether this type has been finished (decorators called)
    pub is_finished: bool,
    /// Symbol ID for late-bound symbols
    pub symbol: Option<NodeId>,
}

impl Union {
    pub fn new(id: NodeId) -> Self {
        Self {
            id,
            name: None,
            node: None,
            namespace: None,
            variants: HashMap::new(),
            expression: false,
            template_mapper: None,
            template_node: None,
            decorators: Vec::new(),
            is_finished: false,
            symbol: None,
        }
    }

    pub fn with_name(id: NodeId, name: String) -> Self {
        Self {
            id,
            name: Some(name),
            node: None,
            namespace: None,
            variants: HashMap::new(),
            expression: false,
            template_mapper: None,
            template_node: None,
            decorators: Vec::new(),
            is_finished: false,
            symbol: None,
        }
    }

    pub fn kind(&self) -> TypeKind {
        TypeKind::Union
    }

    pub fn add_variant(&mut self, key: String, variant_id: NodeId) {
        self.variants.insert(key, variant_id);
    }
}

/// UnionVariant - represents a variant in a union
#[derive(Debug, Clone)]
pub struct UnionVariant {
    /// Node ID for this variant
    pub id: NodeId,
    /// Source node ID
    pub node: Option<NodeId>,
    /// Name of the variant (can be anonymous)
    pub name: Option<String>,
    /// The type of this variant
    pub r#type: NodeId,
    /// The union containing this variant
    pub union_id: Option<NodeId>,
    /// Decorators applied to this variant
    pub decorators: Vec<DecoratorApplication>,
    /// Whether this type has been finished (decorators called)
    pub is_finished: bool,
}

impl UnionVariant {
    pub fn new(id: NodeId, variant_type: NodeId) -> Self {
        Self {
            id,
            node: None,
            name: None,
            r#type: variant_type,
            union_id: None,
            decorators: Vec::new(),
            is_finished: false,
        }
    }

    pub fn with_name(id: NodeId, name: String, variant_type: NodeId) -> Self {
        Self {
            id,
            node: None,
            name: Some(name),
            r#type: variant_type,
            union_id: None,
            decorators: Vec::new(),
            is_finished: false,
        }
    }

    pub fn kind(&self) -> TypeKind {
        TypeKind::UnionVariant
    }
}
