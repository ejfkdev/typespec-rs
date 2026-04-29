//! HTTP operation types and decorators
//!
//! Ported from TypeSpec @typespec/http operations

use super::HttpVerb;
use super::auth::Authentication;
use super::types::*;
use crate::checker::types::TypeId;
use crate::state_accessors::StateAccessors;

// ============================================================================
// HTTP Route types
// Ported from TS RoutePath, RouteOptions, etc.
// ============================================================================

/// Route path information.
/// Ported from TS `interface RoutePath`.
#[derive(Debug, Clone)]
pub struct RoutePath {
    /// The route path string
    pub path: String,
    /// Whether this is a shared route
    pub shared: bool,
}

// ============================================================================
// HTTP Operation parameter types
// Ported from TS HeaderFieldOptions, QueryParameterOptions, etc.
// ============================================================================

/// Resolved header field options.
/// Ported from TS `interface HeaderFieldOptions`.
#[derive(Debug, Clone)]
pub struct HeaderFieldOptions {
    /// Parameter kind (always "header")
    pub param_type: &'static str,
    /// Header name
    pub name: String,
    /// Whether to explode array/object values
    pub explode: Option<bool>,
}

impl HeaderFieldOptions {
    /// Create new header field options
    pub fn new(name: String, explode: Option<bool>) -> Self {
        Self {
            param_type: "header",
            name,
            explode,
        }
    }
}

/// Resolved cookie parameter options.
/// Ported from TS `interface CookieParameterOptions`.
#[derive(Debug, Clone)]
pub struct CookieParameterOptions {
    /// Parameter kind (always "cookie")
    pub param_type: &'static str,
    /// Cookie name
    pub name: String,
}

impl CookieParameterOptions {
    /// Create new cookie parameter options
    pub fn new(name: String) -> Self {
        Self {
            param_type: "cookie",
            name,
        }
    }
}

/// Resolved query parameter options.
/// Ported from TS `interface QueryParameterOptions`.
#[derive(Debug, Clone)]
pub struct QueryParameterOptions {
    /// Parameter kind (always "query")
    pub param_type: &'static str,
    /// Query parameter name
    pub name: String,
    /// Whether to explode array/object values
    pub explode: Option<bool>,
}

impl QueryParameterOptions {
    /// Create new query parameter options
    pub fn new(name: String, explode: Option<bool>) -> Self {
        Self {
            param_type: "query",
            name,
            explode,
        }
    }
}

/// Resolve query options with defaults.
/// Ported from TS `resolveQueryOptionsWithDefaults()`.
/// Fills in default values: explode defaults to false.
pub fn resolve_query_options_with_defaults(
    options: &QueryOptions,
    name: &str,
) -> QueryParameterOptions {
    QueryParameterOptions {
        param_type: "query",
        name: options.name.as_deref().unwrap_or(name).to_string(),
        explode: Some(options.explode.unwrap_or(false)),
    }
}

/// Resolve path options with defaults.
/// Ported from TS `resolvePathOptionsWithDefaults()`.
/// Fills in default values: explode=false, allowReserved=false, style=simple.
pub fn resolve_path_options_with_defaults(
    options: &PathOptions,
    name: &str,
) -> PathParameterOptions {
    PathParameterOptions {
        param_type: "path",
        name: options.name.as_deref().unwrap_or(name).to_string(),
        explode: Some(options.explode.unwrap_or(false)),
        style: Some(options.style.unwrap_or(PathStyle::Simple)),
        allow_reserved: Some(options.allow_reserved.unwrap_or(false)),
    }
}

/// Resolved path parameter options.
/// Ported from TS `interface PathParameterOptions`.
#[derive(Debug, Clone)]
pub struct PathParameterOptions {
    /// Parameter kind (always "path")
    pub param_type: &'static str,
    /// Path parameter name
    pub name: String,
    /// Whether to explode values
    pub explode: Option<bool>,
    /// Interpolation style
    pub style: Option<PathStyle>,
    /// Whether to allow reserved characters
    pub allow_reserved: Option<bool>,
}

