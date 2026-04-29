//! TypeSpec Emit API
//!
//! Provides a simple and powerful API for converting TypeSpec source code to YAML/JSON.
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
//!     }
//! "#;
//!
//! // Convert to YAML
//! let result = to_yaml(src).unwrap();
//! println!("{}", result.output);
//!
//! // Convert to JSON
//! let result = to_json(src).unwrap();
//! println!("{}", result.output);
//! ```

pub mod emitter;
pub mod json_emitter;
pub mod openapi_emitter;
pub mod yaml_emitter;

pub use emitter::Emitter;
pub use json_emitter::JsonEmitter;
pub use openapi_emitter::OpenAPIEmitter;
pub use yaml_emitter::YamlEmitter;

use crate::parser::parse;

// ============================================================================
// EmitResult - Conversion Result Type
// ============================================================================

/// Result of a TypeSpec emission/conversion
#[derive(Debug)]
pub struct EmitResult {
    /// The formatted output string (YAML/JSON/etc)
    pub output: String,
    /// The format of the output ("yaml", "json", etc)
    pub format: String,
    /// Number of diagnostics (errors/warnings)
    pub diagnostics_count: usize,
}

impl EmitResult {
    pub(crate) fn new(output: String, format: &str, diagnostics_count: usize) -> Self {
        Self {
            output,
            format: format.to_string(),
            diagnostics_count,
        }
    }
}

impl EmitResult {
    /// Check if there are any errors
    pub fn has_errors(&self) -> bool {
        self.diagnostics_count > 0
    }
}

// ============================================================================
// Basic Conversion Functions
// ============================================================================

/// Convert TypeSpec source to YAML
///
/// # Example
/// ```ignore
/// use typespec_rs::emit::to_yaml;
///
/// let src = r#"
///     model Pet {
///         id: string;
///         name: string;
///     }
/// "#;
///
/// match to_yaml(src) {
///     Ok(result) => println!("{}", result.output),
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
pub fn to_yaml(src: &str) -> Result<EmitResult, String> {
    let parse_result = parse(src);
    let emitter = YamlEmitter::new();
    emitter.emit(&parse_result)
}

/// Convert TypeSpec source to JSON
///
/// # Example
/// ```ignore
/// use typespec_rs::emit::to_json;
///
/// let src = r#"
///     model Pet {
///         id: string;
///         name: string;
///     }
/// "#;
///
/// match to_json(src) {
///     Ok(result) => println!("{}", result.output),
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
pub fn to_json(src: &str) -> Result<EmitResult, String> {
    let parse_result = parse(src);
    let emitter = JsonEmitter::new();
    emitter.emit(&parse_result)
}

/// Convert TypeSpec source with a custom emitter
///
/// # Example
/// ```ignore
/// use typespec_rs::emit::{convert, YamlEmitter};
///
/// let src = r#"
///     model Pet {
///         id: string;
///         name: string;
///     }
/// "#;
///
/// let result = convert(src, YamlEmitter::new()).unwrap();
/// println!("{}", result.output);
/// ```
pub fn convert(src: &str, emitter: impl Emitter) -> Result<EmitResult, String> {
    let parse_result = parse(src);
    emitter.emit(&parse_result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_yaml_simple() {
        let src = r#"
            model Pet {
                id: string;
                name: string;
            }
        "#;

        let result = to_yaml(src).unwrap();
        assert!(!result.output.is_empty());
        assert_eq!(result.format, "yaml");
        assert!(!result.has_errors());
    }

    #[test]
    fn test_to_json_simple() {
        let src = r#"
            model Pet {
                id: string;
                name: string;
            }
        "#;

        let result = to_json(src).unwrap();
        assert!(!result.output.is_empty());
        assert_eq!(result.format, "json");
        assert!(!result.has_errors());
    }

    #[test]
    fn test_convert_with_emitter() {
        let src = r#"
            model Pet {
                id: string;
                name: string;
            }
        "#;

        let result = convert(src, YamlEmitter::new()).unwrap();
        assert!(!result.output.is_empty());
        assert_eq!(result.format, "yaml");
    }

    #[test]
    fn test_yaml_contains_model() {
        let src = r#"
            model Pet {
                name: string;
            }
        "#;

        let result = to_yaml(src).unwrap();
        assert!(result.output.contains("Pet") || result.output.contains("model"));
    }

    #[test]
    fn test_json_contains_model() {
        let src = r#"
            model Pet {
                name: string;
            }
        "#;

        let result = to_json(src).unwrap();
        assert!(result.output.contains("Pet") || result.output.contains("model"));
    }
}
