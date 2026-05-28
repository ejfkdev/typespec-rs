//! Public helper methods for external consumers
//!
//! Convenience methods on Checker for common operations like finding decorators,
//! extracting constraint values, resolving types by name, etc.

use super::*;

impl Checker {
    // ========================================================================
    // Decorator lookup helpers
    // ========================================================================

    /// Find a decorator application on a type by name.
    ///
    /// Searches the type's own decorators. For operations, also searches
    /// inherited decorators via `get_effective_decorators()`.
    ///
    /// # Example
    /// ```ignore
    /// if let Some(dec) = checker.find_decorator(prop_id, "minValue") {
    ///     // found @minValue decorator
    /// }
    /// ```
    pub fn find_decorator(&self, type_id: TypeId, name: &str) -> Option<&DecoratorApplication> {
        // For operations, use get_effective_decorators (includes interface decorators)
        // For all other types, use their own decorators directly
        let decorators: Vec<&DecoratorApplication> = match self.get_type(type_id) {
            Some(Type::Operation(_)) => self.get_effective_decorators(type_id),
            Some(t) => t
                .decorators()
                .map(|d| d.iter().collect())
                .unwrap_or_default(),
            None => return None,
        };
        decorators
            .into_iter()
            .find(|dec| self.decorator_matches_name(dec, name))
    }

    /// Find all decorator applications on a type by name.
    pub fn find_decorators(&self, type_id: TypeId, name: &str) -> Vec<&DecoratorApplication> {
        let decorators: Vec<&DecoratorApplication> = match self.get_type(type_id) {
            Some(Type::Operation(_)) => self.get_effective_decorators(type_id),
            Some(t) => t
                .decorators()
                .map(|d| d.iter().collect())
                .unwrap_or_default(),
            None => return Vec::new(),
        };
        decorators
            .into_iter()
            .filter(|dec| self.decorator_matches_name(dec, name))
            .collect()
    }

    /// Get a string argument from a decorator by index.
    /// Returns None if the decorator isn't found or the argument isn't a string.
    pub fn get_decorator_string_arg(
        &self,
        type_id: TypeId,
        name: &str,
        arg_index: usize,
    ) -> Option<String> {
        let dec = self.find_decorator(type_id, name)?;
        let arg = dec.args.get(arg_index)?;
        match &arg.js_value {
            Some(DecoratorMarshalledValue::String(s)) => Some(s.clone()),
            _ => None,
        }
    }

    /// Get a numeric argument from a decorator by index.
    pub fn get_decorator_numeric_arg(
        &self,
        type_id: TypeId,
        name: &str,
        arg_index: usize,
    ) -> Option<f64> {
        let dec = self.find_decorator(type_id, name)?;
        let arg = dec.args.get(arg_index)?;
        match &arg.js_value {
            Some(DecoratorMarshalledValue::Number(n)) => Some(*n),
            _ => None,
        }
    }

    /// Get a boolean argument from a decorator by index.
    pub fn get_decorator_bool_arg(
        &self,
        type_id: TypeId,
        name: &str,
        arg_index: usize,
    ) -> Option<bool> {
        let dec = self.find_decorator(type_id, name)?;
        let arg = dec.args.get(arg_index)?;
        match &arg.js_value {
            Some(DecoratorMarshalledValue::Boolean(b)) => Some(*b),
            _ => None,
        }
    }

    /// Check if a decorator application matches a given name.
    /// Handles both simple names ("doc") and qualified names ("TypeSpec.doc").
    fn decorator_matches_name(&self, dec: &DecoratorApplication, name: &str) -> bool {
        // Try via the decorator definition (DecoratorType has a name field)
        if let Some(def_id) = dec.definition
            && let Some(Type::Decorator(dt)) = self.get_type(def_id)
            && dt.name == name
        {
            return true;
        }

        // Fallback: try matching via the AST decorator node name
        if let Some(ast) = self.ast.as_ref()
            && let Some(AstNode::DecoratorExpression(expr)) = ast.id_to_node(dec.decorator)
        {
            let full_name = Self::get_identifier_name(ast, expr.target);
            if full_name == name || full_name.ends_with(&format!(".{}", name)) {
                return true;
            }
        }

        false
    }

