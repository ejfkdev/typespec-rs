//! tspc — TypeSpec Compiler CLI
//!
//! Converts TypeSpec (.tsp) files to JSON, YAML, OpenAPI, JSON Schema and other formats.
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
#[command(
    name = "tspc",
    version,
    about = "TypeSpec Compiler CLI",
    long_about = "Compile TypeSpec (.tsp) files and emit JSON, YAML, OpenAPI, or JSON Schema.\n\n\
The HTTP library is preloaded, so decorators such as @service, @route, @get, \
@header, @query, @body, and @statusCode are available without imports. \
The JSON Schema library is preloaded too: `using TypeSpec.JsonSchema;` brings \
@extension into scope.\n\
Compiler features (for example union-extends, declaration-expressions, \
auto-decorators) can be enabled with --features.",
    after_help = "Examples:\n  \
  tspc api.tsp                         Emit JSON (default)\n  \
  tspc -f yaml api.tsp                 Emit YAML\n  \
  tspc -f openapi api.tsp              Emit OpenAPI 3.0\n  \
  tspc -f openapi --openapi-version 3.1.0 api.tsp\n  \
  tspc -f json-schema api.tsp          Emit a JSON Schema bundle\n  \
  tspc --no-emit api.tsp               Type-check only\n  \
  tspc --features union-extends api.tsp"
)]
struct Cli {
    /// TypeSpec source file path, or "-" for stdin
    input: String,

    /// Output format: json, yaml, openapi, json-schema
    #[arg(short, long, default_value = "json", value_name = "FORMAT")]
    format: String,

    /// OpenAPI version: 3.0.0, 3.1.0
    #[arg(long, default_value = "3.0.0")]
    openapi_version: String,

    /// JSON Schema: also write each fixed-width numeric scalar's name
    /// (int8, uint16, float32, ...) as `format`
    #[arg(long)]
    width_format: bool,

    /// JSON Schema: close object schemas to undeclared keys
    /// (the official `seal-object-schemas` option)
    #[arg(long)]
    seal_object_schemas: bool,

    /// Output file path (default: stdout)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Don't load the standard library
    #[arg(long)]
    no_stdlib: bool,

    /// Type-check only, don't emit output
    #[arg(long)]
    no_emit: bool,

    /// Enable compiler features (comma separated): union-extends,
    /// declaration-expressions, auto-decorators, function-declarations
    #[arg(long, value_delimiter = ',', value_name = "FEATURE")]
    features: Vec<String>,

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
    typespec_rs::parser::register_library("http", typespec_rs::libs::http::http_library_source());
    typespec_rs::parser::register_library(
        "http-client",
        typespec_rs::libs::http_client::HTTP_CLIENT_TSP.to_string(),
    );
    typespec_rs::parser::register_library(
        "json-schema",
        typespec_rs::libs::json_schema::JSON_SCHEMA_DECORATORS_TSP.to_string(),
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
        width_format: cli.width_format,
        seal_object_schemas: cli.seal_object_schemas,
        output: cli.output,
        no_stdlib: cli.no_stdlib,
        no_emit: cli.no_emit,
        features: cli.features,
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
