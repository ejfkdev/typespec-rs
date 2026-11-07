use super::*;
use crate::parser::AstNode;

impl Checker {
    pub(crate) fn check_union(&mut self, ctx: &CheckContext, node_id: NodeId) -> TypeId {
        // Circular reference detection
        if let Some(type_id) = self.check_circular_ref(node_id) {
            return type_id;
        }

        let (ast, node) = require_ast_node!(self, node_id, UnionDeclaration, self.error_type);

        // Mark as pending AFTER all early returns to avoid leaking entries
        self.pending_type_checks.insert(node_id);

        let name = Self::get_identifier_name(&ast, node.name);

        // Declarations used as expressions (microsoft/typespec#11019).
        self.check_declaration_expression_usage(
            ctx,
            node_id,
            node.expression,
            node.template_parameters.len(),
        );

        self.check_template_declaration(ctx, &ast, &node.template_parameters);

        let template_node =
            self.compute_template_node(&node.template_parameters, ctx.mapper.as_ref(), node_id);

        let current_ns = self.current_namespace;

        // Use pre-registered type if available, otherwise create new
        let type_id = if ctx.mapper.is_none()
            && let Some(&existing_id) = self.node_type_map.get(&node_id)
        {
            // Update pre-registered type in-place
            if let Some(t) = self.get_type_mut(existing_id)
                && let Type::Union(u) = t
            {
                u.template_node = template_node;
                u.namespace = current_ns;
            }
            existing_id
        } else {
            let new_id = {
                let mut u = UnionType::new(
                    self.next_type_id(),
                    name.clone(),
                    Some(node_id),
                    current_ns,
                    node.expression,
                );
                u.template_node = template_node;
                self.create_type(Type::Union(u))
            };

            if node.expression {
                self.register_expression_type(node_id, new_id);
            } else {
                self.register_type(node_id, new_id, &name, ctx.mapper.as_ref());
            }
            new_id
        };

        // Process directives (e.g., #deprecated) early so deprecated context works
        self.process_and_mark_directives(node_id, type_id);

        // Resolve the `extends` clause, if any (microsoft/typespec#11771). The
        // base type is only a constraint on the variants: it does not create
        // an inheritance relationship and is not added as a variant.
        let base_type = if let Some(extends_node) = node.extends {
            if ctx.mapper.is_none()
                && !self.is_compiler_feature_enabled("union-extends", Some(node_id))
            {
                self.error_at(
                    extends_node,
                    "union-extends-disabled",
                    "Union `extends` clauses require the 'union-extends' feature to be enabled. Add 'union-extends' to the 'features' list in your tspconfig.yaml.",
                );
            }
            let resolved = self.check_node(ctx, extends_node);
            if resolved == self.error_type {
                None
            } else if self.union_expression_references(resolved, type_id) {
                if ctx.mapper.is_none() {
                    self.error_at(
                        extends_node,
                        "circular-base-type",
                        &format!(
                            "Type '{}' recursively references itself as a base type.",
                            name
                        ),
                    );
                }
                None
            } else if matches!(self.get_type(resolved), Some(Type::TemplateParameter(_))) {
                // Unresolved template parameter: validation happens on
                // instantiation.
                None
            } else if {
                // Aliases are Scalar wrappers in this port, so look through the
                // alias chain before classifying the base type.
                let resolved = self.resolve_alias_chain(resolved);
                matches!(
                    self.get_type(resolved),
                    Some(Type::Model(m)) if self.is_model_expression_type(m)
                )
            } {
                self.error_at(
                    extends_node,
                    "extend-union",
                    "Unions cannot extend model expressions.",
                );
                None
            } else if !matches!(
                self.get_type(resolved),
                Some(Type::Model(_))
                    | Some(Type::Scalar(_))
                    | Some(Type::Enum(_))
                    | Some(Type::Union(_))
            ) {
                self.error_at(
                    extends_node,
                    "extend-union",
                    "Union `extends` must specify a model, scalar, enum, or union.",
                );
                None
            } else {
                Some(resolved)
            }
        } else {
            None
        };

        if let Some(base) = base_type
            && let Some(t) = self.get_type_mut(type_id)
            && let Type::Union(u) = t
        {
            u.base_type = Some(base);
        }

        let mut variants = HashMap::new();
        let mut variant_names = Vec::new();
        for (variant_index, &variant_id) in node.variants.iter().enumerate() {
            let variant_node = match ast.id_to_node(variant_id) {
                Some(AstNode::UnionVariant(v)) => v.clone(),
                _ => continue,
            };

            let variant_name = variant_node
                .name
                .map(|n| Self::get_identifier_name(&ast, n))
                .unwrap_or_default();
            // Anonymous variants need distinct map keys; upstream keys them by
            // symbol. Keep `UnionVariantType.name` empty for display.
            let variant_key = if variant_name.is_empty() {
                format!("\u{0}anonymous{variant_index}")
            } else {
                variant_name.clone()
            };

            // Check for duplicate variant
            if !variant_name.is_empty() && variants.contains_key(&variant_name) {
                self.error_at(
                    variant_id,
                    "union-duplicate",
                    &format!("Union has duplicate variant '{}'.", variant_name),
                );
                continue; // Skip adding the duplicate
            }

            let variant_type = self.check_node(ctx, variant_node.value);

            // Validate the variant against the union's `extends` constraint.
            // Uninstantiated template declarations are validated per
            // instantiation instead (microsoft/typespec#11771).
            if let Some(base) = base_type
                && (ctx.mapper.is_some() || node.template_parameters.is_empty())
                && variant_type != self.error_type
            {
                let (is_assignable, _) =
                    self.is_type_assignable_to(variant_type, base, variant_node.value);
                if !is_assignable {
                    let source = self.type_to_string(variant_type);
                    let target = self.type_to_string(base);
                    self.error_at(
                        variant_node.value,
                        "unassignable",
                        &format!("Type '{}' is not assignable to type '{}'", source, target),
                    );
                }
            }

            let variant_type_id = self.create_type(Type::UnionVariant(UnionVariantType {
                id: self.next_type_id(),
                name: variant_name.clone(),
                node: Some(variant_id),
                r#type: variant_type,
                union: Some(type_id),
                decorators: Vec::new(),
                doc: None,
                summary: None,
                is_finished: true,
            }));

            self.node_type_map.insert(variant_id, variant_type_id);

            // Check variant decorators
            self.check_and_store_decorators(ctx, variant_type_id, &variant_node.decorators);
            self.apply_doc_comment(variant_type_id, variant_id);

            variants.insert(variant_key.clone(), variant_type_id);
            variant_names.push(variant_key);
        }

        if let Some(t) = self.get_type_mut(type_id)
            && let Type::Union(u) = t
        {
            u.variants = variants;
            u.variant_names = variant_names;
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

    /// Whether the model was declared as a model expression (`{ ... }`).
    fn is_model_expression_type(&self, model: &ModelType) -> bool {
        self.ast
            .as_ref()
            .and_then(|ast| model.node.and_then(|n| ast.id_to_node(n)))
            .is_some_and(|n| matches!(n, AstNode::ModelExpression(_)))
    }

    /// Check whether `target` is reachable from `type` through union
    /// expressions only. Named unions, models, arrays, ... stop the traversal:
    /// a union referencing itself from those positions builds a valid cyclic
    /// type graph.
    ///
    /// Ported from TS `unionExpressionReferences` (microsoft/typespec#11771).
    fn union_expression_references(&self, type_id: TypeId, target: TypeId) -> bool {
        if type_id == target {
            return true;
        }
        if let Some(Type::Union(u)) = self.get_type(type_id)
            && u.expression
        {
            for name in &u.variant_names {
                if let Some(&variant_id) = u.variants.get(name)
                    && let Some(Type::UnionVariant(v)) = self.get_type(variant_id)
                    && self.union_expression_references(v.r#type, target)
                {
                    return true;
                }
            }
        }
        false
    }
}
