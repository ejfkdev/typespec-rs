//! @typespec/versioning - API Versioning Decorators and Types
//!
//! Ported from TypeSpec packages/versioning
//!
//! Provides decorators for API versioning:
//! - `@versioned` - Mark a namespace as versioned with an enum of versions
//! - `@added` - Mark a type as added in a specific version
//! - `@removed` - Mark a type as removed in a specific version
//! - `@renamedFrom` - Mark a type as renamed from an old name
//! - `@madeOptional` - Mark a property as made optional in a version
//! - `@madeRequired` - Mark a property as made required in a version
//! - `@typeChangedFrom` - Mark a property as having a changed type
//! - `@returnTypeChangedFrom` - Mark an operation as having a changed return type
//! - `@useDependency` - Declare dependency on another versioned namespace
//!
//! ## Note
//! The full TS implementation depends heavily on Program and Type systems.
//! This port provides the type definitions, diagnostics, and state-based
//! decorator implementations. Validation logic that requires walking the
//! type graph will be added when the checker infrastructure is complete.

use crate::checker::types::TypeId;
use crate::diagnostics::{DiagnosticDefinition, DiagnosticMap};
use crate::state_accessors::StateAccessors;
use std::collections::{HashMap, HashSet};

// ============================================================================
// Namespace and state keys
// ============================================================================

/// Namespace for versioning types
pub const VERSIONING_NAMESPACE: &str = "TypeSpec.Versioning";

/// State key for @addedOn decorator
pub const STATE_ADDED_ON: &str = "TypeSpec.Versioning.addedOn";
/// State key for @removedOn decorator
pub const STATE_REMOVED_ON: &str = "TypeSpec.Versioning.removedOn";
/// State key for @versioned decorator
pub const STATE_VERSIONS: &str = "TypeSpec.Versioning.versions";
/// State key for @useDependency on Namespaces
pub const STATE_USE_DEPENDENCY_NAMESPACE: &str = "TypeSpec.Versioning.useDependencyNamespace";
/// State key for @useDependency on Enums
pub const STATE_USE_DEPENDENCY_ENUM: &str = "TypeSpec.Versioning.useDependencyEnum";
/// State key for @renamedFrom decorator
pub const STATE_RENAMED_FROM: &str = "TypeSpec.Versioning.renamedFrom";
/// State key for @madeOptional decorator
pub const STATE_MADE_OPTIONAL: &str = "TypeSpec.Versioning.madeOptional";
/// State key for @madeRequired decorator
pub const STATE_MADE_REQUIRED: &str = "TypeSpec.Versioning.madeRequired";
/// State key for @typeChangedFrom decorator
pub const STATE_TYPE_CHANGED_FROM: &str = "TypeSpec.Versioning.typeChangedFrom";
/// State key for @returnTypeChangedFrom decorator
pub const STATE_RETURN_TYPE_CHANGED_FROM: &str = "TypeSpec.Versioning.returnTypeChangedFrom";

// ============================================================================
// Types
// ============================================================================

/// Represents a version of an API.
/// Ported from TS Version interface.
#[derive(Debug, Clone)]
pub struct Version {
    /// Version name (enum member name)
    pub name: String,
    /// Version value (enum member value or name if no value)
    pub value: String,
    /// Index in the version enum (for ordering)
    pub index: usize,
}

/// A renamed-from record linking a version to the old name.
/// Ported from TS RenamedFrom interface.
#[derive(Debug, Clone)]
pub struct RenamedFromRecord {
    /// The version when the rename happened
    pub version: Version,
    /// The old name
    pub old_name: String,
}

/// Version resolution result.
/// Ported from TS VersionResolution interface.
#[derive(Debug, Clone)]
pub struct VersionResolution {
    /// Version for the root namespace (None if not versioned)
    pub root_version: Option<Version>,
    /// Resolved versions for all referenced namespaces
    pub versions: Vec<(TypeId, Version)>,
}

string_enum! {
    /// Availability of a type in a specific version.
    /// Ported from TS Availability enum.
    pub enum Availability {
        /// Type is not available in this version
        Unavailable => "Unavailable",
        /// Type was added in this version
        Added => "Added",
        /// Type is available in this version
        Available => "Available",
        /// Type was removed in this version
        Removed => "Removed",
    }
}

/// A map of enum member indices to Version structs.
/// Ported from TS VersionMap class.
#[derive(Debug, Clone)]
pub struct VersionMap {
    /// The namespace TypeId this version map belongs to
    pub namespace: TypeId,
    /// Ordered list of versions
    versions: Vec<Version>,
}

impl VersionMap {
    /// Create a new VersionMap from a list of version names.
    /// Values default to the name if not provided.
    pub fn new(namespace: TypeId, version_names: &[(&str, Option<&str>)]) -> Self {
        let versions: Vec<Version> = version_names
            .iter()
            .enumerate()
            .map(|(index, (name, value))| Version {
                name: name.to_string(),
                value: value.unwrap_or(name).to_string(),
                index,
            })
            .collect();
        Self {
            namespace,
            versions,
        }
    }

    /// Get the number of versions
    pub fn len(&self) -> usize {
        self.versions.len()
    }

    /// Check if the map is empty
    pub fn is_empty(&self) -> bool {
        self.versions.is_empty()
    }

    /// Get all versions
    pub fn get_versions(&self) -> &[Version] {
        &self.versions
    }

    /// Get a version by its index
    pub fn get_by_index(&self, index: usize) -> Option<&Version> {
        self.versions.get(index)
    }

    /// Find a version by name
    pub fn get_by_name(&self, name: &str) -> Option<&Version> {
        self.versions.iter().find(|v| v.name == name)
    }
}

/// State key for @useDependency decorator
pub const STATE_USE_DEPENDENCY: &str = "TypeSpec.Versioning.useDependency";
/// State key for version index
pub const STATE_VERSION_INDEX: &str = "TypeSpec.Versioning.versionIndex";

