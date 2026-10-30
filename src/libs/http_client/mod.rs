//! @typespec/http-client — shared client library for SDK generators
//!
//! Ported from TypeSpec `packages/http-client`. This is the TypeSpec-level
//! layer the official Java, TypeScript, Python and C# client SDK generators
//! build on:
//!
//! - `@experimental` with per-emitter scoping (`emitterScope`)
//! - client discovery / naming / parent-child relationships
//! - client operation and data type collection
//! - client constructor parameters (endpoint, server parameters, credential)
//!
//! The generators themselves are language-specific code emitters and are not
//! ported; this module provides the semantic model they consume.

pub mod client_library;

use std::collections::HashMap;

use crate::checker::Checker;
use crate::checker::types::TypeId;
use crate::diagnostics::{DiagnosticDefinition, DiagnosticMap};

pub use client_library::{
    Client, ClientConstructor, ClientConstructorParameter, ClientId, ClientLibrary,
    ClientLibraryResult, ClientOperation, ClientParameterType, InternalClient,
    create_client_library, flat_clients, get_client, get_client_auth, get_client_constructor,
    get_client_name, get_parent_client, have_same_constructor, is_publicly_initializable,
    list_clients, list_clients_of_client, list_http_operations, list_namespaces, list_servers,
};

// ============================================================================
// Namespace, state, and diagnostics
// ============================================================================

/// Namespace of the shared client library.
pub const HTTP_CLIENT_NAMESPACE: &str = "TypeSpec.HttpClient";

/// The `@experimental` decorator marks an API as experimental for the given
/// emitters (microsoft/typespec @typespec/http-client).
pub const DIAG_INCLUDE_AND_EXCLUDE_SCOPES: &str = "include-and-exclude-scopes";
/// Querying a scoped lifecycle without an emitter name.
pub const DIAG_USE_CLIENT_CONTEXT_WITHOUT_PROVIDER: &str = "use-client-context-without-provider";
/// `getClient` was called without a namespace.
pub const DIAG_UNDEFINED_NAMESPACE_FOR_CLIENT: &str = "undefined-namespace-for-client";
/// No client could be found in the specification.
pub const DIAG_CANT_FIND_CLIENT: &str = "cant-find-client";

/// Build the diagnostic map for the library.
pub fn create_http_client_library() -> DiagnosticMap {
    HashMap::from([
        (
            DIAG_INCLUDE_AND_EXCLUDE_SCOPES.to_string(),
            DiagnosticDefinition::error(
                "The @experimental should only either include or exclude scopes, not both.",
            ),
        ),
        (
            DIAG_USE_CLIENT_CONTEXT_WITHOUT_PROVIDER.to_string(),
            DiagnosticDefinition::error("No emitter scope provided to getClientFeatureLifecycle."),
        ),
        (
            DIAG_UNDEFINED_NAMESPACE_FOR_CLIENT.to_string(),
            DiagnosticDefinition::error("Undefined namespace for client."),
        ),
        (
            DIAG_CANT_FIND_CLIENT.to_string(),
            DiagnosticDefinition::error("Can't find any clients in the spec"),
        ),
    ])
}

// ============================================================================
// Emitter scoping
// ============================================================================

/// Emitters a lifecycle marker applies to.
///
/// Parsed from `emitterScope`, a comma separated list where a `!` prefix
/// excludes an emitter, e.g. `"myEmitter, !otherEmitter"`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EmitterFilter {
    /// Emitters the marker applies to.
    pub included_emitters: Vec<String>,
    /// Emitters the marker does not apply to.
    pub excluded_emitters: Vec<String>,
    /// Whether an `emitterScope` was supplied at all.
    pub is_scoped: bool,
}

impl EmitterFilter {
    /// Whether the filter contains both include and exclude entries, which is
    /// not allowed.
    pub fn has_include_and_exclude(&self) -> bool {
        !self.included_emitters.is_empty() && !self.excluded_emitters.is_empty()
    }
}

/// Parse an `emitterScope` value into an [`EmitterFilter`].
pub fn parse_scope_filter(scope: Option<&str>) -> EmitterFilter {
    let Some(scope) = scope else {
        return EmitterFilter::default();
    };
    let mut filter = EmitterFilter {
        is_scoped: true,
        ..EmitterFilter::default()
    };
    for part in scope.split(',') {
        let trimmed = part.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(excluded) = trimmed.strip_prefix('!') {
            filter.excluded_emitters.push(excluded.trim().to_string());
        } else {
            filter.included_emitters.push(trimmed.to_string());
        }
    }
    filter
}

/// Lifecycle stage of an API surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeatureLifecycleStage {
    /// The API is experimental.
    Experimental,
}

