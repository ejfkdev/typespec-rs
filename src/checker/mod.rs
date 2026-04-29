//! TypeSpec Checker - Core type checking for TypeSpec-Rust
//! Ported from TypeSpec compiler/src/core/checker.ts
//!
//! The checker is responsible for:
//! - Type checking and validation
//! - Type relationship verification (assignability)
//! - Constraint checking
//! - Error diagnostics
//! - Value vs Type distinction

// Module declarations
mod check_alias;
mod check_augment;
mod check_clone;
mod check_declarations;
mod check_decorators;
mod check_directives;
mod check_enum;
mod check_expressions;
mod check_helpers;
mod check_interface;
mod check_literals;
pub(crate) mod check_model;
mod check_namespace;
mod check_namespace_helpers;
mod check_operation;
mod check_program_flow;
mod check_reference_resolution;
mod check_scalar;
mod check_spread;
mod check_stdlib;
mod check_template;
mod check_template_instantiation;
mod check_union;
mod check_visibility;
pub mod decorator_utils;
pub mod type_relation;
pub mod type_utils;
pub mod types;

// Re-exports
pub use decorator_utils::*;
pub use type_relation::{Related, TypeRelationChecker, TypeRelationError, TypeRelationErrorCode};
pub use type_utils::*;
pub use types::*;

// ============================================================================
// Local macros for common patterns
// ============================================================================

/// Get the AstBuilder reference or return early with the given value.
/// Usage: `let ast = require_ast_or!(self, self.error_type);`
/// Or without explicit return (defaults to `()`): `let ast = require_ast_or!(self);`
#[macro_export]
#[doc(hidden)]
macro_rules! require_ast_or {
    // Form without explicit return (defaults to ())
    ($self:expr) => {{
        match $self.require_ast() {
            Some(ast) => ast,
            None => return,
        }
    }};
    // Form with explicit return value
    ($self:expr, $ret:expr) => {{
        match $self.require_ast() {
            Some(ast) => ast,
            None => return $ret,
        }
    }};
}

/// Get the AstBuilder reference and extract a specific AST node variant.
/// Returns `(ast, node)` tuple. Use `let (_, node) = ...` if ast isn't needed.
/// Usage: `let (ast, node) = require_ast_node!(self, node_id, ModelStatement, self.error_type);`
/// Or without explicit return (defaults to `()`): `let (ast, node) = require_ast_node!(self, node_id, ConstStatement);`
#[macro_export]
#[doc(hidden)]
macro_rules! require_ast_node {
    // Form without explicit return value (defaults to ())
    ($self:expr, $node_id:expr, $variant:ident) => {{
        let ast = match $self.require_ast() {
            Some(ast) => ast,
            None => return,
        };
        let node = match ast.id_to_node($node_id) {
            Some($crate::parser::AstNode::$variant(decl)) => decl.clone(),
            _ => return,
        };
        (ast, node)
    }};
    // Form with explicit return value
    ($self:expr, $node_id:expr, $variant:ident, $ret:expr) => {{
        let ast = match $self.require_ast() {
            Some(ast) => ast,
            None => return $ret,
        };
        let node = match ast.id_to_node($node_id) {
            Some($crate::parser::AstNode::$variant(decl)) => decl.clone(),
            _ => return $ret,
        };
        (ast, node)
    }};
}

// ============================================================================
// Checker Imports
// ============================================================================

// Re-export macros for sub-modules (use super::*; will include these)
pub(crate) use crate::require_ast_node;
pub(crate) use crate::require_ast_or;

use crate::ast::node::NodeId;
use crate::ast::types::SyntaxKind;
use crate::diagnostics::Diagnostic;
use crate::modifiers::{self, ModifierFlags};
use crate::parser::{AstBuilder, AstNode};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

// ============================================================================
// Check Context and Flags
// ============================================================================

/// Checker flags for controlling checking behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckFlags {
    None = 0,
    /// Currently checking within an uninstantiated template declaration
    InTemplateDeclaration = 1 << 0,
}

/// Re-export TypeMapper from types module
pub use crate::checker::types::TypeMapper;

/// Check context for tracking type mapping during checking
#[derive(Debug, Clone)]
pub struct CheckContext {
    /// The type mapper associated with this context, if any
    pub mapper: Option<TypeMapper>,
    /// The flags enabled in this context
    pub flags: CheckFlags,
}

impl CheckContext {
    /// Create a default check context
    pub fn new() -> Self {
        Self {
            mapper: None,
            flags: CheckFlags::None,
        }
    }

    /// Create a check context with a mapper
    pub fn with_mapper(mapper: Option<TypeMapper>) -> Self {
        Self {
            mapper,
            flags: CheckFlags::None,
        }
    }

    /// Create a new context with the given flags added
    pub fn with_flags(&self, flags: CheckFlags) -> Self {
        Self {
            mapper: self.mapper.clone(),
            flags,
        }
    }

    /// Check if ALL of the given flags are enabled
    pub fn has_flags(&self, flags: CheckFlags) -> bool {
        (self.flags as u32 & flags as u32) == flags as u32
    }
}

impl Default for CheckContext {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Symbol Links
// ============================================================================

/// Symbol links for tracking type information per symbol
#[derive(Debug, Clone, Default)]
pub struct SymbolLinks {
    /// Declared type for the symbol
    pub declared_type: Option<TypeId>,
    /// Type for the symbol
    pub type_id: Option<TypeId>,
    /// Instantiation map for template types
    /// Key: vector of TypeIds (template arguments), Value: instantiated TypeId
    pub instantiations: Option<HashMap<Vec<TypeId>, TypeId>>,
    /// Whether this node resolved to a template instantiation (e.g., `Foo<string>`)
    /// Ported from TS NodeLinks.isTemplateInstantiation
    pub is_template_instantiation: bool,
}

// ============================================================================
// Checker
// ============================================================================

/// The main checker struct
pub struct Checker {
    // ---- Type/Value stores ----
    /// Type store for all created types
    pub type_store: TypeStore,
    /// Value store for all created values
    pub value_store: ValueStore,

