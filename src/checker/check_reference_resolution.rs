//! Reference resolution (member expressions and identifiers)
//!
//! Ported from TypeSpec compiler reference resolution methods

use super::*;

impl Checker {
    pub(crate) fn check_member_expression(
        &mut self,
        ctx: &CheckContext,
        node_id: NodeId,
    ) -> TypeId {
        let (ast, node) = require_ast_node!(self, node_id, MemberExpression, self.error_type);

        // Check the object (base) part first
        let base_type = self.check_node(ctx, node.object);

        // TS: When referencing a template type without providing arguments
        // (e.g., A.t where A is alias A<T>), report invalid-template-args.
        // The base_type returned by check_node for a template declaration
        // is the declaration itself (not an instance), which is not valid
        // for member access.
        if base_type != self.error_type {
            let is_template_decl = self
                .get_type(base_type)
                .is_some_and(|t| t.template_node().is_some());
            if is_template_decl {
                let required_param_count = self.get_required_template_param_count(base_type);
                if required_param_count > 0 {
                    let base_name = Self::get_identifier_name(&ast, node.object);
                    let missing_param = self.get_missing_template_param_name(base_type, 0);
                    self.error(
                        "invalid-template-args",
                        &format!(
                            "Template argument '{}' is required for '{}'.",
                            missing_param, base_name
                        ),
                    );
                    return self.error_type;
                }
            }
        }

        // Get the property name
        let prop_name = Self::get_identifier_name(&ast, node.property);

        // Check internal visibility for member access (e.g., "TypeSpec.indexer")
        let full_name = Self::get_identifier_name(&ast, node_id);
        if let Some(decl_id) = self.declared_types.get(full_name.as_str()).copied()
            && self.is_internal_type(decl_id)
            && !self.is_current_context_compiler()
        {
            self.error("invalid-ref", &format!("Symbol '{}' is internal and can only be accessed from within its declaring package.", full_name));
            return self.error_type;
        }

        // Look up the property in the base type
        let result_type = match self.get_type(base_type) {
            Some(Type::Namespace(ns)) => {
                if let Some(member_id) = ns.lookup_member(&prop_name) {
                    self.check_internal_visibility(member_id);
                    return member_id;
                }
                self.error(
                    "invalid-ref",
                    &format!("Namespace '{}' has no member '{}'", ns.name, prop_name),
                );
                self.error_type
            }
            Some(Type::Model(m)) => {
                // Model property access - look up by name
                if let Some(&prop_id) = m.properties.get(&prop_name) {
                    // Check if this property is currently being resolved (circular-prop)
                    if let Some(Type::ModelProperty(prop)) = self.get_type(prop_id)
                        && let Some(prop_node) = prop.node
                        && self.pending_type_checks.contains(&prop_node)
                    {
                        self.error(
                            "circular-prop",
                            &format!(
                                "Property '{}' recursively references itself.",
                                prop_name
                            ),
                        );
                        return self.error_type;
                    }
                    return prop_id;
                }
                self.error(
                    "invalid-ref",
                    &format!("Model '{}' has no property '{}'", m.name, prop_name),
                );
                self.error_type
            }
            Some(Type::Enum(e)) => {
                // Enum member access
                if let Some(&member_id) = e.members.get(&prop_name) {
                    return member_id;
                }
                self.error(
                    "invalid-ref",
                    &format!("Enum '{}' has no member '{}'", e.name, prop_name),
                );
                self.error_type
            }
            Some(Type::Union(u)) => {
                // Union variant access
                if let Some(&variant_id) = u.variants.get(&prop_name) {
                    return variant_id;
                }
                self.error(
                    "invalid-ref",
                    &format!("Union '{}' has no variant '{}'", u.name, prop_name),
                );
                self.error_type
            }
            Some(Type::Interface(iface)) => {
                // Interface operation access
                if let Some(&op_id) = iface.operations.get(&prop_name) {
                    return op_id;
                }
                self.error(
                    "invalid-ref",
                    &format!(
                        "Interface '{}' has no operation '{}'",
                        iface.name, prop_name
                    ),
                );
                self.error_type
            }
            Some(Type::Scalar(s)) => {
                // Scalar may be an alias wrapping another type (e.g., alias MyBase = Base<string>)
                // Resolve through the alias chain to access the underlying type's members
                if s.base_scalar.is_some() {
                    let resolved = self.resolve_alias_chain(base_type);
                    if resolved != base_type && resolved != self.error_type {
                        // Re-dispatch member access on the resolved type
                        return match self.get_type(resolved) {
                            Some(Type::Interface(iface)) => {
                                if let Some(&op_id) = iface.operations.get(&prop_name) {
                                    op_id
                                } else {
                                    self.error(
                                        "invalid-ref",
                                        &format!(
                                            "Interface '{}' has no operation '{}'",
                                            iface.name, prop_name
                                        ),
                                    );
                                    self.error_type
                                }
                            }
                            Some(Type::Namespace(ns)) => match ns.lookup_member(&prop_name) {
                                Some(id) => id,
                                None => {
                                    self.error(
                                        "invalid-ref",
                                        &format!(
                                            "Namespace '{}' has no member '{}'",
                                            ns.name, prop_name
                                        ),
                                    );
                                    self.error_type
                                }
                            },
                            _ => {
                                // Fall through to std type check
                                if let Some(&std_id) = self.std_types.get(&prop_name) {
                                    std_id
                                } else {
                                    self.error(
                                        "invalid-ref",
                                        &format!(
                                            "Cannot access member '{}' on scalar '{}'",
                                            prop_name, s.name
                                        ),
                                    );
                                    self.error_type
                                }
                            }
                        };
                    }
                }
                // Check for standard type access (e.g., string.length)
                if let Some(&std_id) = self.std_types.get(&prop_name) {
                    return std_id;
                }
                self.error(
                    "invalid-ref",
                    &format!(
                        "Cannot access member '{}' on scalar '{}'",
                        prop_name, s.name
                    ),
                );
                self.error_type
            }
            _ => {
                self.error(
                    "invalid-ref",
                    &format!("Cannot access member '{}' on this type", prop_name),
                );
                self.error_type
            }
        };

        self.node_type_map.insert(node_id, result_type);
        result_type
    }

