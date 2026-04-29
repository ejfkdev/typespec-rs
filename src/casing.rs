//! Casing utilities for TypeSpec-Rust
//!
//! This module provides string casing utilities.

/// Capitalize the first character of a string
pub fn capitalize<S: AsRef<str>>(s: S) -> String {
    let s = s.as_ref();
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(chars).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capitalize_hello_world() {
        assert_eq!(capitalize("hello world"), "Hello world");
    }

    #[test]
    fn test_capitalize_empty() {
        assert_eq!(capitalize(""), "");
    }

    #[test]
    fn test_capitalize_single_char() {
        assert_eq!(capitalize("a"), "A");
    }

    #[test]
    fn test_capitalize_already_uppercase() {
        assert_eq!(capitalize("Hello"), "Hello");
    }

    #[test]
    fn test_capitalize_with_numbers() {
        assert_eq!(capitalize("123abc"), "123abc");
    }
}
