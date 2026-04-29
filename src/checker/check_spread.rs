//! Spread property resolution
//!
//! Ported from TypeSpec compiler/src/lib/checker.ts spread-related methods

use super::*;

impl Checker {
    // ========================================================================
    // Finish type
    // ========================================================================

    /// Finish a type - mark it as finished
    pub fn finish_type(&mut self, type_id: TypeId) -> TypeId {
        if let Some(t) = self.type_store.get_mut(type_id) {
            t.set_finished(true);
        }

        // Process pending spreads that were waiting for this type to finish.
        // TS: ensureResolved() callback fires when the awaited type finishes.
        if let Some(parent_type_ids) = self.pending_spreads.remove(&type_id) {
            for parent_type_id in parent_type_ids {
                self.resolve_pending_spread(parent_type_id, type_id);
            }
        }

        type_id
    }

    /// Resolve a pending spread: copy properties from the now-finished target model
    /// into the parent model that was waiting for it.
    pub(crate) fn resolve_pending_spread(
        &mut self,
        parent_type_id: TypeId,
        target_type_id: TypeId,
    ) {
        // Resolve through alias chains
        let resolved_target_id = self.resolve_alias_chain(target_type_id);

        let target_type = self.get_type(resolved_target_id).cloned();
        let Some(Type::Model(target_model)) = target_type else {
            return;
        };

        // Don't spread self
        if resolved_target_id == parent_type_id {
            return;
        }

        // Check if target is an array model
        if type_utils::is_array_model_type(&self.type_store, &target_model) {
            return;
        }

        // Get parent model name for diagnostic messages
        let parent_name = self
            .get_type(parent_type_id)
            .and_then(|t| match t {
                Type::Model(m) => Some(m.name.clone()),
                _ => None,
            })
            .unwrap_or_default();

        // Copy properties from the spread target (including inherited)
        let prop_names_to_copy: Vec<(String, TypeId)> = {
            let mut result = Vec::new();
            let mut current: Option<TypeId> = Some(resolved_target_id);
            while let Some(mid) = current {
                if let Some(Type::Model(cur_model)) = self.get_type(mid) {
                    for (name, &prop_id) in &cur_model.properties {
                        result.push((name.clone(), prop_id));
                    }
                    current = cur_model.base_model;
                } else {
                    break;
                }
            }
            result
        };

        // Track which properties are new (not already in parent)
        let mut new_prop_names: Vec<String> = Vec::new();

        for (prop_name, src_prop_id) in prop_names_to_copy {
            // Check for duplicate
            if let Some(t) = self.get_type(parent_type_id)
                && let Type::Model(m) = t
                && m.properties.contains_key(&prop_name)
            {
                self.error(
                    "duplicate-property",
                    &Self::format_duplicate_property_msg(&parent_name, &prop_name),
                );
                continue;
            }
            let cloned_id = self.clone_type(src_prop_id);
            if let Some(prop) = self.get_type_mut(cloned_id)
                && let Type::ModelProperty(p) = prop
            {
                p.source_property = Some(src_prop_id);
                p.model = Some(parent_type_id);
            }
            if let Some(t) = self.get_type_mut(parent_type_id)
                && let Type::Model(m) = t
            {
                m.properties.insert(prop_name.clone(), cloned_id);
                m.property_names.push(prop_name.clone());
                new_prop_names.push(prop_name);
            }
        }

        // Copy indexer if present
        if let Some((key_id, value_id)) = target_model.indexer
            && let Some(t) = self.get_type_mut(parent_type_id)
            && let Type::Model(m) = t
        {
            m.indexer = Some((key_id, value_id));
        }

        // Record the spread source relationship for cascade propagation
        self.spread_sources
            .entry(resolved_target_id)
            .or_default()
            .push(parent_type_id);

        // Cascade: propagate new properties to types that already spread from parent_type_id
        if !new_prop_names.is_empty() {
            self.cascade_spread_properties(parent_type_id, &new_prop_names);
        }
    }

    /// Cascade new properties to types that previously spread from the given source.
    /// When a source type gains new properties (e.g., from a resolved pending spread),
    /// any type that already spread from it needs those new properties too.
    pub(crate) fn cascade_spread_properties(
        &mut self,
        source_type_id: TypeId,
        new_prop_names: &[String],
    ) {
        // Find types that spread from source_type_id
        let dependents = self
            .spread_sources
            .get(&source_type_id)
            .cloned()
            .unwrap_or_default();

        if dependents.is_empty() {
            return;
        }

        // For each dependent, copy the new properties
        for dependent_id in dependents {
            for prop_name in new_prop_names {
                // Get the source property from the source model
                let src_prop_id = match self.get_type(source_type_id) {
                    Some(Type::Model(m)) => m.properties.get(prop_name).copied(),
                    _ => None,
                };
                let Some(src_prop_id) = src_prop_id else {
                    continue;
                };

                // Check for duplicate
                if let Some(t) = self.get_type(dependent_id)
                    && let Type::Model(m) = t
                    && m.properties.contains_key(prop_name)
                {
                    continue; // Already have this property, skip
                }

                let cloned_id = self.clone_type(src_prop_id);
                if let Some(prop) = self.get_type_mut(cloned_id)
                    && let Type::ModelProperty(p) = prop
                {
                    p.source_property = Some(src_prop_id);
                    p.model = Some(dependent_id);
                }
                if let Some(t) = self.get_type_mut(dependent_id)
                    && let Type::Model(m) = t
                {
                    m.properties.insert(prop_name.clone(), cloned_id);
                    m.property_names.push(prop_name.clone());
                }
            }

            // Recursively cascade to dependents of this dependent
            self.cascade_spread_properties(dependent_id, new_prop_names);
        }
    }
}
