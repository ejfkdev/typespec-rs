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
use crate::checker::Checker;
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

    /// Whether a tracked suppression has matched a diagnostic so far.
    pub fn is_used(&self, directive_node: NodeId) -> bool {
        self.suppressions
            .get(&directive_node)
            .is_some_and(|record| record.used)
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

// ============================================================================
// Suppression inspection APIs (microsoft/typespec#12095)
// ============================================================================

/// A declaration or member in a suppression's structural context.
///
/// Ported from TS `SuppressionScope` in `ast/suppressions.ts`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuppressionScope {
    /// The declaration, member, or anonymous container node.
    pub node: NodeId,
    /// Decoded declaration name, if the node is named.
    pub name: Option<String>,
}

/// A `#suppress` directive together with its target, location, and structural
/// context in one source file.
///
/// Ported from TS `Suppression`.
#[derive(Debug, Clone)]
pub struct Suppression {
    /// Parsed directive, preserving its written code and justification.
    pub directive: SuppressDirective,
    /// The syntax node carrying the directive.
    pub target: NodeId,
    /// Directive range in the source file.
    pub span: crate::ast::Span,
    /// Structural context, outermost first, including the target when it is a
    /// declaration or member. Anonymous containers (`model { ... }`) appear
    /// with no name. This is a syntactic context, not a semantic scope.
    pub scope: Vec<SuppressionScope>,
}

/// A project suppression with its observed usage in a compilation.
///
/// Ported from TS `ProgramSuppression`. `used == false` means no diagnostic
/// matched so far; it does not prove the suppression is removable (a rule may
/// be disabled, a source unavailable, or compilation may have stopped early).
#[derive(Debug, Clone)]
pub struct ProgramSuppression {
    /// Parsed directive.
    pub directive: SuppressDirective,
    /// The syntax node carrying the directive.
    pub target: NodeId,
    /// Directive range in the source file.
    pub span: crate::ast::Span,
    /// Structural context, outermost first.
    pub scope: Vec<SuppressionScope>,
    /// Whether the directive matched a diagnostic so far in this compilation.
    pub used: bool,
}

/// Collect `#suppress` directives from a parsed file without binding or
/// compiling it.
///
/// Codes are returned as written (unresolved), duplicates are preserved, and
/// directives in injected library source are excluded. Directives in source
/// order; a directive reachable through several AST paths is returned once,
/// attached to the deepest node.
///
/// Ported from TS `collectSuppressions`.
pub fn collect_suppressions(result: &crate::parser::ParseResult) -> Vec<Suppression> {
    collect_from_script(&result.builder, result.root_id)
}

fn collect_from_script(ast: &AstBuilder, root_id: NodeId) -> Vec<Suppression> {
    let mut found: HashMap<NodeId, Suppression> = HashMap::new();
    let mut scope = Vec::new();
    walk_for_suppressions(ast, root_id, &mut scope, &mut found);
    let mut suppressions: Vec<Suppression> = found.into_values().collect();
    suppressions.sort_by_key(|s| s.directive.node);
    suppressions
}

/// Whether a node lives in injected library source.
fn is_library_node(ast: &AstBuilder, node_id: NodeId) -> bool {
    ast.node_span(node_id)
        .is_some_and(|span| span.start.line as usize <= ast.library_line_offset)
}

fn record_suppressions(
    ast: &AstBuilder,
    target: NodeId,
    scope: &[SuppressionScope],
    found: &mut HashMap<NodeId, Suppression>,
) {
    let Some(directives) = ast.directives_map.get(&target) else {
        return;
    };
    for &dir_id in directives {
        let Some(AstNode::DirectiveExpression(expr)) = ast.id_to_node(dir_id) else {
            continue;
        };
        if let Some(ParsedDirective::Suppress(directive)) = parse_directive(expr, ast) {
            // Overwriting on a deeper visit keeps the deepest attachment.
            found.insert(
                dir_id,
                Suppression {
                    directive,
                    target,
                    span: expr.span,
                    scope: scope.to_vec(),
                },
            );
        }
    }
}

