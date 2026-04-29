//! @typespec/openapi - OpenAPI Decorators and Types
//!
//! Ported from TypeSpec packages/openapi
//!
//! Provides decorators for OpenAPI-specific annotations:
//! - `@operationId` - Set a specific operation ID
//! - `@extension` - Add OpenAPI extension (x-* properties)
//! - `@defaultResponse` - Mark a model as default response
//! - `@externalDocs` - Add external documentation link
//! - `@info` - Set OpenAPI info object
//! - `@tagMetadata` - Add metadata to a tag
//!
//! ## Types
//! - `ExtensionKey` - Type for OpenAPI extension keys (must start with "x-")
//! - `AdditionalInfo` - OpenAPI info object
//! - `ExternalDocs` - External documentation
//! - `Contact`, `License` - Info sub-types

use crate::checker::types::TypeId;
use crate::diagnostics::{DiagnosticDefinition, DiagnosticMap};
use crate::state_accessors::StateAccessors;
use std::collections::HashMap;

// ============================================================================
// Namespace and state keys
// ============================================================================

/// Namespace for OpenAPI types
pub const OPENAPI_NAMESPACE: &str = "TypeSpec.OpenAPI";

/// State key for @operationId decorator
pub const STATE_OPERATION_ID: &str = "TypeSpec.OpenAPI.operationId";
/// State key for OpenAPI extensions
pub const STATE_EXTENSION: &str = "TypeSpec.OpenAPI.extension";
/// State key for @defaultResponse decorator
pub const STATE_DEFAULT_RESPONSE: &str = "TypeSpec.OpenAPI.defaultResponse";
/// State key for @externalDocs decorator
pub const STATE_EXTERNAL_DOCS: &str = "TypeSpec.OpenAPI.externalDocs";
/// State key for @info decorator
pub const STATE_INFO: &str = "TypeSpec.OpenAPI.info";
/// State key for @tagMetadata decorator
pub const STATE_TAG_METADATA: &str = "TypeSpec.OpenAPI.tagMetadata";

// ============================================================================
// Types
// ============================================================================

/// Contact information for the API.
/// Ported from TS Contact interface.
#[derive(Debug, Clone)]
pub struct Contact {
    /// The identifying name of the contact person/organization.
    pub name: Option<String>,
    /// The URL pointing to the contact information.
    pub url: Option<String>,
    /// The email address of the contact person/organization.
    pub email: Option<String>,
}

/// License information for the API.
/// Ported from TS License interface.
#[derive(Debug, Clone)]
pub struct License {
    /// The license name used for the API.
    pub name: String,
    /// A URL to the license used for the API.
    pub url: Option<String>,
}

/// OpenAPI additional information.
/// Ported from TS AdditionalInfo interface.
#[derive(Debug, Clone, Default)]
pub struct AdditionalInfo {
    /// The title of the API. Overrides the @service title.
    pub title: Option<String>,
    /// A short summary of the API. Overrides the @summary.
    pub summary: Option<String>,
    /// A description of the API. Overrides the @doc.
    pub description: Option<String>,
    /// The version of the OpenAPI document.
    pub version: Option<String>,
    /// A URL to the Terms of Service for the API.
    pub terms_of_service: Option<String>,
    /// The contact information for the exposed API.
    pub contact: Option<Contact>,
    /// The license information for the exposed API.
    pub license: Option<License>,
}

/// External documentation.
/// Ported from TS ExternalDocs interface.
#[derive(Debug, Clone)]
pub struct ExternalDocs {
    /// Documentation URL
    pub url: String,
    /// Optional description
    pub description: Option<String>,
}

/// Tag metadata for OpenAPI tags.
/// Ported from TS TagMetadata type.
#[derive(Debug, Clone)]
pub struct TagMetadata {
    /// Description of the tag
    pub description: Option<String>,
    /// External documentation for the tag
    pub external_docs: Option<ExternalDocs>,
    /// The name of a tag that this tag is nested under.
    /// Only supported in OpenAPI 3.2. For 3.0 and 3.1, this will be converted to x-parent.
    pub parent: Option<String>,
}

// ============================================================================
// Diagnostic definitions
// ============================================================================

