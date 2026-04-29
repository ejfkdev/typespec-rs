//! Compilation pipeline: source → parse → check → emit

#![allow(dead_code)]

use std::io::Write;
use std::path::PathBuf;
use typespec_rs::checker::Checker;
use typespec_rs::diagnostics::DiagnosticSeverity;
use typespec_rs::emit::{Emitter, JsonEmitter, OpenAPIEmitter, YamlEmitter};
use typespec_rs::parser::parse;

/// Compilation pipeline configuration
pub struct Pipeline {
    pub source: String,
    pub format: String,
    pub openapi_version: String,
    pub output: Option<PathBuf>,
    pub no_stdlib: bool,
    pub no_emit: bool,
    pub extensions: Vec<PathBuf>,
    pub verbose: bool,
    pub quiet: bool,
}

impl Pipeline {
    /// Run the full compilation pipeline
    pub fn run(&self) -> Result<(), String> {
        // 1. Parse
        let parse_result = parse(&self.source);

        if !self.quiet && self.verbose {
            eprintln!("Parsed {} AST nodes", parse_result.builder.nodes.len());
        }

        // 2. Type check
        let mut checker = Checker::new();
        checker.set_parse_result(parse_result.root_id, parse_result.builder.clone());
        checker.check_program();

        let error_count = checker
            .diagnostics()
            .iter()
            .filter(|d| d.severity == DiagnosticSeverity::Error)
            .count();
        let warning_count = checker
            .diagnostics()
            .iter()
            .filter(|d| d.severity == DiagnosticSeverity::Warning)
            .count();

        // 3. Report diagnostics
        for diag in checker.diagnostics() {
            let loc = diag
                .location
                .as_ref()
                .map(|l| format!(":{}:{}", l.file, l.start))
                .unwrap_or_default();
            eprintln!("{}{}: {}: {}", diag.severity, loc, diag.code, diag.message);
        }

        if !self.quiet && (error_count > 0 || warning_count > 0) {
            eprintln!(
                "Found {} error(s), {} warning(s)",
                error_count, warning_count
            );
        }

        if error_count > 0 {
            return Err(format!("Compilation failed with {} error(s)", error_count));
        }

        // 4. Emit
        if self.no_emit {
            if !self.quiet {
                eprintln!("Type check passed (no-emit mode)");
            }
            return Ok(());
        }

        // Handle WASM extensions
        #[cfg(feature = "wasm-extensions")]
        if !self.extensions.is_empty() {
            return self.run_with_extensions(&checker);
        }

        // Built-in emitters
        let output = self.emit_builtin(&parse_result, &checker)?;

        // 5. Write output
        self.write_output(&output)?;

        if !self.quiet && self.verbose {
            eprintln!("Output format: {}", self.format);
        }

        Ok(())
    }

    /// Emit using built-in emitters
    fn emit_builtin(
        &self,
        parse_result: &typespec_rs::parser::ParseResult,
        checker: &Checker,
    ) -> Result<String, String> {
        match self.format.as_str() {
            "json" => {
                let emitter = JsonEmitter::new();
                let result = emitter.emit(parse_result)?;
                Ok(result.output)
            }
            "yaml" => {
                let emitter = YamlEmitter::new();
                let result = emitter.emit(parse_result)?;
                Ok(result.output)
            }
            "openapi" | "openapi3" => {
                let mut emitter = OpenAPIEmitter::new();
                if self.openapi_version.starts_with("3.1") {
                    emitter = emitter.version("3.1.0");
                }
                let result = emitter.emit_from_checker(checker)?;
                Ok(result.output)
            }
            _ => Err(format!(
                "Unknown format '{}'. Supported: json, yaml, openapi",
                self.format
            )),
        }
    }

    /// Write output to file or stdout
    fn write_output(&self, content: &str) -> Result<(), String> {
        if let Some(ref path) = self.output_path() {
            std::fs::write(path, content)
                .map_err(|e| format!("Failed to write '{}': {}", path.display(), e))?;
            if !self.quiet && self.verbose {
                eprintln!("Written to {}", path.display());
            }
        } else {
            let stdout = std::io::stdout();
            let mut out = stdout.lock();
            out.write_all(content.as_bytes())
                .map_err(|e| format!("Failed to write to stdout: {}", e))?;
            out.write_all(b"\n")
                .map_err(|e| format!("Failed to write newline: {}", e))?;
        }
        Ok(())
    }

    fn output_path(&self) -> Option<&PathBuf> {
        self.output.as_ref()
    }

    /// Run with WASM extensions
    #[cfg(feature = "wasm-extensions")]
    fn run_with_extensions(&self, checker: &Checker) -> Result<(), String> {
        use crate::type_serializer::TypeGraphSerializer;

        // Serialize type graph for WASM consumption
        let graph_json = TypeGraphSerializer::serialize(checker);

        // Load and run each extension
        for ext_path in &self.extensions {
            if !self.quiet {
                eprintln!("Loading extension: {}", ext_path.display());
            }

            let mut host = crate::wasm_host::WasmHost::new(checker)
                .map_err(|e| format!("Failed to initialize WASM runtime: {}", e))?;

            let extension = host
                .load_extension(ext_path)
                .map_err(|e| format!("Failed to load extension '{}': {}", ext_path.display(), e))?;

            // Check if this extension provides the requested format
            if extension.formats.contains(&self.format) {
                let output = host
                    .emit(&extension, &graph_json)
                    .map_err(|e| format!("Extension emission failed: {}", e))?;

                self.write_output(&output)?;
                return Ok(());
            }
        }

        // Fallback to built-in if no extension handles the format
        Err(format!(
            "No extension provides format '{}'. Loaded extensions: {}",
            self.format,
            self.extensions
                .iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ))
    }
}