impl FeatureLifecycleStage {
    /// The stage name used by upstream (`"Experimental"`).
    pub fn as_str(self) -> &'static str {
        match self {
            FeatureLifecycleStage::Experimental => "Experimental",
        }
    }
}

/// Details attached to a lifecycle marker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeatureLifecycleDetails {
    /// Lifecycle stage.
    pub stage: FeatureLifecycleStage,
    /// Diagnostic id emitters should surface for this API.
    pub diagnostic_id: Option<String>,
    /// Experimental features this API depends on.
    pub depends_on: Vec<String>,
}

/// A lifecycle marker together with the emitters it applies to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopedFeatureLifecycle {
    /// Emitters the marker applies to.
    pub emitter_filter: EmitterFilter,
    /// Marker details.
    pub details: FeatureLifecycleDetails,
}

/// Options accepted by `@experimental`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExperimentalOptions {
    /// Comma separated emitter scope (include and `!` exclude entries).
    pub emitter_scope: Option<String>,
    /// Diagnostic id emitters should surface.
    pub diagnostic_id: Option<String>,
    /// Experimental features this API depends on.
    pub depends_on: Vec<String>,
}

/// Build the scoped lifecycle marker for `@experimental`.
///
/// Returns the marker and whether the options mixed include and exclude
/// scopes (in which case the caller reports
/// [`DIAG_INCLUDE_AND_EXCLUDE_SCOPES`]).
pub fn create_scoped_feature_lifecycle(
    options: &ExperimentalOptions,
) -> (ScopedFeatureLifecycle, bool) {
    let emitter_filter = parse_scope_filter(options.emitter_scope.as_deref());
    let conflict = emitter_filter.has_include_and_exclude();
    (
        ScopedFeatureLifecycle {
            emitter_filter,
            details: FeatureLifecycleDetails {
                stage: FeatureLifecycleStage::Experimental,
                diagnostic_id: options.diagnostic_id.clone(),
                depends_on: options.depends_on.clone(),
            },
        },
        conflict,
    )
}

/// A diagnostic produced while querying a lifecycle marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeatureLifecycleDiagnostic {
    /// A scoped marker was queried without an emitter name.
    MissingEmitterScope,
}

impl FeatureLifecycleDiagnostic {
    /// Diagnostic code for this warning.
    pub fn code(self) -> &'static str {
        match self {
            FeatureLifecycleDiagnostic::MissingEmitterScope => {
                DIAG_USE_CLIENT_CONTEXT_WITHOUT_PROVIDER
            }
        }
    }
}

/// Result of a lifecycle query: the value plus any diagnostics the caller
/// should report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeatureLifecycleResult<T> {
    /// Resolved value, or `None` when not set / not in scope.
    pub value: Option<T>,
    /// Diagnostics produced by the query.
    pub diagnostics: Vec<FeatureLifecycleDiagnostic>,
}

fn resolve_lifecycle(
    lifecycle: Option<&ScopedFeatureLifecycle>,
    emitter_name: Option<&str>,
) -> FeatureLifecycleResult<FeatureLifecycleDetails> {
    let Some(lifecycle) = lifecycle else {
        return FeatureLifecycleResult {
            value: None,
            diagnostics: Vec::new(),
        };
    };

    if !lifecycle.emitter_filter.is_scoped {
        return FeatureLifecycleResult {
            value: Some(lifecycle.details.clone()),
            diagnostics: Vec::new(),
        };
    }

    let Some(emitter_name) = emitter_name else {
        // Scoped marker, but the caller cannot say which emitter it is.
        return FeatureLifecycleResult {
            value: None,
            diagnostics: vec![FeatureLifecycleDiagnostic::MissingEmitterScope],
        };
    };

    let in_scope = if !lifecycle.emitter_filter.included_emitters.is_empty() {
        lifecycle
            .emitter_filter
            .included_emitters
            .iter()
            .any(|e| e == emitter_name)
    } else {
        !lifecycle
            .emitter_filter
            .excluded_emitters
            .iter()
            .any(|e| e == emitter_name)
    };

    FeatureLifecycleResult {
        value: in_scope.then(|| lifecycle.details.clone()),
        diagnostics: Vec::new(),
    }
}

/// Get the lifecycle phase of `target` for a given emitter.
///
/// Ported from `getClientFeatureLifecycle`.
pub fn get_client_feature_lifecycle(
    checker: &Checker,
    target: TypeId,
    emitter_name: Option<&str>,
) -> FeatureLifecycleResult<FeatureLifecycleStage> {
    let result = get_client_feature_lifecycle_details(checker, target, emitter_name);
    FeatureLifecycleResult {
        value: result.value.map(|d| d.stage),
        diagnostics: result.diagnostics,
    }
}

