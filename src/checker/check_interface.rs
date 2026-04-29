use super::*;
use crate::parser::AstNode;

impl Checker {
    pub(crate) fn check_interface(&mut self, ctx: &CheckContext, node_id: NodeId) -> TypeId {
        // Circular reference detection: if we're already checking this node, return
        // the existing type (even if unfinished) to break the cycle.
        if let Some(type_id) = self.check_circular_ref(node_id) {
            return type_id;
        }

        let (ast, node) = require_ast_node!(self, node_id, InterfaceDeclaration, self.error_type);

        let name = Self::get_identifier_name(&ast, node.name);

        self.check_template_declaration(ctx, &ast, &node.template_parameters);

        // Mark this node as currently being type-checked (for circular detection)
        // Insert AFTER all early returns to avoid leaking entries in pending_type_checks.
        self.pending_type_checks.insert(node_id);

        let mut extends = Vec::new();
        let mut extends_type_ids = Vec::new();
        let mut resolved_extends_ids = Vec::new();
        for &extends_id in &node.extends {
            let extends_type = self.check_node(ctx, extends_id);
            extends_type_ids.push(extends_type);

            // Check that extended type is an interface
            if extends_type != self.error_type {
                let resolved = self.resolve_alias_chain(extends_type);
                resolved_extends_ids.push(resolved);
                match self.get_type(resolved) {
                    Some(Type::Interface(_)) => {
                        extends.push(extends_type);
                    }
                    Some(_) => {
                        self.error(
                            "extends-interface",
                            &format!("Interface '{}' can only extend other interfaces.", name),
                        );
                    }
                    None => {}
                }
            } else {
                resolved_extends_ids.push(self.error_type);
            }
        }

        let template_node =
            self.compute_template_node(&node.template_parameters, ctx.mapper.as_ref(), node_id);

        // Use pre-registered type if available, otherwise create new
        let type_id = if let Some(&existing_id) = self.node_type_map.get(&node_id) {
            // Update pre-registered type in-place
            if let Some(t) = self.get_type_mut(existing_id)
                && let Type::Interface(i) = t
            {
                i.extends = extends;
                i.template_node = template_node;
            }
            existing_id
        } else {
            let new_id = {
                let mut i = InterfaceType::new(
                    self.next_type_id(),
                    name.clone(),
                    Some(node_id),
                    self.current_namespace,
                );
                i.extends = extends;
                i.template_node = template_node;
                self.create_type(Type::Interface(i))
            };

            self.register_type(node_id, new_id, &name, ctx.mapper.as_ref());
            new_id
        };

        // Process directives (e.g., #deprecated) BEFORE checking operations,
        // so that the type is marked as deprecated when checking operation type references.
        self.process_and_mark_directives(node_id, type_id);

        let mut operations = HashMap::new();
        let mut operation_names = Vec::new();

        // Copy operations from extended interfaces
        // TS: checkInterface → for each sourceInterface → clone operations
        for (i, &extends_type_id) in extends_type_ids.iter().enumerate() {
            if extends_type_id == self.error_type {
                continue;
            }
            let resolved_extends = resolved_extends_ids[i];
            if let Some(Type::Interface(ext_iface)) = self.get_type(resolved_extends).cloned() {
                // Track source interfaces
                if let Some(t) = self.get_type_mut(type_id)
                    && let Type::Interface(i) = t
                    && !i.source_interfaces.contains(&extends_type_id)
                {
                    i.source_interfaces.push(extends_type_id);
                }
                // Copy operations from extended interface
                for ext_op_name in &ext_iface.operation_names {
                    if let Some(&ext_op_id) = ext_iface.operations.get(ext_op_name) {
                        // Only add if not already defined (own operations override)
                        if !operations.contains_key(ext_op_name) {
                            let cloned_op_id = self.clone_type(ext_op_id);
                            // Re-parent cloned operation to this interface
                            if let Some(o) = self.get_type_mut(cloned_op_id)
                                && let Type::Operation(op) = o
                            {
                                op.interface_ = Some(type_id);
                            }
                            operations.insert(ext_op_name.clone(), cloned_op_id);
                            operation_names.push(ext_op_name.clone());
                        }
                    }
                }
            }
        }

        // Check own operations (these override inherited ones)
        let mut own_op_names_seen = HashSet::new();
        for &op_id in &node.operations {
            let op_node = match ast.id_to_node(op_id) {
                Some(AstNode::OperationDeclaration(op)) => op.clone(),
                _ => continue,
            };

            let op_name = Self::get_identifier_name(&ast, op_node.name);

            if own_op_names_seen.contains(&op_name) {
                self.error(
                    "interface-duplicate",
                    &format!("Interface already has an operation named '{}'.", op_name),
                );
                // Skip to avoid map/names mismatch
                continue;
            }
            own_op_names_seen.insert(op_name.clone());

            let op_type_id = self.check_operation_internal(ctx, op_id, Some(type_id));

            operations.insert(op_name.clone(), op_type_id);
            operation_names.push(op_name);
        }

        // Check for duplicate member names across extended interfaces
        // TS: "doesn't allow extensions to contain duplicate members"
        let mut inherited_op_sources: HashMap<String, TypeId> = HashMap::new();
        for (i, &extends_type_id) in extends_type_ids.iter().enumerate() {
            if extends_type_id == self.error_type {
                continue;
            }
            let resolved_extends = resolved_extends_ids[i];
            if let Some(Type::Interface(ext_iface)) = self.get_type(resolved_extends).cloned() {
                for ext_op_name in &ext_iface.operation_names {
                    if ext_iface.operations.contains_key(ext_op_name) {
                        // Only report duplicate if this operation is NOT overridden by own operations
                        if !own_op_names_seen.contains(ext_op_name) {
                            if let Some(&prev_source) = inherited_op_sources.get(ext_op_name)
                                && prev_source != extends_type_id
                            {
                                self.error("extends-interface-duplicate", &format!("Interface '{}' has duplicate member '{}' from multiple extended interfaces.", name, ext_op_name));
                                break; // Only report once
                            } else {
                                inherited_op_sources.insert(ext_op_name.clone(), extends_type_id);
                            }
                        }
                    }
                }
            }
        }

        if let Some(t) = self.get_type_mut(type_id)
            && let Type::Interface(i) = t
        {
            i.operations = operations;
            i.operation_names = operation_names;
        }

        self.finalize_type_check(
            ctx,
            type_id,
            node_id,
            &node.template_parameters,
            &node.decorators,
            ctx.mapper.as_ref(),
        );
        type_id
    }
}
