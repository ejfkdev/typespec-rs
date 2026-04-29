//! Program - Central compilation state container
//!
//! Ported from TypeSpec compiler/src/core/program.ts
//!
//! The Program holds all state for a single compilation: source files,
//! the type checker, diagnostics, and compiler options.

use std::collections::HashMap;

use crate::ast::node::NodeId;
use crate::checker::types::TypeId;
use crate::diagnostics::Diagnostic;

/// Re-export CompilerOptions from diagnostics module (the canonical definition)
pub use crate::diagnostics::CompilerOptions;

/// Source file representation within a Program.
/// This is distinct from `source_file::SourceFile` which focuses on
/// text/position processing. This struct links a source file to its AST.
#[derive(Debug, Clone)]
pub struct ProgramSourceFile {
    /// File path
    pub path: String,
    /// Source content
    pub content: String,
    /// Root AST node ID for this file
    pub ast: NodeId,
}

/// State map key - uses a string key for storing arbitrary state
/// (TS uses Symbol keys; we use String for simplicity)
pub type StateMapKey = String;

/// Program - the central compilation state
pub struct Program {
    /// Compiler options used for this compilation
    pub options: CompilerOptions,
    /// Source files loaded into the program
    pub source_files: HashMap<String, ProgramSourceFile>,
    /// Project root directory
    pub project_root: Option<String>,
    /// Diagnostics collected during compilation
    diagnostics: Vec<Diagnostic>,
    /// Arbitrary state maps (for decorator metadata storage)
    /// TS uses Symbol keys; we use String keys
    state_maps: HashMap<StateMapKey, HashMap<TypeId, TypeId>>,
}

impl Program {
    /// Create a new empty program
    pub fn new() -> Self {
        Self {
            options: CompilerOptions::default(),
            source_files: HashMap::new(),
            project_root: None,
            diagnostics: Vec::new(),
            state_maps: HashMap::new(),
        }
    }

    /// Create a program with options
    pub fn with_options(options: CompilerOptions) -> Self {
        Self {
            options,
            ..Self::new()
        }
    }

    /// Check if the program has any error diagnostics
    pub fn has_error(&self) -> bool {
        self.diagnostics.iter().any(|d| d.is_error())
    }

    /// Report a diagnostic
    pub fn report_diagnostic(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    /// Report multiple diagnostics
    pub fn report_diagnostics(&mut self, diagnostics: Vec<Diagnostic>) {
        self.diagnostics.extend(diagnostics);
    }

    /// Get all diagnostics
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Get the number of diagnostics
    pub fn diagnostic_count(&self) -> usize {
        self.diagnostics.len()
    }

    /// Get or create a state map for the given key
    /// This is used by decorators to store metadata about types
    pub fn state_map(&mut self, key: &str) -> &mut HashMap<TypeId, TypeId> {
        self.state_maps.entry(key.to_string()).or_default()
    }

    /// Get a state map (read-only)
    pub fn get_state_map(&self, key: &str) -> Option<&HashMap<TypeId, TypeId>> {
        self.state_maps.get(key)
    }

    /// Add a source file to the program
    pub fn add_source_file(&mut self, file: ProgramSourceFile) {
        self.source_files.insert(file.path.clone(), file);
    }

    /// Get a source file by path
    pub fn get_source_file(&self, path: &str) -> Option<&ProgramSourceFile> {
        self.source_files.get(path)
    }
}

impl Default for Program {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_program_new() {
        let program = Program::new();
        assert!(program.source_files.is_empty());
        assert!(!program.has_error());
        assert_eq!(program.diagnostic_count(), 0);
    }

    #[test]
    fn test_program_with_options() {
        let options = CompilerOptions::default();
        let program = Program::with_options(options);
        assert!(!program.options.no_emit);
    }

    #[test]
    fn test_report_diagnostic() {
        let mut program = Program::new();
        let diag = Diagnostic::error("test", "test error");
        program.report_diagnostic(diag);
        assert!(program.has_error());
        assert_eq!(program.diagnostic_count(), 1);
    }

    #[test]
    fn test_report_diagnostics() {
        let mut program = Program::new();
        let diags = vec![
            Diagnostic::error("test", "error 1"),
            Diagnostic::warning("test", "warning 1"),
        ];
        program.report_diagnostics(diags);
        assert!(program.has_error());
        assert_eq!(program.diagnostic_count(), 2);
    }

    #[test]
    fn test_state_map() {
        let mut program = Program::new();
        let map = program.state_map("test_key");
        assert!(map.is_empty());
    }

    #[test]
    fn test_add_source_file() {
        let mut program = Program::new();
        let file = ProgramSourceFile {
            path: "test.tsp".to_string(),
            content: "model Foo {}".to_string(),
            ast: 1,
        };
        program.add_source_file(file);
        assert!(program.get_source_file("test.tsp").is_some());
        assert!(program.get_source_file("nonexistent.tsp").is_none());
    }

    #[test]
    fn test_compiler_options_default() {
        let opts = CompilerOptions::default();
        assert!(!opts.no_emit);
    }
}
