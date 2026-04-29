//! Augment decorator and using declaration checking
//!
//! Ported from TypeSpec compiler augment/using checking methods

use super::*;
use crate::ast::types::MemberSelector;

impl Checker {
    // ========================================================================
    // Using and augment decorator checking
    // ========================================================================

    pub(crate) fn check_using(&mut self, node_id: NodeId) {
        let (ast, node) = require_ast_node!(self, node_id, UsingDeclaration);

        // Get the namespace name from the using declaration
        let ns_name = Self::get_identifier_name(&ast, node.name);

        // Check for duplicate using of the same namespace
        if self
            .using_declarations
            .iter()
            .any(|(_, existing_name)| existing_name == &ns_name)
        {
            self.error(
                "duplicate-using",
                &format!("Using of namespace '{}' is already declared.", ns_name),
            );
            return;
        }

        let ctx = CheckContext::new();
        let target_type = self.check_node(&ctx, node.name);

        match self.get_type(target_type) {
            Some(Type::Namespace(_)) => {
                // Record this using declaration for unused-using detection
                self.using_declarations.push((node_id, ns_name));
            }
            Some(_) | None => {
                self.error("using-invalid-ref", "Using must reference a namespace.");
            }
        }
    }

    /// Try to resolve a name via using declarations.
    /// Looks for the name in all using'd namespaces and returns the TypeId if found.
    /// Ported from TS name-resolver.ts resolveIdentifierInScope.
    pub(crate) fn resolve_via_using(&self, name: &str) -> Option<TypeId> {
        for (_, using_ns_name) in &self.using_declarations {
            // Find the namespace type by name
            if let Some(&ns_id) = self.declared_types.get(using_ns_name)
                && let Some(Type::Namespace(ns)) = self.get_type(ns_id)
                && let Some(type_id) = ns.lookup_member(name)
            {
                return Some(type_id);
            }
        }
        None
    }

    /// Check if a type name was resolved via a using'd namespace, and if so, mark that using as used.
    pub(crate) fn mark_using_as_used_if_applicable(&mut self, _name: &str, type_id: TypeId) {
        // Get the namespace that this type belongs to
        let ns_id = self.get_type(type_id).and_then(|t| t.namespace());
        if let Some(ns_id) = ns_id
            && let Some(ns_name) = self.get_type(ns_id).and_then(|t| match t {
                Type::Namespace(ns) => Some(ns.name.clone()),
                _ => None,
            })
        {
            // If this using's namespace matches, mark it as used
            if self
                .using_declarations
                .iter()
                .any(|(_, using_ns)| *using_ns == ns_name)
            {
                self.used_using_names.insert(ns_name);
            }
        }
    }

    /// Report unused using declarations as warnings.
    /// Validate internal decorator usage.
    /// Ported from TS checker.ts internalDecoratorValidation().
    /// In the TS implementation, this primarily calls validateInheritanceDiscriminatedUnions().
    /// The actual access control for internal symbols is handled by check_internal_visibility
    /// during identifier/member resolution.
    /// TODO: implement validateInheritanceDiscriminatedUnions when needed.
    pub(crate) fn internal_decorator_validation(&mut self) {}

    pub(crate) fn report_unused_usings(&mut self) {
        // Collect unused usings first to avoid borrow conflicts
        let unused: Vec<(NodeId, String)> = self
            .using_declarations
            .iter()
            .filter(|(_, ns_name)| !self.used_using_names.contains(ns_name))
            .cloned()
            .collect();

        for (_node_id, ns_name) in unused {
            // Only report unused-using if the namespace itself is valid (no invalid-ref errors)
            // Check if the namespace was resolved successfully by looking in declared_types
            // or by checking if the using name refers to a known namespace type
            let is_valid_namespace = self.declared_types.contains_key(&ns_name)
                || self.is_valid_namespace_name(&ns_name);
            if is_valid_namespace {
                self.warning(
                    "unused-using",
                    &format!("'using {}' is declared but never used.", ns_name),
                );
            }
        }
    }

    /// Check if a name refers to a valid namespace (including dotted names like A.B)
    pub(crate) fn is_valid_namespace_name(&self, name: &str) -> bool {
        // For dotted names like A.B, check if A is a namespace containing B
        if let Some(dot_pos) = name.find('.') {
            let parent_name = &name[..dot_pos];
            let child_name = &name[dot_pos + 1..];

            if let Some(&parent_id) = self.declared_types.get(parent_name)
                && let Some(Type::Namespace(ns)) = self.get_type(parent_id)
            {
                return ns.namespaces.contains_key(child_name);
            }
            false
        } else {
            // Simple name - check if it's a namespace type in declared_types
            self.declared_types
                .get(name)
                .is_some_and(|&id| matches!(self.get_type(id), Some(Type::Namespace(_))))
        }
    }

