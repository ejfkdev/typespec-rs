//! HTTP Authentication types and decorators
//!
//! Ported from TypeSpec @typespec/http authentication.ts

use crate::checker::types::TypeId;
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
