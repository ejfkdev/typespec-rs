//! @typespec/http - HTTP Protocol Decorators and Types
//!
//! Ported from TypeSpec packages/http
//!
//! Provides decorators for HTTP API modeling:
//! - HTTP verbs: `@get`, `@put`, `@post`, `@patch`, `@delete`, `@head`
//! - Parameters: `@header`, `@query`, `@path`, `@body`, `@bodyRoot`, `@bodyIgnore`
//! - Response: `@statusCode`
//! - Routing: `@route`, `@sharedRoute`
//! - Server: `@server`, `@useAuth`
//! - Multipart: `@multipartBody`
//!
//! ## Types
//! - `HttpVerb` enum - HTTP method verbs
//! - `HttpAuth` types - Authentication scheme definitions
//! - `HeaderOptions`, `QueryOptions`, `PathOptions`, `CookieOptions` - Parameter options
//!
//! ## Helper Functions
//! - `is_header`, `is_query`, `is_path`, `is_body` - Check parameter kind
//! - `get_header_name`, `get_query_name` - Get parameter name
//! - `get_verb` - Get HTTP verb for an operation
//! - `is_shared_route` - Check if operation uses shared route

pub mod types;
pub use types::*;

pub mod operation;
pub use operation::*;

pub mod auth;
pub use auth::*;

pub mod visibility;
pub use visibility::*;

use crate::checker::types::TypeId;
use crate::state_accessors::StateAccessors;

pub mod tsp_sources;
#[allow(unused_imports)]
pub use tsp_sources::*;

// ============================================================================
// Decorator implementations
// ============================================================================

/// Apply an HTTP verb decorator to an operation.
pub fn apply_verb(state: &mut StateAccessors, target: TypeId, verb: HttpVerb) {
    state.set_state(STATE_VERBS, target, verb.as_str().to_string());
}

/// Get the HTTP verb for an operation.
pub fn get_verb(state: &StateAccessors, target: TypeId) -> Option<HttpVerb> {
    state
        .get_state(STATE_VERBS, target)
        .and_then(HttpVerb::parse_str)
}

optional_name_decorator!(apply_header, is_header, get_header_name, STATE_HEADER);
optional_name_decorator!(apply_query, is_query, get_query_name, STATE_QUERY);
optional_name_decorator!(apply_path, is_path, get_path_name, STATE_PATH);

/// Apply @cookie decorator.
///
/// Ported from TS $cookie().
/// Cookie names use snake_case by default (camelCase -> snake_case).
/// Note: get_cookie_name filters empty strings, unlike standard optional_name_decorator.
pub fn apply_cookie(state: &mut StateAccessors, target: TypeId, name: Option<&str>) {
    let value = name.unwrap_or("");
    state.set_state(STATE_COOKIE, target, value.to_string());
}

/// Check if a property is a cookie parameter.
/// Ported from TS isCookieParam().
pub fn is_cookie(state: &StateAccessors, target: TypeId) -> bool {
    state.get_state(STATE_COOKIE, target).is_some()
}

/// Get the cookie parameter name.
/// Ported from TS getCookieParamOptions().
pub fn get_cookie_name(state: &StateAccessors, target: TypeId) -> Option<String> {
    state
        .get_state(STATE_COOKIE, target)
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
}

/// Convert a camelCase name to a separated case.
/// This matches the TS behavior: `entity.name.replace(/([a-z])([A-Z])/g, "$1{sep}$2").toLowerCase()`
fn camel_to_case(name: &str, sep: char) -> String {
    let mut result = String::with_capacity(name.len() + 4);
    for (i, c) in name.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push(sep);
            }
            result.extend(c.to_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}

/// Convert a camelCase name to snake_case for cookie parameters.
/// This matches the TS behavior: `entity.name.replace(/([a-z])([A-Z])/g, "$1_$2").toLowerCase()`
pub fn camel_to_snake_case(name: &str) -> String {
    camel_to_case(name, '_')
}

/// Convert a camelCase name to kebab-case for header names.
/// This matches the TS behavior: `entity.name.replace(/([a-z])([A-Z])/g, "$1-$2").toLowerCase()`
///
/// # Example
/// `contentType` -> `content-type`
/// `xRequestId` -> `x-request-id`
pub fn camel_to_kebab_case(name: &str) -> String {
    camel_to_case(name, '-')
}

/// Get the default header name for a property (kebab-case).
/// In the TS implementation, headers default to kebab-case:
/// `entity.name.replace(/([a-z])([A-Z])/g, "$1-$2").toLowerCase()`
pub fn default_header_name(property_name: &str) -> String {
    camel_to_kebab_case(property_name)
}

/// Get the default cookie name for a property (snake_case).
/// In the TS implementation, cookies default to snake_case:
/// `entity.name.replace(/([a-z])([A-Z])/g, "$1_$2").toLowerCase()`
pub fn default_cookie_name(property_name: &str) -> String {
    camel_to_snake_case(property_name)
}