    // ========================================================================
    // Value extraction helpers
    // ========================================================================

    /// Extract the default value of a ModelProperty as a string representation.
    ///
    /// ModelPropertyType.default_value is a TypeId pointing to a literal type
    /// (String, Number, Boolean). This method resolves it to a human-readable
    /// string value.
    pub fn extract_default_value(&self, type_id: TypeId) -> Option<String> {
        let dv_id = match self.get_type(type_id) {
            Some(Type::ModelProperty(p)) => p.default_value?,
            _ => return None,
        };

        match self.get_type(dv_id) {
            Some(Type::String(s)) => Some(s.value.clone()),
            Some(Type::Number(n)) => Some(n.value_as_string.clone()),
            Some(Type::Boolean(b)) => Some(b.value.to_string()),
            Some(Type::Intrinsic(i)) if i.name == IntrinsicTypeName::Null => {
                Some("null".to_string())
            }
            _ => None,
        }
    }

    /// Extract the value of an EnumMember as a string representation.
    ///
    /// EnumMemberType.value is a TypeId pointing to a String or Numeric type.
    pub fn extract_enum_member_value(&self, type_id: TypeId) -> Option<String> {
        let value_id = match self.get_type(type_id) {
            Some(Type::EnumMember(m)) => m.value?,
            _ => return None,
        };

        match self.get_type(value_id) {
            Some(Type::String(s)) => Some(s.value.clone()),
            Some(Type::Number(n)) => Some(n.value_as_string.clone()),
            _ => None,
        }
    }

    // ========================================================================
    // Type lookup helpers
    // ========================================================================

    /// Resolve a type by its fully qualified name (e.g., "MyNamespace.MyModel").
    ///
    /// Walks the namespace hierarchy to find the type. Returns None if not found.
    /// For simple names (no dots), also checks `declared_types`.
    pub fn lookup_type_by_fqn(&self, fqn: &str) -> Option<TypeId> {
        let parts: Vec<&str> = fqn.split('.').collect();
        if parts.is_empty() {
            return None;
        }

        // Start from the global namespace
        let global_ns = self.global_namespace_type?;

        if parts.len() == 1 {
            // Simple name — look in declared_types (FQN-aware)
            return self.resolve_declared_name(parts[0]);
        }

        // Qualified name — walk namespace hierarchy
        let mut current_ns = global_ns;
        for (i, part) in parts.iter().enumerate() {
            if i == parts.len() - 1 {
                // Last part — look for the type in the namespace
                return self.find_type_in_namespace(current_ns, part);
            } else {
                // Intermediate part — navigate to child namespace
                current_ns = self.find_child_namespace(current_ns, part)?;
            }
        }

        None
    }

    /// Find a type by name within a namespace.
    fn find_type_in_namespace(&self, ns_id: TypeId, name: &str) -> Option<TypeId> {
        match self.get_type(ns_id) {
            Some(Type::Namespace(ns)) => {
                // Check models
                if let Some(&id) = ns.models.get(name) {
                    return Some(id);
                }
                // Check operations
                if let Some(&id) = ns.operations.get(name) {
                    return Some(id);
                }
                // Check interfaces
                if let Some(&id) = ns.interfaces.get(name) {
                    return Some(id);
                }
                // Check enums
                if let Some(&id) = ns.enums.get(name) {
                    return Some(id);
                }
                // Check unions
                if let Some(&id) = ns.unions.get(name) {
                    return Some(id);
                }
                // Check scalars
                if let Some(&id) = ns.scalars.get(name) {
                    return Some(id);
                }
                // Check child namespaces
                if let Some(&id) = ns.namespaces.get(name) {
                    return Some(id);
                }
                None
            }
            _ => None,
        }
    }

