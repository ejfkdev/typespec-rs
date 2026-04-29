//! HTTP Visibility types and helpers
//!
//! Ported from TypeSpec @typespec/http visibility.ts

use super::HttpVerb;
use super::{
    is_body, is_body_root, is_cookie, is_header, is_multipart_body, is_path, is_query,
    is_status_code,
};
use crate::checker::types::TypeId;
use crate::state_accessors::StateAccessors;

// ============================================================================
// HTTP Visibility
// Ported from http/src/metadata.ts
// ============================================================================

/// HTTP visibility flags for properties.
///
/// In the TS HTTP library, visibility is a flags enum that controls which
/// properties are included in requests vs responses. Each flag represents
/// a lifecycle stage or request context.
///
/// Ported from TS `enum Visibility { Read, Create, Update, Delete, Query,
///   None, All, Item, Patch, Synthetic }`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Visibility(u32);

#[allow(non_upper_case_globals)]
impl Visibility {
    /// Property is visible in read/response context
    pub const Read: Visibility = Visibility(1 << 0);
    /// Property is visible in create request context
    pub const Create: Visibility = Visibility(1 << 1);
    /// Property is visible in update request context
    pub const Update: Visibility = Visibility(1 << 2);
    /// Property is visible in delete request context
    pub const Delete: Visibility = Visibility(1 << 3);
    /// Property is visible in query context (GET/HEAD)
    pub const Query: Visibility = Visibility(1 << 4);

    /// No visibility flags set
    pub const None: Visibility = Visibility(0);
    /// All standard visibility flags
    pub const All: Visibility = Visibility((1 << 0) | (1 << 1) | (1 << 2) | (1 << 3) | (1 << 4));

    /// Flag indicating the property is nested in a collection
    /// and therefore no metadata is applicable.
    pub const Item: Visibility = Visibility(1 << 20);

    /// Flag indicating the verb is PATCH and fields should be made optional
    /// if the request visibility includes update.
    /// Whether this flag is set is determined by @patch options.
    pub const Patch: Visibility = Visibility(1 << 21);

    /// Synthetic flags that are not real visibilities
    pub const Synthetic: Visibility = Visibility((1 << 20) | (1 << 21));

    /// Get the raw bits
    pub fn bits(&self) -> u32 {
        self.0
    }

    /// Check if a specific flag is set
    pub fn contains(&self, other: Visibility) -> bool {
        (self.0 & other.0) != 0
    }

    /// Combine two visibility flags
    pub fn union(&self, other: Visibility) -> Visibility {
        Visibility(self.0 | other.0)
    }

    /// Remove a flag
    pub fn difference(&self, other: Visibility) -> Visibility {
        Visibility(self.0 & !other.0)
    }

    /// Check if no flags are set
    pub fn is_none(&self) -> bool {
        self.0 == 0
    }

    /// Convert visibility to an array of visibility names.
    /// Synthetic flags (Item, Patch) are excluded.
    ///
    /// Ported from TS `visibilityToArray()`.
    pub fn to_vec(&self) -> Vec<&'static str> {
        // Synthetic flags are not real visibilities
        let v = self.difference(Visibility::Synthetic);
        let mut result = Vec::new();
        if v.contains(Visibility::Read) {
            result.push("read");
        }
        if v.contains(Visibility::Create) {
            result.push("create");
        }
        if v.contains(Visibility::Update) {
            result.push("update");
        }
        if v.contains(Visibility::Delete) {
            result.push("delete");
        }
        if v.contains(Visibility::Query) {
            result.push("query");
        }
        result
    }
}

