use super::*;
use crate::parser::AstNode;

impl Checker {
    pub(crate) fn check_scalar(&mut self, ctx: &CheckContext, node_id: NodeId) -> TypeId {
        // Circular reference detection: return the existing type (even if unfinished)
        // to break the cycle, consistent with check_model.
        if let Some(type_id) = self.check_circular_ref(node_id) {
            return type_id;
        }

        let (ast, node) = require_ast_node!(self, node_id, ScalarDeclaration, self.error_type);

        let name = Self::get_identifier_name(&ast, node.name);

        self.check_template_declaration(ctx, &ast, &node.template_parameters);

        if ctx.mapper.is_none()
            && let Some(&type_id) = self.node_type_map.get(&node_id)
            && let Some(t) = self.get_type(type_id)
            && t.is_finished()
        {
            return type_id;
        }

        let base_scalar = if let Some(extends_id) = node.extends {
            // Track by name so check_type_reference can detect circular base references
            if !name.is_empty() {
                self.pending_type_names.insert(name.clone());
                self.pending_base_type_names.insert(name.clone());
            }
            let extends_type = self.check_node(ctx, extends_id);
            if !name.is_empty() {
                self.pending_type_names.remove(&name);
                self.pending_base_type_names.remove(&name);
            }

            // Check that scalar extends another scalar (not a model, etc.)
            if extends_type != self.error_type {
                let resolved = self.resolve_alias_chain(extends_type);
                match self.get_type(resolved) {
                    Some(Type::Model(_)) => {
                        self.error(
                            "extend-scalar",
                            &format!("Scalar '{}' must extend another scalar, not a model.", name),
                        );
                    }
                    Some(Type::Interface(_)) => {
                        self.error(
                            "extend-scalar",
                            &format!("Scalar '{}' must extend another scalar.", name),
                        );
                    }
                    _ => {}
                }
            }

            Some(extends_type)
        } else {
            None
        };

        // Mark this node as currently being type-checked (for circular detection)
        // Insert AFTER all early returns to avoid leaking entries in pending_type_checks.
        self.pending_type_checks.insert(node_id);

        let template_node =
            self.compute_template_node(&node.template_parameters, ctx.mapper.as_ref(), node_id);

        // Check scalar constructors (init members) and detect duplicates
        // TS: checkScalarConstructors — inherits constructors from base, then checks own
        let mut constructor_names: HashMap<String, TypeId> = HashMap::new();

        // Inherit constructors from base scalar
        if let Some(base_id) = base_scalar {
            let resolved_base = self.resolve_alias_chain(base_id);
            if let Some(Type::Scalar(base_s)) = self.get_type(resolved_base).cloned() {
                for &ctor_id in &base_s.constructors {
                    if let Some(Type::ScalarConstructor(ctor)) = self.get_type(ctor_id) {
                        constructor_names.insert(ctor.name.clone(), ctor_id);
                    }
                }
            }
        }

        // Check own constructors and detect duplicates
        let mut own_constructors: Vec<TypeId> = Vec::new();
        for &ctor_node_id in &node.constructors {
            let ctor = self.check_scalar_constructor(ctx, ctor_node_id);
            let ctor_name = if let Some(Type::ScalarConstructor(sc)) = self.get_type(ctor) {
                sc.name.clone()
            } else {
                continue;
            };

            if constructor_names.contains_key(&ctor_name) {
                self.error(
                    "constructor-duplicate",
                    &format!("A constructor already exists with name {}", ctor_name),
                );
                continue;
            }
            constructor_names.insert(ctor_name.clone(), ctor);
            own_constructors.push(ctor);
        }

        // Use pre-registered type if available, otherwise create new
        let type_id = if let Some(&existing_id) = self.node_type_map.get(&node_id) {
            // Update pre-registered type in-place
            if let Some(t) = self.get_type_mut(existing_id)
                && let Type::Scalar(s) = t
            {
                s.base_scalar = base_scalar;
                s.template_node = template_node;
                s.constructors = own_constructors;
            }
            existing_id
        } else {
            let new_id = {
                let mut s = ScalarType::new(
                    self.next_type_id(),
                    name.clone(),
                    Some(node_id),
                    self.current_namespace,
                    base_scalar,
                );
                s.constructors = own_constructors;
                s.template_node = template_node;
                self.create_type(Type::Scalar(s))
            };

            self.register_type(node_id, new_id, &name, ctx.mapper.as_ref());
            new_id
        };

        // Process directives (e.g., #deprecated) early so deprecated context works
        self.process_and_mark_directives(node_id, type_id);

        if let Some(base_id) = base_scalar
            && let Some(t) = self.get_type_mut(base_id)
            && let Type::Scalar(base) = t
        {
            base.derived_scalars.push(type_id);
        }
        // TS: copyDeprecation(type.baseScalar, type)
        if let Some(base_id) = base_scalar {
            self.copy_deprecation(base_id, type_id);
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

    /// Check a scalar constructor (init member)
    pub(crate) fn check_scalar_constructor(
        &mut self,
        ctx: &CheckContext,
        node_id: NodeId,
    ) -> TypeId {
        let (ast, node) = require_ast_node!(self, node_id, ScalarConstructor, self.error_type);

        let name = Self::get_identifier_name(&ast, node.name);

        let mut parameters = Vec::new();
        for &param_id in &node.parameters {
            let param_node = match ast.id_to_node(param_id) {
                Some(AstNode::FunctionParameter(fp)) => fp.clone(),
                _ => continue,
            };
            let param_name = Self::get_identifier_name(&ast, param_node.name);
            let param_type = param_node.type_annotation.map(|t| self.check_node(ctx, t));

            let param_type_id = self.create_type(Type::FunctionParameter(FunctionParameterType {
                id: self.next_type_id(),
                name: param_name,
                node: Some(param_id),
                r#type: param_type,
                optional: param_node.optional,
                rest: param_node.rest,
                is_finished: true,
            }));
            parameters.push(param_type_id);
        }

        let type_id = self.create_type(Type::ScalarConstructor(ScalarConstructorType {
            id: self.next_type_id(),
            name: name.clone(),
            node: Some(node_id),
            scalar: None,
            parameters,
            is_finished: true,
        }));

        self.node_type_map.insert(node_id, type_id);
        type_id
    }
}
