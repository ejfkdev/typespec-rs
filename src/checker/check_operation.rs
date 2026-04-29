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

        self.check_template_declaration(ctx, &ast, &node.template_parameters);

        // Handle `is` (signature reference) vs normal declaration
        let (parameters, return_type, source_operation) = match ast.id_to_node(node.signature) {
            Some(AstNode::OperationSignatureDeclaration(sig)) => {
                let params = Some(self.check_node(ctx, sig.parameters));
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
                            self.error(
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

        let type_id = {
            let mut o = OperationType::new(
                self.next_type_id(),
                name.clone(),
                Some(node_id),
                self.current_namespace,
            );
            o.parameters = parameters;
            o.return_type = return_type;
            o.source_operation = source_operation;
            o.interface_ = parent_interface;
            o.template_node = template_node;
            self.create_type(Type::Operation(o))
        };

        self.node_type_map.insert(node_id, type_id);

        // Register in symbol_links and declared_types (consistent with other check_* functions)
        if ctx.mapper.is_none() && parent_interface.is_none() && !name.is_empty() {
            self.declared_types.insert(name.clone(), type_id);
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

        // Process directives (e.g., #deprecated) early so deprecated context works
        self.process_and_mark_directives(node_id, type_id);

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
