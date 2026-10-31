//! Client discovery, operations, data types, and constructors.
//!
//! Ported from the `client-library` and `client` kits of
//! `packages/http-client` (the shared layer used by the official Java,
//! TypeScript, Python, and C# SDK generators).
//!
//! The typekit versions create synthetic TypeSpec types for client
//! constructors. The Rust port models constructors as plain data instead, so
//! callers do not need a mutable [`Checker`](crate::checker::Checker).

use std::collections::{HashMap, HashSet};

use crate::checker::Checker;
use crate::checker::types::{Type, TypeId};
use crate::libs::compiler::is_service;
use crate::libs::http::auth::{
    Authentication, get_authentication_scheme_types, parse_authentication,
};
use crate::libs::http::get_http_part;
use crate::libs::http::operation::{HttpServer, get_server};
use crate::libs::http::operations::get_http_operation;
use crate::state_accessors::StateAccessors;

/// Identifier of a client inside a [`ClientLibrary`].
pub type ClientId = usize;

/// A discovered client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InternalClient {
    /// Client name (`<namespace>Client`).
    pub name: String,
    /// The namespace or interface this client models.
    pub type_id: TypeId,
    /// The nearest enclosing `@service` namespace.
    pub service: TypeId,
}

/// An operation bound to a client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientOperation {
    /// Operation name.
    pub name: String,
    /// The operation type.
    pub operation_id: TypeId,
    /// Owning client.
    pub client: ClientId,
}

/// A client with its operations and sub-clients.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Client {
    /// Client name.
    pub name: String,
    /// The namespace or interface this client models.
    pub type_id: TypeId,
    /// The nearest enclosing `@service` namespace.
    pub service: TypeId,
    /// Operations declared directly on this client.
    pub operations: Vec<ClientOperation>,
    /// Nested clients (sub-namespaces and interfaces).
    pub sub_clients: Vec<ClientId>,
    /// Parent client, if any.
    pub parent: Option<ClientId>,
}

/// The complete client model for a specification.
#[derive(Debug, Clone, Default)]
pub struct ClientLibrary {
    /// Arena of all clients.
    pub clients: Vec<Client>,
    /// Top-level clients to generate.
    pub top_level: Vec<ClientId>,
    /// Named data types referenced by the clients.
    pub data_types: Vec<TypeId>,
    /// Operation type -> owning client.
    pub operation_client: HashMap<TypeId, ClientId>,
}

impl ClientLibrary {
    /// Get a client by id.
    pub fn client(&self, id: ClientId) -> &Client {
        &self.clients[id]
    }

    /// Get the client that owns an operation.
    pub fn get_client_for_operation(&self, operation_id: TypeId) -> Option<&Client> {
        self.operation_client
            .get(&operation_id)
            .and_then(|id| self.clients.get(*id))
    }

    /// Flatten a client and all its sub-clients (depth first).
    pub fn flat(&self, id: ClientId) -> Vec<ClientId> {
        let mut stack = vec![id];
        let mut result = Vec::new();
        while let Some(current) = stack.pop() {
            result.push(current);
            stack.extend(self.clients[current].sub_clients.iter().copied());
        }
        result
    }
}

/// A diagnostic produced while building a client library.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientLibraryDiagnostic {
    /// Diagnostic code.
    pub code: &'static str,
    /// Diagnostic message.
    pub message: String,
}

/// Result of [`create_client_library`].
#[derive(Debug, Clone, Default)]
pub struct ClientLibraryResult {
    /// The discovered clients.
    pub library: ClientLibrary,
    /// Diagnostics to report.
    pub diagnostics: Vec<ClientLibraryDiagnostic>,
}

// ============================================================================
// Naming and discovery
// ============================================================================

/// `<name>Client`, unless the name already ends in `Client`. The global
/// namespace (empty name) becomes `Client`.
pub fn get_client_name(name: &str) -> String {
    if name.ends_with("Client") {
        name.to_string()
    } else {
        format!("{name}Client")
    }
}

fn type_name(checker: &Checker, type_id: TypeId) -> String {
    checker
        .get_type(type_id)
        .and_then(|t| t.name().map(str::to_string))
        .unwrap_or_default()
}

/// Find the nearest enclosing `@service` namespace.
fn find_service_namespace(checker: &Checker, state: &StateAccessors, type_id: TypeId) -> TypeId {
    let mut current = match checker.get_type(type_id) {
        Some(Type::Namespace(_)) => Some(type_id),
        Some(Type::Interface(i)) => i.namespace,
        _ => None,
    };
    while let Some(ns_id) = current {
        if is_service(state, ns_id) {
            return ns_id;
        }
        current = match checker.get_type(ns_id) {
            Some(Type::Namespace(ns)) => ns.namespace,
            _ => None,
        };
    }
    checker.get_global_namespace_type().unwrap_or(type_id)
}

