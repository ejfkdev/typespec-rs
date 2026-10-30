//! HTTP Authentication types and decorators
//!
//! Ported from TypeSpec @typespec/http authentication.ts

use crate::checker::Checker;
use crate::checker::types::{Type, TypeId};
use crate::state_accessors::StateAccessors;

// ============================================================================
// State keys
// ============================================================================

/// State key for @useAuth decorator
pub const STATE_AUTHENTICATION: &str = "TypeSpec.Http.authentication";
/// State key for @useHttpAuth decorator
pub const STATE_USE_HTTP_AUTH: &str = "TypeSpec.Http.useHttpAuth";

// ============================================================================
// HTTP authentication types
// ============================================================================

/// Authentication configuration for a service.
/// Ported from TS Authentication interface.
#[derive(Debug, Clone)]
pub struct Authentication {
    /// Authentication options (any one can be used independently)
    pub options: Vec<AuthenticationOption>,
}

/// A single authentication option.
/// Ported from TS AuthenticationOption interface.
#[derive(Debug, Clone)]
pub struct AuthenticationOption {
    /// All schemes in this option must be used together
    pub schemes: Vec<HttpAuth>,
}

/// HTTP authentication scheme types.
/// Ported from TS HttpAuth union type.
#[derive(Debug, Clone)]
pub enum HttpAuth {
    Basic(BasicAuth),
    Bearer(BearerAuth),
    ApiKey(ApiKeyAuth),
    Oauth2(Oauth2Auth),
    OpenIdConnect(OpenIdConnectAuth),
    NoAuth(NoAuth),
}

/// Base type for HTTP authentication schemes.
#[derive(Debug, Clone)]
pub struct HttpAuthBase {
    /// Identifier for the authentication scheme
    pub id: String,
    /// Optional description
    pub description: Option<String>,
}

/// Basic authentication scheme.
#[derive(Debug, Clone)]
pub struct BasicAuth {
    /// Base properties
    pub base: HttpAuthBase,
}

/// Bearer token authentication scheme.
#[derive(Debug, Clone)]
pub struct BearerAuth {
    /// Base properties
    pub base: HttpAuthBase,
}

/// API key authentication scheme.
#[derive(Debug, Clone)]
pub struct ApiKeyAuth {
    /// Base properties
    pub base: HttpAuthBase,
    /// Location of the API key
    pub location: ApiKeyLocation,
    /// Name of the API key parameter
    pub name: String,
}

/// Where the API key is sent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiKeyLocation {
    Header,
    Query,
    Cookie,
}

/// OAuth2 authentication scheme.
#[derive(Debug, Clone)]
pub struct Oauth2Auth {
    /// Base properties
    pub base: HttpAuthBase,
    /// OAuth2 flows
    pub flows: Vec<OAuth2Flow>,
}

/// OAuth2 flow definition.
#[derive(Debug, Clone)]
pub struct OAuth2Flow {
    /// Flow type
    pub flow_type: OAuth2FlowType,
    /// Authorization URL (for implicit/authorizationCode)
    pub authorization_url: Option<String>,
    /// Token URL (for password/clientCredentials/authorizationCode)
    pub token_url: Option<String>,
    /// Refresh URL
    pub refresh_url: Option<String>,
    /// Scopes
    pub scopes: Vec<OAuth2Scope>,
}

/// OAuth2 flow types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OAuth2FlowType {
    Implicit,
    Password,
    ClientCredentials,
    AuthorizationCode,
}

/// OAuth2 scope definition.
#[derive(Debug, Clone)]
pub struct OAuth2Scope {
    /// Scope value
    pub value: String,
    /// Scope description
    pub description: Option<String>,
}

/// OpenID Connect authentication scheme.
#[derive(Debug, Clone)]
pub struct OpenIdConnectAuth {
    /// Base properties
    pub base: HttpAuthBase,
    /// OpenID Connect discovery URL
    pub open_id_connect_url: String,
    /// Scope names required for operations that use this scheme
    /// (microsoft/typespec#11153).
    pub scopes: Vec<String>,
}

/// No authentication scheme.
#[derive(Debug, Clone)]
pub struct NoAuth {
    /// Base properties
    pub base: HttpAuthBase,
}