/// Get the lifecycle details of `target` for a given emitter.
///
/// Ported from `getClientFeatureLifecycleDetails`.
pub fn get_client_feature_lifecycle_details(
    checker: &Checker,
    target: TypeId,
    emitter_name: Option<&str>,
) -> FeatureLifecycleResult<FeatureLifecycleDetails> {
    resolve_lifecycle(checker.client_feature_lifecycles.get(&target), emitter_name)
}

// ============================================================================
// Library TypeSpec source
// ============================================================================

/// The TypeSpec source of the `TypeSpec.HttpClient` library.
pub const HTTP_CLIENT_TSP: &str = r#"
namespace TypeSpec.HttpClient;

/**
 * Specifies the target emitters that the decorator should apply. If not set,
 * the decorator applies to all emitters by default. Use "!" to exclude
 * specific emitters, for example: `!@typespec/http-client-js`.
 */
model ClientDecoratorOptions {
  /** Comma separated list of emitter names the decorator applies to. */
  emitterScope?: string;
}

/**
 * Options describing where a feature lifecycle decorator such as
 * `@experimental` applies.
 */
model FeatureLifecycleOptions {
  ...ClientDecoratorOptions;

  /** The diagnostic identifier surfaced by emitters for this experimental API. */
  diagnosticId?: string;

  /**
   * Diagnostic identifiers for experimental features this API depends on.
   */
  dependsOn?: string[];
}

/**
 * Mark a type or member as experimental, so generated clients can surface it
 * as such.
 */
