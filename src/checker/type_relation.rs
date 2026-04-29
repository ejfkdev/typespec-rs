//! Type relation checking for TypeSpec-Rust
//! Ported from TypeSpec compiler/src/core/type-relation-checker.ts
//!
//! This module handles checking type assignability and type relationships.
//! It works directly with TypeId and Type from the checker's type store.

use crate::ast::node::NodeId;
use crate::checker::types::{IntrinsicTypeName, ModelType, ScalarType, Type, TypeId, TypeStore};
use crate::numeric_ranges::is_value_in_range;
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

impl std::fmt::Display for TypeRelationErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeRelationErrorCode::Unassignable => write!(f, "unassignable"),
            TypeRelationErrorCode::PropertyUnassignable => write!(f, "property-unassignable"),
            TypeRelationErrorCode::MissingIndex => write!(f, "missing-index"),
            TypeRelationErrorCode::PropertyRequired => write!(f, "property-required"),
            TypeRelationErrorCode::MissingProperty => write!(f, "missing-property"),
            TypeRelationErrorCode::UnexpectedProperty => write!(f, "unexpected-property"),
            TypeRelationErrorCode::ParameterRequired => write!(f, "parameter-required"),
        }
    }
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

impl Related {
    pub fn is_false(&self) -> bool {
        matches!(self, Related::False)
    }

    pub fn is_true(&self) -> bool {
        matches!(self, Related::True)
    }
}

/// Type relation checker for evaluating type assignability
/// Ported from TS compiler/src/core/type-relation-checker.ts
pub struct TypeRelationChecker {
    /// Cache for type relationships to avoid repeated computation
    relation_cache: HashMap<(TypeId, TypeId), Related>,
}

impl TypeRelationChecker {
    pub fn new() -> Self {
        Self {
            relation_cache: HashMap::new(),
        }
    }

    /// Check assignability with direct type access
    /// This is the main entry point used by the Checker
    /// Full type relation check with type store access for recursive checks
    pub fn is_related_with_store(
        &mut self,
        store: &TypeStore,
        source_id: TypeId,
        target_id: TypeId,
    ) -> Related {
        if source_id == target_id {
            return Related::True;
        }

        // Check cache
        let cache_key = (source_id, target_id);
        if let Some(&cached) = self.relation_cache.get(&cache_key) {
            return cached;
        }

        let source_type = match store.get(source_id) {
            Some(t) => t,
            None => return Related::False,
        };
        let target_type = match store.get(target_id) {
            Some(t) => t,
            None => return Related::False,
        };

        // Prevent infinite recursion
        self.relation_cache.insert(cache_key, Related::Maybe);
        let result =
            self.compute_related_with_store(store, source_type, target_type, source_id, target_id);
        self.relation_cache.insert(cache_key, result);
        result
    }