    /// Find a child namespace by name within a parent namespace.
    fn find_child_namespace(&self, ns_id: TypeId, name: &str) -> Option<TypeId> {
        match self.get_type(ns_id) {
            Some(Type::Namespace(ns)) => ns.namespaces.get(name).copied(),
            _ => None,
        }
    }

    // ========================================================================
    // Type iteration helpers
    // ========================================================================

    /// Iterate all types in the type store.
    /// Returns an iterator of (TypeId, &Type) pairs.
    pub fn iter_types(&self) -> impl Iterator<Item = (TypeId, &Type)> {
        self.type_store.iter()
    }

    /// Iterate all models in the type store.
    pub fn iter_models(&self) -> impl Iterator<Item = (TypeId, &ModelType)> {
        self.type_store.iter().filter_map(|(id, t)| {
            if let Type::Model(m) = t {
                Some((id, m))
            } else {
                None
            }
        })
    }

    /// Iterate all operations in the type store.
    pub fn iter_operations(&self) -> impl Iterator<Item = (TypeId, &OperationType)> {
        self.type_store.iter().filter_map(|(id, t)| {
            if let Type::Operation(o) = t {
                Some((id, o))
            } else {
                None
            }
        })
    }

    /// Iterate all enums in the type store.
    pub fn iter_enums(&self) -> impl Iterator<Item = (TypeId, &EnumType)> {
        self.type_store.iter().filter_map(|(id, t)| {
            if let Type::Enum(e) = t {
                Some((id, e))
            } else {
                None
            }
        })
    }

    /// Iterate all interfaces in the type store.
    pub fn iter_interfaces(&self) -> impl Iterator<Item = (TypeId, &InterfaceType)> {
        self.type_store.iter().filter_map(|(id, t)| {
            if let Type::Interface(i) = t {
                Some((id, i))
            } else {
                None
            }
        })
    }

    /// Iterate all namespaces in the type store.
    pub fn iter_namespaces(&self) -> impl Iterator<Item = (TypeId, &NamespaceType)> {
        self.type_store.iter().filter_map(|(id, t)| {
            if let Type::Namespace(ns) = t {
                Some((id, ns.as_ref()))
            } else {
                None
            }
        })
    }

    // ========================================================================
    // Constraint accessor helpers
    // ========================================================================

    /// Get the @minValue constraint for a type as f64.
    pub fn get_type_min_value(&self, type_id: TypeId) -> Option<f64> {
        crate::intrinsic_type_state::get_min_value(&self.state_accessors, type_id)
    }

    /// Get the @maxValue constraint for a type as f64.
    pub fn get_type_max_value(&self, type_id: TypeId) -> Option<f64> {
        crate::intrinsic_type_state::get_max_value(&self.state_accessors, type_id)
    }

    /// Get the @minValueExclusive constraint for a type as f64.
    pub fn get_type_min_value_exclusive(&self, type_id: TypeId) -> Option<f64> {
        crate::intrinsic_type_state::get_min_value_exclusive(&self.state_accessors, type_id)
    }

    /// Get the @maxValueExclusive constraint for a type as f64.
    pub fn get_type_max_value_exclusive(&self, type_id: TypeId) -> Option<f64> {
        crate::intrinsic_type_state::get_max_value_exclusive(&self.state_accessors, type_id)
    }

    /// Get the @minLength constraint for a type as f64.
    pub fn get_type_min_length(&self, type_id: TypeId) -> Option<f64> {
        crate::intrinsic_type_state::get_min_length(&self.state_accessors, type_id)
    }

    /// Get the @maxLength constraint for a type as f64.
    pub fn get_type_max_length(&self, type_id: TypeId) -> Option<f64> {
        crate::intrinsic_type_state::get_max_length(&self.state_accessors, type_id)
    }

