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
| Type Checker | Working |
| YAML Emitter | Working |
| JSON Emitter | Working |
| OpenAPI 3 Emitter | Working |
| JSON Schema Emitter | Subset (`json-schema` feature) |
| Protobuf Library | Working |
| CLI (`tspc`) | Working |
| WASM Extensions | Working |
| Programmatic Decorator API | Working |
| Auto Decorators (`auto dec`) | Working |

3,400+ tests passing (synced with `@typespec/compiler` 1.17.0, upstream `microsoft/typespec@a22c5330`).

**MSRV:** Rust 1.97. **Upstream sync:** `@typespec/compiler` 1.17.0.

## Quick Start

### Library

Add to your `Cargo.toml`:

```toml
[dependencies]
typespec_rs = "0.6.0"
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

JSON Schema output needs the `json-schema` feature, which adds `serde_json`:

```toml
[dependencies]
typespec_rs = { version = "0.6.0", features = ["json-schema"] }
```

```rust
use typespec_rs::checker::Checker;
use typespec_rs::emit::JsonSchemaEmitter;
use typespec_rs::parser::parse;

let pr = parse("model Pet { name: string; age?: uint8; }");
let mut checker = Checker::new();
checker.set_parse_result(pr.root_id, pr.builder.clone());
checker.check_program();

// A serde_json::Value
let schema = JsonSchemaEmitter::new()
    .width_format(true)         // also write `format: "uint8"` etc.
    .seal_object_schemas(true)  // reject undeclared keys
    .emit_value(&checker);
println!("{schema:#}");
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

# JSON Schema (draft 2020-12) bundle
tspc -f json-schema petstore.tsp

# JSON Schema with widths as `format`, and objects closed to undeclared keys
tspc -f json-schema --width-format --seal-object-schemas petstore.tsp

# Write to file
tspc -f openapi petstore.tsp -o petstore.openapi.json

# Type-check only
tspc --no-emit petstore.tsp

# Enable compiler features (comma separated)
tspc --features union-extends,declaration-expressions petstore.tsp

# List all options and examples
tspc --help

# Read from stdin
echo 'model Pet { name: string }' | tspc -f json -
```

## Custom Decorators

### Public Checker API

After running `check_program()`, the `Checker` provides rich accessor methods for downstream consumers (emitters, code generators, validation tools):

```rust
use typespec_rs::checker::Checker;
use typespec_rs::parser;

let result = parser::parse(source);
let mut checker = Checker::new();
checker.set_parse_result(result.root_id, result.builder);
checker.check_program();

// --- Decorator lookup ---
let model_id = checker.lookup_type_by_fqn("MyNs.Pet").unwrap();

// Find decorators by name (handles both "doc" and "TypeSpec.doc")
if let Some(dec) = checker.find_decorator(model_id, "doc") {
    // inspect dec.args, etc.
}

// Extract typed arguments directly
let doc: Option<String> = checker.get_decorator_string_arg(model_id, "doc", 0);
let min_val: Option<f64> = checker.get_decorator_numeric_arg(model_id, "minValue", 0);

// --- Constraint accessors ---
checker.get_type_min_value(model_id);   // from @minValue
checker.get_type_max_value(model_id);   // from @maxValue
checker.get_type_min_length(model_id);  // from @minLength
checker.get_type_max_length(model_id);  // from @maxLength
checker.get_type_pattern(model_id);     // from @pattern
checker.get_type_format(model_id);      // from @format
checker.get_type_doc(model_id);         // from @doc
checker.get_type_summary(model_id);     // from @summary
checker.is_type_error(model_id);        // from @error

// --- Type lookup ---
checker.lookup_type_by_fqn("MyNs.Pet");           // Option<TypeId>
checker.lookup_type_by_fqn("A.B.C.NestedModel");  // nested namespaces

// --- Type iteration ---
for (id, model) in checker.iter_models() { /* ... */ }
for (id, op) in checker.iter_operations() { /* ... */ }
for (id, en) in checker.iter_enums() { /* ... */ }
for (id, ns) in checker.iter_namespaces() { /* ... */ }
for (id, iface) in checker.iter_interfaces() { /* ... */ }

