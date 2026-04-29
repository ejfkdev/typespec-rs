//! NameResolver - Resolves identifiers to symbols and manages symbol tables
//!
//! Ported from TypeSpec compiler/src/core/name-resolver.ts
//!
//! The name resolver is responsible for:
//! - Resolving identifiers to symbols
//! - Creating symbols for types discovered during resolution
//! - Managing symbol tables and namespace scopes
//! - Tracking augmented symbol tables for using statements
//!
//! Name resolution does not alter any AST nodes. Instead, symbols are stored
//! in augmented symbol tables or as merged symbols.

use crate::ast::node::NodeId;
use crate::program::Program;
use std::collections::{HashMap, HashSet};
use std::ops::BitOr;

/// Resolution result flags indicating the outcome of name resolution
/// Ported from TS types.ts ResolutionResultFlags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ResolutionResultFlags {
    /// No specific result
    #[default]
    None = 0,
    /// Resolution succeeded
    Resolved = 1 << 1,
    /// Resolution failed - symbol is unknown (late-bound)
    Unknown = 1 << 2,
    /// Resolution failed due to ambiguity
    Ambiguous = 1 << 3,
    /// Resolution failed - symbol not found
    NotFound = 1 << 4,
    /// Any failure mode
    ResolutionFailed = (1 << 2) | (1 << 3) | (1 << 4), // Unknown | Ambiguous | NotFound
}

impl ResolutionResultFlags {
    /// Check if resolution failed
    pub fn is_failed(&self) -> bool {
        matches!(
            self,
            Self::NotFound | Self::Unknown | Self::Ambiguous | Self::ResolutionFailed
        )
    }
}

/// Result of resolving a type or value reference
#[derive(Debug, Clone)]
pub struct ResolutionResult {
    /// The resolved symbol
    pub resolved_symbol: Option<Sym>,
    /// The final symbol after following aliases
    pub final_symbol: Option<Sym>,
    /// Resolution outcome flags
    pub resolution_result: ResolutionResultFlags,
    /// Whether this is a template instantiation
    pub is_template_instantiation: bool,
    /// Ambiguous symbols if resolution was ambiguous
    pub ambiguous_symbols: Vec<Sym>,
}

impl Default for ResolutionResult {
    fn default() -> Self {
        Self {
            resolved_symbol: None,
            final_symbol: None,
            resolution_result: ResolutionResultFlags::None,
            is_template_instantiation: false,
            ambiguous_symbols: Vec::new(),
        }
    }
}

impl ResolutionResult {
    /// Create a resolved result with the given symbol
    fn resolved(sym: Sym) -> Self {
        Self {
            resolved_symbol: Some(sym.clone()),
            final_symbol: Some(sym),
            resolution_result: ResolutionResultFlags::Resolved,
            is_template_instantiation: false,
            ambiguous_symbols: Vec::new(),
        }
    }

    /// Create a failed result
    fn failed(flags: ResolutionResultFlags) -> Self {
        Self {
            resolved_symbol: None,
            final_symbol: None,
            resolution_result: flags,
            is_template_instantiation: false,
            ambiguous_symbols: Vec::new(),
        }
    }

    /// Create an ambiguous result
    #[allow(dead_code)]
    fn ambiguous(symbols: Vec<Sym>) -> Self {
        Self {
            resolved_symbol: None,
            final_symbol: None,
            resolution_result: ResolutionResultFlags::Ambiguous,
            is_template_instantiation: false,
            ambiguous_symbols: symbols,
        }
    }
}

/// Symbol flags indicating the kind of symbol
/// Ported from TS types.ts SymbolFlags — bit values MUST match TS exactly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SymbolFlags(u32);

