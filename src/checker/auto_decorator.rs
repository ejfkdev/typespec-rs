//! Auto decorators
//!
//! Ported from TypeSpec `compiler/src/core/auto-decorator.ts`
//! (microsoft/typespec#10197).
//!
//! Auto decorators (`auto dec`) are simple metadata annotations whose
//! arguments the compiler stores automatically — no external implementation
//! is needed. The stored value is always a record keyed by parameter name:
//!
//! | Parameters (beyond target) | Stored value                  |
//! |----------------------------|-------------------------------|
//! | None                       | `{}`                          |
//! | One                        | `{ paramName: value }`        |
//! | Multiple                   | `{ p1: v1, p2: v2, ... }`     |
//!
//! Applying the same auto decorator twice on the same declaration emits a
//! `duplicate-decorator` warning but still stores (last-write-wins), matching
//! extern decorator behavior.

use super::*;

/// A value stored in auto decorator state.
#[derive(Debug, Clone)]
pub enum AutoDecoratorValue {
    /// A single marshalled argument value.
    Value(DecoratorMarshalledValue),
    /// A rest parameter's collected argument values.
    Array(Vec<DecoratorMarshalledValue>),
}

/// Get the state key for an auto decorator given its fully-qualified name.
///
/// Uses the `dec:` prefix so the key is based on decorator identity, not
/// declaration style — allows seamless migration from auto to extern.
///
/// Ported from TS `getAutoDecoratorStateKey`.
pub fn get_auto_decorator_state_key(decorator_fqn: &str) -> String {
    format!("dec:{}", decorator_fqn)
}

impl Checker {
    /// Programmatically apply an auto decorator to a target, storing its
    /// argument values.
    ///
    /// Mirrors what the synthesized `auto dec` implementation does when the
    /// decorator is written in source, so emitters and other consumers can
    /// mark synthetic types the same way without reaching into the state map
    /// directly. Pass an empty record for a no-arg decorator.
    ///
    /// Ported from TS `setAutoDecorator` (microsoft/typespec#11247).
    pub fn set_auto_decorator(
        &mut self,
        decorator_fqn: &str,
        target: TypeId,
        value: Vec<(String, AutoDecoratorValue)>,
    ) {
        let key = get_auto_decorator_state_key(decorator_fqn);
        self.auto_decorator_state
            .entry(key)
            .or_default()
            .insert(target, value);
    }

    /// Store the arguments of an applied auto decorator for a target type.
    ///
    /// Ported from the implementation built by TS
    /// `createAutoDecoratorImplementation` (the duplicate warning is emitted
    /// by the caller, which sees all applications on the node).
    pub(crate) fn apply_auto_decorator(
        &mut self,
        decorator_fqn: &str,
        target: TypeId,
        record: Vec<(String, AutoDecoratorValue)>,
    ) {
        let key = get_auto_decorator_state_key(decorator_fqn);
        self.auto_decorator_state
            .entry(key)
            .or_default()
            .insert(target, record);
    }

    /// Check if an auto decorator has been applied to a target.
    ///
    /// Ported from TS `hasAutoDecorator`.
    pub fn has_auto_decorator(&self, decorator_fqn: &str, target: TypeId) -> bool {
        let key = get_auto_decorator_state_key(decorator_fqn);
        self.auto_decorator_state
            .get(&key)
            .is_some_and(|targets| targets.contains_key(&target))
    }

    /// Get the stored record for an auto decorator applied to a target.
    /// Always a record of `(paramName, value)` entries (empty for no-arg
    /// decorators). Returns `None` if the decorator was not applied.
    ///
    /// Ported from TS `getAutoDecoratorValue`.
    pub fn get_auto_decorator_value(
        &self,
        decorator_fqn: &str,
        target: TypeId,
    ) -> Option<&Vec<(String, AutoDecoratorValue)>> {
        let key = get_auto_decorator_state_key(decorator_fqn);
        self.auto_decorator_state.get(&key)?.get(&target)
    }

    /// Get all targets that have a specific auto decorator applied, along
    /// with their stored records.
    ///
    /// Ported from TS `getAutoDecoratorTargets`.
    pub fn get_auto_decorator_targets(
        &self,
        decorator_fqn: &str,
    ) -> Option<&HashMap<TypeId, Vec<(String, AutoDecoratorValue)>>> {
        let key = get_auto_decorator_state_key(decorator_fqn);
        self.auto_decorator_state.get(&key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_key_prefix() {
        assert_eq!(
            get_auto_decorator_state_key("MyLib.label"),
            "dec:MyLib.label"
        );
    }
}
