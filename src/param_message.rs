//! Param message utilities for TypeSpec-Rust
//!
//! Ported from TypeSpec compiler/src/core/param-message.ts
//!
//! This module provides template string interpolation utilities.

use std::collections::HashMap;

/// A callable message template with key placeholders
#[derive(Debug, Clone)]
pub struct CallableMessage {
    /// The string parts between interpolations
    strings: Vec<String>,
    /// The keys for each interpolation
    keys: Vec<String>,
}

impl CallableMessage {
    /// Create a new callable message from string parts and keys
    pub fn new(strings: Vec<String>, keys: Vec<String>) -> Self {
        Self { strings, keys }
    }

    /// Invoke the message with the given values
    pub fn invoke(&self, values: &HashMap<String, String>) -> String {
        let mut result = self.strings[0].clone();
        for (i, key) in self.keys.iter().enumerate() {
            if let Some(value) = values.get(key) {
                result.push_str(value);
            }
            // strings[i + 1] is the part after this interpolation
            if i + 1 < self.strings.len() {
                result.push_str(&self.strings[i + 1]);
            }
        }
        result
    }

    /// Get the keys for this message
    pub fn keys(&self) -> &[String] {
        &self.keys
    }
}

/// Param message template function
pub struct ParamMessage;

impl ParamMessage {
    /// Create a message with 1 interpolation
    pub fn m1(s1: &str, k0: &str, s2: &str) -> CallableMessage {
        CallableMessage::new(vec![s1.to_string(), s2.to_string()], vec![k0.to_string()])
    }

    /// Create a message with 2 interpolations
    pub fn m2(s1: &str, k0: &str, s2: &str, k1: &str, s3: &str) -> CallableMessage {
        CallableMessage::new(
            vec![s1.to_string(), s2.to_string(), s3.to_string()],
            vec![k0.to_string(), k1.to_string()],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_param_message_single_param_middle() {
        // "My name is ${"name"}."
        let msg = CallableMessage::new(
            vec!["My name is ".to_string(), ".".to_string()],
            vec!["name".to_string()],
        );
        let mut values = HashMap::new();
        values.insert("name".to_string(), "Foo".to_string());
        assert_eq!(msg.invoke(&values), "My name is Foo.");
    }

    #[test]
    fn test_param_message_single_param_start() {
        // "${"name"} is my name."
        let msg = CallableMessage::new(
            vec!["".to_string(), " is my name.".to_string()],
            vec!["name".to_string()],
        );
        let mut values = HashMap::new();
        values.insert("name".to_string(), "Foo".to_string());
        assert_eq!(msg.invoke(&values), "Foo is my name.");
    }

    #[test]
    fn test_param_message_single_param_end() {
        // "My name: ${"name"}"
        let msg = CallableMessage::new(
            vec!["My name: ".to_string(), "".to_string()],
            vec!["name".to_string()],
        );
        let mut values = HashMap::new();
        values.insert("name".to_string(), "Foo".to_string());
        assert_eq!(msg.invoke(&values), "My name: Foo");
    }

    #[test]
    fn test_param_message_multiple_params() {
        // "My name is ${"name"} and my age is ${"age"}."
        let msg = CallableMessage::new(
            vec![
                "My name is ".to_string(),
                " and my age is ".to_string(),
                ".".to_string(),
            ],
            vec!["name".to_string(), "age".to_string()],
        );
        let mut values = HashMap::new();
        values.insert("name".to_string(), "Foo".to_string());
        values.insert("age".to_string(), "34".to_string());
        assert_eq!(msg.invoke(&values), "My name is Foo and my age is 34.");
    }

    #[test]
    fn test_param_message_adjacent_params() {
        // "My username is ${"name"}${"age"}."
        let msg = CallableMessage::new(
            vec![
                "My username is ".to_string(),
                "".to_string(),
                ".".to_string(),
            ],
            vec!["name".to_string(), "age".to_string()],
        );
        let mut values = HashMap::new();
        values.insert("name".to_string(), "Foo".to_string());
        values.insert("age".to_string(), "34".to_string());
        assert_eq!(msg.invoke(&values), "My username is Foo34.");
    }

    #[test]
    fn test_param_message_missing_value() {
        // "Hello ${"name"}!"
        let msg = CallableMessage::new(
            vec!["Hello ".to_string(), "!".to_string()],
            vec!["name".to_string()],
        );
        let mut values = HashMap::new();
        values.insert("other".to_string(), "Foo".to_string());
        // Missing key - nothing is pushed
        assert_eq!(msg.invoke(&values), "Hello !");
    }

    #[test]
    fn test_param_message_keys() {
        let msg = CallableMessage::new(
            vec!["Test ".to_string(), "".to_string()],
            vec!["key0".to_string()],
        );
        assert_eq!(msg.keys(), &["key0".to_string()]);
    }
}