/// Recursively walk the AST, recording directives with their structural scope.
fn walk_for_suppressions(
    ast: &AstBuilder,
    node_id: NodeId,
    scope: &mut Vec<SuppressionScope>,
    found: &mut HashMap<NodeId, Suppression>,
) {
    // Top-level library declarations are skipped as whole subtrees.
    if is_library_node(ast, node_id)
        && !matches!(ast.id_to_node(node_id), Some(AstNode::TypeSpecScript(_)))
    {
        return;
    }

    let mut pushed = 0usize;
    match ast.id_to_node(node_id) {
        Some(AstNode::NamespaceDeclaration(ns)) => {
            // `namespace A.B;` contributes one scope entry per dotted segment.
            for segment in crate::checker::Checker::get_identifier_name(ast, ns.name).split('.') {
                if !segment.is_empty() {
                    scope.push(SuppressionScope {
                        node: node_id,
                        name: Some(segment.to_string()),
                    });
                    pushed += 1;
                }
            }
        }
        Some(AstNode::ModelDeclaration(decl)) => {
            push_named_scope(ast, node_id, decl.name, decl.expression, scope, &mut pushed);
        }
        Some(AstNode::ModelExpression(_)) => {
            scope.push(SuppressionScope {
                node: node_id,
                name: None,
            });
            pushed += 1;
        }
        Some(AstNode::UnionDeclaration(decl)) => {
            push_named_scope(ast, node_id, decl.name, decl.expression, scope, &mut pushed);
        }
        Some(AstNode::EnumDeclaration(decl)) => {
            push_named_scope(ast, node_id, decl.name, decl.expression, scope, &mut pushed);
        }
        Some(AstNode::ScalarDeclaration(decl)) => {
            push_named_scope(ast, node_id, decl.name, decl.expression, scope, &mut pushed);
        }
        Some(AstNode::ModelProperty(prop)) => {
            push_name(ast, node_id, prop.name, scope, &mut pushed);
        }
        Some(AstNode::InterfaceDeclaration(decl)) => {
            push_name(ast, node_id, decl.name, scope, &mut pushed);
        }
        Some(AstNode::OperationDeclaration(decl)) => {
            push_name(ast, node_id, decl.name, scope, &mut pushed);
        }
        Some(AstNode::UnionVariant(variant)) => match variant.name {
            Some(name) => push_name(ast, node_id, name, scope, &mut pushed),
            None => {
                scope.push(SuppressionScope {
                    node: node_id,
                    name: None,
                });
                pushed += 1;
            }
        },
        Some(AstNode::EnumMember(member)) => {
            push_name(ast, node_id, member.name, scope, &mut pushed);
        }
        Some(AstNode::ScalarConstructor(ctor)) => {
            push_name(ast, node_id, ctor.name, scope, &mut pushed);
        }
        Some(AstNode::AliasStatement(decl)) => {
            push_name(ast, node_id, decl.name, scope, &mut pushed);
        }
        Some(AstNode::ConstStatement(decl)) => {
            push_name(ast, node_id, decl.name, scope, &mut pushed);
        }
        Some(AstNode::DecoratorDeclaration(decl)) => {
            push_name(ast, node_id, decl.name, scope, &mut pushed);
        }
        Some(AstNode::FunctionDeclaration(decl)) => {
            push_name(ast, node_id, decl.name, scope, &mut pushed);
        }
        Some(AstNode::FunctionParameter(param)) => {
            push_name(ast, node_id, param.name, scope, &mut pushed);
        }
        _ => {}
    }

    record_suppressions(ast, node_id, scope, found);

    match ast.id_to_node(node_id) {
        Some(AstNode::TypeSpecScript(script)) => {
            for &stmt in &script.statements {
                walk_for_suppressions(ast, stmt, scope, found);
            }
        }
        Some(AstNode::NamespaceDeclaration(ns)) => {
            for &stmt in &ns.statements {
                walk_for_suppressions(ast, stmt, scope, found);
            }
        }
        Some(AstNode::ModelDeclaration(decl)) => {
            for &prop in &decl.properties {
                walk_for_suppressions(ast, prop, scope, found);
            }
            walk_optional(ast, decl.extends, scope, found);
            walk_optional(ast, decl.is, scope, found);
            for &param in &decl.template_parameters {
                walk_for_suppressions(ast, param, scope, found);
            }
        }
        Some(AstNode::ModelExpression(expr)) => {
            for &prop in &expr.properties {
                walk_for_suppressions(ast, prop, scope, found);
            }
        }
        Some(AstNode::ModelProperty(prop)) => {
            walk_for_suppressions(ast, prop.value, scope, found);
            walk_optional(ast, prop.default, scope, found);
        }
        Some(AstNode::InterfaceDeclaration(decl)) => {
            for &op in &decl.operations {
                walk_for_suppressions(ast, op, scope, found);
            }
        }
        Some(AstNode::OperationDeclaration(decl)) => {
            walk_for_suppressions(ast, decl.signature, scope, found);
        }
        Some(AstNode::OperationSignatureDeclaration(sig)) => {
            // Operation parameters are an anonymous parameter model; upstream
            // visits them with the operation's scope, not an extra container.
            match ast.id_to_node(sig.parameters) {
                Some(AstNode::ModelExpression(params)) => {
                    for &prop in &params.properties {
                        walk_for_suppressions(ast, prop, scope, found);
                    }
                }
                _ => walk_for_suppressions(ast, sig.parameters, scope, found),
            }
            walk_for_suppressions(ast, sig.return_type, scope, found);
        }
        Some(AstNode::OperationSignatureReference(sig)) => {
            walk_for_suppressions(ast, sig.base_operation, scope, found);
        }
        Some(AstNode::UnionDeclaration(decl)) => {
            for &variant in &decl.variants {
                walk_for_suppressions(ast, variant, scope, found);
            }
            walk_optional(ast, decl.extends, scope, found);
        }
        Some(AstNode::UnionVariant(variant)) => {
            walk_for_suppressions(ast, variant.value, scope, found);
        }
        Some(AstNode::EnumDeclaration(decl)) => {
            for &member in &decl.members {
                walk_for_suppressions(ast, member, scope, found);
            }
        }
        Some(AstNode::EnumMember(member)) => {
            walk_optional(ast, member.value, scope, found);
        }
        Some(AstNode::ScalarDeclaration(decl)) => {
            for &ctor in &decl.constructors {
                walk_for_suppressions(ast, ctor, scope, found);
            }
            walk_optional(ast, decl.extends, scope, found);
        }
        Some(AstNode::ScalarConstructor(ctor)) => {
            for &param in &ctor.parameters {
                walk_for_suppressions(ast, param, scope, found);
            }
        }
        Some(AstNode::AliasStatement(decl)) => {
            walk_for_suppressions(ast, decl.value, scope, found);
        }
        Some(AstNode::ConstStatement(decl)) => {
            walk_for_suppressions(ast, decl.value, scope, found);
            walk_optional(ast, decl.type_annotation, scope, found);
        }
        Some(AstNode::DecoratorDeclaration(decl)) => {
            for &param in &decl.parameters {
                walk_for_suppressions(ast, param, scope, found);
            }
        }
        Some(AstNode::FunctionDeclaration(decl)) => {
            for &param in &decl.parameters {
                walk_for_suppressions(ast, param, scope, found);
            }
            walk_optional(ast, decl.return_type, scope, found);
        }
        Some(AstNode::FunctionParameter(param)) => {
            walk_optional(ast, param.type_annotation, scope, found);
        }
        Some(AstNode::TemplateParameterDeclaration(param)) => {
            walk_optional(ast, param.constraint, scope, found);
            walk_optional(ast, param.default, scope, found);
        }
        Some(AstNode::TemplateArgument(arg)) => {
            walk_for_suppressions(ast, arg.argument, scope, found);
        }
        Some(AstNode::TypeReference(reference)) => {
            for &arg in &reference.arguments {
                walk_for_suppressions(ast, arg, scope, found);
            }
        }
        Some(AstNode::CallExpression(call)) => {
            for &arg in &call.arguments {
                walk_for_suppressions(ast, arg, scope, found);
            }
        }
        Some(AstNode::ValueOfExpression(expr)) => {
            walk_for_suppressions(ast, expr.target, scope, found);
        }
        Some(AstNode::TypeOfExpression(expr)) => {
            walk_for_suppressions(ast, expr.target, scope, found);
        }
        Some(AstNode::UnionExpression(expr)) => {
            for &option in &expr.options {
                walk_for_suppressions(ast, option, scope, found);
            }
        }
        Some(AstNode::IntersectionExpression(expr)) => {
            for &option in &expr.options {
                walk_for_suppressions(ast, option, scope, found);
            }
        }
        Some(AstNode::TupleExpression(tuple)) => {
            for &value in &tuple.values {
                walk_for_suppressions(ast, value, scope, found);
            }
        }
        Some(AstNode::ArrayLiteral(array)) => {
            for &value in &array.values {
                walk_for_suppressions(ast, value, scope, found);
            }
        }
        Some(AstNode::ObjectLiteral(object)) => {
            for &prop in &object.properties {
                walk_for_suppressions(ast, prop, scope, found);
            }
        }
        Some(AstNode::ObjectLiteralProperty(prop)) => {
            walk_for_suppressions(ast, prop.value, scope, found);
        }
        Some(AstNode::ObjectLiteralSpreadProperty(spread)) => {
            walk_for_suppressions(ast, spread.target, scope, found);
        }
        Some(AstNode::ModelSpreadProperty(spread)) => {
            walk_for_suppressions(ast, spread.target, scope, found);
        }
        Some(AstNode::EnumSpreadMember(spread)) => {
            walk_for_suppressions(ast, spread.target, scope, found);
        }
        Some(AstNode::DecoratorExpression(dec)) => {
            for &arg in &dec.arguments {
                walk_for_suppressions(ast, arg, scope, found);
            }
        }
        _ => {}
    }

    scope.truncate(scope.len() - pushed);
}

