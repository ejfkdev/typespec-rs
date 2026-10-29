use super::*;
use crate::parser::AstNode;

impl Checker {
    pub(crate) fn check_operation(&mut self, ctx: &CheckContext, node_id: NodeId) -> TypeId {
        self.check_operation_internal(ctx, node_id, None)
    }

    pub(crate) fn check_operation_internal(
        &mut self,
        ctx: &CheckContext,
        node_id: NodeId,
        parent_interface: Option<TypeId>,
    ) -> TypeId {
        let (ast, node) = require_ast_node!(self, node_id, OperationDeclaration, self.error_type);

        let name = Self::get_identifier_name(&ast, node.name);
        let current_ns = self.current_namespace;

        // Resolve (or create) the operation type before anything else so that
        // directives — notably #deprecated — are applied before the signature
        // is checked. A deprecated operation suppresses deprecation warnings
        // for types referenced in its own signature
        // (microsoft/typespec#12314).
        let type_id = if ctx.mapper.is_none()
            && let Some(&existing_id) = self.node_type_map.get(&node_id)
        {
            existing_id
        } else {
            let template_node =
                self.compute_template_node(&node.template_parameters, ctx.mapper.as_ref(), node_id);
            let new_id = {
                let mut o = OperationType::new(
                    self.next_type_id(),
                    name.clone(),
                    Some(node_id),
                    current_ns,
                );
                o.interface_ = parent_interface;
                o.template_node = template_node;
                self.create_type(Type::Operation(o))
            };
            self.node_type_map.insert(node_id, new_id);
            if ctx.mapper.is_none() && parent_interface.is_none() && !name.is_empty() {
                self.register_declared_type(&name, new_id);
            }
            new_id
        };
        self.process_and_mark_directives(node_id, type_id);

        // Keep this operation in the pending stack while its signature is
        // checked so `in_deprecated_context` sees it (microsoft/typespec#12314).
        self.pending_type_checks.insert(node_id);

        self.check_template_declaration(ctx, &ast, &node.template_parameters);

        // Handle `is` (signature reference) vs normal declaration
        let (parameters, return_type, source_operation) = match ast.id_to_node(node.signature) {
            Some(AstNode::OperationSignatureDeclaration(sig)) => {
                let params = Some(self.check_node(ctx, sig.parameters));
                // Name the anonymous parameter model `<op>::parameters` for
                // diagnostics (microsoft/typespec#11551).
                if let Some(params_id) = params
                    && matches!(self.get_type(params_id), Some(Type::Model(m)) if m.name.is_empty())
                {
                    let op_fqn = self.build_fqn(&name);
                    let ns_prefix = op_fqn.strip_suffix(&name).unwrap_or_default().to_string();
                    let iface_prefix = parent_interface
                        .and_then(|id| {
                            self.get_type(id)
                                .and_then(|t| t.name().map(|s| s.to_string()))
                        })
                        .map(|n| format!("{}.", n))
                        .unwrap_or_default();
                    self.type_display_names.insert(
                        params_id,
                        format!("{}{}{}::parameters", ns_prefix, iface_prefix, name),
                    );
                }
                let ret = Some(self.check_node(ctx, sig.return_type));
                (params, ret, None)
            }
            Some(AstNode::OperationSignatureReference(sig)) => {
                // Track this operation name for circular detection
                if !name.is_empty() {
                    self.pending_op_signature_names.insert(name.clone());
                }

                let base_op_type = self.check_node(ctx, sig.base_operation);

                if !name.is_empty() {
                    self.pending_op_signature_names.remove(&name);
                }

                // Check if base is a valid operation type
                if base_op_type != self.error_type {
                    let resolved_base = self.resolve_alias_chain(base_op_type);
                    match self.get_type(resolved_base) {
                        Some(Type::Operation(base_op_cloned)) => {
                            // Valid - use the base operation's parameters and return type
                            let params = base_op_cloned.parameters;
                            let ret = base_op_cloned.return_type;
                            (params, ret, Some(resolved_base))
                        }
                        Some(_) => {
                            // Base is not an operation - report is-operation diagnostic
                            self.error_at(
                                node_id,
                                "is-operation",
                                "Operation 'is' must reference another operation.",
                            );
                            // Create an empty operation with void return
                            (None, Some(self.void_type), None)
                        }
                        None => (None, Some(self.void_type), None),
                    }
                } else {
                    // Error type - circular reference was already reported
                    (None, Some(self.void_type), None)
                }
            }
            _ => (None, Some(self.void_type), None),
        };

        let template_node =
            self.compute_template_node(&node.template_parameters, ctx.mapper.as_ref(), node_id);

        // Fill in the signature on the operation type created above.
        if let Some(t) = self.get_type_mut(type_id)
            && let Type::Operation(o) = t
        {
            o.parameters = parameters;
            o.return_type = return_type;
            o.source_operation = source_operation;
            o.interface_ = parent_interface;
            o.template_node = template_node;
            o.namespace = current_ns;
        }
        // Always update symbol_links so template instantiation and is_template_declaration work
        let links = self.symbol_links.entry(node_id).or_default();
        if let Some(mapper) = ctx.mapper.as_ref() {
            if !mapper.partial {
                let instantiations = links.instantiations.get_or_insert_with(HashMap::new);
                instantiations.insert(mapper.args.clone(), type_id);
            }
            links.type_id = Some(type_id);
        } else {
            links.declared_type = Some(type_id);
            links.type_id = Some(type_id);
        }

        self.pending_type_checks.remove(&node_id);

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
