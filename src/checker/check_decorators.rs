//! Decorator checking
//!
//! Ported from TypeSpec compiler decorator checking methods

use super::*;

impl Checker {
    // ========================================================================
    // Check and store decorators
    // ========================================================================

    /// Check decorator expressions and store them on the type
    pub fn check_and_store_decorators(
        &mut self,
        ctx: &CheckContext,
        type_id: TypeId,
        decorator_ids: &[NodeId],
    ) {
        if decorator_ids.is_empty() {
            return;
        }

        let ast = require_ast_or!(self);

        let mut decorator_apps = Vec::new();

        for &dec_id in decorator_ids {
            let dec_node = match ast.id_to_node(dec_id) {
                Some(AstNode::DecoratorExpression(dec_expr)) => dec_expr.clone(),
                _ => continue,
            };

            // Resolve decorator name to find its declaration
            let decorator_name = Self::get_identifier_name(&ast, dec_node.target);

            // Resolve the decorator TypeId from the name (handles dotted names like "TypeSpec.indexer")
            let declaration_type_id = self.resolve_decorator_by_name(&decorator_name);

            // Check if this is a compiler-internal decorator that cannot be used from user code
            // Ported from TS checkSymbolAccess — check visibility of the resolved declaration
            if let Some(decl_id) = declaration_type_id {
                if self.is_internal_type(decl_id) {
                    self.check_internal_visibility_for(decl_id, &decorator_name);
                    // If it's internal and we're not in compiler context, skip it
                    if self.internal_declarations.contains(&decl_id)
                        && !self.is_current_context_compiler()
                    {
                        continue;
                    }
                }
            }

            // Ported from TS checker.ts:5718 — check that @target resolves to a decorator
            if let Some(decl_type_id) = declaration_type_id
                && !matches!(self.get_type(decl_type_id), Some(Type::Decorator(_)))
            {
                self.error(
                    "invalid-decorator",
                    &format!("{} is not a decorator", decorator_name),
                );
                continue;
            }

            // Validate decorator arguments against declaration
            if let Some(decl_type_id) = declaration_type_id
                && let Some(Type::Decorator(decl)) = self.get_type(decl_type_id)
            {
                // Ported from TS checker.ts checkDecoratorArguments
                let min_args = decl
                    .parameters
                    .iter()
                    .filter(|p| !p.optional && !p.rest)
                    .count();
                let max_args = if decl.parameters.last().is_some_and(|p| p.rest) {
                    None
                } else {
                    Some(decl.parameters.len())
                };

                let actual_args = dec_node.arguments.len();
                if actual_args < min_args || max_args.is_some_and(|max| actual_args > max) {
                    let expected = match max_args {
                        None => format!("at least {}", min_args),
                        Some(max) if min_args == max => format!("{}", min_args),
                        Some(max) => format!("{}-{}", min_args, max),
                    };
                    self.error(
                        "invalid-argument-count",
                        &format!(
                            "Decorator '{}' expects {} argument(s), but got {}.",
                            decorator_name, expected, actual_args
                        ),
                    );
                }
            }

            // Validate decorator target type against declaration's target constraint
            if let Some(decl_type_id) = declaration_type_id
                && let Some(Type::Decorator(decl)) = self.get_type(decl_type_id)
            {
                // Ported from TS checker.ts checkDecoratorTarget
                let target_constraint = &decl.target_type;
                if !target_constraint.is_empty() {
                    // Check if the decorated type is assignable to the target constraint
                    let target_type_name = match self.get_type(type_id) {
                        Some(Type::Model(_)) => "Model",
                        Some(Type::Scalar(_)) => "Scalar",
                        Some(Type::Interface(_)) => "Interface",
                        Some(Type::Union(u)) if !u.name.is_empty() => "Union",
                        Some(Type::Enum(_)) => "Enum",
                        Some(Type::Operation(_)) => "Operation",
                        Some(Type::Namespace(_)) => "Namespace",
                        _ => "unknown",
                    };
                    // Simple check: if target constraint is "Model" but decorated type is "Enum"
                    if target_constraint != "unknown" && target_constraint != target_type_name {
                        self.error(
                            "decorator-wrong-target",
                            &format!(
                                "Decorator '{}' cannot be applied to {}. Expected {}.",
                                decorator_name, target_type_name, target_constraint
                            ),
                        );
                    }
                }
            }

            let mut args = Vec::new();
            for (index, &arg_id) in dec_node.arguments.iter().enumerate() {
                let arg_type = self.check_node(ctx, arg_id);

                // Validate argument type against declaration parameter constraint
                if let Some(decl_type_id) = declaration_type_id
                    && let Some(Type::Decorator(decl)) = self.get_type(decl_type_id)
                    && index < decl.parameters.len()
                {
                    let param = &decl.parameters[index];
                    if let Some(expected_type_id) = param.r#type {
                        // Use type relation checker for proper assignability
                        let (is_assignable, _) =
                            self.is_type_assignable_to(arg_type, expected_type_id, 0);
                        if !is_assignable {
                            let arg_name = self.type_to_string(arg_type);
                            let expected_name = self.type_to_string(expected_type_id);
                            self.error("invalid-argument", &format!("Argument of type '{}' is not assignable to parameter of type '{}'.", arg_name, expected_name));
                        }
                    }
                }

                args.push(DecoratorArgument {
                    value: arg_id,
                    js_value: None,
                    node: Some(arg_id),
                });
            }

            decorator_apps.push(DecoratorApplication {
                definition: declaration_type_id,
                decorator: dec_id,
                args,
                node: Some(dec_id),
            });
        }

