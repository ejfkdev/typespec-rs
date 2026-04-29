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
                    self.error(
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

        // Extern decorator must have a JS implementation
        // Only emit this for extern-modified declarations
        let has_extern = node.modifiers.iter().any(|&mod_id| {
            matches!(ast.id_to_node(mod_id), Some(AstNode::Modifier(m)) if m.kind == ModifierKind::Extern)
        });
        if has_extern {
            self.error(
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
            is_finished: true,
        }));

        self.node_type_map.insert(node_id, type_id);
        if !name.is_empty() {
            self.declared_types.insert(name, type_id);
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

        // Check for extern modifier - requires JS implementation binding
        let mut modifier_flags = ModifierFlags::None;
        for &mod_id in &node.modifiers {
            if let Some(AstNode::Modifier(m)) = ast.id_to_node(mod_id) {
                modifier_flags = modifier_flags | modifiers::modifier_to_flag(m.kind);
            }
        }
        let has_extern = modifier_flags.contains(ModifierFlags::Extern);
        if has_extern {
            // extern fn must have a JS implementation - for now emit missing-implementation
            self.error(
                "missing-implementation",
                &format!(
                    "Function '{}' is declared as extern but has no implementation.",
                    Self::get_identifier_name(&ast, node.name)
                ),
            );
        }

        let name = Self::get_identifier_name(&ast, node.name);

        let mut parameters = Vec::new();
        for &param_id in &node.parameters {
            let param_type = self.check_node(ctx, param_id);
            parameters.push(param_type);
        }

        let return_type = node.return_type.map(|ret_id| self.check_node(ctx, ret_id));

        let type_id = self.create_type(Type::FunctionType(FunctionTypeType {
            id: self.next_type_id(),
            name: name.clone(),
            node: Some(node_id),
            namespace: self.current_namespace,
            parameters,
            return_type,
            is_finished: true,
        }));

        self.node_type_map.insert(node_id, type_id);
        if !name.is_empty() {
            self.declared_types.insert(name, type_id);
        }

        type_id
    }
}
