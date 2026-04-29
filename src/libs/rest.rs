//! @typespec/rest - REST API Decorators and Resource Types
//!
//! Ported from TypeSpec packages/rest
//!
//! Provides decorators for REST API modeling:
//! - `@autoRoute` - Auto-generate route from resource keys
//! - `@segment` - Define URL path segment
//! - `@segmentOf` - Get URL segment of a model
//! - `@actionSeparator` - Define action separator in routes
//! - `@resource` - Mark model as a resource type
//! - `@parentResource` - Define parent resource relationship
//! - `@readsResource`, `@createsResource`, etc. - CRUD operation decorators
//! - `@action`, `@collectionAction` - Custom action decorators
//! - `@copyResourceKeyParameters` - Copy key parameters
//!
//! Also provides:
//! - `ResourceLocation<T>` scalar type
//!
//! ## Helper Functions
//! - `is_auto_route(state, target)` - Check if target has @autoRoute
//! - `get_segment(state, target)` - Get the segment for a target
//! - `is_resource(state, target)` - Check if a model is a resource type
//! - `get_parent_resource(state, target)` - Get parent resource
//! - `is_key(state, target)` - Check if a property is a resource key
//!
//! Depends on @typespec/http

use crate::checker::types::TypeId;
use crate::diagnostics::{DiagnosticDefinition, DiagnosticMap};
use crate::state_accessors::StateAccessors;
use std::collections::HashMap;

// ============================================================================
// Diagnostic codes
// ============================================================================

/// Diagnostic: Cannot copy keys from non-key type
pub const DIAG_NOT_KEY_TYPE: &str = "not-key-type";
/// Diagnostic: Resource missing @key decorator
pub const DIAG_RESOURCE_MISSING_KEY: &str = "resource-missing-key";
/// Diagnostic: Resource missing @error decorator
pub const DIAG_RESOURCE_MISSING_ERROR: &str = "resource-missing-error";
/// Diagnostic: Duplicate key on resource
pub const DIAG_DUPLICATE_KEY: &str = "duplicate-key";
/// Diagnostic: Key name conflicts with parent/child
pub const DIAG_DUPLICATE_PARENT_KEY: &str = "duplicate-parent-key";
/// Diagnostic: Action name cannot be empty
pub const DIAG_INVALID_ACTION_NAME: &str = "invalid-action-name";
/// Diagnostic: Shared route needs explicit action name
pub const DIAG_SHARED_ROUTE_UNSPECIFIED_ACTION_NAME: &str = "shared-route-unspecified-action-name";
/// Diagnostic: Circular parent resource
pub const DIAG_CIRCULAR_PARENT_RESOURCE: &str = "circular-parent-resource";

// ============================================================================
// State keys (fully qualified with namespace)
// ============================================================================

/// State key for @autoRoute decorator
pub const STATE_AUTO_ROUTE: &str = "TypeSpec.Rest.autoRoute";
/// State key for @segment decorator
pub const STATE_SEGMENT: &str = "TypeSpec.Rest.segment";
/// State key for @resource decorator
pub const STATE_RESOURCE: &str = "TypeSpec.Rest.resource";
/// State key for @parentResource decorator
pub const STATE_PARENT_RESOURCE: &str = "TypeSpec.Rest.parentResource";
/// State key for @key decorator
pub const STATE_KEY: &str = "TypeSpec.Rest.key";
/// State key for action type tracking
pub const STATE_ACTION_TYPE: &str = "TypeSpec.Rest.actionType";
/// State key for @segmentOf decorator
pub const STATE_SEGMENT_OF: &str = "TypeSpec.Rest.segmentOf";
/// State key for @actionSeparator decorator
pub const STATE_ACTION_SEPARATOR: &str = "TypeSpec.Rest.actionSeparator";
/// State key for resource operation decorators
pub const STATE_RESOURCE_OPERATION: &str = "TypeSpec.Rest.resourceOperation";
/// State key for @actionSegment decorator (private)
pub const STATE_ACTION_SEGMENT: &str = "TypeSpec.Rest.actionSegment";
/// State key for @resourceLocation decorator (private)
pub const STATE_RESOURCE_LOCATION: &str = "TypeSpec.Rest.resourceLocation";
/// State key for @resourceTypeForKeyParam decorator (private)
pub const STATE_RESOURCE_TYPE_FOR_KEY_PARAM: &str = "TypeSpec.Rest.resourceTypeForKeyParam";
/// State key for @copyResourceKeyParameters decorator
pub const STATE_COPY_RESOURCE_KEY_PARAMS: &str = "TypeSpec.Rest.copyResourceKeyParameters";
/// State key for resource type key mapping
pub const STATE_RESOURCE_TYPE_KEY: &str = "TypeSpec.Rest.resourceTypeKey";

