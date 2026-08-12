# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