/// Apply @useDependency decorator.
/// Ported from TS $useDependency().
pub fn apply_use_dependency(state: &mut StateAccessors, target: TypeId, version_refs: &[TypeId]) {
    let value: String = version_refs
        .iter()
        .map(|id| id.to_string())
        .collect::<Vec<_>>()
        .join(",");
    state.set_state(STATE_USE_DEPENDENCY, target, value);
}

/// Get @useDependency version references.
/// Ported from TS getUseDependency().
pub fn get_use_dependency(state: &StateAccessors, target: TypeId) -> Vec<TypeId> {
    state
        .get_state(STATE_USE_DEPENDENCY, target)
        .map(|s| s.split(',').filter_map(|v| v.parse().ok()).collect())
        .unwrap_or_default()
}

// ============================================================================
// Diagnostic definitions
// ============================================================================

/// Create the versioning library diagnostic map.
/// Ported from TS $lib diagnostics.
pub fn create_versioning_library() -> DiagnosticMap {
    HashMap::from([
        (
            "versioned-dependency-tuple".to_string(),
            DiagnosticDefinition::error(
                "Versioned dependency mapping must be a tuple [SourceVersion, TargetVersion].",
            ),
        ),
        (
            "versioned-dependency-tuple-enum-member".to_string(),
            DiagnosticDefinition::error(
                "Versioned dependency mapping must be between enum members.",
            ),
        ),
        (
            "versioned-dependency-same-namespace".to_string(),
            DiagnosticDefinition::error(
                "Versioned dependency mapping must all point to the same namespace.",
            ),
        ),
        (
            "versioned-dependency-not-picked".to_string(),
            DiagnosticDefinition::error(
                "The versionedDependency decorator must provide a version of the dependency.",
            ),
        ),
        (
            "version-not-found".to_string(),
            DiagnosticDefinition::error("The provided version is not declared as a version enum."),
        ),
        (
            "version-duplicate".to_string(),
            DiagnosticDefinition::error(
                "Multiple versions resolve to the same value. Version enums must resolve to unique values.",
            ),
        ),
        (
            "invalid-renamed-from-value".to_string(),
            DiagnosticDefinition::error("@renamedFrom.oldName cannot be empty string."),
        ),
        // Ported from TS lib.ts - incompatible-versioned-reference with all message variants
        (
            "incompatible-versioned-reference".to_string(),
            DiagnosticDefinition::error_with_messages(vec![
                (
                    "default",
                    "'{sourceName}' is referencing versioned type '{targetName}' but is not versioned itself.",
                ),
                (
                    "addedAfter",
                    "'{sourceName}' was added in version '{sourceAddedOn}' but referencing type '{targetName}' added in version '{targetAddedOn}'.",
                ),
                (
                    "dependentAddedAfter",
                    "'{sourceName}' was added in version '{sourceAddedOn}' but contains type '{targetName}' added in version '{targetAddedOn}'.",
                ),
                (
                    "removedBefore",
                    "'{sourceName}' was removed in version '{sourceRemovedOn}' but referencing type '{targetName}' removed in version '{targetRemovedOn}'.",
                ),
                (
                    "dependentRemovedBefore",
                    "'{sourceName}' was removed in version '{sourceRemovedOn}' but contains type '{targetName}' removed in version '{targetRemovedOn}'.",
                ),
                (
                    "versionedDependencyAddedAfter",
                    "'{sourceName}' is referencing type '{targetName}' added in version '{targetAddedOn}' but version used is '{dependencyVersion}'.",
                ),
                (
                    "versionedDependencyRemovedBefore",
                    "'{sourceName}' is referencing type '{targetName}' removed in version '{targetRemovedOn}' but version used is '{dependencyVersion}'.",
                ),
                (
                    "doesNotExist",
                    "'{sourceName}' is referencing type '{targetName}' which does not exist in version '{version}'.",
                ),
            ]),
        ),
        (
            "incompatible-versioned-namespace-use-dependency".to_string(),
            DiagnosticDefinition::error(
                "The useDependency decorator can only be used on a Namespace if the namespace is unversioned. For versioned namespaces, put the useDependency decorator on the version enum members.",
            ),
        ),
        (
            "made-optional-not-optional".to_string(),
            DiagnosticDefinition::error("Property marked with @madeOptional but is required."),
        ),
        (
            "made-required-optional".to_string(),
            DiagnosticDefinition::error("Property marked with @madeRequired but is optional."),
        ),
        (
            "renamed-duplicate-property".to_string(),
            DiagnosticDefinition::error(
                "Property marked with '@renamedFrom' conflicts with existing property.",
            ),
        ),
    ])
}

// ============================================================================
// Decorator implementations (state-based)
// ============================================================================

/// Apply a version index to a comma-separated state key (used by @added and @removed).
fn apply_version_index(
    state: &mut StateAccessors,
    state_key: &str,
    target: TypeId,
    version_index: usize,
) {
    let existing = state.get_state(state_key, target).unwrap_or("").to_string();
    let new_value = if existing.is_empty() {
        version_index.to_string()
    } else {
        let mut indices: Vec<usize> = existing.split(',').filter_map(|s| s.parse().ok()).collect();
        if !indices.contains(&version_index) {
            indices.push(version_index);
        }
        indices.sort();
        indices
            .iter()
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(",")
    };
    state.set_state(state_key, target, new_value);
}

/// Apply @added decorator.
/// Ported from TS $added().
pub fn apply_added(state: &mut StateAccessors, target: TypeId, version_index: usize) {
    apply_version_index(state, STATE_ADDED_ON, target, version_index);
}

/// Get the version indices when a type was added.
/// Ported from TS getAddedOnVersions().
pub fn get_added_on_versions(state: &StateAccessors, target: TypeId) -> Vec<usize> {
    state
        .get_state(STATE_ADDED_ON, target)
        .map(|s| s.split(',').filter_map(|v| v.parse().ok()).collect())
        .unwrap_or_default()
}

/// Apply @removed decorator.
/// Ported from TS $removed().
pub fn apply_removed(state: &mut StateAccessors, target: TypeId, version_index: usize) {
    apply_version_index(state, STATE_REMOVED_ON, target, version_index);
}

