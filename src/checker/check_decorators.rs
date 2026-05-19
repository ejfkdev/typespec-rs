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

            // If the decorator was resolved via a using'd namespace, mark that using as used
            if let Some(decl_id) = declaration_type_id {
                self.mark_using_as_used_if_applicable(&decorator_name, decl_id);
            }

            // Check if this is a compiler-internal decorator that cannot be used from user code
            // Ported from TS checkSymbolAccess — check visibility of the resolved declaration
            if let Some(decl_id) = declaration_type_id
                && self.is_internal_type(decl_id)
            {
                self.check_internal_visibility_for(decl_id, &decorator_name);
                // If it's internal and we're not in compiler context, skip it
                if self.internal_declarations.contains(&decl_id)
                    && !self.is_current_context_compiler()
                {
                    continue;
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
                // Skip argument count validation when decorator has no parameter
                // declarations — this means it was registered programmatically
                // without parameter info, so we accept any argument count.
                if !decl.parameters.is_empty() {
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
                        Some(Type::ModelProperty(_)) => "ModelProperty",
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

                // Marshal the argument value for downstream consumers (emitters, etc.)
                let js_value = self.marshal_decorator_arg(arg_id, arg_type);

                args.push(DecoratorArgument {
                    value: arg_id,
                    js_value,
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

        if let Some(t) = self.get_type_mut(type_id)
            && let Some(decs) = t.decorators_mut()
        {
            *decs = decorator_apps;
        }

        // Evaluate well-known decorators: apply their side effects to
        // StateAccessors and Type.doc/summary fields.
        self.evaluate_std_decorators(type_id);
    }

    /// Evaluate well-known TypeSpec decorators after they're stored on a type.
    /// This applies side effects like setting StateAccessors entries and
    /// populating Type.doc/summary fields.
    fn evaluate_std_decorators(&mut self, type_id: TypeId) {
        let ast = match self.ast.as_ref() {
            Some(a) => a.clone(),
            None => return,
        };

        let decorator_apps: Vec<(String, Vec<DecoratorArgument>)> = match self.get_type(type_id) {
            Some(t) => match t.decorators() {
                Some(decs) => decs
                    .iter()
                    .map(|d| {
                        // Primary: try to get name from DecoratorType definition
                        let name = d
                            .definition
                            .and_then(|def_id| {
                                self.get_type(def_id).and_then(|t| match t {
                                    Type::Decorator(dt) => Some(dt.name.clone()),
                                    _ => None,
                                })
                            })
                            .unwrap_or_else(|| {
                                // Fallback: extract name from AST decorator node
                                if let Some(AstNode::DecoratorExpression(expr)) =
                                    ast.id_to_node(d.decorator)
                                {
                                    let full_name = Self::get_identifier_name(&ast, expr.target);
                                    // Extract short name from qualified name (e.g., "TypeSpec.doc" -> "doc")
                                    if let Some(pos) = full_name.rfind('.') {
                                        full_name[pos + 1..].to_string()
                                    } else {
                                        full_name
                                    }
                                } else {
                                    String::new()
                                }
                            });
                        (name, d.args.clone())
                    })
                    .collect(),
                None => return,
            },
            None => return,
        };

        for (dec_name, args) in decorator_apps {
            match dec_name.as_str() {
                "doc" => {
                    if let Some(arg) = args.first()
                        && let Some(DecoratorMarshalledValue::String(doc_text)) = &arg.js_value
                    {
                        self.set_type_doc(type_id, doc_text.clone());
                        crate::libs::compiler::apply_doc_with_source(
                            &mut self.state_accessors,
                            type_id,
                            doc_text,
                            crate::intrinsic_type_state::DocSource::Decorator,
                        );
                    }
                }
                "summary" => {
                    if let Some(arg) = args.first()
                        && let Some(DecoratorMarshalledValue::String(summary_text)) = &arg.js_value
                    {
                        self.set_type_summary(type_id, summary_text.clone());
                        crate::libs::compiler::apply_summary(
                            &mut self.state_accessors,
                            type_id,
                            summary_text,
                        );
                    }
                }
                "minValue" => {
                    if let Some(arg) = args.first()
                        && let Some(DecoratorMarshalledValue::Number(v)) = &arg.js_value
                    {
                        crate::libs::compiler::apply_min_value(
                            &mut self.state_accessors,
                            type_id,
                            *v,
                        );
                    }
                }
                "maxValue" => {
                    if let Some(arg) = args.first()
                        && let Some(DecoratorMarshalledValue::Number(v)) = &arg.js_value
                    {
                        crate::libs::compiler::apply_max_value(
                            &mut self.state_accessors,
                            type_id,
                            *v,
                        );
                    }
                }
                "minValueExclusive" => {
                    if let Some(arg) = args.first()
                        && let Some(DecoratorMarshalledValue::Number(v)) = &arg.js_value
                    {
                        crate::libs::compiler::apply_min_value_exclusive(
                            &mut self.state_accessors,
                            type_id,
                            *v,
                        );
                    }
                }
                "maxValueExclusive" => {
                    if let Some(arg) = args.first()
                        && let Some(DecoratorMarshalledValue::Number(v)) = &arg.js_value
                    {
                        crate::libs::compiler::apply_max_value_exclusive(
                            &mut self.state_accessors,
                            type_id,
                            *v,
                        );
                    }
                }
                "minLength" => {
                    if let Some(arg) = args.first()
                        && let Some(DecoratorMarshalledValue::Number(v)) = &arg.js_value
                    {
                        crate::libs::compiler::apply_min_length(
                            &mut self.state_accessors,
                            type_id,
                            *v as i64,
                        );
                    }
                }
                "maxLength" => {
                    if let Some(arg) = args.first()
                        && let Some(DecoratorMarshalledValue::Number(v)) = &arg.js_value
                    {
                        crate::libs::compiler::apply_max_length(
                            &mut self.state_accessors,
                            type_id,
                            *v as i64,
                        );
                    }
                }
                "pattern" => {
                    if let Some(arg) = args.first()
                        && let Some(DecoratorMarshalledValue::String(p)) = &arg.js_value
                    {
                        crate::libs::compiler::apply_pattern(
                            &mut self.state_accessors,
                            type_id,
                            p,
                            None,
                        );
                    }
                }
                "format" => {
                    if let Some(arg) = args.first()
                        && let Some(DecoratorMarshalledValue::String(f)) = &arg.js_value
                    {
                        crate::libs::compiler::apply_format(&mut self.state_accessors, type_id, f);
                    }
                }
                "minItems" => {
                    if let Some(arg) = args.first()
                        && let Some(DecoratorMarshalledValue::Number(v)) = &arg.js_value
                        && let Ok(n) = crate::numeric::Numeric::new(&v.to_string())
                    {
                        crate::intrinsic_type_state::set_min_items(
                            &mut self.state_accessors,
                            type_id,
                            &n,
                        );
                    }
                }
                "maxItems" => {
                    if let Some(arg) = args.first()
                        && let Some(DecoratorMarshalledValue::Number(v)) = &arg.js_value
                        && let Ok(n) = crate::numeric::Numeric::new(&v.to_string())
                    {
                        crate::intrinsic_type_state::set_max_items(
                            &mut self.state_accessors,
                            type_id,
                            &n,
                        );
                    }
                }
                "error" => {
                    crate::libs::compiler::apply_error(&mut self.state_accessors, type_id);
                }
                "tag" => {
                    if let Some(arg) = args.first()
                        && let Some(DecoratorMarshalledValue::String(t)) = &arg.js_value
                    {
                        crate::libs::compiler::apply_tag(&mut self.state_accessors, type_id, t);
                    }
                }
                "discriminator" => {
                    if let Some(arg) = args.first()
                        && let Some(DecoratorMarshalledValue::String(d)) = &arg.js_value
                    {
                        crate::libs::compiler::apply_discriminator(
                            &mut self.state_accessors,
                            type_id,
                            d,
                        );
                    }
                }
                "encode" => {
                    if let Some(arg) = args.first()
                        && let Some(DecoratorMarshalledValue::String(e)) = &arg.js_value
                    {
                        crate::libs::compiler::apply_encode(
                            &mut self.state_accessors,
                            type_id,
                            Some(e),
                            None,
                        );
                    }
                }
                "patch" => {
                    // Check if {implicitOptionality: false} was explicitly passed
                    let mut explicit_opt_out = false;
                    if let Some(arg) = args.first() {
                        if let Some(DecoratorMarshalledValue::Record(ref map)) = arg.js_value {
                            if let Some(_type_id) = map.get("implicitOptionality") {
                                // User explicitly passed the option, treat as opt-out of warning
                                explicit_opt_out = true;
                            }
                        } else if let Some(DecoratorMarshalledValue::Boolean(b)) = arg.js_value {
                            // Edge case: @patch(false) — also opt out
                            explicit_opt_out = !b;
                        }
                    }
                    crate::libs::http::operation::apply_patch(
                        &mut self.state_accessors,
                        type_id,
                        None,
                    );
                    if !explicit_opt_out {
                        self.warning(
                            "deprecated-implicit-optionality",
                            "@patch with implicit optionality is deprecated. Pass {implicitOptionality: false} to opt out.",
                        );
                    }
                }

                // ---- HTTP verb decorators ----
                "get" => {
                    crate::libs::http::apply_verb(
                        &mut self.state_accessors,
                        type_id,
                        crate::libs::http::types::HttpVerb::Get,
                    );
                }
                "post" => {
                    crate::libs::http::apply_verb(
                        &mut self.state_accessors,
                        type_id,
                        crate::libs::http::types::HttpVerb::Post,
                    );
                }
                "put" => {
                    crate::libs::http::apply_verb(
                        &mut self.state_accessors,
                        type_id,
                        crate::libs::http::types::HttpVerb::Put,
                    );
                }
                "delete" => {
                    crate::libs::http::apply_verb(
                        &mut self.state_accessors,
                        type_id,
                        crate::libs::http::types::HttpVerb::Delete,
                    );
                }
                "head" => {
                    crate::libs::http::apply_verb(
                        &mut self.state_accessors,
                        type_id,
                        crate::libs::http::types::HttpVerb::Head,
                    );
                }

                // ---- HTTP route decorator ----
                "route" => {
                    if let Some(arg) = args.first()
                        && let Some(DecoratorMarshalledValue::String(route_str)) = &arg.js_value
                    {
                        crate::libs::http::apply_route(
                            &mut self.state_accessors,
                            type_id,
                            route_str,
                        );
                    }
                }

                // ---- HTTP parameter/metadata decorators ----
                "header" => {
                    let name = args.first().and_then(|a| {
                        a.js_value.as_ref().and_then(|v| match v {
                            DecoratorMarshalledValue::String(s) => Some(s.clone()),
                            _ => None,
                        })
                    });
                    crate::libs::http::apply_header(
                        &mut self.state_accessors,
                        type_id,
                        name.as_deref(),
                    );
                }
                "query" => {
                    let name = args.first().and_then(|a| {
                        a.js_value.as_ref().and_then(|v| match v {
                            DecoratorMarshalledValue::String(s) => Some(s.clone()),
                            _ => None,
                        })
                    });
                    crate::libs::http::apply_query(
                        &mut self.state_accessors,
                        type_id,
                        name.as_deref(),
                    );
                }
                "path" => {
                    let name = args.first().and_then(|a| {
                        a.js_value.as_ref().and_then(|v| match v {
                            DecoratorMarshalledValue::String(s) => Some(s.clone()),
                            _ => None,
                        })
                    });
                    crate::libs::http::apply_path(
                        &mut self.state_accessors,
                        type_id,
                        name.as_deref(),
                    );
                }
                "body" => {
                    crate::libs::http::apply_body(&mut self.state_accessors, type_id);
                }
                "bodyRoot" => {
                    crate::libs::http::apply_body_root(&mut self.state_accessors, type_id);
                }
                "bodyIgnore" => {
                    crate::libs::http::apply_body_ignore(&mut self.state_accessors, type_id);
                }
                "statusCode" => {
                    crate::libs::http::apply_status_code(&mut self.state_accessors, type_id);
                }
                "cookie" => {
                    let name = args.first().and_then(|a| {
                        a.js_value.as_ref().and_then(|v| match v {
                            DecoratorMarshalledValue::String(s) => Some(s.clone()),
                            _ => None,
                        })
                    });
                    crate::libs::http::apply_cookie(
                        &mut self.state_accessors,
                        type_id,
                        name.as_deref(),
                    );
                }
                "multipartBody" => {
                    crate::libs::http::apply_multipart_body(&mut self.state_accessors, type_id);
                }

                // ---- HTTP visibility decorator ----
                "visibility" => {
                    // @visibility takes one or more enum member references
                    // For now, store the argument TypeIds as a simple marker
                    crate::libs::http::apply_visibility_decorator(
                        &mut self.state_accessors,
                        type_id,
                    );
                }

                // ---- HTTP shared route decorator ----
                "sharedRoute" => {
                    crate::libs::http::apply_shared_route(&mut self.state_accessors, type_id);
                }

                // ---- HTTP server decorator ----
                "server" => {
                    if let Some(arg) = args.first()
                        && let Some(DecoratorMarshalledValue::String(url)) = &arg.js_value
                    {
                        let description = args.get(1).and_then(|a| {
                            a.js_value.as_ref().and_then(|v| match v {
                                DecoratorMarshalledValue::String(s) => Some(s.clone()),
                                _ => None,
                            })
                        });
                        crate::libs::http::apply_server(
                            &mut self.state_accessors,
                            type_id,
                            url,
                            description.as_deref(),
                        );
                    }
                }

                // ---- HTTP useAuth decorator ----
                "useAuth" => {
                    crate::libs::http::apply_use_auth(&mut self.state_accessors, type_id);
                }

                // ---- HTTP service decorator ----
                "service" => {
                    crate::libs::http::apply_service(&mut self.state_accessors, type_id);
                }
                _ => {}
            }
        }
    }

    /// Set the `doc` field on a type.
    fn set_type_doc(&mut self, type_id: TypeId, doc: String) {
        if let Some(t) = self.get_type_mut(type_id) {
            match t {
                Type::Model(m) => m.doc = Some(doc),
                Type::ModelProperty(p) => p.doc = Some(doc),
                Type::Interface(i) => i.doc = Some(doc),
                Type::Operation(o) => o.doc = Some(doc),
                Type::Enum(e) => e.doc = Some(doc),
                Type::EnumMember(m) => m.doc = Some(doc),
                Type::Union(u) => u.doc = Some(doc),
                Type::UnionVariant(v) => v.doc = Some(doc),
                Type::Scalar(s) => s.doc = Some(doc),
                Type::Namespace(ns) => ns.doc = Some(doc),
                _ => {}
            }
        }
    }

    /// Set the `summary` field on a type.
    fn set_type_summary(&mut self, type_id: TypeId, summary: String) {
        if let Some(t) = self.get_type_mut(type_id) {
            match t {
                Type::Model(m) => m.summary = Some(summary),
                Type::ModelProperty(p) => p.summary = Some(summary),
                Type::Interface(i) => i.summary = Some(summary),
                Type::Operation(o) => o.summary = Some(summary),
                Type::Enum(e) => e.summary = Some(summary),
                Type::EnumMember(m) => m.summary = Some(summary),
                Type::Union(u) => u.summary = Some(summary),
                Type::UnionVariant(v) => v.summary = Some(summary),
                Type::Scalar(s) => s.summary = Some(summary),
                Type::Namespace(ns) => ns.summary = Some(summary),
                _ => {}
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

    /// Marshal a decorator argument AST node into a `DecoratorMarshalledValue`.
    /// This converts literal values (strings, numbers, booleans, null) and type
    /// references into a structured form that downstream consumers (emitters,
    /// WASM extensions) can use without going back to the AST.
    fn marshal_decorator_arg(
        &self,
        arg_node_id: NodeId,
        arg_type_id: TypeId,
    ) -> Option<DecoratorMarshalledValue> {
        let ast = self.require_ast()?;

        match ast.id_to_node(arg_node_id) {
            Some(AstNode::StringLiteral(s)) => {
                Some(DecoratorMarshalledValue::String(s.value.clone()))
            }
            Some(AstNode::NumericLiteral(n)) => Some(DecoratorMarshalledValue::Number(n.value)),
            Some(AstNode::BooleanLiteral(b)) => Some(DecoratorMarshalledValue::Boolean(b.value)),
            Some(AstNode::ObjectLiteral(obj)) => {
                let mut record = HashMap::new();
                for &prop_id in &obj.properties {
                    match ast.id_to_node(prop_id) {
                        Some(AstNode::ObjectLiteralProperty(prop)) => {
                            let key = Self::get_identifier_name(&ast, prop.key);
                            let val_type = self.node_type_map.get(&prop.value).copied();
                            if let Some(val_id) = val_type {
                                record.insert(key, val_id);
                            }
                        }
                        Some(AstNode::ObjectLiteralSpreadProperty(_)) => {}
                        _ => {}
                    }
                }
                Some(DecoratorMarshalledValue::Record(record))
            }
            Some(AstNode::ModelExpression(model_expr)) => {
                // `{name: "X-Request-Id"}` in decorator args is a ModelExpression
                let mut record = HashMap::new();
                for &prop_id in &model_expr.properties {
                    if let Some(AstNode::ModelProperty(prop)) = ast.id_to_node(prop_id) {
                        let key = Self::get_identifier_name(&ast, prop.name);
                        let val_type = self.node_type_map.get(&prop.value).copied();
                        if let Some(val_id) = val_type {
                            record.insert(key, val_id);
                        }
                    }
                }
                Some(DecoratorMarshalledValue::Record(record))
            }
            Some(AstNode::ArrayLiteral(arr)) => {
                let mut elements = Vec::new();
                for &elem_id in &arr.values {
                    if let Some(&elem_type) = self.node_type_map.get(&elem_id) {
                        elements.push(elem_type);
                    }
                }
                Some(DecoratorMarshalledValue::Array(elements))
            }
            _ => {
                // For type references, identifiers, member expressions, etc.
                // Use the resolved type as a Type reference
                if arg_type_id != self.error_type {
                    Some(DecoratorMarshalledValue::Type(arg_type_id))
                } else {
                    None
                }
            }
        }
    }
}
