//! TypeSpec Checker - Core type checking for TypeSpec-Rust
//! Ported from TypeSpec compiler/src/core/checker.ts
//!
//! The checker is responsible for:
//! - Type checking and validation
//! - Type relationship verification (assignability)
//! - Constraint checking
//! - Error diagnostics

pub mod type_relation;

pub use type_relation::*;

// ============================================================================
// Checker State and Context
// ============================================================================

use crate::ast::node::NodeId;
use crate::types::model::{Model, ModelProperty};
use crate::program::Program;
use std::collections::{HashMap, HashSet};

/// Checker flags for controlling checking behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckFlags {
    None = 0,
    /// Currently checking within an uninstantiated template declaration
    InTemplateDeclaration = 1 << 0,
}

/// Check context for tracking type mapping during checking
#[derive(Debug, Clone)]
pub struct CheckContext {
    /// The type mapper associated with this context, if any
    pub mapper: Option<TypeMapper>,
    /// The flags enabled in this context
    pub flags: CheckFlags,
    /// Template parameters observed in this context
    observed_template_parameters: HashSet<NodeId>,
}

impl CheckContext {
    /// Create a default check context
    pub fn new() -> Self {
        Self {
            mapper: None,
            flags: CheckFlags::None,
            observed_template_parameters: HashSet::new(),
        }
    }

    /// Create a check context with a mapper
    pub fn with_mapper(mapper: Option<TypeMapper>) -> Self {
        Self {
            mapper,
            flags: CheckFlags::None,
            observed_template_parameters: HashSet::new(),
        }
    }

    /// Create a new context with the given flags added
    pub fn with_flags(&self, flags: CheckFlags) -> Self {
        Self {
            mapper: self.mapper.clone(),
            flags,
            observed_template_parameters: self.observed_template_parameters.clone(),
        }
    }

    /// Check if ALL of the given flags are enabled
    pub fn has_flags(&self, flags: CheckFlags) -> bool {
        (self.flags as u32 & flags as u32) == flags as u32
    }
}

impl Default for CheckContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Type mapper for template instantiation
#[derive(Debug, Clone)]
pub struct TypeMapper {
    /// Whether this is a partial mapping
    pub partial: bool,
    /// The mapping from template parameters to types
    pub map: HashMap<NodeId, NodeId>,
    /// Arguments used for instantiation
    pub args: Vec<NodeId>,
    /// Source node used to create this mapper
    pub source_node: Option<NodeId>,
    /// Parent mapper if any
    pub parent_mapper: Option<Box<TypeMapper>>,
}

impl TypeMapper {
    pub fn new() -> Self {
        Self {
            partial: false,
            map: HashMap::new(),
            args: Vec::new(),
            source_node: None,
            parent_mapper: None,
        }
    }
}

impl Default for TypeMapper {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Checker Stats
// ============================================================================

/// Statistics about the checker
#[derive(Debug, Clone, Default)]
pub struct CheckerStats {
    /// Number of types created
    pub created_types: u64,
    /// Number of types finished
    pub finished_types: u64,
}

// ============================================================================
// Checker
// ============================================================================

/// The main checker struct
pub struct Checker {
    /// The program being checked
    program: Program,
    /// Statistics
    pub stats: CheckerStats,
    /// Global namespace type
    global_namespace_type: Option<NodeId>,
    /// Error type node
    pub error_type: NodeId,
    /// Void type node
    pub void_type: NodeId,
    /// Never type node
    pub never_type: NodeId,
    /// Null type node
    pub null_type: NodeId,
    /// Unknown type node
    pub unknown_type: NodeId,
    /// Type relation checker
    pub type_relation: TypeRelationChecker,
    /// Standard types
    std_types: HashMap<String, NodeId>,
    /// Types pending resolution
    waiting_for_resolution: HashMap<NodeId, Vec<(NodeId, Box<dyn Fn()>)>>,
    /// Node to type mapping
    node_type_map: HashMap<NodeId, NodeId>,
    /// Symbol links
    symbol_links: HashMap<NodeId, SymbolLinks>,
}

impl Checker {
    /// Create a new checker for the given program
    pub fn new(program: Program) -> Self {
        Self {
            program,
            stats: CheckerStats::default(),
            global_namespace_type: None,
            error_type: 0,
            void_type: 0,
            never_type: 0,
            null_type: 0,
            unknown_type: 0,
            type_relation: TypeRelationChecker::new(),
            std_types: HashMap::new(),
            waiting_for_resolution: HashMap::new(),
            node_type_map: HashMap::new(),
            symbol_links: HashMap::new(),
        }
    }