/// Namespace for REST types
pub const REST_NAMESPACE: &str = "TypeSpec.Rest";

// ============================================================================
// Resource operation kinds
// ============================================================================

/// Kinds of resource operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceOperationKind {
    /// Read operation
    Reads,
    /// Create operation
    Creates,
    /// Create or replace operation
    CreatesOrReplaces,
    /// Create or update operation
    CreatesOrUpdates,
    /// Update operation
    Updates,
    /// Delete operation
    Deletes,
    /// List operation
    Lists,
}

impl ResourceOperationKind {
    /// Get the decorator name for this operation kind
    pub fn decorator_name(&self) -> &'static str {
        match self {
            ResourceOperationKind::Reads => "readsResource",
            ResourceOperationKind::Creates => "createsResource",
            ResourceOperationKind::CreatesOrReplaces => "createsOrReplacesResource",
            ResourceOperationKind::CreatesOrUpdates => "createsOrUpdatesResource",
            ResourceOperationKind::Updates => "updatesResource",
            ResourceOperationKind::Deletes => "deletesResource",
            ResourceOperationKind::Lists => "listsResource",
        }
    }
}

// ============================================================================
// Action type
// ============================================================================

/// Type of action (item-scoped or collection-scoped)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActionType {
    /// Item-scoped action (e.g., /pets/{petId}/my-action)
    ItemAction,
    /// Collection-scoped action (e.g., /pets/my-action)
    CollectionAction,
}

// ============================================================================
// Action separator
// ============================================================================

string_enum! {
    /// Separator between action segment and the rest of the URL
    pub enum ActionSeparator {
        /// Slash separator: /
        Slash => "/",
        /// Colon separator: :
        Colon => ":",
        /// Slash-colon separator: /:
        SlashColon => "/:",
    }
}

// ============================================================================
// Action details
// ============================================================================

/// Details about an action decorator.
/// Ported from TS ActionDetails.
#[derive(Debug, Clone)]
pub struct ActionDetails {
    /// The kind of action (automatic name or explicit)
    pub kind: ActionNameKind,
    /// The resource type for collection actions
    pub resource_type: Option<TypeId>,
}

string_enum! {
    /// Whether the action name was automatically derived or explicitly provided.
    pub enum ActionNameKind {
        /// Name was automatically derived from operation name
        Automatic => "automatic",
        /// Name was explicitly provided in decorator argument
        Explicit => "explicit",
    }
}

// ============================================================================
// Resource key information
// ============================================================================

/// Information about a resource key property.
/// Ported from TS ResourceKey.
#[derive(Debug, Clone, PartialEq)]
pub struct ResourceKey {
    /// The resource type that owns the key
    pub resource_type: TypeId,
    /// The key property
    pub key_property: TypeId,
}

// ============================================================================
// Decorator implementations
// ============================================================================

flag_decorator!(apply_auto_route, is_auto_route, STATE_AUTO_ROUTE);
string_decorator!(apply_segment, get_segment, STATE_SEGMENT);

string_decorator!(apply_resource, get_resource_collection_name, STATE_RESOURCE);

/// Check if a model is a resource type.
pub fn is_resource(state: &StateAccessors, target: TypeId) -> bool {
    state.get_state(STATE_RESOURCE, target).is_some()
}

typeid_decorator!(
    apply_parent_resource,
    get_parent_resource,
    STATE_PARENT_RESOURCE
);

flag_decorator!(apply_key, is_key, STATE_KEY);

/// Implementation of the `@action` decorator.
/// Marks an operation as an item-scoped action.
pub fn apply_action(state: &mut StateAccessors, target: TypeId, name: Option<&str>) {
    let kind = if name.is_some() {
        ActionNameKind::Explicit
    } else {
        ActionNameKind::Automatic
    };
    let value = format!("action:{}:{}", kind.as_str(), name.unwrap_or(""));
    state.set_state(STATE_ACTION_TYPE, target, value);
}

