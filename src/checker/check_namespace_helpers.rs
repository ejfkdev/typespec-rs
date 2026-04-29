//! Namespace helper methods
//!
//! Ported from TypeSpec compiler namespace helper methods

use super::*;

impl Checker {
    // ========================================================================
    // Helper: populate namespace with child types
    // ========================================================================

    /// Walk all types in node_type_map and register those whose namespace matches
    /// the given namespace TypeId into the namespace's type collections.
    pub fn populate_namespace(&mut self, type_id: TypeId) {
        // Collect matching children first (need immutable borrow of type_store)
        let children: Vec<(String, TypeId, &'static str)> = self
            .node_type_map
            .iter()
            .filter_map(|(&_node_id, &child_type_id)| {
                let child_type = self.get_type(child_type_id)?;
                if child_type.namespace() != Some(type_id) {
                    return None;
                }
                let name = child_type.name()?.to_string();
                if name.is_empty() {
                    return None;
                }
                let kind = child_type.kind_name();
                Some((name, child_type_id, kind))
            })
            .collect();

        // Now insert into namespace (mutable borrow)
        for (child_name, child_type_id, kind) in children {
            if let Some(Type::Namespace(ns)) = self.get_type_mut(type_id) {
                let (map, names) = match kind {
                    "Model" => (&mut ns.models, &mut ns.model_names),
                    "Scalar" => (&mut ns.scalars, &mut ns.scalar_names),
                    "Interface" => (&mut ns.interfaces, &mut ns.interface_names),
                    "Operation" => (&mut ns.operations, &mut ns.operation_names),
                    "Enum" => (&mut ns.enums, &mut ns.enum_names),
                    "Union" => (&mut ns.unions, &mut ns.union_names),
                    "Namespace" => (&mut ns.namespaces, &mut ns.namespace_names),
                    "Decorator" => (
                        &mut ns.decorator_declarations,
                        &mut ns.decorator_declaration_names,
                    ),
                    "FunctionType" => (
                        &mut ns.function_declarations,
                        &mut ns.function_declaration_names,
                    ),
                    _ => continue,
                };
                if map.insert(child_name.clone(), child_type_id).is_none() {
                    names.push(child_name);
                }
            }
        }
    }

    // ========================================================================
    // Helper: circular base detection
    // ========================================================================

    /// Check if adding target_id as a base of type_id would create a circular chain
    pub fn is_circular_base(&self, type_id: TypeId, target_id: TypeId) -> bool {
        let mut current = Some(target_id);
        while let Some(cur_id) = current {
            if cur_id == type_id {
                return true;
            }
            match self.get_type(cur_id) {
                Some(Type::Model(m)) => {
                    // Check both base_model (extends) and source_model (is)
                    current = m.base_model.or(m.source_model);
                }
                _ => break,
            }
        }
        false
    }

    // ========================================================================
    // Helper: get identifier name
    // ========================================================================

    /// Get the text of an identifier node (also handles MemberExpression for dotted names)
    pub fn get_identifier_name(ast: &AstBuilder, id: NodeId) -> String {
        match ast.id_to_node(id) {
            Some(AstNode::Identifier(ident)) => ident.value.clone(),
            Some(AstNode::MemberExpression(expr)) => {
                let base = Self::get_identifier_name(ast, expr.object);
                let prop = Self::get_identifier_name(ast, expr.property);
                if base.is_empty() {
                    prop
                } else if prop.is_empty() {
                    base
                } else {
                    format!("{}.{}", base, prop)
                }
            }
            _ => String::new(),
        }
    }

    /// Simple type compatibility check for decorator argument validation
    /// Returns true if arg_type is assignable to expected_type
    /// Get only the last segment of a dotted name (e.g., "B" from "A.B")
    pub fn get_last_identifier_name(ast: &AstBuilder, id: NodeId) -> String {
        match ast.id_to_node(id) {
            Some(AstNode::Identifier(ident)) => ident.value.clone(),
            Some(AstNode::MemberExpression(expr)) => {
                Self::get_last_identifier_name(ast, expr.property)
            }
            _ => String::new(),
        }
    }

    /// Ensure parent namespaces exist for dotted names like A.B
    /// For A.B, ensures namespace A exists and returns its TypeId.
    /// For simple names like Foo, returns None (no parent needed).
    pub fn ensure_parent_namespaces(
        &mut self,
        ast: &AstBuilder,
        name_id: NodeId,
        current_parent: Option<TypeId>,
    ) -> Option<TypeId> {
        match ast.id_to_node(name_id) {
            Some(AstNode::Identifier(_)) => {
                // Simple name - no parent namespace to create
                None
            }
            Some(AstNode::MemberExpression(expr)) => {
                // Dotted name like A.B - ensure A exists first
                let base_id = self.ensure_parent_namespaces(ast, expr.object, current_parent);
                let prop_name = Self::get_identifier_name(ast, expr.property);

                // If the base is a MemberExpression, base_id is the immediate parent
                // If the base is a simple Identifier, we need to find/create it
                let parent_id = if let Some(id) = base_id {
                    id
                } else {
                    // Base is a simple identifier - look it up or create it
                    let base_name = Self::get_identifier_name(ast, expr.object);
                    if let Some(&existing_id) = self.declared_types.get(&base_name) {
                        existing_id
                    } else {
                        // Create the parent namespace
                        let new_id =
                            self.create_type(Type::Namespace(Box::new(NamespaceType::new(
                                self.next_type_id(),
                                base_name.clone(),
                                None,
                                current_parent,
                                true,
                            ))));
                        self.declared_types.insert(base_name.clone(), new_id);

                        // Register in global namespace
                        if let Some(global_ns) = self.global_namespace_type
                            && let Some(t) = self.get_type_mut(global_ns)
                            && let Type::Namespace(ns) = t
                        {
                            if !ns.namespaces.contains_key(&base_name) {
                                ns.namespace_names.push(base_name.clone());
                            }
                            ns.namespaces.entry(base_name).or_insert(new_id);
                        }
                        new_id
                    }
                };

                // Register prop_name as sub-namespace of parent
                if !prop_name.is_empty() {
                    let err_type = self.error_type;
                    if let Some(t) = self.get_type_mut(parent_id)
                        && let Type::Namespace(ns) = t
                    {
                        // Don't overwrite if already exists
                        if !ns.namespaces.contains_key(&prop_name) {
                            ns.namespace_names.push(prop_name.clone());
                        }
                        ns.namespaces.entry(prop_name).or_insert(err_type);
                    }
                }

                Some(parent_id)
            }
            _ => None,
        }
    }
}