impl PathParameterOptions {
    /// Create new path parameter options
    pub fn new(
        name: String,
        explode: Option<bool>,
        style: Option<PathStyle>,
        allow_reserved: Option<bool>,
    ) -> Self {
        Self {
            param_type: "path",
            name,
            explode,
            style,
            allow_reserved,
        }
    }
}

/// HTTP operation parameter (header, cookie, query, or path).
/// Ported from TS `type HttpOperationParameter`.
#[derive(Debug, Clone)]
pub enum HttpOperationParameter {
    /// Header parameter
    Header(HttpOperationHeaderParameter),
    /// Cookie parameter
    Cookie(HttpOperationCookieParameter),
    /// Query parameter
    Query(HttpOperationQueryParameter),
    /// Path parameter
    Path(HttpOperationPathParameter),
}

/// HTTP operation header parameter.
/// Ported from TS `type HttpOperationHeaderParameter`.
#[derive(Debug, Clone)]
pub struct HttpOperationHeaderParameter {
    /// Header options
    pub options: HeaderFieldOptions,
    /// The model property TypeId this parameter comes from
    pub param: TypeId,
}

/// HTTP operation cookie parameter.
/// Ported from TS `type HttpOperationCookieParameter`.
#[derive(Debug, Clone)]
pub struct HttpOperationCookieParameter {
    /// Cookie options
    pub options: CookieParameterOptions,
    /// The model property TypeId this parameter comes from
    pub param: TypeId,
}

/// HTTP operation query parameter.
/// Ported from TS `type HttpOperationQueryParameter`.
#[derive(Debug, Clone)]
pub struct HttpOperationQueryParameter {
    /// Query options
    pub options: QueryParameterOptions,
    /// The model property TypeId this parameter comes from
    pub param: TypeId,
}

/// HTTP operation path parameter.
/// Ported from TS `type HttpOperationPathParameter`.
#[derive(Debug, Clone)]
pub struct HttpOperationPathParameter {
    /// Path options
    pub options: PathParameterOptions,
    /// The model property TypeId this parameter comes from
    pub param: TypeId,
}

// ============================================================================
// HTTP Server types
// Ported from TS HttpServer interface
// ============================================================================

/// HTTP server definition.
/// Ported from TS HttpServer interface.
#[derive(Debug, Clone, PartialEq)]
pub struct HttpServer {
    /// Server URL (may contain template parameters)
    pub url: String,
    /// Description of the server
    pub description: Option<String>,
    /// Parameter types (TypeId references)
    pub parameters: Vec<TypeId>,
}

/// Apply @server decorator.
/// Ported from TS $server().
pub fn apply_server(
    state: &mut StateAccessors,
    target: TypeId,
    url: &str,
    description: Option<&str>,
) {
    // Use \x01 (SOH) as separator - cannot appear in URLs, unlike "::" which
    // breaks IPv6 addresses like https://[::1]:3000
    let value = format!("{}\x01{}", url, description.unwrap_or(""));
    state.set_state(STATE_SERVERS, target, value);
}

/// Get servers for a namespace.
/// Ported from TS getServers().
/// Returns the URL and description from the stored server state.
pub fn get_server(state: &StateAccessors, target: TypeId) -> Option<HttpServer> {
    state.get_state(STATE_SERVERS, target).map(|s| {
        let parts: Vec<&str> = s.splitn(2, '\x01').collect();
        let url = parts.first().unwrap_or(&"").to_string();
        let description = parts
            .get(1)
            .filter(|d| !d.is_empty())
            .map(|d| d.to_string());
        HttpServer {
            url,
            description,
            parameters: vec![],
        }
    })
}

// ============================================================================
// Status code types
// Re-exported from status_codes module
// ============================================================================

pub use crate::libs::status_codes::HttpStatusCodeRange;
pub use crate::libs::status_codes::StatusCodeEntry as HttpStatusCodesEntry;

/// Set status codes programmatically.
/// Ported from TS setStatusCode().
pub fn set_status_codes(state: &mut StateAccessors, target: TypeId, codes: &[&str]) {
    state.set_state(STATE_STATUS_CODE, target, codes.join(","));
}

