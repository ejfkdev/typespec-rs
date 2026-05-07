//! Compilation pipeline: source → parse → check → emit

#![allow(dead_code)]

use std::io::Write;
use std::path::PathBuf;
use typespec_rs::checker::Checker;
#[cfg(feature = "wasm-extensions")]
use typespec_rs::checker::types::{DecoratorApplication, Type, TypeId};
use typespec_rs::diagnostics::DiagnosticSeverity;
use typespec_rs::emit::{Emitter, JsonEmitter, OpenAPIEmitter, YamlEmitter};
use typespec_rs::parser::{ParseOptions, parse_with_libraries};

/// Compilation pipeline configuration
pub struct Pipeline {
    pub source: String,
    pub format: String,
    pub openapi_version: String,
    pub output: Option<PathBuf>,
    pub no_stdlib: bool,
    pub no_emit: bool,
    pub extensions: Vec<PathBuf>,
    /// Custom decorators to register programmatically (name, namespace, target_type)
    pub custom_decorators: Vec<(String, String, String)>,
    pub verbose: bool,
    pub quiet: bool,
}

impl Pipeline {
    /// Run the full compilation pipeline
    pub fn run(&self) -> Result<(), String> {
        // 1. Parse (with globally registered libraries + any extra)
        let parse_result = if self.no_stdlib {
            parse_with_libraries(&self.source, vec![])
        } else {
            let options = ParseOptions::default();
            parse_with_libraries(&self.source, options.libraries)
        };

        if !self.quiet && self.verbose {
            eprintln!("Parsed {} AST nodes", parse_result.builder.nodes.len());
        }

        // 2. Type check
        let mut checker = Checker::new();
        checker.set_parse_result(parse_result.root_id, parse_result.builder.clone());

        // Register custom decorators before check_program()
        for (name, namespace, target_type) in &self.custom_decorators {
            checker.register_decorator(name, namespace, target_type);
        }

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
            return self.run_with_extensions(&mut checker, &parse_result);
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
    fn run_with_extensions(
        &self,
        checker: &mut Checker,
        parse_result: &typespec_rs::parser::ParseResult,
    ) -> Result<(), String> {
        use crate::type_serializer::TypeGraphSerializer;
        use std::cell::RefCell;
        use std::rc::Rc;

        // Serialize type graph for WASM consumption
        let graph_json = TypeGraphSerializer::serialize(checker);

        // Extract state_accessors from checker, wrap in Rc<RefCell<>>
        let state = Rc::new(RefCell::new(std::mem::take(&mut checker.state_accessors)));

        let mut host = crate::wasm_host::WasmHost::new()
            .map_err(|e| format!("Failed to initialize WASM runtime: {}", e))?;

        let mut loaded_extensions: Vec<crate::wasm_host::WasmExtension> = Vec::new();

        // Load each extension
        for ext_path in &self.extensions {
            if !self.quiet {
                eprintln!("Loading extension: {}", ext_path.display());
            }
            match host.load_extension(ext_path, state.clone()) {
                Ok(ext) => loaded_extensions.push(ext),
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to load extension '{}': {}",
                        ext_path.display(),
                        e
                    );
                }
            }
        }

        // Decorator handling phase
        for ext in &mut loaded_extensions {
            let decorator_defs: Vec<(String, String)> = ext
                .manifest
                .decorators
                .iter()
                .map(|d| (d.name.clone(), d.namespace.clone()))
                .collect();

            for (dec_name, dec_namespace) in &decorator_defs {
                for type_id in 0..checker.type_store.len() as u32 {
                    let Some(t) = checker.type_store.get(type_id) else {
                        continue;
                    };
                    let Some(decorators) = t.decorators() else {
                        continue;
                    };
                    for dec_app in decorators {
                        if let Some(def_id) = dec_app.definition
                            && let Some(Type::Decorator(dt)) =
                                checker.type_store.get(def_id)
                            && dt.name == *dec_name
                            && Self::namespace_matches(
                                checker,
                                dt.namespace,
                                dec_namespace,
                            )
                            {
                                let input = Self::build_decorator_input(
                                    type_id, &dt.name, dec_app,
                                );
                                if let Err(e) = ext.handle_decorator(&input) {
                                    eprintln!(
                                        "Warning: decorator handler failed: {}",
                                        e
                                    );
                                }
                            }
                    }
                }
            }
        }

        // Merge diagnostics from extensions
        for ext in &mut loaded_extensions {
            for diag in ext.take_diagnostics() {
                let severity = match diag.severity {
                    0 => DiagnosticSeverity::Error,
                    1 => DiagnosticSeverity::Warning,
                    _ => DiagnosticSeverity::Warning,
                };
                checker.add_diagnostic(typespec_rs::diagnostics::Diagnostic {
                    code: diag.code,
                    severity,
                    message: diag.message,
                    url: None,
                    location: None,
                    related: Vec::new(),
                });
            }
        }

        // Put state_accessors back into checker
        checker.state_accessors = Rc::try_unwrap(state)
            .map(|rc| rc.into_inner())
            .unwrap_or_else(|rc| rc.borrow().clone());

        // Emit phase: find extension that handles the requested format
        for ext in &mut loaded_extensions {
            if ext.manifest.formats.contains(&self.format) {
                let output = ext
                    .emit(&graph_json)
                    .map_err(|e| format!("Extension emission failed: {}", e))?;
                self.write_output(&output)?;
                return Ok(());
            }
        }

        // Fallback to built-in emitters if no extension handles the format
        let output = self.emit_builtin(parse_result, checker)?;
        self.write_output(&output)?;

        Ok(())
    }

    #[cfg(feature = "wasm-extensions")]
    fn namespace_matches(
        checker: &Checker,
        ns_type_id: Option<TypeId>,
        expected_ns: &str,
    ) -> bool {
        match ns_type_id {
            None => expected_ns.is_empty(),
            Some(id) => {
                if let Some(Type::Namespace(ns)) = checker.type_store.get(id) {
                    ns.name == expected_ns
                } else {
                    false
                }
            }
        }
    }

    #[cfg(feature = "wasm-extensions")]
    fn build_decorator_input(
        type_id: u32,
        decorator_name: &str,
        dec_app: &DecoratorApplication,
    ) -> String {
        let args: Vec<String> = dec_app
            .args
            .iter()
            .map(|arg| match &arg.js_value {
                Some(typespec_rs::checker::types::DecoratorMarshalledValue::String(s)) => {
                    format!(
                        "\"{}\"",
                        s.replace('\\', "\\\\").replace('"', "\\\"")
                    )
                }
                Some(typespec_rs::checker::types::DecoratorMarshalledValue::Number(n)) => {
                    format!("{}", n)
                }
                Some(typespec_rs::checker::types::DecoratorMarshalledValue::Boolean(b)) => {
                    format!("{}", b)
                }
                Some(typespec_rs::checker::types::DecoratorMarshalledValue::Null) => {
                    "null".to_string()
                }
                Some(typespec_rs::checker::types::DecoratorMarshalledValue::Type(id)) => {
                    format!("{{\"type\":{}}}", id)
                }
                Some(typespec_rs::checker::types::DecoratorMarshalledValue::Value(id)) => {
                    format!("{{\"value\":{}}}", id)
                }
                _ => "null".to_string(),
            })
            .collect();

        format!(
            r#"{{"type_id":{},"decorator_name":"{}","args":[{}]}}"#,
            type_id,
            decorator_name.replace('"', "\\\""),
            args.join(",")
        )
    }
}
