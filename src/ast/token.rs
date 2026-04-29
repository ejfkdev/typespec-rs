//! Re-exports of token types from the scanner module.
//!
//! These types are defined in `scanner/lexer.rs` and re-exported here
//! for convenience by AST and parser modules.

pub use crate::scanner::{Position, Span};
