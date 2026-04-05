//! Model types for TypeSpec-Rust
//! Ported from TypeSpec compiler/src/core/types.ts

use super::decorator::DecoratorApplication;
use super::primitive::{ModelIndexer, TypeKind};
use crate::ast::node::NodeId;
use std::collections::{HashMap, HashSet};

/// Source model - represents how a model was used in source
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceModelUsage {
    /// `model A is B`
    Is,
    /// `model A {...B}`
    Spread,
    /// `alias A = B & C`
    Intersection,
}

/// Source model - a model that was used to build another model
#[derive(Debug, Clone)]
pub struct SourceModel {
    /// How this model was used
    pub usage: SourceModelUsage,
    /// The source model
    pub model: NodeId,
    /// Node where this source model was referenced
    pub node: Option<NodeId>,
}

/// Model - represents a TypeSpec model type
/// Equivalent to interfaces/classes in other languages
#[derive(Debug, Clone)]
pub struct Model {
    /// Node ID for this model
    pub id: NodeId,
    /// Name of the model
    pub name: String,
    /// Source node ID
    pub node: Option<NodeId>,
    /// Namespace containing this model
    pub namespace: Option<NodeId>,
    /// Indexer for array-style access (e.g., `model Array<T> { [index: string]: T }`)
    pub indexer: Option<ModelIndexer>,
    /// Properties of the model (ordered as they appear in source)
    pub properties: HashMap<String, ModelProperty>,
    /// Model this model extends (inheritance)
    pub base_model: Option<NodeId>,
    /// Direct children models (reverse of baseModel)
    pub derived_models: HashSet<NodeId>,
    /// Source model referenced via `model is`
    pub source_model: Option<NodeId>,
    /// Models used to build this model (via `model is`, `...`, or intersection)
    pub source_models: Vec<SourceModel>,
    /// Template mapper if this is a template instantiation
    pub template_mapper: Option<NodeId>,
    /// Template node if this is a template declaration
    pub template_node: Option<NodeId>,
    /// Decorators applied to this model
    pub decorators: Vec<DecoratorApplication>,
    /// Whether this type has been finished (decorators called)
    pub is_finished: bool,
    /// Symbol ID for late-bound symbols
    pub symbol: Option<NodeId>,
}

impl Model {
    pub fn new(id: NodeId, name: String) -> Self {
        Self {
            id,
            name,
            node: None,
            namespace: None,
            indexer: None,
            properties: HashMap::new(),
            base_model: None,
            derived_models: HashSet::new(),
            source_model: None,
            source_models: Vec::new(),
            template_mapper: None,
            template_node: None,
            decorators: Vec::new(),
            is_finished: false,
            symbol: None,
        }
    }

    pub fn kind(&self) -> TypeKind {
        TypeKind::Model
    }
}

/// ModelProperty - represents a property in a model
#[derive(Debug, Clone)]
pub struct ModelProperty {
    /// Node ID for this property
    pub id: NodeId,
    /// Source node ID
    pub node: Option<NodeId>,
    /// Name of the property
    pub name: String,
    /// Type of the property
    pub r#type: NodeId,
    /// Source property if this was copied via spread/intersection
    pub source_property: Option<NodeId>,
    /// Whether this property is optional (`prop?`)
    pub optional: bool,
    /// Default value expression
    pub default_value: Option<NodeId>,
    /// Model containing this property
    pub model: Option<NodeId>,
    /// Decorators applied to this property
    pub decorators: Vec<DecoratorApplication>,
    /// Whether this type has been finished (decorators called)
    pub is_finished: bool,
}

impl ModelProperty {
    pub fn new(id: NodeId, name: String, property_type: NodeId) -> Self {
        Self {
            id,
            node: None,
            name,
            r#type: property_type,
            source_property: None,
            optional: false,
            default_value: None,
            model: None,
            decorators: Vec::new(),
            is_finished: false,
        }
    }

    pub fn kind(&self) -> TypeKind {
        TypeKind::ModelProperty
    }
}

/// ArrayModelType - a model that represents an array
#[derive(Debug, Clone)]
pub struct ArrayModelType {
    /// The underlying model
    pub model: NodeId,
    /// The indexer defining array behavior
    pub indexer: ModelIndexer,
}

/// RecordModelType - a model that represents a record/object
#[derive(Debug, Clone)]
pub struct RecordModelType {
    /// The underlying model
    pub model: NodeId,
    /// The indexer defining record behavior
    pub indexer: ModelIndexer,
}