// ============================================================================
// HTTP Auth Reference types
// Ported from TS HttpAuthRef, AuthenticationReference, etc.
// ============================================================================

/// Reference to an HTTP authentication scheme.
/// Ported from TS `type HttpAuthRef = AnyHttpAuthRef | OAuth2HttpAuthRef | NoHttpAuthRef`.
#[derive(Debug, Clone)]
pub enum HttpAuthRef {
    /// Reference to any HTTP auth scheme
    Any(AnyHttpAuthRef),
    /// Reference to an OAuth2 auth scheme with scope subset
    Oauth2(OAuth2HttpAuthRef),
    /// Reference to no auth
    NoAuth(NoHttpAuthRef),
}

/// Reference to any HTTP auth scheme.
/// Ported from TS `interface AnyHttpAuthRef`.
#[derive(Debug, Clone)]
pub struct AnyHttpAuthRef {
    /// Scope names required for this scheme in the containing auth option.
    /// Empty for schemes that do not carry scopes; populated for
    /// `openIdConnect` (microsoft/typespec#11153).
    pub scopes: Vec<String>,
    /// The referenced auth scheme
    pub auth: HttpAuth,
}

/// Reference to no auth scheme.
/// Ported from TS `interface NoHttpAuthRef`.
#[derive(Debug, Clone)]
pub struct NoHttpAuthRef {
    /// The referenced NoAuth scheme
    pub auth: NoAuth,
}

/// Reference to an OAuth2 auth scheme with a subset of scopes.
/// Ported from TS `interface OAuth2HttpAuthRef`.
#[derive(Debug, Clone)]
pub struct OAuth2HttpAuthRef {
    /// The referenced OAuth2 auth scheme
    pub auth: Oauth2Auth,
    /// The scopes needed (subset of all scopes defined at auth)
    pub scopes: Vec<String>,
}

/// Authentication reference for an operation.
/// Ported from TS `interface AuthenticationReference`.
#[derive(Debug, Clone)]
pub struct AuthenticationReference {
    /// Authentication options (any one can be used independently)
    pub options: Vec<AuthenticationOptionReference>,
}

/// A single authentication option reference.
/// Ported from TS `interface AuthenticationOptionReference`.
#[derive(Debug, Clone)]
pub struct AuthenticationOptionReference {
    /// All auth refs in this option must be used together
    pub all: Vec<HttpAuthRef>,
}

/// Full authentication information for an HTTP service.
/// Ported from TS `interface HttpServiceAuthentication`.
#[derive(Debug, Clone)]
pub struct HttpServiceAuthentication {
    /// All authentication schemes used in this service
    pub schemes: Vec<HttpAuth>,
    /// Default authentication for operations in this service
    pub default_auth: AuthenticationReference,
    /// Authentication overrides for individual operations (keyed by operation TypeId)
    pub operations_auth: Vec<(TypeId, AuthenticationReference)>,
}

// ============================================================================
// OAuth2 flow types (detailed)
// Ported from TS AuthorizationCodeFlow, ImplicitFlow, etc.
// ============================================================================

/// OAuth2 Authorization Code flow.
/// Ported from TS `interface AuthorizationCodeFlow`.
#[derive(Debug, Clone)]
pub struct AuthorizationCodeFlow {
    /// Authorization URL
    pub authorization_url: String,
    /// Token URL
    pub token_url: String,
    /// Refresh URL
    pub refresh_url: Option<String>,
    /// Scopes
    pub scopes: Vec<OAuth2Scope>,
}

/// OAuth2 Implicit flow.
/// Ported from TS `interface ImplicitFlow`.
#[derive(Debug, Clone)]
pub struct ImplicitFlow {
    /// Authorization URL
    pub authorization_url: String,
    /// Refresh URL
    pub refresh_url: Option<String>,
    /// Scopes
    pub scopes: Vec<OAuth2Scope>,
}

/// OAuth2 Resource Owner Password flow.
/// Ported from TS `interface PasswordFlow`.
#[derive(Debug, Clone)]
pub struct PasswordFlow {
    /// Authorization URL
    pub authorization_url: String,
    /// Refresh URL
    pub refresh_url: Option<String>,
    /// Scopes
    pub scopes: Vec<OAuth2Scope>,
}

