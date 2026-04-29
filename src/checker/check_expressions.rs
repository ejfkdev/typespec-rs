//! Expression checking
//!
//! Ported from TypeSpec compiler expression checking methods

use super::*;

impl Checker {
    // Expression checking
    // ========================================================================

    pub(crate) fn check_array_expression(&mut self, ctx: &CheckContext, node_id: NodeId) -> TypeId {
        let (_, node) = require_ast_node!(self, node_id, ArrayExpression, self.error_type);

        let element_type = self.check_node(ctx, node.element_type);

        let integer_scalar = self.std_types.get("integer").copied();

        let type_id = {
            let mut m = ModelType::new(
                self.next_type_id(),
                "Array".to_string(),
                Some(node_id),
                None,
            );
            m.indexer = Some((integer_scalar.unwrap_or(self.error_type), element_type));
            m.is_finished = true;
            self.create_type(Type::Model(m))
        };

        self.node_type_map.insert(node_id, type_id);
        type_id
    }

    pub(crate) fn check_tuple_expression(&mut self, ctx: &CheckContext, node_id: NodeId) -> TypeId {
        let (_, node) = require_ast_node!(self, node_id, TupleExpression, self.error_type);

        let mut values = Vec::new();
        for &val_id in &node.values {
            let val_type = self.check_node(ctx, val_id);
            values.push(val_type);
        }

        let type_id = self.create_type(Type::Tuple(TupleType {
            id: self.next_type_id(),
            node: Some(node_id),
            values,
            is_finished: true,
        }));

        self.node_type_map.insert(node_id, type_id);
        type_id
    }

    pub(crate) fn check_union_expression(&mut self, ctx: &CheckContext, node_id: NodeId) -> TypeId {
        let (_, node) = require_ast_node!(self, node_id, UnionExpression, self.error_type);

        let mut variants = HashMap::new();
        let mut variant_names = Vec::new();
        for (i, &option_id) in node.options.iter().enumerate() {
            // Ported from TS parser.ts:1302-1308 — fn-in-union-expression
            // Check if a function type expression appears directly in a union
            // without being parenthesized
            if let Some(ast) = self.require_ast()
                && matches!(
                    ast.id_to_node(option_id),
                    Some(AstNode::FunctionTypeExpression(_))
                )
            {
                self.error(
                    "fn-in-union-expression",
                    "Function types in anonymous union expressions must be parenthesized.",
                );
            }

            let option_type = self.check_node(ctx, option_id);

            let variant_name = format!("v{}", i);
            let variant_id = self.create_type(Type::UnionVariant(UnionVariantType {
                id: self.next_type_id(),
                name: variant_name.clone(),
                node: Some(option_id),
                r#type: option_type,
                union: None,
                decorators: Vec::new(),
                is_finished: true,
            }));

            variants.insert(variant_name.clone(), variant_id);
            variant_names.push(variant_name);
        }

        let type_id = {
            let mut u = UnionType::new(
                self.next_type_id(),
                String::new(),
                Some(node_id),
                self.current_namespace,
                true,
            );
            u.variants = variants.clone();
            u.variant_names = variant_names.clone();
            u.template_node = if ctx.mapper.is_some() {
                Some(node_id)
            } else {
                None
            };
            u.is_finished = true;
            self.create_type(Type::Union(u))
        };

        // Re-parent all variants to the new union
        for name in &variant_names {
            if let Some(&variant_id) = variants.get(name)
                && let Some(t) = self.get_type_mut(variant_id)
                && let Type::UnionVariant(v) = t
            {
                v.union = Some(type_id);
            }
        }

        // TS: linkMapper(unionType, ctx.mapper) — Propagate mapper to inner union
        // expressions created during template instantiation, so isTemplateInstance()
        // returns true for them.
        if let Some(ref mapper) = ctx.mapper {
            self.link_template_mapper(type_id, mapper.clone());
        }

        self.node_type_map.insert(node_id, type_id);
        type_id
    }