/// Get the version indices when a type was removed.
/// Ported from TS getRemovedOnVersions().
pub fn get_removed_on_versions(state: &StateAccessors, target: TypeId) -> Vec<usize> {
    state
        .get_state(STATE_REMOVED_ON, target)
        .map(|s| s.split(',').filter_map(|v| v.parse().ok()).collect())
        .unwrap_or_default()
}

/// Apply @versioned decorator.
/// Ported from TS $versioned().
pub fn apply_versioned(state: &mut StateAccessors, target: TypeId, versions_enum_name: &str) {
    state.set_state(STATE_VERSIONS, target, versions_enum_name.to_string());
}

/// Check if a namespace is versioned.
pub fn is_versioned(state: &StateAccessors, target: TypeId) -> bool {
    state.get_state(STATE_VERSIONS, target).is_some()
}

/// Get the version enum name for a namespace.
pub fn get_version_enum_name(state: &StateAccessors, target: TypeId) -> Option<String> {
    state
        .get_state(STATE_VERSIONS, target)
        .map(|s| s.to_string())
}

/// Apply an "index::name" entry to a `||`-separated state key.
/// Used by @renamedFrom, @typeChangedFrom, @returnTypeChangedFrom.
fn apply_indexed_entry(
    state: &mut StateAccessors,
    state_key: &str,
    target: TypeId,
    version_index: usize,
    name: &str,
) {
    let existing = state.get_state(state_key, target).unwrap_or("").to_string();
    let entry = format!("{}::{}", version_index, name);
    let new_value = if existing.is_empty() {
        entry
    } else {
        format!("{}||{}", existing, entry)
    };
    state.set_state(state_key, target, new_value);
}