#[allow(non_upper_case_globals)]
impl SymbolFlags {
    /// No flags
    pub const None: Self = SymbolFlags(0);
    /// Symbol is a model
    pub const Model: Self = SymbolFlags(1 << 1);
    /// Symbol is a scalar
    pub const Scalar: Self = SymbolFlags(1 << 2);
    /// Symbol is an operation
    pub const Operation: Self = SymbolFlags(1 << 3);
    /// Symbol is an enum
    pub const Enum: Self = SymbolFlags(1 << 4);
    /// Symbol is an interface
    pub const Interface: Self = SymbolFlags(1 << 5);
    /// Symbol is a union
    pub const Union: Self = SymbolFlags(1 << 6);
    /// Symbol is an alias
    pub const Alias: Self = SymbolFlags(1 << 7);
    /// Symbol is a namespace
    pub const Namespace: Self = SymbolFlags(1 << 8);
    /// Symbol is a decorator
    pub const Decorator: Self = SymbolFlags(1 << 9);
    /// Symbol is a template parameter
    pub const TemplateParameter: Self = SymbolFlags(1 << 10);
    /// Symbol is a function
    pub const Function: Self = SymbolFlags(1 << 11);
    /// Symbol is a function parameter
    pub const FunctionParameter: Self = SymbolFlags(1 << 12);
    /// Symbol is a using statement
    pub const Using: Self = SymbolFlags(1 << 13);
    /// Symbol is a duplicate using
    pub const DuplicateUsing: Self = SymbolFlags(1 << 14);
    /// Symbol is a source file
    pub const SourceFile: Self = SymbolFlags(1 << 15);
    /// Symbol is a member (model property, enum member, etc.)
    pub const Member: Self = SymbolFlags(1 << 16);
    /// Symbol is a const declaration
    pub const Const: Self = SymbolFlags(1 << 17);
    /// Symbol is a declaration
    pub const Declaration: Self = SymbolFlags(1 << 20);
    /// Symbol is an implementation (vs declaration)
    pub const Implementation: Self = SymbolFlags(1 << 21);
    /// Symbol was late-bound
    pub const LateBound: Self = SymbolFlags(1 << 22);
    /// Symbol is internal (same-package only)
    pub const Internal: Self = SymbolFlags(1 << 23);

    // Composite flags (must match TS: ExportContainer = Namespace | SourceFile)
    /// Export container (namespace or source file)
    pub const ExportContainer: Self = SymbolFlags((1 << 8) | (1 << 15)); // Namespace | SourceFile
    /// Member container (model, enum, union, interface, scalar)
    pub const MemberContainer: Self =
        SymbolFlags((1 << 1) | (1 << 4) | (1 << 6) | (1 << 5) | (1 << 2)); // Model | Enum | Union | Interface | Scalar

    pub fn bits(&self) -> u32 {
        self.0
    }

    pub fn from_bits(bits: u32) -> Self {
        Self(bits)
    }

    pub fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    /// Check if this is a member container (models, interfaces, enums, etc.)
    /// Ported from TS: MemberContainer = Model | Enum | Union | Interface | Scalar
    pub fn is_member_container(&self) -> bool {
        (self.0 & Self::MemberContainer.0) != 0
    }

    /// Check if this is an export container (namespace, TypeSpecScript)
    /// Ported from TS: ExportContainer = Namespace | SourceFile
    pub fn is_export_container(&self) -> bool {
        (self.0 & Self::ExportContainer.0) != 0
    }
}

impl BitOr for SymbolFlags {
    type Output = Self;

    fn bitor(self, other: Self) -> Self::Output {
        Self(self.0 | other.0)
    }
}

/// A symbol representing a named declaration
#[derive(Debug, Clone)]
pub struct Sym {
    /// Unique identifier for this symbol
    pub id: Option<u32>,
    /// The name of this symbol
    pub name: String,
    /// Flags indicating what kind of symbol this is
    pub flags: SymbolFlags,
    /// The node where this symbol is declared
    pub node: Option<NodeId>,
    /// The declarations associated with this symbol
    pub declarations: Vec<NodeId>,
    /// For merged symbols, the merged exports table
    pub exports: Option<HashMap<String, Sym>>,
    /// Symbol table for members (for models, interfaces, etc.)
    pub members: Option<HashMap<String, Sym>>,
    /// Source symbol for using statements
    pub symbol_source: Option<Box<Sym>>,
    /// Metatype members for operations
    pub metatype_members: Option<HashMap<String, Sym>>,
    /// Parent namespace
    pub parent: Option<Box<Sym>>,
}

impl Sym {
    /// Create a new symbol
    pub fn new(name: &str, flags: SymbolFlags) -> Self {
        Self {
            id: None,
            name: name.to_string(),
            flags,
            node: None,
            declarations: Vec::new(),
            exports: None,
            members: None,
            symbol_source: None,
            metatype_members: None,
            parent: None,
        }
    }

    /// Check if this symbol has the given flags
    pub fn has_flag(&self, flag: SymbolFlags) -> bool {
        self.flags.contains(flag)
    }
}

/// Symbol table mapping names to symbols
pub type SymbolTable = HashMap<String, Sym>;

