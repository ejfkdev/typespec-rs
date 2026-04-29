use super::*;

impl Checker {
    pub(crate) fn check_alias(&mut self, ctx: &CheckContext, node_id: NodeId) -> TypeId {
        // Circular reference detection: if we're already checking this node, return error
        if let Some(type_id) = self.check_circular_ref(node_id) {
            return type_id;
        }

        // If already fully checked, return cached type
        if let Some(&type_id) = self.node_type_map.get(&node_id)
            && let Some(t) = self.get_type(type_id)
            && t.is_finished()
        {
            return type_id;
        }

        let (ast, node) = require_ast_node!(self, node_id, AliasStatement, self.error_type);

        let name = Self::get_identifier_name(&ast, node.name);

        self.pending_type_checks.insert(node_id);
        // Track by name so check_type_reference/check_identifier can detect circular references
        if !name.is_empty() {
            self.pending_type_names.insert(name.clone());
        }

        self.check_template_declaration(ctx, &ast, &node.template_parameters);

        let target_type = self.check_node(ctx, node.value);

        self.pending_type_checks.remove(&node_id);
        if !name.is_empty() {
            self.pending_type_names.remove(&name);
        }

        // If target_type is error_type, a circular reference was already reported
        // Still create/update the alias type but point base_scalar to error_type

        let template_node =
            self.compute_template_node(&node.template_parameters, ctx.mapper.as_ref(), node_id);

        // Check if pre-registered type exists
        let type_id = if let Some(&existing_id) = self.node_type_map.get(&node_id) {
            // Update the pre-registered type in-place
            if let Some(t) = self.get_type_mut(existing_id)
                && let Type::Scalar(s) = t
            {
                s.base_scalar = Some(target_type);
                s.template_node = template_node;
                s.is_finished = false;
            }
            existing_id
        } else {
            let new_id = {
                let mut s = ScalarType::new(
                    self.next_type_id(),
                    name.clone(),
                    Some(node_id),
                    self.current_namespace,
                    Some(target_type),
                );
                s.template_node = template_node;
                self.create_type(Type::Scalar(s))
            };

            self.register_type(node_id, new_id, &name, ctx.mapper.as_ref());
            new_id
        };

        // Process directives (e.g., #deprecated) early so deprecated context works
        self.process_and_mark_directives(node_id, type_id);

        self.finalize_type_check(
            ctx,
            type_id,
            node_id,
            &node.template_parameters,
            &[],
            ctx.mapper.as_ref(),
        );
        type_id
    }

    pub(crate) fn check_const(&mut self, ctx: &CheckContext, node_id: NodeId) {
        let (ast, node) = require_ast_node!(self, node_id, ConstStatement);

        let name = Self::get_identifier_name(&ast, node.name);

        // Check for circular const reference
        if self.pending_const_checks.contains(&node_id) {
            self.error(
                "circular-const",
                &format!("const '{}' recursively references itself.", name),
            );
            return;
        }
        self.pending_const_checks.insert(node_id);

        let value_entity = self.check_node_entity(ctx, node.value);

        // Remove from pending after checking
        self.pending_const_checks.remove(&node_id);

        let value_type_id = self.entity_to_type_id(&value_entity);

        let value_id = match &value_entity {
            Entity::Value(vid) => *vid,
            _ => {
                let vid = self.resolve_value_from_type(value_type_id, Some(node_id));
                // Track exact type for scalar inference
                self.value_exact_types.insert(vid, value_type_id);
                vid
            }
        };

        self.node_value_map.insert(node_id, value_id);
        self.node_type_map.insert(node_id, value_type_id);
        if !name.is_empty() {
            self.declared_values.insert(name, value_id);
        }

        // Check type assignability if there's a type annotation
        if let Some(type_annot_id) = node.type_annotation {
            let annot_entity = self.check_node_entity(ctx, type_annot_id);
            let annot_type_id = self.entity_to_type_id(&annot_entity);
            if annot_type_id != self.error_type && value_type_id != self.error_type {
                let (is_assignable, _) =
                    self.is_type_assignable_to(value_type_id, annot_type_id, node_id);
                if !is_assignable {
                    self.error_unassignable("unassignable", value_type_id, annot_type_id);
                }
            }
            // TS: copyValue(value, { type }) — when const has an explicit type,
            // the value's type becomes the declared type, not the literal type.
            // e.g. const a: int32 = 123 → value.type = int32, not Number(123)
            if annot_type_id != self.error_type {
                self.set_value_type(value_id, annot_type_id);

                // Ported from TS checker.ts:6339 inferScalarsFromConstraints
                // When a primitive value has no scalar yet, infer it from the
                // constraint type (the type annotation). This also detects
                // ambiguity (e.g., int32 | int64 for numeric literal).
                self.infer_scalars_from_constraints(value_id, annot_type_id);

                // Also check for ambiguous scalar types (e.g., int32 | int64 with numeric literal)
                // This must happen after infer_scalars_from_constraints so the value type is set.
                self.emit_ambiguous_scalar_diagnostic_and_resolve(
                    Some(annot_type_id),
                    value_type_id,
                );
            }
        }
    }