/// Build a client for a namespace or interface.
pub fn get_client(
    checker: &Checker,
    state: &StateAccessors,
    type_id: TypeId,
) -> Option<InternalClient> {
    match checker.get_type(type_id) {
        Some(Type::Namespace(_)) | Some(Type::Interface(_)) => Some(InternalClient {
            name: get_client_name(&type_name(checker, type_id)),
            type_id,
            service: find_service_namespace(checker, state, type_id),
        }),
        _ => None,
    }
}

/// List user-defined child namespaces.
///
/// With no parent, returns the project's top-level namespaces (library
/// namespaces such as `TypeSpec` are excluded). With a parent, returns its
/// user-defined child namespaces.
pub fn list_namespaces(checker: &Checker, parent: Option<TypeId>) -> Vec<TypeId> {
    let parent_id = match parent {
        Some(id) => id,
        None => match checker.get_global_namespace_type() {
            Some(id) => id,
            None => return Vec::new(),
        },
    };
    let Some(Type::Namespace(ns)) = checker.get_type(parent_id) else {
        return Vec::new();
    };
    let ns = ns.clone();
    ns.namespace_names
        .iter()
        .filter_map(|name| ns.namespaces.get(name).copied())
        .filter(|&child| {
            checker
                .get_type(child)
                .and_then(|t| match t {
                    Type::Namespace(n) => n.node,
                    _ => None,
                })
                .is_some_and(|node| !checker.is_library_node(node))
        })
        .collect()
}

/// Direct sub-clients of `client` (sub-namespaces and interfaces).
///
/// Ported from `clientLibrary.listClients(client)` when a client object is
/// passed (as opposed to a namespace).
pub fn list_clients_of_client(
    checker: &Checker,
    state: &StateAccessors,
    client: &InternalClient,
) -> Vec<InternalClient> {
    let Some(Type::Namespace(ns)) = checker.get_type(client.type_id) else {
        return Vec::new();
    };
    let ns = ns.clone();
    let mut result: Vec<InternalClient> = list_namespaces(checker, Some(client.type_id))
        .into_iter()
        .filter_map(|id| get_client(checker, state, id))
        .collect();
    for name in &ns.interface_names {
        if let Some(&iface_id) = ns.interfaces.get(name)
            && let Some(iface_client) = get_client(checker, state, iface_id)
        {
            result.push(iface_client);
        }
    }
    result
}

/// List clients for a namespace or interface.
///
/// Ported from the typekit `clientLibrary.listClients`:
/// - a service namespace yields its own client
/// - any other namespace yields its sub-namespaces, interfaces, and itself
/// - interfaces yield no clients (their enclosing namespace does)
pub fn list_clients(
    checker: &Checker,
    state: &StateAccessors,
    target: TypeId,
) -> Vec<InternalClient> {
    let Some(Type::Namespace(_)) = checker.get_type(target) else {
        return Vec::new();
    };

    if is_service(state, target) {
        return get_client(checker, state, target).into_iter().collect();
    }

    let mut result: Vec<InternalClient> = list_namespaces(checker, Some(target))
        .into_iter()
        .filter_map(|id| get_client(checker, state, id))
        .collect();

    if let Some(Type::Namespace(ns)) = checker.get_type(target) {
        for name in &ns.interface_names {
            if let Some(&iface_id) = ns.interfaces.get(name)
                && let Some(iface_client) = get_client(checker, state, iface_id)
            {
                result.push(iface_client);
            }
        }
    }

    // Include the namespace itself when it is not a service (upstream pushes
    // the namespace after its children).
    if let Some(client) = get_client(checker, state, target) {
        result.push(client);
    }
    result
}

/// List the HTTP operations declared directly on a client.
///
/// Template declarations and template instances are skipped, matching
/// `client.listHttpOperations`.
pub fn list_http_operations(
    checker: &Checker,
    state: &StateAccessors,
    client: &InternalClient,
) -> Vec<crate::libs::http::HttpOperation> {
    let operation_ids: Vec<TypeId> = match checker.get_type(client.type_id) {
        Some(Type::Namespace(ns)) => ns
            .operation_names
            .iter()
            .filter_map(|name| ns.operations.get(name).copied())
            .collect(),
        Some(Type::Interface(iface)) => {
            if crate::checker::type_utils::is_template_declaration(checker, client.type_id) {
                return Vec::new();
            }
            iface
                .operation_names
                .iter()
                .filter_map(|name| iface.operations.get(name).copied())
                .collect()
        }
        _ => return Vec::new(),
    };

    operation_ids
        .into_iter()
        .filter(|&id| {
            // Skip template declarations and instances (upstream
            // isTemplateDeclarationOrInstance). An operation's own
            // `template_node` is set for both.
            let templated = matches!(
                checker.get_type(id),
                Some(Type::Operation(o)) if o.template_node.is_some()
            );
            !templated
                && !crate::checker::type_utils::is_template_declaration_or_instance(checker, id)
        })
        .filter_map(|id| get_http_operation(checker, state, id))
        .collect()
}

