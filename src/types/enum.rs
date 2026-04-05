//! Enum types for TypeSpec-Rust
//! Ported from TypeSpec compiler/src/core/types.ts

use super::decorator::DecoratorApplication;
use super::primitive::TypeKind;
use crate::ast::node::NodeId;
use std::collections::HashMap;

/// Enum - represents a TypeSpec enum type
#[derive(Debug, Clone)]
pub struct Enum {
    /// Node ID for this enum
    pub id: NodeId,
    /// Name of the enum
    pub name: String,
    /// Source node ID
    pub node: Option<NodeId>,
    /// Namespace containing this enum
    pub namespace: Option<NodeId>,
    /// Members of the enum (ordered as they appear in source)
    pub members: HashMap<String, EnumMember>,
    /// Decorators applied to this enum
    pub decorators: Vec<DecoratorApplication>,
    /// Whether this type has been finished (decorators called)
    pub is_finished: bool,
    /// Symbol ID for late-bound symbols
    pub symbol: Option<NodeId>,
}

impl Enum {
    pub fn new(id: NodeId, name: String) -> Self {
        Self {
            id,
            name,
            node: None,
            namespace: None,
            members: HashMap::new(),
            decorators: Vec::new(),
            is_finished: false,
            symbol: None,
        }
    }

    pub fn kind(&self) -> TypeKind {
        TypeKind::Enum
    }

    pub fn add_member(&mut self, member: EnumMember) {
        self.members.insert(member.name}

/// EnumMember - represents a.clone(), member);
    }
 member of an enum
#[derive(Debug, Clone)]
pub struct EnumMember {
    /// Node ID for this member
    pub id: NodeId,
    /// Source node ID
    pub node: Option<NodeId>,
    /// Name of the member
    pub name: String,
    /// The enum containing this member
    pub enum_id: Option<NodeId>,
    /// Value of the member (string or number if specified explicitly)
    pub value: Option<EnumMemberValue>,
    /// Source member if this was copied via spread
    pub source_member: Option<NodeId>,
    /// Decorators applied to this member
    pub decorators: Vec<DecoratorApplication>,
    /// Whether this type has been finished (decorators called)
    pub is_finished: bool,
}

impl EnumMember {
    pub fn new(id: NodeId, name: String) -> Self {
        Self {
            id,
            node: None,
            name,
            enum_id: None,
            value: None,
            source_member: None,
            decorators: Vec::new(),
            is_finished: false,
        }
    }

    pub fn kind(&self) -> TypeKind {
        TypeKind::EnumMember
    }
}

/// EnumMemberValue - the value of an enum member
#[derive(Debug, Clone)]
pub enum EnumMemberValue {
    String(String),
    Integer(i64),
    Float(f64),
}