    // ---- Parse result ----
    /// AST builder from parser
    ast: Option<Rc<AstBuilder>>,
    /// Root node ID from parse result
    pub root_id: NodeId,

    // ---- Well-known types ----
    /// ErrorType intrinsic TypeId
    pub error_type: TypeId,
    /// Void intrinsic TypeId
    pub void_type: TypeId,
    /// Never intrinsic TypeId
    pub never_type: TypeId,
    /// Null intrinsic TypeId
    pub null_type: TypeId,
    /// Unknown intrinsic TypeId
    pub unknown_type: TypeId,

    // ---- Namespace tracking ----
    /// Global namespace TypeId
    pub global_namespace_type: Option<TypeId>,
    /// Current namespace stack during checking
    pub current_namespace: Option<TypeId>,
    /// TypeSpec namespace TypeId (the "TypeSpec" namespace containing stdlib types)
    pub typespec_namespace_id: Option<TypeId>,

    // ---- Declaration maps ----
    /// Map from name to TypeId for all declared types
    pub declared_types: HashMap<String, TypeId>,
    /// Map from name to ValueId for all declared values (const declarations)
    pub declared_values: HashMap<String, ValueId>,

    // ---- Node-to-type/value mapping ----
    /// Map from AST NodeId to TypeId
    pub node_type_map: HashMap<NodeId, TypeId>,
    /// Map from AST NodeId to ValueId
    pub node_value_map: HashMap<NodeId, ValueId>,

    // ---- Symbol links ----
    /// Symbol links per node
    pub symbol_links: HashMap<NodeId, SymbolLinks>,

    // ---- Standard types ----
    /// Standard TypeSpec scalar types (string, int32, etc.)
    pub std_types: HashMap<String, TypeId>,

    // ---- Diagnostics ----
    /// Collected diagnostics
    diagnostics_list: Vec<Diagnostic>,

    // ---- Circular reference detection ----
    /// Names currently being checked for circular base type references
    pub pending_base_type_names: HashSet<String>,
    /// Nodes currently being type-checked
    pub pending_type_checks: HashSet<NodeId>,
    /// Names currently being type-checked (for circular alias/type detection by name)
    pub pending_type_names: HashSet<String>,
    /// Names of operations currently being checked for circular `is` references
    pub pending_op_signature_names: HashSet<String>,
    /// Template parameter names currently being checked (for circular constraint detection)
    pub pending_template_constraint_names: HashSet<String>,
    /// Recursion depth counter for safety
    pub check_depth: u32,

    // ---- Type relation ----
    /// Type relation checker
    pub type_relation: TypeRelationChecker,

    // ---- Deprecation tracking ----
    /// Deprecation tracker
    pub deprecation_tracker: crate::deprecation::DeprecationTracker,

    /// Map from declaration NodeId to diagnostic codes suppressed by #suppress directives
    pub suppressed_diagnostics: HashMap<NodeId, Vec<String>>,

    /// Set of NodeIds for which directives have already been processed
    pub directives_processed: HashSet<NodeId>,

    // ---- Value exact type tracking ----
    /// Map from ValueId to exact TypeId (literal type) for values
    pub value_exact_types: HashMap<ValueId, TypeId>,

    // ---- Internal visibility tracking ----
    /// Set of TypeIds that are declared with the `internal` modifier
    pub internal_declarations: HashSet<TypeId>,

    // ---- Template parameter scope ----
    /// Stack of template parameter scopes (each scope maps name to NodeId)
    pub template_param_scope: Vec<HashMap<String, NodeId>>,

    // ---- Unused using tracking ----
    /// List of (NodeId, namespace_name) for all using declarations
    pub using_declarations: Vec<(NodeId, String)>,
    /// Set of namespace names that have been used (referenced during type resolution)
    pub used_using_names: HashSet<String>,

    // ---- Literal type interning ----
    /// Cache for string literal types (value -> TypeId) to ensure same-value literals share TypeId
    pub string_literal_cache: HashMap<String, TypeId>,
    /// Cache for numeric literal types (value_as_string -> TypeId) to ensure same-value literals share TypeId
    pub numeric_literal_cache: HashMap<String, TypeId>,
    /// Cache for boolean literal types (value -> TypeId) to ensure same-value booleans share TypeId
    pub boolean_literal_cache: HashMap<bool, TypeId>,

    // ---- Spread property tracking ----
    /// Map from parent model TypeId to list of child model TypeIds whose spreads are pending
    pub pending_spreads: HashMap<TypeId, Vec<TypeId>>,
    /// Map from source model TypeId to list of target model TypeIds that spread from it
    pub spread_sources: HashMap<TypeId, Vec<TypeId>>,

    // ---- Const circular reference detection ----
    /// Set of NodeIds for const statements currently being checked (for circular detection)
    pub pending_const_checks: HashSet<NodeId>,