fn push_name(
    ast: &AstBuilder,
    node_id: NodeId,
    name: NodeId,
    scope: &mut Vec<SuppressionScope>,
    pushed: &mut usize,
) {
    scope.push(SuppressionScope {
        node: node_id,
        name: Some(crate::checker::Checker::get_identifier_name(ast, name)),
    });
    *pushed += 1;
}

/// Push a scope entry for a declaration that may be an anonymous declaration
/// expression (`model { ... }`, which has no name).
fn push_named_scope(
    ast: &AstBuilder,
    node_id: NodeId,
    name: NodeId,
    is_expression: bool,
    scope: &mut Vec<SuppressionScope>,
    pushed: &mut usize,
) {
    let name = crate::checker::Checker::get_identifier_name(ast, name);
    let entry_name = if is_expression && name.is_empty() {
        None
    } else {
        Some(name)
    };
    scope.push(SuppressionScope {
        node: node_id,
        name: entry_name,
    });
    *pushed += 1;
}

fn walk_optional(
    ast: &AstBuilder,
    node: Option<NodeId>,
    scope: &mut Vec<SuppressionScope>,
    found: &mut HashMap<NodeId, Suppression>,
) {
    if let Some(node_id) = node {
        walk_for_suppressions(ast, node_id, scope, found);
    }
}

