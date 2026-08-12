//! Directive processing
//!
//! Ported from TypeSpec compiler directive processing methods

use super::*;

/// A suppressed diagnostic code together with the `#suppress` directive node
/// that declared it (so the suppression tracker can mark it used).
#[derive(Debug, Clone)]
pub struct SuppressedCode {
    pub code: String,
    pub directive_node: NodeId,
}

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
                                self.error_at(
                                    node_id,
                                    "duplicate-deprecation",
                                    "Duplicate #deprecated directive",
                                );
                            } else {
                                has_deprecated = true;
                                // Validate that #deprecated has a string argument
                                if dir_expr.arguments.is_empty() {
                                    self.error_at(
                                        node_id,
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
                                        self.error_at(node_id, "invalid-deprecation-argument", "#deprecated directive requires a string message argument.");
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
                                    .extend(suppressed_codes.into_iter().map(|code| {
                                        SuppressedCode {
                                            code,
                                            directive_node: dir_id,
                                        }
                                    }));
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

        // Don't emit if "deprecated" is suppressed on any currently pending
        // declaration; mark the suppression as used.
        if let Some(dir_node) = self.find_suppressing_directive("deprecated") {
            self.suppression_tracker.mark_used(dir_node);
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

    /// Find the `#suppress` directive node suppressing the given diagnostic
    /// code on any currently pending declaration node.
    ///
    /// Ported from TS `findDirectiveSuppressingOnNode`, adapted to the Rust
    /// port's pending-declaration model (the set of declarations currently
    /// being checked stands in for the target-node ancestor walk).
    pub(crate) fn find_suppressing_directive(&mut self, code: &str) -> Option<NodeId> {
        // Short diagnostic codes are normalized through the code resolver when
        // available (microsoft/typespec#11209).
        let resolved_code = self.resolve_diagnostic_code(code);
        let pending: Vec<NodeId> = self.pending_type_checks.iter().copied().collect();
        for node_id in pending {
            self.ensure_suppressions_collected(node_id);
            if let Some(entries) = self.suppressed_diagnostics.get(&node_id) {
                for entry in entries {
                    if self.resolve_diagnostic_code(&entry.code) == resolved_code {
                        return Some(entry.directive_node);
                    }
                }
            }
        }
        None
    }

    /// Normalize a diagnostic code through the code resolver when present
    /// (short library names resolve to their full `${package}/${code}` form).
    pub(crate) fn resolve_diagnostic_code(&self, code: &str) -> String {
        match &self.diagnostic_code_resolver {
            Some(resolver) => resolver.resolve_code(code),
            None => code.to_string(),
        }
    }

    /// Lazily collect `#suppress` directives attached to a node into
    /// `suppressed_diagnostics` (idempotent).
    ///
    /// Declarations run `process_directives` eagerly, but directives can also
    /// appear on members (model properties, enum members, ...) which are not
    /// declaration-checked; upstream's suppression lookup walks the target
    /// node's ancestors, so any pending node may carry suppressions.
    fn ensure_suppressions_collected(&mut self, node_id: NodeId) {
        if !self.suppression_scanned.insert(node_id) {
            return;
        }
        let Some(ast) = self.require_ast() else {
            return;
        };
        let Some(directive_ids) = ast.get_directives(node_id).cloned() else {
            return;
        };
        for dir_id in directive_ids {
            let Some(AstNode::DirectiveExpression(dir_expr)) = ast.id_to_node(dir_id) else {
                continue;
            };
            if let Some(suppression_tracking::ParsedDirective::Suppress(directive)) =
                suppression_tracking::parse_directive(dir_expr, &ast)
            {
                self.suppressed_diagnostics
                    .entry(node_id)
                    .or_default()
                    .push(SuppressedCode {
                        code: directive.code,
                        directive_node: dir_id,
                    });
            }
        }
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