    // ---- State accessors for decorator state ----
    /// State maps for intrinsic decorator data (@minValue, @maxValue, etc.)
    pub state_accessors: crate::state_accessors::StateAccessors,
}

impl Checker {
    /// Create a new checker with empty stores, initializing well-known intrinsic types.
    pub fn new() -> Self {
        let mut type_store = TypeStore::new();
        let value_store = ValueStore::new();

        // Create intrinsic types
        let error_type = type_store.add(Type::Intrinsic(IntrinsicType {
            id: type_store.next_type_id(),
            name: IntrinsicTypeName::ErrorType,
            node: None,
            is_finished: true,
        }));
        let void_type = type_store.add(Type::Intrinsic(IntrinsicType {
            id: type_store.next_type_id(),
            name: IntrinsicTypeName::Void,
            node: None,
            is_finished: true,
        }));
        let never_type = type_store.add(Type::Intrinsic(IntrinsicType {
            id: type_store.next_type_id(),
            name: IntrinsicTypeName::Never,
            node: None,
            is_finished: true,
        }));
        let null_type = type_store.add(Type::Intrinsic(IntrinsicType {
            id: type_store.next_type_id(),
            name: IntrinsicTypeName::Null,
            node: None,
            is_finished: true,
        }));
        let unknown_type = type_store.add(Type::Intrinsic(IntrinsicType {
            id: type_store.next_type_id(),
            name: IntrinsicTypeName::Unknown,
            node: None,
            is_finished: true,
        }));

        Self {
            type_store,
            value_store,
            ast: None,
            root_id: 0,
            error_type,
            void_type,
            never_type,
            null_type,
            unknown_type,
            global_namespace_type: None,
            current_namespace: None,
            typespec_namespace_id: None,
            declared_types: HashMap::new(),
            declared_values: HashMap::new(),
            node_type_map: HashMap::new(),
            node_value_map: HashMap::new(),
            symbol_links: HashMap::new(),
            std_types: HashMap::new(),
            diagnostics_list: Vec::new(),
            pending_base_type_names: HashSet::new(),
            pending_type_checks: HashSet::new(),
            pending_type_names: HashSet::new(),
            pending_op_signature_names: HashSet::new(),
            pending_template_constraint_names: HashSet::new(),
            check_depth: 0,
            type_relation: TypeRelationChecker::new(),
            deprecation_tracker: crate::deprecation::DeprecationTracker::new(),
            suppressed_diagnostics: HashMap::new(),
            directives_processed: HashSet::new(),
            value_exact_types: HashMap::new(),
            internal_declarations: HashSet::new(),
            template_param_scope: Vec::new(),
            using_declarations: Vec::new(),
            used_using_names: HashSet::new(),
            string_literal_cache: HashMap::new(),
            numeric_literal_cache: HashMap::new(),
            boolean_literal_cache: HashMap::new(),
            pending_spreads: HashMap::new(),
            spread_sources: HashMap::new(),
            pending_const_checks: HashSet::new(),
            state_accessors: crate::state_accessors::StateAccessors::new(),
        }
    }

    /// Set the parse result from the parser
    pub fn set_parse_result(&mut self, root_id: NodeId, builder: AstBuilder) {
        self.root_id = root_id;
        self.ast = Some(Rc::new(builder));
    }

    // ========================================================================
    // Type/Value store access
    // ========================================================================

    /// Get a type by TypeId
    pub fn get_type(&self, id: TypeId) -> Option<&Type> {
        self.type_store.get(id)
    }

    /// Get a mutable reference to a type by TypeId
    pub fn get_type_mut(&mut self, id: TypeId) -> Option<&mut Type> {
        self.type_store.get_mut(id)
    }

    /// Get a value by ValueId
    pub fn get_value(&self, id: ValueId) -> Option<&Value> {
        self.value_store.get(id)
    }

    /// Get a mutable reference to a value by ValueId
    pub fn get_value_mut(&mut self, id: ValueId) -> Option<&mut Value> {
        self.value_store.get_mut(id)
    }

    /// Set the type of a value
    pub fn set_value_type(&mut self, value_id: ValueId, new_type: TypeId) {
        if let Some(v) = self.value_store.get_mut(value_id) {
            v.set_value_type(new_type);
        }
    }

    /// Allocate the next TypeId
    pub fn next_type_id(&self) -> TypeId {
        self.type_store.next_type_id()
    }

    /// Allocate the next ValueId
    pub fn next_value_id(&self) -> ValueId {
        self.value_store.next_value_id()
    }

    /// Create a new type and add it to the store
    pub fn create_type(&mut self, t: Type) -> TypeId {
        self.type_store.add(t)
    }

    /// Create a new value and add it to the store
    pub fn create_value(&mut self, v: Value) -> ValueId {
        self.value_store.add(v)
    }

    // ========================================================================
    // Diagnostics
    // ========================================================================

    /// Add a diagnostic
    pub fn add_diagnostic(&mut self, diag: Diagnostic) {
        self.diagnostics_list.push(diag);
    }

    /// Add an error diagnostic (convenience shorthand)
    pub(crate) fn error(&mut self, code: &str, msg: &str) {
        self.add_diagnostic(Diagnostic::error(code, msg));
    }

    /// Add a warning diagnostic (convenience shorthand)
    pub(crate) fn warning(&mut self, code: &str, msg: &str) {
        self.add_diagnostic(Diagnostic::warning(code, msg));
    }

    /// Get a human-readable string for a type (convenience shorthand)
    pub(crate) fn type_to_string(&self, type_id: TypeId) -> String {
        type_utils::type_to_string(&self.type_store, type_id)
    }

    /// Report an "unassignable" or "invalid-argument" type error
    pub(crate) fn error_unassignable(&mut self, code: &str, source: TypeId, target: TypeId) {
        self.error(
            code,
            &format!(
                "Type '{}' is not assignable to type '{}'",
                self.type_to_string(source),
                self.type_to_string(target),
            ),
        );
    }

    /// Get all diagnostics
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics_list
    }

    // ========================================================================
    // Literal type creation
    // ========================================================================