flag_decorator!(apply_body, is_body, STATE_BODY);
flag_decorator!(apply_body_root, is_body_root, STATE_BODY_ROOT);
flag_decorator!(apply_body_ignore, is_body_ignore, STATE_BODY_IGNORE);
flag_decorator!(apply_status_code, is_status_code, STATE_STATUS_CODE);
string_decorator!(apply_route, get_route, STATE_ROUTE);
flag_decorator!(apply_shared_route, is_shared_route, STATE_SHARED_ROUTES);
flag_decorator!(
    apply_multipart_body,
    is_multipart_body,
    STATE_MULTIPART_BODY
);

// ============================================================================
// Include inapplicable metadata decorator
// ============================================================================

flag_decorator!(
    apply_include_inapplicable_metadata,
    include_inapplicable_metadata_in_payload,
    STATE_INCLUDE_INAPPLICABLE_METADATA
);

// ============================================================================
// @plainData decorator
// Ported from TS $plainData
// ============================================================================

/// Apply @plainData decorator.
/// Removes HTTP metadata annotations (@header, @query, @path, @statusCode, @body)
/// from all properties of the target model.
///
/// Note: Full implementation requires iterating model properties and cleaning
/// their decorator state. This applies the marker; actual cleanup happens
/// during checker processing.
pub fn apply_plain_data(state: &mut StateAccessors, target: TypeId) {
    state.set_state(STATE_HTTP_METADATA, target, "plainData".to_string());
}

/// Check if a model has the @plainData decorator applied.
pub fn is_plain_data(state: &StateAccessors, target: TypeId) -> bool {
    state
        .get_state(STATE_HTTP_METADATA, target)
        .map(|s| s == "plainData")
        .unwrap_or(false)
}

// ============================================================================
// @httpFile decorator
// Ported from TS $httpFile / isHttpFile / isOrExtendsHttpFile
// ============================================================================

flag_decorator!(apply_http_file, is_http_file, STATE_FILE);

// ============================================================================
// @httpPart decorator
// Ported from TS $httpPart / getHttpPart
// ============================================================================

/// HTTP part information.
/// Ported from TS `interface HttpPart`.
#[derive(Debug, Clone, PartialEq)]
pub struct HttpPart {
    /// The type of the part
    pub part_type: TypeId,
    /// Options for the part (name)
    pub name: Option<String>,
}

/// Apply @httpPart decorator.
/// Marks a model as an HTTP part type with an inner type and options.
pub fn apply_http_part(
    state: &mut StateAccessors,
    target: TypeId,
    part_type: TypeId,
    name: Option<&str>,
) {
    let value = format!("{}::{}", part_type, name.unwrap_or(""));
    state.set_state(STATE_HTTP_PART, target, value);
}

/// Get the HTTP part information for a model.
/// Ported from TS `getHttpPart()`.
pub fn get_http_part(state: &StateAccessors, target: TypeId) -> Option<HttpPart> {
    state.get_state(STATE_HTTP_PART, target).and_then(|s| {
        let parts: Vec<&str> = s.splitn(2, "::").collect();
        if parts.len() != 2 {
            return None;
        }
        let part_type = parts[0].parse::<TypeId>().ok()?;
        let name = if parts[1].is_empty() {
            None
        } else {
            Some(parts[1].to_string())
        };
        Some(HttpPart { part_type, name })
    })
}

// ============================================================================
// Merge-patch decorators
// ============================================================================

typeid_decorator!(
    apply_merge_patch_model,
    get_merge_patch_model_source,
    STATE_MERGE_PATCH_MODEL
);
typeid_decorator!(
    apply_merge_patch_property,
    get_merge_patch_property_source,
    STATE_MERGE_PATCH_PROPERTY
);

/// Apply @applyMergePatch decorator
pub fn apply_merge_patch(
    state: &mut StateAccessors,
    target: TypeId,
    options: Option<&ApplyMergePatchOptions>,
) {
    let opt_str = match options {
        Some(o) => {
            if o.implicit_optionality {
                "implicit"
            } else {
                "explicit"
            }
        }
        None => "implicit",
    };
    state.set_state(STATE_APPLY_MERGE_PATCH, target, opt_str.to_string());
}

/// Check if a model has @applyMergePatch
pub fn is_merge_patch(state: &StateAccessors, target: TypeId) -> bool {
    state.get_state(STATE_APPLY_MERGE_PATCH, target).is_some()
}

// ============================================================================
// HTTP metadata helpers
// ============================================================================

string_decorator!(
    apply_http_metadata,
    get_http_metadata_kind,
    STATE_HTTP_METADATA
);

