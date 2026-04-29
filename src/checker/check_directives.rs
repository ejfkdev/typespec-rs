//! Directive processing
//!
//! Ported from TypeSpec compiler directive processing methods

use super::*;

impl Checker {
    /// Process directives attached to a declaration node (e.g., #deprecated, #suppress).
    /// This reads directives from the AST builder's directives_map and applies them
    /// to the type (e.g., marking as deprecated).
    pub(crate) fn process_directives(&mut self, node_id: NodeId, type_id: TypeId) {
        let ast = require_ast_or!(self);

        let directive_ids = match ast.get_directives(node_id) {
            Some(ids) => ids.clone(),
            None => return,
        };

        let mut has_deprecated = false;
        for &dir_id in &directive_ids {
            let dir_node = match ast.id_to_node(dir_id) {
                Some(n) => n.clone(),
                None => continue,
            };

            match &dir_node {
                AstNode::DirectiveExpression(dir_expr) => {
                    // Get the directive name
                    let target_name = match ast.id_to_node(dir_expr.target) {
                        Some(AstNode::Identifier(id)) => id.value.clone(),
                        _ => continue,
                    };

                    match target_name.as_str() {
                        "deprecated" => {
                            if has_deprecated {
                                self.error(
                                    "duplicate-deprecation",
                                    "Duplicate #deprecated directive",
                                );
                            } else {
                                has_deprecated = true;
                                // Validate that #deprecated has a string argument
                                if dir_expr.arguments.is_empty() {
                                    self.error(
                                        "invalid-deprecation-argument",
                                        "#deprecated directive requires a string message argument.",
                                    );
                                } else {
                                    // Get the deprecation message from the first string argument
                                    let message = dir_expr.arguments.first().and_then(|&arg_id| {
                                        match ast.id_to_node(arg_id) {
                                            Some(AstNode::StringLiteral(s)) => {
                                                Some(s.value.clone())
                                            }
                                            _ => None,
                                        }
                                    });

                                    if let Some(msg) = message {
                                        self.mark_deprecated(type_id, msg);
                                    } else {
                                        // First argument is not a string literal
                                        self.error("invalid-deprecation-argument", "#deprecated directive requires a string message argument.");
                                    }
                                }
                            }
                        }
                        "suppress" => {
                            // Store suppress directives for later use in diagnostic filtering
                            let suppressed_codes: Vec<String> = dir_expr
                                .arguments
                                .first()
                                .and_then(|&arg_id| match ast.id_to_node(arg_id) {
                                    Some(AstNode::StringLiteral(s)) => Some(vec![s.value.clone()]),
                                    _ => None,
                                })
                                .unwrap_or_default();

                            if !suppressed_codes.is_empty() {
                                self.suppressed_diagnostics
                                    .entry(node_id)
                                    .or_default()
                                    .extend(suppressed_codes);
                            }
                        }
                        _ => {}
                    }
                }
                _ => continue,
            }
        }
    }

    /// Process directives for a node and mark it as processed.
    /// Combines process_directives + directives_processed.insert into one call.
    pub(crate) fn process_and_mark_directives(&mut self, node_id: NodeId, type_id: TypeId) {
        self.process_directives(node_id, type_id);
        self.directives_processed.insert(node_id);
    }

    /// Emit a deprecation warning if the referenced type is deprecated and the
    /// current context is not itself deprecated.
    pub(crate) fn emit_deprecated_warning_if_needed(&mut self, type_id: TypeId) {
        if !self.is_deprecated(type_id) {
            return;
        }

        // Don't emit if we're inside a deprecated context
        if self.in_deprecated_context() {
            return;
        }

        // Don't emit if "deprecated" is suppressed on any currently pending declaration
        if self.is_suppressed("deprecated") {
            return;
        }

        let details = self
            .get_deprecation_details(type_id)
            .map(|d| d.message.clone())
            .unwrap_or_else(|| "deprecated".to_string());

        // Get a name for the deprecated type
        let type_name = self
            .get_type(type_id)
            .and_then(|t| t.name().map(|s| s.to_string()))
            .unwrap_or_else(|| "type".to_string());

        self.warning(
            "deprecated",
            &format!("{} is deprecated: {}", type_name, details),
        );
    }

    /// Check if a diagnostic code is suppressed on any currently pending declaration node.
    pub(crate) fn is_suppressed(&self, code: &str) -> bool {
        for &node_id in &self.pending_type_checks {
            if let Some(codes) = self.suppressed_diagnostics.get(&node_id)
                && codes.iter().any(|c| c == code)
            {
                return true;
            }
        }
        false
    }

    /// Check if we're currently inside a deprecated context (i.e., checking
    /// a declaration that is itself marked as deprecated).
    pub(crate) fn in_deprecated_context(&self) -> bool {
        // Check if any of the types being currently checked (pending_type_checks)
        // are deprecated. This means we're inside a deprecated declaration.
        for &node_id in &self.pending_type_checks {
            if let Some(&type_id) = self.node_type_map.get(&node_id)
                && self.is_deprecated(type_id)
            {
                return true;
            }
        }
        false
    }
}