    /// Resolve a TypeId to a ValueId for const value creation.
    /// Used when converting literal types (String, Number, Boolean, Model, Tuple)
    /// into their corresponding Value types.
    pub(crate) fn resolve_value_from_type(
        &mut self,
        type_id: TypeId,
        node: Option<NodeId>,
    ) -> ValueId {
        match self.get_type(type_id).cloned() {
            Some(Type::String(s)) => self.create_value(Value::StringValue(StringValue {
                type_id,
                value: s.value.clone(),
                scalar: None,
                node,
            })),
            Some(Type::Number(n)) => self.create_value(Value::NumericValue(NumericValue {
                type_id,
                value: n.value,
                scalar: None,
                node,
            })),
            Some(Type::Boolean(b)) => self.create_value(Value::BooleanValue(BooleanValue {
                type_id,
                value: b.value,
                scalar: None,
                node,
            })),
            Some(Type::Model(m)) => {
                let mut properties = Vec::new();
                for prop_name in &m.property_names {
                    if let Some(&prop_id) = m.properties.get(prop_name) {
                        let prop_value_id = match self.get_type(prop_id).cloned() {
                            Some(Type::ModelProperty(mp)) => {
                                self.resolve_value_from_type(mp.r#type, node)
                            }
                            _ => self.create_value(Value::NullValue(NullValue {
                                type_id: self.null_type,
                                node: None,
                            })),
                        };
                        properties.push(ObjectValueProperty {
                            name: prop_name.clone(),
                            value: prop_value_id,
                        });
                    }
                }
                self.create_value(Value::ObjectValue(ObjectValue {
                    type_id,
                    properties,
                    node,
                }))
            }
            Some(Type::Tuple(t)) => {
                let mut values = Vec::new();
                for &elem_type_id in &t.values {
                    let elem_value_id = self.resolve_value_from_type(elem_type_id, node);
                    values.push(elem_value_id);
                }
                self.create_value(Value::ArrayValue(ArrayValue {
                    type_id,
                    values,
                    node,
                }))
            }
            // Ported from TS 5cf48cbab: when a TemplateParameter has a valueof
            // constraint, synthesize a TemplateValue placeholder instead of NullValue.
            // This allows template parameters with valueof constraints to be used
            // in default value, function argument, and template argument positions.
            Some(Type::TemplateParameter(tp)) => {
                if let Some(constraint_id) = tp.constraint {
                    if self.is_value_type(constraint_id) {
                        self.create_value(Value::TemplateValue(TemplateValue {
                            type_id: constraint_id,
                        }))
                    } else {
                        self.create_value(Value::NullValue(NullValue { type_id, node }))
                    }
                } else {
                    self.create_value(Value::NullValue(NullValue { type_id, node }))
                }
            }
            _ => self.create_value(Value::NullValue(NullValue { type_id, node })),
        }
    }

    /// Check if a type is a value type (can be used in valueof position).
    /// Value types include: string, numeric, boolean, and scalar extending those.
    pub(crate) fn is_value_type(&self, type_id: TypeId) -> bool {
        match self.get_type(type_id) {
            Some(Type::Intrinsic(_)) => false, // Void, Never, Unknown, Null are NOT value types
            Some(Type::String(_)) | Some(Type::Number(_)) | Some(Type::Boolean(_)) => true,
            Some(Type::Scalar(_)) => true, // scalars extend primitive types
            Some(Type::Union(_)) => true,  // unions of value types
            Some(Type::Enum(_)) => true,   // enums are value types
            _ => false,
        }
    }
}
