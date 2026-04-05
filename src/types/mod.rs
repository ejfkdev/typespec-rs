//! Type system for TypeSpec-Rust
//! Ported from TypeSpec compiler/src/core/types.ts

pub mod builder;
pub mod checker;
pub mod decorator;
pub mod r#enum;
pub mod interface;
pub mod model;
pub mod namespace;
pub mod operation;
pub mod primitive;
pub mod union;

pub use builder::*;
pub use checker::*;
pub use decorator::*;
pub use r#enum::*;
pub use interface::*;
pub use model::*;
pub use namespace::*;
pub use operation::*;
pub use primitive::*;
pub use union::*;