    pub(crate) fn check_augment_decorator(&mut self, ctx: &CheckContext, node_id: NodeId) {
        let (ast, node) = require_ast_node!(self, node_id, AugmentDecoratorStatement);

        // node.target = decorator name (e.g., "doc" in @@doc(A))
        // node.target_type = type to augment (e.g., TypeReference for "A" in @@doc(A))
        // node.arguments = decorator arguments after the target type

        // Per TS checkDecoratorApplication: resolve the decorator name (node.target)
        // and validate it references a valid decorator declaration.
        // If not found, this will produce an invalid-ref diagnostic.
        let decorator_name = Self::get_identifier_name(&ast, node.target);
        let decorator_found = self.declared_types.contains_key(&decorator_name)
            || self.node_type_map.contains_key(&node.target);
        if !decorator_found {
            // Try to check the target node - this may produce invalid-ref
            let dec_target_type = self.check_node(ctx, node.target);
            // If it resolved to error type and we didn't get a diagnostic, add one
            if dec_target_type == self.error_type
                && !self.diagnostics().iter().any(|d| d.code == "invalid-ref")
            {
                self.error(
                    "invalid-ref",
                    &format!("Unknown decorator '{}'", decorator_name),
                );
            }
        }

        // Check the target type reference - this resolves the type and validates it exists
        let target_type = self.check_node(ctx, node.target_type);

        // If target resolved to error, check the AST for template instance patterns
        // that we can detect even without full member expression resolution.
        // This handles cases like: Foo.test<string>, FooString.test, test::returnType
        if target_type == self.error_type {
            let mut found_template_instance_target = false;
            Self::check_augment_target_ast_for_template_instance(
                &ast,
                node.target_type,
                &mut found_template_instance_target,
            );

            // Also handle metatype references like `test::returnType` where
            // the resolved type would be a template instance.
            // TS resolves these through nodeInterfaces/TypePrototypes at the
            // name-resolver level. Our member expression resolution doesn't
            // handle `::` metatype yet, so we do it manually here.
            if !found_template_instance_target {
                found_template_instance_target =
                    self.check_metatype_template_instance(&ast, node.target_type);
            }

            if found_template_instance_target {
                self.error(
                    "augment-decorator-target",
                    "Augment decorator cannot target a template instance.",
                );
            }
        }

        // Resolve through alias (Scalar with base_scalar) to get the actual underlying type
        let resolved_type = self.resolve_type_target(target_type);

        // Per TS checker.ts checkAugmentDecorator:
        // 1. If the target is a template instantiation → error "noInstance"
        // 2. If the target is not a declaration or member (e.g., model/union expression) → error
        // 3. If the target is an alias → check the alias value's AST kind

        match self.get_type(resolved_type) {
            Some(Type::Model(m)) if m.name.is_empty() => {
                self.error(
                    "augment-decorator-target",
                    "Augment decorator cannot target a model expression.",
                );
            }
            Some(Type::Union(u)) if u.name.is_empty() => {
                self.error(
                    "augment-decorator-target",
                    "Augment decorator cannot target a union expression.",
                );
            }
            Some(Type::Intrinsic(i)) if i.name == IntrinsicTypeName::ErrorType => {}
            _ => {}
        }

        // Check for template instantiation targets
        // TS: checkAugmentDecorator checks links.isTemplateInstantiation
        // This is set when the targetType node resolves to a template instantiation like Foo<string>
        if let Some(links) = self.symbol_links.get(&node.target_type)
            && links.is_template_instantiation
        {
            self.error(
                "augment-decorator-target",
                "Augment decorator cannot target a template instance.",
            );
        }
        // Also check if the resolved type itself is a template instance
        // (for cases where is_template_instantiation wasn't set on the node directly,
        //  e.g., via alias that resolves to a template instance)
        if resolved_type != self.error_type {
            let is_template_instance = self.is_template_instance(resolved_type);
            if is_template_instance {
                self.error(
                    "augment-decorator-target",
                    "Augment decorator cannot target a template instance.",
                );
            }
        }

        // Check if the target is a member of a template instance container
        // e.g., @@doc(FooString.test) where FooString = Foo<string>
        // In TS, this is checked by verifying the member's container is a template instantiation
        // The target_type is a TypeReference, and its name may be a MemberExpression
        let target_type_name = match ast.id_to_node(node.target_type) {
            Some(AstNode::TypeReference(tr)) => Some(tr.name),
            _ => None,
        };
        let member_expr_id = target_type_name.and_then(|name_id| match ast.id_to_node(name_id) {
            Some(AstNode::MemberExpression(_)) => Some(name_id),
            _ => None,
        });
        if let Some(me_id) = member_expr_id
            && let Some(AstNode::MemberExpression(me)) = ast.id_to_node(me_id)
        {
            let obj_name = Self::get_identifier_name(&ast, me.object);
            if let Some(&type_id) = self.declared_types.get(&obj_name) {
                // Check if the object type is a template instance
                if self.is_template_instance(type_id) {
                    self.error(
                        "augment-decorator-target",
                        "Augment decorator cannot target a member of a template instance.",
                    );
                }
                // Check through alias chain
                let resolved = self.resolve_alias_chain(type_id);
                if resolved != type_id && self.is_template_instance(resolved) {
                    self.error(
                        "augment-decorator-target",
                        "Augment decorator cannot target a member of a template instance.",
                    );
                }
                // Check if alias value has template arguments
                if let Some(Type::Scalar(s)) = self.get_type(type_id)
                    && let Some(alias_node_id) = s.node
                    && let Some(AstNode::AliasStatement(alias_decl)) = ast.id_to_node(alias_node_id)
                    && let Some(AstNode::TypeReference(tr)) = ast.id_to_node(alias_decl.value)
                    && !tr.arguments.is_empty()
                {
                    self.error(
                        "augment-decorator-target",
                        "Augment decorator cannot target a member of a template instance.",
                    );
                }
            }
        }

        // Also check if the target type is an alias pointing to a model/union expression
        // TS: if finalSymbol is an Alias, check the aliasNode.value.kind
        if target_type != self.error_type
            && let Some(Type::Scalar(s)) = self.get_type(target_type)
        {
            // This is an alias - check what the alias value's AST kind is
            if let Some(alias_node_id) = s.node
                && let Some(AstNode::AliasStatement(alias_decl)) = ast.id_to_node(alias_node_id)
            {
                match ast.id_to_node(alias_decl.value) {
                    Some(AstNode::ModelExpression(_)) => {
                        self.error(
                            "augment-decorator-target",
                            "Augment decorator cannot target a model expression.",
                        );
                    }
                    Some(AstNode::UnionExpression(_)) => {
                        self.error(
                            "augment-decorator-target",
                            "Augment decorator cannot target a union expression.",
                        );
                    }
                    _ => {}
                }
            }
        }

        // Check decorator arguments
        for &arg_id in &node.arguments {
            self.check_node(ctx, arg_id);
        }
    }

