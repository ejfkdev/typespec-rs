//! Macros for generating repetitive decorator accessor patterns
//!
//! These macros eliminate boilerplate for common decorator patterns:
//! - `flag_decorator!` — apply/is pair (no value, just a flag)
//! - `string_decorator!` — apply/get pair (single string value)
//! - `optional_name_decorator!` — apply/is/get_name triple (optional name)
//! - `numeric_decorator!` — apply/get pair (numeric value, stored as string)
//! - `typeid_decorator!` — apply/get pair (TypeId value, stored as string)
//! - `visibility_decorator!` — apply/get pair (comma-separated visibility modifiers)
//! - `string_enum!` — enum with as_str()/parse_str() methods

/// Generate a flag decorator (apply + is pair).
///
/// Creates `apply_$name` which adds the target to state, and `is_$name` which checks state.
///
/// # Example
/// ```
/// flag_decorator!(apply_error, is_error, "TypeSpec.error");
/// ```
#[macro_export]
macro_rules! flag_decorator {
    ($apply:ident, $is:ident, $state_key:expr) => {
        /// Apply decorator (flag - no value).
        pub fn $apply(
            state: &mut $crate::state_accessors::StateAccessors,
            target: $crate::checker::types::TypeId,
        ) {
            state.add_to_state($state_key, target);
        }

        /// Check if decorator is applied.
        pub fn $is(
            state: &$crate::state_accessors::StateAccessors,
            target: $crate::checker::types::TypeId,
        ) -> bool {
            state.has_state($state_key, target)
        }
    };
}

/// Generate a string decorator (apply + get pair).
///
/// Creates `apply_$name` which sets a string value, and `get_$name` which retrieves it.
///
/// # Example
/// ```
/// string_decorator!(apply_summary, get_summary, "TypeSpec.summary");
/// ```
#[macro_export]
macro_rules! string_decorator {
    ($apply:ident, $get:ident, $state_key:expr) => {
        /// Apply decorator (string value).
        pub fn $apply(
            state: &mut $crate::state_accessors::StateAccessors,
            target: $crate::checker::types::TypeId,
            value: &str,
        ) {
            state.set_state($state_key, target, value.to_string());
        }

        /// Get decorator value.
        pub fn $get(
            state: &$crate::state_accessors::StateAccessors,
            target: $crate::checker::types::TypeId,
        ) -> Option<String> {
            state.get_state($state_key, target).map(|s| s.to_string())
        }
    };
}

/// Generate an optional-name decorator (apply + is + get_name triple).
///
/// Creates:
/// - `apply_$name` — stores optional name (empty string if None)
/// - `is_$name` — checks if decorator is applied
/// - `get_$name` — retrieves the name (None if empty or not set)
///
/// # Example
/// ```
/// optional_name_decorator!(apply_key, is_key, get_key_name, "TypeSpec.key");
/// ```
#[macro_export]
macro_rules! optional_name_decorator {
    ($apply:ident, $is:ident, $get_name:ident, $state_key:expr) => {
        /// Apply decorator (optional name).
        pub fn $apply(
            state: &mut $crate::state_accessors::StateAccessors,
            target: $crate::checker::types::TypeId,
            alt_name: Option<&str>,
        ) {
            state.set_state($state_key, target, alt_name.unwrap_or("").to_string());
        }

        /// Check if decorator is applied.
        pub fn $is(
            state: &$crate::state_accessors::StateAccessors,
            target: $crate::checker::types::TypeId,
        ) -> bool {
            state.has_state_value($state_key, target)
        }

        /// Get the optional name value.
        pub fn $get_name(
            state: &$crate::state_accessors::StateAccessors,
            target: $crate::checker::types::TypeId,
        ) -> Option<String> {
            state
                .get_state($state_key, target)
                .map(|s| s.to_string())
                .filter(|s| !s.is_empty())
        }
    };
}

