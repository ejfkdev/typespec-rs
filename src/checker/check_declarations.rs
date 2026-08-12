//! Declaration checking
//!
//! Ported from TypeSpec compiler declaration checking methods

use super::*;
use crate::ast::types::ModifierKind;
use crate::modifiers::{self, ModifierFlags};

impl Checker {
    // ========================================================================
    // Decorator declaration checking
    // ========================================================================

    pub(crate) fn check_decorator_declaration(
        &mut self,
        ctx: &CheckContext,
        node_id: NodeId,
    ) -> TypeId {
        let (ast, node) = require_ast_node!(self, node_id, DecoratorDeclaration, self.error_type);

        let name = Self::get_identifier_name(&ast, node.name);

        // Check the target parameter to get its type constraint
        let target_type_id = self.check_node(ctx, node.target);

        // Resolve the target type name for constraint checking
        let target_type_name = match self.get_type(target_type_id) {
            Some(Type::Model(_)) => "Model",
            Some(Type::Union(_)) => "Union",
            Some(Type::Interface(_)) => "Interface",
            Some(Type::Enum(_)) => "Enum",
            Some(Type::Scalar(_)) => "Scalar",
            Some(Type::Operation(_)) => "Operation",
            Some(Type::Namespace(_)) => "Namespace",
            Some(Type::Intrinsic(i)) => match i.name {
                IntrinsicTypeName::ErrorType => "ErrorType",
                IntrinsicTypeName::Void => "void",
                IntrinsicTypeName::Never => "never",
                IntrinsicTypeName::Unknown => "unknown",
                IntrinsicTypeName::Null => "null",
            },
            _ => "unknown",
        }
        .to_string();

        let target = Some(target_type_id);

        let mut parameters = Vec::new();
        for &param_id in &node.parameters {
            let param_node = match ast.id_to_node(param_id) {
                Some(AstNode::FunctionParameter(fp)) => fp.clone(),
                _ => continue,
            };
            let param_name = Self::get_identifier_name(&ast, param_node.name);
            let param_type = param_node.type_annotation.map(|t| self.check_node(ctx, t));

            // Check rest parameter must be array type
            if param_node.rest
                && let Some(type_ann) = param_node.type_annotation
            {
                // Check if the type annotation is an array expression (...: string[])
                let is_array_expr =
                    matches!(ast.id_to_node(type_ann), Some(AstNode::ArrayExpression(_)));
                if !is_array_expr {
                    self.error_at(
                        node_id,
                        "rest-parameter-array",
                        "A rest parameter must be of an array type.",
                    );
                }
            }

            parameters.push(FunctionParameterType {
                id: self.next_type_id(),
                name: param_name,
                node: Some(param_id),
                r#type: param_type,
                optional: param_node.optional,
                rest: param_node.rest,
                is_finished: true,
            });
        }

        // Determine the declaration kind: `auto dec` or `extern dec`
        // (microsoft/typespec#10197).
        let has_extern = node.modifiers.iter().any(|&mod_id| {
            matches!(ast.id_to_node(mod_id), Some(AstNode::Modifier(m)) if m.kind == ModifierKind::Extern)
        });
        let is_auto = node.modifiers.iter().any(|&mod_id| {
            matches!(ast.id_to_node(mod_id), Some(AstNode::Modifier(m)) if m.kind == ModifierKind::Auto)
        });

        if is_auto && !self.is_compiler_feature_enabled("auto-decorators", Some(node_id)) {
            self.error_at(
                node_id,
                "auto-decorator-disabled",
                "Auto decorator declarations require the 'auto-decorators' feature to be enabled. Add 'auto-decorators' to the 'features' list in your tspconfig.yaml.",
            );
        }

        if is_auto {
            // Auto decorators get a compiler-generated implementation that
            // stores their arguments — no external implementation needed.
        } else if has_extern {
            // Extern decorator must have a JS implementation
            self.error_at(
                node_id,
                "missing-implementation",
                "Extern declaration must have an implementation in JS file.",
            );
        }

        let type_id = self.create_type(Type::Decorator(DecoratorType {
            id: self.next_type_id(),
            name: name.clone(),
            node: Some(node_id),
            namespace: self.current_namespace,
            target,
            target_type: target_type_name,
            parameters,
            declaration_kind: if is_auto {
                DecoratorDeclarationKind::Auto
            } else {
                DecoratorDeclarationKind::Extern
            },
            is_finished: true,
        }));

        self.node_type_map.insert(node_id, type_id);
        if !name.is_empty() {
            // Record the decorator's FQN for auto decorator state keys
            // (microsoft/typespec#10197).
            let fqn = self.build_fqn(&name);
            self.decorator_fqns.insert(type_id, fqn);
            self.register_declared_type(&name, type_id);
        }

        type_id
    }