    /// Create an anonymous union type from a list of option types.
    /// Each option becomes a UnionVariant. The union is marked as `expression: true`
    /// (anonymous) and is immediately finished.
    /// Ported from TS checker createUnion().
    pub fn create_union(&mut self, options: Vec<TypeId>) -> TypeId {
        // Empty union → create a Union type with expression=true (represents never)
        if options.is_empty() {
            let mut u = UnionType::new(0, String::new(), None, None, true);
            u.is_finished = true;
            return self.create_type(Type::Union(u));
        }

        for &opt in &options {
            if opt == self.error_type {
                return self.error_type;
            }
        }

        // Create variant types first (with placeholder union=0)
        let mut variant_ids = Vec::new();
        let mut variant_names = Vec::new();
        for (i, &opt) in options.iter().enumerate() {
            let name = format!("v{}", i);
            let variant_id = self.create_type(Type::UnionVariant(UnionVariantType {
                id: 0, // will be corrected by create_type
                name: name.clone(),
                node: None,
                r#type: opt,
                union: None, // will be backfilled
                decorators: Vec::new(),
                is_finished: true,
            }));
            variant_ids.push(variant_id);
            variant_names.push(name);
        }

        // Create the union type
        let mut variant_map = HashMap::new();
        for (i, &vid) in variant_ids.iter().enumerate() {
            variant_map.insert(variant_names[i].clone(), vid);
        }
        let union_id = {
            let mut u = UnionType::new(0, String::new(), None, None, true);
            u.variants = variant_map;
            u.variant_names = variant_names;
            u.is_finished = true;
            self.create_type(Type::Union(u))
        };

        // Backfill union field on variants
        for &vid in &variant_ids {
            if let Some(Type::UnionVariant(v)) = self.get_type_mut(vid) {
                v.union = Some(union_id);
            }
        }

        union_id
    }

    /// Create a string literal type with caching.
    /// Same values always return the same TypeId (interning).
    /// Ported from TS checker createLiteralType().
    pub fn create_literal_type_string(&mut self, value: String) -> TypeId {
        if let Some(&cached) = self.string_literal_cache.get(&value) {
            return cached;
        }
        let type_id = self.create_type(Type::String(StringType {
            id: self.next_type_id(),
            value: value.clone(),
            node: None,
            is_finished: true,
        }));
        self.string_literal_cache.insert(value, type_id);
        type_id
    }

    /// Create a numeric literal type with caching.
    /// Ported from TS checker createLiteralType(value: number).
    pub fn create_literal_type_number(&mut self, value: f64, value_as_string: String) -> TypeId {
        if let Some(&cached) = self.numeric_literal_cache.get(&value_as_string) {
            return cached;
        }
        let type_id = self.create_type(Type::Number(NumericType {
            id: self.next_type_id(),
            value,
            value_as_string: value_as_string.clone(),
            node: None,
            is_finished: true,
        }));
        self.numeric_literal_cache.insert(value_as_string, type_id);
        type_id
    }

    /// Create a boolean literal type with caching.
    /// Ported from TS checker createLiteralType(value: boolean).
    pub fn create_literal_type_boolean(&mut self, value: bool) -> TypeId {
        if let Some(&cached) = self.boolean_literal_cache.get(&value) {
            return cached;
        }
        let type_id = self.create_type(Type::Boolean(BooleanType {
            id: self.next_type_id(),
            value,
            node: None,
            is_finished: true,
        }));
        self.boolean_literal_cache.insert(value, type_id);
        type_id
    }

    // ========================================================================
    // AST access
    // ========================================================================

    /// Get a reference to the AST builder, returning None if not set.
    /// This replaces the repeated `let ast = match &self.ast { Some(a) => a.clone(), None => return }` pattern.
    pub(crate) fn require_ast(&self) -> Option<Rc<AstBuilder>> {
        self.ast.clone()
    }

    // ========================================================================
    // Node dispatch
    // ========================================================================

    /// Check a node and return its TypeId.
    /// Wraps check_node_impl with depth limiting and directive processing.
    pub fn check_node(&mut self, ctx: &CheckContext, node_id: NodeId) -> TypeId {
        // Safety: prevent infinite recursion with a depth limit
        self.check_depth += 1;
        if self.check_depth > 200 {
            self.check_depth -= 1;
            return self.error_type;
        }

        let result = self.check_node_impl(ctx, node_id);

        // Process directives attached to this node (e.g., #deprecated, #suppress)
        // Skip if already processed (some check functions like check_model process
        // directives early to set up deprecation context before checking children)
        if result != self.error_type && !self.directives_processed.contains(&node_id) {
            self.process_directives(node_id, result);
            self.directives_processed.insert(node_id);
        }

        self.check_depth -= 1;

        // Record the type in node_type_map
        self.node_type_map.insert(node_id, result);
        result
    }

    /// Check modifiers on a declaration node and emit appropriate diagnostics.
    /// Ported from TS modifiers.ts checkModifiers()
    pub(crate) fn check_modifiers_and_report(
        &mut self,
        node_id: NodeId,
        kind: SyntaxKind,
        modifiers: &[NodeId],
    ) {
        let ast = match &self.ast {
            Some(ast) => ast.clone(),
            None => return,
        };

        // Compute modifier flags from the modifier nodes
        let mut modifier_flags = ModifierFlags::None;
        for &mod_id in modifiers {
            if let Some(AstNode::Modifier(m)) = ast.id_to_node(mod_id) {
                modifier_flags = modifier_flags | modifiers::modifier_to_flag(m.kind);
            }
        }

        // Per TS: emit experimental-feature warning for any use of 'internal' modifier
        if modifier_flags.contains(ModifierFlags::Internal) {
            self.warning(
                "experimental-feature",
                "The 'internal' modifier is an experimental feature.",
            );
            // Track this declaration as internal for visibility checking
            if let Some(&type_id) = self.node_type_map.get(&node_id) {
                self.internal_declarations.insert(type_id);
            }
        }

        // Check modifier validity using the shared check_modifiers function
        let result = modifiers::check_modifiers(modifier_flags, kind);

        // Emit invalid-modifier diagnostics for disallowed modifiers
        for invalid in &result.invalid_modifiers {
            self.error(
                "invalid-modifier",
                &format!(
                    "Modifier '{}' is not allowed on {}.",
                    invalid,
                    modifiers::get_declaration_kind_text(kind)
                ),
            );
        }

        // Emit invalid-modifier diagnostics for missing required modifiers
        for missing in &result.missing_modifiers {
            self.error(
                "invalid-modifier",
                &format!(
                    "Modifier '{}' is required on {}.",
                    missing,
                    modifiers::get_declaration_kind_text(kind)
                ),
            );
        }
    }

