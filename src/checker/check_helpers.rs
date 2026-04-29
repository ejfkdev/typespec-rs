//! Helper methods for type checking
//!
//! Ported from TypeSpec checker helper methods

use super::*;

impl Checker {
    /// Directives (#deprecated, #suppress) are processed early by `process_directives()`
    /// during their parent node's checking. This function handles the rare case where
    /// a DirectiveExpression node is visited directly via `check_node` dispatch —
    /// in that case the directive has already been processed, so no action is needed.
    pub(crate) fn check_directive(&mut self, _node_id: NodeId) {}

    // ========================================================================
    // Std type accessors
    // ========================================================================

    /// Get a standard type by name
    pub fn get_std_type(&self, name: &str) -> Option<TypeId> {
        self.std_types.get(name).copied()
    }

    /// Check if a type is a standard TypeSpec built-in type.
    /// Ported from TS checker.isStdType().
    pub fn is_std_type(&self, type_id: TypeId, std_type: Option<&str>) -> bool {
        if let Some(name) = std_type {
            self.std_types.get(name) == Some(&type_id)
        } else {
            self.std_types.values().any(|&id| id == type_id)
        }
    }

    /// Create and immediately finish a type.
    /// Ported from TS checker.createAndFinishType().
    pub fn create_and_finish_type(&mut self, t: Type) -> TypeId {
        let id = self.create_type(t);
        self.finish_type(id)
    }

    /// Create a type mapper from template parameters and arguments.
    /// Ported from TS createTypeMapper().
    pub fn create_type_mapper(
        parameters: &[NodeId],
        args: &[TypeId],
        source_node: Option<NodeId>,
        parent_mapper: Option<TypeMapper>,
    ) -> TypeMapper {
        let mut map = HashMap::new();
        for (i, &param_id) in parameters.iter().enumerate() {
            if i < args.len() {
                map.insert(param_id, args[i]);
            }
        }
        TypeMapper {
            partial: parameters.len() > args.len(),
            map,
            args: args.to_vec(),
            source_node,
            parent_mapper: parent_mapper.map(Box::new),
        }
    }

    /// Get the global namespace type
    pub fn get_global_namespace_type(&self) -> Option<TypeId> {
        self.global_namespace_type
    }

    // ========================================================================
    // Deprecation tracking
    // ========================================================================

    /// Check if a type is deprecated
    pub fn is_deprecated(&self, type_id: TypeId) -> bool {
        self.deprecation_tracker.is_deprecated(type_id)
    }

    /// Get deprecation details for a type
    pub fn get_deprecation_details(
        &self,
        type_id: TypeId,
    ) -> Option<&crate::deprecation::DeprecationDetails> {
        self.deprecation_tracker.get_deprecation_details(type_id)
    }

    /// Mark a type as deprecated with a message
    pub fn mark_deprecated(&mut self, type_id: TypeId, message: String) {
        self.deprecation_tracker
            .mark_deprecated(type_id, crate::deprecation::DeprecationDetails { message });
    }

    /// Copy deprecation information from source type to destination type.
    /// If the source type is deprecated, the destination type will also be marked
    /// as deprecated with the same message.
    /// Ported from TS checker copyDeprecation().
    pub fn copy_deprecation(&mut self, source_type_id: TypeId, dest_type_id: TypeId) {
        if let Some(details) = self.get_deprecation_details(source_type_id).cloned() {
            self.deprecation_tracker
                .mark_deprecated(dest_type_id, details);
        }
    }

    // ========================================================================
    // Property helpers
    // ========================================================================

