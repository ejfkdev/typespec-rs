//! Type reference and template instantiation
//!
//! Ported from TypeSpec compiler type reference and template instantiation methods

use super::*;

impl Checker {
    // ========================================================================
    // Type reference checking
    // ========================================================================

    pub(crate) fn check_type_reference(&mut self, ctx: &CheckContext, node_id: NodeId) -> TypeId {
        let result = self.check_type_reference_inner(ctx, node_id);
        self.node_type_map.insert(node_id, result);
        result
    }

    pub(crate) fn check_type_reference_inner(
        &mut self,
        ctx: &CheckContext,
        node_id: NodeId,
    ) -> TypeId {
        let (ast, node) = require_ast_node!(self, node_id, TypeReference, self.error_type);

        // If the name part is a MemberExpression (e.g., A.t in TypeReference),
        // we need to resolve it as a member access. This handles cases like:
        // - alias A<T> = { t: T }; A.t  (member access on uninstantiated template)
        // - Foo.test<string>  (member access with template args on the property)
        if let Some(AstNode::MemberExpression(_)) = ast.id_to_node(node.name) {
            let member_type = self.check_member_expression(ctx, node.name);
            // If there are template arguments on the TypeReference (e.g., Foo.test<string>),
            // instantiate the resolved member type with those arguments.
            if !node.arguments.is_empty() && member_type != self.error_type {
                // Check if the member is actually templated
                // TS: checkTypeReferenceSymbol → if (!isTemplatedNode(decl)) → "notTemplate"
                let member_template_node = self
                    .get_type(member_type)
                    .is_some_and(|t| t.template_node().is_some());
                let member_ast_template_count = self.get_template_param_count(member_type);
                if !member_template_node && member_ast_template_count == 0 {
                    let prop_name = Self::get_identifier_name(&ast, node.name);
                    self.error(
                        "invalid-template-args",
                        &format!(
                            "Can't pass template arguments to non-templated type '{}'.",
                            prop_name
                        ),
                    );
                    return self.error_type;
                }
                return self.instantiate_template(ctx, node_id, member_type, &node.arguments);
            }
            return member_type;
        }

        let name = Self::get_identifier_name(&ast, node.name);

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

            // Track using: if this name was resolved via a using'd namespace, mark it as used
            self.mark_using_as_used_if_applicable(&name, type_id);

            // Lazy type checking: if the type was pre-registered but not yet fully checked,
            // trigger its full check now. This is how TS handles forward references — when
            // a type reference resolves to an unfinished type, checkTypeReferenceSymbol
            // calls checkDeclaredTypeOrIndeterminate which runs checkModel/checkScalar/etc.
            // TS: checkTypeReferenceSymbol → checkDeclaredTypeOrIndeterminate → checkModel(ctx, node)
            //
            // Skip lazy check for template declarations (they are never finished by design
            // — only their instantiations are finished). Re-checking them would duplicate
            // their properties.
            if ctx.mapper.is_none() {
                let is_template_decl = self
                    .get_type(type_id)
                    .is_some_and(|t| t.template_node().is_some());
                let needs_check =
                    !is_template_decl && self.get_type(type_id).is_none_or(|t| !t.is_finished());
                if needs_check
                    && let Some(decl_node_id) =
                        self.get_type(type_id).and_then(|t| t.node_id_from_type())
                    && !self.pending_type_checks.contains(&decl_node_id)
                {
                    self.check_node(ctx, decl_node_id);
                }
            }

            // Check template_node from the type store (may be set even if not finished)
            let has_template_node = self
                .get_type(type_id)
                .is_some_and(|t| t.template_node().is_some());

            // Also check the AST node directly for template parameters
            // (needed when type is pre-registered but not yet fully checked)
            let ast_template_param_count = self.get_template_param_count(type_id);
            let is_template = has_template_node || ast_template_param_count > 0;

            if !node.arguments.is_empty() {
                if !is_template {
                    // Template arguments on a non-templated type
                    self.error(
                        "invalid-template-args",
                        &format!(
                            "Can't pass template arguments to non-templated type '{}'.",
                            name
                        ),
                    );
                    return self.error_type;
                }
                // Check argument count against template parameter count
                let template_param_count = if ast_template_param_count > 0 {
                    ast_template_param_count
                } else {
                    self.get_template_param_count(type_id)
                };
                let required_param_count = self.get_required_template_param_count(type_id);
                if node.arguments.len() > template_param_count {
                    self.error(
                        "invalid-template-args",
                        &format!(
                            "Too many template arguments for '{}'. Expected at most {}, got {}.",
                            name,
                            template_param_count,
                            node.arguments.len()
                        ),
                    );
                    return self.error_type;
                }
                if node.arguments.len() < required_param_count {
                    // Find which required param is missing
                    let missing_param =
                        self.get_missing_template_param_name(type_id, node.arguments.len());
                    self.error(
                        "invalid-template-args",
                        &format!(
                            "Template argument '{}' is required for '{}'.",
                            missing_param, name
                        ),
                    );
                    return self.error_type;
                }
                let inst_type_id =
                    self.instantiate_template(ctx, node_id, type_id, &node.arguments);
                // Mark this node as a template instantiation
                // Ported from TS name-resolver.ts: isTemplateInstantiation
                self.symbol_links
                    .entry(node_id)
                    .or_default()
                    .is_template_instantiation = true;
                // Emit deprecation warning for deprecated template instances
                self.emit_deprecated_warning_if_needed(inst_type_id);
                return inst_type_id;
            } else if is_template {
                // Referenced a template type without providing arguments
                let required_param_count = self.get_required_template_param_count(type_id);
                if required_param_count > 0 {
                    let missing_param = self.get_missing_template_param_name(type_id, 0);
                    self.error(
                        "invalid-template-args",
                        &format!(
                            "Template argument '{}' is required for '{}'.",
                            missing_param, name
                        ),
                    );
                    return self.error_type;
                }
                // All template parameters have defaults → auto-instantiate with defaults
                let total_param_count = self.get_template_param_count(type_id);
                if total_param_count > 0 {
                    let default_args = self.get_template_default_args(type_id);
                    if !default_args.is_empty() {
                        let inst_type_id =
                            self.instantiate_template(ctx, node_id, type_id, &default_args);
                        self.symbol_links
                            .entry(node_id)
                            .or_default()
                            .is_template_instantiation = true;
                        return inst_type_id;
                    }
                }
            }

            // If the declared type is not yet finished, trigger its checking.
            if let Some(t) = self.get_type(type_id)
                && !t.is_finished()
                && let Some(node_id_for_type) = t.node_id_from_type()
            {
                return self.check_node(ctx, node_id_for_type);
            }

            // Emit deprecation warning if the referenced type is deprecated
            self.emit_deprecated_warning_if_needed(type_id);

            // Check internal visibility
            self.check_internal_visibility(type_id);

            return type_id;
        }

