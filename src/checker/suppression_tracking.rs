//! Suppression tracking
//!
//! Ported from TypeSpec `compiler/src/core/suppression-tracking.ts`
//! (microsoft/typespec#10805 "Report unused suppressions").
//!
//! The tracker collects every `#suppress` directive in project source, records
//! which ones were actually used to suppress a diagnostic, and can report the
//! unused ones. A suppression is only reported as unused when its code is
//! "available" — i.e. it belongs to the compiler, the built-in linter, or a
//! loaded library — so suppressions aimed at other tools are left alone.

use std::collections::HashMap;

use crate::ast::node::NodeId;
use crate::ast::types::DirectiveExpression;
use crate::parser::{AstBuilder, AstNode};

/// Name of the built-in linter library.
///
/// Ported from TS `builtInLinterLibraryName` in linter.ts. (The standalone
/// `src/linter.rs` module carries the same constant but is not yet wired into
/// the crate; duplicated here to avoid enabling dead code.)
pub const BUILT_IN_LINTER_LIBRARY_NAME: &str = "@typespec/compiler";

/// A parsed `#suppress` directive.
///
/// Ported from TS `SuppressDirective` in types.ts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuppressDirective {
    /// The diagnostic code being suppressed.
    pub code: String,
    /// The justification message (empty string when omitted).
    pub message: String,
    /// The `DirectiveExpression` AST node of the directive.
    pub node: NodeId,
}

/// An unused `#suppress` directive.
///
/// Ported from TS `UnusedSuppression`.
#[derive(Debug, Clone)]
pub struct UnusedSuppression {
    pub directive: SuppressDirective,
}

/// A parsed directive (either `#suppress` or `#deprecated`).
///
/// Ported from TS `Directive` union type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedDirective {
    Suppress(SuppressDirective),
    Deprecated { message: String, node: NodeId },
}

impl ParsedDirective {
    /// The directive name (`"suppress"` or `"deprecated"`).
    pub fn name(&self) -> &'static str {
        match self {
            ParsedDirective::Suppress(_) => "suppress",
            ParsedDirective::Deprecated { .. } => "deprecated",
        }
    }
}

/// Parse a `DirectiveExpression` node into a typed directive.
///
/// Ported from TS `parseDirective`. Returns `None` when the directive is
/// unknown or its first argument is not a string literal (upstream returns
/// `undefined` in both cases).
pub fn parse_directive(node: &DirectiveExpression, ast: &AstBuilder) -> Option<ParsedDirective> {
    let target_name = match ast.id_to_node(node.target) {
        Some(AstNode::Identifier(id)) => id.value.clone(),
        _ => return None,
    };

    // Directive arguments may be string literals or identifiers; upstream maps
    // identifiers to their text and literals to their value. Only string
    // arguments are accepted as the code/message below (TS checks
    // `typeof args[0] !== "string"`).
    let args: Vec<Option<String>> = node
        .arguments
        .iter()
        .map(|&arg_id| match ast.id_to_node(arg_id) {
            Some(AstNode::StringLiteral(s)) => Some(s.value.clone()),
            Some(AstNode::Identifier(id)) => Some(id.value.clone()),
            _ => None,
        })
        .collect();

    match target_name.as_str() {
        "suppress" => {
            let code = args.first().and_then(|a| a.clone())?;
            let message = args.get(1).and_then(|a| a.clone()).unwrap_or_default();
            Some(ParsedDirective::Suppress(SuppressDirective {
                code,
                message,
                node: node.id,
            }))
        }
        "deprecated" => {
            let message = args.first().and_then(|a| a.clone())?;
            Some(ParsedDirective::Deprecated {
                message,
                node: node.id,
            })
        }
        _ => None,
    }
}

#[derive(Debug)]
struct SuppressionRecord {
    directive: SuppressDirective,
    used: bool,
}

/// Tracks `#suppress` directives and which of them were used.
///
/// Ported from TS `SuppressionTracker` / `createSuppressionTracker`.
#[derive(Debug, Default)]
pub struct SuppressionTracker {
    /// Records keyed by the directive's `DirectiveExpression` node id.
    suppressions: HashMap<NodeId, SuppressionRecord>,
    /// Insertion order for deterministic iteration (TS preserves Map order).
    order: Vec<NodeId>,
}

