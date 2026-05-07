//! tspc — TypeSpec Compiler CLI
//!
//! Converts TypeSpec (.tsp) files to JSON, YAML, OpenAPI and other formats.
//! Supports WASM extensions for custom decorators and output formats.

mod pipeline;
mod type_serializer;
#[cfg(feature = "wasm-extensions")]
mod wasm_host;

use clap::Parser;
use std::path::PathBuf;
use std::process;

/// TypeSpec Compiler — convert .tsp files to various output formats
#[derive(Parser, Debug)]
#[command(name = "tspc", version, about = "TypeSpec Compiler CLI")]
struct Cli {
    /// TypeSpec source file path, or "-" for stdin
    input: String,

    /// Output format: json, yaml, openapi
    #[arg(short, long, default_value = "json")]
    format: String,

    /// OpenAPI version: 3.0.0, 3.1.0
    #[arg(long, default_value = "3.0.0")]
    openapi_version: String,

    /// Output file path (default: stdout)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Don't load the standard library
    #[arg(long)]
    no_stdlib: bool,

    /// Type-check only, don't emit output
    #[arg(long)]
    no_emit: bool,

    /// Load a WASM extension (.wasm file), can be repeated
    #[arg(short, long)]
    extension: Vec<PathBuf>,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Suppress non-error output
    #[arg(short, long)]
    quiet: bool,
}

fn main() {
    // Register built-in libraries
    typespec_rs::parser::register_library(
        "http",
        typespec_rs::libs::http::http_library_source(),
    );

    let cli = Cli::parse();

    // Collect custom decorators from WASM extension manifests
    #[allow(unused_mut)]
    let mut custom_decorators: Vec<(String, String, String)> = Vec::new();

    #[cfg(feature = "wasm-extensions")]
    if !cli.extension.is_empty() {
        match crate::wasm_host::WasmHost::new() {
            Ok(mut host) => {
                for ext_path in &cli.extension {
                    match host.extract_manifest(ext_path) {
                        Ok(manifest) => {
                            for dec in &manifest.decorators {
                                custom_decorators.push((
                                    dec.name.clone(),
                                    dec.namespace.clone(),
                                    "unknown".to_string(),
                                ));
                            }
                        }
                        Err(e) => {
                            eprintln!(
                                "Warning: Could not read manifest from {}: {}",
                                ext_path.display(),
                                e
                            );
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Warning: Failed to initialize WASM runtime: {}", e);
            }
        }
    }

    // Read input
    let source = if cli.input == "-" {
        use std::io::Read;
        let mut buf = String::new();
        if let Err(e) = std::io::stdin().read_to_string(&mut buf) {
            eprintln!("Error reading stdin: {}", e);
            process::exit(1);
        }
        buf
    } else {
        match std::fs::read_to_string(&cli.input) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Error reading '{}': {}", cli.input, e);
                process::exit(1);
            }
        }
    };

    // Run pipeline
    let pipeline = pipeline::Pipeline {
        source,
        format: cli.format,
        openapi_version: cli.openapi_version,
        output: cli.output,
        no_stdlib: cli.no_stdlib,
        no_emit: cli.no_emit,
        extensions: cli.extension,
        custom_decorators,
        verbose: cli.verbose,
        quiet: cli.quiet,
    };

    if let Err(e) = pipeline.run() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
