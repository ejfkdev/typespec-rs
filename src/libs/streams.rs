//! @typespec/streams - Stream Protocol Types
//!
//! Ported from TypeSpec packages/streams
//!
//! Provides the `@streamOf` decorator and `Stream<T>` model for
//! describing stream protocol types in TypeSpec.
//!
//! ## Decorators
//! - `@streamOf(Type)` - Mark a model as representing a stream protocol type
//!
//! ## Types
//! - `Stream<T>` - Generic stream model with `@streamOf(Type)` applied
//!
//! ## Helper Functions
//! - `isStream(program, model)` - Check if a model is a stream
//! - `getStreamOf(program, model)` - Get the stream data type

use crate::checker::types::TypeId;
use crate::diagnostics::DiagnosticMap;
use crate::state_accessors::StateAccessors;

/// Namespace for streams types
pub const STREAMS_NAMESPACE: &str = "TypeSpec.Streams";

/// State key for @streamOf decorator
pub const STATE_STREAM_OF: &str = "TypeSpec.Streams.streamOf";

/// Create the @typespec/streams library definition with diagnostics
pub fn create_streams_library() -> DiagnosticMap {
    DiagnosticMap::new()
}

// ============================================================================
// Decorator implementations
// ============================================================================

typeid_decorator!(apply_stream_of, get_stream_of, STATE_STREAM_OF);

/// Check if a model is a stream (has @streamOf applied).
/// Ported from TS isStream().
pub fn is_stream(state: &StateAccessors, target: TypeId) -> bool {
    state.get_state(STATE_STREAM_OF, target).is_some()
}

/// The TypeSpec source for the streams library decorators
pub const STREAMS_DECORATORS_TSP: &str = r#"
using TypeSpec.Reflection;

namespace TypeSpec.Streams;

/**
 * Specify that a model represents a stream protocol type whose data is described
 * by `Type`.
 *
 * @param type The type that models the underlying data of the stream.
 *
 * @example
 *
 * ```typespec
 * model Message {
 *   id: string;
 *   text: string;
 * }
 *
 * @streamOf(Message)
 * model Response {
 *   @body body: string;
 * }
 * ```
 */
extern dec streamOf(target: Model, type: unknown);
"#;

/// The TypeSpec source for the streams library types
pub const STREAMS_TYPES_TSP: &str = r#"
namespace TypeSpec.Streams;

/**
 * Defines a model that represents a stream protocol type whose data is described
 * by `Type`.
 *
 * This can be useful when the underlying data type is not relevant, or to serve as
 * a base type for custom streams.
 *
 * @template Type The type of the stream's data.
 */
@doc("")
@streamOf(Type)
model Stream<Type> {}
"#;

/// Combined TSP source for the streams library (main.tsp)
pub const STREAMS_MAIN_TSP: &str = r#"
import "../dist/src/tsp-index.js";
import "./decorators.tsp";
import "./types.tsp";
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_streams_namespace() {
        assert_eq!(STREAMS_NAMESPACE, "TypeSpec.Streams");
    }

    #[test]
    fn test_state_key() {
        assert!(STATE_STREAM_OF.starts_with("TypeSpec.Streams"));
    }

    #[test]
    fn test_create_streams_library() {
        let lib = create_streams_library();
        assert!(lib.is_empty(), "streams library has no diagnostics");
    }

    #[test]
    fn test_decorators_tsp_not_empty() {
        assert!(!STREAMS_DECORATORS_TSP.is_empty());
        assert!(STREAMS_DECORATORS_TSP.contains("streamOf"));
    }

    #[test]
    fn test_types_tsp_not_empty() {
        assert!(!STREAMS_TYPES_TSP.is_empty());
        assert!(STREAMS_TYPES_TSP.contains("Stream"));
    }

    #[test]
    fn test_is_stream_and_get_stream_of() {
        let mut state = StateAccessors::new();

        // Initially not a stream
        assert!(!is_stream(&state, 1));
        assert_eq!(get_stream_of(&state, 1), None);

        // Apply @streamOf
        apply_stream_of(&mut state, 1, 42);
        assert!(is_stream(&state, 1));
        assert_eq!(get_stream_of(&state, 1), Some(42));

        // Different model not affected
        assert!(!is_stream(&state, 2));
    }

    #[test]
    fn test_apply_stream_of_overwrite() {
        let mut state = StateAccessors::new();
        apply_stream_of(&mut state, 1, 10);
        assert_eq!(get_stream_of(&state, 1), Some(10));

        apply_stream_of(&mut state, 1, 20);
        assert_eq!(get_stream_of(&state, 1), Some(20));
    }
}