    pub(crate) fn check_intersection_expression(
        &mut self,
        ctx: &CheckContext,
        node_id: NodeId,
    ) -> TypeId {
        let (_, node) = require_ast_node!(self, node_id, IntersectionExpression, self.error_type);

        let mut merged_properties: HashMap<String, TypeId> = HashMap::new();
        let mut merged_property_names: Vec<String> = Vec::new();
        // Track the resolved source types for type_relation
        let mut source_models: Vec<crate::checker::types::SourceModel> = Vec::new();

        for &option_id in &node.options {
            let option_type = self.check_node(ctx, option_id);

            // Resolve through alias chains to get the underlying type
            let resolved = self.resolve_alias_chain(option_type);
            let is_model = matches!(self.get_type(resolved), Some(Type::Model(_)));
            if !is_model && option_type != self.error_type {
                self.error(
                    "intersect-non-model",
                    "Cannot intersect a non-model type. Intersection operands must be models.",
                );
            }

            // Check for intersect-invalid-index: array model in intersection
            if let Some(Type::Model(m)) = self.get_type(resolved)
                && m.indexer.is_some()
            {
                self.error(
                    "intersect-invalid-index",
                    "Cannot intersect an array model with a non-array model.",
                );
            }

            // Track source types for type_relation intersection resolution
            // If the resolved type is a model, track it; otherwise track the alias (scalar)
            if is_model {
                source_models.push(crate::checker::types::SourceModel {
                    usage: crate::checker::types::SourceModelUsage::Intersection,
                    model: resolved,
                    node: Some(option_id),
                });
            } else if option_type != self.error_type {
                source_models.push(crate::checker::types::SourceModel {
                    usage: crate::checker::types::SourceModelUsage::Intersection,
                    model: option_type,
                    node: Some(option_id),
                });
            }

            // Collect properties including inherited ones, cloning with source_property
            // Also detect intersect-duplicate-property
            self.collect_model_properties_with_source(
                option_type,
                &mut merged_properties,
                &mut merged_property_names,
                true, // report_duplicate_property
            );
        }

        // Special case: intersection of non-model types (scalars, literals)
        // e.g., "string & string" = string, "int32 & numeric" = int32, "string & int32" = never
        // TS resolves these at the type level, not as models
        if source_models.len() >= 2 && merged_properties.is_empty() {
            // All operands are non-model types (scalars, literals, etc.)
            // Compute the intersection result directly
            let source_model_ids: Vec<TypeId> = source_models.iter().map(|s| s.model).collect();
            let type_id = self.compute_non_model_intersection(&source_model_ids);
            self.node_type_map.insert(node_id, type_id);
            return type_id;
        }

        let result_model_id = self.next_type_id();
        // Set model reference on all collected properties
        for name in &merged_property_names {
            if let Some(&prop_id) = merged_properties.get(name)
                && let Some(prop) = self.get_type_mut(prop_id)
                && let Type::ModelProperty(p) = prop
            {
                p.model = Some(result_model_id);
            }
        }

        let type_id = {
            let mut m = ModelType::new(
                result_model_id,
                String::new(),
                Some(node_id),
                self.current_namespace,
            );
            m.properties = merged_properties;
            m.property_names = merged_property_names;
            m.source_models = source_models;
            m.is_finished = true;
            self.create_type(Type::Model(m))
        };

        self.node_type_map.insert(node_id, type_id);
        type_id
    }

    /// Compute the result of intersecting non-model types (scalars, literals)
    /// Returns the effective TypeId: same type → that type, subtype → subtype, incompatible → never
    pub(crate) fn compute_non_model_intersection(&mut self, type_ids: &[TypeId]) -> TypeId {
        // For two-type intersections:
        // - Same type → return that type
        // - Subtype relationship → return the subtype (more specific)
        // - Incompatible → return never
        if type_ids.len() == 2 {
            let a = type_ids[0];
            let b = type_ids[1];

            // Same TypeId → same type
            if a == b {
                return a;
            }

            // Check if A is assignable to B (A is subtype of B)
            let a_to_b = self
                .type_relation
                .is_related_with_store(&self.type_store, a, b);
            // Check if B is assignable to A (B is subtype of A)
            let b_to_a = self
                .type_relation
                .is_related_with_store(&self.type_store, b, a);

            if a_to_b.is_true() && b_to_a.is_true() {
                // Both assignable to each other → equivalent → return A
                return a;
            } else if a_to_b.is_true() {
                // A is subtype of B → return A (more specific)
                return a;
            } else if b_to_a.is_true() {
                // B is subtype of A → return B (more specific)
                return b;
            }
        }

        // Incompatible types or >2 operands → return never
        self.never_type
    }