    pub(crate) fn check_identifier(&mut self, ctx: &CheckContext, node_id: NodeId) -> TypeId {
        let result = self.check_identifier_inner(ctx, node_id);
        self.node_type_map.insert(node_id, result);
        result
    }

    pub(crate) fn check_identifier_inner(&mut self, ctx: &CheckContext, node_id: NodeId) -> TypeId {
        let ast = require_ast_or!(self, self.error_type);

        let name = Self::get_identifier_name(&ast, node_id);

        if let Some(&std_id) = self.std_types.get(&name) {
            return std_id;
        }

        // Check if this name is currently being resolved (circular reference detection)
        if let Some(error) = self.check_circular_reference(&name) {
            return error;
        }

        if let Some(&type_id) = self.declared_types.get(&name) {
            // Check if the resolved type is a decorator or function — can't be used as type references
            if let Some(error) = self.check_invalid_type_ref_kind(type_id) {
                return error;
            }

            // If the declared type is not yet finished, trigger its checking.
            // This ensures circular references across aliases (e.g. alias A = B; alias B = A)
            // are detected, because checking the referenced type will add its name
            // to pending_type_names, and if we're already checking it, the circular
            // detection at the top of this function will fire.
            // EXCEPTION: For Namespace types, don't re-trigger checking as they may
            // be in the process of being populated (e.g. during check_source_file),
            // which could cause infinite recursion.
            if let Some(t) = self.get_type(type_id)
                && !t.is_finished()
                && !matches!(t, Type::Namespace(_))
                && let Some(node_id_for_type) = t.node_id_from_type()
            {
                return self.check_node(&CheckContext::new(), node_id_for_type);
            }
            return type_id;
        }

        if let Some(&type_id) = self.node_type_map.get(&node_id) {
            return type_id;
        }

        // Try to resolve as a template parameter name (e.g., T in model Foo<T> { a: T })
        // Search from innermost scope outward to handle shadowing correctly
        for scope in self.template_param_scope.iter().rev() {
            if let Some(&type_id) = scope.get(&name) {
                // TS: checkTemplateParameterDeclaration returns
                // ctx.mapper ? ctx.mapper.getMappedType(type) : type
                // When instantiating a template, the mapper maps TemplateParameter NodeIds
                // to the actual argument TypeIds.
                if let Some(ref mapper) = ctx.mapper {
                    // The mapper keys are NodeIds of TemplateParameterDeclaration AST nodes.
                    // We need to find the NodeId for this TemplateParameter type.
                    if let Some(t) = self.get_type(type_id)
                        && let Type::TemplateParameter(tp) = t
                        && let Some(tp_node_id) = tp.node
                        && let Some(&mapped_type_id) = mapper.map.get(&tp_node_id)
                    {
                        return mapped_type_id;
                    }
                }
                return type_id;
            }
        }

        // Try to resolve name via using declarations
        if let Some(type_id) = self.resolve_via_using(&name) {
            // Check internal visibility for the resolved type
            self.check_internal_visibility(type_id);
            return type_id;
        }

        // Name not found - report invalid-ref diagnostic
        self.error("invalid-ref", &format!("Unknown type '{}'", name));
        self.error_type
    }
}