    // ========================================================================
    // Function declaration checking
    // ========================================================================

    pub(crate) fn check_function_declaration(
        &mut self,
        ctx: &CheckContext,
        node_id: NodeId,
    ) -> TypeId {
        let (ast, node) = require_ast_node!(self, node_id, FunctionDeclaration, self.error_type);

        let name = Self::get_identifier_name(&ast, node.name);

        // Check modifiers — extern is required on fn declarations, auto is
        // not allowed (handled by check_modifiers_and_report, which mirrors
        // upstream SYNTAX_MODIFIERS for FunctionDeclarationStatement).
        let mut modifier_flags = ModifierFlags::None;
        for &mod_id in &node.modifiers {
            if let Some(AstNode::Modifier(m)) = ast.id_to_node(mod_id) {
                modifier_flags = modifier_flags | modifiers::modifier_to_flag(m.kind);
            }
        }
        let _has_extern = modifier_flags.contains(ModifierFlags::Extern);

        // Function declarations are gated behind the "function-declarations"
        // compiler feature (microsoft/typespec#10826). Without it, emit the
        // experimental-feature warning (TS messageId: functionDeclarations).
        if !self.is_compiler_feature_enabled("function-declarations", Some(node_id)) {
            self.warning_at(
                node_id,
                "experimental-feature",
                "Function declarations are an experimental feature that may change in the future. Use with caution and consider providing feedback to the TypeSpec team.",
            );
        }

        // Manually unpack parameters (same pattern as check_decorator_declaration)
        // because check_node_impl has no FunctionParameter arm
        let mut parameters = Vec::new();
        let mut seen_param_names = std::collections::HashSet::new();
        for &param_id in &node.parameters {
            let param_node = match ast.id_to_node(param_id) {
                Some(AstNode::FunctionParameter(fp)) => fp.clone(),
                _ => continue,
            };
            let param_name = Self::get_identifier_name(&ast, param_node.name);
            let param_type = param_node.type_annotation.map(|t| self.check_node(ctx, t));

            // Check for duplicate parameter names
            if !param_name.is_empty() && seen_param_names.contains(&param_name) {
                self.error_at(
                    node_id,
                    "duplicate-parameter",
                    &format!(
                        "Duplicate parameter name '{}' in function '{}'.",
                        param_name, name
                    ),
                );
            }
            seen_param_names.insert(param_name.clone());

            if param_node.rest
                && let Some(type_ann) = param_node.type_annotation
            {
                let is_array_expr =
                    matches!(ast.id_to_node(type_ann), Some(AstNode::ArrayExpression(_)));
                if !is_array_expr {
                    self.error_at(
                        node_id,
                        "rest-parameter-array",
                        "A rest parameter must be of an array type.",
                    );
                }
            }

            parameters.push(FunctionParameterType {
                id: self.next_type_id(),
                name: param_name,
                node: Some(param_id),
                r#type: param_type,
                optional: param_node.optional,
                rest: param_node.rest,
                is_finished: true,
            });
        }

        let return_type = node.return_type.map(|ret_id| self.check_node(ctx, ret_id));

        // Use pre-registered type if available, otherwise create new
        let current_ns = self.current_namespace;
        let type_id = if let Some(&existing_id) = self.node_type_map.get(&node_id) {
            // Update pre-registered type in-place
            if let Some(t) = self.get_type_mut(existing_id)
                && let Type::FunctionType(f) = t
            {
                f.parameters = parameters;
                f.return_type = return_type;
                f.namespace = current_ns;
                f.is_finished = true;
            }
            existing_id
        } else {
            let new_id = self.create_type(Type::FunctionType(FunctionTypeType {
                id: self.next_type_id(),
                name: name.clone(),
                node: Some(node_id),
                namespace: current_ns,
                parameters,
                return_type,
                is_finished: true,
            }));
            self.node_type_map.insert(node_id, new_id);
            if !name.is_empty() {
                self.register_declared_type(&name, new_id);
            }
            new_id
        };

        // Also create FunctionParameter types in the type store for each parameter
        // (needed for type graph serialization and lookup)
        if let Some(Type::FunctionType(ft)) = self.get_type(type_id).cloned() {
            for param in &ft.parameters {
                let param_type_id = self.create_type(Type::FunctionParameter(param.clone()));
                let _ = param_type_id;
            }
        }

        // Register function in its namespace
        if !name.is_empty()
            && let Some(ns_id) = current_ns
            && let Some(Type::Namespace(ns)) = self.get_type_mut(ns_id)
            && !ns.function_declarations.contains_key(&name)
        {
            ns.function_declarations.insert(name.clone(), type_id);
            ns.function_declaration_names.push(name.clone());
        }

        type_id
    }
}