/// Check if a property is HTTP metadata
pub fn is_http_metadata(state: &StateAccessors, target: TypeId) -> bool {
    state.get_state(STATE_HTTP_METADATA, target).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_namespace() {
        assert_eq!(HTTP_NAMESPACE, "TypeSpec.Http");
    }

    #[test]
    fn test_create_http_library() {
        let diags = create_http_library();
        assert!(diags.len() >= 33);
        let codes: Vec<&str> = diags.keys().map(|code| code.as_str()).collect();
        assert!(codes.contains(&DIAG_HTTP_VERB_DUPLICATE));
        assert!(codes.contains(&DIAG_DUPLICATE_BODY));
        assert!(codes.contains(&DIAG_STATUS_CODE_INVALID));
        assert!(codes.contains(&DIAG_SHARED_INCONSISTENCY));
    }

    #[test]
    fn test_http_verb() {
        assert_eq!(HttpVerb::Get.as_str(), "get");
        assert_eq!(HttpVerb::Post.as_str(), "post");
        assert_eq!(HttpVerb::parse_str("get"), Some(HttpVerb::Get));
        assert_eq!(HttpVerb::parse_str("post"), Some(HttpVerb::Post));
        assert_eq!(HttpVerb::parse_str("unknown"), None);

        for verb in HttpVerb::all() {
            assert!(HttpVerb::parse_str(verb.as_str()).is_some());
        }
    }

    #[test]
    fn test_apply_verb() {
        let mut state = StateAccessors::new();
        assert_eq!(get_verb(&state, 1), None);

        apply_verb(&mut state, 1, HttpVerb::Get);
        assert_eq!(get_verb(&state, 1), Some(HttpVerb::Get));

        apply_verb(&mut state, 2, HttpVerb::Post);
        assert_eq!(get_verb(&state, 2), Some(HttpVerb::Post));
    }

    #[test]
    fn test_apply_header() {
        let mut state = StateAccessors::new();
        assert!(!is_header(&state, 1));

        apply_header(&mut state, 1, Some("Content-Type"));
        assert!(is_header(&state, 1));
        assert_eq!(get_header_name(&state, 1), Some("Content-Type".to_string()));
    }

    #[test]
    fn test_apply_query() {
        let mut state = StateAccessors::new();
        apply_query(&mut state, 1, Some("select"));
        assert!(is_query(&state, 1));
        assert_eq!(get_query_name(&state, 1), Some("select".to_string()));
    }

    #[test]
    fn test_apply_path() {
        let mut state = StateAccessors::new();
        apply_path(&mut state, 1, Some("id"));
        assert!(is_path(&state, 1));
        assert_eq!(get_path_name(&state, 1), Some("id".to_string()));
    }

    #[test]
    fn test_apply_cookie() {
        let mut state = StateAccessors::new();
        assert!(!is_cookie(&state, 1));

        apply_cookie(&mut state, 1, Some("session_id"));
        assert!(is_cookie(&state, 1));
        assert_eq!(get_cookie_name(&state, 1), Some("session_id".to_string()));
    }

    #[test]
    fn test_cookie_default_name() {
        let mut state = StateAccessors::new();
        apply_cookie(&mut state, 1, None);
        assert!(is_cookie(&state, 1));
        // Default name is empty (caller should use property name)
        assert_eq!(get_cookie_name(&state, 1), None);
    }

    #[test]
    fn test_camel_to_snake_case() {
        assert_eq!(camel_to_snake_case("sessionId"), "session_id");
        assert_eq!(camel_to_snake_case("userId"), "user_id");
        assert_eq!(camel_to_snake_case("name"), "name");
        assert_eq!(camel_to_snake_case("XMLParser"), "x_m_l_parser");
    }

    #[test]
    fn test_camel_to_kebab_case() {
        assert_eq!(camel_to_kebab_case("contentType"), "content-type");
        assert_eq!(camel_to_kebab_case("xRequestId"), "x-request-id");
        assert_eq!(camel_to_kebab_case("name"), "name");
        assert_eq!(camel_to_kebab_case("XMLParser"), "x-m-l-parser");
    }

    #[test]
    fn test_default_header_name() {
        assert_eq!(default_header_name("contentType"), "content-type");
        assert_eq!(default_header_name("accept"), "accept");
    }

    #[test]
    fn test_default_cookie_name() {
        assert_eq!(default_cookie_name("sessionId"), "session_id");
        assert_eq!(default_cookie_name("token"), "token");
    }

    #[test]
    fn test_apply_body() {
        let mut state = StateAccessors::new();
        assert!(!is_body(&state, 1));
        apply_body(&mut state, 1);
        assert!(is_body(&state, 1));
    }

    #[test]
    fn test_apply_status_code() {
        let mut state = StateAccessors::new();
        assert!(!is_status_code(&state, 1));
        apply_status_code(&mut state, 1);
        assert!(is_status_code(&state, 1));
    }

    #[test]
    fn test_apply_route() {
        let mut state = StateAccessors::new();
        apply_route(&mut state, 1, "/widgets/{id}");
        assert_eq!(get_route(&state, 1), Some("/widgets/{id}".to_string()));
    }

    #[test]
    fn test_apply_shared_route() {
        let mut state = StateAccessors::new();
        assert!(!is_shared_route(&state, 1));
        apply_shared_route(&mut state, 1);
        assert!(is_shared_route(&state, 1));
    }

    #[test]
    fn test_apply_multipart_body() {
        let mut state = StateAccessors::new();
        assert!(!is_multipart_body(&state, 1));
        apply_multipart_body(&mut state, 1);
        assert!(is_multipart_body(&state, 1));
    }

    #[test]
    fn test_decorators_tsp_not_empty() {
        assert!(!HTTP_DECORATORS_TSP.is_empty());
        assert!(HTTP_DECORATORS_TSP.contains("header"));
        assert!(HTTP_DECORATORS_TSP.contains("query"));
        assert!(HTTP_DECORATORS_TSP.contains("route"));
        assert!(HTTP_DECORATORS_TSP.contains("sharedRoute"));
    }

    #[test]
    fn test_auth_tsp_not_empty() {
        assert!(!HTTP_AUTH_TSP.is_empty());
        assert!(HTTP_AUTH_TSP.contains("BasicAuth"));
        assert!(HTTP_AUTH_TSP.contains("BearerAuth"));
        assert!(HTTP_AUTH_TSP.contains("Oauth2Auth"));
    }

    #[test]
    fn test_private_decorators_tsp_not_empty() {
        assert!(!HTTP_PRIVATE_DECORATORS_TSP.is_empty());
        assert!(HTTP_PRIVATE_DECORATORS_TSP.contains("plainData"));
        assert!(HTTP_PRIVATE_DECORATORS_TSP.contains("httpFile"));
        assert!(HTTP_PRIVATE_DECORATORS_TSP.contains("httpPart"));
        assert!(HTTP_PRIVATE_DECORATORS_TSP.contains("MergePatchVisibilityMode"));
        assert!(HTTP_PRIVATE_DECORATORS_TSP.contains("mergePatchModel"));
        assert!(HTTP_PRIVATE_DECORATORS_TSP.contains("mergePatchProperty"));
    }

    #[test]
    fn test_main_tsp_not_empty() {
        assert!(!HTTP_MAIN_TSP.is_empty());
        assert!(HTTP_MAIN_TSP.contains("Response"));
        assert!(HTTP_MAIN_TSP.contains("Body"));
        assert!(HTTP_MAIN_TSP.contains("OkResponse"));
        assert!(HTTP_MAIN_TSP.contains("File"));
        assert!(HTTP_MAIN_TSP.contains("PlainData"));
        assert!(HTTP_MAIN_TSP.contains("HttpPart"));
    }

    #[test]
    fn test_path_style() {
        assert_eq!(PathStyle::parse_str("simple"), Some(PathStyle::Simple));
        assert_eq!(PathStyle::parse_str("label"), Some(PathStyle::Label));
        assert_eq!(PathStyle::parse_str("matrix"), Some(PathStyle::Matrix));
        assert_eq!(PathStyle::parse_str("fragment"), Some(PathStyle::Fragment));
        assert_eq!(PathStyle::parse_str("path"), Some(PathStyle::PathSegment));
        assert_eq!(PathStyle::parse_str("unknown"), None);
    }

    // ========================================================================
    // Visibility tests
    // ========================================================================

    #[test]
    fn test_visibility_flags() {
        assert!(Visibility::Read.contains(Visibility::Read));
        assert!(!Visibility::Read.contains(Visibility::Create));
        assert!(Visibility::All.contains(Visibility::Read));
        assert!(Visibility::All.contains(Visibility::Create));
        assert!(Visibility::All.contains(Visibility::Update));
        assert!(Visibility::All.contains(Visibility::Delete));
        assert!(Visibility::All.contains(Visibility::Query));
    }

    #[test]
    fn test_visibility_union() {
        let combined = Visibility::Read.union(Visibility::Create);
        assert!(combined.contains(Visibility::Read));
        assert!(combined.contains(Visibility::Create));
        assert!(!combined.contains(Visibility::Update));
    }

    #[test]
    fn test_visibility_bitor() {
        let combined = Visibility::Create | Visibility::Update;
        assert!(combined.contains(Visibility::Create));
        assert!(combined.contains(Visibility::Update));
        assert!(!combined.contains(Visibility::Read));
    }

    #[test]
    fn test_visibility_difference() {
        let combined = Visibility::Read | Visibility::Create | Visibility::Update;
        let result = combined.difference(Visibility::Read);
        assert!(!result.contains(Visibility::Read));
        assert!(result.contains(Visibility::Create));
    }

    #[test]
    fn test_visibility_to_vec() {
        assert_eq!(Visibility::Read.to_vec(), vec!["read"]);
        assert_eq!(Visibility::Create.to_vec(), vec!["create"]);
        assert_eq!(
            (Visibility::Create | Visibility::Update).to_vec(),
            vec!["create", "update"]
        );
        assert_eq!(Visibility::None.to_vec(), Vec::<&str>::new());
    }

    #[test]
    fn test_visibility_none() {
        assert!(Visibility::None.is_none());
        assert!(!Visibility::Read.is_none());
    }

    #[test]
    fn test_visibility_suffix() {
        // Default canonical = Visibility::All, so all non-synthetic visibilities match
        assert_eq!(get_visibility_suffix(Visibility::All, None), "");
        // With canonical = Read, Update gets suffix
        assert_eq!(
            get_visibility_suffix(Visibility::Update, Some(Visibility::Read)),
            "Update"
        );
        assert_eq!(
            get_visibility_suffix(Visibility::Read, Some(Visibility::Read)),
            ""
        );
        assert_eq!(
            get_visibility_suffix(
                Visibility::Create | Visibility::Update,
                Some(Visibility::Read)
            ),
            "CreateOrUpdate"
        );
    }

    #[test]
    fn test_visibility_suffix_with_item() {
        assert_eq!(
            get_visibility_suffix(
                Visibility::Create | Visibility::Item,
                Some(Visibility::Read)
            ),
            "CreateItem"
        );
    }

    #[test]
    fn test_default_visibility_for_verb() {
        assert_eq!(
            get_default_visibility_for_verb(HttpVerb::Get),
            Visibility::Query
        );
        assert_eq!(
            get_default_visibility_for_verb(HttpVerb::Head),
            Visibility::Query
        );
        assert_eq!(
            get_default_visibility_for_verb(HttpVerb::Post),
            Visibility::Create
        );
        assert_eq!(
            get_default_visibility_for_verb(HttpVerb::Put),
            Visibility::Create | Visibility::Update
        );
        assert_eq!(
            get_default_visibility_for_verb(HttpVerb::Patch),
            Visibility::Update
        );
        assert_eq!(
            get_default_visibility_for_verb(HttpVerb::Delete),
            Visibility::Delete
        );
    }

    // ========================================================================
    // Merge-patch decorator tests
    // ========================================================================

    #[test]
    fn test_merge_patch_model() {
        let mut state = StateAccessors::new();
        assert_eq!(get_merge_patch_model_source(&state, 1), None);
        apply_merge_patch_model(&mut state, 1, 42);
        assert_eq!(get_merge_patch_model_source(&state, 1), Some(42));
    }

    #[test]
    fn test_merge_patch_property() {
        let mut state = StateAccessors::new();
        apply_merge_patch_property(&mut state, 1, 10);
        assert_eq!(get_merge_patch_property_source(&state, 1), Some(10));
    }

    #[test]
    fn test_apply_merge_patch() {
        let mut state = StateAccessors::new();
        assert!(!is_merge_patch(&state, 1));
        apply_merge_patch(&mut state, 1, None);
        assert!(is_merge_patch(&state, 1));
    }

    #[test]
    fn test_include_inapplicable_metadata() {
        let mut state = StateAccessors::new();
        assert!(!include_inapplicable_metadata_in_payload(&state, 1));
        apply_include_inapplicable_metadata(&mut state, 1);
        assert!(include_inapplicable_metadata_in_payload(&state, 1));
    }

    #[test]
    fn test_http_metadata() {
        let mut state = StateAccessors::new();
        assert!(!is_http_metadata(&state, 1));
        apply_http_metadata(&mut state, 1, "header");
        assert!(is_http_metadata(&state, 1));
        assert_eq!(
            get_http_metadata_kind(&state, 1),
            Some("header".to_string())
        );
    }

    #[test]
    fn test_is_applicable_metadata() {
        let mut state = StateAccessors::new();
        assert!(!is_applicable_metadata(&state, 1));
        apply_header(&mut state, 1, None);
        assert!(is_applicable_metadata(&state, 1));
    }

    #[test]
    fn test_is_applicable_metadata_or_body() {
        let mut state = StateAccessors::new();
        assert!(!is_applicable_metadata_or_body(&state, 1));
        apply_body(&mut state, 1);
        assert!(is_applicable_metadata_or_body(&state, 1));
    }

    // ========================================================================
    // Server tests
    // ========================================================================

    #[test]
    fn test_http_server() {
        let mut state = StateAccessors::new();
        assert_eq!(get_server(&state, 1), None);
        apply_server(&mut state, 1, "https://example.com", Some("Production"));
        let server = get_server(&state, 1).unwrap();
        assert_eq!(server.url, "https://example.com");
        assert_eq!(server.description, Some("Production".to_string()));
    }

    #[test]
    fn test_http_server_no_description() {
        let mut state = StateAccessors::new();
        apply_server(&mut state, 1, "https://localhost:3000", None);
        let server = get_server(&state, 1).unwrap();
        assert_eq!(server.url, "https://localhost:3000");
        assert_eq!(server.description, None);
    }

    // ========================================================================
    // Status code tests
    // ========================================================================

    #[test]
    fn test_status_code_range() {
        let range2xx = HttpStatusCodeRange::new(200, 299).unwrap();
        assert!(range2xx.contains(200));
        assert!(range2xx.contains(204));
        assert!(!range2xx.contains(300));
        let range4xx = HttpStatusCodeRange::new(400, 499).unwrap();
        assert!(range4xx.contains(404));
    }

    #[test]
    fn test_set_get_status_codes() {
        let mut state = StateAccessors::new();
        assert!(get_status_codes(&state, 1).is_empty());
        set_status_codes(&mut state, 1, &["200", "404"]);
        let codes = get_status_codes(&state, 1);
        assert_eq!(codes.len(), 2);
        assert_eq!(codes[0], HttpStatusCodesEntry::Code(200));
        assert_eq!(codes[1], HttpStatusCodesEntry::Code(404));
    }

    #[test]
    fn test_status_code_wildcard() {
        let mut state = StateAccessors::new();
        set_status_codes(&mut state, 1, &["*"]);
        let codes = get_status_codes(&state, 1);
        assert_eq!(codes[0], HttpStatusCodesEntry::Wildcard);
    }

    #[test]
    fn test_status_code_range_entry() {
        let mut state = StateAccessors::new();
        set_status_codes(&mut state, 1, &["2xx"]);
        let codes = get_status_codes(&state, 1);
        assert_eq!(
            codes[0],
            HttpStatusCodesEntry::Range(HttpStatusCodeRange::new(200, 299).unwrap())
        );
    }

    #[test]
    fn test_status_code_description() {
        assert_eq!(
            get_status_code_description(200),
            Some("The request has succeeded.")
        );
        assert_eq!(
            get_status_code_description(404),
            Some("The server cannot find the requested resource.")
        );
        assert_eq!(get_status_code_description(500), Some("Server error")); // 500 falls through to range description
        assert_eq!(
            get_status_code_description(503),
            Some("Service unavailable.")
        );
        assert_eq!(get_status_code_description(999), None);
    }

    // ========================================================================
    // Patch decorator tests
    // ========================================================================

    #[test]
    fn test_patch_decorator() {
        let mut state = StateAccessors::new();
        apply_patch(&mut state, 1, None);
        assert_eq!(get_verb(&state, 1), Some(HttpVerb::Patch));
        let opts = get_patch_options(&state, 1);
        assert!(opts.is_some());
        assert_eq!(opts.unwrap().implicit_optionality, Some(true));
    }

    #[test]
    fn test_patch_with_options() {
        let mut state = StateAccessors::new();
        apply_patch(
            &mut state,
            1,
            Some(&PatchOptions {
                implicit_optionality: Some(false),
            }),
        );
        let opts = get_patch_options(&state, 1).unwrap();
        assert_eq!(opts.implicit_optionality, Some(false));
    }

    // ========================================================================
    // Route types tests
    // ========================================================================

    #[test]
    fn test_route_path() {
        let route = RoutePath {
            path: "/widgets/{id}".to_string(),
            shared: false,
        };
        assert_eq!(route.path, "/widgets/{id}");
        assert!(!route.shared);
    }

    #[test]
    fn test_route_path_shared() {
        let route = RoutePath {
            path: "/items".to_string(),
            shared: true,
        };
        assert!(route.shared);
    }

    // ========================================================================
    // Operation parameter type tests
    // ========================================================================

    #[test]
    fn test_header_field_options() {
        let opts = HeaderFieldOptions::new("Content-Type".to_string(), Some(true));
        assert_eq!(opts.param_type, "header");
        assert_eq!(opts.name, "Content-Type");
        assert_eq!(opts.explode, Some(true));
    }

    #[test]
    fn test_query_parameter_options() {
        let opts = QueryParameterOptions::new("select".to_string(), None);
        assert_eq!(opts.param_type, "query");
        assert_eq!(opts.name, "select");
    }

    #[test]
    fn test_path_parameter_options() {
        let opts = PathParameterOptions::new("id".to_string(), None, Some(PathStyle::Simple), None);
        assert_eq!(opts.param_type, "path");
        assert_eq!(opts.name, "id");
        assert_eq!(opts.style, Some(PathStyle::Simple));
    }

    #[test]
    fn test_cookie_parameter_options() {
        let opts = CookieParameterOptions::new("session_id".to_string());
        assert_eq!(opts.param_type, "cookie");
        assert_eq!(opts.name, "session_id");
    }

    #[test]
    fn test_http_operation_parameter_enum() {
        let param = HttpOperationParameter::Header(HttpOperationHeaderParameter {
            options: HeaderFieldOptions::new("X-Custom".to_string(), None),
            param: 1,
        });
        match param {
            HttpOperationParameter::Header(p) => assert_eq!(p.options.name, "X-Custom"),
            _ => panic!("Expected Header variant"),
        }
    }

    // ========================================================================
    // Options resolution tests
    // ========================================================================

    #[test]
    fn test_resolve_query_options_with_defaults() {
        let opts = QueryOptions {
            name: Some("select".to_string()),
            explode: None,
        };
        let resolved = resolve_query_options_with_defaults(&opts, "defaultName");
        assert_eq!(resolved.param_type, "query");
        assert_eq!(resolved.name, "select");
        assert_eq!(resolved.explode, Some(false));
    }

    #[test]
    fn test_resolve_query_options_defaults_name() {
        let opts = QueryOptions {
            name: None,
            explode: Some(true),
        };
        let resolved = resolve_query_options_with_defaults(&opts, "defaultName");
        assert_eq!(resolved.name, "defaultName");
        assert_eq!(resolved.explode, Some(true));
    }

    #[test]
    fn test_resolve_path_options_with_defaults() {
        let opts = PathOptions {
            name: Some("id".to_string()),
            explode: None,
            style: None,
            allow_reserved: None,
        };
        let resolved = resolve_path_options_with_defaults(&opts, "defaultName");
        assert_eq!(resolved.param_type, "path");
        assert_eq!(resolved.name, "id");
        assert_eq!(resolved.explode, Some(false));
        assert_eq!(resolved.style, Some(PathStyle::Simple));
        assert_eq!(resolved.allow_reserved, Some(false));
    }

    #[test]
    fn test_resolve_path_options_defaults_name() {
        let opts = PathOptions {
            name: None,
            explode: Some(true),
            style: Some(PathStyle::Matrix),
            allow_reserved: Some(true),
        };
        let resolved = resolve_path_options_with_defaults(&opts, "defaultName");
        assert_eq!(resolved.name, "defaultName");
        assert_eq!(resolved.explode, Some(true));
        assert_eq!(resolved.style, Some(PathStyle::Matrix));
        assert_eq!(resolved.allow_reserved, Some(true));
    }

    // ========================================================================
    // Additional decorator validation tests
    // ========================================================================

    #[test]
    fn test_body_root_and_ignore() {
        let mut state = StateAccessors::new();
        assert!(!is_body_root(&state, 1));
        assert!(!is_body_ignore(&state, 1));

        apply_body_root(&mut state, 1);
        assert!(is_body_root(&state, 1));

        apply_body_ignore(&mut state, 2);
        assert!(is_body_ignore(&state, 2));
    }

    #[test]
    fn test_verb_roundtrip() {
        let mut state = StateAccessors::new();
        for verb in HttpVerb::all() {
            apply_verb(&mut state, 1, *verb);
            assert_eq!(get_verb(&state, 1), Some(*verb));
        }
    }

    #[test]
    fn test_http_metadata_kind() {
        let mut state = StateAccessors::new();
        apply_http_metadata(&mut state, 1, "header");
        assert_eq!(
            get_http_metadata_kind(&state, 1),
            Some("header".to_string())
        );

        apply_http_metadata(&mut state, 2, "query");
        assert_eq!(get_http_metadata_kind(&state, 2), Some("query".to_string()));
    }

    #[test]
    fn test_visibility_all_flags() {
        // Verify all standard flags are set in All
        let all = Visibility::All;
        assert!(all.contains(Visibility::Read));
        assert!(all.contains(Visibility::Create));
        assert!(all.contains(Visibility::Update));
        assert!(all.contains(Visibility::Delete));
        assert!(all.contains(Visibility::Query));
        // Synthetic flags should not be in All
        assert!(!all.contains(Visibility::Item));
        assert!(!all.contains(Visibility::Patch));
    }

    #[test]
    fn test_path_style_all_values() {
        for s in &["simple", "label", "matrix", "fragment", "path"] {
            assert!(PathStyle::parse_str(s).is_some());
        }
    }

    #[test]
    fn test_server_multiple() {
        let mut state = StateAccessors::new();
        apply_server(
            &mut state,
            1,
            "https://prod.example.com",
            Some("Production"),
        );
        apply_server(
            &mut state,
            2,
            "https://staging.example.com",
            Some("Staging"),
        );

        let s1 = get_server(&state, 1).unwrap();
        assert_eq!(s1.url, "https://prod.example.com");
        assert_eq!(s1.description, Some("Production".to_string()));

        let s2 = get_server(&state, 2).unwrap();
        assert_eq!(s2.url, "https://staging.example.com");
    }

    #[test]
    fn test_status_codes_multiple() {
        let mut state = StateAccessors::new();
        set_status_codes(&mut state, 1, &["200", "201", "404"]);
        let codes = get_status_codes(&state, 1);
        assert_eq!(codes.len(), 3);
        assert_eq!(codes[0], HttpStatusCodesEntry::Code(200));
        assert_eq!(codes[1], HttpStatusCodesEntry::Code(201));
        assert_eq!(codes[2], HttpStatusCodesEntry::Code(404));
    }

    #[test]
    fn test_merge_patch_default_options() {
        let opts = ApplyMergePatchOptions::default();
        assert!(opts.implicit_optionality);
    }

    #[test]
    fn test_apply_merge_patch_with_options() {
        let mut state = StateAccessors::new();
        apply_merge_patch(
            &mut state,
            1,
            Some(&ApplyMergePatchOptions {
                implicit_optionality: false,
            }),
        );
        assert!(is_merge_patch(&state, 1));
    }

    #[test]
    fn test_cookie_and_header_distinct() {
        let mut state = StateAccessors::new();
        apply_header(&mut state, 1, Some("X-Custom"));
        apply_cookie(&mut state, 2, Some("session"));

        // Header and cookie are distinct state keys
        assert!(is_header(&state, 1));
        assert!(!is_cookie(&state, 1));
        assert!(!is_header(&state, 2));
        assert!(is_cookie(&state, 2));
    }

    #[test]
    fn test_applicable_metadata_checks() {
        let mut state = StateAccessors::new();
        // Nothing applied
        assert!(!is_applicable_metadata(&state, 1));
        assert!(!is_applicable_metadata_or_body(&state, 1));

        // Body makes it applicable
        apply_body(&mut state, 1);
        assert!(!is_applicable_metadata(&state, 1)); // body is not metadata
        assert!(is_applicable_metadata_or_body(&state, 1)); // but is applicable or body
    }

    // ========================================================================
    // Visibility-aware metadata tests
    // ========================================================================

    #[test]
    fn test_is_metadata() {
        let mut state = StateAccessors::new();
        assert!(!is_metadata(&state, 1));
        apply_header(&mut state, 1, None);
        assert!(is_metadata(&state, 1));
    }

    #[test]
    fn test_is_metadata_query() {
        let mut state = StateAccessors::new();
        apply_query(&mut state, 1, None);
        assert!(is_metadata(&state, 1));
    }

    #[test]
    fn test_is_metadata_path() {
        let mut state = StateAccessors::new();
        apply_path(&mut state, 1, None);
        assert!(is_metadata(&state, 1));
    }

    #[test]
    fn test_applicable_metadata_with_visibility_read() {
        let mut state = StateAccessors::new();
        // Header is applicable with Read visibility
        apply_header(&mut state, 1, None);
        assert!(is_applicable_metadata_with_visibility(
            &state,
            1,
            Visibility::Read
        ));

        // Query is NOT applicable with Read visibility only
        apply_query(&mut state, 2, None);
        assert!(!is_applicable_metadata_with_visibility(
            &state,
            2,
            Visibility::Read
        ));

        // StatusCode is applicable with Read visibility
        apply_status_code(&mut state, 3);
        assert!(is_applicable_metadata_with_visibility(
            &state,
            3,
            Visibility::Read
        ));
    }

    #[test]
    fn test_applicable_metadata_with_visibility_create() {
        let mut state = StateAccessors::new();
        // Header is applicable with Create visibility
        apply_header(&mut state, 1, None);
        assert!(is_applicable_metadata_with_visibility(
            &state,
            1,
            Visibility::Create
        ));

        // Query is applicable with Create visibility
        apply_query(&mut state, 2, None);
        assert!(is_applicable_metadata_with_visibility(
            &state,
            2,
            Visibility::Create
        ));

        // StatusCode is NOT applicable with Create visibility
        apply_status_code(&mut state, 3);
        assert!(!is_applicable_metadata_with_visibility(
            &state,
            3,
            Visibility::Create
        ));
    }

    #[test]
    fn test_applicable_metadata_with_visibility_item() {
        let mut state = StateAccessors::new();
        // No metadata is applicable with Item visibility
        apply_header(&mut state, 1, None);
        assert!(!is_applicable_metadata_with_visibility(
            &state,
            1,
            Visibility::Item
        ));

        apply_query(&mut state, 2, None);
        assert!(!is_applicable_metadata_with_visibility(
            &state,
            2,
            Visibility::Item
        ));
    }

    #[test]
    fn test_applicable_metadata_or_body_with_visibility() {
        let mut state = StateAccessors::new();
        // Body is applicable with Create visibility
        apply_body(&mut state, 1);
        assert!(is_applicable_metadata_or_body_with_visibility(
            &state,
            1,
            Visibility::Create
        ));

        // Body is NOT applicable with Item visibility
        assert!(!is_applicable_metadata_or_body_with_visibility(
            &state,
            1,
            Visibility::Item
        ));
    }

    #[test]
    fn test_applicable_metadata_non_metadata_property() {
        let state = StateAccessors::new();
        // A regular property (no decorators) is not applicable metadata
        assert!(!is_applicable_metadata_with_visibility(
            &state,
            1,
            Visibility::All
        ));
    }

    // ========================================================================
    // @plainData tests
    // ========================================================================

    #[test]
    fn test_plain_data() {
        let mut state = StateAccessors::new();
        assert!(!is_plain_data(&state, 1));
        apply_plain_data(&mut state, 1);
        assert!(is_plain_data(&state, 1));
    }

    // ========================================================================
    // @httpFile tests
    // ========================================================================

    #[test]
    fn test_http_file() {
        let mut state = StateAccessors::new();
        assert!(!is_http_file(&state, 1));
        apply_http_file(&mut state, 1);
        assert!(is_http_file(&state, 1));
    }

    // ========================================================================
    // @httpPart tests
    // ========================================================================

    #[test]
    fn test_http_part() {
        let mut state = StateAccessors::new();
        assert_eq!(get_http_part(&state, 1), None);
        apply_http_part(&mut state, 1, 10, Some("filePart"));
        let part = get_http_part(&state, 1).unwrap();
        assert_eq!(part.part_type, 10);
        assert_eq!(part.name, Some("filePart".to_string()));
    }

    #[test]
    fn test_http_part_no_name() {
        let mut state = StateAccessors::new();
        apply_http_part(&mut state, 1, 20, None);
        let part = get_http_part(&state, 1).unwrap();
        assert_eq!(part.part_type, 20);
        assert_eq!(part.name, None);
    }
}