/// Get the parent client of a client.
pub fn get_parent_client(
    checker: &Checker,
    state: &StateAccessors,
    client: &InternalClient,
) -> Option<InternalClient> {
    match checker.get_type(client.type_id) {
        Some(Type::Namespace(ns)) => {
            let parent = ns.namespace?;
            if Some(parent) == checker.get_global_namespace_type() {
                None
            } else {
                get_client(checker, state, parent)
            }
        }
        Some(Type::Interface(iface)) => iface
            .namespace
            .and_then(|ns| get_client(checker, state, ns)),
        _ => None,
    }
}

/// Flatten a client and all its sub-clients.
pub fn flat_clients(
    checker: &Checker,
    state: &StateAccessors,
    client: &InternalClient,
) -> Vec<InternalClient> {
    let mut stack = vec![client.clone()];
    let mut result = Vec::new();
    while let Some(current) = stack.pop() {
        stack.extend(list_clients_of_client(checker, state, &current));
        result.push(current);
    }
    result
}

/// Whether a client can be constructed publicly (namespace clients can).
pub fn is_publicly_initializable(client: &InternalClient) -> bool {
    // Interfaces are reached through their enclosing namespace client.
    // This is checked by the caller with the type store; the data model
    // records only the id, so expose it via `is_namespace_client`.
    !client.name.is_empty()
        && !client.name.ends_with("InterfaceClient")
        && !client.name.ends_with("Interface")
}

/// Servers for a client, walking up the enclosing namespaces.
pub fn list_servers(
    state: &StateAccessors,
    checker: &Checker,
    client: &InternalClient,
) -> Option<HttpServer> {
    let mut current = Some(client.service);
    while let Some(ns_id) = current {
        if let Some(server) = get_server(state, ns_id) {
            return Some(server);
        }
        current = match checker.get_type(ns_id) {
            Some(Type::Namespace(ns)) => ns.namespace,
            _ => None,
        };
    }
    None
}

/// Authentication configured for a client's service.
pub fn get_client_auth(state: &StateAccessors, client: &InternalClient) -> Option<Authentication> {
    parse_authentication(state, client.service)
}

// ============================================================================
// Client constructors
// ============================================================================

/// Type of a client constructor parameter.
#[derive(Debug, Clone, PartialEq)]
pub enum ClientParameterType {
    /// Plain `string`.
    String,
    /// A string literal (credential scheme or default endpoint).
    StringLiteral(String),
    /// A union of alternatives.
    Union(Vec<ClientParameterType>),
    /// A server parameter, identified by its `ModelProperty`.
    ServerParameter(TypeId),
}

/// A client constructor parameter.
#[derive(Debug, Clone, PartialEq)]
pub struct ClientConstructorParameter {
    /// Parameter name.
    pub name: String,
    /// Parameter type.
    pub parameter_type: ClientParameterType,
    /// Whether the parameter is optional.
    pub optional: bool,
    /// Default value, if any.
    pub default_value: Option<String>,
}

/// A client constructor (the synthesized operation that creates a client).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ClientConstructor {
    /// Constructor parameters, in order.
    pub parameters: Vec<ClientConstructorParameter>,
}

/// Build the constructor for a client.
///
/// Ported from `getConstructors` in `utils/client-helpers.ts`:
/// - no `@server`: one required `endpoint` string parameter
/// - one `@server`: server parameters plus an `endpoint` parameter defaulting
///   to the server URL (renamed `_endpoint` on a name collision)
/// - authentication: a required `credential` parameter whose type is the
///   string literal for a single scheme or a union of scheme literals
pub fn get_client_constructor(
    checker: &Checker,
    state: &StateAccessors,
    client: &InternalClient,
) -> ClientConstructor {
    let mut parameters = Vec::new();

    match list_servers(state, checker, client) {
        None => parameters.push(ClientConstructorParameter {
            name: "endpoint".to_string(),
            parameter_type: ClientParameterType::String,
            optional: false,
            default_value: None,
        }),
        Some(server) => {
            let server_param_names: Vec<String> = server
                .parameters
                .iter()
                .filter_map(|&prop_id| match checker.get_type(prop_id) {
                    Some(Type::ModelProperty(p)) => Some((p.name.clone(), p.optional)),
                    _ => None,
                })
                .map(|(name, _)| name)
                .collect();
            for &prop_id in &server.parameters {
                if let Some(Type::ModelProperty(p)) = checker.get_type(prop_id) {
                    parameters.push(ClientConstructorParameter {
                        name: p.name.clone(),
                        parameter_type: ClientParameterType::ServerParameter(prop_id),
                        optional: p.optional,
                        default_value: None,
                    });
                }
            }
            let endpoint_name = if server_param_names.iter().any(|n| n == "endpoint") {
                "_endpoint"
            } else {
                "endpoint"
            };
            parameters.push(ClientConstructorParameter {
                name: endpoint_name.to_string(),
                parameter_type: ClientParameterType::String,
                optional: false,
                default_value: Some(server.url),
            });
        }
    }

    let credential_types = get_authentication_scheme_types(state, client.service);
    match credential_types.len() {
        0 => {}
        1 => parameters.push(ClientConstructorParameter {
            name: "credential".to_string(),
            parameter_type: ClientParameterType::StringLiteral(credential_types[0].clone()),
            optional: false,
            default_value: None,
        }),
        _ => parameters.push(ClientConstructorParameter {
            name: "credential".to_string(),
            parameter_type: ClientParameterType::Union(
                credential_types
                    .into_iter()
                    .map(ClientParameterType::StringLiteral)
                    .collect(),
            ),
            optional: false,
            default_value: None,
        }),
    }

    ClientConstructor { parameters }
}