impl std::ops::BitOr for Visibility {
    type Output = Visibility;
    fn bitor(self, rhs: Visibility) -> Visibility {
        Visibility(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for Visibility {
    fn bitor_assign(&mut self, rhs: Visibility) {
        self.0 |= rhs.0;
    }
}

impl std::ops::BitAnd for Visibility {
    type Output = Visibility;
    fn bitand(self, rhs: Visibility) -> Visibility {
        Visibility(self.0 & rhs.0)
    }
}

impl std::ops::Not for Visibility {
    type Output = Visibility;
    fn not(self) -> Visibility {
        Visibility(!self.0)
    }
}

/// Get the visibility suffix for naming models after visibility filtering.
///
/// The canonical visibility (default Visibility::All) gets empty suffix,
/// otherwise visibilities are joined in pascal-case with `Or`. And `Item` is
/// appended if `Visibility::Item` is set.
///
/// Examples (with canonicalVisibility = Visibility::Read):
/// - Visibility::Read => ""
/// - Visibility::Update => "Update"
/// - Visibility::Create | Visibility::Update => "CreateOrUpdate"
/// - Visibility::Create | Visibility::Item => "CreateItem"
///
/// Ported from TS `getVisibilitySuffix()`.
pub fn get_visibility_suffix(
    visibility: Visibility,
    canonical_visibility: Option<Visibility>,
) -> String {
    let canonical = canonical_visibility.unwrap_or(Visibility::All);
    let mut suffix = String::new();

    let non_synthetic = visibility.difference(Visibility::Synthetic);
    if non_synthetic != canonical {
        let visibilities = non_synthetic.to_vec();
        suffix = visibilities
            .iter()
            .map(|v| {
                let mut chars = v.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().chain(chars).collect(),
                }
            })
            .collect::<Vec<_>>()
            .join("Or");
    }

    if visibility.contains(Visibility::Item) {
        suffix.push_str("Item");
    }

    suffix
}

/// Determines the default visibility to use for a request with the given verb.
///
/// - GET | HEAD => Visibility::Query
/// - POST => Visibility::Create
/// - PUT => Visibility::Create | Visibility::Update
/// - PATCH => Visibility::Update
/// - DELETE => Visibility::Delete
///
/// Ported from TS `getDefaultVisibilityForVerb()`.
pub fn get_default_visibility_for_verb(verb: HttpVerb) -> Visibility {
    match verb {
        HttpVerb::Get | HttpVerb::Head => Visibility::Query,
        HttpVerb::Post => Visibility::Create,
        HttpVerb::Put => Visibility::Create.union(Visibility::Update),
        HttpVerb::Patch => Visibility::Update,
        HttpVerb::Delete => Visibility::Delete,
    }
}

// ============================================================================
// HTTP Metadata helpers
// Ported from http/src/metadata.ts
// ============================================================================

/// Determines if a property is metadata.
/// A property is metadata if it is marked @header, @cookie, @query, @path, or @statusCode.
///
/// Ported from TS `isMetadata()`.
pub fn is_metadata(state: &StateAccessors, target: TypeId) -> bool {
    is_header(state, target)
        || is_cookie(state, target)
        || is_query(state, target)
        || is_path(state, target)
        || is_status_code(state, target)
}

/// Determines if the given property is metadata that is applicable with the
/// given visibility.
///
/// - No metadata is applicable with Visibility::Item present.
/// - If only Visibility::Read is present, then only @header and @statusCode
///   properties are applicable.
/// - If Visibility::Read is not present, all metadata properties other than
///   @statusCode are applicable.
///
/// Ported from TS `isApplicableMetadata()`.
pub fn is_applicable_metadata_with_visibility(
    state: &StateAccessors,
    target: TypeId,
    visibility: Visibility,
) -> bool {
    is_applicable_metadata_core(state, target, visibility, false)
}

/// Determines if the given property is metadata or marked @body and
/// applicable with the given visibility.
///
/// Ported from TS `isApplicableMetadataOrBody()`.
pub fn is_applicable_metadata_or_body_with_visibility(
    state: &StateAccessors,
    target: TypeId,
    visibility: Visibility,
) -> bool {
    is_applicable_metadata_core(state, target, visibility, true)
}

fn is_applicable_metadata_core(
    state: &StateAccessors,
    target: TypeId,
    visibility: Visibility,
    treat_body_as_metadata: bool,
) -> bool {
    // No metadata is applicable to collection items
    if visibility.contains(Visibility::Item) {
        return false;
    }

    // Body/bodyRoot/multipartBody are always applicable if treating body as metadata
    if treat_body_as_metadata
        && (is_body(state, target)
            || is_body_root(state, target)
            || is_multipart_body(state, target))
    {
        return true;
    }

    // Must be metadata to be applicable metadata
    if !is_metadata(state, target) {
        return false;
    }

    // If read visibility, only header and statusCode are applicable
    if visibility.contains(Visibility::Read) {
        return is_header(state, target) || is_status_code(state, target);
    }

    // Non-read visibility: all metadata except statusCode is applicable
    !is_status_code(state, target)
}

/// Check if a property is HTTP metadata (any of @header, @query, @path, @statusCode, @cookie).
/// This is the same as `is_metadata()`.
///
/// For visibility-aware checking, use `is_applicable_metadata_with_visibility()`.
pub fn is_applicable_metadata(state: &StateAccessors, target: TypeId) -> bool {
    is_metadata(state, target)
}

/// Check if a property is either applicable metadata or a body parameter.
/// Simplified version: uses Visibility::All (no filtering).
///
/// For visibility-aware checking, use `is_applicable_metadata_or_body_with_visibility()`.
pub fn is_applicable_metadata_or_body(state: &StateAccessors, target: TypeId) -> bool {
    is_applicable_metadata(state, target) || is_body(state, target) || is_body_root(state, target)
}