// --- Property access ---
let prop_id = checker.get_model_property(model_id, "name").unwrap();
let prop_type = checker.get_property_type(prop_id);  // resolves aliases

// Walk all properties including inherited ones (base first)
for (name, prop_id) in checker.walk_model_properties(model_id) {
    // ...
}

// --- Value extraction ---
checker.extract_default_value(prop_id);       // "42" from `count: int32 = 42`
checker.extract_enum_member_value(member_id); // "red" from `Red: "red"`
```

All `Type` variants now expose `doc()` and `summary()` methods that return `Option<&str>`, populated from `@doc` / `@summary` decorators. `ModelProperty`, `EnumMember`, and `UnionVariant` also carry their own `doc` and `summary` fields.

### Custom Decorators

typespec-rs provides two ways to register custom decorators:

### Programmatic Registration (Recommended)

Register decorators directly on the `Checker` before calling `check_program()`. This bypasses source parsing, so it works with reserved keywords (`flag`, `arg`, `env`) and complex types without type resolution issues:

```rust
use typespec_rs::checker::Checker;
use typespec_rs::parser;

let parse_result = parser::parse(source);
let mut checker = Checker::new();
checker.set_parse_result(parse_result.root_id, parse_result.builder.clone());

// Register custom decorators
checker.register_decorator("command", "CLI", "Operation");
checker.register_decorator("flag", "CLI", "unknown");
checker.register_decorator("env", "CLI", "unknown");

// Or batch registration
checker.register_decorators(vec![
    ("route", "HTTP", "Operation"),
    ("tag", "API", "unknown"),
]);

checker.check_program();
```

If the namespace doesn't exist yet, it will be created automatically under the global namespace. Duplicate registrations (same name + namespace) are silently skipped.

### Library Source Injection

For decorators that need parameter type information, inject TypeSpec source via the library registry:

```rust
use typespec_rs::parser::{parse, register_library};
use typespec_rs::libs::http::http_library_source;

// Register at startup
register_library("http", http_library_source());

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
    custom_lib_source,
])).parse();