extern dec experimental(target: unknown, options?: valueof FeatureLifecycleOptions);
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::types::{Type, TypeId};
    use crate::parser;

    fn check_with_http_client(source: &str) -> Checker {
        let parse_result = parser::parse(source);
        let mut checker = Checker::new();
        checker.register_decorators(vec![
            ("experimental", HTTP_CLIENT_NAMESPACE, "unknown"),
            ("service", "TypeSpec.Http", "Namespace"),
            ("get", "TypeSpec.Http", "Operation"),
            ("useAuth", "TypeSpec.Http", "unknown"),
        ]);
        checker.set_parse_result(parse_result.root_id, parse_result.builder.clone());
        checker.check_program();
        checker
    }

    fn property_of(checker: &Checker, model_name: &str, property: &str) -> TypeId {
        let model_id = checker.declared_types[model_name];
        match checker.get_type(model_id) {
            Some(Type::Model(m)) => m.properties[property],
            other => panic!("expected model, got {other:?}"),
        }
    }

    // ---- Ported from http-client/test/lib/experimental-decorator.test.ts ----

    #[test]
    fn test_feature_lifecycle_for_model_property() {
        let checker = check_with_http_client(
            r#"
            model MyModel {
              id: string;
              @experimental
              betaProp: string;
            }
            "#,
        );
        let prop = property_of(&checker, "MyModel", "betaProp");
        let result = get_client_feature_lifecycle(&checker, prop, None);
        assert_eq!(result.value, Some(FeatureLifecycleStage::Experimental));
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_feature_lifecycle_details() {
        let checker = check_with_http_client(
            r#"
            model MyModel {
              @experimental(#{ diagnosticId: "C", dependsOn: #["A", "B"] })
              betaProp: string;
            }
            "#,
        );
        let prop = property_of(&checker, "MyModel", "betaProp");
        let result = get_client_feature_lifecycle_details(&checker, prop, None);
        assert_eq!(
            result.value,
            Some(FeatureLifecycleDetails {
                stage: FeatureLifecycleStage::Experimental,
                diagnostic_id: Some("C".to_string()),
                depends_on: vec!["A".to_string(), "B".to_string()],
            })
        );
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_feature_lifecycle_filters_by_emitter_scope() {
        let checker = check_with_http_client(
            r#"
            model MyModel {
              @experimental(#{ emitterScope: "myEmitter", diagnosticId: "C", dependsOn: #["A"] })
              betaProp: string;
            }
            "#,
        );
        let prop = property_of(&checker, "MyModel", "betaProp");
        let in_scope = get_client_feature_lifecycle_details(&checker, prop, Some("myEmitter"));
        assert_eq!(in_scope.value.unwrap().diagnostic_id.as_deref(), Some("C"));
        let out_of_scope = get_client_feature_lifecycle_details(&checker, prop, Some("other"));
        assert_eq!(out_of_scope.value, None);
    }

    #[test]
    fn test_feature_lifecycle_multiple_scopes() {
        let checker = check_with_http_client(
            r#"
            model MyModel {
              @experimental(#{ emitterScope: "myEmitter, otherEmitter" })
              betaProp: string;
            }
            "#,
        );
        let prop = property_of(&checker, "MyModel", "betaProp");
        assert_eq!(
            get_client_feature_lifecycle(&checker, prop, Some("myEmitter")).value,
            Some(FeatureLifecycleStage::Experimental)
        );
        assert_eq!(
            get_client_feature_lifecycle(&checker, prop, Some("otherEmitter")).value,
            Some(FeatureLifecycleStage::Experimental)
        );
    }

    #[test]
    fn test_unscoped_marker_applies_to_any_emitter() {
        let checker = check_with_http_client(
            r#"
            model MyModel {
              @experimental
              betaProp: string;
            }
            "#,
        );
        let prop = property_of(&checker, "MyModel", "betaProp");
        assert_eq!(
            get_client_feature_lifecycle(&checker, prop, Some("myEmitter")).value,
            Some(FeatureLifecycleStage::Experimental)
        );
    }

    #[test]
    fn test_feature_lifecycle_not_in_scope() {
        let checker = check_with_http_client(
            r#"
            model MyModel {
              @experimental(#{ emitterScope: "notMyEmitter" })
              betaProp: string;
            }
            "#,
        );
        let prop = property_of(&checker, "MyModel", "betaProp");
        assert_eq!(
            get_client_feature_lifecycle(&checker, prop, Some("myEmitter")).value,
            None
        );
    }

    #[test]
    fn test_scoped_marker_without_emitter_query_reports_warning() {
        let checker = check_with_http_client(
            r#"
            model MyModel {
              @experimental(#{ emitterScope: "notMyEmitter" })
              betaProp: string;
            }
            "#,
        );
        let prop = property_of(&checker, "MyModel", "betaProp");
        let result = get_client_feature_lifecycle(&checker, prop, None);
        assert_eq!(result.value, None);
        assert_eq!(
            result.diagnostics,
            vec![FeatureLifecycleDiagnostic::MissingEmitterScope]
        );
    }

    #[test]
    fn test_excluded_scope_applies_outside_exclusion() {
        let checker = check_with_http_client(
            r#"
            model MyModel {
              @experimental(#{ emitterScope: "!notMyEmitter" })
              betaProp: string;
            }
            "#,
        );
        let prop = property_of(&checker, "MyModel", "betaProp");
        assert_eq!(
            get_client_feature_lifecycle(&checker, prop, Some("myEmitter")).value,
            Some(FeatureLifecycleStage::Experimental)
        );
    }

    #[test]
    fn test_excluded_scope_does_not_apply_to_excluded_emitter() {
        let checker = check_with_http_client(
            r#"
            model MyModel {
              @experimental(#{ emitterScope: "!myEmitter" })
              betaProp: string;
            }
            "#,
        );
        let prop = property_of(&checker, "MyModel", "betaProp");
        assert_eq!(
            get_client_feature_lifecycle(&checker, prop, Some("myEmitter")).value,
            None
        );
    }

    #[test]
    fn test_include_and_exclude_reports_diagnostic() {
        let checker = check_with_http_client(
            r#"
            model MyModel {
              @experimental(#{ emitterScope: "!myEmitter,otherEmitter" })
              betaProp: string;
            }
            "#,
        );
        assert!(
            checker.diagnostics().iter().any(|d| {
                d.code == DIAG_INCLUDE_AND_EXCLUDE_SCOPES
                    && d.message
                        == "The @experimental should only either include or exclude scopes, not both."
            }),
            "expected include-and-exclude-scopes diagnostic, got {:?}",
            checker.diagnostics()
        );
    }

    #[test]
    fn test_scope_filter_parsing() {
        assert_eq!(parse_scope_filter(None), EmitterFilter::default());
        assert_eq!(
            parse_scope_filter(Some("a, b")),
            EmitterFilter {
                included_emitters: vec!["a".to_string(), "b".to_string()],
                excluded_emitters: vec![],
                is_scoped: true,
            }
        );
        assert_eq!(
            parse_scope_filter(Some("!a,!b")),
            EmitterFilter {
                included_emitters: vec![],
                excluded_emitters: vec!["a".to_string(), "b".to_string()],
                is_scoped: true,
            }
        );
    }

    #[test]
    fn test_http_client_tsp_declarations() {
        assert!(HTTP_CLIENT_TSP.contains("dec experimental"));
        assert!(HTTP_CLIENT_TSP.contains("ClientDecoratorOptions"));
        assert!(HTTP_CLIENT_TSP.contains("FeatureLifecycleOptions"));
        let diagnostics = create_http_client_library();
        assert!(diagnostics.contains_key(DIAG_CANT_FIND_CLIENT));
        assert!(diagnostics.contains_key(DIAG_INCLUDE_AND_EXCLUDE_SCOPES));
    }
}