/// Generate a numeric decorator (apply + get pair).
///
/// Creates `apply_$name` which stores a numeric value as string, and `get_$name` which parses it back.
///
/// # Example
/// ```
/// numeric_decorator!(apply_min_items, get_min_items, "TypeSpec.minItems", i64);
/// ```
#[macro_export]
macro_rules! numeric_decorator {
    ($apply:ident, $get:ident, $state_key:expr, $ty:ty) => {
        /// Apply decorator (numeric value).
        pub fn $apply(
            state: &mut $crate::state_accessors::StateAccessors,
            target: $crate::checker::types::TypeId,
            value: $ty,
        ) {
            state.set_state($state_key, target, value.to_string());
        }

        /// Get decorator value.
        pub fn $get(
            state: &$crate::state_accessors::StateAccessors,
            target: $crate::checker::types::TypeId,
        ) -> Option<$ty> {
            state
                .get_state($state_key, target)
                .and_then(|s| s.parse::<$ty>().ok())
        }
    };
}

/// Generate a TypeId decorator (apply + get pair).
///
/// Creates `apply_$name` which stores a TypeId as string, and `get_$name` which parses it back.
///
/// # Example
/// ```
/// typeid_decorator!(apply_overload, get_overload, "TypeSpec.overload");
/// ```
#[macro_export]
macro_rules! typeid_decorator {
    ($apply:ident, $get:ident, $state_key:expr) => {
        /// Apply decorator (TypeId value).
        pub fn $apply(
            state: &mut $crate::state_accessors::StateAccessors,
            target: $crate::checker::types::TypeId,
            value: $crate::checker::types::TypeId,
        ) {
            state.set_state($state_key, target, value.to_string());
        }

        /// Get decorator value.
        pub fn $get(
            state: &$crate::state_accessors::StateAccessors,
            target: $crate::checker::types::TypeId,
        ) -> Option<$crate::checker::types::TypeId> {
            state
                .get_state($state_key, target)
                .and_then(|s| s.parse::<$crate::checker::types::TypeId>().ok())
        }
    };
}

/// Generate a visibility decorator (apply + get pair for comma-separated modifiers).
///
/// Creates `apply_$name` which joins and stores modifiers, and `get_$name` which splits them back.
///
/// # Example
/// ```
/// visibility_decorator!(apply_visibility, get_visibility, "TypeSpec.visibility", get_comma_list);
/// ```
#[macro_export]
macro_rules! visibility_decorator {
    ($apply:ident, $get:ident, $state_key:expr, $get_list_fn:ident) => {
        /// Apply decorator (comma-separated visibility modifiers).
        pub fn $apply(
            state: &mut $crate::state_accessors::StateAccessors,
            target: $crate::checker::types::TypeId,
            modifiers: &[&str],
        ) {
            state.set_state($state_key, target, modifiers.join(","));
        }

        /// Get decorator value as list of strings.
        pub fn $get(
            state: &$crate::state_accessors::StateAccessors,
            target: $crate::checker::types::TypeId,
        ) -> Vec<String> {
            $get_list_fn(state, $state_key, target)
        }
    };
}

/// Generate an enum with `as_str()` and `parse_str()` methods.
///
/// This eliminates the boilerplate of writing identical match arms for
/// string conversion and parsing across many enum types.
///
/// Supports optional doc comments on the enum and variants, plus optional
/// extra derive attributes.
///
/// # Example
/// ```
/// string_enum! {
///     /// File type for output
///     pub enum FileType {
///         Json => "json",
///         Yaml => "yaml",
///     }
/// }
///
/// string_enum! {
///     #[derive(PartialOrd, Ord)]
///     pub enum Version {
///         V1 => "1.0",
///         V2 => "2.0",
///     }
/// }
///
/// string_enum! {
///     pub enum Mode {
///         /// Read-only mode
///         Read => "read",
///         /// Write-only mode
///         Write => "write",
///     }
/// }
/// ```
#[macro_export]
macro_rules! string_enum {
    // With optional doc comments and extra derive attributes
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident => $str:expr
            ),+ $(,)?
        }
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        $(#[$meta])*
        $vis enum $name {
            $(
                $(#[$variant_meta])*
                $variant
            ),+
        }

        impl $name {
            /// Get the string representation.
            pub fn as_str(&self) -> &'static str {
                match self {
                    $(Self::$variant => $str),+
                }
            }

            /// Parse from string value.
            pub fn parse_str(s: &str) -> Option<Self> {
                match s {
                    $($str => Some(Self::$variant),)+
                    _ => None,
                }
            }
        }
    };
}
