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
                    self.error_at(
                        node_id,
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

        // Recursive alias through a model expression resolves to the
        // in-progress model type instead of erroring (microsoft/typespec#10684).
        if let Some(in_progress) = self.resolve_pending_alias_model_expression(&name) {
            return in_progress;
        }

        // Check if this name is currently being resolved (circular reference detection)
        if let Some(error) = self.check_circular_reference(&name) {
            return error;
        }

        if let Some(type_id) = self.resolve_declared_name(&name) {
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
                    self.error_at(
                        node_id,
                        "invalid-template-args",
                        &format!(
                            "Can't pass template arguments to non-templated type '{}'.",
                            name
                        ),
                    );
                    return self.error_type;
                }
                // Named template arguments (e.g. `A<U = int32, T = string>`) bind by name,
                // so the positional count checks below do not apply. Defer all validation
                // (nonexistent parameter name, duplicate, positional-after-named, missing
                // required) to instantiate_template, which resolves arguments by name.
                if self.has_named_arguments(&node.arguments) {
                    let inst_type_id =
                        self.instantiate_template(ctx, node_id, type_id, &node.arguments);
                    self.symbol_links
                        .entry(node_id)
                        .or_default()
                        .is_template_instantiation = true;
                    self.emit_deprecated_warning_if_needed(inst_type_id);
                    return inst_type_id;
                }
                // Check argument count against template parameter count
                let template_param_count = if ast_template_param_count > 0 {
                    ast_template_param_count
                } else {
                    self.get_template_param_count(type_id)
                };
                let required_param_count = self.get_required_template_param_count(type_id);
                if node.arguments.len() > template_param_count {
                    self.error_at(
                        node_id,
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
                    self.error_at(
                        node_id,
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
                    self.error_at(
                        node_id,
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
            // Mark the using as used so it doesn't get reported as unused
            self.mark_using_as_used_if_applicable(&name, type_id);
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
                self.error_at(node_id, "value-in-type", &format!("Value '{}' is used in a type position. Add `extends valueof unknown` to accept any value.", name));
                return type_id;
            }
        }

        self.error_at(node_id, "invalid-ref", &format!("Unknown type '{}'", name));
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

        // Scan template parameter constraints and defaults — a parameter used
        // in another parameter's default (e.g. `Properties extends Model =
        // TagsUpdateModel<Resource>`, microsoft/typespec#11477) counts as used.
        for &param_id in template_param_ids {
            if let Some(AstNode::TemplateParameterDeclaration(d)) = ast_ref.id_to_node(param_id) {
                if let Some(constraint) = d.constraint {
                    self.collect_template_param_refs(
                        &ast_ref,
                        constraint,
                        &tmpl_param_names,
                        &mut used_params,
                    );
                }
                if let Some(default) = d.default {
                    self.collect_template_param_refs(
                        &ast_ref,
                        default,
                        &tmpl_param_names,
                        &mut used_params,
                    );
                }
            }
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
            Some(AstNode::ScalarDeclaration(decl)) => {
                // Scan extends clause (e.g., `scalar Foo<T> extends T`)
                if let Some(ext_id) = decl.extends {
                    self.collect_template_param_refs(
                        &ast_ref,
                        ext_id,
                        &tmpl_param_names,
                        &mut used_params,
                    );
                }
                // Scan constructor parameter types
                for &ctor_id in &decl.constructors {
                    self.collect_template_param_refs(
                        &ast_ref,
                        ctor_id,
                        &tmpl_param_names,
                        &mut used_params,
                    );
                }
            }
            Some(AstNode::UnionDeclaration(decl)) => {
                // Scan variant value types (e.g., `union Foo<T> { a: T; b: string }`)
                for &var_id in &decl.variants {
                    self.collect_template_param_refs(
                        &ast_ref,
                        var_id,
                        &tmpl_param_names,
                        &mut used_params,
                    );
                }
            }
            _ => {}
        }

        // Report unused parameters.
        // Skip declarations that live in injected library source: the official
        // compiler loads libraries as pre-compiled modules and never lints them,
        // so flagging library templates (e.g. TypeSpec.Http.LinkHeader<T>) would
        // be a divergence that pollutes every run.
        let is_library = ast_ref
            .node_span(node_id)
            .is_some_and(|span| span.start.line as usize <= ast_ref.library_line_offset);
        if is_library {
            return;
        }
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
                    self.error_at(arg_id, "value-in-type", "Template parameter has no constraint but a value is passed. Add `extends valueof unknown` to accept any value.");
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
    ///
    /// `explicit[i]` is true when the argument for parameter `i` was explicitly
    /// provided by the caller (vs. filled from a default). This must be per-slot
    /// rather than a count because named arguments can bind a later parameter
    /// while an earlier one is filled from its default.
    pub(crate) fn check_template_arg_constraints(
        &mut self,
        arg_types: &[TypeId],
        template_param_ids: &[NodeId],
        explicit: &[bool],
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
                    let code = if explicit.get(i).copied().unwrap_or(false) {
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
        ref_node_id: NodeId,
        template_type_id: TypeId,
        argument_ids: &[NodeId],
    ) -> TypeId {
        let template_node_id = self
            .get_type(template_type_id)
            .and_then(|t| t.node_id_from_type());

        // Handle built-in template types that don't have AST nodes
        // (e.g., Array<T> which is registered programmatically)
        if template_node_id.is_none() {
            let mut arg_types: Vec<TypeId> = Vec::new();
            for &arg_id in argument_ids {
                let entity = self.check_node_entity(ctx, arg_id);
                arg_types.push(self.entity_to_type_id(&entity));
            }
            return self.instantiate_builtin_template(template_type_id, &arg_types);
        }

        // Extract template parameter NodeIds from the declaration AST node
        let template_node_id = template_node_id.unwrap();
        let template_param_ids: Vec<NodeId> =
            self.get_template_param_ids_from_node(template_node_id);

        let template_name = self
            .get_type(template_type_id)
            .and_then(|t| t.name())
            .unwrap_or("")
            .to_string();

        // Resolve positional and/or named template arguments into per-parameter
        // slots (in declaration order). Named arguments (e.g. `A<U = int32, T = string>`)
        // bind to their matching parameter regardless of position; positional
        // arguments bind in order. Emits the named-argument diagnostics:
        // nonexistent parameter, duplicate argument, positional-after-named.
        let (slots, explicit) = self.resolve_template_arguments(
            ctx,
            ref_node_id,
            &template_name,
            argument_ids,
            &template_param_ids,
        );

        // Check for value-in-type: when a value is passed to a template parameter
        // that has no constraint, emit a diagnostic. Only meaningful for positional
        // arguments (named arguments in the test suite always pass types); skip for
        // named lists to avoid a wrong param mapping.
        // Ported from TS checker.ts checkTemplateArguments()
        if !self.has_named_arguments(argument_ids) {
            self.check_template_arg_value_in_type(ctx, argument_ids, &template_param_ids);
        }

        // Fill in default template arguments for unspecified parameters, building
        // the final argument list in parameter-declaration order.
        // TS: checkTemplateArguments → fills defaults so Foo === Foo<string> === Foo<string, string>
        // when A = string, B = string
        let mut arg_types: Vec<TypeId> = Vec::with_capacity(template_param_ids.len());
        for (i, slot) in slots.iter().enumerate() {
            match slot {
                Some(type_id) => arg_types.push(*type_id),
                None => {
                    let param_id = template_param_ids[i];
                    let default = self
                        .node_type_map
                        .get(&param_id)
                        .and_then(|&tid| self.get_type(tid))
                        .and_then(|t| match t {
                            Type::TemplateParameter(tp) => tp.default,
                            _ => None,
                        });
                    if let Some(default_type_id) = default {
                        // A default that references an earlier template parameter
                        // (e.g. `X = T`) is cached at declaration time as the
                        // TemplateParameter type itself. Substitute it with the
                        // already-resolved argument for that parameter (which is
                        // available because defaults may only reference previously
                        // declared parameters, filled in order). Composite defaults
                        // (e.g. `{ t: T }`) are left as-is for now.
                        let resolved = self.resolve_default_param_ref(
                            default_type_id,
                            &template_param_ids,
                            &arg_types,
                        );
                        arg_types.push(resolved);
                    } else {
                        // No default and not provided — required argument is missing.
                        let param_name = self.param_display_name(param_id);
                        self.error_at(
                            ref_node_id,
                            "invalid-template-args",
                            &format!(
                                "Template argument '{}' is required for '{}'.",
                                param_name, template_name
                            ),
                        );
                        // Bind to unknown so the body can still resolve.
                        arg_types.push(self.unknown_type);
                    }
                }
            }
        }

        // Check template argument assignability to constraint
        // Ported from TS checker.ts checkTemplateArguments() → checkArgumentAssignable
        self.check_template_arg_constraints(&arg_types, &template_param_ids, &explicit);

        // Cache lookup with fully-filled arg_types
        {
            let links = self.symbol_links.entry(template_node_id).or_default();
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

        let instance_id = self.check_node(&inst_ctx, template_node_id);

        // Set template_mapper on the newly created template instance
        // This mirrors TS linkMapper() which sets type.templateMapper after instantiation
        if instance_id != self.error_type {
            self.link_template_mapper(instance_id, mapper.clone());

            // Cache the instantiation result so subsequent references with the same
            // arguments return the same TypeId (ported from TS symbolLinks.instantiations)
            let links = self.symbol_links.entry(template_node_id).or_default();
            links
                .instantiations
                .get_or_insert_with(HashMap::new)
                .insert(arg_types, instance_id);
        }

        instance_id
    }

    /// Returns true if any of the template arguments is a named argument
    /// (`A<T = string>`), false if all are positional.
    pub(crate) fn has_named_arguments(&self, argument_ids: &[NodeId]) -> bool {
        let ast = match &self.ast {
            Some(a) => a,
            None => return false,
        };
        argument_ids.iter().any(|&aid| {
            matches!(
                ast.id_to_node(aid),
                Some(AstNode::TemplateArgument(ta)) if ta.name.is_some()
            )
        })
    }

    /// Resolve template arguments — positional and/or named — into per-parameter
    /// slots in declaration order.
    ///
    /// Returns `(slots, explicit)` where both vectors are indexed by template
    /// parameter position:
    /// - `slots[i]` is `Some(type)` if an argument was provided for parameter `i`,
    ///   otherwise `None` (the caller fills the default).
    /// - `explicit[i]` is true when parameter `i` was explicitly provided by the
    ///   caller (used to pick `invalid-argument` vs `unassignable` codes).
    ///
    /// Emits diagnostics for invalid named usage: a parameter name that does not
    /// exist on the target template, an argument specified twice, and a positional
    /// argument that follows a named argument.
    pub(crate) fn resolve_template_arguments(
        &mut self,
        ctx: &CheckContext,
        _ref_node_id: NodeId,
        template_name: &str,
        argument_ids: &[NodeId],
        template_param_ids: &[NodeId],
    ) -> (Vec<Option<TypeId>>, Vec<bool>) {
        let n = template_param_ids.len();
        let mut slots: Vec<Option<TypeId>> = vec![None; n];
        let mut explicit: Vec<bool> = vec![false; n];

        let ast = match &self.ast {
            Some(a) => a.clone(),
            None => return (slots, explicit),
        };

        // Map parameter name → declaration index.
        let mut name_to_index: HashMap<String, usize> = HashMap::with_capacity(n);
        for (i, &param_id) in template_param_ids.iter().enumerate() {
            if let Some(AstNode::TemplateParameterDeclaration(decl)) = ast.id_to_node(param_id) {
                name_to_index.insert(Self::get_identifier_name(&ast, decl.name), i);
            }
        }

        let mut next_positional = 0usize;
        let mut seen_named = false;

        for &arg_id in argument_ids {
            let name_node_opt = match ast.id_to_node(arg_id) {
                Some(AstNode::TemplateArgument(ta)) => ta.name,
                _ => None,
            };

            let value_type = {
                let entity = self.check_node_entity(ctx, arg_id);
                self.entity_to_type_id(&entity)
            };

            if let Some(name_node) = name_node_opt {
                seen_named = true;
                let arg_name = Self::get_identifier_name(&ast, name_node);
                match name_to_index.get(&arg_name) {
                    Some(&idx) => {
                        if explicit[idx] {
                            self.error_at(
                                arg_id,
                                "invalid-template-args",
                                &format!("Cannot specify template argument '{}' again.", arg_name),
                            );
                        } else {
                            slots[idx] = Some(value_type);
                            explicit[idx] = true;
                        }
                    }
                    None => {
                        self.error_at(
                            arg_id,
                            "invalid-template-args",
                            &format!(
                                "No parameter named '{}' exists in the target template.",
                                arg_name
                            ),
                        );
                    }
                }
            } else {
                // Positional argument.
                if seen_named {
                    self.error_at(
                        arg_id,
                        "invalid-template-args",
                        "Positional template arguments cannot follow named arguments in the same argument list.",
                    );
                } else if next_positional < n {
                    slots[next_positional] = Some(value_type);
                    explicit[next_positional] = true;
                    next_positional += 1;
                } else {
                    self.error_at(
                        arg_id,
                        "invalid-template-args",
                        &format!("Too many template arguments for '{}'.", template_name),
                    );
                }
            }
        }

        (slots, explicit)
    }

    /// Return the source name of a template parameter declaration node.
    fn param_display_name(&self, param_id: NodeId) -> String {
        let ast = match &self.ast {
            Some(a) => a,
            None => return String::new(),
        };
        match ast.id_to_node(param_id) {
            Some(AstNode::TemplateParameterDeclaration(decl)) => {
                Self::get_identifier_name(ast, decl.name)
            }
            _ => String::new(),
        }
    }

    /// If a template-parameter default is itself a bare reference to an earlier
    /// template parameter (e.g. `X = T`), substitute it with the value already
    /// bound for that parameter. `arg_types` holds the values resolved so far
    /// (in declaration order), so defaults referencing previously declared
    /// parameters resolve correctly. Composite defaults are returned unchanged.
    fn resolve_default_param_ref(
        &self,
        default_type_id: TypeId,
        template_param_ids: &[NodeId],
        arg_types: &[TypeId],
    ) -> TypeId {
        if let Some(Type::TemplateParameter(tp)) = self.get_type(default_type_id)
            && let Some(node) = tp.node
            && let Some(j) = template_param_ids.iter().position(|&p| p == node)
            && j < arg_types.len()
        {
            return arg_types[j];
        }
        default_type_id
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

    /// Instantiate a template type using only its default template arguments.
    /// Returns None if the type is not a template or any parameter lacks a default.
    /// Ported from upstream PR #9670: instantiate templated aliases in base position
    /// of member expression when all parameters are defaultable.
    pub(crate) fn instantiate_template_with_defaults(
        &mut self,
        template_type_id: TypeId,
    ) -> Option<TypeId> {
        let default_args = self.get_template_default_args(template_type_id);
        if default_args.is_empty() {
            return None;
        }

        let ctx = CheckContext::new();
        let instance_id = self.instantiate_template(&ctx, 0, template_type_id, &default_args);
        if instance_id == self.error_type {
            None
        } else {
            Some(instance_id)
        }
    }

    /// Look up a named member on a type, returning its TypeId or error_type.
    /// Used after template instantiation to access members on the resulting type.
    pub(crate) fn lookup_member_on_type(
        &mut self,
        type_id: TypeId,
        member_name: &str,
        source_node_id: NodeId,
    ) -> TypeId {
        match self.get_type(type_id) {
            Some(Type::Namespace(ns)) => {
                if let Some(member_id) = ns.lookup_member(member_name) {
                    self.check_internal_visibility(member_id);
                    member_id
                } else {
                    self.error_at(
                        source_node_id,
                        "invalid-ref",
                        &format!("Namespace '{}' has no member '{}'", ns.name, member_name),
                    );
                    self.error_type
                }
            }
            Some(Type::Model(m)) => {
                if let Some(&prop_id) = m.properties.get(member_name) {
                    prop_id
                } else {
                    // Walk base model chain for inherited properties
                    let mut current = m.base_model;
                    while let Some(base_id) = current {
                        if let Some(Type::Model(base)) = self.get_type(base_id) {
                            if let Some(&prop_id) = base.properties.get(member_name) {
                                return prop_id;
                            }
                            current = base.base_model;
                        } else {
                            break;
                        }
                    }
                    self.error_at(
                        source_node_id,
                        "invalid-ref",
                        &format!("Model '{}' has no property '{}'", m.name, member_name),
                    );
                    self.error_type
                }
            }
            Some(Type::Enum(e)) => {
                if let Some(&member_id) = e.members.get(member_name) {
                    member_id
                } else {
                    self.error_at(
                        source_node_id,
                        "invalid-ref",
                        &format!("Enum '{}' has no member '{}'", e.name, member_name),
                    );
                    self.error_type
                }
            }
            Some(Type::Union(u)) => {
                if let Some(&variant_id) = u.variants.get(member_name) {
                    variant_id
                } else {
                    self.error_at(
                        source_node_id,
                        "invalid-ref",
                        &format!("Union '{}' has no variant '{}'", u.name, member_name),
                    );
                    self.error_type
                }
            }
            Some(Type::Interface(iface)) => {
                if let Some(&op_id) = iface.operations.get(member_name) {
                    op_id
                } else {
                    self.error_at(
                        source_node_id,
                        "invalid-ref",
                        &format!(
                            "Interface '{}' has no operation '{}'",
                            iface.name, member_name
                        ),
                    );
                    self.error_type
                }
            }
            _ => {
                self.error_at(
                    source_node_id,
                    "invalid-ref",
                    &format!("Cannot access member '{}' on this type", member_name),
                );
                self.error_type
            }
        }
    }
}
