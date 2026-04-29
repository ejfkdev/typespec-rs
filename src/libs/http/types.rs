//! HTTP type definitions
//!
//! Ported from TypeSpec @typespec/http types

use crate::diagnostics::{DiagnosticDefinition, DiagnosticMap};
use std::collections::HashMap;

// ============================================================================
// Namespace and state keys
// ============================================================================

/// Namespace for HTTP types
pub const HTTP_NAMESPACE: &str = "TypeSpec.Http";

/// State key for @header decorator
pub const STATE_HEADER: &str = "TypeSpec.Http.header";
/// State key for @cookie decorator
pub const STATE_COOKIE: &str = "TypeSpec.Http.cookie";
/// State key for @query decorator
pub const STATE_QUERY: &str = "TypeSpec.Http.query";
/// State key for @path decorator
pub const STATE_PATH: &str = "TypeSpec.Http.path";
/// State key for @body decorator
pub const STATE_BODY: &str = "TypeSpec.Http.body";
/// State key for @bodyRoot decorator
pub const STATE_BODY_ROOT: &str = "TypeSpec.Http.bodyRoot";
/// State key for @bodyIgnore decorator
pub const STATE_BODY_IGNORE: &str = "TypeSpec.Http.bodyIgnore";
/// State key for @multipartBody decorator
pub const STATE_MULTIPART_BODY: &str = "TypeSpec.Http.multipartBody";
/// State key for @statusCode decorator
pub const STATE_STATUS_CODE: &str = "TypeSpec.Http.statusCode";
/// State key for HTTP verb decorators
pub const STATE_VERBS: &str = "TypeSpec.Http.verbs";
/// State key for @patch decorator options
pub const STATE_PATCH_OPTIONS: &str = "TypeSpec.Http.patchOptions";
/// State key for @server decorator
pub const STATE_SERVERS: &str = "TypeSpec.Http.servers";
/// State key for @route decorator
pub const STATE_ROUTE: &str = "TypeSpec.Http.route";
/// State key for @sharedRoute decorator
pub const STATE_SHARED_ROUTES: &str = "TypeSpec.Http.sharedRoutes";
/// State key for @includeInapplicableMetadataInPayload decorator
pub const STATE_INCLUDE_INAPPLICABLE_METADATA: &str =
    "TypeSpec.Http.includeInapplicableMetadataInPayload";
/// State key for @mergePatchModel decorator
pub const STATE_MERGE_PATCH_MODEL: &str = "TypeSpec.Http.mergePatchModel";
/// State key for @mergePatchProperty decorator
pub const STATE_MERGE_PATCH_PROPERTY: &str = "TypeSpec.Http.mergePatchProperty";
/// State key for @applyMergePatch decorator
pub const STATE_APPLY_MERGE_PATCH: &str = "TypeSpec.Http.applyMergePatch";
/// State key for HTTP metadata (marks properties as metadata)
pub const STATE_HTTP_METADATA: &str = "TypeSpec.Http.httpMetadata";
/// State key for external interfaces (route resolution)
pub const STATE_EXTERNAL_INTERFACES: &str = "TypeSpec.Http.externalInterfaces";
/// State key for route producer
pub const STATE_ROUTE_PRODUCER: &str = "TypeSpec.Http.routeProducer";
/// State key for resolved routes
pub const STATE_ROUTES: &str = "TypeSpec.Http.routes";
/// State key for route options
pub const STATE_ROUTE_OPTIONS: &str = "TypeSpec.Http.routeOptions";
/// State key for @Private.file decorator
pub const STATE_FILE: &str = "TypeSpec.Http.file";
/// State key for @Private.httpPart decorator
pub const STATE_HTTP_PART: &str = "TypeSpec.Http.httpPart";
/// State key for merge patch property options
pub const STATE_MERGE_PATCH_PROPERTY_OPTIONS: &str = "TypeSpec.Http.mergePatchPropertyOptions";

// ============================================================================
// Diagnostic codes
// ============================================================================