    /// Define a property on a model type, checking for duplicates and override compatibility.
    /// Returns true if the property was added, false if it was rejected (duplicate or incompatible override).
    /// Ported from TS checker defineProperty().
    pub fn define_property(&mut self, model_type_id: TypeId, prop_type_id: TypeId) -> bool {
        let (prop_name, new_prop_type, is_optional) = match self.get_type(prop_type_id) {
            Some(Type::ModelProperty(p)) => (p.name.clone(), p.r#type, p.optional),
            _ => return false,
        };

        // Check if model already has a property with this name
        let has_duplicate = match self.get_type(model_type_id) {
            Some(Type::Model(m)) => m.properties.contains_key(&prop_name),
            _ => false,
        };

        if has_duplicate {
            let model_name = self
                .get_type(model_type_id)
                .and_then(|t| match t {
                    Type::Model(m) => Some(m.name.clone()),
                    _ => None,
                })
                .unwrap_or_default();
            let message = Self::format_duplicate_property_msg(&model_name, &prop_name);
            self.error("duplicate-property", &message);
            return false;
        }

        // Check override compatibility
        let overridden_prop = self.get_overridden_property_for(prop_type_id);
        if let Some(overridden_id) = overridden_prop {
            let override_info = self.get_type(overridden_id).and_then(|t| match t {
                Type::ModelProperty(p) => Some((p.r#type, p.optional)),
                _ => None,
            });
            if let Some((overridden_type, overridden_optional)) = override_info {
                // Check type assignability
                let is_assignable = {
                    let related = self.type_relation.is_related_with_store(
                        &self.type_store,
                        new_prop_type,
                        overridden_type,
                    );
                    related.is_true()
                };
                if !is_assignable {
                    let parent_type_name = self.type_to_string(overridden_type);
                    let new_type_name = self.type_to_string(new_prop_type);
                    self.error("override-property-mismatch", &format!("Property '{}' has type '{}' which is not assignable to the overridden property type '{}'.", prop_name, new_type_name, parent_type_name));
                    return false;
                }

                // Check that non-optional property isn't overridden as optional
                if !overridden_optional && is_optional {
                    self.error("override-property-mismatch", &format!("Property '{}' cannot be optional because the overridden property is not optional.", prop_name));
                    return false;
                }
            }
        }

        // Add the property to the model
        if let Some(t) = self.get_type_mut(model_type_id)
            && let Type::Model(m) = t
            && !m.properties.contains_key(&prop_name)
        {
            m.properties.insert(prop_name.clone(), prop_type_id);
            m.property_names.push(prop_name);
        }

        true
    }

    /// Find the base property that the given property overrides.
    /// Takes a ModelProperty TypeId, looks up its name and parent model,
    /// then walks the base model chain for the overridden property.
    /// Ported from TS checker getOverriddenProperty().
    pub fn get_overridden_property_for(&self, property_id: TypeId) -> Option<TypeId> {
        let prop = self.get_type(property_id);
        if let Some(Type::ModelProperty(prop)) = prop
            && let Some(model_id) = prop.model
        {
            self.get_overridden_property(model_id, &prop.name)
        } else {
            None
        }
    }

    /// Walk all properties of a model, including inherited ones.
    /// Returns properties from the model itself first, then from base models
    /// (closest base first).
    /// Ported from TS checker walkPropertiesInherited().
    pub fn walk_properties_inherited(&self, model_id: TypeId) -> Vec<TypeId> {
        let mut result = Vec::new();
        let mut visited_names = HashSet::new();
        self.walk_properties_inherited_impl(model_id, &mut result, &mut visited_names);
        result
    }

    pub(crate) fn walk_properties_inherited_impl(
        &self,
        model_id: TypeId,
        result: &mut Vec<TypeId>,
        visited_names: &mut HashSet<String>,
    ) {
        // First walk base model properties
        if let Some(Type::Model(m)) = self.get_type(model_id) {
            if let Some(base_id) = m.base_model {
                self.walk_properties_inherited_impl(base_id, result, visited_names);
            }
            // Then add own properties (not overridden)
            for name in &m.property_names {
                if !visited_names.contains(name)
                    && let Some(&prop_id) = m.properties.get(name)
                {
                    result.push(prop_id);
                    visited_names.insert(name.clone());
                }
            }
        }
    }

    // ========================================================================
    // Indexer
    // ========================================================================

    /// Find the indexer on a model type.
    /// Returns Some((key_type_id, value_type_id)) if the model has an indexer,
    /// or walks the base_model chain if not found directly.
    pub fn find_indexer(&self, type_id: TypeId) -> Option<(TypeId, TypeId)> {
        match self.get_type(type_id) {
            Some(Type::Model(m)) => {
                if let Some(indexer) = m.indexer {
                    return Some(indexer);
                }
                // Walk base_model chain
                if let Some(base_id) = m.base_model {
                    return self.find_indexer(base_id);
                }
                None
            }
            _ => None,
        }
    }

    // ========================================================================
    // Value checking methods
    // ========================================================================

    /// Infer the scalar type for a primitive value given a constraint type.
    /// Ported from TS checker.ts inferScalarForPrimitiveValue.
    /// Returns InferredScalar which includes ambiguity detection for the
    /// `ambiguous-scalar-type` diagnostic.
    pub fn infer_scalar_for_primitive_value(
        &self,
        constraint: Option<TypeId>,
        value_type_id: TypeId,
    ) -> InferredScalar {
        // Get the primitive kind from the value type
        let primitive_name = match self.get_type(value_type_id) {
            Some(Type::String(_)) => "string",
            Some(Type::Number(_)) => "numeric",
            Some(Type::Boolean(_)) => "boolean",
            _ => return InferredScalar::none(),
        };

        // Get the value name for diagnostics
        let value_name = match self.get_type(value_type_id) {
            Some(Type::String(s)) => s.value.clone(),
            Some(Type::Number(n)) => n.value_as_string.clone(),
            Some(Type::Boolean(b)) => b.value.to_string(),
            _ => String::new(),
        };

        // Try to find a matching scalar in the constraint
        if let Some(constraint_id) = constraint {
            // Direct scalar match
            if let Some(Type::Scalar(s)) = self.get_type(constraint_id)
                && self.scalar_matches_primitive(s.name.as_str(), primitive_name)
            {
                return InferredScalar::single(constraint_id);
            }

            // Walk union variants to find a matching scalar
            // Ported from TS checker.ts:4393-4416 — detect ambiguity
            if let Some(Type::Union(u)) = self.get_type(constraint_id) {
                let mut found: Option<TypeId> = None;
                let mut found_name: Option<String> = None;
                for name in &u.variant_names {
                    if let Some(&variant_id) = u.variants.get(name) {
                        // Resolve UnionVariant wrapper if present — union expressions
                        // store variants as UnionVariant(type=actual_type), not bare types
                        let inner_type = match self.get_type(variant_id) {
                            Some(Type::UnionVariant(uv)) => uv.r#type,
                            Some(_) => variant_id, // Already a direct type
                            None => continue,
                        };
                        let resolved_inner = self.resolve_alias_chain(inner_type);
                        if let Some(Type::Scalar(s)) = self.get_type(resolved_inner) {
                            // Check if this scalar derives from the expected primitive.
                            // scalar_matches_primitive checks name only (for builtins like int32),
                            // scalar_extends_primitive_kind checks the full inheritance chain
                            // (for custom scalars like myBool extends boolean)
                            let matches = self
                                .scalar_matches_primitive(s.name.as_str(), primitive_name)
                                || self
                                    .scalar_extends_primitive_kind(resolved_inner, primitive_name);
                            if matches {
                                if found.is_some() {
                                    // Ambiguous: multiple scalars match the same primitive value
                                    let first_name = found_name.as_deref().unwrap_or("unknown");
                                    let type_names = format!("{}, {}", first_name, s.name);
                                    let example_name = first_name.to_string();
                                    return InferredScalar::ambiguous(
                                        value_name,
                                        type_names,
                                        example_name,
                                    );
                                }
                                found = Some(resolved_inner);
                                found_name = Some(s.name.clone());
                            }
                        }
                    }
                }
                if let Some(id) = found {
                    return InferredScalar::single(id);
                }
            }
        }

        // Fallback: return the std scalar for this primitive
        match self.std_types.get(primitive_name).copied() {
            Some(id) => InferredScalar::single(id),
            None => InferredScalar::none(),
        }
    }

    /// Check if a scalar type extends a primitive scalar (string, numeric, boolean)
    /// Ported from TS checker.ts — used for named-init-required / invalid-primitive-init checks
    pub(crate) fn scalar_extends_primitive(&self, type_id: TypeId) -> bool {
        self.scalar_extends_primitive_kind(type_id, "string")
            || self.scalar_extends_primitive_kind(type_id, "numeric")
            || self.scalar_extends_primitive_kind(type_id, "boolean")
    }

    /// Check if a scalar name matches a primitive kind
    pub(crate) fn scalar_matches_primitive(&self, scalar_name: &str, primitive: &str) -> bool {
        crate::std::primitives::scalar_matches_primitive(scalar_name, primitive)
    }

    /// Check if a scalar type extends a primitive kind (string, numeric, boolean)
    /// by walking its base_scalar chain. Used for ambiguous-scalar-type detection
    /// when custom scalars extend primitives (e.g., myBool extends boolean).
    pub(crate) fn scalar_extends_primitive_kind(&self, type_id: TypeId, primitive: &str) -> bool {
        let resolved = self.resolve_alias_chain(type_id);
        match self.get_type(resolved) {
            Some(Type::Scalar(s)) => {
                if self.scalar_matches_primitive(s.name.as_str(), primitive) {
                    return true;
                }
                // Walk base_scalar chain
                s.base_scalar
                    .is_some_and(|base| self.scalar_extends_primitive_kind(base, primitive))
            }
            _ => false,
        }
    }

    /// Check a string literal value and create a StringValue.
    /// Emit ambiguous-scalar-type diagnostic if needed, and return the resolved scalar TypeId.
    pub(crate) fn emit_ambiguous_scalar_diagnostic_and_resolve(
        &mut self,
        constraint: Option<TypeId>,
        value_type_id: TypeId,
    ) -> Option<TypeId> {
        let inferred = self.infer_scalar_for_primitive_value(constraint, value_type_id);
        if let Some((value_name, type_names, example_name)) = inferred.ambiguous {
            self.error("ambiguous-scalar-type", &format!("Value {} type is ambiguous between {}. To resolve be explicit when instantiating this value(e.g. '{}({})').", value_name, type_names, example_name, value_name));
        }
        constraint.or(inferred.scalar)
    }

    /// Returns the ValueId of the created value.
    /// Ported from TS checker.ts createStringValue.
    pub fn check_string_value(
        &mut self,
        value_type_id: TypeId,
        constraint: Option<TypeId>,
        _target_node: NodeId,
    ) -> Option<ValueId> {
        let value = match self.get_type(value_type_id) {
            Some(Type::String(s)) => s.value.clone(),
            _ => return None,
        };
        let scalar = self.emit_ambiguous_scalar_diagnostic_and_resolve(constraint, value_type_id);

        let value_id = self.create_value(Value::StringValue(StringValue {
            type_id: constraint.unwrap_or(value_type_id),
            value,
            scalar,
            node: None,
        }));

        // Track exact type
        self.value_exact_types.insert(value_id, value_type_id);

        Some(value_id)
    }

    /// Check a numeric literal value and create a NumericValue.
    /// Ported from TS checker.ts createNumericValue.
    pub fn check_numeric_value(
        &mut self,
        value_type_id: TypeId,
        constraint: Option<TypeId>,
        _target_node: NodeId,
    ) -> Option<ValueId> {
        let value = match self.get_type(value_type_id) {
            Some(Type::Number(n)) => n.value,
            _ => return None,
        };
        let scalar = self.emit_ambiguous_scalar_diagnostic_and_resolve(constraint, value_type_id);

        let value_id = self.create_value(Value::NumericValue(NumericValue {
            type_id: constraint.unwrap_or(value_type_id),
            value,
            scalar,
            node: None,
        }));

        self.value_exact_types.insert(value_id, value_type_id);

        Some(value_id)
    }

    /// Check a boolean literal value and create a BooleanValue.
    /// Ported from TS checker.ts createBooleanValue.
    pub fn check_boolean_value(
        &mut self,
        value_type_id: TypeId,
        constraint: Option<TypeId>,
        _target_node: NodeId,
    ) -> Option<ValueId> {
        let value = match self.get_type(value_type_id) {
            Some(Type::Boolean(b)) => b.value,
            _ => return None,
        };
        let scalar = self.emit_ambiguous_scalar_diagnostic_and_resolve(constraint, value_type_id);

        let value_id = self.create_value(Value::BooleanValue(BooleanValue {
            type_id: constraint.unwrap_or(value_type_id),
            value,
            scalar,
            node: None,
        }));

        self.value_exact_types.insert(value_id, value_type_id);

        Some(value_id)
    }

    /// Check a null value and create a NullValue.
    /// Ported from TS checker.ts createNullValue.
    pub fn check_null_value(
        &mut self,
        null_type_id: TypeId,
        _constraint: Option<TypeId>,
        _target_node: NodeId,
    ) -> Option<ValueId> {
        let value_id = self.create_value(Value::NullValue(NullValue {
            type_id: null_type_id,
            node: None,
        }));

        self.value_exact_types.insert(value_id, null_type_id);

        Some(value_id)
    }

    /// Get the exact type of a value (the literal type, not the storage/constraint type).
    pub fn get_value_exact_type(&self, value_id: ValueId) -> Option<TypeId> {
        self.value_exact_types.get(&value_id).copied()
    }

    /// Check if a value type matches a constraint type.
    /// Ported from TS checker.ts checkTypeOfValueMatchConstraint.
    pub fn check_type_of_value_match_constraint(
        &mut self,
        value_type: TypeId,
        constraint_type: TypeId,
    ) -> bool {
        let related =
            self.type_relation
                .is_related_with_store(&self.type_store, value_type, constraint_type);
        related.is_true()
    }

    /// Format a duplicate property error message.
    /// Shared by define_property, check_model, and check_spread.
    pub(crate) fn format_duplicate_property_msg(type_name: &str, prop_name: &str) -> String {
        format!(
            "Property '{}' already exists in type '{}'.",
            prop_name, type_name
        )
    }
}