impl SuppressionTracker {
    /// Create an empty tracker.
    pub fn new() -> Self {
        Self::default()
    }

    /// Collect all `#suppress` directives from the AST.
    ///
    /// Ported from TS `collectSuppressions`. Only directives in project code
    /// are collected (upstream filters by `locationContexts.type === "project"`;
    /// the Rust port distinguishes injected library source via
    /// `library_line_offset`).
    pub fn collect_from_ast(ast: &AstBuilder) -> Self {
        let mut tracker = Self::new();
        for_each_project_directives(ast, |_decl_node_id, directive_ids| {
            for &dir_id in directive_ids {
                let Some(AstNode::DirectiveExpression(dir_expr)) = ast.id_to_node(dir_id) else {
                    continue;
                };
                if let Some(ParsedDirective::Suppress(directive)) = parse_directive(dir_expr, ast) {
                    tracker.suppressions.insert(
                        dir_id,
                        SuppressionRecord {
                            directive,
                            used: false,
                        },
                    );
                    tracker.order.push(dir_id);
                }
            }
        });
        tracker
    }

    /// Mark a suppression directive as used.
    ///
    /// Ported from TS `markUsed`.
    pub fn mark_used(&mut self, directive_node: NodeId) {
        if let Some(record) = self.suppressions.get_mut(&directive_node) {
            record.used = true;
        }
    }

    /// Return all suppressions that were never used and whose code is
    /// available from a known diagnostic source.
    ///
    /// Ported from TS `getUnusedSuppressions`. The `is_code_available`
    /// predicate mirrors TS `getSuppressionSourceAvailability`.
    pub fn get_unused_suppressions(
        &self,
        is_code_available: impl Fn(&str) -> bool,
    ) -> Vec<UnusedSuppression> {
        let mut unused = Vec::new();
        for &node_id in &self.order {
            let Some(record) = self.suppressions.get(&node_id) else {
                continue;
            };
            if record.used {
                continue;
            }
            if !is_code_available(&record.directive.code) {
                continue;
            }
            unused.push(UnusedSuppression {
                directive: record.directive.clone(),
            });
        }
        unused
    }

    /// Get the suppress directive for a directive node, if any.
    pub fn get(&self, directive_node: NodeId) -> Option<&SuppressDirective> {
        self.suppressions.get(&directive_node).map(|r| &r.directive)
    }

    /// Number of tracked suppressions.
    pub fn len(&self) -> usize {
        self.suppressions.len()
    }

    /// Whether no suppressions are tracked.
    pub fn is_empty(&self) -> bool {
        self.suppressions.is_empty()
    }
}

/// Find `#suppress` directives that duplicate an already-suppressed code on
/// the same node.
///
/// Ported from TS `findDuplicateSuppressions` (microsoft/typespec#11113).
/// Only project source is scanned (injected library regions are skipped, as
/// in [`SuppressionTracker::collect_from_ast`]).
pub fn find_duplicate_suppressions(ast: &AstBuilder) -> Vec<SuppressDirective> {
    let mut duplicates = Vec::new();
    for_each_project_directives(ast, |_decl_node_id, directive_ids| {
        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
        for &dir_id in directive_ids {
            let Some(AstNode::DirectiveExpression(dir_expr)) = ast.id_to_node(dir_id) else {
                continue;
            };
            if let Some(ParsedDirective::Suppress(directive)) = parse_directive(dir_expr, ast)
                && !seen.insert(directive.code.clone())
            {
                duplicates.push(directive);
            }
        }
    });
    duplicates
}