// Builder pattern
let result = Parser::new(source, ParseOptions::new(vec![])
    .library(http_library_source())
    .library(custom_lib),
).parse();
```

## WASM Extensions (Experimental)

`tspc` supports loading WASM extensions for custom decorators and output formats. Enable the `wasm-extensions` feature:

```toml
[dependencies]
tspc = { version = "0.6.0", features = ["wasm-extensions"] }
```

```bash
# Load a WASM extension
tspc -f markdown -e my_extension.wasm petstore.tsp
```

WASM extensions can:
- Declare custom decorators (registered programmatically, avoiding keyword conflicts)
- Handle decorator invocation during type checking
- Read/write decorator state via `tsp.state_set` / `tsp.state_get` host functions
- Report diagnostics back to the compiler
- Provide custom emitters for new output formats

### WASM Extension ABI

A WASM extension must export the following functions:

| Export | Signature | Description |
|--------|-----------|-------------|
| `allocate` | `(i32) -> i32` | Allocate `n` bytes in guest memory |
| `deallocate` | `(i32, i32)` | Free guest memory |
| `tsp_ext_manifest` | `() -> i32` | Return pointer to JSON manifest |
| `tsp_ext_manifest_len` | `() -> i32` | Return manifest length |
| `tsp_ext_init` | `(i32, i32) -> i32` | Initialize with options JSON |
| `tsp_ext_handle_decorator` | `(i32, i32) -> i32` | Handle a decorator application |
| `tsp_ext_emit` | `(i32, i32) -> i32` | Emit output from type graph JSON |
| `tsp_ext_emit_len` | `() -> i32` | Return emit output length |

And can import host functions from the `tsp` namespace:

| Import | Signature | Description |
|--------|-----------|-------------|
| `tsp.log` | `(ptr, len)` | Log a message |
| `tsp.state_set` | `(key_ptr, key_len, type_id, val_ptr, val_len)` | Set decorator state |
| `tsp.state_get` | `(key_ptr, key_len, type_id) -> i32` | Get state (returns length or -1) |
| `tsp.state_get_read` | `(buf_ptr)` | Copy last state_get result to guest |
| `tsp.state_add` | `(key_ptr, key_len, type_id)` | Add to state set |
| `tsp.state_has` | `(key_ptr, key_len, type_id) -> i32` | Check state (1/0) |
| `tsp.report_diagnostic` | `(severity, code_ptr, code_len, msg_ptr, msg_len)` | Report diagnostic |

### Manifest Format

The manifest JSON returned by `tsp_ext_manifest`:

```json
{
  "name": "my-extension",
  "version": "1.0.0",
  "formats": ["markdown", "custom"],
  "decorators": [
    {
      "name": "myDecorator",
      "namespace": "MyExt",
      "target_types": ["Model"],
      "parameters": [{ "name": "value", "type": "string" }]
    }
  ]
}
```

### Execution Flow

1. **Pre-parse**: Extension manifests are extracted; decorators registered on the Checker
2. **Parse + Check**: User source is parsed and type-checked (decorators resolved from registry)
3. **Decorator handling**: For each type with a matching decorator, `tsp_ext_handle_decorator` is called with `{ "type_id": N, "decorator_name": "...", "args": [...] }`
4. **Emit**: If the extension handles the requested format, `tsp_ext_emit` is called with the serialized type graph JSON

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
- **Emitter** (`emit/`) — Converts checked types to YAML, JSON, OpenAPI, or JSON Schema
- **CLI** (`crates/tspc/`) — Command-line interface with WASM extension support
- **Libs** (`libs/`) — Built-in library sources (HTTP, OpenAPI, etc.)

## Feature Coverage

What's ported from the [TypeSpec compiler](https://github.com/microsoft/typespec):

- Full scanner with doc comments, string templates, conflict markers
- Augment decorators (`@@doc`, `@@summary`, `@@returnsDoc`, `@@errorsDoc`)
  apply after declaration decorators, including member/`::parameters` targets
- `/** ... */` doc comments populate `@doc` on models, properties, scalars,
  enums/members, unions/variants, operations, interfaces, and namespaces;
  `@param`/`@prop`/`@returns`/`@errors` tags populate parameter, property,
  return, and error documentation, and explicit decorators take precedence
- Complete parser for all declaration types
- Type system with 25+ type kinds (Model, Interface, Enum, Union, Scalar, Template, etc.)
- Type relation/assignability checking
- Decorator application and validation
- Template declaration and instantiation
- Standard library types (string, int32, float64, utcDateTime, etc.)
- Helper libraries: HTTP types, status codes, content types, URI templates, OpenAPI/OpenAPI3/JSON Schema/protobuf/versioning type definitions
- `@typespec/http-client` shared SDK layer: `@experimental` with per-emitter scoping, client/sub-client discovery, operation-to-client mapping, data type collection, and client constructors (endpoint, server parameters, credential)
- External library injection API for custom decorator declarations
- Programmatic decorator registration (`Checker::register_decorator`) — bypasses source parsing, supports reserved keywords
- WASM extension system with wasmtime runtime — custom decorators and emitters
- CLI with cross-platform binary releases (UPX compressed)
- `fn` function declarations with `extern` modifier and call validation
- `internal` access control with project/library visibility scoping
- HTTP response handling: union variant flattening, plain body detection,
  response indexing, `@returnsDoc`/`@errorsDoc` precedence, wildcard error
  responses, and default status-code descriptions
- HTTP operation validation: duplicate parameter annotations/bodies,
  metadata ignored inside `@body`, content-type errors, implicit multipart,
  response cookies, status-code ranges, URI-template parameters and options,
  duplicate routes/operations, verb duplicates, shared-route consistency,
  and server URL parameters
- Multipart request bodies: `HttpPart<T, Options>` in tuple and model form,
  part names/headers/filenames/`multi`, default part content types, and
  `multipart-*` / `formdata-no-part-name` diagnostics
- `Http.File` models: file body resolution (content types, filename,
  contents, `isText`), derived-model validation, and structured-body
  fallback for explicit `Content-Type` headers or unions
- HTTP overloads: `@overload(base)` route/verb inheritance, bidirectional
  links, and endpoint-aware duplicate-route checks
- JSON Merge Patch: `MergePatchUpdate<T>` / `MergePatchCreateOrUpdate<T>`
  transforms resource models into optional, nullable property graphs
  (recursively through models, arrays, records, unions, and tuples), sets the
  `application/merge-patch+json` media type, records source links, and emits
  the `merge-patch-contains-null`, `merge-patch-contains-metadata`, and
  `merge-patch-content-type` diagnostics
- Linter rule options with JSON Schema validation and default value merging
- OpenAPI emitter with HTTP-protocol-aware verb/route/parameter resolution
- Protobuf proto3 `optional` label logic with array/map warnings
- ICE-protected diagnostic creation (fallback instead of panic)
- Templated alias member expression resolution with default parameter instantiation
- Public Checker helper API: decorator lookup, constraint accessors, type iteration, property walking, value extraction
- `@doc`/`@summary` fields on Model, ModelProperty, EnumMember, UnionVariant, Namespace — populated from decorator evaluation
- Standard decorator evaluation pipeline: `@doc`, `@summary`, `@minValue`, `@maxValue`, `@pattern`, `@format`, `@minLength`, `@maxLength`, `@minItems`, `@maxItems`, `@error`, `@tag`, `@discriminator`, `@encode`
- Official-compliant keyword handling: three-tier classification (active/reserved/modifier), reserved keywords allowed as property/enum/variant names, backtick-escaped identifiers (e.g. `` `chrome-devtools-mcp` ``)
- Auto decorators (`auto dec`): compiler-generated metadata storage without JS implementations, with `set_auto_decorator`/`has_auto_decorator`/`get_auto_decorator_value` accessors
- Compiler feature flags (`CompilerOptions.features`) with per-library scoping (e.g. `auto-decorators`, `function-declarations`, `union-extends`, `declaration-expressions`)
- Blockless file namespaces (`namespace Foo;`) with correct declaration scoping and `using` resolution rules
- `#suppress` directive tracking: unused-suppression and duplicate-suppression diagnostics
- Suppression inspection APIs (#12095): `collect_suppressions` on a parsed file (target, location, structural scope) and `Checker::get_suppressions` with observed usage (`used`)
- Diagnostic code short-name resolution (`@typespec/http/x` ↔ `http/x`, library aliases, ambiguity detection)
- Compilation stage tracking (`parsing` → `checking` → `validating` → `linting` → `emitting`) with stage-gated per-type caching (`use_cache`, HTTP operation resolution cache)
- Union `extends` base-type clause (`union-extends` feature) — variant assignability validation and circular-base detection, with no inheritance semantics
- Declarations as expressions (`model { ... }`, `union { ... }`, `enum { ... }`, `scalar ...`) in argument/return position (`declaration-expressions` feature)
- `@statusCode` numeric ranges: bounds inherited from the property type and base scalar chain, with per-property bounds narrowing the inherited range
- OpenAPI response descriptions preserve `@doc` from return-type union variants (with union-level fallback)
- `@useAuth` scheme resolution (`BasicAuth`, `BearerAuth`, `ApiKeyAuth`, `OAuth2Auth`, `OpenIdConnectAuth`, unions/tuples of schemes) and `@server` parameter models
- `@encode(string)` on boolean types; SPDX license `identifier` (mutually exclusive with `url`); OpenID Connect auth scopes

What's not yet ported:

- General `@@augment` execution for arbitrary custom decorators (documentation
  decorators are supported)
- Visibility-aware merge-patch mutator filtering (`@visibility`/lifecycle
  filtering during the transform)
- Full `Program` pipeline (multi-file compilation, import resolution)
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
| [`serde_json`](https://crates.io/crates/serde_json) | JSON Schema emitter output (optional, `json-schema` feature) |

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

This project is licensed under the [MIT License](LICENSE).

## Security

See [SECURITY.md](SECURITY.md) for vulnerability reporting guidelines.