    pub(crate) fn check_node_impl(&mut self, ctx: &CheckContext, node_id: NodeId) -> TypeId {
        let ast = require_ast_or!(self, self.error_type);

        let node = match ast.id_to_node(node_id) {
            Some(n) => n.clone(),
            None => return self.error_type,
        };

        match &node {
            AstNode::ModelDeclaration(decl) => {
                self.check_modifiers_and_report(
                    node_id,
                    SyntaxKind::ModelStatement,
                    &decl.modifiers,
                );
                self.check_model(ctx, node_id)
            }
            AstNode::ModelExpression(_) => self.check_model(ctx, node_id),
            AstNode::ScalarDeclaration(decl) => {
                self.check_modifiers_and_report(
                    node_id,
                    SyntaxKind::ScalarStatement,
                    &decl.modifiers,
                );
                self.check_scalar(ctx, node_id)
            }
            AstNode::InterfaceDeclaration(decl) => {
                self.check_modifiers_and_report(
                    node_id,
                    SyntaxKind::InterfaceStatement,
                    &decl.modifiers,
                );
                self.check_interface(ctx, node_id)
            }
            AstNode::EnumDeclaration(decl) => {
                self.check_modifiers_and_report(
                    node_id,
                    SyntaxKind::EnumStatement,
                    &decl.modifiers,
                );
                self.check_enum(ctx, node_id)
            }
            AstNode::UnionDeclaration(decl) => {
                self.check_modifiers_and_report(
                    node_id,
                    SyntaxKind::UnionStatement,
                    &decl.modifiers,
                );
                self.check_union(ctx, node_id)
            }
            AstNode::NamespaceDeclaration(decl) => {
                self.check_modifiers_and_report(
                    node_id,
                    SyntaxKind::NamespaceStatement,
                    &decl.modifiers,
                );
                self.check_namespace(ctx, node_id)
            }
            AstNode::OperationDeclaration(decl) => {
                self.check_modifiers_and_report(
                    node_id,
                    SyntaxKind::OperationStatement,
                    &decl.modifiers,
                );
                self.check_operation(ctx, node_id)
            }
            AstNode::AliasStatement(decl) => {
                self.check_modifiers_and_report(
                    node_id,
                    SyntaxKind::AliasStatement,
                    &decl.modifiers,
                );
                self.check_alias(ctx, node_id)
            }
            AstNode::ConstStatement(decl) => {
                self.check_modifiers_and_report(
                    node_id,
                    SyntaxKind::ConstStatement,
                    &decl.modifiers,
                );
                self.check_const(ctx, node_id);
                self.error_type // const returns a value, not a type
            }
            AstNode::DecoratorDeclaration(decl) => {
                self.check_modifiers_and_report(
                    node_id,
                    SyntaxKind::DecoratorDeclarationStatement,
                    &decl.modifiers,
                );
                self.check_decorator_declaration(ctx, node_id)
            }
            AstNode::UsingDeclaration(_) => {
                self.check_using(node_id);
                self.void_type
            }
            AstNode::ImportStatement(decl) => {
                // Report import-not-found for any import (we don't have multi-file support yet)
                let path_str = match ast.id_to_node(decl.path) {
                    Some(AstNode::StringLiteral(sl)) => sl.value.clone(),
                    _ => String::new(),
                };
                if !path_str.is_empty() {
                    self.error(
                        "import-not-found",
                        &format!("Cannot find import '{}'", path_str),
                    );
                }
                self.void_type
            }
            AstNode::AugmentDecoratorStatement(_) => {
                self.check_augment_decorator(ctx, node_id);
                self.void_type
            }
            AstNode::FunctionDeclaration(decl) => {
                self.check_modifiers_and_report(
                    node_id,
                    SyntaxKind::FunctionDeclarationStatement,
                    &decl.modifiers,
                );
                self.check_function_declaration(ctx, node_id)
            }
            // Expression types
            AstNode::StringLiteral(_) => self.check_string_literal(node_id),
            AstNode::NumericLiteral(_) => self.check_numeric_literal(node_id),
            AstNode::BooleanLiteral(_) => self.check_boolean_literal(node_id),
            AstNode::TypeReference(_) => self.check_type_reference(ctx, node_id),
            AstNode::ArrayExpression(_) => self.check_array_expression(ctx, node_id),
            AstNode::TupleExpression(_) => self.check_tuple_expression(ctx, node_id),
            AstNode::UnionExpression(_) => self.check_union_expression(ctx, node_id),
            AstNode::IntersectionExpression(_) => self.check_intersection_expression(ctx, node_id),
            AstNode::VoidKeyword(_) => self.void_type,
            AstNode::NeverKeyword(_) => self.never_type,
            AstNode::UnknownKeyword(_) => self.unknown_type,
            AstNode::ValueOfExpression(_) => self.check_valueof(ctx, node_id),
            AstNode::TypeOfExpression(_) => self.check_typeof(ctx, node_id),
            AstNode::StringTemplateExpression(_) => self.check_string_template(node_id),
            AstNode::CallExpression(_) => self.check_call_expression(ctx, node_id),
            AstNode::ObjectLiteral(_) => self.check_object_literal(ctx, node_id),
            AstNode::ArrayLiteral(_) => self.check_array_literal(ctx, node_id),
            AstNode::MemberExpression(_) => self.check_member_expression(ctx, node_id),
            AstNode::Identifier(_) => self.check_identifier(ctx, node_id),
            AstNode::DecoratorExpression(_) => self.void_type,
            AstNode::DirectiveExpression(_) => {
                self.check_directive(node_id);
                self.void_type
            }
            AstNode::Doc(_) | AstNode::DocText(_) => self.void_type,
            AstNode::LineComment(_)
            | AstNode::BlockComment(_)
            | AstNode::EmptyStatement(_)
            | AstNode::StringTemplateSpan(_) => self.void_type,
            AstNode::TemplateArgument(tmpl_arg) => {
                // Unwrap template argument: check the inner value node
                self.check_node(ctx, tmpl_arg.argument)
            }
            AstNode::StringTemplateHead(_)
            | AstNode::StringTemplateMiddle(_)
            | AstNode::StringTemplateTail(_) => self.check_string_template_part(node_id),
            _ => self.error_type,
        }
    }