/// Diagnostic: HTTP verb already applied
pub const DIAG_HTTP_VERB_DUPLICATE: &str = "http-verb-duplicate";
/// Diagnostic: Missing URI parameter
pub const DIAG_MISSING_URI_PARAM: &str = "missing-uri-param";
/// Diagnostic: Incompatible URI parameter type
pub const DIAG_INCOMPATIBLE_URI_PARAM: &str = "incompatible-uri-param";
/// Diagnostic: Double slash in route
pub const DIAG_DOUBLE_SLASH: &str = "double-slash";
/// Diagnostic: Missing server parameter
pub const DIAG_MISSING_SERVER_PARAM: &str = "missing-server-param";
/// Diagnostic: Duplicate body parameter
pub const DIAG_DUPLICATE_BODY: &str = "duplicate-body";
/// Diagnostic: Duplicate route decorator
pub const DIAG_DUPLICATE_ROUTE_DECORATOR: &str = "duplicate-route-decorator";
/// Diagnostic: Duplicate operation
pub const DIAG_DUPLICATE_OPERATION: &str = "duplicate-operation";
/// Diagnostic: Multiple status codes
pub const DIAG_MULTIPLE_STATUS_CODES: &str = "multiple-status-codes";
/// Diagnostic: Invalid status code
pub const DIAG_STATUS_CODE_INVALID: &str = "status-code-invalid";
/// Diagnostic: Content type must be string
pub const DIAG_CONTENT_TYPE_STRING: &str = "content-type-string";
/// Diagnostic: Content type ignored
pub const DIAG_CONTENT_TYPE_IGNORED: &str = "content-type-ignored";
/// Diagnostic: Metadata ignored in body
pub const DIAG_METADATA_IGNORED: &str = "metadata-ignored";
/// Diagnostic: No service found
pub const DIAG_NO_SERVICE_FOUND: &str = "no-service-found";
/// Diagnostic: Invalid auth type
pub const DIAG_INVALID_TYPE_FOR_AUTH: &str = "invalid-type-for-auth";
/// Diagnostic: Shared route inconsistency
pub const DIAG_SHARED_INCONSISTENCY: &str = "shared-inconsistency";
/// Diagnostic: Use URI template instead
pub const DIAG_USE_URI_TEMPLATE: &str = "use-uri-template";
/// Diagnostic: FormData missing part name
pub const DIAG_FORMDATA_NO_PART_NAME: &str = "formdata-no-part-name";
/// Diagnostic: HTTP file content type not string
pub const DIAG_HTTP_FILE_CONTENT_TYPE_NOT_STRING: &str = "http-file-content-type-not-string";
/// Diagnostic: HTTP file contents not scalar
pub const DIAG_HTTP_FILE_CONTENTS_NOT_SCALAR: &str = "http-file-contents-not-scalar";
/// Diagnostic: HTTP file disallowed metadata
pub const DIAG_HTTP_FILE_DISALLOWED_METADATA: &str = "http-file-disallowed-metadata";
/// Diagnostic: HTTP file extra property
pub const DIAG_HTTP_FILE_EXTRA_PROPERTY: &str = "http-file-extra-property";
/// Diagnostic: HTTP file structured content
pub const DIAG_HTTP_FILE_STRUCTURED: &str = "http-file-structured";
/// Diagnostic: Merge patch contains metadata
pub const DIAG_MERGE_PATCH_CONTAINS_METADATA: &str = "merge-patch-contains-metadata";
/// Diagnostic: Merge patch contains null
pub const DIAG_MERGE_PATCH_CONTAINS_NULL: &str = "merge-patch-contains-null";
/// Diagnostic: Merge patch content type issue
pub const DIAG_MERGE_PATCH_CONTENT_TYPE: &str = "merge-patch-content-type";
/// Diagnostic: Multipart invalid content type
pub const DIAG_MULTIPART_INVALID_CONTENT_TYPE: &str = "multipart-invalid-content-type";
/// Diagnostic: Multipart model issue
pub const DIAG_MULTIPART_MODEL: &str = "multipart-model";
/// Diagnostic: Nested multipart
pub const DIAG_MULTIPART_NESTED: &str = "multipart-nested";
/// Diagnostic: Multipart part issue
pub const DIAG_MULTIPART_PART: &str = "multipart-part";
/// Diagnostic: No implicit multipart
pub const DIAG_NO_IMPLICIT_MULTIPART: &str = "no-implicit-multipart";
/// Diagnostic: Operation param duplicate type
pub const DIAG_OPERATION_PARAM_DUPLICATE_TYPE: &str = "operation-param-duplicate-type";
/// Diagnostic: Response cookie not supported
pub const DIAG_RESPONSE_COOKIE_NOT_SUPPORTED: &str = "response-cookie-not-supported";

// ============================================================================
// HttpVerb enum
// ============================================================================

string_enum! {
    /// HTTP method verbs.
    /// Ported from TS HttpVerb type.
    pub enum HttpVerb {
        Get => "get",
        Put => "put",
        Post => "post",
        Patch => "patch",
        Delete => "delete",
        Head => "head",
    }
}

