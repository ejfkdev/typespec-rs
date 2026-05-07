//! Source Loader for TypeSpec-Rust
//!
//! This module implements cross-file import functionality for TypeSpec.
//! It loads .tsp files, resolves import paths, and tracks seen source files
//! to prevent circular references.
//!
//! Ported from TypeSpec compiler/src/core/source-loader.ts

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::ast::NodeId;
use crate::diagnostics::{Diagnostic, DiagnosticCollector};
use crate::parser::{AstBuilder, AstNode};
use crate::parser::{ParseOptions, ParseResult};
use crate::path_utils::{get_directory_path, resolve_path};

/// Diagnostic codes for import-related errors
pub const DIAGNOSTIC_IMPORT_NOT_FOUND: &str = "import-not-found";
pub const DIAGNOSTIC_INVALID_IMPORT: &str = "invalid-import";

/// Source loader for loading and resolving TypeSpec files
pub struct SourceLoader {
    /// Map of loaded source file paths to their parsed AST root node IDs
    source_files: HashMap<String, NodeId>,
    /// Set of already-seen source file paths (for cycle detection)
    seen_source_files: std::collections::HashSet<String>,
    /// Diagnostic collector
    diagnostics: DiagnosticCollector,
    /// AST builder storage (needed to access nodes)
    ast_roots: HashMap<String, ParseResult>,
}

impl Default for SourceLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl SourceLoader {
    /// Create a new source loader
    pub fn new() -> Self {
        Self {
            source_files: HashMap::new(),
            seen_source_files: std::collections::HashSet::new(),
            diagnostics: DiagnosticCollector::new(),
            ast_roots: HashMap::new(),
        }
    }

    /// Get all loaded source files
    pub fn source_files(&self) -> &HashMap<String, NodeId> {
        &self.source_files
    }

    /// Get all collected diagnostics
    pub fn diagnostics(&self) -> &[Diagnostic] {
        self.diagnostics.diagnostics()
    }

    /// Get the parse result for a source file
    pub fn get_parse_result(&self, path: &str) -> Option<&ParseResult> {
        self.ast_roots.get(path)
    }

    /// Load a TypeSpec file from the given path
    ///
    /// This will:
    /// 1. Check for circular references
    /// 2. Read the file from disk
    /// 3. Parse it into an AST
    /// 4. Load any imports from the file
    pub fn load_type_spec_file(&mut self, path: &str) -> Result<NodeId, LoadFileError> {
        let path = normalize_path(path);

        // Check if we've already seen this file (circular reference check)
        if self.seen_source_files.contains(&path) {
            // Already processed, skip but don't error
            return self
                .source_files
                .get(&path)
                .copied()
                .ok_or(LoadFileError::FileNotFound(path));
        }

        // Check if file exists
        if !Path::new(&path).exists() {
            self.diagnostics.add(Diagnostic::error(
                DIAGNOSTIC_IMPORT_NOT_FOUND,
                &format!("Import file not found: {}", path),
            ));
            return Err(LoadFileError::FileNotFound(path));
        }

        // Mark as seen immediately to prevent circular references
        self.seen_source_files.insert(path.clone());

        // Read the file
        let content = fs::read_to_string(&path)
            .map_err(|e| LoadFileError::IoError(format!("Failed to read file {}: {}", path, e)))?;

        // Parse the file
        let parse_result = parse_with_options(&content);
        for diag in &parse_result.diagnostics {
            self.diagnostics
                .add(Diagnostic::error(diag.code, &diag.message));
        }

        let root_id = parse_result.root_id;

        // Store the parse result
        self.ast_roots.insert(path.clone(), parse_result);

        // Add to source files
        self.source_files.insert(path.clone(), root_id);

        // Load imports from this file
        self.load_script_imports(&path, root_id)?;

        Ok(root_id)
    }

    /// Load all import statements from a parsed TypeSpec script
    fn load_script_imports(
        &mut self,
        file_path: &str,
        root_id: NodeId,
    ) -> Result<(), LoadFileError> {
        let parse_result = self
            .ast_roots
            .get(file_path)
            .ok_or(LoadFileError::ParseError(
                "Could not find parse result".to_string(),
            ))?;

        // Get the TypeSpecScript node
        let script = parse_result.builder.id_to_node(root_id);
        let statements = match script {
            Some(AstNode::TypeSpecScript(ts)) => &ts.statements,
            _ => return Ok(()), // Not a TypeSpecScript, nothing to import
        };

        // First, collect all import paths (to avoid borrow issues)
        let import_paths: Vec<(String, NodeId)> = {
            let mut paths = Vec::new();
            for &stmt_id in statements {
                if let Some(AstNode::ImportStatement(import)) =
                    parse_result.builder.id_to_node(stmt_id)
                    && let Some(import_path_str) =
                        extract_string_literal_value(import.path, &parse_result.builder)
                {
                    paths.push((import_path_str, stmt_id));
                }
            }
            paths
        };

        // Now load each import (we can mutably borrow self now)
        // Note: file_path is the full path to the file, not base_dir
        for (import_path_str, stmt_id) in import_paths {
            self.resolve_and_load_import(&import_path_str, file_path, stmt_id)?;
        }

        Ok(())
    }