        // Try to resolve name via using declarations
        if let Some(type_id) = self.resolve_via_using(&name) {
            // Check internal visibility for the resolved type
            self.check_internal_visibility(type_id);
            self.emit_deprecated_warning_if_needed(type_id);
            return type_id;
        }

        // Try to resolve as a template parameter name (e.g., T in model Foo<T> { a: T })
        // Search from innermost scope outward to handle shadowing correctly
        for scope in self.template_param_scope.iter().rev() {
            if let Some(&type_id) = scope.get(&name) {
                return type_id;
            }
        }

        // Try to resolve as a value reference (const declaration)
        // When a const name is used in a type position, return the value's type
        // but emit value-in-type diagnostic
        if let Some(&value_id) = self.declared_values.get(&name) {
            let value_type = self.get_value(value_id).map(|v| v.value_type());
            if let Some(type_id) = value_type {
                // Emit value-in-type: a value is being used where a type is expected
                self.error("value-in-type", &format!("Value '{}' is used in a type position. Add `extends valueof unknown` to accept any value.", name));
                return type_id;
            }
        }

        self.error("invalid-ref", &format!("Unknown type '{}'", name));
        self.error_type
    }

    // ========================================================================
    // Template parameter helpers
    // ========================================================================

    /// Get the total number of template parameters for a type.
    /// Checks both template_node (set after checking) and the AST node directly.
    pub(crate) fn get_template_param_count(&self, type_id: TypeId) -> usize {
        // First try template_node (set after check_model runs)
        let template_node_id = self.get_type(type_id).and_then(|t| t.template_node());
        if let Some(node_id) = template_node_id {
            let count = self.get_template_param_count_from_node(node_id);
            if count > 0 {
                return count;
            }
        }
        // Fallback: check the AST node directly (for pre-registered types)
        let ast_node_id = self.get_type(type_id).and_then(|t| t.node_id_from_type());
        if let Some(node_id) = ast_node_id {
            let count = self.get_template_param_count_from_node(node_id);
            if count > 0 {
                return count;
            }
        }
        // Built-in template types (Array, Record) don't have AST nodes
        // but they do have template_node set with a fake value and indexers
        if let Some(t) = self.get_type(type_id)
            && let Type::Model(m) = t
            && m.indexer.is_some()
            && m.node.is_none()
            && (m.name == "Array" || m.name == "Record")
        {
            return 1; // Built-in template types have 1 type parameter
        }
        0
    }

    pub(crate) fn get_template_param_count_from_node(&self, node_id: NodeId) -> usize {
        self.get_template_param_ids_from_node(node_id).len()
    }

    /// Get the template parameter NodeIds for a type
    pub(crate) fn get_template_param_ids(&self, type_id: TypeId) -> Vec<NodeId> {
        let template_node_id = self.get_type(type_id).and_then(|t| t.template_node());
        let ast_node_id = self.get_type(type_id).and_then(|t| t.node_id_from_type());
        let node_id = template_node_id.or(ast_node_id);
        match node_id {
            Some(id) => self.get_template_param_ids_from_node(id),
            None => Vec::new(),
        }
    }

    /// Core helper: extract template parameter NodeIds from an AST node
    pub(crate) fn get_template_param_ids_from_node(&self, node_id: NodeId) -> Vec<NodeId> {
        let ast = require_ast_or!(self, Vec::new());
        match ast.id_to_node(node_id) {
            Some(AstNode::ModelDeclaration(decl)) => decl.template_parameters.clone(),
            Some(AstNode::InterfaceDeclaration(decl)) => decl.template_parameters.clone(),
            Some(AstNode::UnionDeclaration(decl)) => decl.template_parameters.clone(),
            Some(AstNode::ScalarDeclaration(decl)) => decl.template_parameters.clone(),
            Some(AstNode::OperationDeclaration(decl)) => decl.template_parameters.clone(),
            Some(AstNode::AliasStatement(decl)) => decl.template_parameters.clone(),
            _ => Vec::new(),
        }
    }

    /// Check for unused template parameters in operations, interfaces, and aliases.
    /// This scans the AST subtree of the declaration node for references to template param names.
    pub(crate) fn check_unused_template_params(
        &mut self,
        node_id: NodeId,
        template_param_ids: &[NodeId],
        decorators: &[NodeId],
    ) {
        let ast_ref = match &self.ast {
            Some(a) => a.clone(),
            None => return,
        };

        // Collect template parameter names
        let tmpl_param_names: Vec<String> = template_param_ids
            .iter()
            .map(|&pid| {
                Self::get_identifier_name(
                    &ast_ref,
                    match ast_ref.id_to_node(pid) {
                        Some(AstNode::TemplateParameterDeclaration(d)) => d.name,
                        _ => return String::new(),
                    },
                )
            })
            .collect();

        let mut used_params: HashSet<String> = HashSet::new();

        // Scan decorator arguments
        for &dec_id in decorators {
            self.collect_template_param_refs(&ast_ref, dec_id, &tmpl_param_names, &mut used_params);
        }

        // Scan declaration-specific AST subtrees
        match ast_ref.id_to_node(node_id) {
            Some(AstNode::OperationDeclaration(decl)) => {
                // Scan the signature (parameters + return type)
                self.collect_template_param_refs(
                    &ast_ref,
                    decl.signature,
                    &tmpl_param_names,
                    &mut used_params,
                );
            }
            Some(AstNode::InterfaceDeclaration(decl)) => {
                // Scan all operations within the interface
                for &op_id in &decl.operations {
                    self.collect_template_param_refs(
                        &ast_ref,
                        op_id,
                        &tmpl_param_names,
                        &mut used_params,
                    );
                }
                // Scan extends clauses
                for &ext_id in &decl.extends {
                    self.collect_template_param_refs(
                        &ast_ref,
                        ext_id,
                        &tmpl_param_names,
                        &mut used_params,
                    );
                }
            }
            Some(AstNode::AliasStatement(decl)) => {
                // Scan the alias value/target expression
                self.collect_template_param_refs(
                    &ast_ref,
                    decl.value,
                    &tmpl_param_names,
                    &mut used_params,
                );
            }
            Some(AstNode::ModelDeclaration(decl)) => {
                // Scan properties
                for &prop_id in &decl.properties {
                    self.collect_template_param_refs(
                        &ast_ref,
                        prop_id,
                        &tmpl_param_names,
                        &mut used_params,
                    );
                }
                // Scan extends clause
                if let Some(ext_id) = decl.extends {
                    self.collect_template_param_refs(
                        &ast_ref,
                        ext_id,
                        &tmpl_param_names,
                        &mut used_params,
                    );
                }
                // Scan is clause
                if let Some(is_id) = decl.is {
                    self.collect_template_param_refs(
                        &ast_ref,
                        is_id,
                        &tmpl_param_names,
                        &mut used_params,
                    );
                }
            }
            _ => {}
        }

        // Report unused parameters
        for name in &tmpl_param_names {
            if !name.is_empty() && !used_params.contains(name) {
                self.warning(
                    "unused-template-parameter",
                    &format!("Template parameter '{}' is unused.", name),
                );
            }
        }
    }

    /// Handle the common post-check pattern for template declarations:
    /// - If template declaration: check unused template params, don't finish type
    /// - If not template declaration: finish type
    pub(crate) fn finish_template_or_type(
        &mut self,
        type_id: TypeId,
        node_id: NodeId,
        template_params: &[NodeId],
        decorators: &[NodeId],
        mapper: Option<&TypeMapper>,
    ) {
        let is_template_decl = !template_params.is_empty() && mapper.is_none();
        if is_template_decl {
            self.check_unused_template_params(node_id, template_params, decorators);
        } else {
            self.finish_type(type_id);
        }
        // Pop the template parameter scope that was pushed in check_template_declaration
        if !template_params.is_empty() {
            self.pop_template_param_scope();
        }
    }

    /// Get the number of required (non-defaulted) template parameters for a type
    pub(crate) fn get_required_template_param_count(&self, type_id: TypeId) -> usize {
        // Built-in template types (Array, Record) require their type parameter
        if let Some(t) = self.get_type(type_id)
            && let Type::Model(m) = t
            && m.indexer.is_some()
            && m.node.is_none()
            && (m.name == "Array" || m.name == "Record")
        {
            return 1; // Built-in template types have 1 required parameter
        }
        let params = self.get_template_param_ids(type_id);
        let ast = require_ast_or!(self, 0);
        let mut count = 0;
        for &param_id in &params {
            let has_default = match ast.id_to_node(param_id) {
                Some(AstNode::TemplateParameterDeclaration(decl)) => decl.default.is_some(),
                _ => false,
            };
            if !has_default {
                count += 1;
            }
        }
        count
    }

    /// Get the default argument NodeIds for a template type.
    /// Returns the default expression NodeIds for all template parameters.
    /// If any parameter doesn't have a default, returns empty vec.
    pub(crate) fn get_template_default_args(&self, type_id: TypeId) -> Vec<NodeId> {
        let params = self.get_template_param_ids(type_id);
        let ast = require_ast_or!(self, Vec::new());
        let mut defaults = Vec::new();
        for &param_id in &params {
            match ast.id_to_node(param_id) {
                Some(AstNode::TemplateParameterDeclaration(decl)) => {
                    match decl.default {
                        Some(default_node) => defaults.push(default_node),
                        None => return Vec::new(), // No default for this param
                    }
                }
                _ => return Vec::new(),
            }
        }
        defaults
    }

    /// Get the name of the first missing required template parameter
    pub(crate) fn get_missing_template_param_name(
        &self,
        type_id: TypeId,
        provided_count: usize,
    ) -> String {
        let params = self.get_template_param_ids(type_id);
        let ast = require_ast_or!(self, String::new());
        for (idx, &param_id) in params.iter().enumerate() {
            let has_default = match ast.id_to_node(param_id) {
                Some(AstNode::TemplateParameterDeclaration(decl)) => decl.default.is_some(),
                _ => false,
            };
            if idx >= provided_count && !has_default {
                return match ast.id_to_node(param_id) {
                    Some(AstNode::TemplateParameterDeclaration(decl)) => {
                        Self::get_identifier_name(&ast, decl.name)
                    }
                    _ => String::new(),
                };
            }
        }
        String::new()
    }

    // ========================================================================
    // Template instantiation
    // ========================================================================

    /// Check for value-in-type: when a value is passed to a template parameter
    /// that has no constraint, emit a diagnostic.
    /// Ported from TS checker.ts checkTemplateArguments()
    pub(crate) fn check_template_arg_value_in_type(
        &mut self,
        ctx: &CheckContext,
        argument_ids: &[NodeId],
        template_param_ids: &[NodeId],
    ) {
        for (i, &arg_id) in argument_ids.iter().enumerate() {
            if i >= template_param_ids.len() {
                break;
            }

            // Check if this argument resolves to a value
            let entity = self.check_node_entity(ctx, arg_id);
            let is_value = entity.is_value();

            // Also check if the argument is a reference to a declared value (const)
            // This handles cases where check_node_entity doesn't properly resolve
            // const identifiers
            let is_const_ref = if !is_value {
                self.is_const_reference(arg_id)
            } else {
                false
            };

            if is_value || is_const_ref {
                // Check if the corresponding template parameter has a constraint
                // that accepts values (valueof)
                let param_id = template_param_ids[i];
                let has_value_constraint = self
                    .node_type_map
                    .get(&param_id)
                    .and_then(|&tid| self.get_type(tid))
                    .and_then(|t| match t {
                        Type::TemplateParameter(tp) => tp.constraint,
                        _ => None,
                    })
                    .is_some();

                if !has_value_constraint {
                    self.error("value-in-type", "Template parameter has no constraint but a value is passed. Add `extends valueof unknown` to accept any value.");
                }
            }
        }
    }

    /// Check if a node is a reference to a declared value (const)
    pub(crate) fn is_const_reference(&self, node_id: NodeId) -> bool {
        let ast = require_ast_or!(self, false);
        match ast.id_to_node(node_id) {
            Some(AstNode::Identifier(ident)) => self.declared_values.contains_key(&ident.value),
            Some(AstNode::TypeReference(ref_node)) => {
                let name = Self::get_identifier_name(&ast, ref_node.name);
                self.declared_values.contains_key(&name)
            }
            _ => false,
        }
    }

    /// Check template argument assignability to constraints.
    /// Ported from TS checker.ts checkTemplateArguments() → checkArgumentAssignable()
    pub(crate) fn check_template_arg_constraints(
        &mut self,
        arg_types: &[TypeId],
        template_param_ids: &[NodeId],
        explicit_arg_count: usize,
    ) {
        for (i, &arg_type_id) in arg_types.iter().enumerate() {
            if i >= template_param_ids.len() {
                break;
            }

            let param_id = template_param_ids[i];
            let constraint_id = self
                .node_type_map
                .get(&param_id)
                .and_then(|&tid| self.get_type(tid))
                .and_then(|t| match t {
                    Type::TemplateParameter(tp) => tp.constraint,
                    _ => None,
                });

            if let Some(constraint_id) = constraint_id {
                // Skip check if either type is error
                if arg_type_id == self.error_type || constraint_id == self.error_type {
                    continue;
                }

                // Check if the argument type is assignable to the constraint
                // Special case: if the arg is a TemplateParameter itself (e.g., T from
                // an enclosing template), check its constraint instead.
                // An unconstrained TemplateParameter is NOT assignable to a specific constraint.
                let effective_arg = match self.get_type(arg_type_id) {
                    Some(Type::TemplateParameter(tp)) => {
                        // Use the template parameter's constraint as the effective type
                        // If no constraint, use unknown_type (which is not assignable to
                        // specific types like string)
                        tp.constraint.unwrap_or(self.unknown_type)
                    }
                    _ => arg_type_id,
                };

                let (is_assignable, _) =
                    self.is_type_assignable_to(effective_arg, constraint_id, param_id);
                if !is_assignable {
                    // Use different diagnostic codes for defaults vs explicit args
                    let code = if i < explicit_arg_count {
                        "invalid-argument"
                    } else {
                        "unassignable"
                    };
                    self.error_unassignable(code, arg_type_id, constraint_id);
                }
            }
        }
    }

    pub(crate) fn instantiate_template(
        &mut self,
        ctx: &CheckContext,
        _ref_node_id: NodeId,
        template_type_id: TypeId,
        argument_ids: &[NodeId],
    ) -> TypeId {
        let mut arg_types: Vec<TypeId> = Vec::new();
        for &arg_id in argument_ids {
            let entity = self.check_node_entity(ctx, arg_id);
            arg_types.push(self.entity_to_type_id(&entity));
        }

        let template_node_id = self
            .get_type(template_type_id)
            .and_then(|t| t.node_id_from_type());

        // Handle built-in template types that don't have AST nodes
        // (e.g., Array<T> which is registered programmatically)
        if template_node_id.is_none() {
            return self.instantiate_builtin_template(template_type_id, &arg_types);
        }

        // Extract template parameter NodeIds from the declaration AST node
        let template_param_ids: Vec<NodeId> = match template_node_id {
            Some(nid) => self.get_template_param_ids_from_node(nid),
            None => vec![],
        };

        // Check for value-in-type: when a value is passed to a template parameter
        // that has no constraint, emit a diagnostic.
        // Ported from TS checker.ts checkTemplateArguments()
        self.check_template_arg_value_in_type(ctx, argument_ids, &template_param_ids);

        // Check template argument assignability to constraint
        // Ported from TS checker.ts checkTemplateArguments() → checkArgumentAssignable
        self.check_template_arg_constraints(&arg_types, &template_param_ids, argument_ids.len());

        // Fill in default template arguments for missing parameters
        // TS: checkTemplateArguments → fills defaults so Foo === Foo<string> === Foo<string, string>
        // when A = string, B = string
        if arg_types.len() < template_param_ids.len() {
            for (i, &param_id) in template_param_ids.iter().enumerate() {
                if i >= arg_types.len() {
                    // Look up the TemplateParameterType to get its default value
                    if let Some(&param_type_id) = self.node_type_map.get(&param_id)
                        && let Some(Type::TemplateParameter(tp)) = self.get_type(param_type_id)
                    {
                        if let Some(default_type_id) = tp.default {
                            arg_types.push(default_type_id);
                        } else {
                            // No default — report invalid-template-args (missing required arg)
                            break;
                        }
                    }
                }
            }
        }

        // Cache lookup with fully-filled arg_types
        if let Some(node_id) = template_node_id {
            let links = self.symbol_links.entry(node_id).or_default();
            if let Some(ref instantiations) = links.instantiations
                && let Some(&existing_id) = instantiations.get(&arg_types)
            {
                return existing_id;
            }
        }

        let mut mapper = TypeMapper::new();
        for (i, &param_id) in template_param_ids.iter().enumerate() {
            if i < arg_types.len() {
                mapper.map.insert(param_id, arg_types[i]);
            }
        }
        mapper.args = arg_types.clone();

        let inst_ctx = CheckContext::with_mapper(Some(mapper.clone()));

        let instance_id = if let Some(node_id) = template_node_id {
            self.check_node(&inst_ctx, node_id)
        } else {
            self.error_type
        };

        // Set template_mapper on the newly created template instance
        // This mirrors TS linkMapper() which sets type.templateMapper after instantiation
        if instance_id != self.error_type {
            self.link_template_mapper(instance_id, mapper.clone());

            // Cache the instantiation result so subsequent references with the same
            // arguments return the same TypeId (ported from TS symbolLinks.instantiations)
            if let Some(node_id) = template_node_id {
                let links = self.symbol_links.entry(node_id).or_default();
                links
                    .instantiations
                    .get_or_insert_with(HashMap::new)
                    .insert(arg_types, instance_id);
            }
        }

        instance_id
    }

    /// Link a template mapper to a type instance.
    /// Ported from TS checker linkMapper().
    pub(crate) fn link_template_mapper(&mut self, type_id: TypeId, mapper: TypeMapper) {
        if let Some(t) = self.get_type_mut(type_id) {
            t.set_template_mapper_if_none(Box::new(mapper));
        }
    }

    /// Instantiate a built-in template type (e.g., Array<T>) that doesn't have an AST node
    pub(crate) fn instantiate_builtin_template(
        &mut self,
        template_type_id: TypeId,
        arg_types: &[TypeId],
    ) -> TypeId {
        let template_name = match self.get_type(template_type_id) {
            Some(Type::Model(m)) => m.name.clone(),
            _ => return self.error_type,
        };

        // Build mapper for built-in template instance
        let mut mapper = TypeMapper::new();
        mapper.args = arg_types.to_vec();

        match template_name.as_str() {
            "Array" => {
                // Array<T> creates a model with an integer indexer
                let integer_id = self.std_types.get("integer").copied();
                let element_type = arg_types.first().copied().unwrap_or(self.error_type);

                let instance_id = {
                    let mut m =
                        ModelType::new(self.next_type_id(), "Array".to_string(), None, None);
                    m.indexer = integer_id.map(|id| (id, element_type));
                    m.template_node = Some(template_type_id); // Mark as template instance
                    m.is_finished = true;
                    self.create_type(Type::Model(m))
                };
                self.link_template_mapper(instance_id, mapper);
                instance_id
            }
            "Record" => {
                // Record<T> creates a model with a string indexer
                let string_id = self.std_types.get("string").copied();
                let element_type = arg_types.first().copied().unwrap_or(self.error_type);

                let instance_id = {
                    let mut m =
                        ModelType::new(self.next_type_id(), "Record".to_string(), None, None);
                    m.indexer = string_id.map(|id| (id, element_type));
                    m.template_node = Some(template_type_id); // Mark as template instance
                    m.is_finished = true;
                    self.create_type(Type::Model(m))
                };
                self.link_template_mapper(instance_id, mapper);
                instance_id
            }
            _ => self.error_type,
        }
    }
}
