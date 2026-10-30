//! Client library typekit operations
//!
//! Ported from `@typespec/http-client` typekit kit `clientLibrary`.

use std::collections::HashSet;

use crate::checker::Checker;
use crate::checker::types::{Type, TypeId};
use crate::libs::http_client::client_library::collect_data_types;
use crate::state_accessors::StateAccessors;

pub use crate::libs::http_client::{
    ClientLibrary, ClientLibraryResult, InternalClient, create_client_library, get_client,
    list_clients, list_clients_of_client, list_namespaces,
};

/// List the models declared in a namespace plus the models referenced by its
/// operation parameter models.
///
/// Ported from the `clientLibrary.listModels` kit.
pub fn list_models(checker: &Checker, namespace_id: TypeId) -> Vec<TypeId> {
    let ns = match checker.get_type(namespace_id) {
        Some(Type::Namespace(ns)) => ns.clone(),
        _ => return Vec::new(),
    };

    let mut result: Vec<TypeId> = Vec::new();
    let push_unique = |id: TypeId, result: &mut Vec<TypeId>| {
        if !result.contains(&id) && ns.models.values().any(|&m| m == id) {
            result.push(id);
        }
    };

    for name in &ns.operation_names {
        let Some(&op_id) = ns.operations.get(name) else {
            continue;
        };
        let Some(Type::Operation(op)) = checker.get_type(op_id) else {
            continue;
        };
        let Some(params) = op.parameters else {
            continue;
        };
        let Some(Type::Model(params_model)) = checker.get_type(params) else {
            continue;
        };
        for prop_name in &params_model.property_names {
            let Some(&prop_id) = params_model.properties.get(prop_name) else {
                continue;
            };
            let Some(Type::ModelProperty(prop)) = checker.get_type(prop_id) else {
                continue;
            };
            if let Some(Type::Model(param_model)) = checker.get_type(prop.r#type) {
                push_unique(prop.r#type, &mut result);
                for nested_name in &param_model.property_names {
                    if let Some(&nested_id) = param_model.properties.get(nested_name)
                        && let Some(Type::ModelProperty(nested)) = checker.get_type(nested_id)
                        && matches!(checker.get_type(nested.r#type), Some(Type::Model(_)))
                    {
                        push_unique(nested.r#type, &mut result);
                    }
                }
            }
        }
    }

    result
}

/// List the enums declared in a namespace.
///
/// Ported from the `clientLibrary.listEnums` kit.
pub fn list_enums(checker: &Checker, namespace_id: TypeId) -> Vec<TypeId> {
    match checker.get_type(namespace_id) {
        Some(Type::Namespace(ns)) => ns
            .enum_names
            .iter()
            .filter_map(|name| ns.enums.get(name).copied())
            .collect(),
        _ => Vec::new(),
    }
}

/// Collect the data types referenced by a client's operations and constructor.
///
/// Ported from the `clientLibrary.listDataTypes` kit.
pub fn list_data_types(
    checker: &Checker,
    state: &StateAccessors,
    client: &InternalClient,
) -> Vec<TypeId> {
    let mut data_types = Vec::new();
    let mut seen = HashSet::new();
    for flat_client in crate::libs::http_client::flat_clients(checker, state, client) {
        for http_op in crate::libs::http_client::list_http_operations(checker, state, &flat_client)
        {
            let Some(Type::Operation(op)) = checker.get_type(http_op.operation) else {
                continue;
            };
            if let Some(params) = op.parameters {
                collect_data_types(checker, state, params, &mut data_types, &mut seen);
            }
            if let Some(ret) = op.return_type {
                collect_data_types(checker, state, ret, &mut data_types, &mut seen);
            }
        }
    }
    data_types
}