/// Implementation of the `@collectionAction` decorator.
/// Marks an operation as a collection-scoped action.
pub fn apply_collection_action(
    state: &mut StateAccessors,
    target: TypeId,
    resource_type: TypeId,
    name: Option<&str>,
) {
    let kind = if name.is_some() {
        ActionNameKind::Explicit
    } else {
        ActionNameKind::Automatic
    };
    let value = format!(
        "collectionAction:{}:{}:{}",
        kind.as_str(),
        resource_type,
        name.unwrap_or("")
    );
    state.set_state(STATE_ACTION_TYPE, target, value);
}

/// Get action details for an operation.
pub fn get_action_details(state: &StateAccessors, target: TypeId) -> Option<ActionDetails> {
    state.get_state(STATE_ACTION_TYPE, target).and_then(|s| {
        let parts: Vec<&str> = s.splitn(4, ':').collect();
        if parts.len() < 2 {
            return None;
        }
        match parts[0] {
            "action" => Some(ActionDetails {
                kind: ActionNameKind::parse_str(parts[1])?,
                resource_type: None,
            }),
            "collectionAction" => {
                let resource_type = parts.get(2).and_then(|s| s.parse::<u32>().ok());
                Some(ActionDetails {
                    kind: ActionNameKind::parse_str(parts[1])?,
                    resource_type,
                })
            }
            _ => None,
        }
    })
}

// ============================================================================
// @segmentOf decorator
// ============================================================================

typeid_decorator!(apply_segment_of, get_segment_of, STATE_SEGMENT_OF);

// ============================================================================
// @actionSeparator decorator
// ============================================================================

/// Apply @actionSeparator decorator.
/// Ported from TS $actionSeparator().
pub fn apply_action_separator(
    state: &mut StateAccessors,
    target: TypeId,
    separator: ActionSeparator,
) {
    state.set_state(
        STATE_ACTION_SEPARATOR,
        target,
        separator.as_str().to_string(),
    );
}

/// Get the action separator for a target.
/// Ported from TS getActionSeparator().
pub fn get_action_separator(state: &StateAccessors, target: TypeId) -> Option<ActionSeparator> {
    state
        .get_state(STATE_ACTION_SEPARATOR, target)
        .and_then(ActionSeparator::parse_str)
}

// ============================================================================
// Resource operation decorators
// Ported from TS $readsResource, $createsResource, etc.
// ============================================================================

/// Resource operation data stored in state.
/// Format: "kind::resourceTypeId"
#[derive(Debug, Clone, PartialEq)]
pub struct ResourceOperation {
    /// The kind of resource operation
    pub kind: ResourceOperationKind,
    /// The resource type TypeId
    pub resource_type: TypeId,
}

/// Set resource operation for an operation.
/// Ported from TS setResourceOperation().
pub fn set_resource_operation(
    state: &mut StateAccessors,
    target: TypeId,
    kind: ResourceOperationKind,
    resource_type: TypeId,
) {
    let value = format!("{}::{}", kind.decorator_name(), resource_type);
    state.set_state(STATE_RESOURCE_OPERATION, target, value);
}

/// Get resource operation for an operation.
/// Ported from TS getResourceOperation().
pub fn get_resource_operation(state: &StateAccessors, target: TypeId) -> Option<ResourceOperation> {
    state
        .get_state(STATE_RESOURCE_OPERATION, target)
        .and_then(|s| {
            let parts: Vec<&str> = s.splitn(2, "::").collect();
            if parts.len() != 2 {
                return None;
            }
            let kind = ResourceOperationKind::from_decorator_name(parts[0])?;
            let resource_type = parts[1].parse::<TypeId>().ok()?;
            Some(ResourceOperation {
                kind,
                resource_type,
            })
        })
}

impl ResourceOperationKind {
    /// Parse from decorator name string
    pub fn from_decorator_name(name: &str) -> Option<Self> {
        match name {
            "readsResource" => Some(ResourceOperationKind::Reads),
            "createsResource" => Some(ResourceOperationKind::Creates),
            "createsOrReplacesResource" => Some(ResourceOperationKind::CreatesOrReplaces),
            "createsOrUpdatesResource" => Some(ResourceOperationKind::CreatesOrUpdates),
            "updatesResource" => Some(ResourceOperationKind::Updates),
            "deletesResource" => Some(ResourceOperationKind::Deletes),
            "listsResource" => Some(ResourceOperationKind::Lists),
            _ => None,
        }
    }
}

macro_rules! define_resource_op {
    ($name:ident, $kind:expr) => {
        pub fn $name(state: &mut StateAccessors, target: TypeId, resource_type: TypeId) {
            set_resource_operation(state, target, $kind, resource_type);
        }
    };
}