/// Get status codes for a property.
/// Ported from TS getStatusCodes().
pub fn get_status_codes(state: &StateAccessors, target: TypeId) -> Vec<HttpStatusCodesEntry> {
    state
        .get_state(STATE_STATUS_CODE, target)
        .map(|s| {
            s.split(',')
                .filter_map(|code| {
                    let code = code.trim();
                    if code == "*" {
                        Some(HttpStatusCodesEntry::Wildcard)
                    } else if code.ends_with("xx") {
                        // Parse range like "2xx" -> HttpStatusCodeRange
                        let start = match code {
                            "1xx" => 100,
                            "2xx" => 200,
                            "3xx" => 300,
                            "4xx" => 400,
                            "5xx" => 500,
                            _ => return None,
                        };
                        let end = start + 99;
                        Some(HttpStatusCodesEntry::Range(HttpStatusCodeRange::new(
                            start, end,
                        )?))
                    } else {
                        code.parse::<u16>().ok().map(HttpStatusCodesEntry::Code)
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Get description for a status code.
/// Delegates to `status_codes::get_status_code_description()`.
pub fn get_status_code_description(code: u16) -> Option<&'static str> {
    crate::libs::status_codes::get_status_code_description(code)
}

// ============================================================================
// Patch decorator options
// Ported from TS $patch / getPatchOptions
// ============================================================================

/// Apply @patch decorator with options.
/// Ported from TS $patch().
pub fn apply_patch(state: &mut StateAccessors, target: TypeId, options: Option<&PatchOptions>) {
    let opt_str = match options {
        Some(o) => format!("{}", o.implicit_optionality.unwrap_or(true)),
        None => "true".to_string(),
    };
    state.set_state(STATE_PATCH_OPTIONS, target, opt_str);
    state.set_state(STATE_VERBS, target, HttpVerb::Patch.as_str().to_string());
}

/// Get @patch options for an operation.
/// Ported from TS getPatchOptions().
pub fn get_patch_options(state: &StateAccessors, target: TypeId) -> Option<PatchOptions> {
    state.get_state(STATE_PATCH_OPTIONS, target).map(|s| {
        let implicit = s.parse::<bool>().unwrap_or(true);
        PatchOptions {
            implicit_optionality: Some(implicit),
        }
    })
}

// ============================================================================
// HTTP Service type
// Ported from TS HttpService interface
// ============================================================================

/// HTTP service definition.
/// Ported from TS HttpService interface.
#[derive(Debug, Clone)]
pub struct HttpService {
    /// The service namespace TypeId
    pub namespace: TypeId,
    /// Service title
    pub title: Option<String>,
    /// Service version
    pub version: Option<String>,
    /// Authentication configuration
    pub authentication: Option<Authentication>,
    /// Server list
    pub servers: Vec<HttpServer>,
}

// ============================================================================
// HTTP Operation types
// Ported from http/src/types.ts
// ============================================================================

/// An HTTP operation with resolved route, parameters, and responses.
/// Ported from TS `interface HttpOperation`.
#[derive(Debug, Clone)]
pub struct HttpOperation {
    /// The fully resolved URI template as defined by RFC 6570
    pub uri_template: String,
    /// Route path
    pub path: String,
    /// HTTP verb
    pub verb: HttpVerb,
    /// Parent type (interface or namespace) TypeId
    pub container: TypeId,
    /// Parameters
    pub parameters: HttpOperationParameters,
    /// Responses
    pub responses: Vec<HttpOperationResponse>,
    /// Operation TypeId
    pub operation: TypeId,
    /// Operation authentication override
    pub authentication: Option<Authentication>,
    /// Overload base operation
    pub overloading: Option<TypeId>,
    /// Operations that overload this one
    pub overloads: Vec<TypeId>,
}

/// HTTP operation parameters.
/// Ported from TS `interface HttpOperationParameters`.
#[derive(Debug, Clone)]
pub struct HttpOperationParameters {
    /// HTTP properties
    pub properties: Vec<HttpProperty>,
    /// HTTP operation parameters (header, cookie, query, path)
    pub parameters: Vec<HttpOperationParameter>,
    /// Request body
    pub body: Option<HttpPayloadBody>,
    /// The verb (determined during parameter processing)
    pub verb: HttpVerb,
}

/// HTTP property kind.
/// Ported from TS `type HttpProperty`.
#[derive(Debug, Clone)]
pub enum HttpProperty {
    /// Header property
    Header(HeaderPropertyData),
    /// Cookie property
    Cookie(CookiePropertyData),
    /// Content-Type property
    ContentType(ContentTypePropertyData),
    /// Query property
    Query(QueryPropertyData),
    /// Path property
    Path(PathPropertyData),
    /// Status code property
    StatusCode(StatusCodePropertyData),
    /// Body property
    Body(BodyPropertyData),
    /// Body root property
    BodyRoot(BodyRootPropertyData),
    /// Multipart body property
    MultipartBody(MultipartBodyPropertyData),
    /// Body property (included in body)
    BodyProperty(BodyPropertyPropertyData),
}

/// Base data for HTTP properties.
#[derive(Debug, Clone)]
pub struct HttpPropertyBase {
    /// The model property TypeId
    pub property: TypeId,
    /// Path from root operation to this property
    pub path: Vec<PathSegment>,
}

/// A path segment (string key or numeric index).
#[derive(Debug, Clone)]
pub enum PathSegment {
    /// String key
    Key(String),
    /// Numeric index
    Index(usize),
}

/// Header property data.
#[derive(Debug, Clone)]
pub struct HeaderPropertyData {
    /// Base data
    pub base: HttpPropertyBase,
    /// Header options
    pub options: HeaderFieldOptions,
}

/// Cookie property data.
#[derive(Debug, Clone)]
pub struct CookiePropertyData {
    /// Base data
    pub base: HttpPropertyBase,
    /// Cookie options
    pub options: CookieParameterOptions,
}

/// Content-Type property data.
#[derive(Debug, Clone)]
pub struct ContentTypePropertyData {
    /// Base data
    pub base: HttpPropertyBase,
}

/// Query property data.
#[derive(Debug, Clone)]
pub struct QueryPropertyData {
    /// Base data
    pub base: HttpPropertyBase,
    /// Query options
    pub options: QueryParameterOptions,
}

/// Path property data.
#[derive(Debug, Clone)]
pub struct PathPropertyData {
    /// Base data
    pub base: HttpPropertyBase,
    /// Path options
    pub options: PathParameterOptions,
}

/// Status code property data.
#[derive(Debug, Clone)]
pub struct StatusCodePropertyData {
    /// Base data
    pub base: HttpPropertyBase,
}

/// Body property data.
#[derive(Debug, Clone)]
pub struct BodyPropertyData {
    /// Base data
    pub base: HttpPropertyBase,
}

/// Body root property data.
#[derive(Debug, Clone)]
pub struct BodyRootPropertyData {
    /// Base data
    pub base: HttpPropertyBase,
}

/// Multipart body property data.
#[derive(Debug, Clone)]
pub struct MultipartBodyPropertyData {
    /// Base data
    pub base: HttpPropertyBase,
}

/// Body property (included in body) data.
#[derive(Debug, Clone)]
pub struct BodyPropertyPropertyData {
    /// Base data
    pub base: HttpPropertyBase,
}

/// The possible bodies of an HTTP operation.
/// Ported from TS `type HttpPayloadBody`.
#[derive(Debug, Clone)]
pub enum HttpPayloadBody {
    /// Single body
    Single(HttpOperationBody),
    /// Multipart body
    Multipart(HttpOperationMultipartBody),
    /// File body
    File(HttpOperationFileBody),
}

/// HTTP operation body (single).
/// Ported from TS `interface HttpOperationBody`.
#[derive(Debug, Clone)]
pub struct HttpOperationBody {
    /// Content types
    pub content_types: Vec<String>,
    /// Content type property TypeId
    pub content_type_property: Option<TypeId>,
    /// The payload property that defined this body
    pub property: Option<TypeId>,
    /// The body type
    pub body_type: TypeId,
    /// If the body was explicitly set with @body
    pub is_explicit: bool,
    /// If the body contains metadata annotations to ignore
    pub contains_metadata_annotations: bool,
    /// Body kind (always "single")
    pub body_kind: &'static str,
}

/// HTTP operation multipart body.
/// Ported from TS `type HttpOperationMultipartBody`.
#[derive(Debug, Clone)]
pub struct HttpOperationMultipartBody {
    /// Content types
    pub content_types: Vec<String>,
    /// Content type property TypeId
    pub content_type_property: Option<TypeId>,
    /// Property annotated with @multipartBody
    pub property: TypeId,
    /// Multipart kind
    pub multipart_kind: MultipartKind,
    /// The type (model or tuple)
    pub model_type: TypeId,
    /// Parts
    pub parts: Vec<HttpOperationPart>,
    /// Body kind (always "multipart")
    pub body_kind: &'static str,
}

/// Multipart body kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MultipartKind {
    /// Model-based multipart
    Model,
    /// Tuple-based multipart
    Tuple,
}

/// HTTP operation part.
/// Ported from TS `type HttpOperationPart`.
#[derive(Debug, Clone)]
pub enum HttpOperationPart {
    /// Model part
    Model(HttpOperationModelPart),
    /// Tuple part
    Tuple(HttpOperationTuplePart),
}

/// HTTP operation model part.
/// Ported from TS `interface HttpOperationModelPart`.
#[derive(Debug, Clone)]
pub struct HttpOperationModelPart {
    /// Part body
    pub body: HttpPayloadBody,
    /// Filename property TypeId
    pub filename: Option<TypeId>,
    /// Part headers
    pub headers: Vec<HeaderPropertyData>,
    /// If multiple of this part
    pub multi: bool,
    /// Part name
    pub name: String,
    /// If the part is optional
    pub optional: bool,
    /// Property TypeId
    pub property: TypeId,
}

/// HTTP operation tuple part.
/// Ported from TS `interface HttpOperationTuplePart`.
#[derive(Debug, Clone)]
pub struct HttpOperationTuplePart {
    /// Part body
    pub body: HttpPayloadBody,
    /// Filename property TypeId
    pub filename: Option<TypeId>,
    /// Part headers
    pub headers: Vec<HeaderPropertyData>,
    /// If multiple of this part
    pub multi: bool,
    /// Part name
    pub name: Option<String>,
    /// If the part is optional
    pub optional: bool,
}

/// HTTP operation file body.
/// Ported from TS `interface HttpOperationFileBody`.
#[derive(Debug, Clone)]
pub struct HttpOperationFileBody {
    /// Content types
    pub content_types: Vec<String>,
    /// Content type property TypeId
    pub content_type_property: TypeId,
    /// The model type (is or extends Http.File)
    pub model_type: TypeId,
    /// Whether file contents are textual
    pub is_text: bool,
    /// Filename property TypeId
    pub filename: TypeId,
    /// Contents property TypeId
    pub contents: TypeId,
    /// Body kind (always "file")
    pub body_kind: &'static str,
}

/// HTTP operation response.
/// Ported from TS `interface HttpOperationResponse`.
#[derive(Debug, Clone)]
pub struct HttpOperationResponse {
    /// Status code or range
    pub status_codes: HttpStatusCodesEntry,
    /// Response type TypeId
    pub response_type: TypeId,
    /// Response description
    pub description: Option<String>,
    /// Response contents
    pub responses: Vec<HttpOperationResponseContent>,
}

/// HTTP operation response content.
/// Ported from TS `interface HttpOperationResponseContent`.
#[derive(Debug, Clone)]
pub struct HttpOperationResponseContent {
    /// HTTP properties for this response
    pub properties: Vec<HttpProperty>,
    /// Response headers
    pub headers: Vec<(String, TypeId)>,
    /// Response body
    pub body: Option<HttpPayloadBody>,
}

/// HTTP payload disposition (request, response, or multipart).
/// Ported from TS `enum HttpPayloadDisposition`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpPayloadDisposition {
    /// Request payload
    Request,
    /// Response payload
    Response,
    /// Multipart payload
    Multipart,
}