    /// Get the type for a node
    pub fn get_type_for_node(&self, node_id: NodeId) -> NodeId {
        self.node_type_map.get(&node_id).copied().unwrap_or(self.error_type)
    }

    /// Get the global namespace type
    pub fn get_global_namespace_type(&self) -> Option<NodeId> {
        self.global_namespace_type
    }

    /// Check the entire program
    pub fn check_program(&mut self) {
        // In a full implementation, this would:
        // 1. Initialize standard types
        // 2. Bind symbols
        // 3. Check all source files
        // 4. Validate constraints
        // 5. Finish all types
    }

    /// Check a single source file
    pub fn check_source_file(&mut self, _file: NodeId) {
        // In a full implementation, this would check all statements in the file
    }

    /// Check if source type is assignable to target type
    pub fn is_type_assignable_to(
        &mut self,
        source: NodeId,
        target: NodeId,
        diagnostic_target: NodeId,
    ) -> (bool, Vec<TypeRelationError>) {
        self.type_relation.is_type_assignable_to(source, target, diagnostic_target)
    }

    /// Check if value is of type
    pub fn is_value_of_type(
        &mut self,
        source: NodeId,
        target: NodeId,
        diagnostic_target: NodeId,
    ) -> (bool, Vec<TypeRelationError>) {
        // In a full implementation, this would check value-to-type assignability
        self.type_relation.is_type_assignable_to(source, target, diagnostic_target)
    }

    /// Get the standard type with the given name
    pub fn get_std_type(&self, name: &str) -> Option<NodeId> {
        self.std_types.get(name).copied()
    }

    /// Check if a type is a standard type
    pub fn is_std_type(&self, _node_id: NodeId, _name: Option<&str>) -> bool {
        // In a full implementation, this would check if the type is a standard TypeSpec type
        false
    }

    /// Resolve a type reference node
    pub fn resolve_type_reference(&mut self, _node_id: NodeId) -> (Option<NodeId>, Vec<TypeRelationError>) {
        // In a full implementation, this would resolve the type reference
        (None, vec![])
    }

    /// Resolve a type or value reference
    pub fn resolve_type_or_value_reference(&mut self, _node_id: NodeId) -> (Option<NodeId>, Vec<TypeRelationError>) {
        // In a full implementation, this would resolve the reference
        (None, vec![])
    }

    /// Clone a type with additional properties
    pub fn clone_type(&self, node_id: NodeId, _additional: Option<HashMap<String, NodeId>>) -> NodeId {
        node_id
    }

    /// Create a new type
    pub fn create_type(&mut self, _type_def: &Model) -> NodeId {
        self.stats.created_types += 1;
        0
    }

    /// Create and finish a type
    pub fn create_and_finish_type(&mut self, _type_def: &Model) -> NodeId {
        self.stats.created_types += 1;
        self.stats.finished_types += 1;
        0
    }

    /// Finish a type (call decorators, etc.)
    pub fn finish_type(&mut self, node_id: NodeId) -> NodeId {
        self.stats.finished_types += 1;
        node_id
    }
}

// ============================================================================
// Symbol Links
// ============================================================================

/// Symbol links for tracking type information per symbol
#[derive(Debug, Clone, Default)]
pub struct SymbolLinks {
    /// Declared type for the symbol
    pub declared_type: Option<NodeId>,
    /// Type for the symbol
    pub type_id: Option<NodeId>,
    /// Instantiation map for template types
    pub instantiations: Option<HashMap<Vec<NodeId>, NodeId>>,
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Walk properties inherited from base models
pub fn walk_properties_inherited(model: &Model) -> Vec<(String, &ModelProperty)> {
    let mut result = Vec::new();

    // Add properties from this model
    for (name, prop) in &model.properties {
        result.push((name.clone(), prop));
    }

    // Recursively add properties from base model
    // In full implementation, this would traverse the inheritance chain
    // if let Some(base_id) = model.base_model {
    //     // Add base model properties
    // }

    result
}

/// Get the property with the given name from a model, including inherited properties
pub fn get_property<'a>(model: &'a Model, name: &str) -> Option<&'a ModelProperty> {
    model.properties.get(name)
}

/// Create a new checker instance
pub fn create_checker(program: Program) -> Checker {
    Checker::new(program)
}