    /// Collect model properties with source_property tracing for effective type resolution.
    /// Clones properties and sets source_property to the original.
    pub(crate) fn collect_model_properties_with_source(
        &mut self,
        type_id: TypeId,
        properties: &mut HashMap<String, TypeId>,
        property_names: &mut Vec<String>,
        report_duplicate_property: bool,
    ) {
        match self.get_type(type_id) {
            Some(Type::Model(_m)) => {
                // Walk inherited properties (self + base chain)
                let mut current: Option<TypeId> = Some(type_id);
                while let Some(mid) = current {
                    let cur_props: Vec<(String, TypeId)> = match self.get_type(mid) {
                        Some(Type::Model(cur_model)) => {
                            let result: Vec<(String, TypeId)> = cur_model
                                .property_names
                                .iter()
                                .filter_map(|name| {
                                    cur_model.properties.get(name).map(|&v| (name.clone(), v))
                                })
                                .collect();
                            current = cur_model.base_model;
                            result
                        }
                        _ => break,
                    };
                    for (name, src_prop_id) in cur_props {
                        if properties.contains_key(&name) {
                            // Duplicate property in intersection
                            if report_duplicate_property {
                                self.error("intersect-duplicate-property", &format!("Property '{}' has conflicting definitions in intersection.", name));
                            }
                        } else {
                            // Clone property and set source_property
                            let cloned_id = self.clone_type(src_prop_id);
                            if let Some(prop) = self.get_type_mut(cloned_id)
                                && let Type::ModelProperty(p) = prop
                            {
                                p.source_property = Some(src_prop_id);
                                // model will be set after the intersection model is created
                            }
                            properties.insert(name.clone(), cloned_id);
                            property_names.push(name);
                        }
                    }
                }
            }
            Some(Type::Scalar(s)) => {
                if let Some(base_id) = s.base_scalar {
                    self.collect_model_properties_with_source(
                        base_id,
                        properties,
                        property_names,
                        report_duplicate_property,
                    );
                }
            }
            _ => {}
        }
    }

    pub(crate) fn check_valueof(&mut self, ctx: &CheckContext, node_id: NodeId) -> TypeId {
        let (_, node) = require_ast_node!(self, node_id, ValueOfExpression, self.error_type);

        self.check_node(ctx, node.target)
    }

    pub(crate) fn check_typeof(&mut self, ctx: &CheckContext, node_id: NodeId) -> TypeId {
        let (ast, node) = require_ast_node!(self, node_id, TypeOfExpression, self.error_type);

        // typeof expects a value reference, not a type reference.
        // First check if target is a value (const variable), then fall back to type.
        // Check if target is an identifier referencing a declared value
        if let Some(AstNode::Identifier(ident)) = ast.id_to_node(node.target)
            && let Some(&value_id) = self.declared_values.get(&ident.value)
        {
            // Found a value reference - return its type
            // Prefer value_type (which may be updated by set_value_type for typed consts)
            // over exact_type (which is the original literal type)
            return self
                .get_value(value_id)
                .map(|v| v.value_type())
                .or_else(|| self.get_value_exact_type(value_id))
                .unwrap_or(self.error_type);
        }
        // Also check TypeReference with simple name
        if let Some(AstNode::TypeReference(ref_node)) = ast.id_to_node(node.target) {
            let name = Self::get_identifier_name(&ast, ref_node.name);
            if let Some(&value_id) = self.declared_values.get(&name) {
                return self
                    .get_value(value_id)
                    .map(|v| v.value_type())
                    .or_else(|| self.get_value_exact_type(value_id))
                    .unwrap_or(self.error_type);
            }
        }

        // Check via check_node_entity for other cases
        let entity = self.check_node_entity(ctx, node.target);
        match &entity {
            Entity::Value(value_id) => self
                .get_value(*value_id)
                .map(|v| v.value_type())
                .or_else(|| self.get_value_exact_type(*value_id))
                .unwrap_or(self.error_type),
            Entity::Indeterminate(type_id) => *type_id,
            Entity::Type(_) => {
                // typeof on a pure type (like int32, a model) - emit expect-value diagnostic
                self.error(
                    "expect-value",
                    "typeof must be used with a value, not a type.",
                );
                self.error_type
            }
            Entity::MixedConstraint(c) => {
                // typeof on a mixed constraint - return the type constraint if available
                c.type_constraint.unwrap_or(self.error_type)
            }
        }
    }