impl HttpVerb {
    /// Get all HTTP verbs
    pub fn all() -> &'static [HttpVerb] {
        &[
            HttpVerb::Get,
            HttpVerb::Put,
            HttpVerb::Post,
            HttpVerb::Patch,
            HttpVerb::Delete,
            HttpVerb::Head,
        ]
    }
}

// ============================================================================
// HTTP parameter options
// ============================================================================

/// Options for @header decorator.
/// Ported from TS HeaderOptions model.
#[derive(Debug, Clone)]
pub struct HeaderOptions {
    /// Name of the header when sent over HTTP
    pub name: Option<String>,
    /// Whether to explode array/object values
    pub explode: Option<bool>,
}

/// Options for @query decorator.
/// Ported from TS QueryOptions model.
#[derive(Debug, Clone)]
pub struct QueryOptions {
    /// Name of the query parameter
    pub name: Option<String>,
    /// Whether to explode array/object values
    pub explode: Option<bool>,
}

/// Options for @path decorator.
/// Ported from TS PathOptions model.
#[derive(Debug, Clone)]
pub struct PathOptions {
    /// Name of the path parameter
    pub name: Option<String>,
    /// Whether to explode values
    pub explode: Option<bool>,
    /// Interpolation style for the path parameter
    pub style: Option<PathStyle>,
    /// Whether to allow reserved characters
    pub allow_reserved: Option<bool>,
}

string_enum! {
    /// Path parameter interpolation style.
    pub enum PathStyle {
        Simple => "simple",
        Label => "label",
        Matrix => "matrix",
        Fragment => "fragment",
        PathSegment => "path",
    }
}

/// Options for @cookie decorator.
/// Ported from TS CookieOptions model.
#[derive(Debug, Clone)]
pub struct CookieOptions {
    /// Name of the cookie
    pub name: Option<String>,
}

/// Options for @patch decorator.
/// Ported from TS PatchOptions model.
#[derive(Debug, Clone)]
pub struct PatchOptions {
    /// If false, disables the implicit transform that makes the body deeply optional
    pub implicit_optionality: Option<bool>,
}

// ============================================================================
// Merge-patch types
// Ported from http/src/merge-patch.ts
// ============================================================================

/// Merge-patch options for @applyMergePatch decorator.
/// Ported from TS ApplyMergePatchOptions model.
#[derive(Debug, Clone)]
pub struct ApplyMergePatchOptions {
    /// Whether to make non-optional properties optional
    pub implicit_optionality: bool,
}

impl Default for ApplyMergePatchOptions {
    fn default() -> Self {
        Self {
            implicit_optionality: true,
        }
    }
}

// ============================================================================
// Library creation
// ============================================================================

