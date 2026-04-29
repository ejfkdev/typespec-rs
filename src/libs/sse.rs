//! @typespec/sse - Server-Sent Events Types
//!
//! Ported from TypeSpec packages/sse
//!
//! Provides types for Server-Sent Events (SSE):
//! - `@terminalEvent` - Mark a union variant as a terminal event
//! - `SSEStream<T>` - Model for SSE streams
//!
//! ## Diagnostics
//! - `terminal-event-not-in-events` - @terminalEvent on variant not in @events union
//! - `sse-stream-union-not-events` - SSEStream type param not decorated with @events
//!
//! ## Validation
//! - `checkForIncorrectlyAssignedTerminalEvents` - Validate terminal events are in @events union
//! - `checkForSSEStreamWithoutEventsDecorator` - Validate SSEStream uses @events-decorated union
//!
//! Depends on @typespec/http (HttpStream) and @typespec/events

use crate::diagnostics::{DiagnosticDefinition, DiagnosticMap};
#[cfg(test)]
use crate::state_accessors::StateAccessors;
use std::collections::HashMap;

/// Namespace for SSE types
pub const SSE_NAMESPACE: &str = "TypeSpec.SSE";

/// State key for @terminalEvent decorator
pub const STATE_TERMINAL_EVENT: &str = "TypeSpec.SSE.terminalEvent";

// ============================================================================
// Diagnostic codes
// ============================================================================

/// Diagnostic: @terminalEvent on variant not in @events union
pub const DIAG_TERMINAL_EVENT_NOT_IN_EVENTS: &str = "terminal-event-not-in-events";
/// Diagnostic: SSEStream type param not decorated with @events
pub const DIAG_SSE_STREAM_UNION_NOT_EVENTS: &str = "sse-stream-union-not-events";

// ============================================================================
// Library creation
// ============================================================================

/// Create the @typespec/sse library diagnostic map.
/// Ported from TS $lib definition in lib.ts.
pub fn create_sse_library() -> DiagnosticMap {
    HashMap::from([
        (
            DIAG_TERMINAL_EVENT_NOT_IN_EVENTS.to_string(),
            DiagnosticDefinition::error(
                "A field marked as '@terminalEvent' must be a member of a type decorated with '@TypeSpec.Events.events'.",
            ),
        ),
        (
            DIAG_SSE_STREAM_UNION_NOT_EVENTS.to_string(),
            DiagnosticDefinition::error(
                "SSEStream type parameter must be a union decorated with '@TypeSpec.Events.events'.",
            ),
        ),
    ])
}

// ============================================================================
// Decorator implementations
// ============================================================================

flag_decorator!(
    apply_terminal_event,
    is_terminal_event,
    STATE_TERMINAL_EVENT
);

// ============================================================================
// TSP Sources
// ============================================================================

/// The TypeSpec source for the SSE library decorators
pub const SSE_DECORATORS_TSP: &str = r#"
using TypeSpec.Reflection;

namespace TypeSpec.SSE;

/**
 * Indicates that the presence of this event is a terminal event,
 * and the client should disconnect from the server.
 */
extern dec terminalEvent(target: UnionVariant);
"#;

/// The TypeSpec source for the SSE library types
pub const SSE_TYPES_TSP: &str = r#"
import "@typespec/http/streams";

using Http.Streams;

namespace TypeSpec.SSE;

/**
 * Describes a stream of server-sent events.
 *
 * The content-type is set to `text/event-stream`.
 *
 * The server-sent events are described by `Type`.
 * The event type for any event can be defined by using named union variants.
 * When a union variant is not named, it is considered a 'message' event.
 *
 * @template Type The set of models describing the server-sent events.
 *
 * @example Mix of named union variants and terminal event
 *
 * ```typespec
 * model UserConnect {
 *   username: string;
 *   time: string;
 * }
 *
 * model UserMessage {
 *   username: string;
 *   time: string;
 *   text: string;
 * }
 *
 * model UserDisconnect {
 *   username: string;
 *   time: string;
 * }
 *
 * @TypeSpec.Events.events
 * union ChannelEvents {
 *   userconnect: UserConnect,
 *   usermessage: UserMessage,
 *   userdisconnect: UserDisconnect,
 *
 *   @Events.contentType("text/plain")
 *   @terminalEvent
 *   "[unsubscribe]",
 * }
 *
 * op subscribeToChannel(): SSEStream<ChannelEvents>;
 * ```
 */
@doc("")
model SSEStream<Type extends TypeSpec.Reflection.Union> is HttpStream<Type, "text/event-stream">;
"#;

/// The TypeSpec source for the SSE library main entry
pub const SSE_MAIN_TSP: &str = r#"
import "../dist/src/tsp-index.js";
import "./decorators.tsp";
import "./types.tsp";
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sse_namespace() {
        assert_eq!(SSE_NAMESPACE, "TypeSpec.SSE");
    }

    #[test]
    fn test_state_key() {
        assert!(STATE_TERMINAL_EVENT.starts_with("TypeSpec.SSE"));
    }

    #[test]
    fn test_create_sse_library() {
        let diags = create_sse_library();
        assert_eq!(diags.len(), 2);
        let codes: Vec<&str> = diags.keys().map(|code| code.as_str()).collect();
        assert!(codes.contains(&DIAG_TERMINAL_EVENT_NOT_IN_EVENTS));
        assert!(codes.contains(&DIAG_SSE_STREAM_UNION_NOT_EVENTS));
    }

    #[test]
    fn test_decorators_tsp_not_empty() {
        assert!(!SSE_DECORATORS_TSP.is_empty());
        assert!(SSE_DECORATORS_TSP.contains("terminalEvent"));
    }

    #[test]
    fn test_types_tsp_not_empty() {
        assert!(!SSE_TYPES_TSP.is_empty());
        assert!(SSE_TYPES_TSP.contains("SSEStream"));
        assert!(SSE_TYPES_TSP.contains("text/event-stream"));
    }

    #[test]
    fn test_is_terminal_event() {
        let mut state = StateAccessors::new();
        assert!(!is_terminal_event(&state, 1));

        apply_terminal_event(&mut state, 1);
        assert!(is_terminal_event(&state, 1));
        assert!(!is_terminal_event(&state, 2));
    }
}