    /// Get the @minItems constraint for a type as f64.
    pub fn get_type_min_items(&self, type_id: TypeId) -> Option<f64> {
        crate::intrinsic_type_state::get_min_items(&self.state_accessors, type_id)
    }

    /// Get the @maxItems constraint for a type as f64.
    pub fn get_type_max_items(&self, type_id: TypeId) -> Option<f64> {
        crate::intrinsic_type_state::get_max_items(&self.state_accessors, type_id)
    }

    /// Get the @doc documentation for a type.
    pub fn get_type_doc(&self, type_id: TypeId) -> Option<String> {
        crate::intrinsic_type_state::get_doc(&self.state_accessors, type_id)
    }

    /// Get the @summary for a type.
    pub fn get_type_summary(&self, type_id: TypeId) -> Option<String> {
        crate::libs::compiler::get_summary(&self.state_accessors, type_id)
    }

    /// Check if a type is marked with @error.
    pub fn is_type_error(&self, type_id: TypeId) -> bool {
        crate::libs::compiler::is_error(&self.state_accessors, type_id)
    }

    /// Get the @pattern constraint for a type.
    pub fn get_type_pattern(&self, type_id: TypeId) -> Option<String> {
        crate::libs::compiler::get_pattern(&self.state_accessors, type_id)
    }

    /// Get the @format constraint for a type.
    pub fn get_type_format(&self, type_id: TypeId) -> Option<String> {
        crate::libs::compiler::get_format(&self.state_accessors, type_id)
    }

    /// Get the @discriminator for a type.
    pub fn get_type_discriminator(
        &self,
        type_id: TypeId,
    ) -> Option<crate::intrinsic_type_state::Discriminator> {
        crate::intrinsic_type_state::get_discriminator(&self.state_accessors, type_id)
    }

    // ========================================================================
    // Property access helpers
    // ========================================================================

    /// Get a model property by name from a model type.
    /// Returns the TypeId of the ModelPropertyType, or None if not found.
    pub fn get_model_property(&self, model_id: TypeId, property_name: &str) -> Option<TypeId> {
        match self.get_type(model_id) {
            Some(Type::Model(m)) => m.properties.get(property_name).copied(),
            _ => None,
        }
    }

    /// Walk all properties of a model, including inherited ones.
    /// Returns a Vec of (property_name, property_type_id) pairs.
    /// Properties from base models appear before derived model properties.
    pub fn walk_model_properties(&self, model_id: TypeId) -> Vec<(String, TypeId)> {
        let mut result = Vec::new();
        let mut seen = std::collections::HashSet::new();
        self.walk_model_properties_inner(model_id, &mut result, &mut seen);
        result
    }

    fn walk_model_properties_inner(
        &self,
        model_id: TypeId,
        result: &mut Vec<(String, TypeId)>,
        seen: &mut std::collections::HashSet<String>,
    ) {
        let (properties, base_model) = match self.get_type(model_id) {
            Some(Type::Model(m)) => {
                let props: Vec<_> = m
                    .property_names
                    .iter()
                    .filter_map(|n| m.properties.get(n).map(|&id| (n.clone(), id)))
                    .collect();
                (props, m.base_model)
            }
            _ => return,
        };

        // Walk base model first (inherited properties come first)
        if let Some(base) = base_model {
            self.walk_model_properties_inner(base, result, seen);
        }

        // Add own properties (skip already seen from base)
        for (name, id) in properties {
            if seen.insert(name.clone()) {
                result.push((name, id));
            }
        }
    }

    /// Get the type that a ModelProperty points to, resolving aliases.
    pub fn get_property_type(&self, prop_id: TypeId) -> TypeId {
        match self.get_type(prop_id) {
            Some(Type::ModelProperty(p)) => self.resolve_alias_chain(p.r#type),
            _ => self.error_type,
        }
    }
}