define_resource_op!(apply_reads_resource, ResourceOperationKind::Reads);
define_resource_op!(apply_creates_resource, ResourceOperationKind::Creates);
define_resource_op!(
    apply_creates_or_replaces_resource,
    ResourceOperationKind::CreatesOrReplaces
);
define_resource_op!(
    apply_creates_or_updates_resource,
    ResourceOperationKind::CreatesOrUpdates
);
define_resource_op!(apply_updates_resource, ResourceOperationKind::Updates);
define_resource_op!(apply_deletes_resource, ResourceOperationKind::Deletes);
define_resource_op!(apply_lists_resource, ResourceOperationKind::Lists);

/// Check if an operation is a list operation.
/// Ported from TS isListOperation().
pub fn is_list_operation(state: &StateAccessors, target: TypeId) -> bool {
    get_resource_operation(state, target)
        .map(|op| op.kind == ResourceOperationKind::Lists)
        .unwrap_or(false)
}

// ============================================================================
// @actionSegment decorator (private)
// ============================================================================

/// Apply @actionSegment decorator (private).
/// Ported from TS Private.$actionSegment().
pub fn apply_action_segment(state: &mut StateAccessors, target: TypeId, value: &str) {
    state.set_state(STATE_ACTION_SEGMENT, target, value.to_string());
}

/// Get @actionSegment value.
/// Ported from TS getActionSegment().
pub fn get_action_segment(state: &StateAccessors, target: TypeId) -> Option<String> {
    state
        .get_state(STATE_ACTION_SEGMENT, target)
        .map(|s| s.to_string())
}

// ============================================================================
// @resourceLocation decorator (private)
// ============================================================================

typeid_decorator!(
    apply_resource_location,
    get_resource_location_type,
    STATE_RESOURCE_LOCATION
);

// ============================================================================
// @resourceTypeForKeyParam decorator (private)
// ============================================================================

typeid_decorator!(
    apply_resource_type_for_key_param,
    get_resource_type_for_key_param,
    STATE_RESOURCE_TYPE_FOR_KEY_PARAM
);

// ============================================================================
// @copyResourceKeyParameters decorator
// ============================================================================

/// Apply @copyResourceKeyParameters decorator.
/// Ported from TS $copyResourceKeyParameters().
/// Filter is stored as optional string.
pub fn apply_copy_resource_key_parameters(
    state: &mut StateAccessors,
    target: TypeId,
    filter: Option<&str>,
) {
    state.set_state(
        STATE_COPY_RESOURCE_KEY_PARAMS,
        target,
        filter.unwrap_or("").to_string(),
    );
}

/// Get the filter for @copyResourceKeyParameters.
pub fn get_copy_resource_key_parameters_filter(
    state: &StateAccessors,
    target: TypeId,
) -> Option<String> {
    state
        .get_state(STATE_COPY_RESOURCE_KEY_PARAMS, target)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
}

// ============================================================================
// Resource type key mapping
// Ported from TS setResourceTypeKey/getResourceTypeKey
// ============================================================================

/// Set the resource type key for a model.
/// Ported from TS setResourceTypeKey().
pub fn set_resource_type_key(state: &mut StateAccessors, target: TypeId, key: &ResourceKey) {
    let value = format!("{}::{}", key.resource_type, key.key_property);
    state.set_state(STATE_RESOURCE_TYPE_KEY, target, value);
}

/// Get the resource type key for a model.
/// Ported from TS getResourceTypeKey().
pub fn get_resource_type_key(state: &StateAccessors, target: TypeId) -> Option<ResourceKey> {
    state
        .get_state(STATE_RESOURCE_TYPE_KEY, target)
        .and_then(|s| {
            let parts: Vec<&str> = s.split("::").collect();
            if parts.len() == 2 {
                let resource_type = parts[0].parse::<TypeId>().ok()?;
                let key_property = parts[1].parse::<TypeId>().ok()?;
                Some(ResourceKey {
                    resource_type,
                    key_property,
                })
            } else {
                None
            }
        })
}

// ============================================================================
// Library creation
// ============================================================================

