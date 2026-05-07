# typespec-rs

[![CI](https://github.com/ejfkdev/typespec-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/ejfkdev/typespec-rs/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![crates.io](https://img.shields.io/crates/v/typespec_rs.svg)](https://crates.io/crates/typespec_rs)
[![tspc](https://img.shields.io/crates/v/tspc.svg)](https://crates.io/crates/tspc)

A Rust implementation of the [TypeSpec](https://typespec.io/) type system — parser, type checker, and emitter.

## What is TypeSpec?

[TypeSpec](https://typespec.io/) is a language for describing cloud service APIs. It lets you define models, operations, and services declaratively, then generate OpenAPI, JSON Schema, protobuf, and other formats from a single source of truth.

**typespec-rs** is an independent Rust port of the TypeSpec compiler — not a binding to the TypeScript compiler. It implements the parser, checker, and emitter pipeline natively in Rust.

> **Note:** This project is not affiliated with or endorsed by Microsoft. TypeSpec is a trademark of Microsoft Corporation.

## Status

This project is in **early development**. The core compiler pipeline works:

| Component | Status |
|-----------|--------|
| Scanner/Lexer | Complete |
| Parser | Complete |
| Type Checker | Partial |
| YAML Emitter | Working |
| JSON Emitter | Working |
| OpenAPI 3 Emitter | Working |
| CLI (`tspc`) | Working |
| WASM Extensions | Experimental |

2,800+ tests passing.

## Quick Start

### Library

Add to your `Cargo.toml`:

```toml
[dependencies]
typespec_rs = "0.1.0"
```

Parse TypeSpec and emit YAML/JSON:

```rust
use typespec_rs::emit::{to_yaml, to_json};

let src = r#"
    model Pet {
        id: string;
        name: string;
        age?: int32;
        type: "dog" | "cat" | "bird";
    }
"#;

let yaml = to_yaml(src).unwrap();
println!("{}", yaml.output);

let json = to_json(src).unwrap();
println!("{}", json.output);
```

### CLI

Install `tspc`:

```bash
cargo install tspc
```

Or download pre-built binaries from [GitHub Releases](https://github.com/ejfkdev/typespec-rs/releases).

Usage:

```bash
# Parse to JSON (default)
tspc petstore.tsp

# Parse to YAML
tspc -f yaml petstore.tsp

# Generate OpenAPI 3.0
tspc -f openapi petstore.tsp

# OpenAPI 3.1
tspc -f openapi --openapi-version 3.1.0 petstore.tsp

# Write to file
tspc -f openapi petstore.tsp -o petstore.openapi.json

# Type-check only
tspc --no-emit petstore.tsp

# Read from stdin
echo 'model Pet { name: string }' | tspc -f json -
```

## Custom Decorators (Library Injection)

You can inject custom decorator declarations without modifying typespec-rs source code. This is useful for defining domain-specific decorators (e.g., CLI commands, custom protocols).

### Global Registration

Register libraries once at program startup — all subsequent `parse()` calls automatically include them:

```rust
use typespec_rs::parser::{parse, register_library};
use typespec_rs::libs::http::http_library_source;

// Register at startup
register_library("http", http_library_source());
register_library("cli", r#"namespace CLI;
    extern dec command(target: Operation, name?: valueof string);
    extern dec flag(target: ModelProperty, name?: valueof string);
    extern dec arg(target: ModelProperty, name?: valueof string);
"#.to_string());

// parse() now automatically injects all registered libraries
let result = parse(source);
```

### ParseOptions Presets

For one-off usage without global state:

```rust
use typespec_rs::parser::{Parser, ParseOptions};

// With HTTP library
let result = Parser::new(source, ParseOptions::with_http()).parse();

// With HTTP + custom libraries
let result = Parser::new(source, ParseOptions::with_http_and(vec![
    r#"namespace CLI; extern dec command(target: Operation);"#.to_string(),
])).parse();

// Builder pattern
let result = Parser::new(source, ParseOptions::new(vec![])
    .library(http_library_source())
    .library(custom_lib),
).parse();
```

### Bypassing the Registry

Use `parse_with_libraries` for explicit control:

```rust
use typespec_rs::parser::parse_with_libraries;

let result = parse_with_libraries(source, vec![http_library_source()]);
```

## WASM Extensions (Experimental)

`tspc` supports loading WASM extensions for custom decorators and output formats. Enable the `wasm-extensions` feature:

```toml
[dependencies]
tspc = { version = "0.1.0", features = ["wasm-extensions"] }
```

```bash
# Load a WASM extension
tspc -f markdown -e my_extension.wasm petstore.tsp
```

WASM extensions can:
- Register custom decorator declarations
- Handle decorator invocation during type checking
- Provide custom emitters for new output formats

See the [WASM Extension ABI](#) documentation for details on building extensions.

## Examples

Run with `cargo run --example <name>`:

| Example | Description |
|---------|-------------|
| `quick_start` | Convert TypeSpec to YAML/JSON in 5 lines |
| `model_examples` | Model definitions with optional fields, unions, arrays |
| `parse_and_inspect` | Low-level AST parsing and inspection |
| `petstore` | Full PetStore API parsing example |
| `tsp_to_json` | Parse and emit with JSON/YAML output |

## Architecture

```
Source Code → Scanner → Parser → AST → Checker → Typed AST → Emitter → Output
```

- **Scanner** (`scanner/`) — Tokenizes TypeSpec source into `TokenKind` stream
- **Parser** (`parser/`) — Builds an AST from the token stream
- **Checker** (`checker/`) — Type checking, symbol resolution, decorator validation
- **Emitter** (`emit/`) — Converts checked types to YAML, JSON, or OpenAPI
- **CLI** (`crates/tspc/`) — Command-line interface with WASM extension support
- **Libs** (`libs/`) — Built-in library sources (HTTP, OpenAPI, etc.)

## Feature Coverage

What's ported from the [TypeSpec compiler](https://github.com/microsoft/typespec):

- Full scanner with doc comments, string templates, conflict markers
- Complete parser for all declaration types
- Type system with 25+ type kinds (Model, Interface, Enum, Union, Scalar, Template, etc.)
- Type relation/assignability checking
- Decorator application and validation
- Template declaration and instantiation
- Standard library types (string, int32, float64, utcDateTime, etc.)
- Helper libraries: HTTP types, status codes, content types, URI templates, OpenAPI/OpenAPI3/JSON Schema/protobuf/versioning type definitions
- External library injection API for custom decorator declarations
- CLI with cross-platform binary releases (UPX compressed)

What's not yet ported:

- Full `Program` pipeline (multi-file compilation, import resolution)
- Decorator execution at `finishType` time
- HTTP route/payload resolution
- OpenAPI3 / JSON Schema emitter implementations
- Language Server Protocol (LSP) support
- Source loader (async I/O)

## Development

```bash
# Run tests
cargo test --lib

# Run CLI tests
cargo test -p tspc

# Run linter
cargo clippy --all-targets

# Check formatting
cargo fmt --all -- --check

# Build docs
cargo doc --no-deps

# Run examples
cargo run --example quick_start
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| [`regex`](https://crates.io/crates/regex) | Pattern matching in scanner |
| [`bitflags`](https://crates.io/crates/bitflags) | Bitflag types for visibility, symbol flags |

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

This project is licensed under the [MIT License](LICENSE).

## Security

See [SECURITY.md](SECURITY.md) for vulnerability reporting guidelines.
