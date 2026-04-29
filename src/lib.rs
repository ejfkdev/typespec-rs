//! # typespec-rs
//!
//! A Rust implementation of the [TypeSpec](https://typespec.io/) type system —
//! parser, type checker, and emitter.
//!
//! This is an **independent Rust port** of the TypeSpec compiler, not a binding
//! to the TypeScript compiler. It implements the parser, checker, and emitter
//! pipeline natively in Rust.
//!
//! # Quick Start
//!
//! ```ignore
//! use typespec_rs::emit::{to_yaml, to_json};
//!
//! let src = r#"
//!     model Pet {
//!         id: string;
//!         name: string;
//!         age?: int32;
//!     }
//! "#;
//!
//! let yaml = to_yaml(src).unwrap();
//! let json = to_json(src).unwrap();
//! ```
//!
//! # Module Organization
//!
//! - **[`emit`]** — Convert TypeSpec to YAML/JSON output (primary user-facing API)
//! - **[`parser`]** — Parse TypeSpec source into AST
//! - **[`checker`]** — Type checking and semantic analysis
//! - **[`ast`]** — AST node types
//!
//! Internal modules (`#[doc(hidden)]`) are used by the compiler pipeline
//! but are not part of the stable public API.
//!
//! # Status
//!
//! This project is in early development. The scanner, parser, and core type checker
//! are functional. See the [repository](https://github.com/ejfkdev/typespec-rs) for
//! the latest status.

// Primary user-facing modules
pub mod ast;
pub mod checker;
pub mod emit;
pub mod parser;

// Secondary public modules (used by tests/integrations)
pub mod code_fixes;
pub mod diagnostics;
pub mod mime_type;
pub mod param_message;
pub mod path_utils;
pub mod perf;
pub mod source_file;
pub mod utils;

// Internal modules — not part of the stable public API
#[doc(hidden)]
pub mod casing;
#[doc(hidden)]
pub mod charcode;
#[doc(hidden)]
pub mod deprecation;
#[doc(hidden)]
pub mod helpers;
#[doc(hidden)]
pub mod intrinsic_type_state;
#[doc(hidden)]
pub mod libs;
#[doc(hidden)]
pub mod linter;
#[doc(hidden)]
pub mod loader;
#[doc(hidden)]
pub mod modifiers;
#[doc(hidden)]
pub mod numeric;
#[doc(hidden)]
pub mod numeric_ranges;
#[doc(hidden)]
pub mod program;
#[doc(hidden)]
pub mod resolver;
#[doc(hidden)]
pub mod scanner;
#[doc(hidden)]
pub mod semantic_walker;
#[doc(hidden)]
pub mod state_accessors;
#[doc(hidden)]
pub mod stats;
#[doc(hidden)]
pub mod std;
#[doc(hidden)]
pub mod text_utils;
#[doc(hidden)]
pub mod typekit;
#[doc(hidden)]
pub mod visibility;