/// Create the @typespec/http library diagnostic map.
/// Ported from TS $lib definition in lib.ts.
pub fn create_http_library() -> DiagnosticMap {
    HashMap::from([
        (
            DIAG_HTTP_VERB_DUPLICATE.to_string(),
            DiagnosticDefinition::error("HTTP verb already applied to entity"),
        ),
        (
            DIAG_MISSING_URI_PARAM.to_string(),
            DiagnosticDefinition::error(
                "Route reference parameter but wasn't found in operation parameters",
            ),
        ),
        (
            DIAG_INCOMPATIBLE_URI_PARAM.to_string(),
            DiagnosticDefinition::error(
                "Parameter is defined in the uri but is annotated as a different kind",
            ),
        ),
        (
            DIAG_DOUBLE_SLASH.to_string(),
            DiagnosticDefinition::warning("Route will result in duplicate slashes"),
        ),
        (
            DIAG_MISSING_SERVER_PARAM.to_string(),
            DiagnosticDefinition::error(
                "Server url contains parameter but wasn't found in given parameters",
            ),
        ),
        (
            DIAG_DUPLICATE_BODY.to_string(),
            DiagnosticDefinition::error("Operation has multiple @body parameters declared"),
        ),
        (
            DIAG_DUPLICATE_ROUTE_DECORATOR.to_string(),
            DiagnosticDefinition::error(
                "@route was defined twice on this namespace and has different values",
            ),
        ),
        (
            DIAG_DUPLICATE_OPERATION.to_string(),
            DiagnosticDefinition::error("Duplicate operation routed at the same verb and path"),
        ),
        (
            DIAG_MULTIPLE_STATUS_CODES.to_string(),
            DiagnosticDefinition::error(
                "Multiple @statusCode decorators defined for this operation response",
            ),
        ),
        (
            DIAG_STATUS_CODE_INVALID.to_string(),
            DiagnosticDefinition::error(
                "statusCode value must be a three digit code between 100 and 599",
            ),
        ),
        (
            DIAG_CONTENT_TYPE_STRING.to_string(),
            DiagnosticDefinition::error(
                "contentType parameter must be a string literal or union of string literals",
            ),
        ),
        (
            DIAG_CONTENT_TYPE_IGNORED.to_string(),
            DiagnosticDefinition::warning("Content-Type header ignored because there is no body"),
        ),
        (
            DIAG_METADATA_IGNORED.to_string(),
            DiagnosticDefinition::warning(
                "Property will be ignored as it is inside of a @body property",
            ),
        ),
        (
            DIAG_NO_SERVICE_FOUND.to_string(),
            DiagnosticDefinition::warning(
                "No namespace with @service was found, but namespace contains routes",
            ),
        ),
        (
            DIAG_INVALID_TYPE_FOR_AUTH.to_string(),
            DiagnosticDefinition::error(
                "@useAuth only accepts Auth model, Tuple of auth model, or union of auth model",
            ),
        ),
        (
            DIAG_SHARED_INCONSISTENCY.to_string(),
            DiagnosticDefinition::error(
                "Each operation routed at the same verb and path needs to have the @sharedRoute decorator",
            ),
        ),
        (
            DIAG_USE_URI_TEMPLATE.to_string(),
            DiagnosticDefinition::error("Parameter is already defined in the uri template"),
        ),
        (
            DIAG_FORMDATA_NO_PART_NAME.to_string(),
            DiagnosticDefinition::error("FormData part must have a @header name"),
        ),
        (
            DIAG_HTTP_FILE_CONTENT_TYPE_NOT_STRING.to_string(),
            DiagnosticDefinition::error("File content type must be a string"),
        ),
        (
            DIAG_HTTP_FILE_CONTENTS_NOT_SCALAR.to_string(),
            DiagnosticDefinition::error("File contents must be a scalar type"),
        ),
        (
            DIAG_HTTP_FILE_DISALLOWED_METADATA.to_string(),
            DiagnosticDefinition::error(
                "File type cannot have metadata properties like @header, @query, or @path",
            ),
        ),
        (
            DIAG_HTTP_FILE_EXTRA_PROPERTY.to_string(),
            DiagnosticDefinition::error(
                "File type can only have contentType and contents properties",
            ),
        ),
        (
            DIAG_HTTP_FILE_STRUCTURED.to_string(),
            DiagnosticDefinition::error(
                "File type must have a string contents property, not a structured model",
            ),
        ),
        (
            DIAG_MERGE_PATCH_CONTAINS_METADATA.to_string(),
            DiagnosticDefinition::error("Merge patch model cannot contain metadata properties"),
        ),
        (
            DIAG_MERGE_PATCH_CONTAINS_NULL.to_string(),
            DiagnosticDefinition::error("Merge patch model cannot contain null types"),
        ),
        (
            DIAG_MERGE_PATCH_CONTENT_TYPE.to_string(),
            DiagnosticDefinition::error("Merge patch model cannot have a content type property"),
        ),
        (
            DIAG_MULTIPART_INVALID_CONTENT_TYPE.to_string(),
            DiagnosticDefinition::error("Multipart content type must be 'multipart/form-data'"),
        ),
        (
            DIAG_MULTIPART_MODEL.to_string(),
            DiagnosticDefinition::error("Multipart body must be a model type"),
        ),
        (
            DIAG_MULTIPART_NESTED.to_string(),
            DiagnosticDefinition::error("Nested multipart bodies are not supported"),
        ),
        (
            DIAG_MULTIPART_PART.to_string(),
            DiagnosticDefinition::error("Multipart part must be a model or scalar type"),
        ),
        (
            DIAG_NO_IMPLICIT_MULTIPART.to_string(),
            DiagnosticDefinition::error(
                "Multipart body must be explicitly marked with @multipartBody",
            ),
        ),
        (
            DIAG_OPERATION_PARAM_DUPLICATE_TYPE.to_string(),
            DiagnosticDefinition::error("Operation parameter has duplicate type annotation"),
        ),
        (
            DIAG_RESPONSE_COOKIE_NOT_SUPPORTED.to_string(),
            DiagnosticDefinition::warning("Cookies in responses are not supported in HTTP"),
        ),
    ])
}