impl Checker {
    /// Get project suppressions and their observed usage.
    ///
    /// Returns a snapshot of usage at the time of the call, preserving written
    /// codes. Libraries' suppressions are excluded. Results may change as
    /// validators, linter rules, or emitters report more diagnostics; call
    /// after compilation to include all stages that ran.
    ///
    /// Ported from TS `getSuppressions`.
    pub fn get_suppressions(&self) -> Vec<ProgramSuppression> {
        let Some(ast) = &self.ast else {
            return Vec::new();
        };
        collect_from_script(ast, self.root_id)
            .into_iter()
            .map(|suppression| ProgramSuppression {
                used: self.suppression_tracker.is_used(suppression.directive.node),
                directive: suppression.directive,
                target: suppression.target,
                span: suppression.span,
                scope: suppression.scope,
            })
            .collect()
    }
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

    // ========================================================================
    // Suppression inspection APIs (microsoft/typespec#12095)
    // ========================================================================

    fn collect(source: &str) -> Vec<Suppression> {
        let result = parse(source);
        assert!(
            result.diagnostics.is_empty(),
            "unexpected parse diagnostics: {:?}",
            result.diagnostics
        );
        collect_suppressions(&result)
    }

    fn scope_names(scope: &[SuppressionScope]) -> Vec<Option<String>> {
        scope.iter().map(|s| s.name.clone()).collect()
    }

    fn names(values: &[&str]) -> Vec<Option<String>> {
        values.iter().map(|v| Some(v.to_string())).collect()
    }

    #[test]
    fn test_collect_suppressions_without_resolving_codes() {
        let suppressions = collect(
            r#"
            import "not-installed";
            #deprecated "old"
            #suppress "not-installed/rule" "first"
            #suppress "not-installed/rule" "first"
            #suppress deprecated
            model M {}
            "#,
        );
        let pairs: Vec<(String, String)> = suppressions
            .iter()
            .map(|s| (s.directive.code.clone(), s.directive.message.clone()))
            .collect();
        assert_eq!(
            pairs,
            vec![
                ("not-installed/rule".to_string(), "first".to_string()),
                ("not-installed/rule".to_string(), "first".to_string()),
                ("deprecated".to_string(), String::new()),
            ]
        );
        let nodes: std::collections::HashSet<NodeId> =
            suppressions.iter().map(|s| s.directive.node).collect();
        assert_eq!(nodes.len(), 3, "each directive has its own node");
    }