/// Create the @typespec/rest library diagnostic map.
/// Ported from TS $lib definition in lib.ts.
pub fn create_rest_library() -> DiagnosticMap {
    HashMap::from([
        (
            DIAG_NOT_KEY_TYPE.to_string(),
            DiagnosticDefinition::error(
                "Cannot copy keys from a non-key type (KeysOf<T> or ParentKeysOf<T>)",
            ),
        ),
        (
            DIAG_RESOURCE_MISSING_KEY.to_string(),
            DiagnosticDefinition::error(
                "Type '{modelName}' is used as a resource and therefore must have a key. Use @key to designate a property as the key.",
            ),
        ),
        (
            DIAG_RESOURCE_MISSING_ERROR.to_string(),
            DiagnosticDefinition::error(
                "Type '{modelName}' is used as an error and therefore must have the @error decorator applied.",
            ),
        ),
        (
            DIAG_DUPLICATE_KEY.to_string(),
            DiagnosticDefinition::error("More than one key found on model type {resourceName}"),
        ),
        (
            DIAG_DUPLICATE_PARENT_KEY.to_string(),
            DiagnosticDefinition::error(
                "Resource type '{resourceName}' has a key property named '{keyName}' which conflicts with the key name of a parent or child resource.",
            ),
        ),
        (
            DIAG_INVALID_ACTION_NAME.to_string(),
            DiagnosticDefinition::error("Action name cannot be empty string."),
        ),
        (
            DIAG_SHARED_ROUTE_UNSPECIFIED_ACTION_NAME.to_string(),
            DiagnosticDefinition::error(
                "An operation marked as '@sharedRoute' must have an explicit collection action name passed to '{decoratorName}'.",
            ),
        ),
        (
            DIAG_CIRCULAR_PARENT_RESOURCE.to_string(),
            DiagnosticDefinition::error("Resource has a parent cycle ({cycle})"),
        ),
    ])
}

// ============================================================================
// TSP Sources
// ============================================================================

/// The TypeSpec source for the REST library decorators
pub const REST_DECORATORS_TSP: &str = r#"
namespace TypeSpec.Rest;

using TypeSpec.Reflection;

/**
 * This interface or operation should resolve its route automatically.
 */
extern dec autoRoute(target: Interface | Operation);

/**
 * Defines the preceding path segment for a @path parameter in auto-generated routes.
 *
 * @param name Segment that will be inserted into the operation route before the path parameter's name field.
 */
extern dec segment(target: Model | ModelProperty | Operation, name: valueof string);

/**
 * Returns the URL segment of a given model if it has @segment and @key decorator.
 * @param type Target model
 */
extern dec segmentOf(target: Operation, type: Model);

/**
 * Defines the separator string that is inserted before the action name in auto-generated routes for actions.
 *
 * @param seperator Seperator seperating the action segment from the rest of the url
 */
extern dec actionSeparator(
  target: Operation | Interface | Namespace,
  seperator: valueof "/" | ":" | "/:"
);

/**
 * Mark this model as a resource type with a name.
 *
 * @param collectionName type's collection name
 */
extern dec resource(target: Model, collectionName: valueof string);

/**
 * Mark model as a child of the given parent resource.
 * @param parent Parent model.
 */
extern dec parentResource(target: Model, parent: Model);

/**
 * Specify that this is a Read operation for a given resource.
 * @param resourceType Resource marked with @resource
 */
extern dec readsResource(target: Operation, resourceType: Model);

/**
 * Specify that this is a Create operation for a given resource.
 * @param resourceType Resource marked with @resource
 */
extern dec createsResource(target: Operation, resourceType: Model);

/**
 * Specify that this is a CreateOrReplace operation for a given resource.
 * @param resourceType Resource marked with @resource
 */
extern dec createsOrReplacesResource(target: Operation, resourceType: Model);

/**
 * Specify that this is a CreatesOrUpdate operation for a given resource.
 * @param resourceType Resource marked with @resource
 */
extern dec createsOrUpdatesResource(target: Operation, resourceType: Model);

/**
 * Specify that this is a Update operation for a given resource.
 * @param resourceType Resource marked with @resource
 */
extern dec updatesResource(target: Operation, resourceType: Model);

/**
 * Specify that this is a Delete operation for a given resource.
 * @param resourceType Resource marked with @resource
 */
extern dec deletesResource(target: Operation, resourceType: Model);

/**
 * Specify that this is a List operation for a given resource.
 * @param resourceType Resource marked with @resource
 */
extern dec listsResource(target: Operation, resourceType: Model);

/**
 * Specify this operation is an action. (Scoped to a resource item /pets/{petId}/my-action)
 * @param name Name of the action. If not specified, the name of the operation will be used.
 */
