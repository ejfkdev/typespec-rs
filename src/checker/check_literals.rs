//! Literal and circular reference checking
//!
//! Ported from TypeSpec compiler literal checking methods

use super::*;

impl Checker {
    pub(crate) fn check_string_literal(&mut self, node_id: NodeId) -> TypeId {
        let (_, node) = require_ast_node!(self, node_id, StringLiteral, self.error_type);

        // Intern string literal types: same value → same TypeId
        let type_id = if let Some(&existing_id) = self.string_literal_cache.get(&node.value) {
            existing_id
        } else {
            let id = self.create_type(Type::String(StringType {
                id: self.next_type_id(),
                value: node.value.clone(),
                node: Some(node_id),
                is_finished: true,
            }));
            self.string_literal_cache.insert(node.value.clone(), id);
            id
        };

        self.node_type_map.insert(node_id, type_id);
        type_id
    }

    pub(crate) fn check_numeric_literal(&mut self, node_id: NodeId) -> TypeId {
        let (_, node) = require_ast_node!(self, node_id, NumericLiteral, self.error_type);

        // Intern numeric literal types: same value_as_string → same TypeId
        let type_id =
            if let Some(&existing_id) = self.numeric_literal_cache.get(&node.value_as_string) {
                existing_id
            } else {
                let id = self.create_type(Type::Number(NumericType {
                    id: self.next_type_id(),
                    value: node.value,
                    value_as_string: node.value_as_string.clone(),
                    node: Some(node_id),
                    is_finished: true,
                }));
                self.numeric_literal_cache
                    .insert(node.value_as_string.clone(), id);
                id
            };

        self.node_type_map.insert(node_id, type_id);
        type_id
    }

    pub(crate) fn check_boolean_literal(&mut self, node_id: NodeId) -> TypeId {
        let (_, node) = require_ast_node!(self, node_id, BooleanLiteral, self.error_type);

        // Intern boolean literal types: same value → same TypeId
        let type_id = if let Some(&existing_id) = self.boolean_literal_cache.get(&node.value) {
            existing_id
        } else {
            let id = self.create_type(Type::Boolean(BooleanType {
                id: self.next_type_id(),
                value: node.value,
                node: Some(node_id),
                is_finished: true,
            }));
            self.boolean_literal_cache.insert(node.value, id);
            id
        };

        self.node_type_map.insert(node_id, type_id);
        type_id
    }

    /// Check if a type is a decorator or function — these can't be used as type references.
    /// Returns `Some(error_type)` with diagnostic emitted if invalid, `None` if valid.
    pub(crate) fn check_invalid_type_ref_kind(&mut self, type_id: TypeId) -> Option<TypeId> {
        match self.get_type(type_id) {
            Some(Type::Decorator(_)) => {
                self.error("invalid-type-ref", "Can't put a decorator in a type");
                Some(self.error_type)
            }
            Some(Type::FunctionType(_)) => {
                self.error("invalid-type-ref", "Can't use a function as a type");
                Some(self.error_type)
            }
            _ => None,
        }
    }

    /// Check for circular references on the given name.
    /// Returns `Some(error_type)` if a circular reference is detected, `None` otherwise.
    pub(crate) fn check_circular_reference(&mut self, name: &str) -> Option<TypeId> {
        if self.pending_type_names.contains(name) {
            if self.pending_base_type_names.contains(name) {
                self.error(
                    "circular-base-type",
                    &format!(
                        "Type '{}' recursively references itself as a base type.",
                        name
                    ),
                );
            } else {
                self.error(
                    "circular-alias-type",
                    &format!("Alias '{}' has a circular reference.", name),
                );
            }
            return Some(self.error_type);
        }

        if self.pending_op_signature_names.contains(name) {
            self.error(
                "circular-op-signature",
                &format!("Operation '{}' has a circular signature reference.", name),
            );
            return Some(self.error_type);
        }

        if self.pending_template_constraint_names.contains(name) {
            self.error(
                "circular-constraint",
                &format!("Template parameter '{}' has a circular constraint.", name),
            );
            return Some(self.error_type);
        }

        None
    }
}