/// OAuth2 Client Credentials flow.
/// Ported from TS `interface ClientCredentialsFlow`.
#[derive(Debug, Clone)]
pub struct ClientCredentialsFlow {
    /// Token URL
    pub token_url: String,
    /// Refresh URL
    pub refresh_url: Option<String>,
    /// Scopes
    pub scopes: Vec<OAuth2Scope>,
}

// ============================================================================
// Authentication decorator functions
// Ported from TS setAuthentication/getAuthentication
// ============================================================================

/// Set authentication configuration for a target.
/// Ported from TS setAuthentication().
pub fn set_authentication(state: &mut StateAccessors, target: TypeId, auth: &Authentication) {
    // Serialize as simplified format: "opt1:scheme1,scheme2;opt2:scheme3"
    let parts: Vec<String> = auth
        .options
        .iter()
        .map(|opt| {
            let schemes: Vec<String> = opt.schemes.iter().map(auth_scheme_name).collect();
            schemes.join(",")
        })
        .collect();
    state.set_state(STATE_AUTHENTICATION, target, parts.join(";"));
}

/// Get authentication configuration for a target.
/// Ported from TS getAuthentication().
pub fn get_authentication(state: &StateAccessors, target: TypeId) -> Option<String> {
    state
        .get_state(STATE_AUTHENTICATION, target)
        .map(|s| s.to_string())
}

/// Parse the authentication configured on a target into structured form.
///
/// The state is serialized by [`set_authentication`]; this restores the scheme
/// kind (and the api key name / OpenID Connect URL when present) so client SDK
/// generators can inspect it.
pub fn parse_authentication(state: &StateAccessors, target: TypeId) -> Option<Authentication> {
    let serialized = get_authentication(state, target)?;
    let options: Vec<AuthenticationOption> = serialized
        .split(';')
        .filter(|option| !option.is_empty())
        .map(|option| {
            let schemes = option
                .split(',')
                .filter(|name| !name.is_empty())
                .filter_map(parse_authentication_scheme)
                .collect();
            AuthenticationOption { schemes }
        })
        .filter(|option: &AuthenticationOption| !option.schemes.is_empty())
        .collect();
    (!options.is_empty()).then_some(Authentication { options })
}

fn parse_authentication_scheme(serialized: &str) -> Option<HttpAuth> {
    let base = HttpAuthBase {
        id: serialized.to_string(),
        description: None,
    };
    if serialized == "BasicAuth" {
        Some(HttpAuth::Basic(BasicAuth { base }))
    } else if serialized == "BearerAuth" {
        Some(HttpAuth::Bearer(BearerAuth { base }))
    } else if let Some(name) = serialized
        .strip_prefix("ApiKeyAuth(")
        .and_then(|s| s.strip_suffix(')'))
    {
        Some(HttpAuth::ApiKey(ApiKeyAuth {
            base,
            location: ApiKeyLocation::Header,
            name: name.to_string(),
        }))
    } else if serialized == "Oauth2Auth" {
        Some(HttpAuth::Oauth2(Oauth2Auth {
            base,
            flows: Vec::new(),
        }))
    } else if let Some(url) = serialized
        .strip_prefix("OpenIdConnectAuth(")
        .and_then(|s| s.strip_suffix(')'))
    {
        Some(HttpAuth::OpenIdConnect(OpenIdConnectAuth {
            base,
            open_id_connect_url: url.to_string(),
            scopes: Vec::new(),
        }))
    } else if serialized == "NoAuth" {
        Some(HttpAuth::NoAuth(NoAuth { base }))
    } else {
        // Unknown scheme name; keep it as an opaque no-auth scheme so callers
        // can still see that authentication is configured.
        Some(HttpAuth::NoAuth(NoAuth { base }))
    }
}

fn auth_scheme_name(auth: &HttpAuth) -> String {
    match auth {
        HttpAuth::Basic(_) => "BasicAuth".to_string(),
        HttpAuth::Bearer(_) => "BearerAuth".to_string(),
        HttpAuth::ApiKey(a) => format!("ApiKeyAuth({})", a.name),
        HttpAuth::Oauth2(_) => "Oauth2Auth".to_string(),
        HttpAuth::OpenIdConnect(a) => format!("OpenIdConnectAuth({})", a.open_id_connect_url),
        HttpAuth::NoAuth(_) => "NoAuth".to_string(),
    }
}