    /// Full compute with type store access - enables recursive model property checks
    fn compute_related_with_store(
        &mut self,
        store: &TypeStore,
        source_type: &Type,
        target_type: &Type,
        source_id: TypeId,
        target_id: TypeId,
    ) -> Related {
        // Resolve UnionVariant types to their inner type
        // UnionVariant is a wrapper around the actual type in a union
        if let Type::UnionVariant(source_variant) = source_type {
            return self.is_related_with_store(store, source_variant.r#type, target_id);
        }
        if let Type::UnionVariant(target_variant) = target_type {
            return self.is_related_with_store(store, source_id, target_variant.r#type);
        }

        // Never is assignable to everything
        if let Type::Intrinsic(intr) = source_type
            && intr.name == IntrinsicTypeName::Never
        {
            return Related::True;
        }

        // Everything is assignable to unknown
        if let Type::Intrinsic(intr) = target_type
            && intr.name == IntrinsicTypeName::Unknown
        {
            return Related::True;
        }

        // Void is only assignable to void
        if let Type::Intrinsic(intr) = target_type
            && intr.name == IntrinsicTypeName::Void
        {
            return if let Type::Intrinsic(src) = source_type {
                if src.name == IntrinsicTypeName::Void {
                    Related::True
                } else {
                    Related::False
                }
            } else {
                Related::False
            };
        }

        // Error type is assignable to anything
        if let Type::Intrinsic(intr) = source_type
            && intr.name == IntrinsicTypeName::ErrorType
        {
            return Related::True;
        }
        if let Type::Intrinsic(intr) = target_type
            && intr.name == IntrinsicTypeName::ErrorType
        {
            return Related::True;
        }

        // Template parameter: if it has a constraint, check if the constraint is
        // assignable to the target. If the constraint is assignable, the template
        // parameter is also assignable (e.g., T extends string is assignable to string).
        // If no constraint, return Maybe (could be anything).
        if let Type::TemplateParameter(tp) = source_type
            && let Some(constraint_id) = tp.constraint
        {
            let constraint_related = self.is_related_with_store(store, constraint_id, target_id);
            if constraint_related.is_true() {
                return Related::True;
            }
            // If constraint is not assignable, the template parameter might still
            // be assignable at instantiation time, so return Maybe
            return Related::Maybe;
        }
        if let Type::TemplateParameter(_) = source_type {
            return Related::Maybe;
        }

        // Resolve alias scalars: if a Scalar has base_scalar pointing to a non-Scalar type,
        // it's an alias that should be transparent for type relation purposes
        // (e.g., alias Foo = string | int32 creates Scalar { base_scalar: Union })
        if let Type::Scalar(source_scalar) = source_type
            && let Some(base_id) = source_scalar.base_scalar
            && let Some(base_type) = store.get(base_id)
        {
            // If the base is not a Scalar (or is a Scalar with different name),
            // resolve through the base type
            match base_type {
                Type::Union(_)
                | Type::Model(_)
                | Type::Tuple(_)
                | Type::Enum(_)
                | Type::Interface(_)
                | Type::Operation(_)
                | Type::Intrinsic(_)
                | Type::String(_)
                | Type::Number(_)
                | Type::Boolean(_) => {
                    // Alias pointing to a non-scalar type — resolve through
                    return self.is_related_with_store(store, base_id, target_id);
                }
                Type::Scalar(base_scalar) => {
                    // If base scalar has a different name, it's a real extends relationship
                    // If same name or the base itself is an alias, resolve further
                    if source_scalar.name != base_scalar.name {
                        // This is a real extends relationship (e.g., myString extends string)
                        // Don't dereference - let are_scalars_related handle it
                    } else {
                        // Same name means this is a re-export or alias
                        return self.is_related_with_store(store, base_id, target_id);
                    }
                }
                _ => {}
            }
        }
        // Target scalar with base_scalar: delegate to base for non-scalar bases
        // (e.g., alias Foo = string | int32 creates Scalar with base=Union).
        // For Scalar bases (e.g., S extends int8), only delegate when the source
        // is a literal type (Number, String, Boolean). This lets
        // is_simple_type_assignable_to handle the range checking against the
        // primitive base. Do NOT delegate for Scalar->Scalar because
        // are_scalars_related handles that (and would incorrectly make string
        // assignable to myString extends string).
        if let Type::Scalar(target_scalar) = target_type
            && let Some(base_id) = target_scalar.base_scalar
            && let Some(base_type) = store.get(base_id)
        {
            match base_type {
                Type::Union(_)
                | Type::Model(_)
                | Type::Tuple(_)
                | Type::Enum(_)
                | Type::Interface(_)
                | Type::Operation(_)
                | Type::Intrinsic(_)
                | Type::String(_)
                | Type::Number(_)
                | Type::Boolean(_) => {
                    // Non-scalar base (e.g., alias pointing to union) — always delegate
                    return self.is_related_with_store(store, source_id, base_id);
                }
                Type::Scalar(base_scalar) => {
                    // Only delegate for literal types to custom scalars
                    // (e.g., Number(9999) assignable to S extends int8?
                    // → check Number(9999) assignable to int8)
                    // Do NOT delegate when target is a builtin scalar —
                    // is_simple_type_assignable_to handles those directly.
                    // Do NOT delegate for Scalar->Scalar (are_scalars_related handles this).
                    let source_is_literal = matches!(
                        source_type,
                        Type::Number(_) | Type::String(_) | Type::Boolean(_)
                    );
                    // Check if TARGET is a builtin scalar (not custom)
                    let target_is_builtin =
                        crate::std::helpers::is_builtin_scalar(&target_scalar.name);
                    if source_is_literal
                        && !target_is_builtin
                        && target_scalar.name != base_scalar.name
                    {
                        // Custom scalar with literal source — delegate to builtin base.
                        // Once we reach a builtin target, is_simple_type_assignable_to
                        // will handle the check directly (no further delegation).
                        return self.is_related_with_store(store, source_id, base_id);
                    }
                    // For other cases, fall through to is_simple_type_assignable_to
                    // and are_scalars_related
                }
                _ => {}
            }
        }

        // Simple type checks
        match self.is_simple_type_assignable_to(source_type, target_type) {
            Some(true) => return Related::True,
            Some(false) => return Related::False,
            None => {}
        }

        // Union source: all variants must be assignable to target
        if let Type::Union(source_union) = source_type {
            for name in &source_union.variant_names {
                if let Some(&variant_id) = source_union.variants.get(name)
                    && self
                        .is_related_with_store(store, variant_id, target_id)
                        .is_false()
                {
                    return Related::False;
                }
            }
            return Related::True;
        }

        // Target is union: source must be assignable to at least one variant
        if let Type::Union(target_union) = target_type {
            for name in &target_union.variant_names {
                if let Some(&variant_id) = target_union.variants.get(name) {
                    let related = self.is_related_with_store(store, source_id, variant_id);
                    if related.is_true() {
                        return Related::True;
                    }
                }
            }
            return Related::False;
        }

        // Model to model
        if let (Type::Model(source_model), Type::Model(target_model)) = (source_type, target_type) {
            return self.are_models_related(
                store,
                source_model,
                target_model,
                source_id,
                target_id,
            );
        }

        // Scalar to scalar
        if let (Type::Scalar(source_scalar), Type::Scalar(target_scalar)) =
            (source_type, target_type)
        {
            return if self.are_scalars_related(store, source_scalar, target_scalar) {
                Related::True
            } else {
                Related::False
            };
        }

        // Tuple to tuple
        if let (Type::Tuple(source_tuple), Type::Tuple(target_tuple)) = (source_type, target_type) {
            if source_tuple.values.len() != target_tuple.values.len() {
                return Related::False;
            }
            for (&src_val, &tgt_val) in source_tuple.values.iter().zip(target_tuple.values.iter()) {
                if self
                    .is_related_with_store(store, src_val, tgt_val)
                    .is_false()
                {
                    return Related::False;
                }
            }
            return Related::True;
        }

        // Enum to enum: only same enum
        if let (Type::Enum(source_enum), Type::Enum(target_enum)) = (source_type, target_type) {
            return if source_enum.name == target_enum.name {
                Related::True
            } else {
                Related::False
            };
        }

        // EnumMember to its parent enum: assignable
        if let (Type::EnumMember(source_member), Type::Enum(_)) = (source_type, target_type) {
            // An enum member is assignable to its parent enum
            if let Some(parent_enum_id) = source_member.r#enum
                && parent_enum_id == target_id
            {
                return Related::True;
            }
            return Related::False;
        }

        // Tuple to Array (Model with integer indexer): each tuple element must be assignable to array element type
        if let (Type::Tuple(source_tuple), Type::Model(target_model)) = (source_type, target_type) {
            // Check if target is an array (has integer-keyed indexer)
            if let Some((_, target_val)) = &target_model.indexer {
                for &src_val in &source_tuple.values {
                    if self
                        .is_related_with_store(store, src_val, *target_val)
                        .is_false()
                    {
                        return Related::False;
                    }
                }
                return Related::True;
            }
        }

        // Intersection type handling: resolve intersection to its effective type
        // Intersection of compatible types → use the more specific type
        // Intersection of incompatible types → never
        // For now, treat intersection source by checking if all intersection members are related to target
        // and intersection target by checking if source is related to all intersection members
        if let Type::Model(source_model) = source_type {
            // Check if source is an intersection model (has intersection_source)
            if !source_model.source_models.is_empty() {
                // Intersection source: all constituent types must be assignable to target
                for src_model in &source_model.source_models {
                    if self
                        .is_related_with_store(store, src_model.model, target_id)
                        .is_false()
                    {
                        return Related::False;
                    }
                }
                return Related::True;
            }
        }
        if let Type::Model(target_model) = target_type {
            // Check if target is an intersection model
            if !target_model.source_models.is_empty() {
                // Intersection target: source must be assignable to all constituent types
                for tgt_model in &target_model.source_models {
                    if self
                        .is_related_with_store(store, source_id, tgt_model.model)
                        .is_false()
                    {
                        return Related::False;
                    }
                }
                return Related::True;
            }
        }

        // Interface to interface: same identity or extends chain
        if let (Type::Interface(source_iface), Type::Interface(_target_iface)) =
            (source_type, target_type)
        {
            // Same interface
            if source_id == target_id {
                return Related::True;
            }
            // Source extends target (directly or transitively)
            for &extends_id in &source_iface.extends {
                if extends_id == target_id {
                    return Related::True;
                }
                // Walk the extends chain
                if self.walk_interface_extends_chain(store, extends_id, target_id) {
                    return Related::True;
                }
            }
            return Related::False;
        }

        Related::False
    }

    /// Simple type assignability checks (intrinsic, literal-to-scalar, same-name)
    /// Returns Some(true) if definitely assignable, Some(false) if definitely not, None if need more checks
    fn is_simple_type_assignable_to(&self, source_type: &Type, target_type: &Type) -> Option<bool> {
        // Same type kind with same name
        match (source_type, target_type) {
            // Scalar assignability: source extends target
            (Type::Scalar(source_scalar), Type::Scalar(target_scalar)) => {
                if source_scalar.name == target_scalar.name {
                    return Some(true);
                }
                // Can't check inheritance chain without store, return None
                return None;
            }

            // String literal is assignable to string scalar
            (Type::String(_), Type::Scalar(target_scalar)) => {
                if target_scalar.name == "string" {
                    return Some(true);
                }
                return Some(false);
            }

            // Numeric literal is assignable to numeric scalar types
            // Ported from TS type-relation-checker.ts:isNumericLiteralRelatedTo
            (Type::Number(num), Type::Scalar(target_scalar)) => {
                let target_name = target_scalar.name.as_str();
                // Unbounded types
                if target_name == "numeric"
                    || target_name == "decimal"
                    || target_name == "decimal128"
                {
                    return Some(true);
                }
                // integer accepts any integer value
                if target_name == "integer" {
                    return Some(num.value.fract() == 0.0);
                }
                // float accepts any value
                if target_name == "float" {
                    return Some(true);
                }
                // Check if target is a numeric scalar type at all
                let is_numeric_target = crate::numeric_ranges::get_numeric_ranges()
                    .iter()
                    .any(|(name, _)| *name == target_name);
                if is_numeric_target {
                    // Known bounded numeric type — check range
                    if let Ok(numeric) = crate::numeric::Numeric::new(&num.value_as_string) {
                        return Some(is_value_in_range(&numeric, target_name));
                    }
                    return Some(true);
                }
                // Non-numeric scalar (string, boolean, etc.) — Number is not assignable
                // unless we need store-based resolution for custom scalars
                if target_scalar.base_scalar.is_some() {
                    // Custom scalar — need store to walk base_scalar chain
                    return None;
                }
                return Some(false);
            }

            // Boolean literal is assignable to boolean scalar
            (Type::Boolean(_), Type::Scalar(target_scalar)) => {
                if target_scalar.name == "boolean" {
                    return Some(true);
                }
                return Some(false);
            }

            // String literal equality
            (Type::String(src), Type::String(tgt)) => {
                return Some(src.value == tgt.value);
            }

            // Numeric literal equality
            (Type::Number(src), Type::Number(tgt)) => {
                return Some(src.value == tgt.value);
            }

            // Boolean literal equality
            (Type::Boolean(src), Type::Boolean(tgt)) => {
                return Some(src.value == tgt.value);
            }

            // Enum member to its parent enum — needs store access, return None
            (Type::EnumMember(_), Type::Enum(_)) => {
                return None;
            }

            _ => {}
        }
        None
    }

    /// Check if two scalars are related (source extends target, directly or transitively)
    fn are_scalars_related(
        &self,
        store: &TypeStore,
        source: &ScalarType,
        target: &ScalarType,
    ) -> bool {
        // Same name (assuming same namespace for now)
        if source.name == target.name {
            return true;
        }
        // Walk source's base_scalar chain
        let mut current_base = source.base_scalar;
        while let Some(base_id) = current_base {
            if let Some(Type::Scalar(base_scalar)) = store.get(base_id) {
                if base_scalar.name == target.name {
                    return true;
                }
                current_base = base_scalar.base_scalar;
            } else {
                break;
            }
        }
        false
    }

    /// Check if source model is assignable to target model
    /// Ported from TS areModelsRelated — uses structural checking for all models,
    /// not just anonymous ones. Named models are also checked structurally.
    fn are_models_related(
        &mut self,
        store: &TypeStore,
        source: &ModelType,
        target: &ModelType,
        source_id: TypeId,
        target_id: TypeId,
    ) -> Related {
        // Same model (same TypeId = same instance)
        if source_id == target_id {
            return Related::True;
        }

        // Collect all properties from both models (including inherited)
        let target_props = self.collect_model_properties_inherited(store, target_id);
        let source_props = self.collect_model_properties_inherited(store, source_id);

        // Track which source properties were matched by target properties
        let mut matched_source_keys: std::collections::HashSet<String> =
            std::collections::HashSet::new();

        // Check each target property against source
        for (prop_name, (target_prop_type, target_optional)) in &target_props {
            if let Some(&(source_prop_type, source_optional)) = source_props.get(prop_name) {
                matched_source_keys.insert(prop_name.clone());

                // Check: source optional → target required is not allowed
                if source_optional && !target_optional {
                    return Related::False;
                }

                // Check property type compatibility
                let prop_related =
                    self.is_related_with_store(store, source_prop_type, *target_prop_type);
                if prop_related.is_false() {
                    return Related::False;
                }
            } else {
                // Property missing in source - only ok if target property is optional
                if !target_optional {
                    return Related::False;
                }
            }
        }

        // Check source properties don't violate target indexer
        if let Some((_, target_val)) = &target.indexer {
            // If target has an indexer, remaining source properties must be assignable to indexer value type
            for (prop_name, &(source_prop_type, _)) in &source_props {
                if matched_source_keys.contains(prop_name) {
                    continue;
                }
                if self
                    .is_related_with_store(store, source_prop_type, *target_val)
                    .is_false()
                {
                    return Related::False;
                }
            }

            // For named source models, also check that source has a compatible indexer
            // (unless the target indexer key is "integer" which means array)
            if !source.name.is_empty() {
                if let Some((source_key, source_val)) = &source.indexer {
                    // Source has indexer — check key and value compatibility
                    if let Some((target_key, _)) = &target.indexer {
                        let key_related =
                            self.is_related_with_store(store, *source_key, *target_key);
                        let val_related =
                            self.is_related_with_store(store, *source_val, *target_val);
                        if key_related.is_true() && val_related.is_true() {
                            return Related::True;
                        }
                        return Related::False;
                    }
                } else {
                    // Named source model without indexer but target has one
                    // Check if target indexer key is not integer (not array)
                    if let Some((target_key, _)) = &target.indexer
                        && let Some(Type::Scalar(s)) = store.get(*target_key)
                        && s.name != "integer"
                    {
                        // Source needs an indexer but doesn't have one
                        return Related::False;
                    }
                }
            }

            // Also check source indexer compatibility if source has one
            if let Some((_, source_val)) = &source.indexer {
                let val_related = self.is_related_with_store(store, *source_val, *target_val);
                return if val_related.is_true() {
                    Related::True
                } else {
                    Related::False
                };
            }
        }

        // All target properties matched, or target has no properties (empty model)
        // But check: if source has an indexer (especially integer = array) and target doesn't,
        // they're not compatible (array cannot be assigned to non-indexed model)
        // Ported from TS: missing-index check
        if source.indexer.is_some() && target.indexer.is_none() {
            // Source has an indexer but target doesn't — only ok if source is anonymous
            // and doesn't require its own indexer to be present
            if !source.name.is_empty() {
                return Related::False;
            }
            // For anonymous source models with indexer, check if the indexer is integer (array)
            if let Some((source_key, _)) = &source.indexer
                && let Some(Type::Scalar(s)) = store.get(*source_key)
                && s.name == "integer"
            {
                // Array model cannot be assigned to non-array target
                return Related::False;
            }
        }

        Related::True
    }

    /// Walk the interface extends chain to check if a source interface
    /// eventually extends the target interface.
    fn walk_interface_extends_chain(
        &mut self,
        store: &TypeStore,
        start_id: TypeId,
        target_id: TypeId,
    ) -> bool {
        let mut visited = std::collections::HashSet::new();
        let mut stack = vec![start_id];
        while let Some(cur_id) = stack.pop() {
            if cur_id == target_id {
                return true;
            }
            if visited.contains(&cur_id) {
                continue;
            }
            visited.insert(cur_id);
            if let Some(Type::Interface(iface)) = store.get(cur_id) {
                for &extends_id in &iface.extends {
                    if !visited.contains(&extends_id) {
                        stack.push(extends_id);
                    }
                }
            }
        }
        false
    }

    /// Collect all properties from a model including inherited ones (walks extends chain)
    /// Returns (property_name -> (property_type_id, is_optional))
    fn collect_model_properties_inherited(
        &self,
        store: &TypeStore,
        model_id: TypeId,
    ) -> HashMap<String, (TypeId, bool)> {
        let mut result = HashMap::new();
        let mut current = Some(model_id);
        while let Some(cur_id) = current {
            if let Some(Type::Model(m)) = store.get(cur_id) {
                for (name, &prop_id) in &m.properties {
                    // Don't override child properties with parent ones
                    if !result.contains_key(name) {
                        // Get the property type and optional flag from the ModelProperty
                        if let Some(Type::ModelProperty(prop)) = store.get(prop_id) {
                            result.insert(name.clone(), (prop.r#type, prop.optional));
                        }
                    }
                }
                current = m.base_model;
            } else {
                break;
            }
        }
        result
    }

    /// Check if value is of the given type
    /// Ported from TS isValueOfType
    pub fn is_value_of_type(
        &mut self,
        store: &TypeStore,
        value_type_id: TypeId,
        target_id: TypeId,
    ) -> bool {
        self.is_related_with_store(store, value_type_id, target_id)
            .is_true()
    }

    /// Clear the relation cache
    pub fn clear_cache(&mut self) {
        self.relation_cache.clear();
    }
}

impl Default for TypeRelationChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::types::*;

    fn create_test_type_store() -> (TypeStore, TypeId, TypeId, TypeId, TypeId, TypeId) {
        let mut store = TypeStore::new();
        let error_type = store.add(Type::Intrinsic(IntrinsicType {
            id: store.next_type_id(),
            name: IntrinsicTypeName::ErrorType,
            node: None,
            is_finished: true,
        }));
        let void_type = store.add(Type::Intrinsic(IntrinsicType {
            id: store.next_type_id(),
            name: IntrinsicTypeName::Void,
            node: None,
            is_finished: true,
        }));
        let never_type = store.add(Type::Intrinsic(IntrinsicType {
            id: store.next_type_id(),
            name: IntrinsicTypeName::Never,
            node: None,
            is_finished: true,
        }));
        let unknown_type = store.add(Type::Intrinsic(IntrinsicType {
            id: store.next_type_id(),
            name: IntrinsicTypeName::Unknown,
            node: None,
            is_finished: true,
        }));
        let null_type = store.add(Type::Intrinsic(IntrinsicType {
            id: store.next_type_id(),
            name: IntrinsicTypeName::Null,
            node: None,
            is_finished: true,
        }));
        (
            store,
            error_type,
            void_type,
            never_type,
            unknown_type,
            null_type,
        )
    }

    #[test]
    fn test_same_type_is_assignable() {
        let mut checker = TypeRelationChecker::new();
        let (mut store, _, _, _, _, _) = create_test_type_store();
        let string_type = store.add(Type::Scalar({
            let mut s =
                ScalarType::new(store.next_type_id(), "string".to_string(), None, None, None);
            s.is_finished = true;
            s
        }));
        assert_eq!(
            checker.is_related_with_store(&store, string_type, string_type),
            Related::True
        );
    }

    #[test]
    fn test_never_assignable_to_anything() {
        let mut checker = TypeRelationChecker::new();
        let (mut store, _, _, never_type, _, _) = create_test_type_store();
        let model_type = store.add(Type::Model({
            let mut m = ModelType::new(store.next_type_id(), "Foo".to_string(), None, None);
            m.is_finished = true;
            m
        }));
        assert_eq!(
            checker.is_related_with_store(&store, never_type, model_type),
            Related::True
        );
    }

    #[test]
    fn test_anything_assignable_to_unknown() {
        let mut checker = TypeRelationChecker::new();
        let (mut store, _, _, _, unknown_type, _) = create_test_type_store();
        let model_type = store.add(Type::Model({
            let mut m = ModelType::new(store.next_type_id(), "Foo".to_string(), None, None);
            m.is_finished = true;
            m
        }));
        assert_eq!(
            checker.is_related_with_store(&store, model_type, unknown_type),
            Related::True
        );
    }

    #[test]
    fn test_void_only_assignable_to_void() {
        let mut checker = TypeRelationChecker::new();
        let (store, _, void_type, _, _, _) = create_test_type_store();
        assert_eq!(
            checker.is_related_with_store(&store, void_type, void_type),
            Related::True
        );
    }

    #[test]
    fn test_string_literal_assignable_to_string_scalar() {
        let mut checker = TypeRelationChecker::new();
        let mut store = TypeStore::new();
        let string_scalar = store.add(Type::Scalar({
            let mut s =
                ScalarType::new(store.next_type_id(), "string".to_string(), None, None, None);
            s.is_finished = true;
            s
        }));
        let string_literal = store.add(Type::String(StringType {
            id: store.next_type_id(),
            value: "hello".to_string(),
            node: None,
            is_finished: true,
        }));
        assert_eq!(
            checker.is_related_with_store(&store, string_literal, string_scalar),
            Related::True
        );
    }

    #[test]
    fn test_numeric_literal_assignable_to_int32() {
        let mut checker = TypeRelationChecker::new();
        let mut store = TypeStore::new();
        let int32_scalar = store.add(Type::Scalar({
            let mut s =
                ScalarType::new(store.next_type_id(), "int32".to_string(), None, None, None);
            s.is_finished = true;
            s
        }));
        let num_literal = store.add(Type::Number(NumericType {
            id: store.next_type_id(),
            value: 42.0,
            value_as_string: "42".to_string(),
            node: None,
            is_finished: true,
        }));
        assert_eq!(
            checker.is_related_with_store(&store, num_literal, int32_scalar),
            Related::True
        );
    }

    #[test]
    fn test_boolean_literal_assignable_to_boolean_scalar() {
        let mut checker = TypeRelationChecker::new();
        let mut store = TypeStore::new();
        let bool_scalar = store.add(Type::Scalar({
            let mut s = ScalarType::new(
                store.next_type_id(),
                "boolean".to_string(),
                None,
                None,
                None,
            );
            s.is_finished = true;
            s
        }));
        let bool_literal = store.add(Type::Boolean(BooleanType {
            id: store.next_type_id(),
            value: true,
            node: None,
            is_finished: true,
        }));
        assert_eq!(
            checker.is_related_with_store(&store, bool_literal, bool_scalar),
            Related::True
        );
    }

    #[test]
    fn test_different_enums_not_assignable() {
        let mut checker = TypeRelationChecker::new();
        let mut store = TypeStore::new();
        let enum1 = store.add(Type::Enum({
            let mut e = EnumType::new(store.next_type_id(), "Status".to_string(), None, None);
            e.is_finished = true;
            e
        }));
        let enum2 = store.add(Type::Enum({
            let mut e = EnumType::new(store.next_type_id(), "Color".to_string(), None, None);
            e.is_finished = true;
            e
        }));
        assert_eq!(
            checker.is_related_with_store(&store, enum1, enum2),
            Related::False
        );
    }

    #[test]
    fn test_same_enum_assignable() {
        let mut checker = TypeRelationChecker::new();
        let mut store = TypeStore::new();
        let enum1 = store.add(Type::Enum({
            let mut e = EnumType::new(store.next_type_id(), "Status".to_string(), None, None);
            e.is_finished = true;
            e
        }));
        assert_eq!(
            checker.is_related_with_store(&store, enum1, enum1),
            Related::True
        );
    }

    #[test]
    fn test_error_type_assignable_to_anything() {
        let mut checker = TypeRelationChecker::new();
        let (mut store, error_type, _, _, _, _) = create_test_type_store();
        let model_type = store.add(Type::Model({
            let mut m = ModelType::new(store.next_type_id(), "Foo".to_string(), None, None);
            m.is_finished = true;
            m
        }));
        assert_eq!(
            checker.is_related_with_store(&store, error_type, model_type),
            Related::True
        );
    }

    #[test]
    fn test_scalar_extends_assignable() {
        let mut checker = TypeRelationChecker::new();
        let mut store = TypeStore::new();

        // Create base scalar "string"
        let string_scalar = store.add(Type::Scalar({
            let mut s =
                ScalarType::new(store.next_type_id(), "string".to_string(), None, None, None);
            s.is_finished = true;
            s
        }));

        // Create derived scalar "myString" extends string
        let my_string = store.add(Type::Scalar({
            let mut s = ScalarType::new(
                store.next_type_id(),
                "myString".to_string(),
                None,
                None,
                Some(string_scalar),
            );
            s.is_finished = true;
            s
        }));

        // myString should be assignable to string
        assert_eq!(
            checker.is_related_with_store(&store, my_string, string_scalar),
            Related::True
        );
        // string should NOT be assignable to myString
        assert_eq!(
            checker.is_related_with_store(&store, string_scalar, my_string),
            Related::False
        );
    }

    #[test]
    fn test_model_extends_assignable() {
        let mut checker = TypeRelationChecker::new();
        let mut store = TypeStore::new();

        // Create string scalar for property type
        let string_scalar = store.add(Type::Scalar({
            let mut s =
                ScalarType::new(store.next_type_id(), "string".to_string(), None, None, None);
            s.is_finished = true;
            s
        }));

        // Create property for Base
        let base_prop = store.add(Type::ModelProperty(ModelPropertyType {
            id: store.next_type_id(),
            name: "name".to_string(),
            node: None,
            r#type: string_scalar,
            optional: false,
            default_value: None,
            model: None,
            source_property: None,
            decorators: vec![],
            is_finished: true,
        }));

        // Create Base model
        let base_model = store.add(Type::Model({
            let mut m = ModelType::new(store.next_type_id(), "Base".to_string(), None, None);
            m.properties = HashMap::from([("name".to_string(), base_prop)]);
            m.property_names = vec!["name".to_string()];
            m.is_finished = true;
            m
        }));

        // Create Derived extends Base
        let derived_model = store.add(Type::Model({
            let mut m = ModelType::new(store.next_type_id(), "Derived".to_string(), None, None);
            m.base_model = Some(base_model);
            m.is_finished = true;
            m
        }));

        // Derived should be assignable to Base
        assert_eq!(
            checker.is_related_with_store(&store, derived_model, base_model),
            Related::True
        );
    }

    #[test]
    fn test_union_source_all_variants_must_match() {
        let mut checker = TypeRelationChecker::new();
        let mut store = TypeStore::new();

        let string_scalar = store.add(Type::Scalar({
            let mut s =
                ScalarType::new(store.next_type_id(), "string".to_string(), None, None, None);
            s.is_finished = true;
            s
        }));

        let int32_scalar = store.add(Type::Scalar({
            let mut s =
                ScalarType::new(store.next_type_id(), "int32".to_string(), None, None, None);
            s.is_finished = true;
            s
        }));

        // Source union: string | int32 — both assignable to string? No
        let source_union = store.add(Type::Union({
            let mut u = UnionType::new(store.next_type_id(), String::new(), None, None, true);
            u.variants = HashMap::from([
                ("a".to_string(), string_scalar),
                ("b".to_string(), int32_scalar),
            ]);
            u.variant_names = vec!["a".to_string(), "b".to_string()];
            u.is_finished = true;
            u
        }));

        // Source union to string should be False (not all variants match)
        assert_eq!(
            checker.is_related_with_store(&store, source_union, string_scalar),
            Related::False
        );
    }

    #[test]
    fn test_target_union_at_least_one_variant() {
        let mut checker = TypeRelationChecker::new();
        let mut store = TypeStore::new();

        let string_scalar = store.add(Type::Scalar({
            let mut s =
                ScalarType::new(store.next_type_id(), "string".to_string(), None, None, None);
            s.is_finished = true;
            s
        }));

        let int32_scalar = store.add(Type::Scalar({
            let mut s =
                ScalarType::new(store.next_type_id(), "int32".to_string(), None, None, None);
            s.is_finished = true;
            s
        }));

        // Target union: string | int32
        let target_union = store.add(Type::Union({
            let mut u = UnionType::new(store.next_type_id(), String::new(), None, None, true);
            u.variants = HashMap::from([
                ("a".to_string(), string_scalar),
                ("b".to_string(), int32_scalar),
            ]);
            u.variant_names = vec!["a".to_string(), "b".to_string()];
            u.is_finished = true;
            u
        }));

        // String should be assignable to (string | int32) — True
        assert_eq!(
            checker.is_related_with_store(&store, string_scalar, target_union),
            Related::True
        );
    }

    #[test]
    fn test_tuple_assignability() {
        let mut checker = TypeRelationChecker::new();
        let mut store = TypeStore::new();

        let string_scalar = store.add(Type::Scalar({
            let mut s =
                ScalarType::new(store.next_type_id(), "string".to_string(), None, None, None);
            s.is_finished = true;
            s
        }));

        let int32_scalar = store.add(Type::Scalar({
            let mut s =
                ScalarType::new(store.next_type_id(), "int32".to_string(), None, None, None);
            s.is_finished = true;
            s
        }));

        let tuple1 = store.add(Type::Tuple(TupleType {
            id: store.next_type_id(),
            node: None,
            values: vec![string_scalar, int32_scalar],
            is_finished: true,
        }));

        let tuple2 = store.add(Type::Tuple(TupleType {
            id: store.next_type_id(),
            node: None,
            values: vec![string_scalar, int32_scalar],
            is_finished: true,
        }));

        // Same structure tuples should be assignable
        assert_eq!(
            checker.is_related_with_store(&store, tuple1, tuple2),
            Related::True
        );
    }
}