    /// Check a string template part (Head/Middle/Tail) — they all just produce a string literal type.
    pub(crate) fn check_string_template_part(&mut self, node_id: NodeId) -> TypeId {
        let ast = require_ast_or!(self, self.error_type);
        let value = match ast.id_to_node(node_id) {
            Some(AstNode::StringTemplateHead(h)) => h.value.clone(),
            Some(AstNode::StringTemplateMiddle(m)) => m.value.clone(),
            Some(AstNode::StringTemplateTail(t)) => t.value.clone(),
            _ => return self.error_type,
        };
        let type_id = self.create_literal_type_string(value);
        self.node_type_map.insert(node_id, type_id);
        type_id
    }

    pub(crate) fn check_string_template(&mut self, node_id: NodeId) -> TypeId {
        let (ast, node) =
            require_ast_node!(self, node_id, StringTemplateExpression, self.error_type);

        let ctx = CheckContext::new();
        let mut span_type_ids = Vec::new();

        // Process head (the initial literal before the first interpolation)
        // In the AST, head is a StringTemplateHead node
        if let Some(AstNode::StringTemplateHead(head)) = ast.id_to_node(node.head) {
            let literal_type_id = self.create_literal_type_string(head.value.clone());
            self.node_type_map.insert(node.head, literal_type_id);
            let span_id = self.create_type(Type::StringTemplateSpan(StringTemplateSpanType {
                id: self.next_type_id(),
                node: Some(node.head),
                r#type: Some(literal_type_id),
                expression: None,
                is_finished: true,
            }));
            span_type_ids.push(span_id);
        }

        // Process each span (interpolation + following literal)
        // Track whether we have both value and type interpolations (mixed-string-template)
        let mut has_value_interp = false;
        let mut has_type_interp = false;

        for &span_node_id in &node.spans {
            if let Some(AstNode::StringTemplateSpan(span)) = ast.id_to_node(span_node_id) {
                // Check the interpolated expression
                let expr_type_id = self.check_node(&ctx, span.expression);

                // Determine if this interpolation is a value or a type
                // Check the expression node to determine its nature
                let expr_name = match ast.id_to_node(span.expression) {
                    Some(AstNode::Identifier(ident)) => Some(ident.value.clone()),
                    Some(AstNode::TypeReference(tr)) => {
                        let name = Self::get_identifier_name(&ast, tr.name);
                        if name.is_empty() { None } else { Some(name) }
                    }
                    Some(AstNode::MemberExpression(me)) => {
                        let name = Self::get_identifier_name(&ast, me.object);
                        if name.is_empty() { None } else { Some(name) }
                    }
                    _ => None,
                };

                if let Some(name) = &expr_name
                    && self.declared_values.contains_key(name)
                {
                    has_value_interp = true;
                } else if let Some(name) = &expr_name
                    && self.declared_types.contains_key(name)
                {
                    has_type_interp = true;
                }

                // If the expression type is a string literal, it's a value
                if let Some(resolved) = self.get_type(expr_type_id) {
                    match resolved {
                        Type::String(_) | Type::Number(_) | Type::Boolean(_) => {
                            // Literal types are serializable in string templates
                        }
                        Type::StringTemplate(_) => {
                            // String templates are OK if they are themselves serializable
                            has_type_interp = true;
                        }
                        Type::Intrinsic(_) => {
                            // Error type - don't emit non-literal-string-template
                            // (the error will be reported separately)
                        }
                        Type::TemplateParameter(_) => {
                            // Template parameters may be serializable depending on constraint
                            has_type_interp = true;
                        }
                        Type::Scalar(s) if s.base_scalar.is_some() => {
                            // Alias type - treat as type interpolation
                            // Check if the resolved type is a literal (non-literal-string-template)
                            let Some(base) = s.base_scalar else { continue };
                            let base_resolved = self.resolve_alias_chain(base);
                            if let Some(base_type) = self.get_type(base_resolved) {
                                match base_type {
                                    Type::String(_) | Type::Number(_) | Type::Boolean(_) => {}
                                    Type::Intrinsic(_) => {}
                                    _ => {
                                        self.error("non-literal-string-template", "Value interpolated in this string template cannot be converted to a string. Only literal types can be automatically interpolated.");
                                    }
                                }
                            }
                            has_type_interp = true;
                        }
                        _ => {
                            // Non-literal types cannot be interpolated in string templates
                            self.error("non-literal-string-template", "Value interpolated in this string template cannot be converted to a string. Only literal types can be automatically interpolated.");
                            has_type_interp = true;
                        }
                    }
                }

                // Create an interpolated span
                let interp_span_id =
                    self.create_type(Type::StringTemplateSpan(StringTemplateSpanType {
                        id: self.next_type_id(),
                        node: Some(span_node_id),
                        r#type: Some(expr_type_id),
                        expression: Some(span.expression),
                        is_finished: true,
                    }));
                span_type_ids.push(interp_span_id);

                // Check the literal part (StringTemplateMiddle or StringTemplateTail)
                let literal_type_id = self.check_node(&ctx, span.literal);
                // The literal part is a String node
                let literal_span_id =
                    self.create_type(Type::StringTemplateSpan(StringTemplateSpanType {
                        id: self.next_type_id(),
                        node: Some(span.literal),
                        r#type: Some(literal_type_id),
                        expression: None,
                        is_finished: true,
                    }));
                span_type_ids.push(literal_span_id);
            }
        }

        // Check for mixed-string-template: interpolating both values and types
        if has_value_interp && has_type_interp {
            self.error(
                "mixed-string-template",
                "String template cannot interpolate both values and types.",
            );
        }

        // If there are no interpolations, it's just a plain string
        if span_type_ids.len() == 1 {
            // Only head - no interpolations, return the string type directly
            if let Some(Type::StringTemplateSpan(s)) = self.get_type(span_type_ids[0]).cloned()
                && let Some(type_id) = s.r#type
            {
                self.node_type_map.insert(node_id, type_id);
                return type_id;
            }
        }

        let type_id = self.create_type(Type::StringTemplate(StringTemplateType {
            id: self.next_type_id(),
            node: Some(node_id),
            spans: span_type_ids,
            string_value: None,
            is_finished: true,
        }));

        self.node_type_map.insert(node_id, type_id);
        type_id
    }