// ============================================================================
// Authentication scheme type names (client SDK support)
// ============================================================================

/// The TypeSpec `HttpAuth.type` value for a scheme.
///
/// Ported from the `type` property of the auth models in `@typespec/http`
/// (`http`, `apiKey`, `oauth2`, `openIdConnect`). `NoAuth` has no credential.
pub fn auth_scheme_type_name(auth: &HttpAuth) -> Option<&'static str> {
    match auth {
        HttpAuth::Basic(_) | HttpAuth::Bearer(_) => Some("http"),
        HttpAuth::ApiKey(_) => Some("apiKey"),
        HttpAuth::Oauth2(_) => Some("oauth2"),
        HttpAuth::OpenIdConnect(_) => Some("openIdConnect"),
        HttpAuth::NoAuth(_) => None,
    }
}

/// Get the deduplicated credential type names of the authentication configured
/// on `target` (a service namespace).
///
/// Client SDK generators turn these into the `credential` constructor
/// parameter (one string literal per scheme, or a union of literals).
pub fn get_authentication_scheme_types(state: &StateAccessors, target: TypeId) -> Vec<String> {
    let Some(serialized) = get_authentication(state, target) else {
        return Vec::new();
    };
    let mut types: Vec<String> = Vec::new();
    for scheme in serialized.split([';', ',']) {
        let scheme = scheme.trim();
        if scheme.is_empty() {
            continue;
        }
        let type_name = if scheme.starts_with("BasicAuth") || scheme.starts_with("BearerAuth") {
            "http"
        } else if scheme.starts_with("ApiKeyAuth") {
            "apiKey"
        } else if scheme.starts_with("Oauth2Auth") {
            "oauth2"
        } else if scheme.starts_with("OpenIdConnectAuth") {
            "openIdConnect"
        } else {
            // NoAuth and unknown scheme names carry no credential.
            continue;
        };
        if !types.iter().any(|t| t == type_name) {
            types.push(type_name.to_string());
        }
    }
    types
}

// ============================================================================
// Resolving an @useAuth argument
// ============================================================================

