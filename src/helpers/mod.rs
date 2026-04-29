//! Helper utilities ported from TypeSpec compiler/src/core/helpers/
//!
//! Modules:
//! - path_interpolation: Path template variable interpolation
//! - syntax_utils: Identifier printing and text utilities
//! - string_template_utils: String template serializability checks
//! - operation_utils: Operation listing in containers
//! - usage_resolver: Type usage tracking (input/output)
//! - raw_text_cache: Raw text caching for AST nodes
//! - location_context: Location context resolution
//! - type_name_utils: Type name formatting
//! - discriminator_utils: Discriminated union resolution

pub mod discriminator_utils;
pub mod location_context;
pub mod operation_utils;
pub mod path_interpolation;
pub mod raw_text_cache;
pub mod string_template_utils;
pub mod syntax_utils;
pub mod type_name_utils;
pub mod usage_resolver;