    /// Resolve through alias (Scalar with base_scalar) to get the underlying type.
    /// Unlike resolve_alias_chain which stops when names differ,
    /// this always follows base_scalar to get the final target type.
    pub(crate) fn resolve_type_target(&self, type_id: TypeId) -> TypeId {
        match self.get_type(type_id) {
            Some(Type::Scalar(s)) => {
                if let Some(base_id) = s.base_scalar {
                    self.resolve_type_target(base_id)
                } else {
                    type_id
                }
            }
            _ => type_id,
        }
    }

    /// Check the AST of an augment decorator's target type for template instance patterns.
    /// This is a fallback for when `check_node` returns error_type because member
    /// expression resolution isn't fully implemented.
    /// Walks the AST looking for TypeReferences that have template arguments,
    /// which indicates a template instantiation target.
    pub(crate) fn check_augment_target_ast_for_template_instance(
        ast: &AstBuilder,
        node_id: NodeId,
        found: &mut bool,
    ) {
        match ast.id_to_node(node_id) {
            Some(AstNode::TypeReference(tr)) => {
                if !tr.arguments.is_empty() {
                    *found = true;
                }
                // Also check the name part (could be a MemberExpression)
                Self::check_augment_target_ast_for_template_instance(ast, tr.name, found);
            }
            Some(AstNode::MemberExpression(me)) => {
                Self::check_augment_target_ast_for_template_instance(ast, me.object, found);
                Self::check_augment_target_ast_for_template_instance(ast, me.property, found);
            }
            Some(AstNode::CallExpression(call)) if !call.arguments.is_empty() => {
                *found = true;
            }
            _ => {}
        }
    }

    /// Check if a metatype reference (e.g., `test::returnType`) resolves to
    /// a template instance. This handles the case where our member expression
    /// resolution doesn't support `::` selector metatype properties yet.
    fn check_metatype_template_instance(&self, ast: &AstBuilder, node_id: NodeId) -> bool {
        let me = match ast.id_to_node(node_id) {
            Some(AstNode::MemberExpression(me)) => me,
            Some(AstNode::TypeReference(tr)) => {
                // The name of the TypeReference might be a MemberExpression
                match ast.id_to_node(tr.name) {
                    Some(AstNode::MemberExpression(me)) => me,
                    _ => return false,
                }
            }
            _ => return false,
        };

        // Only handle `::` (double colon) selector for metatype
        if me.selector != MemberSelector::DoubleColon {
            return false;
        }

        // Get the property name (e.g., "returnType", "parameters", "type")
        let property_name = Self::get_identifier_name(ast, me.property);

        // Resolve the object side (e.g., "test")
        let object_name = Self::get_identifier_name(ast, me.object);

        // Look up the object type
        let object_type_id = match self.declared_types.get(&object_name) {
            Some(&id) => id,
            None => return false,
        };

        match property_name.as_str() {
            "returnType" => {
                // For operations, check if the return type is a template instance
                if let Some(Type::Operation(op)) = self.get_type(object_type_id)
                    && let Some(rt) = op.return_type
                {
                    return self.is_template_instance(rt);
                }
            }
            "parameters" => {
                // For operations, check if any parameter type is a template instance
                if let Some(Type::Operation(_op)) = self.get_type(object_type_id) {
                    // Parameters are in the model properties - simplified check
                    return false;
                }
            }
            "type" => {
                // For model properties, check the property type
                if let Some(Type::ModelProperty(prop)) = self.get_type(object_type_id) {
                    return self.is_template_instance(prop.r#type);
                }
            }
            _ => {}
        }

        false
    }
}