extern dec action(target: Operation, name?: valueof string);

/**
 * Specify this operation is a collection action. (Scoped to a resource, /pets/my-action)
 * @param resourceType Resource marked with @resource
 * @param name Name of the action. If not specified, the name of the operation will be used.
 */
extern dec collectionAction(target: Operation, resourceType: Model, name?: valueof string);

/**
 * Copy the resource key parameters on the model
 * @param filter Filter to exclude certain properties.
 */
extern dec copyResourceKeyParameters(target: Model, filter?: valueof string);

namespace Private {
  extern dec resourceLocation(target: string, resourceType: Model);
  extern dec validateHasKey(target: unknown, value: unknown);
  extern dec validateIsError(target: unknown, value: unknown);
  extern dec actionSegment(target: Operation, value: valueof string);
  extern dec resourceTypeForKeyParam(entity: ModelProperty, resourceType: Model);
}
"#;

/// The TypeSpec source for the REST library types
pub const REST_TSP: &str = r#"
import "@typespec/http";
import "./rest-decorators.tsp";
import "./resource.tsp";
import "../dist/src/tsp-index.js";

namespace TypeSpec.Rest;

/**
 * A URL that points to a resource.
 * @template Resource The type of resource that the URL points to.
 */
@doc("The location of an instance of {name}", Resource)
@Private.resourceLocation(Resource)
scalar ResourceLocation<Resource extends {}> extends url;
"#;

/// The TypeSpec source for the REST library resource templates
/// Ported from TS packages/rest/lib/resource.tsp
pub const RESOURCE_TSP: &str = r#"
namespace TypeSpec.Rest.Resource;

using Http;

@doc("The default error response for resource operations.")
model ResourceError {
  code: int32;
  message: string;
}

@doc("Dynamically gathers keys of the model type Resource.")
@copyResourceKeyParameters
@friendlyName("{name}Key", Resource)
model KeysOf<Resource> {}

@doc("Dynamically gathers parent keys of the model type Resource.")
@copyResourceKeyParameters("parent")
@friendlyName("{name}ParentKey", Resource)
model ParentKeysOf<Resource> {}

@doc("Represents operation parameters for resource Resource.")
model ResourceParameters<Resource extends {}> {
  ...KeysOf<Resource>;
}

@doc("Represents collection operation parameters for resource Resource.")
model ResourceCollectionParameters<Resource extends {}> {
  ...ParentKeysOf<Resource>;
}

@doc("Resource create operation completed successfully.")
model ResourceCreatedResponse<Resource> {
  ...CreatedResponse;
  @bodyRoot body: Resource;
}

@friendlyName("{name}Update", Resource)
model ResourceCreateOrUpdateModel<Resource extends {}>
  is OptionalProperties<UpdateableProperties<DefaultKeyVisibility<Resource, Lifecycle.Read>>>;

@friendlyName("{name}Create", Resource)
@withVisibility(Lifecycle.Create)
model ResourceCreateModel<Resource extends {}> is DefaultKeyVisibility<Resource, Lifecycle.Read>;

@doc("Resource deleted successfully.")
model ResourceDeletedResponse {
  @statusCode
  _: 200;
}