/// Create the OpenAPI library diagnostic map.
pub fn create_openapi_library() -> DiagnosticMap {
    HashMap::from([
        (
            "invalid-extension-key".to_string(),
            DiagnosticDefinition::error("OpenAPI extension must start with 'x-' but was '{value}'"),
        ),
        (
            "duplicate-type-name".to_string(),
            DiagnosticDefinition::error_with_messages(vec![
                (
                    "default",
                    "Duplicate type name: '{value}'. Check @friendlyName decorators and overlap with types in TypeSpec or service namespace.",
                ),
                (
                    "parameter",
                    "Duplicate parameter key: '{value}'. Check @friendlyName decorators and overlap with types in TypeSpec or service namespace.",
                ),
            ]),
        ),
        (
            "not-url".to_string(),
            DiagnosticDefinition::error("{property}: {value} is not a valid URL."),
        ),
        (
            "duplicate-tag".to_string(),
            DiagnosticDefinition::error("Metadata for tag '{tagName}' was specified twice."),
        ),
        (
            "tag-metadata-target-service".to_string(),
            DiagnosticDefinition::error(
                "@tagMetadata must be used on the service namespace. Did you mean to annotate '{namespace}' with '@service'?",
            ),
        ),
    ])
}

// ============================================================================
// Utility functions
// ============================================================================

/// Check if a string is a valid OpenAPI extension key (must start with "x-").
pub fn is_valid_extension_key(key: &str) -> bool {
    key.starts_with("x-")
}

/// Validate a URL string (basic check).
pub fn is_valid_url(url: &str) -> bool {
    url.starts_with("http://") || url.starts_with("https://") || url.starts_with("urn:")
}

// ============================================================================
// Decorator implementations (state-based)
// ============================================================================

string_decorator!(apply_operation_id, get_operation_id, STATE_OPERATION_ID);

/// Apply @extension decorator.
/// Stores as "x-keyname::value" format, supporting multiple extensions.
/// Ported from TS $extension().
pub fn apply_extension(state: &mut StateAccessors, target: TypeId, key: &str, value: &str) {
    let existing = state
        .get_state(STATE_EXTENSION, target)
        .unwrap_or("")
        .to_string();
    let entry = format!("{}::{}", key, value);
    let new_value = if existing.is_empty() {
        entry
    } else {
        format!("{}||{}", existing, entry)
    };
    state.set_state(STATE_EXTENSION, target, new_value);
}