    pub(crate) fn check_call_expression(&mut self, ctx: &CheckContext, node_id: NodeId) -> TypeId {
        let (_, call_node) = require_ast_node!(self, node_id, CallExpression, self.error_type);
        let (target_id, arguments) = (call_node.target, call_node.arguments.clone());

        let target_type = self.check_node(ctx, target_id);

        // Ported from TS checker.ts:4543-4586 — checkCallExpression
        // Validate that the target is callable (Scalar, ScalarConstructor, or FunctionType)
        if target_type != self.error_type {
            let resolved = self.resolve_alias_chain(target_type);
            let is_callable = match self.get_type(resolved) {
                Some(Type::Scalar(_))
                | Some(Type::ScalarConstructor(_))
                | Some(Type::FunctionType(_)) => true,
                Some(Type::TemplateParameter(_)) => {
                    // Template parameters need a callable constraint
                    // For now, assume not callable (TS checks constraintIsCallable)
                    self.error("non-callable", "Template parameter is not callable. Ensure it is constrained to a function value or callable type.");
                    false
                }
                Some(Type::Intrinsic(_)) => false, // Error type — don't emit non-callable
                _ => {
                    let kind_name = match self.get_type(resolved) {
                        Some(t) => t.kind_name(),
                        None => "unknown",
                    };
                    self.error(
                        "non-callable",
                        &format!("Type {} is not callable.", kind_name),
                    );
                    false
                }
            };

            if is_callable {
                // Check arguments for scalar constructor calls
                // Ported from TS checker.ts:4629-4643 — checkPrimitiveArg
                // Use target_type (the actual scalar like int8) for range checks,
                // NOT resolved (which would be the base like numeric via resolve_alias_chain).
                let mut is_primitive_or_extends = false;
                let scalar_target = if let Some(Type::Scalar(_)) = self.get_type(target_type) {
                    target_type
                } else {
                    resolved
                };
                if let Some(Type::Scalar(s)) = self.get_type(scalar_target) {
                    // Check if scalar IS a primitive or derives from a primitive
                    // (string/numeric/boolean). Root primitives have no base_scalar
                    // but are themselves primitives.
                    is_primitive_or_extends = self.scalar_extends_primitive(scalar_target);

                    if is_primitive_or_extends {
                        // Primitive scalar init: must have exactly 1 argument
                        if arguments.len() != 1 {
                            self.error("invalid-primitive-init", "Instantiating scalar deriving from 'string', 'numeric' or 'boolean' can only take a single argument.");
                        }
                    } else if s.base_scalar.is_some() {
                        // Non-primitive scalar: requires named constructor
                        // Ported from TS checker.ts:4781 — named-init-required
                        self.error("named-init-required", "Only scalar deriving from 'string', 'numeric' or 'boolean' can be instantiated without a named constructor.");
                    }
                }

                // Check arguments
                for &arg_id in &arguments {
                    self.check_node(ctx, arg_id);
                }

                // Ported from TS checker.ts:4651 — checkValueOfType
                // For primitive scalar constructors, validate the argument value
                // is assignable to the scalar type (e.g., int8(128) should fail)
                // Use scalar_target (the actual scalar, e.g. int8) not resolved (e.g. numeric).
                if is_primitive_or_extends && arguments.len() == 1 {
                    let arg_id = arguments[0];
                    if let Some(&arg_type_id) = self.node_type_map.get(&arg_id) {
                        let (is_assignable, _) =
                            self.is_type_assignable_to(arg_type_id, scalar_target, arg_id);
                        if !is_assignable {
                            self.error_unassignable("unassignable", arg_type_id, scalar_target);
                        }
                    }
                }

                return target_type;
            }
        }

        // Still check arguments even if target is not callable
        for &arg_id in &arguments {
            self.check_node(ctx, arg_id);
        }

        target_type
    }