    /// Resolve an import path and load the imported file
    fn resolve_and_load_import(
        &mut self,
        import_path: &str,
        relative_to: &str,
        _diagnostic_target: NodeId,
    ) -> Result<(), LoadFileError> {
        // Resolve the import path
        let resolved = resolve_import_path(import_path, relative_to)?;

        // Check if it's a directory
        if Path::new(&resolved).is_dir() {
            self.load_directory(&resolved)?;
        } else {
            // It's a file - try with .tsp extension if not already present
            let file_path = ensure_tsp_extension(&resolved);
            self.load_type_spec_file(&file_path)?;
        }

        Ok(())
    }

    /// Load a directory, looking for main.tsp or index.tsp
    pub fn load_directory(&mut self, dir_path: &str) -> Result<NodeId, LoadFileError> {
        let dir_path = normalize_path(dir_path);

        // Try main.tsp first, then index.tsp
        let main_file = Path::new(&dir_path).join("main.tsp");
        let index_file = Path::new(&dir_path).join("index.tsp");

        let file_to_load = if main_file.exists() {
            main_file.to_string_lossy().to_string()
        } else if index_file.exists() {
            index_file.to_string_lossy().to_string()
        } else {
            self.diagnostics.add(Diagnostic::error(
                DIAGNOSTIC_IMPORT_NOT_FOUND,
                &format!(
                    "Directory '{}' does not contain main.tsp or index.tsp",
                    dir_path
                ),
            ));
            return Err(LoadFileError::DirectoryWithoutEntryPoint(dir_path));
        };

        self.load_type_spec_file(&file_to_load)
    }

    /// Resolve a path relative to another path
    ///
    /// Supports:
    /// - Relative paths like `./foo.tsp` or `../foo.tsp`
    /// - Library paths like `myLib` (for now, treated as relative)
    pub fn resolve_path(import_path: &str, relative_to: &str) -> String {
        let base_dir = get_directory_path(relative_to);
        resolve_path(&base_dir, &[import_path])
    }
}

/// Extract the string value from a StringLiteral node
fn extract_string_literal_value(node_id: NodeId, builder: &AstBuilder) -> Option<String> {
    if let Some(AstNode::StringLiteral(sl)) = builder.id_to_node(node_id) {
        Some(sl.value.clone())
    } else {
        None
    }
}

/// Resolve an import path to an absolute path
fn resolve_import_path(import_path: &str, relative_to: &str) -> Result<String, LoadFileError> {
    // If it's already absolute, return as-is
    if Path::new(import_path).is_absolute() {
        return Ok(import_path.to_string());
    }

    // Otherwise, resolve relative to the base directory
    let base_dir = get_directory_path(relative_to);
    let resolved = resolve_path(&base_dir, &[import_path]);
    Ok(resolved)
}

/// Normalize a file path
fn normalize_path(path: &str) -> String {
    crate::path_utils::normalize_path(path)
}

/// Ensure a path has .tsp extension
fn ensure_tsp_extension(path: &str) -> String {
    if path.ends_with(".tsp") || path.ends_with(".ts") {
        path.to_string()
    } else {
        format!("{}.tsp", path)
    }
}

/// Parse source with options
fn parse_with_options(source: &str) -> ParseResult {
    let options = ParseOptions::default();
    crate::parser::Parser::new(source, options).parse()
}

/// Error types for file loading
#[derive(Debug)]
pub enum LoadFileError {
    /// File was not found
    FileNotFound(String),
    /// IO error reading file
    IoError(String),
    /// Parse error
    ParseError(String),
    /// Directory does not have main.tsp or index.tsp
    DirectoryWithoutEntryPoint(String),
}

impl std::fmt::Display for LoadFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadFileError::FileNotFound(path) => write!(f, "File not found: {}", path),
            LoadFileError::IoError(msg) => write!(f, "IO error: {}", msg),
            LoadFileError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            LoadFileError::DirectoryWithoutEntryPoint(dir) => {
                write!(
                    f,
                    "Directory '{}' does not contain main.tsp or index.tsp",
                    dir
                )
            }
        }
    }
}

impl std::error::Error for LoadFileError {}

