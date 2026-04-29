//! Standard library decorator declarations registry
//!
//! Ported from TypeSpec compiler/lib/std/decorators.tsp, lib/prototypes.tsp,
//! and lib/intrinsic/tsp-index.ts.
//!
//! This registry defines all standard library decorator declarations that are
//! programmatically injected into the checker during initialization, replacing
//! the need to parse .tsp files.

/// A standard library decorator definition
#[derive(Debug, Clone)]
pub struct StdDecoratorDef {
    /// Decorator name (e.g., "doc", "indexer")
    pub name: &'static str,
    /// Namespace (e.g., "TypeSpec", "TypeSpec.Prototypes")
    pub namespace: &'static str,
    /// Whether this decorator is internal (cannot be used from user code)
    pub is_internal: bool,
}

/// All standard library decorator declarations.
/// Ported from:
/// - `lib/intrinsics.tsp` + `lib/intrinsic/tsp-index.ts` (intrinsic decorators)
/// - `lib/prototypes.tsp` (prototype decorators)
/// - `lib/std/decorators.tsp` (standard decorators)
/// - `lib/std/visibility.tsp` (visibility decorators)
pub const STD_DECORATORS: &[StdDecoratorDef] = &[
    // ===== Intrinsic decorators (from intrinsics.tsp / tsp-index.ts) =====
    // These are loaded via JS import in TS, marked as internal in the binder.
    StdDecoratorDef {
        name: "indexer",
        namespace: "TypeSpec",
        is_internal: true,
    },
    StdDecoratorDef {
        name: "docFromComment",
        namespace: "TypeSpec",
        is_internal: true,
    },
    // ===== Prototype decorators (from prototypes.tsp) =====
    StdDecoratorDef {
        name: "getter",
        namespace: "TypeSpec.Prototypes",
        is_internal: true,
    },
    // ===== Standard decorators (from lib/std/decorators.tsp) =====
    // Documentation
    StdDecoratorDef {
        name: "summary",
        namespace: "TypeSpec",
        is_internal: false,
    },
    StdDecoratorDef {
        name: "doc",
        namespace: "TypeSpec",
        is_internal: false,
    },
    StdDecoratorDef {
        name: "returnsDoc",
        namespace: "TypeSpec",
        is_internal: false,
    },
    StdDecoratorDef {
        name: "errorsDoc",
        namespace: "TypeSpec",
        is_internal: false,
    },
    // Service
    StdDecoratorDef {
        name: "service",
        namespace: "TypeSpec",
        is_internal: false,
    },
    // Type classification
    StdDecoratorDef {
        name: "error",
        namespace: "TypeSpec",
        is_internal: false,
    },
    StdDecoratorDef {
        name: "mediaTypeHint",
        namespace: "TypeSpec",
        is_internal: false,
    },
    // String constraints
    StdDecoratorDef {
        name: "format",
        namespace: "TypeSpec",
        is_internal: false,
    },
    StdDecoratorDef {
        name: "pattern",
        namespace: "TypeSpec",
        is_internal: false,
    },
    StdDecoratorDef {
        name: "minLength",
        namespace: "TypeSpec",
        is_internal: false,
    },
    StdDecoratorDef {
        name: "maxLength",
        namespace: "TypeSpec",
        is_internal: false,
    },
    // Array constraints
    StdDecoratorDef {
        name: "minItems",
        namespace: "TypeSpec",
        is_internal: false,
    },
    StdDecoratorDef {
        name: "maxItems",
        namespace: "TypeSpec",
        is_internal: false,
    },
    // Numeric constraints
    StdDecoratorDef {
        name: "minValue",
        namespace: "TypeSpec",
        is_internal: false,
    },
    StdDecoratorDef {
        name: "maxValue",
        namespace: "TypeSpec",
        is_internal: false,
    },
    StdDecoratorDef {
        name: "minValueExclusive",
        namespace: "TypeSpec",
        is_internal: false,
    },
    StdDecoratorDef {
        name: "maxValueExclusive",
        namespace: "TypeSpec",
        is_internal: false,
    },
    // Security
    StdDecoratorDef {
        name: "secret",
        namespace: "TypeSpec",
        is_internal: false,
    },
    // Tagging
    StdDecoratorDef {
        name: "tag",
        namespace: "TypeSpec",
        is_internal: false,
    },
    // Naming
    StdDecoratorDef {
        name: "friendlyName",
        namespace: "TypeSpec",
        is_internal: false,
    },
    // Identity
    StdDecoratorDef {
        name: "key",
        namespace: "TypeSpec",
        is_internal: false,
    },
    // Overloading
    StdDecoratorDef {
        name: "overload",
        namespace: "TypeSpec",
        is_internal: false,
    },
    // Encoding
    StdDecoratorDef {
        name: "encodedName",
        namespace: "TypeSpec",
        is_internal: false,
    },
    StdDecoratorDef {
        name: "encode",
        namespace: "TypeSpec",
        is_internal: false,
    },
    // Discrimination
    StdDecoratorDef {
        name: "discriminated",
        namespace: "TypeSpec",
        is_internal: false,
    },
    StdDecoratorDef {
        name: "discriminator",
        namespace: "TypeSpec",
        is_internal: false,
    },
    // Examples
    StdDecoratorDef {
        name: "example",
        namespace: "TypeSpec",
        is_internal: false,
    },
    StdDecoratorDef {
        name: "opExample",
        namespace: "TypeSpec",
        is_internal: false,
    },
    // Model transforms
    StdDecoratorDef {
        name: "withOptionalProperties",
        namespace: "TypeSpec",
        is_internal: false,
    },
    StdDecoratorDef {
        name: "withUpdateableProperties",
        namespace: "TypeSpec",
        is_internal: false,
    },
    StdDecoratorDef {
        name: "withoutOmittedProperties",
        namespace: "TypeSpec",
        is_internal: false,
    },
    StdDecoratorDef {
        name: "withPickedProperties",
        namespace: "TypeSpec",
        is_internal: false,
    },
    StdDecoratorDef {
        name: "withoutDefaultValues",
        namespace: "TypeSpec",
        is_internal: false,
    },
    StdDecoratorDef {
        name: "withDefaultKeyVisibility",
        namespace: "TypeSpec",
        is_internal: false,
    },
    // Lifecycle transforms (from lib/std/visibility.tsp)
    StdDecoratorDef {
        name: "withLifecycleUpdate",
        namespace: "TypeSpec",
        is_internal: false,
    },
    StdDecoratorDef {
        name: "withVisibilityFilter",
        namespace: "TypeSpec",
        is_internal: false,
    },
    // Visibility (from lib/std/visibility.tsp)
    StdDecoratorDef {
        name: "visibility",
        namespace: "TypeSpec",
        is_internal: false,
    },
    StdDecoratorDef {
        name: "removeVisibility",
        namespace: "TypeSpec",
        is_internal: false,
    },
    StdDecoratorDef {
        name: "invisible",
        namespace: "TypeSpec",
        is_internal: false,
    },
    StdDecoratorDef {
        name: "withVisibility",
        namespace: "TypeSpec",
        is_internal: false,
    },
    StdDecoratorDef {
        name: "parameterVisibility",
        namespace: "TypeSpec",
        is_internal: false,
    },
    StdDecoratorDef {
        name: "returnTypeVisibility",
        namespace: "TypeSpec",
        is_internal: false,
    },
    StdDecoratorDef {
        name: "defaultVisibility",
        namespace: "TypeSpec",
        is_internal: false,
    },
    // Paging
    StdDecoratorDef {
        name: "list",
        namespace: "TypeSpec",
        is_internal: false,
    },
    StdDecoratorDef {
        name: "offset",
        namespace: "TypeSpec",
        is_internal: false,
    },
    StdDecoratorDef {
        name: "pageIndex",
        namespace: "TypeSpec",
        is_internal: false,
    },
    StdDecoratorDef {
        name: "pageSize",
        namespace: "TypeSpec",
        is_internal: false,
    },
    StdDecoratorDef {
        name: "pageItems",
        namespace: "TypeSpec",
        is_internal: false,
    },
    StdDecoratorDef {
        name: "continuationToken",
        namespace: "TypeSpec",
        is_internal: false,
    },
    StdDecoratorDef {
        name: "nextLink",
        namespace: "TypeSpec",
        is_internal: false,
    },
    StdDecoratorDef {
        name: "prevLink",
        namespace: "TypeSpec",
        is_internal: false,
    },
    StdDecoratorDef {
        name: "firstLink",
        namespace: "TypeSpec",
        is_internal: false,
    },
    StdDecoratorDef {
        name: "lastLink",
        namespace: "TypeSpec",
        is_internal: false,
    },
    // Debugging
    StdDecoratorDef {
        name: "inspectType",
        namespace: "TypeSpec",
        is_internal: false,
    },
    StdDecoratorDef {
        name: "inspectTypeName",
        namespace: "TypeSpec",
        is_internal: false,
    },
];
