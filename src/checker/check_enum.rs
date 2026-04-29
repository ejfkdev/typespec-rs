use super::*;
use crate::parser::AstNode;

impl Checker {
    pub(crate) fn check_enum(&mut self, ctx: &CheckContext, node_id: NodeId) -> TypeId {
        // Circular reference detection
        if let Some(type_id) = self.check_circular_ref(node_id) {
            return type_id;
        }

        let (ast, node) = require_ast_node!(self, node_id, EnumDeclaration, self.error_type);

        // Mark as pending AFTER all early returns to avoid leaking entries
        self.pending_type_checks.insert(node_id);

        let name = Self::get_identifier_name(&ast, node.name);

        // Use pre-registered type if available, otherwise create new
        let type_id = if let Some(&existing_id) = self.node_type_map.get(&node_id) {
            existing_id
        } else {
            let new_id = self.create_type(Type::Enum(EnumType::new(
                self.next_type_id(),
                name.clone(),
                Some(node_id),
                self.current_namespace,
            )));

            self.register_type(node_id, new_id, &name, ctx.mapper.as_ref());
            new_id
        };

        // Process directives (e.g., #deprecated) early so deprecated context works
        self.process_and_mark_directives(node_id, type_id);

        let mut members = HashMap::new();
        let mut member_names = Vec::new();
        for &member_id in &node.members {
            match ast.id_to_node(member_id) {
                Some(AstNode::EnumMember(m)) => {
                    let member_node = m.clone();
                    let member_name = Self::get_identifier_name(&ast, member_node.name);

                    // Check for duplicate member
                    if members.contains_key(&member_name) {
                        self.error(
                            "enum-member-duplicate",
                            &format!("Enum has duplicate member '{}'.", member_name),
                        );
                        continue; // Skip adding the duplicate
                    }

                    let value = member_node
                        .value
                        .map(|value_id| self.check_node(ctx, value_id));

                    let member_type_id = self.create_type(Type::EnumMember(EnumMemberType {
                        id: self.next_type_id(),
                        name: member_name.clone(),
                        node: Some(member_id),
                        r#enum: Some(type_id),
                        value,
                        source_member: None,
                        decorators: Vec::new(),
                        is_finished: true,
                    }));

                    self.node_type_map.insert(member_id, member_type_id);

                    // Register member before checking decorators so decorator
                    // callbacks can see the current member in the parent's members map.
                    members.insert(member_name.clone(), member_type_id);
                    member_names.push(member_name.clone());

                    // Check member decorators
                    self.check_and_store_decorators(ctx, member_type_id, &member_node.decorators);
                }
                Some(AstNode::EnumSpreadMember(spread_node)) => {
                    let spread_node = spread_node.clone();
                    // Check the target of the spread
                    let target_type_id = self.check_node(ctx, spread_node.target);
                    let target_type = self.get_type(target_type_id).cloned();

                    // Check if the target is an enum
                    match target_type {
                        Some(Type::Enum(target_enum)) => {
                            // Copy members from the target enum
                            for src_member_name in &target_enum.member_names {
                                if let Some(&src_member_id) =
                                    target_enum.members.get(src_member_name)
                                {
                                    // Check for duplicate
                                    if members.contains_key(src_member_name) {
                                        self.error(
                                            "enum-member-duplicate",
                                            &format!(
                                                "Enum has duplicate member '{}'.",
                                                src_member_name
                                            ),
                                        );
                                        continue;
                                    }

                                    // Clone the member and re-parent to the new enum
                                    let cloned_member_id = self.clone_type(src_member_id);
                                    // Set enum reference to the new enum and source_member to original
                                    if let Some(m) = self.get_type_mut(cloned_member_id)
                                        && let Type::EnumMember(em) = m
                                    {
                                        em.r#enum = Some(type_id);
                                        em.source_member = Some(src_member_id);
                                    }
                                    members.insert(src_member_name.clone(), cloned_member_id);
                                    member_names.push(src_member_name.clone());
                                }
                            }
                        }
                        _ => {
                            // Spreading a non-enum type - report spread-enum diagnostic
                            self.error(
                                "spread-enum",
                                "Cannot spread a non-enum type into an enum.",
                            );
                        }
                    }
                }
                _ => continue,
            }
        }

        if let Some(t) = self.get_type_mut(type_id)
            && let Type::Enum(e) = t
        {
            e.members = members;
            e.member_names = member_names;
        }

        // Enums don't support template parameters, pass empty list
        self.finalize_type_check(
            ctx,
            type_id,
            node_id,
            &[],
            &node.decorators,
            ctx.mapper.as_ref(),
        );
        type_id
    }
}