    pub(crate) fn check_object_literal(&mut self, ctx: &CheckContext, node_id: NodeId) -> TypeId {
        let (ast, node) = require_ast_node!(self, node_id, ObjectLiteral, self.error_type);

        let mut properties = HashMap::new();
        let mut property_names = Vec::new();

        for &prop_id in &node.properties {
            let prop_node = match ast.id_to_node(prop_id) {
                Some(AstNode::ObjectLiteralProperty(p)) => p.clone(),
                Some(AstNode::ObjectLiteralSpreadProperty(spread)) => {
                    // Check if the spread target resolves to an ObjectValue
                    // TS: checkObjectSpreadProperty — if not ObjectValue, emit "spread-object"
                    let spread_type = self.check_node(ctx, spread.target);
                    // Check if the spread type is a Model (which represents an object)
                    let is_model = matches!(self.get_type(spread_type), Some(Type::Model(_)));
                    if !is_model {
                        self.error(
                            "spread-object",
                            "Cannot spread properties of non-object type.",
                        );
                    }
                    continue;
                }
                _ => continue,
            };

            let prop_name = Self::get_identifier_name(&ast, prop_node.key);
            let prop_type = self.check_node(ctx, prop_node.value);

            let model_prop_id = self.create_type(Type::ModelProperty(ModelPropertyType {
                id: self.next_type_id(),
                name: prop_name.clone(),
                node: Some(prop_id),
                r#type: prop_type,
                optional: false,
                default_value: None,
                model: None,
                source_property: None,
                decorators: Vec::new(),
                is_finished: true,
            }));

            properties.insert(prop_name.clone(), model_prop_id);
            property_names.push(prop_name);
        }

        let type_id = {
            let mut m = ModelType::new(
                self.next_type_id(),
                String::new(),
                Some(node_id),
                self.current_namespace,
            );
            m.properties = properties;
            m.property_names = property_names;
            m.is_finished = true;
            self.create_type(Type::Model(m))
        };

        self.node_type_map.insert(node_id, type_id);
        type_id
    }

    pub(crate) fn check_array_literal(&mut self, ctx: &CheckContext, node_id: NodeId) -> TypeId {
        let (_, node) = require_ast_node!(self, node_id, ArrayLiteral, self.error_type);

        let mut values = Vec::new();
        for &item_id in &node.values {
            let item_type = self.check_node(ctx, item_id);
            values.push(item_type);
        }

        let type_id = self.create_type(Type::Tuple(TupleType {
            id: self.next_type_id(),
            node: Some(node_id),
            values,
            is_finished: true,
        }));

        self.node_type_map.insert(node_id, type_id);
        type_id
    }
}
