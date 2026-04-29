# tspc — TypeSpec Compiler CLI

[![CI](https://github.com/ejfkdev/typespec-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/ejfkdev/typespec-rs/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![crates.io](https://img.shields.io/crates/v/tspc.svg)](https://crates.io/crates/tspc)

Command-line tool for compiling [TypeSpec](https://typespec.io/) (`.tsp`) files to JSON, YAML, and OpenAPI.

## Install

```bash
cargo install tspc
```

Or download a pre-built binary from the [latest release](https://github.com/ejfkdev/typespec-rs/releases).

## Usage

```bash
# Compile to JSON (default)
tspc petstore.tsp

# Compile to YAML
tspc -f yaml petstore.tsp

# Compile to OpenAPI
tspc -f openapi petstore.tsp

# Read from stdin
echo 'model Pet { name: string }' | tspc -f json -

# Write to file
tspc -f yaml petstore.tsp -o petstore.yaml

# Type-check only
tspc --no-emit petstore.tsp
```

## Options

| Option | Description |
|--------|-------------|
| `<INPUT>` | TypeSpec source file path, or `-` for stdin |
| `-f, --format <FORMAT>` | Output format: `json`, `yaml`, `openapi` (default: `json`) |
| `--openapi-version <VER>` | OpenAPI version: `3.0.0`, `3.1.0` (default: `3.0.0`) |
| `-o, --output <FILE>` | Output file path (default: stdout) |
| `--no-stdlib` | Don't load the standard library |
| `--no-emit` | Type-check only, don't emit output |
| `-e, --extension <PATH>` | Load a WASM extension (repeatable, requires `wasm-extensions` feature) |
| `-v, --verbose` | Verbose output |
| `-q, --quiet` | Suppress non-error output |

## WASM Extensions

Build with WASM extension support:

```bash
cargo install tspc --features wasm-extensions
```

WASM extensions allow custom decorators and output formats. See the [main repository](https://github.com/ejfkdev/typespec-rs) for the extension ABI specification.

## License

MIT