/// Find suppress directives whose code uses an ambiguous short name (one
/// that resolves to two or more loaded libraries).
///
/// Ported from TS `findAmbiguousSuppressions` (microsoft/typespec#11209).
pub fn find_ambiguous_suppressions(
    ast: &AstBuilder,
    code_resolver: Option<&crate::diagnostic_code::DiagnosticCodeResolver>,
) -> Vec<(
    SuppressDirective,
    crate::diagnostic_code::AmbiguousShortName,
)> {
    let mut ambiguous = Vec::new();
    let Some(code_resolver) = code_resolver else {
        return ambiguous;
    };

    for_each_project_directives(ast, |_decl_node_id, directive_ids| {
        for &dir_id in directive_ids {
            let Some(AstNode::DirectiveExpression(dir_expr)) = ast.id_to_node(dir_id) else {
                continue;
            };
            if let Some(ParsedDirective::Suppress(directive)) = parse_directive(dir_expr, ast)
                && let Some(conflict) = code_resolver.get_ambiguous_short_name(&directive.code)
            {
                ambiguous.push((directive, conflict));
            }
        }
    });

    ambiguous
}

/// Iterate over the directive lists of all project (non-library)
/// declarations, in source order.
///
/// Upstream walks each source file whose location context is `"project"`;
/// the Rust port distinguishes injected library source via
/// `library_line_offset`.
fn for_each_project_directives(ast: &AstBuilder, mut f: impl FnMut(NodeId, &[NodeId])) {
    // NodeIds are assigned in parse order, so sorting yields source order.
    let mut decls: Vec<(NodeId, &Vec<NodeId>)> =
        ast.directives_map.iter().map(|(&id, v)| (id, v)).collect();
    decls.sort_by_key(|&(id, _)| id);
    for (decl_node_id, directive_ids) in decls {
        let is_library = ast
            .node_span(decl_node_id)
            .is_some_and(|span| span.start.line as usize <= ast.library_line_offset);
        if is_library {
            continue;
        }
        f(decl_node_id, directive_ids);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    #[test]
    fn test_collect_suppress_directives() {
        let result = parse(
            r#"
            #suppress "deprecated" "not needed anymore"
            model Foo {}
            "#,
        );
        let tracker = SuppressionTracker::collect_from_ast(&result.builder);
        assert_eq!(tracker.len(), 1);
        let dir = tracker.get(*tracker.order.first().unwrap()).unwrap();
        assert_eq!(dir.code, "deprecated");
        assert_eq!(dir.message, "not needed anymore");
    }

    #[test]
    fn test_mark_used() {
        let result = parse(
            r#"
            #suppress "deprecated" "reason"
            model Foo {}
            "#,
        );
        let mut tracker = SuppressionTracker::collect_from_ast(&result.builder);
        let node = *tracker.order.first().unwrap();
        tracker.mark_used(node);
        let unused = tracker.get_unused_suppressions(|_| true);
        assert!(unused.is_empty());
    }

    #[test]
    fn test_unused_filtered_by_availability() {
        let result = parse(
            r#"
            #suppress "test-emitter/not-run" "only emitted by another tool"
            model Foo {}
            "#,
        );
        let tracker = SuppressionTracker::collect_from_ast(&result.builder);
        // Nothing is available -> nothing reported unused.
        let unused = tracker.get_unused_suppressions(|_| false);
        assert!(unused.is_empty());
        // Available -> reported unused.
        let unused = tracker.get_unused_suppressions(|_| true);
        assert_eq!(unused.len(), 1);
        assert_eq!(unused[0].directive.code, "test-emitter/not-run");
    }

    #[test]
    fn test_parse_directive_requires_string_code() {
        // #suppress without arguments parses to no directive (upstream: undefined)
        let result = parse(
            r#"
            #suppress
            model Foo {}
            "#,
        );
        let tracker = SuppressionTracker::collect_from_ast(&result.builder);
        assert!(tracker.is_empty());
    }

    #[test]
    fn test_library_suppressions_not_collected() {
        // Inject a library source containing a #suppress; it must be skipped.
        let lib = "#suppress \"deprecated\" \"library suppression\"\nmodel LibModel {}\n";
        let result = crate::parser::parse_with_libraries("model Foo {}", vec![lib.to_string()]);
        let tracker = SuppressionTracker::collect_from_ast(&result.builder);
        assert!(
            tracker.is_empty(),
            "library suppressions should not be tracked: {:?}",
            tracker.order
        );
    }
}