    // ========================================================================
    // Public API
    // ========================================================================

    /// Check a node and return an Entity (Type, Value, MixedConstraint, or Indeterminate).
    /// Ported from TS checker.ts checkNode() which returns Type | Value | Indeterminate.
    pub fn check_node_entity(&mut self, ctx: &CheckContext, node_id: NodeId) -> Entity {
        let type_id = self.check_node(ctx, node_id);
        Entity::Type(type_id)
    }

    /// Convert an Entity to a TypeId.
    /// For Type entities, returns the TypeId directly.
    /// For Value entities, returns the value's type.
    /// For Indeterminate entities, returns the inner TypeId.
    pub fn entity_to_type_id(&self, entity: &Entity) -> TypeId {
        match entity {
            Entity::Type(id) => *id,
            Entity::Value(id) => self
                .get_value(*id)
                .map(|v| v.value_type())
                .unwrap_or(self.error_type),
            Entity::Indeterminate(id) => *id,
            Entity::MixedConstraint(mc) => mc.type_constraint.unwrap_or(self.error_type),
        }
    }

    /// Check if source type is assignable to target type.
    /// Returns (is_assignable, errors) tuple.
    /// Ported from TS checker.isTypeAssignableTo().
    pub fn is_type_assignable_to(
        &mut self,
        source: TypeId,
        target: TypeId,
        _diagnostic_target: TypeId,
    ) -> (bool, Vec<TypeRelationError>) {
        let result = self
            .type_relation
            .is_related_with_store(&self.type_store, source, target);
        (result.is_true(), Vec::new())
    }

    /// Get the effective model type for a type.
    /// If the type is a named model, returns it directly.
    /// If the type is an anonymous model, checks if all its properties come
    /// from a single named source model and returns that source.
    /// Ported from TS checker.getEffectiveModelType().
    pub fn get_effective_model_type(&self, type_id: TypeId) -> TypeId {
        self.get_effective_model_type_with_filter(type_id, None)
    }

    /// Get the effective model type for a type with a property filter.
    /// Ported from TS checker.getEffectiveModelType().
    pub fn get_effective_model_type_with_filter(
        &self,
        type_id: TypeId,
        filter: Option<&dyn Fn(&Type) -> bool>,
    ) -> TypeId {
        match self.get_type(type_id) {
            Some(Type::Model(m)) => {
                // Named model: return itself (even with filter, per TS behavior)
                if !m.name.is_empty() {
                    return type_id;
                }

                // Check if filter would remove any properties
                if let Some(f) = filter {
                    let needs_filter = m.properties.values().any(|&prop_id| {
                        !f(self
                            .get_type(prop_id)
                            .unwrap_or(&Type::Intrinsic(IntrinsicType {
                                id: 0,
                                name: IntrinsicTypeName::ErrorType,
                                node: None,
                                is_finished: true,
                            })))
                    });
                    if needs_filter {
                        // With filter removing properties, we can't create a new model
                        // from &self — return the input model as-is
                        return type_id;
                    }
                }

                // Anonymous model: try to find a named source model
                self.resolve_effective_model(type_id)
            }
            Some(Type::Union(u)) => {
                let variant_ids: Vec<TypeId> = u
                    .variant_names
                    .iter()
                    .filter_map(|n| u.variants.get(n).copied())
                    .collect();
                if variant_ids.iter().all(|&v| {
                    matches!(
                        self.get_type(self.resolve_alias_chain(v)),
                        Some(Type::Model(_))
                    )
                }) {
                    type_id
                } else {
                    self.error_type
                }
            }
            _ => type_id,
        }
    }

    /// Resolve the effective model for an anonymous model by finding a named
    /// source model that has the same set of properties.
    /// Ported from TS getEffectiveModelType (anonymous model resolution).
    fn resolve_effective_model(&self, type_id: TypeId) -> TypeId {
        let m = match self.get_type(type_id) {
            Some(Type::Model(m)) => m,
            _ => return type_id,
        };

        // Anonymous model with base model shouldn't happen per TS assertion
        // but handle gracefully
        if m.base_model.is_some() {
            return type_id;
        }

        // Empty model: return itself
        if m.properties.is_empty() {
            return type_id;
        }

        // Find candidate named models that could be the source of every property
        // TS NOTE: "We depend on the order of that spread and intersect source
        // properties here, which is that we see properties sourced from derived
        // types before properties sourced from their base types."
        // So we iterate property_names (insertion order) not HashMap values.
        let mut candidates: Option<HashSet<TypeId>> = None;
        for name in &m.property_names {
            let &prop_id = match m.properties.get(name) {
                Some(id) => id,
                None => continue,
            };
            let sources = self.get_named_source_models(prop_id);
            let Some(sources) = sources else {
                // Unsourced property: no possible match
                return type_id;
            };

            match &mut candidates {
                None => {
                    candidates = Some(sources);
                }
                Some(cands) => {
                    // TS: addDerivedModels(sources, candidates)
                    // Adds derived types of candidates into sources if their base is in sources
                    let mut sources_mut = sources;
                    Self::add_derived_models(&mut sources_mut, cands, self);
                    // Remove candidates not common to this property's sources
                    cands.retain(|c| sources_mut.contains(c));
                }
            }
        }

        // Search for a candidate whose property count matches
        if let Some(cands) = candidates {
            let prop_count = m.properties.len();
            for &candidate_id in &cands {
                if let Some(count) = self.count_properties_inherited(candidate_id, None) {
                    if prop_count == count {
                        return candidate_id;
                    }
                }
            }
        }

        type_id
    }

