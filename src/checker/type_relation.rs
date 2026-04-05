//! Type relation checking for TypeSpec-Rust
//! Ported from TypeSpec compiler/src/core/type-relation-checker.ts
//!
//! This module handles checking type assignability and type relationships.

use crate::ast::node::NodeId;
use crate::types::model::Model;
use std::collections::HashMap;

/// Diagnostic code for type relation errors
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeRelationErrorCode {
    Unassignable,
    PropertyUnassignable,
    MissingIndex,
    PropertyRequired,
    MissingProperty,
    UnexpectedProperty,
    ParameterRequired,
}

/// Type relation error with diagnostic information
#[derive(Debug, Clone)]
pub struct TypeRelationError {
    pub code: TypeRelationErrorCode,
    pub message: String,
    pub target: NodeId,
    pub children: Vec<TypeRelationError>,
    pub skip_if_first: bool,
}

/// Relationship result between types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Related {
    False,
    True,
    Maybe,
}

/// Type relation checker for evaluating type assignability
pub struct TypeRelationChecker {
    /// Cache for type relationships to avoid repeated computation
    relation_cache: HashMap<(NodeId, NodeId), Related>,
}

impl TypeRelationChecker {
    /// Create a new type relation checker
    pub fn new() -> Self {
        Self {
            relation_cache: HashMap::new(),
        }
    }

    /// Check if the source type can be assigned to the target type
    pub fn is_type_assignable_to(
        &mut self,
        source: NodeId,
        target: NodeId,
        diagnostic_target: NodeId,
    ) -> (bool, Vec<TypeRelationError>) {
        let result = self.is_type_assignable_to_internal(source, target, diagnostic_target);
        (result.0 == Related::True, result.1)
    }

    /// Internal type assignability check
    fn is_type_assignable_to_internal(
        &mut self,
        source: NodeId,
        target: NodeId,
        diagnostic_target: NodeId,
    ) -> (Related, Vec<TypeRelationError>) {
        // Check cache first
        let cache_key = (source, target);
        if let Some(cached) = self.relation_cache.get(&cache_key) {
            return (*cached, vec![]);
        }

        // Same types are always assignable
        if source == target {
            return (Related::True, vec![]);
        }

        // Perform the actual check
        let result = self.is_type_assignable_to_worker(source, target, diagnostic_target);
        self.relation_cache.insert(cache_key, result.0);
        result
    }

    /// Main worker function for type assignability
    fn is_type_assignable_to_worker(
        &mut self,
        source: NodeId,
        target: NodeId,
        diagnostic_target: NodeId,
    ) -> (Related, Vec<TypeRelationError>) {
        // Same types are always assignable
        if source == target {
            return (Related::True, vec![]);
        }

        // Create unassignable error for basic case
        let unassignable_err = || TypeRelationError {
            code: TypeRelationErrorCode::Unassignable,
            message: format!(
                "Type with id {} is not assignable to type with id {}",
                source, target
            ),
            target: diagnostic_target,
            children: vec![],
            skip_if_first: false,
        };

        // In a full implementation, we would:
        // 1. Look up the actual types using node_type_map
        // 2. Handle template parameters
        // 3. Handle indeterminate entities
        // 4. Check simple type relationships (never, void, unknown)
        // 5. Handle model, union, enum, tuple, function type relationships
        // For now, return unassignable
        (Related::False, vec![unassignable_err()])
    }

    /// Check if a model is an array model type
    pub fn is_array_model_type(model: &Model) -> bool {
        model.indexer.is_some()
    }

    /// Check if the target is a reflection type (TypeSpec.Reflection.*)
    pub fn is_reflection_type(&self, _target: NodeId) -> bool {
        // In full implementation, check if the type is in TypeSpec.Reflection namespace
        false
    }
}

impl Default for TypeRelationChecker {
    fn default() -> Self {
        Self::new()
    }
}