/// Parse a `||`-separated, `index::name`-encoded state key into a Vec<(usize, String)>.
/// Used by get_renamed_from, get_type_changed_from, get_return_type_changed_from.
fn get_indexed_entries(
    state: &StateAccessors,
    state_key: &str,
    target: TypeId,
) -> Vec<(usize, String)> {
    state
        .get_state(state_key, target)
        .map(|s| {
            s.split("||")
                .filter_map(|entry| {
                    let parts: Vec<&str> = entry.splitn(2, "::").collect();
                    if parts.len() == 2 {
                        Some((parts[0].parse().ok()?, parts[1].to_string()))
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Apply @renamedFrom decorator.
/// Ported from TS $renamedFrom().
pub fn apply_renamed_from(
    state: &mut StateAccessors,
    target: TypeId,
    version_index: usize,
    old_name: &str,
) {
    apply_indexed_entry(state, STATE_RENAMED_FROM, target, version_index, old_name);
}

/// Get the renamed-from records for a type.
/// Ported from TS getRenamedFrom().
pub fn get_renamed_from(state: &StateAccessors, target: TypeId) -> Vec<(usize, String)> {
    get_indexed_entries(state, STATE_RENAMED_FROM, target)
}

numeric_decorator!(
    apply_made_optional,
    get_made_optional_on,
    STATE_MADE_OPTIONAL,
    usize
);
numeric_decorator!(
    apply_made_required,
    get_made_required_on,
    STATE_MADE_REQUIRED,
    usize
);

/// Apply @typeChangedFrom decorator.
/// Ported from TS $typeChangedFrom().
pub fn apply_type_changed_from(
    state: &mut StateAccessors,
    target: TypeId,
    version_index: usize,
    old_type_name: &str,
) {
    apply_indexed_entry(
        state,
        STATE_TYPE_CHANGED_FROM,
        target,
        version_index,
        old_type_name,
    );
}

/// Get the type-changed-from records for a property.
/// Ported from TS getTypeChangedFrom().
pub fn get_type_changed_from(state: &StateAccessors, target: TypeId) -> Vec<(usize, String)> {
    get_indexed_entries(state, STATE_TYPE_CHANGED_FROM, target)
}

/// Apply @returnTypeChangedFrom decorator.
/// Ported from TS $returnTypeChangedFrom().
pub fn apply_return_type_changed_from(
    state: &mut StateAccessors,
    target: TypeId,
    version_index: usize,
    old_type_name: &str,
) {
    apply_indexed_entry(
        state,
        STATE_RETURN_TYPE_CHANGED_FROM,
        target,
        version_index,
        old_type_name,
    );
}

/// Get the return-type-changed-from records for an operation.
/// Ported from TS getReturnTypeChangedFrom().
pub fn get_return_type_changed_from(
    state: &StateAccessors,
    target: TypeId,
) -> Vec<(usize, String)> {
    get_indexed_entries(state, STATE_RETURN_TYPE_CHANGED_FROM, target)
}

/// Find the versioned namespace by walking up the namespace hierarchy.
/// Ported from TS findVersionedNamespace().
/// In the Rust port, this requires a parent_resolver function to walk up.
pub fn find_versioned_namespace<F>(
    state: &StateAccessors,
    start: TypeId,
    parent_resolver: F,
) -> Option<TypeId>
where
    F: Fn(TypeId) -> Option<TypeId>,
{
    let mut current = Some(start);
    while let Some(ns) = current {
        if is_versioned(state, ns) {
            return Some(ns);
        }
        current = parent_resolver(ns);
    }
    None
}

/// Get the version indices when a type was renamed.
/// Ported from TS getRenamedFromVersions().
pub fn get_renamed_from_versions(state: &StateAccessors, target: TypeId) -> Vec<usize> {
    get_renamed_from(state, target)
        .iter()
        .map(|(idx, _)| *idx)
        .collect()
}

// ============================================================================
// Versioning mutator type definitions
// Ported from TS packages/versioning/src/mutator.ts
//
// These type definitions represent the output of versioning mutator resolution.
// The actual mutator creation logic depends on Program/Mutator/Realm.
// ============================================================================

/// Discriminator for versioning mutator kinds.
/// Ported from TS VersionedMutators.kind / TransientVersioningMutator.kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersioningMutatorKind {
    /// The service is versioned (has multiple version snapshots)
    Versioned,
    /// The service is not versioned but uses specific versions from another library
    Transient,
}

/// A snapshot of the service at a specific version, with an associated mutator.
/// Ported from TS VersionSnapshotMutation interface.
#[derive(Debug, Clone)]
pub struct VersionSnapshotMutation {
    /// The version this snapshot represents
    pub version: Version,
    /// Human-readable name for the mutator (e.g., "VersionSnapshot v1")
    pub name: String,
}

/// Result when the service is versioned: multiple version snapshots.
/// Ported from TS VersionedMutators interface.
#[derive(Debug, Clone)]
pub struct VersionedMutators {
    /// The list of version snapshots with their mutators
    pub snapshots: Vec<VersionSnapshotMutation>,
}

/// Result when the service itself is not versioned but uses specific versions
/// from another library.
/// Ported from TS TransientVersioningMutator interface.
#[derive(Debug, Clone)]
pub struct TransientVersioningMutator {
    /// Human-readable name for the mutator
    pub name: String,
}

/// The resolved versioning mutator result.
/// Ported from TS getVersioningMutators() return type.
#[derive(Debug, Clone)]
pub enum VersioningMutators {
    /// The service is versioned
    Versioned(VersionedMutators),
    /// The service uses a specific version from another library
    Transient(TransientVersioningMutator),
}

// ============================================================================
// VersioningTimeline and TimelineMoment
// Ported from TS packages/versioning/src/versioning-timeline.ts
// ============================================================================

/// A moment in the versioning timeline.
/// Represents a snapshot of which version each namespace is at.
/// Ported from TS TimelineMoment class.
#[derive(Debug, Clone)]
pub struct TimelineMoment {
    /// Version map: namespace TypeId → Version
    version_map: Vec<(TypeId, Version)>,
    /// Name of the moment (taken from the first version's name)
    pub name: String,
}

impl TimelineMoment {
    /// Create a new TimelineMoment from a version map.
    pub fn new(version_map: Vec<(TypeId, Version)>) -> Self {
        let name = version_map
            .first()
            .map(|(_, v)| v.name.clone())
            .unwrap_or_default();
        Self { version_map, name }
    }

    /// Get the version for a specific namespace in this moment.
    /// Ported from TS TimelineMoment.getVersion().
    pub fn get_version(&self, namespace: TypeId) -> Option<&Version> {
        self.version_map
            .iter()
            .find(|(ns, _)| *ns == namespace)
            .map(|(_, v)| v)
    }

    /// Iterate over all versions in this moment.
    /// Ported from TS TimelineMoment.versions().
    pub fn versions(&self) -> impl Iterator<Item = &Version> {
        self.version_map.iter().map(|(_, v)| v)
    }

    /// Get all namespace-version pairs in this moment.
    pub fn entries(&self) -> &[(TypeId, Version)] {
        &self.version_map
    }
}

/// A timeline of all the versions involved in the versioning of a namespace.
/// Ported from TS VersioningTimeline class.
///
/// Given resolutions (one per root namespace version), this constructs
/// a complete timeline that includes both explicitly referenced versions
/// and intermediate versions that were not explicitly used.
///
/// # Example
/// ```text
/// Library: l1, l2, l3, l4
/// Service: v1 → l1, v2 → l3, v3 → l3
///
/// Timeline:
/// | Service | Library |
/// |---------|---------|
/// |   v1    |   l1    |
/// |         |   l2    |
/// |   v2    |   l3    |
/// |   v3    |   l3    |
/// |         |   l4    |
/// ```
#[derive(Debug, Clone)]
pub struct VersioningTimeline {
    /// The namespaces involved in this timeline
    namespaces: Vec<TypeId>,
    /// Ordered timeline moments
    timeline: Vec<TimelineMoment>,
    /// Index from version (namespace, version.index) to timeline position
    version_index: HashMap<(TypeId, usize), usize>,
}

impl VersioningTimeline {
    /// Create a new VersioningTimeline from a set of version resolutions.
    ///
    /// Each resolution is a map of namespace → version, representing
    /// which version each namespace is at for a particular root version.
    ///
    /// After constructing from the resolutions, intermediate versions
    /// (those not explicitly referenced) are inserted at the correct
    /// positions in the timeline.
    ///
    /// The `all_versions_fn` callback provides all versions for a given namespace,
    /// allowing the timeline to fill in gaps. It should return None if the
    /// namespace has no versions.
    pub fn new<F>(resolutions: &[Vec<(TypeId, Version)>], all_versions_fn: F) -> Self
    where
        F: Fn(TypeId) -> Option<Vec<Version>>,
    {
        let mut indexed_versions: HashSet<(TypeId, usize)> = HashSet::new();
        let mut namespaces_set: HashSet<TypeId> = HashSet::new();

        let mut timeline: Vec<TimelineMoment> = resolutions
            .iter()
            .map(|resolution| TimelineMoment::new(resolution.clone()))
            .collect();

        for resolution in resolutions {
            for &(ns, ref ver) in resolution {
                indexed_versions.insert((ns, ver.index));
                namespaces_set.insert(ns);
            }
        }

        let namespaces: Vec<TypeId> = namespaces_set.into_iter().collect();

        // Insert intermediate versions that were not explicitly referenced
        for &ns in &namespaces {
            if let Some(versions) = all_versions_fn(ns) {
                for ver in versions {
                    if !indexed_versions.contains(&(ns, ver.index)) {
                        indexed_versions.insert((ns, ver.index));

                        // Find the correct position to insert
                        let insert_index = Self::find_index_to_insert(&timeline, ns, &ver);
                        let new_moment = TimelineMoment::new(vec![(ns, ver)]);

                        if let Some(idx) = insert_index {
                            timeline.insert(idx, new_moment);
                        } else {
                            timeline.push(new_moment);
                        }
                    }
                }
            }
        }

        // Build version → timeline position index
        let mut version_index: HashMap<(TypeId, usize), usize> = HashMap::new();
        for (idx, moment) in timeline.iter().enumerate() {
            for (ns, ver) in &moment.version_map {
                // Only record the first occurrence of a version
                version_index.entry((*ns, ver.index)).or_insert(idx);
            }
        }

        Self {
            namespaces,
            timeline,
            version_index,
        }
    }

    /// Find the index where a version should be inserted to maintain order.
    fn find_index_to_insert(
        timeline: &[TimelineMoment],
        namespace: TypeId,
        version: &Version,
    ) -> Option<usize> {
        for (index, moment) in timeline.iter().enumerate() {
            if let Some(ver_at_moment) = moment.get_version(namespace)
                && version.index < ver_at_moment.index
            {
                return Some(index);
            }
        }
        None
    }

    /// Get the TimelineMoment at the given index.
    pub fn get_moment(&self, index: usize) -> Option<&TimelineMoment> {
        self.timeline.get(index)
    }

    /// Get the TimelineMoment containing the specified version.
    /// Ported from TS VersioningTimeline.get().
    pub fn get_moment_for_version(
        &self,
        namespace: TypeId,
        version_index: usize,
    ) -> Option<&TimelineMoment> {
        self.version_index
            .get(&(namespace, version_index))
            .and_then(|&idx| self.timeline.get(idx))
    }

    /// Return the timeline index for a version.
    /// Returns None if the version is not found.
    /// Ported from TS VersioningTimeline.getIndex().
    pub fn get_index(&self, namespace: TypeId, version_index: usize) -> Option<usize> {
        self.version_index.get(&(namespace, version_index)).copied()
    }

    /// Check if one version/moment is before another in the timeline.
    /// Ported from TS VersioningTimeline.isBefore().
    pub fn is_before(&self, ns_a: TypeId, ver_a: usize, ns_b: TypeId, ver_b: usize) -> bool {
        let idx_a = self.get_index(ns_a, ver_a);
        let idx_b = self.get_index(ns_b, ver_b);
        match (idx_a, idx_b) {
            (Some(a), Some(b)) => a < b,
            _ => false,
        }
    }

    /// Get the first moment in the timeline.
    /// Ported from TS VersioningTimeline.first().
    pub fn first(&self) -> Option<&TimelineMoment> {
        self.timeline.first()
    }

    /// Get the number of moments in the timeline.
    pub fn len(&self) -> usize {
        self.timeline.len()
    }

    /// Check if the timeline is empty.
    pub fn is_empty(&self) -> bool {
        self.timeline.is_empty()
    }

    /// Iterate over all moments in the timeline.
    pub fn iter(&self) -> impl Iterator<Item = (usize, &TimelineMoment)> {
        self.timeline.iter().enumerate()
    }

    /// Get the namespaces involved in this timeline.
    pub fn namespaces(&self) -> &[TypeId] {
        &self.namespaces
    }
}

// ============================================================================
// Versioning utility functions (pure, no Program dependency)
// Ported from TS packages/versioning/src/versioning.ts
// ============================================================================

/// Resolve when a type was first added, taking into account implicit versioning
/// from the parent type.
///
/// If a type has no explicit @added or @removed decorators, it inherits from
/// its parent's added version. If it was removed before being added, it was
/// implicitly available before (and thus inherits the parent's added version).
///
/// Ported from TS resolveWhenFirstAdded().
pub fn resolve_when_first_added(
    added: &[Version],
    removed: &[Version],
    parent_added: &Version,
) -> Vec<Version> {
    let implicitly_available = added.is_empty() && removed.is_empty();
    if implicitly_available {
        // If type has no version info, it inherits from the parent
        return vec![parent_added.clone()];
    }

    if !added.is_empty() {
        let added_first = removed.is_empty() || added[0].index < removed[0].index;
        if added_first {
            // If the type was added first, then implicitly it wasn't available before
            // and thus should NOT inherit from its parent
            return added.to_vec();
        }
    }

    if !removed.is_empty() {
        let removed_first = added.is_empty() || removed[0].index < added[0].index;
        if removed_first {
            // If the type was removed first, then implicitly it was available before
            // and thus SHOULD inherit from its parent
            let mut result = vec![parent_added.clone()];
            result.extend(added.iter().cloned());
            return result;
        }
    }

    added.to_vec()
}

/// Resolve when a type was removed, taking into account implicit versioning
/// from the parent type.
///
/// Ported from TS resolveRemoved().
pub fn resolve_removed(
    added: &[Version],
    removed: &[Version],
    parent_removed: Option<&Version>,
) -> Vec<Version> {
    if !removed.is_empty() {
        return removed.to_vec();
    }

    let implicitly_removed =
        added.is_empty() || parent_removed.is_some_and(|pr| added[0].index < pr.index);
    if let Some(pr) = parent_removed
        && implicitly_removed
    {
        return vec![pr.clone()];
    }

    Vec::new()
}

/// Compute the availability map for a type across all versions.
///
/// Given the added/removed version lists, parent added/removed versions,
/// and the full list of versions, returns a map from version index to
/// Availability.
///
/// Returns None if there is no versioning information at all
/// (no added, no removed, no typeChanged, no returnTypeChanged).
///
/// Ported from TS getAvailabilityMap().
pub fn get_availability_map(
    all_versions: &[Version],
    added: &[Version],
    removed: &[Version],
    parent_added: &Version,
    parent_removed: Option<&Version>,
    has_type_changed: bool,
    has_return_type_changed: bool,
) -> Option<Vec<(usize, Availability)>> {
    // If there is absolutely no versioning information, return None
    if added.is_empty() && removed.is_empty() && !has_type_changed && !has_return_type_changed {
        return None;
    }

    let resolved_added = resolve_when_first_added(added, removed, parent_added);
    let resolved_removed = resolve_removed(added, removed, parent_removed);

    let mut result = Vec::new();
    let mut is_avail = false;

    for ver in all_versions {
        let is_added = resolved_added.iter().any(|a| a.index == ver.index);
        let is_removed = resolved_removed.iter().any(|r| r.index == ver.index);

        if is_removed {
            is_avail = false;
            result.push((ver.index, Availability::Removed));
        } else if is_added {
            is_avail = true;
            result.push((ver.index, Availability::Added));
        } else if is_avail {
            result.push((ver.index, Availability::Available));
        } else {
            result.push((ver.index, Availability::Unavailable));
        }
    }

    Some(result)
}

// ============================================================================
// TSP Sources
// ============================================================================

/// The TypeSpec source for the versioning library decorators
pub const VERSIONING_DECORATORS_TSP: &str = r#"
import "../../dist/src/lib/tsp-index.js";

namespace TypeSpec.Versioning;

extern dec versioned(target: Namespace, versions: Enum);
extern dec added(target: Model | ModelProperty | Operation | Enum | EnumMember | Union | UnionVariant | Scalar | Interface, v: EnumMember);
extern dec removed(target: Model | ModelProperty | Operation | Enum | EnumMember | Union | UnionVariant | Scalar | Interface, v: EnumMember);
extern dec renamedFrom(target: Model | ModelProperty | Operation | Enum | EnumMember | Union | UnionVariant | Scalar | Interface, v: EnumMember, oldName: valueof string);
extern dec madeOptional(target: ModelProperty, v: EnumMember);
extern dec madeRequired(target: ModelProperty, v: EnumMember);
extern dec typeChangedFrom(target: ModelProperty, v: EnumMember, oldType: unknown);
extern dec returnTypeChangedFrom(target: Operation, v: EnumMember, oldReturnType: unknown);
extern dec useDependency(target: EnumMember | Namespace, ...versionRecords: EnumMember[]);
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_namespace() {
        assert_eq!(VERSIONING_NAMESPACE, "TypeSpec.Versioning");
    }

    #[test]
    fn test_create_versioning_library() {
        let diags = create_versioning_library();
        assert!(diags.len() >= 12);
    }

    #[test]
    fn test_apply_added() {
        let mut state = StateAccessors::new();
        assert!(get_added_on_versions(&state, 1).is_empty());
        apply_added(&mut state, 1, 2);
        assert_eq!(get_added_on_versions(&state, 1), vec![2]);
    }

    #[test]
    fn test_apply_added_multiple_versions() {
        let mut state = StateAccessors::new();
        apply_added(&mut state, 1, 3);
        apply_added(&mut state, 1, 1);
        // Should be sorted
        assert_eq!(get_added_on_versions(&state, 1), vec![1, 3]);
    }

    #[test]
    fn test_apply_removed() {
        let mut state = StateAccessors::new();
        apply_removed(&mut state, 1, 5);
        assert_eq!(get_removed_on_versions(&state, 1), vec![5]);
    }

    #[test]
    fn test_apply_versioned() {
        let mut state = StateAccessors::new();
        assert!(!is_versioned(&state, 1));
        apply_versioned(&mut state, 1, "ApiVersion");
        assert!(is_versioned(&state, 1));
        assert_eq!(
            get_version_enum_name(&state, 1),
            Some("ApiVersion".to_string())
        );
    }

    #[test]
    fn test_apply_renamed_from() {
        let mut state = StateAccessors::new();
        apply_renamed_from(&mut state, 1, 2, "oldName");
        let records = get_renamed_from(&state, 1);
        assert_eq!(records.len(), 1);
        assert_eq!(records[0], (2, "oldName".to_string()));
    }

    #[test]
    fn test_apply_renamed_from_multiple() {
        let mut state = StateAccessors::new();
        apply_renamed_from(&mut state, 1, 1, "firstOld");
        apply_renamed_from(&mut state, 1, 3, "secondOld");
        let records = get_renamed_from(&state, 1);
        assert_eq!(records.len(), 2);
    }

    #[test]
    fn test_apply_made_optional() {
        let mut state = StateAccessors::new();
        assert_eq!(get_made_optional_on(&state, 1), None);
        apply_made_optional(&mut state, 1, 3);
        assert_eq!(get_made_optional_on(&state, 1), Some(3));
    }

    #[test]
    fn test_apply_made_required() {
        let mut state = StateAccessors::new();
        apply_made_required(&mut state, 1, 2);
        assert_eq!(get_made_required_on(&state, 1), Some(2));
    }

    #[test]
    fn test_apply_type_changed_from() {
        let mut state = StateAccessors::new();
        apply_type_changed_from(&mut state, 1, 2, "string");
        let records = get_type_changed_from(&state, 1);
        assert_eq!(records.len(), 1);
        assert_eq!(records[0], (2, "string".to_string()));
    }

    #[test]
    fn test_apply_return_type_changed_from() {
        let mut state = StateAccessors::new();
        apply_return_type_changed_from(&mut state, 1, 3, "OldModel");
        let records = get_return_type_changed_from(&state, 1);
        assert_eq!(records.len(), 1);
        assert_eq!(records[0], (3, "OldModel".to_string()));
    }

    #[test]
    fn test_decorators_tsp_not_empty() {
        assert!(!VERSIONING_DECORATORS_TSP.is_empty());
        assert!(VERSIONING_DECORATORS_TSP.contains("dec versioned"));
        assert!(VERSIONING_DECORATORS_TSP.contains("dec added"));
        assert!(VERSIONING_DECORATORS_TSP.contains("dec removed"));
        assert!(VERSIONING_DECORATORS_TSP.contains("dec renamedFrom"));
        assert!(VERSIONING_DECORATORS_TSP.contains("dec madeOptional"));
        assert!(VERSIONING_DECORATORS_TSP.contains("dec useDependency"));
    }

    #[test]
    fn test_availability_enum() {
        assert_eq!(Availability::Unavailable.as_str(), "Unavailable");
        assert_eq!(Availability::Added.as_str(), "Added");
        assert_eq!(Availability::Available.as_str(), "Available");
        assert_eq!(Availability::Removed.as_str(), "Removed");
        assert_eq!(Availability::parse_str("Added"), Some(Availability::Added));
        assert_eq!(Availability::parse_str("unknown"), None);
    }

    #[test]
    fn test_use_dependency() {
        let mut state = StateAccessors::new();
        assert!(get_use_dependency(&state, 1).is_empty());
        apply_use_dependency(&mut state, 1, &[10, 20]);
        let deps = get_use_dependency(&state, 1);
        assert_eq!(deps, vec![10, 20]);
    }

    #[test]
    fn test_version_map() {
        let map = VersionMap::new(1, &[("v1", None), ("v2", Some("2024-01-01")), ("v3", None)]);
        assert_eq!(map.len(), 3);
        let versions = map.get_versions();
        assert_eq!(versions[0].name, "v1");
        assert_eq!(versions[0].value, "v1");
        assert_eq!(versions[0].index, 0);
        assert_eq!(versions[1].value, "2024-01-01");
        assert_eq!(versions[1].index, 1);
        assert_eq!(versions[2].index, 2);
    }

    #[test]
    fn test_version_map_lookup() {
        let map = VersionMap::new(1, &[("v1", None), ("v2", None)]);
        assert_eq!(map.get_by_index(0).unwrap().name, "v1");
        assert_eq!(map.get_by_index(1).unwrap().name, "v2");
        assert!(map.get_by_index(2).is_none());
        assert_eq!(map.get_by_name("v2").unwrap().index, 1);
        assert!(map.get_by_name("v99").is_none());
    }

    #[test]
    fn test_version_map_empty() {
        let map = VersionMap::new(1, &[]);
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
    }

    #[test]
    fn test_find_versioned_namespace() {
        let mut state = StateAccessors::new();
        apply_versioned(&mut state, 1, "ApiVersion");
        // Direct match
        let result = find_versioned_namespace(&state, 1, |_| None);
        assert_eq!(result, Some(1));
        // Walk up to find
        let result = find_versioned_namespace(&state, 2, |id| if id == 2 { Some(1) } else { None });
        assert_eq!(result, Some(1));
        // No match
        let result = find_versioned_namespace(&state, 99, |_| None);
        assert_eq!(result, None);
    }

    #[test]
    fn test_get_renamed_from_versions() {
        let mut state = StateAccessors::new();
        apply_renamed_from(&mut state, 1, 2, "oldName1");
        apply_renamed_from(&mut state, 1, 5, "oldName2");
        let versions = get_renamed_from_versions(&state, 1);
        assert_eq!(versions, vec![2, 5]);
    }

    // ========================================================================
    // VersioningTimeline / TimelineMoment tests
    // ========================================================================

    fn make_version(name: &str, index: usize) -> Version {
        Version {
            name: name.to_string(),
            value: name.to_string(),
            index,
        }
    }

    #[test]
    fn test_timeline_moment_basic() {
        let moment =
            TimelineMoment::new(vec![(1, make_version("v1", 0)), (2, make_version("l1", 0))]);
        assert_eq!(moment.name, "v1");
        assert!(moment.get_version(1).is_some());
        assert_eq!(moment.get_version(1).unwrap().name, "v1");
        assert!(moment.get_version(2).is_some());
        assert_eq!(moment.get_version(2).unwrap().name, "l1");
        assert!(moment.get_version(99).is_none());
    }

    #[test]
    fn test_timeline_moment_versions() {
        let moment =
            TimelineMoment::new(vec![(1, make_version("v1", 0)), (2, make_version("l1", 0))]);
        let versions: Vec<&Version> = moment.versions().collect();
        assert_eq!(versions.len(), 2);
    }

    #[test]
    fn test_versioning_timeline_basic() {
        // Simple case: service with v1, v2
        let resolutions = vec![
            vec![(1, make_version("v1", 0))],
            vec![(1, make_version("v2", 1))],
        ];
        let timeline = VersioningTimeline::new(&resolutions, |_| None);
        assert_eq!(timeline.len(), 2);
        assert!(timeline.first().is_some());
        assert_eq!(timeline.first().unwrap().name, "v1");
    }

    #[test]
    fn test_versioning_timeline_with_library() {
        // Service: v1 → l1, v2 → l3
        // Library has l1, l2, l3, l4 - intermediate versions should be inserted
        let service_ns: TypeId = 1;
        let library_ns: TypeId = 2;

        let resolutions = vec![
            vec![
                (service_ns, make_version("v1", 0)),
                (library_ns, make_version("l1", 0)),
            ],
            vec![
                (service_ns, make_version("v2", 1)),
                (library_ns, make_version("l3", 2)),
            ],
        ];

        let timeline = VersioningTimeline::new(&resolutions, |ns| {
            if ns == library_ns {
                Some(vec![
                    make_version("l1", 0),
                    make_version("l2", 1),
                    make_version("l3", 2),
                    make_version("l4", 3),
                ])
            } else if ns == service_ns {
                Some(vec![make_version("v1", 0), make_version("v2", 1)])
            } else {
                None
            }
        });

        // Timeline should have: v1+l1, l2, v2+l3, l4
        assert!(timeline.len() >= 2);

        // v1 should be at index 0
        assert_eq!(timeline.get_index(service_ns, 0), Some(0));

        // Check is_before
        assert!(timeline.is_before(service_ns, 0, service_ns, 1)); // v1 before v2
        assert!(!timeline.is_before(service_ns, 1, service_ns, 0)); // v2 NOT before v1
    }

    #[test]
    fn test_versioning_timeline_get_moment_for_version() {
        let ns: TypeId = 1;
        let resolutions = vec![
            vec![(ns, make_version("v1", 0))],
            vec![(ns, make_version("v2", 1))],
        ];
        let timeline = VersioningTimeline::new(&resolutions, |_| None);

        let moment = timeline.get_moment_for_version(ns, 0);
        assert!(moment.is_some());
        assert_eq!(moment.unwrap().name, "v1");

        let moment2 = timeline.get_moment_for_version(ns, 1);
        assert!(moment2.is_some());
        assert_eq!(moment2.unwrap().name, "v2");

        // Non-existent version
        assert!(timeline.get_moment_for_version(ns, 99).is_none());
    }

    #[test]
    fn test_versioning_timeline_empty() {
        let timeline = VersioningTimeline::new(&[], |_| None);
        assert!(timeline.is_empty());
        assert_eq!(timeline.len(), 0);
        assert!(timeline.first().is_none());
    }

    // ========================================================================
    // resolveWhenFirstAdded / resolveRemoved / getAvailabilityMap tests
    // ========================================================================

    #[test]
    fn test_resolve_when_first_added_implicit() {
        // No explicit added/removed → inherit from parent
        let parent = make_version("v1", 0);
        let result = resolve_when_first_added(&[], &[], &parent);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].index, 0);
    }

    #[test]
    fn test_resolve_when_first_added_explicit_first() {
        // Added first, not removed → no parent inheritance
        let parent = make_version("v1", 0);
        let added = vec![make_version("v2", 1)];
        let result = resolve_when_first_added(&added, &[], &parent);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].index, 1);
    }

    #[test]
    fn test_resolve_when_first_added_removed_first() {
        // Removed first (implies existed before) → inherit parent + keep added
        let parent = make_version("v1", 0);
        let added = vec![make_version("v3", 2)];
        let removed = vec![make_version("v2", 1)];
        let result = resolve_when_first_added(&added, &removed, &parent);
        // Should include parent (index 0) and added (index 2)
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].index, 0); // parent
        assert_eq!(result[1].index, 2); // added
    }

    #[test]
    fn test_resolve_removed_explicit() {
        let added = vec![make_version("v1", 0)];
        let removed = vec![make_version("v3", 2)];
        let result = resolve_removed(&added, &removed, None);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].index, 2);
    }

    #[test]
    fn test_resolve_removed_implicit_from_parent() {
        let added = vec![make_version("v1", 0)];
        let parent_removed = make_version("v2", 1);
        // added[0].index (0) < parent_removed.index (1) → implicitly removed
        let result = resolve_removed(&added, &[], Some(&parent_removed));
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].index, 1);
    }

    #[test]
    fn test_resolve_removed_not_implicit() {
        let added = vec![make_version("v3", 2)];
        let parent_removed = make_version("v1", 0);
        // added[0].index (2) >= parent_removed.index (0) → NOT implicitly removed
        let result = resolve_removed(&added, &[], Some(&parent_removed));
        assert!(result.is_empty());
    }

    #[test]
    fn test_get_availability_map_no_info() {
        let all = vec![make_version("v1", 0), make_version("v2", 1)];
        let parent = make_version("v1", 0);
        let result = get_availability_map(&all, &[], &[], &parent, None, false, false);
        assert!(result.is_none());
    }

    #[test]
    fn test_get_availability_map_added_in_v2() {
        let all = vec![
            make_version("v1", 0),
            make_version("v2", 1),
            make_version("v3", 2),
        ];
        let parent = make_version("v1", 0);
        let added = vec![make_version("v2", 1)];
        let result = get_availability_map(&all, &added, &[], &parent, None, false, false);
        assert!(result.is_some());
        let map = result.unwrap();
        assert_eq!(map[0], (0, Availability::Unavailable)); // v1: not available
        assert_eq!(map[1], (1, Availability::Added)); // v2: added
        assert_eq!(map[2], (2, Availability::Available)); // v3: available
    }

    #[test]
    fn test_get_availability_map_added_and_removed() {
        let all = vec![
            make_version("v1", 0),
            make_version("v2", 1),
            make_version("v3", 2),
        ];
        let parent = make_version("v1", 0);
        let added = vec![make_version("v1", 0)];
        let removed = vec![make_version("v2", 1)];
        let result = get_availability_map(&all, &added, &removed, &parent, None, false, false);
        assert!(result.is_some());
        let map = result.unwrap();
        assert_eq!(map[0], (0, Availability::Added)); // v1: added
        assert_eq!(map[1], (1, Availability::Removed)); // v2: removed
        assert_eq!(map[2], (2, Availability::Unavailable)); // v3: unavailable
    }

    #[test]
    fn test_get_availability_map_with_type_changed() {
        let all = vec![make_version("v1", 0), make_version("v2", 1)];
        let parent = make_version("v1", 0);
        // No explicit added/removed, but has_type_changed = true
        let result = get_availability_map(&all, &[], &[], &parent, None, true, false);
        assert!(result.is_some());
        let map = result.unwrap();
        // Implicitly available from parent (v1)
        assert_eq!(map[0], (0, Availability::Added)); // v1: added (from parent)
        assert_eq!(map[1], (1, Availability::Available)); // v2: available
    }

    // ========================================================================
    // Versioning mutator type definition tests
    // ========================================================================

    #[test]
    fn test_versioning_mutator_types() {
        let versioned = VersioningMutators::Versioned(VersionedMutators {
            snapshots: vec![
                VersionSnapshotMutation {
                    version: make_version("v1", 0),
                    name: "VersionSnapshot v1".to_string(),
                },
                VersionSnapshotMutation {
                    version: make_version("v2", 1),
                    name: "VersionSnapshot v2".to_string(),
                },
            ],
        });
        match versioned {
            VersioningMutators::Versioned(v) => {
                assert_eq!(v.snapshots.len(), 2);
                assert_eq!(v.snapshots[0].version.name, "v1");
            }
            _ => panic!("Expected Versioned variant"),
        }

        let transient = VersioningMutators::Transient(TransientVersioningMutator {
            name: "TransientVersionSnapshot".to_string(),
        });
        match transient {
            VersioningMutators::Transient(t) => {
                assert_eq!(t.name, "TransientVersionSnapshot");
            }
            _ => panic!("Expected Transient variant"),
        }
    }

    #[test]
    fn test_versioning_mutator_kind() {
        assert_eq!(
            VersioningMutatorKind::Versioned,
            VersioningMutatorKind::Versioned
        );
        assert_ne!(
            VersioningMutatorKind::Versioned,
            VersioningMutatorKind::Transient
        );
    }
}
