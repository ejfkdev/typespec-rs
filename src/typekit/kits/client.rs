//! Client typekit operations
//!
//! Ported from `@typespec/http-client` typekit kit `client`.
//! Re-exports the client model implemented in [`crate::libs::http_client`].

pub use crate::libs::http_client::client_library::get_client_name as get_name;
pub use crate::libs::http_client::{
    ClientConstructor, ClientConstructorParameter, ClientParameterType, InternalClient,
    flat_clients as flat, get_client, get_client_auth as get_auth, get_client_constructor,
    get_client_feature_lifecycle, get_client_feature_lifecycle_details, get_parent_client,
    have_same_constructor, is_publicly_initializable, list_http_operations, list_servers,
};