/// Resolve an `@useAuth` argument type into an [`Authentication`].
///
/// Ported from the TS `$useAuth` implementation. The argument is one of:
/// - a single auth scheme model instance (`BasicAuth`, `ApiKeyAuth<...>`, ...)
/// - a tuple of schemes that must all be used together (`[A, B]`)
/// - a union of alternatives (`A | B`)
///
/// Returns `None` when the type is not a recognizable auth reference.
pub fn resolve_auth_reference(checker: &Checker, type_id: TypeId) -> Option<Authentication> {
    let resolved = checker.resolve_alias_chain(type_id);
    match checker.get_type(resolved) {
        Some(Type::Union(u)) => {
            let mut options = Vec::new();
            for name in &u.variant_names {
                if let Some(&variant_id) = u.variants.get(name)
                    && let Some(Type::UnionVariant(v)) = checker.get_type(variant_id)
                    && let Some(auth) = resolve_auth_reference(checker, v.r#type)
                {
                    options.extend(auth.options);
                }
            }
            (!options.is_empty()).then_some(Authentication { options })
        }
        Some(Type::Tuple(t)) => {
            let mut schemes = Vec::new();
            for &value in &t.values {
                if let Some(auth) = resolve_auth_reference(checker, value) {
                    schemes.extend(auth.options.into_iter().flat_map(|o| o.schemes));
                }
            }
            (!schemes.is_empty()).then_some(Authentication {
                options: vec![AuthenticationOption { schemes }],
            })
        }
        Some(Type::Model(m)) => {
            let scheme = resolve_auth_model(checker, m)?;
            Some(Authentication {
                options: vec![AuthenticationOption {
                    schemes: vec![scheme],
                }],
            })
        }
        _ => None,
    }
}

/// Resolve a single auth scheme model instance.
fn resolve_auth_model(
    checker: &Checker,
    model: &crate::checker::types::ModelType,
) -> Option<HttpAuth> {
    let scheme_type = get_model_string_property(checker, model, "type")?;
    let base = HttpAuthBase {
        id: scheme_type.clone(),
        description: None,
    };
    match scheme_type.as_str() {
        "http" => match get_model_string_property(checker, model, "scheme") {
            Some(scheme) if scheme.eq_ignore_ascii_case("basic") => {
                Some(HttpAuth::Basic(BasicAuth { base }))
            }
            Some(scheme) if scheme.eq_ignore_ascii_case("bearer") => {
                Some(HttpAuth::Bearer(BearerAuth { base }))
            }
            // Unknown HTTP scheme: treat as bearer token auth.
            Some(_) => Some(HttpAuth::Bearer(BearerAuth { base })),
            None => None,
        },
        "apiKey" => {
            let location = match get_model_string_property(checker, model, "in").as_deref() {
                Some("query") => ApiKeyLocation::Query,
                Some("cookie") => ApiKeyLocation::Cookie,
                _ => ApiKeyLocation::Header,
            };
            let name = get_model_string_property(checker, model, "name").unwrap_or_default();
            Some(HttpAuth::ApiKey(ApiKeyAuth {
                base,
                location,
                name,
            }))
        }
        "oauth2" => Some(HttpAuth::Oauth2(Oauth2Auth {
            base,
            flows: Vec::new(),
        })),
        "openIdConnect" => Some(HttpAuth::OpenIdConnect(OpenIdConnectAuth {
            base,
            open_id_connect_url: get_model_string_property(checker, model, "openIdConnectUrl")
                .unwrap_or_default(),
            scopes: Vec::new(),
        })),
        "none" => Some(HttpAuth::NoAuth(NoAuth { base })),
        _ => None,
    }
}

/// Read a string-valued property from a model, accepting either a string
/// literal or an enum member (e.g. `in: "header"` or referenced enum members).
fn get_model_string_property(
    checker: &Checker,
    model: &crate::checker::types::ModelType,
    name: &str,
) -> Option<String> {
    let prop_id = model.properties.get(name).copied()?;
    let prop_type = match checker.get_type(prop_id) {
        Some(Type::ModelProperty(p)) => p.r#type,
        _ => return None,
    };
    match checker.get_type(checker.resolve_alias_chain(prop_type)) {
        Some(Type::String(s)) => Some(s.value.clone()),
        Some(Type::EnumMember(m)) => Some(m.name.clone()),
        _ => None,
    }
}

// ============================================================================
// Authentication decorator
// ============================================================================

string_decorator!(apply_use_auth, get_use_auth, STATE_USE_HTTP_AUTH);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_get_authentication() {
        let mut state = StateAccessors::new();
        let auth = Authentication {
            options: vec![AuthenticationOption {
                schemes: vec![HttpAuth::Bearer(BearerAuth {
                    base: HttpAuthBase {
                        id: "bearer".to_string(),
                        description: None,
                    },
                })],
            }],
        };
        assert_eq!(get_authentication(&state, 1), None);
        set_authentication(&mut state, 1, &auth);
        assert!(get_authentication(&state, 1).is_some());
    }

    #[test]
    fn test_use_auth() {
        let mut state = StateAccessors::new();
        assert_eq!(get_use_auth(&state, 1), None);
        apply_use_auth(&mut state, 1, "BearerAuth");
        assert_eq!(get_use_auth(&state, 1), Some("BearerAuth".to_string()));
    }

    #[test]
    fn test_http_auth_ref_any() {
        let auth_ref = HttpAuthRef::Any(AnyHttpAuthRef {
            auth: HttpAuth::Basic(BasicAuth {
                base: HttpAuthBase {
                    id: "basic".to_string(),
                    description: None,
                },
            }),
            scopes: vec![],
        });
        match auth_ref {
            HttpAuthRef::Any(r) => assert!(matches!(r.auth, HttpAuth::Basic(_))),
            _ => panic!("Expected Any variant"),
        }
    }

    #[test]
    fn test_http_auth_ref_oauth2() {
        let auth_ref = HttpAuthRef::Oauth2(OAuth2HttpAuthRef {
            auth: Oauth2Auth {
                base: HttpAuthBase {
                    id: "oauth2".to_string(),
                    description: None,
                },
                flows: vec![],
            },
            scopes: vec!["read".to_string(), "write".to_string()],
        });
        match auth_ref {
            HttpAuthRef::Oauth2(r) => assert_eq!(r.scopes, vec!["read", "write"]),
            _ => panic!("Expected Oauth2 variant"),
        }
    }

    /// Ported from TS (microsoft/typespec#11153): the scheme-agnostic `any`
    /// auth ref carries the requirement scopes for openIdConnect schemes.
    #[test]
    fn test_any_auth_ref_carries_openid_connect_scopes() {
        let auth_ref = HttpAuthRef::Any(AnyHttpAuthRef {
            auth: HttpAuth::OpenIdConnect(OpenIdConnectAuth {
                base: HttpAuthBase {
                    id: "oidc".to_string(),
                    description: None,
                },
                open_id_connect_url: "https://example.org/.well-known/openid-configuration"
                    .to_string(),
                scopes: vec![],
            }),
            scopes: vec!["read".to_string(), "write".to_string()],
        });
        match auth_ref {
            HttpAuthRef::Any(r) => {
                assert!(matches!(r.auth, HttpAuth::OpenIdConnect(_)));
                assert_eq!(r.scopes, vec!["read", "write"]);
            }
            _ => panic!("Expected Any variant"),
        }
    }

    /// Ported from TS (microsoft/typespec#11153): the OpenIdConnectAuth
    /// scheme accepts scope names.
    #[test]
    fn test_openid_connect_auth_scopes_field() {
        let auth = OpenIdConnectAuth {
            base: HttpAuthBase {
                id: "oidc".to_string(),
                description: None,
            },
            open_id_connect_url: "https://example.org/connect".to_string(),
            scopes: vec!["openid".to_string(), "profile".to_string()],
        };
        assert_eq!(auth.scopes, vec!["openid", "profile"]);
    }

    #[test]
    fn test_authentication_reference() {
        let auth_ref = AuthenticationReference {
            options: vec![AuthenticationOptionReference {
                all: vec![HttpAuthRef::NoAuth(NoHttpAuthRef {
                    auth: NoAuth {
                        base: HttpAuthBase {
                            id: "none".to_string(),
                            description: None,
                        },
                    },
                })],
            }],
        };
        assert_eq!(auth_ref.options.len(), 1);
    }

    #[test]
    fn test_http_service_authentication() {
        let service_auth = HttpServiceAuthentication {
            schemes: vec![HttpAuth::NoAuth(NoAuth {
                base: HttpAuthBase {
                    id: "none".to_string(),
                    description: None,
                },
            })],
            default_auth: AuthenticationReference { options: vec![] },
            operations_auth: vec![(42, AuthenticationReference { options: vec![] })],
        };
        assert_eq!(service_auth.schemes.len(), 1);
        assert_eq!(service_auth.operations_auth.len(), 1);
    }

    #[test]
    fn test_authorization_code_flow() {
        let flow = AuthorizationCodeFlow {
            authorization_url: "https://auth.example.com/authorize".to_string(),
            token_url: "https://auth.example.com/token".to_string(),
            refresh_url: None,
            scopes: vec![OAuth2Scope {
                value: "read".to_string(),
                description: None,
            }],
        };
        assert_eq!(flow.authorization_url, "https://auth.example.com/authorize");
        assert_eq!(flow.scopes.len(), 1);
    }

    #[test]
    fn test_implicit_flow() {
        let flow = ImplicitFlow {
            authorization_url: "https://auth.example.com/authorize".to_string(),
            refresh_url: Some("https://auth.example.com/refresh".to_string()),
            scopes: vec![],
        };
        assert!(flow.refresh_url.is_some());
    }

    #[test]
    fn test_client_credentials_flow() {
        let flow = ClientCredentialsFlow {
            token_url: "https://auth.example.com/token".to_string(),
            refresh_url: None,
            scopes: vec![],
        };
        assert_eq!(flow.token_url, "https://auth.example.com/token");
    }
}
