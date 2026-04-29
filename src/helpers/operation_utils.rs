//! Operation listing utilities
//!
//! Ported from TypeSpec compiler/src/core/helpers/operation-utils.ts

use crate::checker::{Checker, Type};

/// Options for listing operations
#[derive(Debug, Clone)]
pub struct ListOperationOptions {
    /// If the container is a namespace, look for operations in sub-namespaces.
    /// Default: true
    pub recursive: bool,
}

impl Default for ListOperationOptions {
    fn default() -> Self {
        Self { recursive: true }
    }
}

/// List operations in the given container (Namespace or Interface).
/// Lists operations recursively by default (checks sub-namespaces).
pub fn list_operations_in(
    checker: &Checker,
    container_id: crate::checker::types::TypeId,
    options: ListOperationOptions,
) -> Vec<crate::checker::types::TypeId> {
    let mut operations = Vec::new();
    add_operations(checker, container_id, &options, &mut operations);
    operations
}

fn add_operations(
    checker: &Checker,
    container_id: crate::checker::types::TypeId,
    options: &ListOperationOptions,
    operations: &mut Vec<crate::checker::types::TypeId>,
) {
    let container = checker.get_type(container_id).cloned();
    match container {
        Some(Type::Interface(iface)) => {
            // Skip template interface operations
            if iface.template_node.is_some() && !iface.is_finished {
                return;
            }
            for name in &iface.operation_names {
                if let Some(&op_id) = iface.operations.get(name) {
                    let op = checker.get_type(op_id).cloned();
                    if let Some(Type::Operation(op_type)) = op {
                        // Skip templated operations
                        if op_type.template_node.is_none() {
                            operations.push(op_id);
                        }
                    }
                }
            }
        }
        Some(Type::Namespace(ns)) => {
            // Skip TypeSpec.Prototypes namespace
            if ns.name == "Prototypes"
                && let Some(parent_ns_id) = ns.namespace
            {
                let parent = checker.get_type(parent_ns_id).cloned();
                if let Some(Type::Namespace(parent_ns)) = parent
                    && parent_ns.name == "TypeSpec"
                {
                    return;
                }
            }

            for name in &ns.operation_names {
                if let Some(&op_id) = ns.operations.get(name) {
                    let op = checker.get_type(op_id).cloned();
                    if let Some(Type::Operation(op_type)) = op {
                        // Skip templated operations
                        if op_type.template_node.is_none() {
                            operations.push(op_id);
                        }
                    }
                }
            }

            if options.recursive {
                for name in &ns.namespace_names {
                    if let Some(&sub_ns_id) = ns.namespaces.get(name) {
                        add_operations(checker, sub_ns_id, options, operations);
                    }
                }
            }

            for name in &ns.interface_names {
                if let Some(&iface_id) = ns.interfaces.get(name) {
                    add_operations(checker, iface_id, options, operations);
                }
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::test_utils::check;

    #[test]
    fn test_list_operations_in_namespace() {
        let checker = check(
            "
            op one(): void;
            namespace Foo {
                op two(): void;
            }
        ",
        );
        let global_ns_id = checker.get_global_namespace_type().unwrap();
        let ops = list_operations_in(&checker, global_ns_id, ListOperationOptions::default());
        let names: Vec<String> = ops
            .iter()
            .filter_map(|&id| {
                checker.get_type(id).and_then(|t| {
                    if let Type::Operation(op) = t {
                        Some(op.name.clone())
                    } else {
                        None
                    }
                })
            })
            .collect();
        assert!(
            names.contains(&"one".to_string()),
            "Should contain 'one': {:?}",
            names
        );
        assert!(
            names.contains(&"two".to_string()),
            "Should contain 'two': {:?}",
            names
        );
    }

    #[test]
    fn test_list_operations_in_interface() {
        let checker = check(
            "
            interface Foo {
                two(): void;
                three(): void;
            }
        ",
        );
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        let ops = list_operations_in(&checker, foo_id, ListOperationOptions::default());
        let names: Vec<String> = ops
            .iter()
            .filter_map(|&id| {
                checker.get_type(id).and_then(|t| {
                    if let Type::Operation(op) = t {
                        Some(op.name.clone())
                    } else {
                        None
                    }
                })
            })
            .collect();
        assert!(names.contains(&"two".to_string()));
        assert!(names.contains(&"three".to_string()));
        assert_eq!(names.len(), 2);
    }

    #[test]
    fn test_list_operations_non_recursive() {
        let checker = check(
            "
            namespace Foo {
                op two(): void;
                namespace Bar {
                    op three(): void;
                }
            }
        ",
        );
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        let ops = list_operations_in(&checker, foo_id, ListOperationOptions { recursive: false });
        let names: Vec<String> = ops
            .iter()
            .filter_map(|&id| {
                checker.get_type(id).and_then(|t| {
                    if let Type::Operation(op) = t {
                        Some(op.name.clone())
                    } else {
                        None
                    }
                })
            })
            .collect();
        assert!(
            names.contains(&"two".to_string()),
            "Should contain 'two': {:?}",
            names
        );
        assert!(
            !names.contains(&"three".to_string()),
            "Should NOT contain 'three' (non-recursive): {:?}",
            names
        );
    }

    // ==================== Ported from TS helpers/operation-utils.test.ts ====================

    #[test]
    fn test_list_operations_includes_subnamespace_by_default() {
        let checker = check(
            "
            namespace Foo {
                op one(): void;
                namespace Bar {
                    op two(): void;
                }
            }
        ",
        );
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        let ops = list_operations_in(&checker, foo_id, ListOperationOptions { recursive: true });
        let names: Vec<String> = ops
            .iter()
            .filter_map(|&id| {
                checker.get_type(id).and_then(|t| {
                    if let Type::Operation(op) = t {
                        Some(op.name.clone())
                    } else {
                        None
                    }
                })
            })
            .collect();
        assert!(names.contains(&"one".to_string()));
        assert!(names.contains(&"two".to_string()));
    }

    #[test]
    fn test_list_operations_can_exclude_subnamespace() {
        let checker = check(
            "
            namespace Foo {
                op one(): void;
                namespace Bar {
                    op two(): void;
                }
            }
        ",
        );
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        let ops = list_operations_in(&checker, foo_id, ListOperationOptions { recursive: false });
        let names: Vec<String> = ops
            .iter()
            .filter_map(|&id| {
                checker.get_type(id).and_then(|t| {
                    if let Type::Operation(op) = t {
                        Some(op.name.clone())
                    } else {
                        None
                    }
                })
            })
            .collect();
        assert!(names.contains(&"one".to_string()));
        assert!(!names.contains(&"two".to_string()));
    }
}