@doc("Paged response of {name} items", Resource)
@friendlyName("{name}CollectionWithNextLink", Resource)
model CollectionWithNextLink<Resource extends {}> {
  @pageItems
  value: Resource[];
  @nextLink
  nextLink?: ResourceLocation<Resource>;
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rest_namespace() {
        assert_eq!(REST_NAMESPACE, "TypeSpec.Rest");
    }

    #[test]
    fn test_create_rest_library() {
        let diags = create_rest_library();
        assert_eq!(diags.len(), 8);
        let codes: Vec<&str> = diags.keys().map(|code| code.as_str()).collect();
        assert!(codes.contains(&DIAG_RESOURCE_MISSING_KEY));
        assert!(codes.contains(&DIAG_CIRCULAR_PARENT_RESOURCE));
        assert!(codes.contains(&DIAG_DUPLICATE_KEY));
    }

    #[test]
    fn test_resource_operation_kinds() {
        assert_eq!(
            ResourceOperationKind::Reads.decorator_name(),
            "readsResource"
        );
        assert_eq!(
            ResourceOperationKind::Creates.decorator_name(),
            "createsResource"
        );
        assert_eq!(
            ResourceOperationKind::Deletes.decorator_name(),
            "deletesResource"
        );
        assert_eq!(
            ResourceOperationKind::Lists.decorator_name(),
            "listsResource"
        );
    }

    #[test]
    fn test_action_separator() {
        assert_eq!(ActionSeparator::Slash.as_str(), "/");
        assert_eq!(ActionSeparator::Colon.as_str(), ":");
        assert_eq!(ActionSeparator::SlashColon.as_str(), "/:");

        assert_eq!(
            ActionSeparator::parse_str("/"),
            Some(ActionSeparator::Slash)
        );
        assert_eq!(
            ActionSeparator::parse_str(":"),
            Some(ActionSeparator::Colon)
        );
        assert_eq!(
            ActionSeparator::parse_str("/:"),
            Some(ActionSeparator::SlashColon)
        );
        assert_eq!(ActionSeparator::parse_str("x"), None);
    }

    #[test]
    fn test_decorators_tsp_not_empty() {
        assert!(!REST_DECORATORS_TSP.is_empty());
        assert!(REST_DECORATORS_TSP.contains("autoRoute"));
        assert!(REST_DECORATORS_TSP.contains("segment"));
        assert!(REST_DECORATORS_TSP.contains("resource"));
        assert!(REST_DECORATORS_TSP.contains("readsResource"));
        assert!(REST_DECORATORS_TSP.contains("action"));
        assert!(REST_DECORATORS_TSP.contains("collectionAction"));
    }

    #[test]
    fn test_rest_tsp_not_empty() {
        assert!(!REST_TSP.is_empty());
        assert!(REST_TSP.contains("ResourceLocation"));
    }

    #[test]
    fn test_resource_tsp_not_empty() {
        assert!(!RESOURCE_TSP.is_empty());
        assert!(RESOURCE_TSP.contains("ResourceError"));
        assert!(RESOURCE_TSP.contains("KeysOf"));
        assert!(RESOURCE_TSP.contains("ResourceParameters"));
        assert!(RESOURCE_TSP.contains("ResourceCreateModel"));
        assert!(RESOURCE_TSP.contains("CollectionWithNextLink"));
    }

    #[test]
    fn test_is_auto_route() {
        let mut state = StateAccessors::new();
        assert!(!is_auto_route(&state, 1));
        apply_auto_route(&mut state, 1);
        assert!(is_auto_route(&state, 1));
        assert!(!is_auto_route(&state, 2));
    }

    #[test]
    fn test_get_segment() {
        let mut state = StateAccessors::new();
        assert_eq!(get_segment(&state, 1), None);
        apply_segment(&mut state, 1, "pets");
        assert_eq!(get_segment(&state, 1), Some("pets".to_string()));
    }

    #[test]
    fn test_is_resource() {
        let mut state = StateAccessors::new();
        assert!(!is_resource(&state, 1));
        apply_resource(&mut state, 1, "pets");
        assert!(is_resource(&state, 1));
        assert_eq!(
            get_resource_collection_name(&state, 1),
            Some("pets".to_string())
        );
    }

    #[test]
    fn test_parent_resource() {
        let mut state = StateAccessors::new();
        assert_eq!(get_parent_resource(&state, 1), None);
        apply_parent_resource(&mut state, 1, 5);
        assert_eq!(get_parent_resource(&state, 1), Some(5));
    }

    #[test]
    fn test_is_key() {
        let mut state = StateAccessors::new();
        assert!(!is_key(&state, 1));
        apply_key(&mut state, 1);
        assert!(is_key(&state, 1));
    }

    #[test]
    fn test_apply_action() {
        let mut state = StateAccessors::new();
        apply_action(&mut state, 1, None);
        let details = get_action_details(&state, 1);
        assert!(details.is_some());
        let details = details.unwrap();
        assert_eq!(details.kind, ActionNameKind::Automatic);
        assert!(details.resource_type.is_none());

        apply_action(&mut state, 2, Some("customAction"));
        let details = get_action_details(&state, 2);
        assert!(details.is_some());
        assert_eq!(details.unwrap().kind, ActionNameKind::Explicit);
    }

    #[test]
    fn test_apply_collection_action() {
        let mut state = StateAccessors::new();
        apply_collection_action(&mut state, 1, 10, Some("list"));
        let details = get_action_details(&state, 1);
        assert!(details.is_some());
        let details = details.unwrap();
        assert_eq!(details.kind, ActionNameKind::Explicit);
        assert_eq!(details.resource_type, Some(10));
    }

    #[test]
    fn test_segment_of() {
        let mut state = StateAccessors::new();
        assert_eq!(get_segment_of(&state, 1), None);
        apply_segment_of(&mut state, 1, 42);
        assert_eq!(get_segment_of(&state, 1), Some(42));
    }

    #[test]
    fn test_action_separator_decorator() {
        let mut state = StateAccessors::new();
        assert_eq!(get_action_separator(&state, 1), None);
        apply_action_separator(&mut state, 1, ActionSeparator::Slash);
        assert_eq!(
            get_action_separator(&state, 1),
            Some(ActionSeparator::Slash)
        );
        apply_action_separator(&mut state, 2, ActionSeparator::Colon);
        assert_eq!(
            get_action_separator(&state, 2),
            Some(ActionSeparator::Colon)
        );
    }

    #[test]
    fn test_resource_operation() {
        let mut state = StateAccessors::new();
        assert_eq!(get_resource_operation(&state, 1), None);
        apply_reads_resource(&mut state, 1, 10);
        let op = get_resource_operation(&state, 1).unwrap();
        assert_eq!(op.kind, ResourceOperationKind::Reads);
        assert_eq!(op.resource_type, 10);
    }

    #[test]
    fn test_creates_resource() {
        let mut state = StateAccessors::new();
        apply_creates_resource(&mut state, 1, 20);
        let op = get_resource_operation(&state, 1).unwrap();
        assert_eq!(op.kind, ResourceOperationKind::Creates);
        assert_eq!(op.resource_type, 20);
    }

    #[test]
    fn test_deletes_resource() {
        let mut state = StateAccessors::new();
        apply_deletes_resource(&mut state, 1, 30);
        let op = get_resource_operation(&state, 1).unwrap();
        assert_eq!(op.kind, ResourceOperationKind::Deletes);
    }

    #[test]
    fn test_lists_resource_is_list() {
        let mut state = StateAccessors::new();
        assert!(!is_list_operation(&state, 1));
        apply_lists_resource(&mut state, 1, 10);
        assert!(is_list_operation(&state, 1));
    }

    #[test]
    fn test_action_segment() {
        let mut state = StateAccessors::new();
        assert_eq!(get_action_segment(&state, 1), None);
        apply_action_segment(&mut state, 1, "customAction");
        assert_eq!(
            get_action_segment(&state, 1),
            Some("customAction".to_string())
        );
    }

    #[test]
    fn test_resource_location() {
        let mut state = StateAccessors::new();
        assert_eq!(get_resource_location_type(&state, 1), None);
        apply_resource_location(&mut state, 1, 50);
        assert_eq!(get_resource_location_type(&state, 1), Some(50));
    }

    #[test]
    fn test_resource_type_for_key_param() {
        let mut state = StateAccessors::new();
        assert_eq!(get_resource_type_for_key_param(&state, 1), None);
        apply_resource_type_for_key_param(&mut state, 1, 99);
        assert_eq!(get_resource_type_for_key_param(&state, 1), Some(99));
    }

    #[test]
    fn test_copy_resource_key_parameters() {
        let mut state = StateAccessors::new();
        apply_copy_resource_key_parameters(&mut state, 1, None);
        assert_eq!(get_copy_resource_key_parameters_filter(&state, 1), None);
        apply_copy_resource_key_parameters(&mut state, 2, Some("excludeId"));
        assert_eq!(
            get_copy_resource_key_parameters_filter(&state, 2),
            Some("excludeId".to_string())
        );
    }

    #[test]
    fn test_resource_type_key() {
        let mut state = StateAccessors::new();
        assert_eq!(get_resource_type_key(&state, 1), None);
        set_resource_type_key(
            &mut state,
            1,
            &ResourceKey {
                resource_type: 5,
                key_property: 10,
            },
        );
        let key = get_resource_type_key(&state, 1).unwrap();
        assert_eq!(key.resource_type, 5);
        assert_eq!(key.key_property, 10);
    }

    #[test]
    fn test_resource_operation_kind_from_decorator_name() {
        assert_eq!(
            ResourceOperationKind::from_decorator_name("readsResource"),
            Some(ResourceOperationKind::Reads)
        );
        assert_eq!(
            ResourceOperationKind::from_decorator_name("createsResource"),
            Some(ResourceOperationKind::Creates)
        );
        assert_eq!(ResourceOperationKind::from_decorator_name("unknown"), None);
    }
}