/// Whether two clients have the same constructor parameter types.
pub fn have_same_constructor(
    checker: &Checker,
    state: &StateAccessors,
    a: &InternalClient,
    b: &InternalClient,
) -> bool {
    let a_params = get_client_constructor(checker, state, a).parameters;
    let b_params = get_client_constructor(checker, state, b).parameters;
    a_params.len() == b_params.len()
        && a_params
            .iter()
            .zip(b_params.iter())
            .all(|(a, b)| a.parameter_type == b.parameter_type)
}

// ============================================================================
// Data type collection
// ============================================================================

/// Collect the named data types referenced by `type_id`.
///
/// Ported from `collectDataTypes`: named models, unions, enums, and scalars
/// are collected. Anonymous types are skipped; `@httpPart` models are replaced
/// by the type they wrap. Built-in `TypeSpec` types are excluded.
pub fn collect_data_types(
    checker: &Checker,
    state: &StateAccessors,
    type_id: TypeId,
    out: &mut Vec<TypeId>,
    seen: &mut HashSet<TypeId>,
) {
    let resolved = checker.resolve_alias_chain(type_id);
    if !seen.insert(resolved) {
        return;
    }

    if let Some(part) = get_http_part(state, resolved) {
        collect_data_types(checker, state, part.part_type, out, seen);
        return;
    }

    match checker.get_type(resolved) {
        Some(Type::Model(m)) => {
            let m = m.clone();
            if is_named_data_type(checker, resolved, &m.name, m.namespace)
                && !out.contains(&resolved)
            {
                out.push(resolved);
            }
            for name in &m.property_names {
                if let Some(&prop_id) = m.properties.get(name) {
                    collect_data_types(checker, state, prop_id, out, seen);
                }
            }
            if let Some(base) = m.base_model {
                collect_data_types(checker, state, base, out, seen);
            }
            if let Some((key, value)) = m.indexer {
                collect_data_types(checker, state, key, out, seen);
                collect_data_types(checker, state, value, out, seen);
            }
            // Derived models are part of the data type graph (upstream
            // navigates with `visitDerivedTypes: true`).
            for &derived in &m.derived_models {
                collect_data_types(checker, state, derived, out, seen);
            }
        }
        Some(Type::Union(u)) => {
            if is_named_data_type(checker, resolved, &u.name, u.namespace)
                && !out.contains(&resolved)
            {
                out.push(resolved);
            }
            for name in &u.variant_names {
                if let Some(&variant_id) = u.variants.get(name) {
                    collect_data_types(checker, state, variant_id, out, seen);
                }
            }
        }
        Some(Type::UnionVariant(v)) => {
            collect_data_types(checker, state, v.r#type, out, seen);
        }
        Some(Type::Enum(e))
            if is_named_data_type(checker, resolved, &e.name, e.namespace)
                && !out.contains(&resolved) =>
        {
            out.push(resolved);
        }
        Some(Type::Scalar(s))
            if is_named_data_type(checker, resolved, &s.name, s.namespace)
                && !out.contains(&resolved) =>
        {
            out.push(resolved);
        }
        Some(Type::ModelProperty(p)) => {
            collect_data_types(checker, state, p.r#type, out, seen);
        }
        Some(Type::Tuple(t)) => {
            for &value in &t.values {
                collect_data_types(checker, state, value, out, seen);
            }
        }
        Some(Type::Operation(o)) => {
            if let Some(params) = o.parameters {
                collect_data_types(checker, state, params, out, seen);
            }
            if let Some(ret) = o.return_type {
                collect_data_types(checker, state, ret, out, seen);
            }
        }
        Some(Type::Interface(i)) => {
            for name in &i.operation_names {
                if let Some(&op_id) = i.operations.get(name) {
                    collect_data_types(checker, state, op_id, out, seen);
                }
            }
        }
        _ => {}
    }
}

fn is_named_data_type(
    checker: &Checker,
    _type_id: TypeId,
    name: &str,
    namespace: Option<TypeId>,
) -> bool {
    if name.is_empty() {
        return false;
    }
    if let Some(ns) = namespace {
        let qualifier =
            crate::checker::type_utils::get_fully_qualified_name(&checker.type_store, ns);
        if qualifier == "TypeSpec" {
            return false;
        }
    }
    true
}

// ============================================================================
// Client library construction
// ============================================================================

fn has_global_operations(checker: &Checker) -> bool {
    let Some(global) = checker.get_global_namespace_type() else {
        return false;
    };
    match checker.get_type(global) {
        Some(Type::Namespace(ns)) => ns.operation_names.iter().any(|name| {
            ns.operations
                .get(name)
                .is_some_and(|&id| crate::typekit::kits::type_kind::is_user_defined(checker, id))
        }),
        _ => false,
    }
}

/// Resolve a namespace to its effective client for top-level discovery.
///
/// A namespace with operations or interfaces is a client. A namespace without
/// content collapses to its single effective descendant (upstream
/// `getEffectiveClient`).
fn get_effective_client(
    checker: &Checker,
    state: &StateAccessors,
    ns_id: TypeId,
) -> Option<InternalClient> {
    let ns = match checker.get_type(ns_id) {
        Some(Type::Namespace(ns)) => ns.clone(),
        _ => return None,
    };
    if !ns.operations.is_empty() || !ns.interfaces.is_empty() {
        return get_client(checker, state, ns_id);
    }

    let mut effective: Vec<InternalClient> = Vec::new();
    for child in list_namespaces(checker, Some(ns_id)) {
        if let Some(client) = get_effective_client(checker, state, child) {
            effective.push(client);
        }
    }

    match effective.len() {
        1 => effective.pop(),
        n if n > 1 => get_client(checker, state, ns_id),
        _ => None,
    }
}

fn user_defined_top_level_clients(
    checker: &Checker,
    state: &StateAccessors,
) -> Vec<InternalClient> {
    let Some(global) = checker.get_global_namespace_type() else {
        return Vec::new();
    };
    list_namespaces(checker, Some(global))
        .into_iter()
        .filter_map(|ns| get_effective_client(checker, state, ns))
        .collect()
}

fn visit_client(
    checker: &Checker,
    state: &StateAccessors,
    internal: InternalClient,
    library: &mut ClientLibrary,
    parent: Option<ClientId>,
    data_types: &mut Vec<TypeId>,
    seen_types: &mut HashSet<TypeId>,
) -> ClientId {
    let id = library.clients.len();
    library.clients.push(Client {
        name: internal.name.clone(),
        type_id: internal.type_id,
        service: internal.service,
        operations: Vec::new(),
        sub_clients: Vec::new(),
        parent,
    });

    let sub_clients: Vec<ClientId> = list_clients_of_client(checker, state, &internal)
        .into_iter()
        .map(|child| {
            visit_client(
                checker,
                state,
                child,
                library,
                Some(id),
                data_types,
                seen_types,
            )
        })
        .collect();

    let mut operations = Vec::new();
    for http_op in list_http_operations(checker, state, &internal) {
        library.operation_client.insert(http_op.operation, id);

        // Collect parameter and return types.
        if let Some(Type::Operation(op)) = checker.get_type(http_op.operation) {
            if let Some(params) = op.parameters {
                collect_data_types(checker, state, params, data_types, seen_types);
            }
            if let Some(ret) = op.return_type {
                collect_data_types(checker, state, ret, data_types, seen_types);
            }
        }

        operations.push(ClientOperation {
            name: type_name(checker, http_op.operation),
            operation_id: http_op.operation,
            client: id,
        });
    }

    // Constructor server parameters are data types too.
    for param in get_client_constructor(checker, state, &internal).parameters {
        if let ClientParameterType::ServerParameter(prop_id) = param.parameter_type
            && let Some(Type::ModelProperty(p)) = checker.get_type(prop_id)
        {
            collect_data_types(checker, state, p.r#type, data_types, seen_types);
        }
    }

    if let Some(client) = library.clients.get_mut(id) {
        client.operations = operations;
        client.sub_clients = sub_clients;
    }
    id
}

/// Discover the clients of a specification.
///
/// Ported from `createClientLibrary`:
/// - global operations make the global namespace the client
/// - otherwise user-defined top-level namespaces are visited (collapsing
///   namespaces that only contain a single client)
/// - a `cant-find-client` diagnostic is produced when nothing is found
pub fn create_client_library(checker: &Checker, state: &StateAccessors) -> ClientLibraryResult {
    let mut library = ClientLibrary::default();
    let mut diagnostics = Vec::new();

    let mut top_level: Vec<InternalClient> = if has_global_operations(checker) {
        checker
            .get_global_namespace_type()
            .and_then(|global| get_client(checker, state, global))
            .into_iter()
            .collect()
    } else {
        user_defined_top_level_clients(checker, state)
    };

    if top_level.is_empty() {
        diagnostics.push(ClientLibraryDiagnostic {
            code: super::DIAG_CANT_FIND_CLIENT,
            message: "Can't find any clients in the spec".to_string(),
        });
    }

    let mut data_types = Vec::new();
    let mut seen_types = HashSet::new();
    let mut top_level_ids = Vec::new();
    for client in top_level.drain(..) {
        top_level_ids.push(visit_client(
            checker,
            state,
            client,
            &mut library,
            None,
            &mut data_types,
            &mut seen_types,
        ));
    }

    library.top_level = top_level_ids;
    library.data_types = data_types;
    ClientLibraryResult {
        library,
        diagnostics,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(source: &str) -> Checker {
        let options = crate::parser::ParseOptions::with_http();
        let result = crate::parser::parse_with_options(source, options);
        let mut checker = Checker::new();
        checker.register_decorators(vec![
            ("service", "TypeSpec.Http", "Namespace"),
            ("get", "TypeSpec.Http", "Operation"),
            ("post", "TypeSpec.Http", "Operation"),
            ("route", "TypeSpec.Http", "Operation"),
            ("server", "TypeSpec.Http", "Namespace"),
            ("useAuth", "TypeSpec.Http", "unknown"),
            ("error", "TypeSpec", "unknown"),
        ]);
        checker.set_parse_result(result.root_id, result.builder.clone());
        checker.check_program();
        checker
    }

    fn library_for(source: &str) -> (Checker, ClientLibrary) {
        let checker = check(source);
        let result = create_client_library(&checker, &checker.state_accessors);
        (checker, result.library)
    }

    // ---- Ported from http-client/test/typekit/client-library.test.ts ----

    #[test]
    fn test_list_namespaces_basic() {
        let checker = check(
            r#"
            @service(#{ title: "Widget Service" })
            namespace DemoService;
            "#,
        );
        let namespaces = list_namespaces(&checker, None);
        assert_eq!(namespaces.len(), 1);
        assert_eq!(
            checker.get_type(namespaces[0]).and_then(|t| t.name()),
            Some("DemoService")
        );
    }

    #[test]
    fn test_list_namespaces_nested() {
        let checker = check(
            r#"
            @service(#{ title: "Widget Service" })
            namespace DemoService {
              namespace NestedService {
                namespace NestedNestedService {}
              }
            }
            "#,
        );
        let top = list_namespaces(&checker, None);
        assert_eq!(top.len(), 1);
        let nested = list_namespaces(&checker, Some(top[0]));
        assert_eq!(nested.len(), 1);
        assert_eq!(
            checker.get_type(nested[0]).and_then(|t| t.name()),
            Some("NestedService")
        );
        let nested2 = list_namespaces(&checker, Some(nested[0]));
        assert_eq!(nested2.len(), 1);
        assert_eq!(
            checker.get_type(nested2[0]).and_then(|t| t.name()),
            Some("NestedNestedService")
        );
    }

    #[test]
    fn test_library_namespaces_excluded() {
        // The TypeSpec stdlib namespace is library code and must not be
        // reported as a user namespace.
        let checker = check("model Foo {}");
        let namespaces = list_namespaces(&checker, None);
        assert!(
            namespaces.is_empty()
                || !namespaces
                    .iter()
                    .any(|&id| { checker.get_type(id).and_then(|t| t.name()) == Some("TypeSpec") })
        );
    }

    #[test]
    fn test_list_clients_no_clients_for_global_ops_less_spec() {
        let checker = check("model Foo {}");
        let global = checker.get_global_namespace_type().unwrap();
        let clients = list_clients(&checker, &checker.state_accessors, global);
        // No operations or namespaces: only the global client is returned by
        // list_clients, but create_client_library reports no clients.
        let result = create_client_library(&checker, &checker.state_accessors);
        assert!(result.library.top_level.is_empty());
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(
            result.diagnostics[0].code,
            super::super::DIAG_CANT_FIND_CLIENT
        );
        let _ = clients;
    }

    #[test]
    fn test_get_client_for_service() {
        let checker = check(
            r#"
            @service(#{ title: "Widget Service" })
            namespace DemoService;
            "#,
        );
        let service = checker.declared_types["DemoService"];
        let clients = list_clients(&checker, &checker.state_accessors, service);
        assert_eq!(clients.len(), 1);
        assert_eq!(clients[0].name, "DemoServiceClient");
        assert_eq!(clients[0].service, service);
        assert_eq!(clients[0].type_id, service);
    }

    #[test]
    fn test_get_sub_clients() {
        let checker = check(
            r#"
            @service(#{ title: "Widget Service" })
            namespace DemoService {
              namespace NestedService {}
            }
            "#,
        );
        let state = &checker.state_accessors;
        let roots = list_clients(&checker, state, checker.declared_types["DemoService"]);
        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0].name, "DemoServiceClient");
        let sub = list_clients_of_client(&checker, state, &roots[0]);
        assert_eq!(sub.len(), 1);
        assert_eq!(sub[0].name, "NestedServiceClient");
    }

    #[test]
    fn test_flat_clients() {
        // Ported from client.test.ts "should get a flattened list of clients".
        let checker = check(
            r#"
            @service(#{ title: "Widget Service" })
            namespace DemoService {
              namespace Foo {
                namespace FooBaz {}
              }
              namespace Bar {
                interface BarBaz {}
              }
            }
            "#,
        );
        let state = &checker.state_accessors;
        let roots = list_clients(&checker, state, checker.declared_types["DemoService"]);
        let flat = flat_clients(&checker, state, &roots[0]);
        assert_eq!(flat.len(), 5);
        let names: Vec<_> = flat.iter().map(|c| c.name.clone()).collect();
        assert!(names.contains(&"BarBazClient".to_string()), "{names:?}");
    }

    #[test]
    fn test_global_operations_create_global_client() {
        let (_, library) = library_for("op foo(): void;");
        assert_eq!(library.top_level.len(), 1);
        assert_eq!(library.client(library.top_level[0]).name, "Client");
    }

    #[test]
    fn test_nested_only_namespace_collapses_to_client() {
        // A top-level namespace with a single effective client collapses to
        // that client (upstream getEffectiveClient).
        let (_, library) = library_for(
            r#"
            namespace Outer {
              @service(#{ title: "Widget Service" })
              namespace DemoService {
                @get op get(): void;
              }
            }
            "#,
        );
        assert_eq!(library.top_level.len(), 1);
        assert_eq!(
            library.client(library.top_level[0]).name,
            "DemoServiceClient"
        );
    }

    #[test]
    fn test_operations_and_data_types() {
        let (checker, library) = library_for(
            r#"
            @service(#{ title: "Widget Service" })
            namespace DemoService {
              model Pet { name: string; }
              enum Color { red, blue }
              @get op getPet(): Pet;
              @post op createPet(@body pet: Pet): Pet;
            }
            "#,
        );
        let root = library.client(library.top_level[0]);
        assert_eq!(root.operations.len(), 2);
        let pet_id = checker.declared_types["DemoService.Pet"];
        assert!(library.data_types.contains(&pet_id));
        let color_id = checker.declared_types["DemoService.Color"];
        assert!(!library.data_types.contains(&color_id), "unused enum");
        assert!(
            library
                .get_client_for_operation(checker.declared_types["DemoService.getPet"])
                .is_some()
        );
    }

    #[test]
    fn test_constructor_endpoint_only() {
        let (checker, _library) = library_for(
            r#"
            @service(#{ title: "Widget Service" })
            namespace DemoService;
            "#,
        );
        let client = get_client(
            &checker,
            &checker.state_accessors,
            checker.declared_types["DemoService"],
        )
        .unwrap();
        let constructor = get_client_constructor(&checker, &checker.state_accessors, &client);
        assert_eq!(constructor.parameters.len(), 1);
        assert_eq!(constructor.parameters[0].name, "endpoint");
        assert_eq!(
            constructor.parameters[0].parameter_type,
            ClientParameterType::String
        );
        assert!(!constructor.parameters[0].optional);
    }

    #[test]
    fn test_constructor_server_default_endpoint() {
        let (checker, _library) = library_for(
            r#"
            @server("https://example.com", "The service endpoint")
            @service(#{ title: "Widget Service" })
            namespace DemoService;
            "#,
        );
        let client = get_client(
            &checker,
            &checker.state_accessors,
            checker.declared_types["DemoService"],
        )
        .unwrap();
        let constructor = get_client_constructor(&checker, &checker.state_accessors, &client);
        assert_eq!(constructor.parameters.len(), 1);
        assert_eq!(
            constructor.parameters[0].default_value.as_deref(),
            Some("https://example.com")
        );
    }

    #[test]
    fn test_constructor_server_parameter() {
        let (checker, _) = library_for(
            r#"
            @server("https://example.com/{name}/foo", "My service url", { name: string })
            @service(#{ title: "Widget Service" })
            namespace DemoService;
            "#,
        );
        let client = get_client(
            &checker,
            &checker.state_accessors,
            checker.declared_types["DemoService"],
        )
        .unwrap();
        let constructor = get_client_constructor(&checker, &checker.state_accessors, &client);
        assert_eq!(constructor.parameters.len(), 2);
        assert_eq!(constructor.parameters[0].name, "name");
        assert!(!constructor.parameters[0].optional);
        assert_eq!(constructor.parameters[1].name, "endpoint");
        assert_eq!(
            constructor.parameters[1].default_value.as_deref(),
            Some("https://example.com/{name}/foo")
        );
    }

    #[test]
    fn test_constructor_endpoint_name_collision() {
        let (checker, _) = library_for(
            r#"
            @server("https://{endpoint}/foo", "My service url", { endpoint: string })
            @service(#{ title: "Widget Service" })
            namespace DemoService;
            "#,
        );
        let client = get_client(
            &checker,
            &checker.state_accessors,
            checker.declared_types["DemoService"],
        )
        .unwrap();
        let constructor = get_client_constructor(&checker, &checker.state_accessors, &client);
        let names: Vec<_> = constructor
            .parameters
            .iter()
            .map(|p| p.name.as_str())
            .collect();
        assert_eq!(names, vec!["endpoint", "_endpoint"]);
        assert_eq!(constructor.parameters[0].default_value, None);
        assert_eq!(
            constructor.parameters[1].default_value.as_deref(),
            Some("https://{endpoint}/foo")
        );
    }

    #[test]
    fn test_constructor_credential_from_use_auth() {
        // Ported from client.test.ts credential tests.
        let (checker, _) = library_for(
            r#"
            using TypeSpec.Http;
            @service(#{ title: "Widget Service" })
            @useAuth(ApiKeyAuth<ApiKeyLocation.header, "x-ms-api-key">)
            namespace DemoService;
            "#,
        );
        let client = get_client(
            &checker,
            &checker.state_accessors,
            checker.declared_types["DemoService"],
        )
        .unwrap();
        let constructor = get_client_constructor(&checker, &checker.state_accessors, &client);
        assert_eq!(constructor.parameters.len(), 2);
        assert_eq!(constructor.parameters[0].name, "endpoint");
        assert_eq!(constructor.parameters[1].name, "credential");
        assert_eq!(
            constructor.parameters[1].parameter_type,
            ClientParameterType::StringLiteral("apiKey".to_string())
        );
    }

    #[test]
    fn test_constructor_credential_oauth2() {
        let (checker, _) = library_for(
            r#"
            using TypeSpec.Http;
            @service(#{ title: "Widget Service" })
            @useAuth(OAuth2Auth<[{
              type: OAuth2FlowType.implicit;
              authorizationUrl: "https://example.com/authorize";
              scopes: ["https://example.com/.default"];
            }]>)
            namespace DemoService;
            "#,
        );
        let client = get_client(
            &checker,
            &checker.state_accessors,
            checker.declared_types["DemoService"],
        )
        .unwrap();
        let constructor = get_client_constructor(&checker, &checker.state_accessors, &client);
        assert_eq!(
            constructor.parameters[1].parameter_type,
            ClientParameterType::StringLiteral("oauth2".to_string())
        );
    }

    #[test]
    fn test_have_same_constructor() {
        let (checker, _) = library_for(
            r#"
            @service(#{ title: "Widget Service" })
            namespace DemoService {
              namespace SubClient {}
            }
            "#,
        );
        let state = &checker.state_accessors;
        let parent = get_client(&checker, state, checker.declared_types["DemoService"]).unwrap();
        let child = get_client(
            &checker,
            state,
            checker.declared_types["DemoService.SubClient"],
        )
        .unwrap();
        assert!(have_same_constructor(&checker, state, &parent, &child));
    }

    // ---- Ported from client.test.ts "getClient" ----

    #[test]
    fn test_get_client_name() {
        assert_eq!(get_client_name(""), "Client");
        assert_eq!(get_client_name("DemoService"), "DemoServiceClient");
        assert_eq!(get_client_name("DemoServiceClient"), "DemoServiceClient");
    }

    #[test]
    fn test_get_client_from_global_namespace() {
        let checker = check("op foo(): void;");
        let global = checker.get_global_namespace_type().unwrap();
        let client = get_client(&checker, &checker.state_accessors, global).unwrap();
        assert_eq!(client.name, "Client");
    }

    #[test]
    fn test_get_client_service_and_type() {
        let checker = check(
            r#"
            @service(#{ title: "Widget Service" })
            namespace DemoService;
            "#,
        );
        let service = checker.declared_types["DemoService"];
        let client = get_client(&checker, &checker.state_accessors, service).unwrap();
        assert_eq!(client.name, "DemoServiceClient");
        assert_eq!(client.service, service);
        assert_eq!(client.type_id, service);
    }

    #[test]
    fn test_list_servers_inherits_from_parent() {
        let checker = check(
            r#"
            @server("https://example.com", "The service endpoint")
            @service(#{ title: "Widget Service" })
            namespace DemoService {
              namespace Sub {}
            }
            "#,
        );
        let state = &checker.state_accessors;
        let sub = get_client(&checker, state, checker.declared_types["DemoService.Sub"]).unwrap();
        let server = list_servers(state, &checker, &sub).unwrap();
        assert_eq!(server.url, "https://example.com");
    }

    #[test]
    fn test_template_operations_are_skipped() {
        let (checker, library) = library_for(
            r#"
            @service(#{ title: "Widget Service" })
            namespace DemoService {
              @get op getPet<T>(value: T): T;
              @get op list(): void;
            }
            "#,
        );
        let root = library.client(library.top_level[0]);
        assert_eq!(root.operations.len(), 1);
        assert_eq!(root.operations[0].name, "list");
        let _ = checker;
    }
}
