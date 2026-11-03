# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

Sync with upstream TypeSpec `a22c5330` (`@typespec/compiler` 1.17.0) and
dependency refresh.

### Added
- Suppression inspection APIs (#12095): `collect_suppressions` returns each
  `#suppress` directive with its target, source span, and structural scope
  without compiling; `Checker::get_suppressions` returns project suppressions
  with the observed `used` flag from the existing tracker
- `null` now resolves as a type in unions, property types, and aliases
- Literal values are checked against scalar constraints (`@minValue`,
  `@maxValue`, exclusive bounds, `@minLength`, `@maxLength`), and object
  literals reject properties the target model lacks (except through spreads
  or an indexer)

### Changed
- Dependencies updated: regex 1.13.1, bitflags 2.13.2, anyhow 1.0.104,
  clap 4.6.7, serde 1.0.229, serde_json 1.0.154, wasmtime 47.0.4.
  wasmtime 48+ requires Rust 1.97, so the MSRV stays at Rust 1.95
  (`rust-toolchain.toml` and both crates pin 1.95).
- CI uses `actions/checkout@v7`.

### Fixed
- `#suppress`/`#deprecated` directives followed by a newline no longer consume
  the next model property or operation parameter name as a directive argument
- Rustdoc warnings for `Box<str>` and a redundant link target

## [0.5.7] - 2026-10-31

Sync with upstream TypeSpec through `fc7cf891`, port the shared
`@typespec/http-client` SDK layer used by the official Java, TypeScript,
Python, and C# client generators, and fix several compiler defects found
while validating those flows.

### Added
- Upstream compiler sync: union `extends` clauses (#11771,
  `union-extends`), declarations as expressions (#11019,
  `declaration-expressions`), array-spread validation (#11559), decorators on
  template-referenced concrete types (#12358), deprecated-type warning
  suppression inside deprecated operations (#12314), `op::parameters` type
  names (#11551), recursive-type assignability fix (#11779), `@encodedName`
  on enum members (#11980), enum member navigation (#12180)
- `@typespec/http-client` shared SDK layer: `@experimental` with per-emitter
  scoping and lifecycle queries, client/sub-client discovery, operation to
  client mapping, data type collection, and client constructors (endpoint,
  `@server` parameters, credential from `@useAuth`)
- Client typekit kits (`client`, `client_library`) including `listModels`,
  `listEnums`, and `listDataTypes`
- `@useAuth` scheme resolution for basic/bearer/api-key/OAuth2/OpenID Connect,
  including unions and tuples of schemes; upstream `auth.tsp` enums and OAuth2
  flow models
- `@server` parameter models are retained for client constructor generation
- `@statusCode` numeric ranges inherit bounds from the scalar chain and can be
  narrowed per property (#12118)
- Union variant `@doc` is preserved in response descriptions (#11785);
  object members named like `Object.prototype` keys are kept (#11744);
  tuple schemas emit `minItems`/`maxItems` (#11893); protobuf package names
  use the anchored identifier rule (#11960)
- CLI: `--features` flag to enable compiler features, richer `--help` with
  examples, and the `TypeSpec.HttpClient` library preloaded
- OpenAPI emitter: walks project namespaces, emits `@body`/`@bodyRoot` as
  `requestBody`, skips request-side metadata, and omits envelope metadata
  (including status-code-only envelopes) from response bodies

### Changed
- `@service` now stores the compiler service state (title), so
  `isService`/`listServices` work; the HTTP marker is kept for existing
  consumers
- JSON/YAML emitters only emit project declarations; injected library types
  are no longer included in output
- Diagnostics for library decorators no longer report
  `missing-implementation`; host-provided implementations are assumed

### Fixed
- Empty string literals (`""`) were misdetected as triple-quoted strings,
  breaking any declaration containing one
- Types resolved through `using` bypassed template instantiation
- Merged blockless namespaces only checked the first declaration segment
- The standard `TypeSpec` namespace was not attached to the global namespace,
  so registering `TypeSpec.*` decorators shadowed the standard library
- `library_line_offset` was off by one when an injected library did not end
  with a newline
- Name resolution inside dotted blockless namespaces did not try enclosing
  prefixes
- Decorator declaration targets resolved the `FunctionParameter` node instead
  of its type annotation, and union targets forced a single-kind constraint

## [0.5.6] - 2026-08-12

Sync with upstream TypeSpec through v1.13 and post-v1.13 changes
(microsoft/typespec@cf390ec8). Also backfills four upstream changes that
landed just before v1.13 (#10684, #10826, #10855, #10880).

### Added
- Auto decorators (#10197): `auto dec` declarations store their arguments
  without any JS implementation; `declaration_kind` on decorator types;
  `set_auto_decorator` / `has_auto_decorator` / `get_auto_decorator_value` /
  `get_auto_decorator_targets` accessors; duplicate application warns
  (last-write-wins); gated behind the `auto-decorators` compiler feature
- Compiler feature flags (#10826, #11235): `CompilerOptions.features`,
  per-library feature scoping so a library can enable features for its own
  code independently of the consuming project
- Suppression tracking (#10805, #11113): unused-suppression warnings and
  duplicate-`#suppress` diagnostics
- Diagnostic code short names (#11209): `DiagnosticCodeResolver` resolves
  scope-stripped/aliased codes (e.g. `http/no-foo` → `@typespec/http/no-foo`)
  and detects ambiguous short names
- Blockless file namespaces (#11552): `namespace Foo;` scopes all following
  top-level declarations; `using` declared before the file namespace resolves
  from the global namespace only
- Compilation stages and caching (#11318): `CompilationStage` tracking
  (parsing → checking → validating → linting → emitting) and stage-gated
  `use_cache`; HTTP operation resolution is cached at the program level
- `@encode(string)` on boolean types with case-insensitive `true|false`
  semantics (#10875)
- OpenAPI `License.identifier` (SPDX expression, mutually exclusive with
  `url`) (#11309)
- OpenID Connect auth `scopes` (#11153)
- `Checker::with_options` constructor; shared test helpers
  (`check_with_features`, `check_with_library`)

### Changed
- `auto` promoted from reserved keyword to modifier keyword (#10197)
- `internal` modifier is no longer experimental (#10855)
- `using` targets are bound in the declaration context instead of being
  re-resolved at each reference site (#11552)
- Template parameter usage now counted in constraints/defaults of other
  parameters; operation template parameter defaults are checked inside the
  template declaration scope (#11477)
- Function call interpolation of template parameters in string templates no
  longer reports spurious diagnostics (#11056)
- Spread cycles between models are detected; recursive aliases through model
  expressions no longer over-report (#10684)
- Function rest arguments are validated against the rest parameter
  constraint; argument-count diagnostics target the call expression (#10880)
- Diagnostic targets resolve through value entities (#10921)

### Fixed
- Stale e2e expectation for template models and non-compiling doctests
- Zero clippy warnings across all targets (including new-clippy findings)

## [0.1.0] - 2026-04-28

### Added
- Complete TypeSpec scanner/lexer with doc comments, string templates, and conflict markers
- Full parser for all TypeSpec declaration types
- Type checker with 25+ type kinds (Model, Interface, Enum, Union, Scalar, Template, etc.)
- Type relation/assignability checking
- Decorator application and validation
- Template declaration and instantiation
- Standard library types (string, int32, float64, utcDateTime, etc.)
- Helper libraries: HTTP types, status codes, content types, URI templates
- JSON and YAML emitters
- OpenAPI 3 emitter (placeholder)
- `tspc` CLI tool for command-line TypeSpec compilation
- 2,800+ tests

### Dependencies
- `regex` for pattern matching in scanner
- `bitflags` for visibility and symbol flags
- `clap` for CLI argument parsing (tspc only)
- `serde`/`serde_json` for JSON serialization (tspc only)
