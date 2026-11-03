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
    ///
    /// Members are registered in declaration order: entries keyed by a type's
    /// own declaration node come first, sorted by node id, then any others.
    pub fn populate_namespace(&mut self, type_id: TypeId) {
        // Collect matching children first (need immutable borrow of type_store)
        let mut children: Vec<(bool, NodeId, String, TypeId, &'static str)> = self
            .node_type_map
            .iter()
            .filter_map(|(&node_id, &child_type_id)| {
                let child_type = self.get_type(child_type_id)?;
                if child_type.namespace() != Some(type_id) {
                    return None;
                }
                let name = child_type.name()?.to_string();
                if name.is_empty() {
                    return None;
                }
                let kind = child_type.kind_name();
                let is_declaration = declaration_node(child_type) == Some(node_id);
                Some((!is_declaration, node_id, name, child_type_id, kind))
            })
            .collect();
        children.sort_by_key(|&(later, node_id, ..)| (later, node_id));
        let children = children
            .into_iter()
            .map(|(_, _, name, id, kind)| (name, id, kind));

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

    /// Ensure the parent namespaces of a dotted name like `A.B.C` exist,
    /// relative to the current namespace, and return the immediate parent
    /// (`A.B`). A simple name has no parent to create and returns `None`.
    pub fn ensure_parent_namespaces(
        &mut self,
        ast: &AstBuilder,
        name_id: NodeId,
    ) -> Option<TypeId> {
        match ast.id_to_node(name_id) {
            Some(AstNode::MemberExpression(expr)) => self.ensure_namespace_path(ast, expr.object),
            _ => None,
        }
    }

    /// Find or create the namespace a (possibly dotted) name refers to,
    /// relative to the current namespace. Each segment becomes its own
    /// namespace, named by that segment and parented to the one before it.
    fn ensure_namespace_path(&mut self, ast: &AstBuilder, name_id: NodeId) -> Option<TypeId> {
        let (parent, segment) = match ast.id_to_node(name_id) {
            Some(AstNode::Identifier(ident)) => (self.current_namespace, ident.value.clone()),
            Some(AstNode::MemberExpression(expr)) => (
                Some(self.ensure_namespace_path(ast, expr.object)?),
                Self::get_identifier_name(ast, expr.property),
            ),
            _ => return None,
        };
        if segment.is_empty() {
            return parent;
        }

        let prev_ns = self.current_namespace;
        self.current_namespace = parent;
        let existing = self
            .get_declared_type_in_scope(&segment)
            .filter(|&id| matches!(self.get_type(id), Some(Type::Namespace(_))));
        let ns_id = existing.unwrap_or_else(|| {
            let id = self.create_type(Type::Namespace(Box::new(NamespaceType::new(
                self.next_type_id(),
                segment.clone(),
                None,
                parent,
                true,
            ))));
            self.register_declared_type(&segment, id);
            id
        });
        self.current_namespace = prev_ns;

        if let Some(parent_id) = parent.or(self.global_namespace_type)
            && let Some(Type::Namespace(ns)) = self.get_type_mut(parent_id)
            && !ns.namespaces.contains_key(&segment)
        {
            ns.namespace_names.push(segment.clone());
            ns.namespaces.insert(segment, ns_id);
        }
        Some(ns_id)
    }
}

/// The node a namespace member was declared by.
fn declaration_node(t: &Type) -> Option<NodeId> {
    match t {
        Type::Model(t) => t.node,
        Type::Scalar(t) => t.node,
        Type::Interface(t) => t.node,
        Type::Operation(t) => t.node,
        Type::Enum(t) => t.node,
        Type::Union(t) => t.node,
        Type::Namespace(t) => t.node,
        Type::Decorator(t) => t.node,
        Type::FunctionType(t) => t.node,
        _ => None,
    }
}
