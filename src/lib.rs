//! TypeSpec-Rust: A Rust implementation of the TypeSpec type system
//!
//! This crate provides types, parsing, and type checking for TypeSpec.

pub mod ast;
pub mod checker;
pub mod parser;
pub mod program;
pub mod resolver;
pub mod scanner;
pub mod std;
pub mod types;