    #[test]
    fn test_collect_suppressions_skips_invalid_directives() {
        assert!(collect("").is_empty());
        let result = parse("#suppress\nmodel M {}\n#suppress 123\nmodel N {}");
        assert!(collect_suppressions(&result).is_empty());
        assert!(
            !result.diagnostics.is_empty(),
            "parse errors stay available to the caller"
        );
    }

    #[test]
    fn test_collect_suppressions_scopes_and_targets() {
        let suppressions = collect(
            r#"#suppress "before"
using Other;
#suppress "namespace"
namespace A.B;
model M {
  #suppress "property"
  p: string;
}
interface I {
  #suppress "operation"
  read(): M;
}"#,
        );
        let scopes: Vec<Vec<Option<String>>> =
            suppressions.iter().map(|s| scope_names(&s.scope)).collect();
        assert_eq!(
            scopes,
            vec![
                vec![],
                names(&["A", "B"]),
                names(&["A", "B", "M", "p"]),
                names(&["A", "B", "I", "read"]),
            ]
        );

        let result = parse(
            r#"#suppress "x"
model M {
  #suppress "property"
  p: string;
}"#,
        );
        let collected = collect_suppressions(&result);
        let property_suppression = collected
            .iter()
            .find(|s| s.directive.code == "property")
            .unwrap();
        assert!(matches!(
            result.builder.id_to_node(property_suppression.target),
            Some(AstNode::ModelProperty(_))
        ));
        assert_eq!(property_suppression.span.start.line, 3);
    }

    #[test]
    fn test_collect_suppressions_dedups_dotted_namespace() {
        let suppressions = collect(
            r#"
            namespace Outer {
              #suppress "rule" "first"
              #suppress "rule" "second"
              namespace A.B {}
            }
            "#,
        );
        let messages: Vec<&str> = suppressions
            .iter()
            .map(|s| s.directive.message.as_str())
            .collect();
        assert_eq!(messages, vec!["first", "second"]);
        for suppression in &suppressions {
            assert_eq!(scope_names(&suppression.scope), names(&["Outer", "A", "B"]));
            assert_eq!(
                suppression.target, suppression.scope[2].node,
                "keeps the deepest attachment"
            );
        }
    }

    #[test]
    fn test_collect_suppressions_anonymous_containers() {
        let operation = collect("op read(#suppress \"r\"\np: string): void;");
        assert_eq!(
            scope_names(&operation[0].scope),
            names(&["read", "p"]),
            "operation parameters do not add an extra container"
        );

        let model = collect("model M { p: { #suppress \"r\"\nq: string; }; }");
        assert_eq!(
            scope_names(&model[0].scope),
            vec![
                Some("M".to_string()),
                Some("p".to_string()),
                None,
                Some("q".to_string()),
            ]
        );

        let alias = collect("alias A = model { #suppress \"r\"\np: string; };");
        assert_eq!(
            scope_names(&alias[0].scope),
            vec![Some("A".to_string()), None, Some("p".to_string())]
        );
    }

    #[test]
    fn test_get_suppressions_reports_usage() {
        let checker = crate::checker::test_utils::check(
            r#"
            #deprecated "old"
            model Old {}
            model M {
              #suppress "deprecated" "intentional"
              p: Old;
            }
            namespace N {
              #suppress "deprecated" "not needed"
              model X {}
              #suppress "not-loaded/rule" "another configuration"
              model Y {}
            }
            "#,
        );
        let suppressions = checker.get_suppressions();
        let summary: Vec<(&str, bool)> = suppressions
            .iter()
            .map(|s| (s.directive.code.as_str(), s.used))
            .collect();
        assert_eq!(
            summary,
            vec![
                ("deprecated", true),
                ("deprecated", false),
                ("not-loaded/rule", false),
            ],
            "usage snapshot: {:?}",
            checker.diagnostics()
        );
        assert_eq!(scope_names(&suppressions[0].scope), names(&["M", "p"]));

        // Repeated queries return a fresh snapshot without mutating earlier results.
        let first = checker.get_suppressions();
        let second = checker.get_suppressions();
        assert_eq!(first.len(), second.len());
        assert_eq!(first[0].used, second[0].used);
    }
}
