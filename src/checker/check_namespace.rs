use super::*;

impl Checker {
    pub(crate) fn check_namespace(&mut self, ctx: &CheckContext, node_id: NodeId) -> TypeId {
        let (ast, node) = require_ast_node!(self, node_id, NamespaceDeclaration, self.error_type);

        let full_name = Self::get_identifier_name(&ast, node.name);

        if let Some(&type_id) = self.node_type_map.get(&node_id)
            && let Some(t) = self.get_type(type_id)
            && t.is_finished()
        {
            return type_id;
        }

        // Check if this namespace was merged with an existing one during pre-registration
        // (namespace merging: same namespace name in different blocks reuses the existing type)
        let type_id = if let Some(&existing_id) = self.node_type_map.get(&node_id) {
            // Reuse the pre-registered (merged) namespace type
            existing_id
        } else {
            let new_id = self.create_type(Type::Namespace(Box::new(NamespaceType::new(
                self.next_type_id(),
                full_name.clone(),
                Some(node_id),
                self.current_namespace,
                false,
            ))));

            self.register_type(node_id, new_id, &full_name, ctx.mapper.as_ref());
            new_id
        };

        // For dotted namespace names like A.B, ensure parent namespaces exist
        // and register this namespace as a child of its immediate parent
        let immediate_name = Self::get_last_identifier_name(&ast, node.name);
        let parent_ns_id = self.ensure_parent_namespaces(&ast, node.name, self.current_namespace);
        if let Some(parent_id) = parent_ns_id {
            // Register this namespace as a child of the parent namespace
            // Always update (overwrite the placeholder from ensure_parent_namespaces)
            if let Some(t) = self.get_type_mut(parent_id)
                && let Type::Namespace(ns) = t
            {
                if !ns.namespaces.contains_key(&immediate_name) {
                    ns.namespace_names.push(immediate_name.clone());
                }
                ns.namespaces.insert(immediate_name, type_id);
            }
            // Update this namespace's parent reference
            if let Some(t) = self.get_type_mut(type_id)
                && let Type::Namespace(ns) = t
            {
                ns.namespace = Some(parent_id);
            }
        }

        let prev_namespace = self.current_namespace;
        self.current_namespace = Some(type_id);

        // Process directives (e.g., #deprecated) early so deprecated context works
        self.process_and_mark_directives(node_id, type_id);

        for &stmt_id in &node.statements {
            self.check_node(ctx, stmt_id);
        }

        // Register newly declared types into the namespace
        self.populate_namespace(type_id);

        self.current_namespace = prev_namespace;

        self.check_and_store_decorators(ctx, type_id, &node.decorators);

        self.finish_type(type_id);
        type_id
    }
}