/// Node links - additional information associated with a node
#[derive(Debug, Clone, Default)]
pub struct NodeLinks {
    /// The resolved symbol for this node (if applicable)
    pub resolved_symbol: Option<Sym>,
    /// The resolution result flags
    pub resolution_result: ResolutionResultFlags,
    /// Whether this is a template instantiation
    pub is_template_instantiation: bool,
    /// For type references, the final symbol after alias resolution
    pub final_symbol: Option<Sym>,
}

impl NodeLinks {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Symbol links - additional information associated with a symbol
#[derive(Debug, Clone, Default)]
pub struct SymbolLinks {
    /// The declared type for this symbol
    pub declared_type: Option<NodeId>,
    /// The type for this symbol
    pub type_id: Option<NodeId>,
    /// Whether members have been bound
    pub members_bound: bool,
    /// Whether this symbol has unknown members
    pub has_unknown_members: bool,
    /// Alias resolution result
    pub alias_resolution_result: ResolutionResultFlags,
    /// The aliased symbol
    pub aliased_symbol: Option<Sym>,
    /// Whether alias resolution was a template instantiation
    pub alias_resolution_is_template: bool,
    /// Constraint resolution result
    pub constraint_resolution_result: ResolutionResultFlags,
    /// The constraint symbol
    pub constraint_symbol: Option<Sym>,
    /// Instantiation map for template types
    pub instantiations: Option<HashMap<Vec<NodeId>, NodeId>>,
}

impl SymbolLinks {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Options for resolving type references
#[derive(Debug, Clone, Default)]
pub struct ResolveTypeReferenceOptions {
    /// Whether to resolve decorators
    pub resolve_decorators: bool,
}

/// The NameResolver is responsible for resolving identifiers to symbols
///
/// Note: This module is scaffolded but not yet integrated into the compiler pipeline.
/// The current checker uses its own resolution logic. This will be integrated in a
/// future release when multi-file compilation and import resolution are implemented.
#[allow(dead_code)]
pub struct NameResolver {
    /// The program being resolved
    program: Program,
    /// Map of merged symbols (namespace merging)
    merged_symbols: HashMap<NodeId, Sym>,
    /// Augmented symbol tables (for using statements)
    augmented_symbol_tables: HashMap<NodeId, SymbolTable>,
    /// Node links for each node
    node_links: HashMap<NodeId, NodeLinks>,
    /// Symbol links for each symbol
    symbol_links: HashMap<NodeId, SymbolLinks>,
    /// Visited nodes during resolution
    visited_nodes: HashSet<NodeId>,
    /// Global namespace node
    global_namespace_node: NodeId,
    /// Global namespace symbol
    global_namespace_sym: Sym,
    /// Null type symbol
    null_sym: Sym,
    /// Used using symbols per file
    used_using_sym: HashMap<NodeId, HashSet<NodeId>>,
    /// Augment decorators for symbols
    augment_decorators_for_sym: HashMap<NodeId, Vec<NodeId>>,
    /// Next available node id
    next_node_id: NodeId,
    /// Next available symbol id
    next_symbol_id: NodeId,
}

impl NameResolver {
    /// Create a new name resolver for the given program
    pub fn new(program: Program) -> Self {
        let global_namespace_sym =
            Sym::new("global", SymbolFlags::Namespace | SymbolFlags::Declaration);
        let null_sym = Sym::new("null", SymbolFlags::None);

        Self {
            program,
            merged_symbols: HashMap::new(),
            augmented_symbol_tables: HashMap::new(),
            node_links: HashMap::new(),
            symbol_links: HashMap::new(),
            visited_nodes: HashSet::new(),
            global_namespace_node: 0,
            global_namespace_sym,
            null_sym,
            used_using_sym: HashMap::new(),
            augment_decorators_for_sym: HashMap::new(),
            next_node_id: 1,
            next_symbol_id: 1,
        }
    }

    /// Resolve all static symbol links in the program
    pub fn resolve_program(&mut self) {
        // Collect file IDs first to avoid borrow conflicts
        let file_ids: Vec<NodeId> = self
            .program
            .source_files
            .values()
            .map(|file| file.ast)
            .collect();

        // Simplified: just visit all nodes
        // A full implementation would merge namespaces, bind usings, etc.
        for file_id in &file_ids {
            self.bind_and_resolve_node(*file_id);
        }
    }

    /// Get the merged symbol or itself if not merged.
    /// Returns Sym directly (never fails — falls back to the input symbol).
    pub fn get_merged_symbol(&self, sym: &Sym) -> Sym {
        match sym.id {
            Some(id) => self
                .merged_symbols
                .get(&id)
                .cloned()
                .unwrap_or_else(|| sym.clone()),
            None => sym.clone(),
        }
    }