    /// Get the set of named source models for a property by following
    /// the source_property chain. Returns None if the property has no source
    /// (i.e., it's an original property, not copied from spread/intersection).
    /// Ported from TS getNamedSourceModels().
    fn get_named_source_models(&self, prop_id: TypeId) -> Option<HashSet<TypeId>> {
        let prop = match self.get_type(prop_id) {
            Some(Type::ModelProperty(p)) => p,
            _ => return None,
        };

        prop.source_property?;

        let mut set = HashSet::new();
        let mut current = Some(prop_id);
        while let Some(pid) = current {
            if let Some(Type::ModelProperty(p)) = self.get_type(pid) {
                if let Some(model_id) = p.model {
                    if let Some(Type::Model(m)) = self.get_type(model_id) {
                        if !m.name.is_empty() {
                            set.insert(model_id);
                        }
                    }
                }
                current = p.source_property;
            } else {
                break;
            }
        }

        if set.is_empty() { None } else { Some(set) }
    }

    /// Add derived types: for each element in `possibly_derived` that isn't
    /// already in `sources`, check if its base model chain includes any model
    /// in `sources`. If so, add the element to `sources`.
    /// Ported from TS addDerivedModels(sources, possiblyDerivedModels).
    fn add_derived_models(
        sources: &mut HashSet<TypeId>,
        possibly_derived: &HashSet<TypeId>,
        checker: &Checker,
    ) {
        for &element in possibly_derived {
            if !sources.contains(&element) {
                let mut current = if let Some(Type::Model(m)) = checker.get_type(element) {
                    m.base_model
                } else {
                    None
                };
                while let Some(tid) = current {
                    if sources.contains(&tid) {
                        sources.insert(element);
                        break;
                    }
                    if let Some(Type::Model(m)) = checker.get_type(tid) {
                        current = m.base_model;
                    } else {
                        break;
                    }
                }
            }
        }
    }

    /// Count all properties of a model including inherited (base model chain).
    /// Ported from TS countPropertiesInherited().
    fn count_properties_inherited(
        &self,
        type_id: TypeId,
        filter: Option<&dyn Fn(&Type) -> bool>,
    ) -> Option<usize> {
        let mut count = 0;
        let mut current: Option<TypeId> = Some(type_id);
        while let Some(mid) = current {
            if let Some(Type::Model(m)) = self.get_type(mid) {
                for &prop_id in m.properties.values() {
                    if let Some(f) = filter {
                        if let Some(prop_type) = self.get_type(prop_id) {
                            if f(prop_type) {
                                count += 1;
                            }
                        }
                    } else {
                        count += 1;
                    }
                }
                current = m.base_model;
            } else {
                return None;
            }
        }
        Some(count)
    }

    /// Resolve an alias chain: follow scalar base_scalar references until we reach
    /// a non-aliased type, detect a cycle, or encounter a Scalar→Scalar extends
    /// relationship (which should NOT be resolved through, as it represents
    /// "extends" not "alias").
    pub fn resolve_alias_chain(&self, type_id: TypeId) -> TypeId {
        let mut current = type_id;
        let mut seen = HashSet::new();
        loop {
            if !seen.insert(current) {
                return current;
            }
            match self.get_type(current) {
                Some(Type::Scalar(s)) if s.base_scalar.is_some() => {
                    let Some(base_id) = s.base_scalar else {
                        return current;
                    };
                    // If base is also a Scalar, this is an extends relationship
                    // (e.g., int32 extends int64), not an alias. Stop resolving.
                    // Aliases like `alias Foo = string | int32` have base_scalar
                    // pointing to non-Scalar types (Union, Model, etc.).
                    // Also stop if the alias name matches the base scalar name
                    // (which shouldn't happen but protects against edge cases).
                    if matches!(self.get_type(base_id), Some(Type::Scalar(base)) if base.name == s.name)
                    {
                        return current;
                    }
                    if matches!(self.get_type(base_id), Some(Type::Scalar(_))) {
                        // Scalar extends Scalar — stop here, don't follow extends chain
                        return current;
                    }
                    current = base_id;
                }
                _ => return current,
            }
        }
    }

    /// Check if a type is a template instance (has template_mapper set).
    /// Ported from TS checker.isTemplateInstance().
    pub fn is_template_instance(&self, type_id: TypeId) -> bool {
        match self.get_type(type_id) {
            Some(t) => type_utils::is_template_instance(t),
            None => false,
        }
    }

    /// Check for circular reference in type checking.
    /// Returns `Some(type_id)` if a circular reference is detected (caller should return early),
    /// or `None` if no circular reference was found.
    ///
    /// This replaces the common pattern at the start of check_* functions:
    /// ```ignore
    /// if self.pending_type_checks.contains(&node_id) {
    ///     if let Some(&type_id) = self.node_type_map.get(&node_id) {
    ///         return type_id;
    ///     }
    ///     return self.error_type;
    /// }
    /// ```
    pub(crate) fn check_circular_ref(&self, node_id: NodeId) -> Option<TypeId> {
        if self.pending_type_checks.contains(&node_id) {
            Some(
                self.node_type_map
                    .get(&node_id)
                    .copied()
                    .unwrap_or(self.error_type),
            )
        } else {
            None
        }
    }

    /// Finalize a type check by processing decorators, finishing template handling,
    /// and removing the node from the pending set.
    ///
    /// This replaces the common pattern at the end of check_* functions:
    /// ```ignore
    /// self.check_and_store_decorators(ctx, type_id, &node.decorators);
    /// self.finish_template_or_type(type_id, node_id, &node.template_parameters, &node.decorators, ctx.mapper.as_ref());
    /// self.pending_type_checks.remove(&node_id);
    /// ```
    pub(crate) fn finalize_type_check(
        &mut self,
        ctx: &CheckContext,
        type_id: TypeId,
        node_id: NodeId,
        template_params: &[NodeId],
        decorators: &[NodeId],
        mapper: Option<&TypeMapper>,
    ) {
        self.check_and_store_decorators(ctx, type_id, decorators);
        self.finish_template_or_type(type_id, node_id, template_params, decorators, mapper);
        self.pending_type_checks.remove(&node_id);
    }

