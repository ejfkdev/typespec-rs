//! TypeKit kits — Type-specific operation modules
//!
//! Each module provides operations for a specific TypeSpec type kind.

#[macro_use]
mod macros;

pub mod array;
pub mod builtin;
pub mod entity;
pub mod enum_member;
pub mod enum_type;
pub mod intrinsic;
pub mod literal;
pub mod model;
pub mod model_property;
pub mod operation;
pub mod record;
pub mod scalar;
pub mod tuple;
pub mod type_kind;
pub mod union;
pub mod union_variant;
pub mod value;
