//! TypeSpec Library Modules
//!
//! This module contains the TypeSpec standard library extensions,
//! ported from the TypeScript packages under @typespec/*.
//!
//! Each sub-module represents a library that provides:
//! - .tsp type definitions (decorator declarations, models, scalars, etc.)
//! - Rust decorator implementations
//! - Validation logic
//! - Diagnostic definitions
//!
//! Currently ported libraries:
//! - `compiler` - TypeSpec std decorators (@doc, @tag, @key, @error, @format, paging, examples, etc.)
//! - `http` - @typespec/http: HTTP protocol decorators and types
//! - `xml` - @typespec/xml: XML serialization decorators and types
//! - `streams` - @typespec/streams: Stream protocol types
//! - `sse` - @typespec/sse: Server-Sent Events types
//! - `events` - @typespec/events: Event system decorators
//! - `rest` - @typespec/rest: REST API decorators and resource types
//! - `protobuf` - @typespec/protobuf: Protocol Buffers types and decorators
//! - `openapi` - @typespec/openapi: OpenAPI annotations and types
//! - `versioning` - @typespec/versioning: API versioning decorators
//!
//! Utility modules:
//! - `uri_template` - RFC 6570 URI template parser
//! - `status_codes` - HTTP status code validation
//! - `content_types` - HTTP content type resolution
//! - `json_schema` - @typespec/json-schema: JSON Schema decorators and types
//! - `openapi3` - @typespec/openapi3: OpenAPI 3.x emitter decorators and types

#[macro_use]
pub mod decorator_macros;
pub mod compiler;
pub mod content_types;
pub mod events;
pub mod http;
pub mod json_schema;
pub mod openapi;
pub mod openapi3;
pub mod protobuf;
pub mod rest;
pub mod sse;
pub mod status_codes;
pub mod streams;
pub mod uri_template;
pub mod versioning;
pub mod xml;