    /// Infer scalar type from constraint for a value.
    pub fn infer_scalars_from_constraints(
        &mut self,
        value_id: ValueId,
        constraint_type: TypeId,
    ) -> TypeId {
        let value_type = self
            .get_value(value_id)
            .map(|v| v.value_type())
            .unwrap_or(self.error_type);

        if value_type == self.error_type {
            return self.error_type;
        }

        if let Some(Type::Scalar(s)) = self.get_type(constraint_type) {
            if s.base_scalar.is_some() {
                return constraint_type;
            }
        }

        value_type
    }

    /// Register a type by name in declared_types, update symbol links,
    /// and set template_node on the type if a mapper is provided.
    /// Ported from TS checker.ts registerType().
    ///
    /// For template instantiations (mapper is Some), only update instantiations
    /// cache — do NOT overwrite declared_types, declared_type, or
    /// is_template_instantiation, as those belong to the template declaration.
    pub(crate) fn register_type(
        &mut self,
        node_id: NodeId,
        type_id: TypeId,
        name: &str,
        mapper: Option<&TypeMapper>,
    ) {
        // Update symbol links
        let links = self.symbol_links.entry(node_id).or_default();

        // If we have a mapper, this is a template instantiation.
        // Only update the instantiations cache; don't overwrite declaration info.
        if let Some(mapper) = mapper {
            if !mapper.partial {
                // Track the instantiation for future reuse
                let instantiations = links.instantiations.get_or_insert_with(HashMap::new);
                instantiations.insert(mapper.args.clone(), type_id);
            }
            // Set type_id for the instantiation (used for cache lookups)
            links.type_id = Some(type_id);
            return;
        }

        // Template declaration (no mapper) — update full registration
        self.declared_types.insert(name.to_string(), type_id);
        links.declared_type = Some(type_id);
        links.type_id = Some(type_id);
    }

    /// Compute the template_node for a type declaration.
    /// If template_parameters is non-empty and there's no mapper (i.e., this is
    /// the template declaration, not an instantiation), set template_node to the
    /// declaration node so that template instances can reference it later.
    /// Ported from TS checker.ts computeTemplateNode().
    pub(crate) fn compute_template_node(
        &mut self,
        template_parameters: &[NodeId],
        mapper: Option<&TypeMapper>,
        node_id: NodeId,
    ) -> Option<NodeId> {
        if !template_parameters.is_empty() && mapper.is_none() {
            Some(node_id)
        } else {
            None
        }
    }
}

impl Default for Checker {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Standalone functions
// ============================================================================

/// Filter model properties, creating a new anonymous model if any properties are removed.
/// If no properties are filtered out, returns the original TypeId.
/// Ported from TS checker.ts filterModelProperties.
pub fn filter_model_properties(
    checker: &mut Checker,
    model_type_id: TypeId,
    filter: &dyn Fn(TypeId) -> bool,
) -> TypeId {
    let (props_to_keep, prop_names_to_keep) = {
        let mut keep_props: Vec<(String, TypeId)> = Vec::new();
        let mut keep_names: Vec<String> = Vec::new();
        if let Some(Type::Model(m)) = checker.get_type(model_type_id) {
            for name in &m.property_names {
                if let Some(&prop_id) = m.properties.get(name)
                    && filter(prop_id)
                {
                    keep_props.push((name.clone(), prop_id));
                    keep_names.push(name.clone());
                }
            }
        }
        (keep_props, keep_names)
    };

    // If no properties were filtered out, return original
    let original_count = match checker.get_type(model_type_id) {
        Some(Type::Model(m)) => m.properties.len(),
        _ => 0,
    };
    if props_to_keep.len() == original_count {
        return model_type_id;
    }

    // Create a new anonymous model with the filtered properties
    {
        let mut m = ModelType::new(checker.next_type_id(), String::new(), None, None);
        m.properties = props_to_keep.into_iter().collect();
        m.property_names = prop_names_to_keep;
        m.source_model = Some(model_type_id);
        m.is_finished = true;
        checker.create_type(Type::Model(m))
    }
}

// ============================================================================
// Test modules
// ============================================================================

#[cfg(test)]
mod alias_tests;
#[cfg(test)]
mod augment_decorator_tests;
#[cfg(test)]
mod check_parse_errors_tests;
#[cfg(test)]
mod circular_ref_tests;
#[cfg(test)]
mod clone_type_tests;
#[cfg(test)]
mod decorators_tests;
#[cfg(test)]
mod deprecation_tests;
#[cfg(test)]
mod doc_comment_tests;
#[cfg(test)]
mod duplicate_ids_tests;
#[cfg(test)]
mod effective_type_tests;
#[cfg(test)]
mod enum_tests;
#[cfg(test)]
mod functions_tests;
#[cfg(test)]
mod global_ns_tests;
#[cfg(test)]
mod imports_tests;
#[cfg(test)]
mod internal_tests;
#[cfg(test)]
mod intersection_tests;
#[cfg(test)]
mod model_tests;
#[cfg(test)]
mod namespace_tests;
#[cfg(test)]
mod operation_tests;
#[cfg(test)]
mod references_tests;
#[cfg(test)]
mod relation_tests;
#[cfg(test)]
mod resolve_type_reference_tests;
#[cfg(test)]
mod scalar_tests;
#[cfg(test)]
mod spread_tests;
#[cfg(test)]
mod string_template_tests;
#[cfg(test)]
mod template_tests;
#[cfg(test)]
pub(crate) mod test_utils;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod type_utils_tests;
#[cfg(test)]
mod typeof_tests;
#[cfg(test)]
mod union_tests;
#[cfg(test)]
mod unused_template_parameter_tests;
#[cfg(test)]
mod unused_using_tests;
#[cfg(test)]
mod using_tests;
#[cfg(test)]
mod value_tests;
#[cfg(test)]
mod valueof_tests;