    /// Get augmented symbol table
    pub fn get_augmented_symbol_table(&mut self, table_id: NodeId) -> &mut SymbolTable {
        self.get_augmented_symbol_table_internal_by_id(table_id)
    }

    #[allow(dead_code)]
    fn get_augmented_symbol_table_internal(&mut self, table: &SymbolTable) -> &mut SymbolTable {
        // For simplicity, we use the table directly if not found
        // A more complete implementation would create augmented copies
        let table_id = self.find_or_create_table_id(table);
        self.get_augmented_symbol_table_internal_by_id(table_id)
    }

    #[allow(dead_code)]
    fn find_or_create_table_id(&mut self, _table: &SymbolTable) -> NodeId {
        // Simplified: return a new id each time
        // A more complete implementation would track existing tables
        let id = self.next_node_id;
        self.next_node_id += 1;
        self.augmented_symbol_tables.insert(id, SymbolTable::new());
        id
    }

    fn get_augmented_symbol_table_internal_by_id(&mut self, table_id: NodeId) -> &mut SymbolTable {
        self.augmented_symbol_tables.entry(table_id).or_default()
    }

    /// Get node links for the given node
    pub fn get_node_links(&mut self, node_id: NodeId) -> &mut NodeLinks {
        self.node_links.entry(node_id).or_default()
    }

    /// Get symbol links for the given symbol
    pub fn get_symbol_links(&mut self, sym: &Sym) -> &mut SymbolLinks {
        let id = sym.id.unwrap_or_else(|| {
            let new_id = self.next_symbol_id;
            self.next_symbol_id += 1;
            new_id
        });
        self.symbol_links.entry(id).or_default()
    }

    /// Return augment decorator nodes that are bound to this symbol
    pub fn get_augment_decorators_for_sym(&self, sym: &Sym) -> Vec<NodeId> {
        sym.id
            .and_then(|id| self.augment_decorators_for_sym.get(&id))
            .cloned()
            .unwrap_or_default()
    }

    /// Get unused using statements
    pub fn get_unused_usings(&self) -> Vec<NodeId> {
        Vec::new() // Simplified implementation
    }

    /// Resolve a type reference
    pub fn resolve_type_reference(
        &mut self,
        node_id: NodeId,
        options: ResolveTypeReferenceOptions,
    ) -> ResolutionResult {
        let links = self.get_node_links(node_id);
        if links.resolution_result != ResolutionResultFlags::None {
            return ResolutionResult {
                resolved_symbol: links.resolved_symbol.clone(),
                final_symbol: links.final_symbol.clone(),
                resolution_result: links.resolution_result,
                is_template_instantiation: links.is_template_instantiation,
                ambiguous_symbols: Vec::new(),
            };
        }

        let result = self.resolve_type_reference_worker(node_id, options);
        let resolved_sym = result.final_symbol.clone();

        // Handle alias unwrapping
        if let Some(ref sym) = resolved_sym {
            if sym.has_flag(SymbolFlags::Alias) {
                let alias_result = self.resolve_alias_internal(node_id);
                return ResolutionResult {
                    resolved_symbol: alias_result.resolved_symbol.or(result.resolved_symbol),
                    final_symbol: alias_result.final_symbol,
                    resolution_result: alias_result.resolution_result,
                    is_template_instantiation: result.is_template_instantiation
                        || alias_result.is_template_instantiation,
                    ambiguous_symbols: Vec::new(),
                };
            }

            if sym.has_flag(SymbolFlags::TemplateParameter) {
                // Handle template parameter with constraint
                let template_result = self.resolve_template_parameter_internal(node_id);
                if template_result.resolution_result != ResolutionResultFlags::None {
                    return template_result;
                }
            }
        }

        // Ensure the referenced node is bound
        if let Some(ref sym) = resolved_sym
            && sym.has_flag(SymbolFlags::Declaration)
            && !sym.has_flag(SymbolFlags::Namespace)
        {
            for decl in &sym.declarations {
                self.bind_and_resolve_node(*decl);
            }
        }

        result
    }

    fn resolve_type_reference_worker(
        &mut self,
        node_id: NodeId,
        options: ResolveTypeReferenceOptions,
    ) -> ResolutionResult {
        // TODO: When Program.nodes is populated, dispatch on node kind.
        // Currently, nodes is always empty, so we delegate to resolve_identifier.
        self.resolve_identifier(node_id, options)
    }