/// Get all extensions for a type.
/// Ported from TS getExtensions().
pub fn get_extensions(state: &StateAccessors, target: TypeId) -> Vec<(String, String)> {
    state
        .get_state(STATE_EXTENSION, target)
        .map(|s| {
            s.split("||")
                .filter_map(|entry| {
                    let parts: Vec<&str> = entry.splitn(2, "::").collect();
                    if parts.len() == 2 {
                        Some((parts[0].to_string(), parts[1].to_string()))
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Get a specific extension value by key.
pub fn get_extension(state: &StateAccessors, target: TypeId, key: &str) -> Option<String> {
    get_extensions(state, target)
        .into_iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v)
}

flag_decorator!(
    apply_default_response,
    is_default_response,
    STATE_DEFAULT_RESPONSE
);

/// Apply @externalDocs decorator.
/// Ported from TS $externalDocs().
pub fn apply_external_docs(
    state: &mut StateAccessors,
    target: TypeId,
    url: &str,
    description: Option<&str>,
) {
    let value = match description {
        Some(desc) => format!("{}|{}", url, desc),
        None => url.to_string(),
    };
    state.set_state(STATE_EXTERNAL_DOCS, target, value);
}

/// Get external docs for a type.
/// Ported from TS getExternalDocs().
pub fn get_external_docs(state: &StateAccessors, target: TypeId) -> Option<ExternalDocs> {
    state.get_state(STATE_EXTERNAL_DOCS, target).map(|s| {
        let parts: Vec<&str> = s.splitn(2, '|').collect();
        ExternalDocs {
            url: parts[0].to_string(),
            description: parts.get(1).map(|d| d.to_string()),
        }
    })
}

// ============================================================================
// @info decorator
// Ported from TS $info() / getInfo() / setInfo()
// ============================================================================

/// Apply @info decorator - set OpenAPI info object for a namespace.
/// Ported from TS $info() / setInfo().
/// The info data is serialized as a simple format for state storage.
pub fn apply_info(state: &mut StateAccessors, target: TypeId, info: &AdditionalInfo) {
    // Serialize as key=value pairs separated by "||"
    let mut parts: Vec<String> = Vec::new();
    if let Some(ref title) = info.title {
        parts.push(format!("title={}", title));
    }
    if let Some(ref summary) = info.summary {
        parts.push(format!("summary={}", summary));
    }
    if let Some(ref description) = info.description {
        parts.push(format!("description={}", description));
    }
    if let Some(ref version) = info.version {
        parts.push(format!("version={}", version));
    }
    if let Some(ref tos) = info.terms_of_service {
        parts.push(format!("termsOfService={}", tos));
    }
    state.set_state(STATE_INFO, target, parts.join("||"));
}

/// Get the OpenAPI info object for a namespace.
/// Ported from TS getInfo().
pub fn get_info(state: &StateAccessors, target: TypeId) -> Option<AdditionalInfo> {
    state.get_state(STATE_INFO, target).map(|s| {
        let mut info = AdditionalInfo {
            title: None,
            summary: None,
            description: None,
            version: None,
            terms_of_service: None,
            contact: None,
            license: None,
        };
        for part in s.split("||") {
            if let Some(value) = part.strip_prefix("title=") {
                info.title = Some(value.to_string());
            } else if let Some(value) = part.strip_prefix("summary=") {
                info.summary = Some(value.to_string());
            } else if let Some(value) = part.strip_prefix("description=") {
                info.description = Some(value.to_string());
            } else if let Some(value) = part.strip_prefix("version=") {
                info.version = Some(value.to_string());
            } else if let Some(value) = part.strip_prefix("termsOfService=") {
                info.terms_of_service = Some(value.to_string());
            }
        }
        info
    })
}

// ============================================================================
// @tagMetadata decorator
// Ported from TS tagMetadataDecorator / getTagsMetadata
// ============================================================================

/// Apply @tagMetadata decorator - add metadata to a tag for a namespace.
/// Ported from TS tagMetadataDecorator().
/// Tags are stored as "tagName<>description<>url<>parent" format with "<>" as sub-separator,
/// and "||" as entry separator, avoiding conflict with `|` in URLs.
pub fn apply_tag_metadata(
    state: &mut StateAccessors,
    target: TypeId,
    tag_name: &str,
    metadata: &TagMetadata,
) {
    let existing = state
        .get_state(STATE_TAG_METADATA, target)
        .unwrap_or("")
        .to_string();
    let desc = metadata.description.as_deref().unwrap_or("");
    let url = metadata
        .external_docs
        .as_ref()
        .map(|d| d.url.as_str())
        .unwrap_or("");
    let parent = metadata.parent.as_deref().unwrap_or("");
    let entry = format!("{}<>{}<>{}<>{}", tag_name, desc, url, parent);
    let new_value = if existing.is_empty() {
        entry
    } else {
        format!("{}||{}", existing, entry)
    };
    state.set_state(STATE_TAG_METADATA, target, new_value);
}

/// Get all tag metadata for a namespace.
/// Ported from TS getTagsMetadata().
pub fn get_tag_metadata(state: &StateAccessors, target: TypeId) -> Vec<(String, TagMetadata)> {
    state
        .get_state(STATE_TAG_METADATA, target)
        .map(|s| {
            s.split("||")
                .filter_map(|entry| {
                    let parts: Vec<&str> = entry.splitn(4, "<>").collect();
                    if parts.is_empty() {
                        return None;
                    }
                    let name = parts[0].to_string();
                    if name.is_empty() {
                        return None;
                    }
                    let description = parts
                        .get(1)
                        .filter(|d| !d.is_empty())
                        .map(|d| d.to_string());
                    let external_docs =
                        parts
                            .get(2)
                            .filter(|u| !u.is_empty())
                            .map(|u| ExternalDocs {
                                url: u.to_string(),
                                description: None,
                            });
                    let parent = parts
                        .get(3)
                        .filter(|p| !p.is_empty())
                        .map(|p| p.to_string());
                    Some((
                        name,
                        TagMetadata {
                            description,
                            external_docs,
                            parent,
                        },
                    ))
                })
                .collect()
        })
        .unwrap_or_default()
}

// ============================================================================
// TSP Sources
// ============================================================================

/// The TypeSpec source for the OpenAPI library
pub const OPENAPI_DECORATORS_TSP: &str = r#"
using TypeSpec.Reflection;

namespace TypeSpec.OpenAPI;

/**
 * Specify the OpenAPI `operationId` property for this operation.
 *
 * @param operationId Operation id value.
 */
extern dec operationId(target: Operation, operationId: valueof string);

/**
 * Attach some custom data to the OpenAPI element generated from this type.
 *
 * @param key Extension key.
 * @param value Extension value.
 */
extern dec extension(target: unknown, key: valueof string, value: valueof unknown);

/**
 * Specify that this model is to be treated as the OpenAPI `default` response.
 * This differs from the compiler built-in `@error` decorator as this does not necessarily represent an error.
 */
extern dec defaultResponse(target: Model);

/**
 * Specify the OpenAPI `externalDocs` property for this type.
 *
 * @param url Url to the docs
 * @param description Description of the docs
 */
extern dec externalDocs(target: unknown, url: valueof string, description?: valueof string);

/** Additional information for the OpenAPI document. */
model AdditionalInfo {
  /** The title of the API. Overrides the `@service` title. */
  title?: string;

  /** A short summary of the API. Overrides the `@summary` provided on the service namespace. */
  summary?: string;

  /** A description of the API. Overrides the `@doc` provided on the service namespace. */
  description?: string;

  /** The version of the OpenAPI document. */
  version?: string;

  /** A URL to the Terms of Service for the API. */
  termsOfService?: url;

  /** The contact information for the exposed API. */
  contact?: Contact;

  /** The license information for the exposed API. */
  license?: License;

  ...Record<unknown>;
}

/** Contact information for the exposed API. */
model Contact {
  /** The identifying name of the contact person/organization. */
  name?: string;

  /** The URL pointing to the contact information. */
  url?: url;

  /** The email address of the contact person/organization. */
  email?: string;

  ...Record<unknown>;
}

/** License information for the exposed API. */
model License {
  /** The license name used for the API. */
  name: string;

  /** A URL to the license used for the API. */
  url?: url;

  ...Record<unknown>;
}

/**
 * Specify OpenAPI additional information.
 * The service `title` is already specified using `@service`.
 * @param additionalInfo Additional information
 */
extern dec info(target: Namespace, additionalInfo: valueof AdditionalInfo);

/** Metadata to a single tag that is used by operations. */
model TagMetadata {
  /** A description of the tag. */
  description?: string;

  /** An external Docs information of the tag. */
  externalDocs?: ExternalDocs;

  /** The name of a tag that this tag is nested under. Only supported in OpenAPI 3.2. */
  parent?: string;

  ...Record<unknown>;
}

/** External Docs information. */
model ExternalDocs {
  /** Documentation url */
  url: string;

  /** Optional description */
  description?: string;

  ...Record<unknown>;
}

/**
 * Specify OpenAPI tag metadata.
 * @param name tag name
 * @param tagMetadata Additional information
 */
extern dec tagMetadata(target: Namespace, name: valueof string, tagMetadata: valueof TagMetadata);
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_namespace() {
        assert_eq!(OPENAPI_NAMESPACE, "TypeSpec.OpenAPI");
    }

    #[test]
    fn test_create_openapi_library() {
        let diags = create_openapi_library();
        assert!(diags.len() >= 5);
    }

    #[test]
    fn test_is_valid_extension_key() {
        assert!(is_valid_extension_key("x-custom"));
        assert!(is_valid_extension_key("x-ms-azure"));
        assert!(!is_valid_extension_key("custom"));
        assert!(!is_valid_extension_key("X-custom"));
    }

    #[test]
    fn test_is_valid_url() {
        assert!(is_valid_url("https://example.com"));
        assert!(is_valid_url("http://localhost:3000"));
        assert!(is_valid_url("urn:example:ns"));
        assert!(!is_valid_url("not-a-url"));
        assert!(!is_valid_url("ftp://example.com"));
    }

    #[test]
    fn test_apply_operation_id() {
        let mut state = StateAccessors::new();
        assert_eq!(get_operation_id(&state, 1), None);
        apply_operation_id(&mut state, 1, "pets_list");
        assert_eq!(get_operation_id(&state, 1), Some("pets_list".to_string()));
    }

    #[test]
    fn test_apply_extension() {
        let mut state = StateAccessors::new();
        apply_extension(&mut state, 1, "x-custom", "value1");
        apply_extension(&mut state, 1, "x-ms-azure", "value2");
        let exts = get_extensions(&state, 1);
        assert_eq!(exts.len(), 2);
        assert_eq!(
            get_extension(&state, 1, "x-custom"),
            Some("value1".to_string())
        );
        assert_eq!(
            get_extension(&state, 1, "x-ms-azure"),
            Some("value2".to_string())
        );
    }

    #[test]
    fn test_apply_default_response() {
        let mut state = StateAccessors::new();
        assert!(!is_default_response(&state, 1));
        apply_default_response(&mut state, 1);
        assert!(is_default_response(&state, 1));
    }

    #[test]
    fn test_apply_external_docs() {
        let mut state = StateAccessors::new();
        apply_external_docs(&mut state, 1, "https://docs.example.com", Some("API Docs"));
        let docs = get_external_docs(&state, 1);
        assert!(docs.is_some());
        let docs = docs.unwrap();
        assert_eq!(docs.url, "https://docs.example.com");
        assert_eq!(docs.description, Some("API Docs".to_string()));
    }

    #[test]
    fn test_apply_external_docs_no_description() {
        let mut state = StateAccessors::new();
        apply_external_docs(&mut state, 1, "https://docs.example.com", None);
        let docs = get_external_docs(&state, 1).unwrap();
        assert_eq!(docs.url, "https://docs.example.com");
        assert_eq!(docs.description, None);
    }

    #[test]
    fn test_additional_info_type() {
        let info = AdditionalInfo {
            title: Some("Pet Store API".to_string()),
            summary: Some("A sample API".to_string()),
            description: None,
            version: Some("1.0.0".to_string()),
            terms_of_service: None,
            contact: Some(Contact {
                name: Some("API Support".to_string()),
                url: Some("https://support.example.com".to_string()),
                email: Some("support@example.com".to_string()),
            }),
            license: Some(License {
                name: "MIT".to_string(),
                url: None,
            }),
        };
        assert_eq!(info.title.as_deref(), Some("Pet Store API"));
        assert!(info.contact.is_some());
        assert!(info.license.is_some());
    }

    #[test]
    fn test_decorators_tsp_not_empty() {
        assert!(!OPENAPI_DECORATORS_TSP.is_empty());
        assert!(OPENAPI_DECORATORS_TSP.contains("dec operationId"));
        assert!(OPENAPI_DECORATORS_TSP.contains("dec extension"));
        assert!(OPENAPI_DECORATORS_TSP.contains("dec defaultResponse"));
        assert!(OPENAPI_DECORATORS_TSP.contains("dec externalDocs"));
        assert!(OPENAPI_DECORATORS_TSP.contains("dec info"));
        assert!(OPENAPI_DECORATORS_TSP.contains("AdditionalInfo"));
    }

    #[test]
    fn test_apply_info() {
        let mut state = StateAccessors::new();
        assert!(get_info(&state, 1).is_none());

        apply_info(
            &mut state,
            1,
            &AdditionalInfo {
                title: Some("Pet Store API".to_string()),
                version: Some("1.0.0".to_string()),
                ..Default::default()
            },
        );

        let info = get_info(&state, 1);
        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.title.as_deref(), Some("Pet Store API"));
        assert_eq!(info.version.as_deref(), Some("1.0.0"));
        assert!(info.description.is_none());
    }

    #[test]
    fn test_apply_info_full() {
        let mut state = StateAccessors::new();
        apply_info(
            &mut state,
            1,
            &AdditionalInfo {
                title: Some("My API".to_string()),
                summary: Some("A summary".to_string()),
                description: Some("A description".to_string()),
                version: Some("2.0.0".to_string()),
                terms_of_service: Some("https://example.com/tos".to_string()),
                contact: None,
                license: None,
            },
        );

        let info = get_info(&state, 1).unwrap();
        assert_eq!(info.title.as_deref(), Some("My API"));
        assert_eq!(info.summary.as_deref(), Some("A summary"));
        assert_eq!(info.description.as_deref(), Some("A description"));
        assert_eq!(info.version.as_deref(), Some("2.0.0"));
        assert_eq!(
            info.terms_of_service.as_deref(),
            Some("https://example.com/tos")
        );
    }

    #[test]
    fn test_apply_tag_metadata() {
        let mut state = StateAccessors::new();
        assert!(get_tag_metadata(&state, 1).is_empty());

        apply_tag_metadata(
            &mut state,
            1,
            "pets",
            &TagMetadata {
                description: Some("Pet operations".to_string()),
                external_docs: Some(ExternalDocs {
                    url: "https://docs.example.com/pets".to_string(),
                    description: None,
                }),
                parent: None,
            },
        );

        let tags = get_tag_metadata(&state, 1);
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].0, "pets");
        assert_eq!(tags[0].1.description.as_deref(), Some("Pet operations"));
        assert!(tags[0].1.external_docs.is_some());
        assert_eq!(
            tags[0].1.external_docs.as_ref().unwrap().url,
            "https://docs.example.com/pets"
        );
    }

    #[test]
    fn test_apply_tag_metadata_multiple() {
        let mut state = StateAccessors::new();
        apply_tag_metadata(
            &mut state,
            1,
            "pets",
            &TagMetadata {
                description: Some("Pet ops".to_string()),
                external_docs: None,
                parent: None,
            },
        );
        apply_tag_metadata(
            &mut state,
            1,
            "users",
            &TagMetadata {
                description: None,
                external_docs: None,
                parent: Some("pets".to_string()),
            },
        );

        let tags = get_tag_metadata(&state, 1);
        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0].0, "pets");
        assert!(tags[0].1.parent.is_none());
        assert_eq!(tags[1].0, "users");
        assert_eq!(tags[1].1.parent.as_deref(), Some("pets"));
    }
}
