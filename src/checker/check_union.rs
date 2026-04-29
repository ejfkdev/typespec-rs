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

        self.check_template_declaration(ctx, &ast, &node.template_parameters);

        let template_node =
            self.compute_template_node(&node.template_parameters, ctx.mapper.as_ref(), node_id);

        // Use pre-registered type if available, otherwise create new
        let type_id = if let Some(&existing_id) = self.node_type_map.get(&node_id) {
            // Update pre-registered type in-place
            if let Some(t) = self.get_type_mut(existing_id)
                && let Type::Union(u) = t
            {
                u.template_node = template_node;
            }
            existing_id
        } else {
            let new_id = {
                let mut u = UnionType::new(
                    self.next_type_id(),
                    name.clone(),
                    Some(node_id),
                    self.current_namespace,
                    false,
                );
                u.template_node = template_node;
                self.create_type(Type::Union(u))
            };

            self.register_type(node_id, new_id, &name, ctx.mapper.as_ref());
            new_id
        };

        // Process directives (e.g., #deprecated) early so deprecated context works
        self.process_and_mark_directives(node_id, type_id);

        let mut variants = HashMap::new();
        let mut variant_names = Vec::new();
        for &variant_id in &node.variants {
            let variant_node = match ast.id_to_node(variant_id) {
                Some(AstNode::UnionVariant(v)) => v.clone(),
                _ => continue,
            };

            let variant_name = variant_node
                .name
                .map(|n| Self::get_identifier_name(&ast, n))
                .unwrap_or_default();

            // Check for duplicate variant
            if !variant_name.is_empty() && variants.contains_key(&variant_name) {
                self.error(
                    "union-duplicate",
                    &format!("Union has duplicate variant '{}'.", variant_name),
                );
                continue; // Skip adding the duplicate
            }

            let variant_type = self.check_node(ctx, variant_node.value);

            let variant_type_id = self.create_type(Type::UnionVariant(UnionVariantType {
                id: self.next_type_id(),
                name: variant_name.clone(),
                node: Some(variant_id),
                r#type: variant_type,
                union: Some(type_id),
                decorators: Vec::new(),
                is_finished: true,
            }));

            self.node_type_map.insert(variant_id, variant_type_id);

            // Check variant decorators
            self.check_and_store_decorators(ctx, variant_type_id, &variant_node.decorators);

            variants.insert(variant_name.clone(), variant_type_id);
            variant_names.push(variant_name);
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
}
