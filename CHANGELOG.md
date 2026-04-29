# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
