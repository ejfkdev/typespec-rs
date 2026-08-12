//! Compiler feature flags
//!
//! Ported from TypeSpec `compiler/src/core/features.ts`
//! (microsoft/typespec#10826 "Add project-scoped compiler feature flags").
//!
//! Experimental language features are gated behind named flags that a project
//! enables via the `features` list (in `tspconfig.yaml` upstream; in the Rust
//! port via [`crate::diagnostics::CompilerOptions::features`]).

/// Definition of a compiler feature.
///
/// Ported from TS `CompilerFeatureDefinition`.
#[derive(Debug, Clone, Copy)]
pub struct CompilerFeatureDefinition {
    pub description: &'static str,
}

/// The set of known compiler feature flags and their descriptions.
///
/// Ported from TS `compilerFeatures`.
pub const COMPILER_FEATURES: &[(&str, CompilerFeatureDefinition)] = &[
    (
        "function-declarations",
        CompilerFeatureDefinition {
            description: "Allows use of function declarations without experimental warnings in project code.",
        },
    ),
    (
        "auto-decorators",
        CompilerFeatureDefinition {
            description: "Allows use of auto decorator declarations without experimental warnings in project code.",
        },
    ),
];

/// All known compiler feature names.
///
/// Ported from TS `compilerFeatureNames`.
pub fn compiler_feature_names() -> Vec<&'static str> {
    COMPILER_FEATURES.iter().map(|(name, _)| *name).collect()
}

/// Whether the given string is a known compiler feature name.
///
/// Ported from TS `isCompilerFeatureName`.
pub fn is_compiler_feature_name(feature: &str) -> bool {
    COMPILER_FEATURES.iter().any(|(name, _)| *name == feature)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_known_feature_names() {
        assert!(is_compiler_feature_name("function-declarations"));
        assert!(is_compiler_feature_name("auto-decorators"));
        assert!(!is_compiler_feature_name("not-a-feature"));
    }

    #[test]
    fn test_feature_names_list() {
        let names = compiler_feature_names();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"function-declarations"));
        assert!(names.contains(&"auto-decorators"));
    }
}