        // Post-processing: validate @overload decorator
        // Ported from TS decorators.ts $overload handler
        for app in &decorator_apps {
            let dec_name = app
                .definition
                .and_then(|def_id| {
                    self.get_type(def_id).and_then(|t| match t {
                        Type::Decorator(d) => Some(d.name.clone()),
                        _ => None,
                    })
                })
                .unwrap_or_default();

            if dec_name == "overload" {
                self.check_overload_decorator(type_id, app);
            }
        }

        if let Some(t) = self.get_type_mut(type_id) {
            if let Some(decs) = t.decorators_mut() {
                *decs = decorator_apps;
            }
        }
    }

    /// Check that @overload target operation is in the same container.
    /// Ported from TS decorators.ts areOperationsInSameContainer.
    ///
    /// Two operations are in the same container if:
    /// - Both are in the same interface (by TypeId equality, or by AST node identity
    ///   as a fallback for cloned types in versioned namespaces)
    /// - Both are in the same namespace (by TypeId equality, or by AST node identity)
    pub(crate) fn are_operations_in_same_container(&self, op1: TypeId, op2: TypeId) -> bool {
        let (iface1, ns1) = match self.get_type(op1) {
            Some(Type::Operation(o)) => (o.interface_, o.namespace),
            _ => return false,
        };
        let (iface2, ns2) = match self.get_type(op2) {
            Some(Type::Operation(o)) => (o.interface_, o.namespace),
            _ => return false,
        };

        if iface1.is_some() || iface2.is_some() {
            // Both must have an interface, and they must be the same (by TypeId or AST node)
            if iface1 == iface2 {
                return true;
            }
            // Fallback: compare AST node identity for cloned types in versioned namespaces
            // Ported from TS: op1.interface?.node !== undefined && op1.interface?.node === op2.interface?.node
            let iface1_node = iface1.and_then(|id| {
                self.get_type(id).and_then(|t| match t {
                    Type::Interface(i) => i.node,
                    _ => None,
                })
            });
            let iface2_node = iface2.and_then(|id| {
                self.get_type(id).and_then(|t| match t {
                    Type::Interface(i) => i.node,
                    _ => None,
                })
            });
            return iface1_node.is_some() && iface1_node == iface2_node;
        }

        // Both are namespace-level operations
        if ns1 == ns2 {
            return true;
        }
        // Fallback: compare AST node identity for cloned namespaces in versioned namespaces
        // Ported from TS: op1.namespace?.node !== undefined && op1.namespace?.node === op2.namespace?.node
        let ns1_node = ns1.and_then(|id| {
            self.get_type(id).and_then(|t| match t {
                Type::Namespace(n) => n.node,
                _ => None,
            })
        });
        let ns2_node = ns2.and_then(|id| {
            self.get_type(id).and_then(|t| match t {
                Type::Namespace(n) => n.node,
                _ => None,
            })
        });
        ns1_node.is_some() && ns1_node == ns2_node
    }

    /// Validate @overload decorator: check that the overload base operation
    /// is in the same container (interface or namespace) as the target.
    /// Ported from TS decorators.ts $overload handler.
    pub(crate) fn check_overload_decorator(
        &mut self,
        target_type_id: TypeId,
        app: &DecoratorApplication,
    ) {
        // The first argument to @overload is the overload base operation
        let overload_base_type_id = if let Some(first_arg) = app.args.first() {
            self.node_type_map.get(&first_arg.value).copied()
        } else {
            return;
        };

        let Some(overload_base_type_id) = overload_base_type_id else {
            return;
        };

        // Both must be Operation types
        if !matches!(self.get_type(target_type_id), Some(Type::Operation(_)))
            || !matches!(
                self.get_type(overload_base_type_id),
                Some(Type::Operation(_))
            )
        {
            return;
        }

        if !self.are_operations_in_same_container(target_type_id, overload_base_type_id) {
            self.error(
                "overload-same-parent",
                "Overload must be in the same interface or namespace.",
            );
        }
    }

    // ========================================================================
    // Check property compatible with model indexer
    // ========================================================================

    pub(crate) fn check_property_compatible_with_model_indexer(
        &mut self,
        model_type_id: TypeId,
        prop_type_id: TypeId,
    ) {
        // Ported from TS checker.ts checkPropertyCompatibleWithModelIndexer
        let indexer = match self.get_type(model_type_id) {
            Some(Type::Model(m)) => m.indexer,
            _ => return,
        };
        let (_key_id, value_id) = match indexer {
            Some((k, v)) => (k, v),
            None => return,
        };

        // Get the property's value type
        let prop_value_type = match self.get_type(prop_type_id) {
            Some(Type::ModelProperty(p)) => p.r#type,
            _ => return,
        };

        // Check if property type is assignable to indexer value type
        let (is_assignable, _) = self.is_type_assignable_to(prop_value_type, value_id, 0);
        if !is_assignable {
            let prop_name = type_utils::get_fully_qualified_name(&self.type_store, prop_value_type);
            let index_name = type_utils::get_fully_qualified_name(&self.type_store, value_id);
            self.error("incompatible-indexer", &format!("Property is incompatible with indexer:\n  Type '{}' is not assignable to type '{}'", prop_name, index_name));
        }
    }
}