/// Create a diagnostic for an import that was not found
pub fn create_import_not_found_diagnostic(import_path: &str, file_path: &str) -> Diagnostic {
    Diagnostic::error(
        DIAGNOSTIC_IMPORT_NOT_FOUND,
        &format!(
            "Cannot find import '{}' in file '{}'",
            import_path, file_path
        ),
    )
}

/// Create a diagnostic for an invalid import
pub fn create_invalid_import_diagnostic(file_path: &str) -> Diagnostic {
    Diagnostic::error(
        DIAGNOSTIC_INVALID_IMPORT,
        &format!("Invalid import in file '{}'", file_path),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn make_temp_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join("typespec_test").join(name);
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_load_simple_file() {
        let temp_dir = make_temp_dir("load_simple");
        let file_path = temp_dir.join("test.tsp");
        fs::write(&file_path, "model Foo {}\n").unwrap();

        let mut loader = SourceLoader::new();
        let result = loader.load_type_spec_file(file_path.to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(loader.source_files().len(), 1);
    }

    #[test]
    fn test_load_file_with_import() {
        let test_dir = make_temp_dir("load_import");
        let main_path = test_dir.join("main.tsp");
        let foo_path = test_dir.join("foo.tsp");

        // Create the imported file first
        fs::write(&foo_path, "model Foo {}\n").unwrap();
        // Create main file that imports foo
        fs::write(&main_path, "import \"./foo.tsp\";\nmodel Bar { x: Foo }\n").unwrap();

        let mut loader = SourceLoader::new();
        let result = loader.load_type_spec_file(main_path.to_str().unwrap());

        assert!(result.is_ok());
        // Should load both main.tsp and foo.tsp
        assert_eq!(loader.source_files().len(), 2);
    }

    #[test]
    fn test_circular_reference_detection() {
        let test_dir = make_temp_dir("circular");

        // Create a.tsp that imports b.tsp
        let a_content = "import \"./b.tsp\";\nmodel A {}\n";
        let a_path = test_dir.join("a.tsp");
        fs::write(&a_path, a_content).unwrap();

        // Create b.tsp that imports a.tsp (circular)
        let b_content = "import \"./a.tsp\";\nmodel B {}\n";
        let b_path = test_dir.join("b.tsp");
        fs::write(&b_path, b_content).unwrap();

        let mut loader = SourceLoader::new();
        let result = loader.load_type_spec_file(a_path.to_str().unwrap());

        // Cycle detection should work
        assert!(result.is_ok());
        // Both files should be loaded (a.tsp first, then b.tsp, then a.tsp skipped due to cycle)
        assert_eq!(loader.source_files().len(), 2);
    }

    #[test]
    fn test_file_not_found() {
        let mut loader = SourceLoader::new();
        let result = loader.load_type_spec_file("/nonexistent/path.tsp");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            LoadFileError::FileNotFound(_)
        ));
    }

    #[test]
    fn test_load_directory() {
        let temp_dir = make_temp_dir("load_dir");

        // Create main.tsp in a subdirectory
        let main_content = "model Main {}\n";
        let subdir = temp_dir.join("mylib");
        fs::create_dir(&subdir).unwrap();
        let main_path = subdir.join("main.tsp");
        fs::write(&main_path, main_content).unwrap();

        let mut loader = SourceLoader::new();
        let result = loader.load_directory(subdir.to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(loader.source_files().len(), 1);
    }

    #[test]
    fn test_resolve_relative_path() {
        let resolved = SourceLoader::resolve_path("./foo.tsp", "/path/to/bar.tsp");
        // join_paths concatenates with ensure_trailing_separator
        assert_eq!(resolved, "/path/to/foo.tsp");
    }

    #[test]
    fn test_resolve_parent_path() {
        // Note: path_utils join_paths concatenates paths without semantic resolution
        // So /path/to/bar + ../foo.tsp becomes /path/to/bar/../foo.tsp
        // which normalizes to /path/to/foo.tsp (not /path/foo.tsp as one might expect)
        let resolved = SourceLoader::resolve_path("../foo.tsp", "/path/to/bar/baz.tsp");
        // After normalization: /path/to/bar/../foo.tsp -> /path/to/foo.tsp
        assert_eq!(resolved, "/path/to/foo.tsp");
    }

    #[test]
    fn test_normalize_path() {
        assert_eq!(normalize_path("a/b\\c"), "a/b/c");
        // normalize_path strips leading ./
        assert_eq!(normalize_path("./foo"), "foo");
    }

    #[test]
    fn test_ensure_tsp_extension() {
        assert_eq!(ensure_tsp_extension("foo"), "foo.tsp");
        assert_eq!(ensure_tsp_extension("foo.tsp"), "foo.tsp");
        assert_eq!(ensure_tsp_extension("foo.ts"), "foo.ts");
    }
}
