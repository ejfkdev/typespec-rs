//! Template declaration checking
//!
//! Ported from TypeSpec compiler template checking methods

use super::*;

impl Checker {
    // ========================================================================
    // Check template declaration
    // ========================================================================

    /// Check template parameter declarations
    pub fn check_template_declaration(
        &mut self,
        ctx: &CheckContext,
        ast: &AstBuilder,
        template_param_ids: &[NodeId],
    ) {
        // Push a new scope for this template's parameters before creating them
        self.template_param_scope.push(HashMap::new());

        // Check for shadowing parent template parameters
        // TS: checkTemplateParameterDeclaration → grandParentNode.locals.has(node.id.sv)
        // In our case, parent scope is the previous entry in template_param_scope stack
        let shadow_params: Vec<(NodeId, String)> = if self.template_param_scope.len() > 1 {
            let parent_scope = &self.template_param_scope[self.template_param_scope.len() - 2];
            template_param_ids
                .iter()
                .filter_map(|&param_id| {
                    let param_node = match ast.id_to_node(param_id) {
                        Some(AstNode::TemplateParameterDeclaration(decl)) => decl.clone(),
                        _ => return None,
                    };
                    let name = Self::get_identifier_name(ast, param_node.name);
                    if parent_scope.contains_key(&name) {
                        Some((param_id, name))
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            vec![]
        };
        // During template instantiation (mapper present) the declaration is
        // re-checked and its parameter declarations are artifacts of that
        // re-check, not new declarations — skip the shadow diagnostic
        // (microsoft/typespec#11477).
        if ctx.mapper.is_none() {
            for &(param_id, ref name) in &shadow_params {
                self.warning_at(
                    param_id,
                    "shadow",
                    &format!(
                        "Shadowing parent template parameter with the same name \"{}\"",
                        name
                    ),
                );
            }
        }

        // Collect parameter names first for circular constraint and invalid default detection
        let param_names: Vec<String> = template_param_ids
            .iter()
            .map(|&param_id| {
                let param_node = match ast.id_to_node(param_id) {
                    Some(AstNode::TemplateParameterDeclaration(decl)) => decl.clone(),
                    _ => return String::new(),
                };
                Self::get_identifier_name(ast, param_node.name)
            })
            .collect();

        // Check for invalid template defaults (default referencing a later parameter or itself)
        // TS: "emits diagnostics when defaulted template use later template parameter"
        // Also handles self-referencing (T = T) and mutual references (T = K, K = T)
        for (i, &param_id) in template_param_ids.iter().enumerate() {
            let param_node = match ast.id_to_node(param_id) {
                Some(AstNode::TemplateParameterDeclaration(decl)) => decl.clone(),
                _ => continue,
            };

            if let Some(default_id) = param_node.default {
                // Check if the default expression references itself or any later parameter
                // Self-reference: param_names[i], later: param_names[i + 1..]
                let invalid_names: Vec<&str> =
                    param_names[i..].iter().map(|s| s.as_str()).collect();
                if !invalid_names.is_empty() {
                    let default_refs_invalid =
                        self.expr_references_names(ast, default_id, &invalid_names);
                    if default_refs_invalid {
                        self.error_at(param_id, "invalid-template-default", &format!("Template parameter '{}' default can only reference previously declared type parameters.", param_names[i]));
                    }
                }
            }
        }

        // Check for default-required (non-defaulted param after defaulted one)
        // TS: "emits diagnostics when non-defaulted template parameter comes after defaulted one"
        let mut found_default = false;
        for (i, &param_id) in template_param_ids.iter().enumerate() {
            let param_node = match ast.id_to_node(param_id) {
                Some(AstNode::TemplateParameterDeclaration(decl)) => decl.clone(),
                _ => continue,
            };

            if param_node.default.is_some() {
                found_default = true;
            } else if found_default {
                // Non-defaulted parameter after a defaulted one
                self.error_at(param_id, "default-required", &format!("Template parameter '{}' must have a default because it follows a parameter with a default.", param_names[i]));
            }
        }

        for (i, &param_id) in template_param_ids.iter().enumerate() {
            let param_node = match ast.id_to_node(param_id) {
                Some(AstNode::TemplateParameterDeclaration(decl)) => decl.clone(),
                _ => continue,
            };

            // Skip the expensive constraint/default checking + type creation if the
            // TemplateParameterType was already built on a previous pass (e.g., when
            // the declaration is re-checked during instantiation). The scope
            // registration below MUST still run on every pass: the scope is popped at
            // the end of each check_template_declaration / finish_template_or_type, so
            // failing to re-register would leave the body unable to resolve the param
            // name (regression: "Unknown type 'T'" during instantiation).
            let already_created = self
                .node_type_map
                .get(&param_id)
                .and_then(|&tid| self.get_type(tid))
                .is_some_and(|t| t.is_finished());

            if !already_created {
                // Check constraint for circular references
                // Add ALL template parameter names to pending set before checking constraint,
                // so mutual constraints (A extends B, B extends A) are detected.
                let (constraint, value_constraint) =
                    if let Some(constraint_id) = param_node.constraint {
                        // Add all param names to detect mutual circular constraints
                        for pname in &param_names {
                            self.pending_template_constraint_names.insert(pname.clone());
                        }

                        // Check for mixed constraint (e.g., `T extends string | valueof int32`)
                        let (type_constraint, value_constraint) =
                            self.check_mixed_constraint(ctx, constraint_id);

                        // Remove all param names after checking
                        for pname in &param_names {
                            self.pending_template_constraint_names.remove(pname);
                        }
                        (type_constraint, value_constraint)
                    } else {
                        (None, None)
                    };

                let default = if let Some(default_id) = param_node.default {
                    let default_type = self.check_node(ctx, default_id);
                    // Check if default is assignable to constraint
                    // Ported from TS checker.ts checkTemplateArguments()
                    if let Some(constraint_id) = constraint
                        && default_type != self.error_type
                        && constraint_id != self.error_type
                    {
                        let (is_assignable, _) =
                            self.is_type_assignable_to(default_type, constraint_id, param_id);
                        if !is_assignable {
                            self.error_unassignable("unassignable", default_type, constraint_id);
                        }
                    }
                    Some(default_type)
                } else {
                    None
                };

                let type_id = self.create_type(Type::TemplateParameter(TemplateParameterType {
                    id: self.next_type_id(),
                    name: param_names[i].clone(),
                    node: Some(param_id),
                    constraint,
                    value_constraint,
                    default,
                    is_finished: true,
                }));

                self.node_type_map.insert(param_id, type_id);
            }

            // The TemplateParameterType is now guaranteed present in node_type_map.
            // Determine the effective type_id for name resolution:
            // - During template declaration (no mapper): use the TemplateParameter type
            // - During instantiation (with mapper): use the mapped argument type
            let template_param_type = self.node_type_map[&param_id];
            // Body name resolution happens through `template_param_scope`, not
            // `node_type_map`, so we must NOT overwrite the entry here. The
            // TemplateParameterType must stay in node_type_map because the
            // default-filling and constraint logic in instantiate_template reads
            // it expecting a TemplateParameterType with a `.default` field.
            // Overwriting it with the mapped argument type breaks default filling,
            // causing `Foo` (auto-filled) and `Foo<string>` (explicit) to get
            // different cache keys → different instances.
            let effective_type_id = if let Some(ref mapper) = ctx.mapper
                && let Some(&mapped_type_id) = mapper.map.get(&param_id)
            {
                mapped_type_id
            } else {
                template_param_type
            };

            // Register template parameter name for name resolution in template body
            // This allows identifiers like `T` to resolve during checking. Runs on every
            // pass because the scope is popped when the template body check completes.
            if let Some(scope) = self.template_param_scope.last_mut() {
                scope.insert(param_names[i].clone(), effective_type_id);
            }
        }
    }

    /// Pop the template parameter scope after finishing template body checking.
    /// Must be called after the template's properties/members/heritage are fully checked.
    pub fn pop_template_param_scope(&mut self) {
        self.template_param_scope.pop();
    }

    /// Check if an expression AST node references any of the given names
    pub(crate) fn expr_references_names(
        &self,
        ast: &AstBuilder,
        node_id: NodeId,
        names: &[&str],
    ) -> bool {
        let node = match ast.id_to_node(node_id) {
            Some(n) => n.clone(),
            None => return false,
        };

        match &node {
            AstNode::Identifier(ident) => names.contains(&ident.value.as_str()),
            AstNode::TypeReference(ref_node) => {
                let name = Self::get_identifier_name(ast, ref_node.name);
                names.contains(&name.as_str())
            }
            // Recursively check child nodes
            AstNode::UnionExpression(expr) => expr
                .options
                .iter()
                .any(|&id| self.expr_references_names(ast, id, names)),
            AstNode::IntersectionExpression(expr) => expr
                .options
                .iter()
                .any(|&id| self.expr_references_names(ast, id, names)),
            AstNode::ArrayExpression(expr) => {
                self.expr_references_names(ast, expr.element_type, names)
            }
            AstNode::TupleExpression(expr) => expr
                .values
                .iter()
                .any(|&id| self.expr_references_names(ast, id, names)),
            AstNode::ModelExpression(expr) => expr
                .properties
                .iter()
                .any(|&id| self.expr_references_names(ast, id, names)),
            AstNode::ModelProperty(prop) => self.expr_references_names(ast, prop.value, names),
            AstNode::ValueOfExpression(expr) => self.expr_references_names(ast, expr.target, names),
            _ => false,
        }
    }

    /// Check a constraint for mixed type/value parts (e.g., `string | valueof int32`).
    /// Returns (type_constraint, value_constraint).
    /// If the constraint is purely a type (no `valueof`), returns (Some(type), None).
    /// If the constraint has both type and value parts, splits them accordingly.
    pub(crate) fn check_mixed_constraint(
        &mut self,
        ctx: &CheckContext,
        constraint_node_id: NodeId,
    ) -> (Option<TypeId>, Option<TypeId>) {
        let ast = require_ast_or!(self, (None, None));

        // Check if the constraint is a union expression containing valueof parts
        if let Some(AstNode::UnionExpression(union_expr)) = ast.id_to_node(constraint_node_id) {
            let mut type_options = Vec::new();
            let mut value_options = Vec::new();

            for &option_id in &union_expr.options {
                if matches!(
                    ast.id_to_node(option_id),
                    Some(AstNode::ValueOfExpression(_))
                ) {
                    // valueof part — check the target type
                    let value_type = self.check_node(ctx, option_id);
                    value_options.push(value_type);
                } else {
                    // Regular type part
                    let type_id = self.check_node(ctx, option_id);
                    type_options.push(type_id);
                }
            }

            if !value_options.is_empty() {
                // Mixed constraint detected
                let type_constraint = if type_options.len() == 1 {
                    Some(type_options.into_iter().next().unwrap())
                } else if type_options.is_empty() {
                    None
                } else {
                    // Create a union of type options
                    Some(self.create_union(type_options))
                };

                let value_constraint = if value_options.len() == 1 {
                    Some(value_options.into_iter().next().unwrap())
                } else {
                    Some(self.create_union(value_options))
                };

                return (type_constraint, value_constraint);
            }
        }

        // Not a mixed constraint — just a regular type constraint
        let constraint_type = self.check_node(ctx, constraint_node_id);
        (Some(constraint_type), None)
    }
}