    #[allow(dead_code)]
    fn resolve_member_expression(
        &mut self,
        _node_id: NodeId,
        _options: ResolveTypeReferenceOptions,
    ) -> ResolutionResult {
        // Simplified: resolve the member expression
        ResolutionResult::failed(ResolutionResultFlags::Unknown)
    }

    fn resolve_identifier(
        &mut self,
        _node_id: NodeId,
        _options: ResolveTypeReferenceOptions,
    ) -> ResolutionResult {
        // Simplified: look up in global namespace
        if let Some(sym) = self
            .global_namespace_sym
            .exports
            .as_ref()
            .and_then(|e| e.get("global"))
        {
            return ResolutionResult::resolved(sym.clone());
        }
        ResolutionResult::failed(ResolutionResultFlags::Unknown)
    }

    fn resolve_alias_internal(&mut self, _node_id: NodeId) -> ResolutionResult {
        ResolutionResult::failed(ResolutionResultFlags::Unknown)
    }

    fn resolve_template_parameter_internal(&mut self, _node_id: NodeId) -> ResolutionResult {
        ResolutionResult::failed(ResolutionResultFlags::Unknown)
    }

    /// Resolve member expression for a symbol
    pub fn resolve_member_expression_for_sym(
        &mut self,
        _sym: &Sym,
        _node_id: NodeId,
        _options: ResolveTypeReferenceOptions,
    ) -> ResolutionResult {
        ResolutionResult::failed(ResolutionResultFlags::Unknown)
    }

    /// Get meta member by name
    pub fn resolve_meta_member_by_name(&mut self, _sym: &Sym, _name: &str) -> ResolutionResult {
        ResolutionResult::failed(ResolutionResultFlags::NotFound)
    }

    /// Merge symbol table from source into target
    #[allow(dead_code)]
    fn merge_symbol_table(&mut self, _target: &Sym, _source_id: NodeId) {
        // Simplified implementation
    }

    /// Set up using statements for a file
    #[allow(dead_code)]
    fn set_usings_for_file(&mut self, _file_id: NodeId) {
        // Simplified implementation
    }

    /// Bind and resolve a node
    fn bind_and_resolve_node(&mut self, node_id: NodeId) {
        if self.visited_nodes.contains(&node_id) {
            return;
        }
        self.visited_nodes.insert(node_id);

        // Simplified: just mark as visited
        // A complete implementation would traverse the AST and resolve references
    }

    /// Get the global namespace symbol
    pub fn global_namespace(&self) -> &Sym {
        &self.global_namespace_sym
    }

    /// Get the null type symbol
    pub fn null_symbol(&self) -> &Sym {
        &self.null_sym
    }
}

/// Create a new name resolver for the given program
pub fn create_resolver(program: Program) -> NameResolver {
    NameResolver::new(program)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolution_result() {
        let result = ResolutionResult::failed(ResolutionResultFlags::NotFound);
        assert_eq!(result.resolution_result, ResolutionResultFlags::NotFound);
        assert!(result.resolved_symbol.is_none());
    }

    #[test]
    fn test_symbol_flags() {
        let flags = SymbolFlags::Namespace | SymbolFlags::Declaration;
        assert!(flags.contains(SymbolFlags::Namespace));
        assert!(flags.contains(SymbolFlags::Declaration));
        assert!(!flags.contains(SymbolFlags::Model));
    }

    #[test]
    fn test_symbol_flags_is_member_container() {
        // TS: MemberContainer = Model | Enum | Union | Interface | Scalar
        let model = SymbolFlags::Model;
        assert!(model.is_member_container());
        assert!(!model.is_export_container());

        let ns = SymbolFlags::Namespace;
        assert!(ns.is_export_container());
        assert!(!ns.is_member_container());

        let source_file = SymbolFlags::SourceFile;
        assert!(source_file.is_export_container());
        assert!(!source_file.is_member_container());

        // Namespace | Declaration is NOT an export container by itself
        // (ExportContainer = Namespace | SourceFile, which does not include Declaration)
        let ns_decl = SymbolFlags::Namespace | SymbolFlags::Declaration;
        assert!(ns_decl.is_export_container()); // contains Namespace
    }

    #[test]
    fn test_resolution_result_flags_default() {
        let flags = ResolutionResultFlags::default();
        assert_eq!(flags, ResolutionResultFlags::None);
    }
}
