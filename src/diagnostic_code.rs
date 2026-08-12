//! Diagnostic code short-name resolution
//!
//! Ported from TypeSpec `compiler/src/core/diagnostic-code.ts`
//! (microsoft/typespec#11209).
//!
//! Full diagnostic codes and linter rule ids have the form
//! `${packageName}/${code}` (e.g. `@typespec/http/no-foo`). Those are
//! verbose, so the compiler also accepts short forms where the package scope
//! is stripped (`http/no-foo`) or replaced by a library-declared alias
//! (`tcgc/no-foo`).

/// Information about a loaded library used to compute short diagnostic codes.
///
/// Ported from TS `LibraryNameInfo`.
#[derive(Debug, Clone)]
pub struct LibraryNameInfo {
    /// Full package name e.g. `@typespec/http`. Matches package.json name.
    pub name: String,
    /// Optional library-declared alias e.g. `tcgc`. Overrides the
    /// auto-stripped name.
    pub alias: Option<String>,
}

/// Compute the short name of a package by stripping the TypeSpec scope or
/// applying a library-declared alias.
///
/// - An explicit `alias` always wins.
/// - `@typespec/<name>` -> `<name>`
/// - `@<scope>/typespec-<name>` -> `<name>`
/// - `typespec-<name>` -> `<name>`
/// - otherwise there is no short form (returns `None`).
///
/// Ported from TS `getPackageShortName`.
pub fn get_package_short_name(name: &str, alias: Option<&str>) -> Option<String> {
    if let Some(alias) = alias
        && !alias.is_empty()
    {
        return Some(alias.to_string());
    }

    if let Some(rest) = name.strip_prefix("@typespec/") {
        return Some(rest.to_string());
    }

    // @<scope>/typespec-<name>
    if let Some(scope_rest) = name.strip_prefix('@')
        && let Some((_scope, after_slash)) = scope_rest.split_once('/')
        && let Some(rest) = after_slash.strip_prefix("typespec-")
    {
        return Some(rest.to_string());
    }

    if let Some(rest) = name.strip_prefix("typespec-") {
        return Some(rest.to_string());
    }

    None
}

/// Format the conflicting full package names of an ambiguous short name for
/// display.
///
/// Ported from TS `formatShortNameCandidates`.
pub fn format_short_name_candidates(candidates: &[String]) -> String {
    candidates
        .iter()
        .map(|name| format!("\"{}\"", name))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Returns whether `alias` is a valid library alias: a non-empty, kebab-case
/// identifier made of lowercase letters and digits, with single hyphens
/// allowed between segments.
///
/// Ported from TS `isValidLibraryAlias`.
pub fn is_valid_library_alias(alias: &str) -> bool {
    if alias.is_empty() {
        return false;
    }
    let mut chars = alias.chars().peekable();
    let mut prev_hyphen = false;
    let mut first = true;
    while let Some(c) = chars.next() {
        let is_lower_digit = c.is_ascii_lowercase() || c.is_ascii_digit();
        if c == '-' {
            // No leading/trailing/double hyphens.
            if first || prev_hyphen || chars.peek().is_none() {
                return false;
            }
            prev_hyphen = true;
        } else if is_lower_digit {
            prev_hyphen = false;
        } else {
            return false;
        }
        first = false;
    }
    true
}

/// Ambiguity information for a short name mapping to multiple libraries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AmbiguousShortName {
    pub short_name: String,
    pub candidates: Vec<String>,
}

/// Resolves short/full diagnostic codes for a given set of loaded libraries.
///
/// Ported from TS `DiagnosticCodeResolver` / `createDiagnosticCodeResolver`.
#[derive(Debug, Clone, Default)]
pub struct DiagnosticCodeResolver {
    /// Short name -> full package name (only for unambiguous short names).
    short_to_full: std::collections::HashMap<String, String>,
    /// Short name -> conflicting full package names.
    ambiguous_short_names: std::collections::HashMap<String, Vec<String>>,
    /// Full package names, longest first so the most specific prefix wins.
    full_names: Vec<String>,
}

/// Create a resolver mapping between full and short diagnostic codes for the
/// given loaded libraries. When two libraries would resolve to the same short
/// name, that short name is considered ambiguous and all conflicting
/// libraries fall back to their full name.
///
/// Ported from TS `createDiagnosticCodeResolver`.
pub fn create_diagnostic_code_resolver(
    libraries: impl IntoIterator<Item = LibraryNameInfo>,
) -> DiagnosticCodeResolver {
    let mut short_to_names: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    let mut full_names: Vec<String> = Vec::new();

    for lib in libraries {
        full_names.push(lib.name.clone());
        if let Some(short) = get_package_short_name(&lib.name, lib.alias.as_deref()) {
            short_to_names.entry(short).or_default().push(lib.name);
        }
    }

    let mut short_to_full = std::collections::HashMap::new();
    let mut ambiguous_short_names = std::collections::HashMap::new();
    for (short, names) in short_to_names {
        if names.len() == 1 {
            short_to_full.insert(short, names.into_iter().next().unwrap());
        } else {
            ambiguous_short_names.insert(short, names);
        }
    }

    // Longest names first so that the most specific package prefix wins.
    full_names.sort_by_key(|name| std::cmp::Reverse(name.len()));

    DiagnosticCodeResolver {
        short_to_full,
        ambiguous_short_names,
        full_names,
    }
}

impl DiagnosticCodeResolver {
    fn match_full_package(&self, code: &str) -> Option<&str> {
        self.full_names
            .iter()
            .find(|name| code.starts_with(&format!("{}/", name)))
            .map(|s| s.as_str())
    }

    /// Normalize a user-provided code (short or full) to its canonical full
    /// `${packageName}/${code}` form. If the code cannot be resolved (unknown
    /// short name, ambiguous short name, or a bare compiler code) it is
    /// returned unchanged.
    ///
    /// Ported from TS `resolveCode`.
    pub fn resolve_code(&self, code: &str) -> String {
        // Already a full code referencing a known library.
        if self.match_full_package(code).is_some() {
            return code.to_string();
        }

        let Some(separator) = code.find('/') else {
            return code.to_string();
        };
        let short_name = &code[..separator];
        let rest = &code[separator + 1..];
        match self.short_to_full.get(short_name) {
            Some(full_name) => format!("{}/{}", full_name, rest),
            None => code.to_string(),
        }
    }

    /// If `code`'s leading short-name segment maps to two or more loaded
    /// libraries, return the ambiguity information; otherwise `None`. A full
    /// code that is already prefixed with a known package is never ambiguous.
    ///
    /// Ported from TS `getAmbiguousShortName`.
    pub fn get_ambiguous_short_name(&self, code: &str) -> Option<AmbiguousShortName> {
        // A full code referencing a known library is never ambiguous.
        if self.match_full_package(code).is_some() {
            return None;
        }

        let separator = code.find('/')?;
        let short_name = &code[..separator];
        let candidates = self.ambiguous_short_names.get(short_name)?;
        Some(AmbiguousShortName {
            short_name: short_name.to_string(),
            candidates: candidates.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn libs(entries: &[(&str, Option<&str>)]) -> Vec<LibraryNameInfo> {
        entries
            .iter()
            .map(|(name, alias)| LibraryNameInfo {
                name: name.to_string(),
                alias: alias.map(|s| s.to_string()),
            })
            .collect()
    }

    // ========================================================================
    // getPackageShortName
    // ========================================================================

    #[test]
    fn test_short_name_prefers_explicit_alias() {
        assert_eq!(
            get_package_short_name("@azure-tools/typespec-client-generator-core", Some("tcgc")),
            Some("tcgc".to_string())
        );
    }

    #[test]
    fn test_short_name_strips_typespec_scope() {
        assert_eq!(
            get_package_short_name("@typespec/http", None),
            Some("http".to_string())
        );
        assert_eq!(
            get_package_short_name("@typespec/compiler", None),
            Some("compiler".to_string())
        );
    }

    #[test]
    fn test_short_name_strips_scoped_typespec_prefix() {
        assert_eq!(
            get_package_short_name("@azure-tools/typespec-autorest", None),
            Some("autorest".to_string())
        );
    }

    #[test]
    fn test_short_name_strips_unscoped_typespec_prefix() {
        assert_eq!(
            get_package_short_name("typespec-foo", None),
            Some("foo".to_string())
        );
    }

    #[test]
    fn test_short_name_none_when_not_applicable() {
        assert_eq!(get_package_short_name("some-random-package", None), None);
        assert_eq!(get_package_short_name("@scope/other", None), None);
    }

    // ========================================================================
    // isValidLibraryAlias
    // ========================================================================

    #[test]
    fn test_valid_library_aliases() {
        for alias in ["tcgc", "http", "my-lib-2", "a", "client-generator-core"] {
            assert!(is_valid_library_alias(alias), "should accept {}", alias);
        }
    }

    #[test]
    fn test_invalid_library_aliases() {
        for alias in [
            "", "Foo", "foo/bar", "foo bar", "-foo", "foo-", "foo--bar", "foo_bar",
        ] {
            assert!(!is_valid_library_alias(alias), "should reject {:?}", alias);
        }
    }

    // ========================================================================
    // createDiagnosticCodeResolver / resolveCode
    // ========================================================================

    fn standard_resolver() -> DiagnosticCodeResolver {
        create_diagnostic_code_resolver(libs(&[
            ("@typespec/http", None),
            ("@typespec/compiler", None),
            ("@azure-tools/typespec-autorest", None),
            ("@azure-tools/typespec-client-generator-core", Some("tcgc")),
        ]))
    }

    #[test]
    fn test_resolve_scope_stripped_short_name() {
        let resolver = standard_resolver();
        assert_eq!(
            resolver.resolve_code("http/no-foo"),
            "@typespec/http/no-foo"
        );
    }

    #[test]
    fn test_resolve_typespec_stripped_short_name() {
        let resolver = standard_resolver();
        assert_eq!(
            resolver.resolve_code("autorest/no-foo"),
            "@azure-tools/typespec-autorest/no-foo"
        );
    }

    #[test]
    fn test_resolve_aliased_short_name() {
        let resolver = standard_resolver();
        assert_eq!(
            resolver.resolve_code("tcgc/no-foo"),
            "@azure-tools/typespec-client-generator-core/no-foo"
        );
    }

    #[test]
    fn test_resolve_keeps_full_code() {
        let resolver = standard_resolver();
        assert_eq!(
            resolver.resolve_code("@typespec/http/no-foo"),
            "@typespec/http/no-foo"
        );
    }

    #[test]
    fn test_resolve_keeps_nested_rule_path_prefix() {
        let resolver = standard_resolver();
        assert_eq!(
            resolver.resolve_code("http/casing/rule"),
            "@typespec/http/casing/rule"
        );
    }

    #[test]
    fn test_resolve_keeps_unknown_short_name() {
        let resolver = standard_resolver();
        assert_eq!(resolver.resolve_code("unknown/no-foo"), "unknown/no-foo");
    }

    #[test]
    fn test_resolve_keeps_bare_compiler_code() {
        let resolver = standard_resolver();
        assert_eq!(
            resolver.resolve_code("unknown-identifier"),
            "unknown-identifier"
        );
    }

    // ========================================================================
    // Short name conflicts
    // ========================================================================

    #[test]
    fn test_conflict_does_not_resolve_ambiguous_short_name() {
        let conflicting = create_diagnostic_code_resolver(libs(&[
            ("@typespec/http", None),
            ("typespec-http", None),
        ]));
        assert_eq!(conflicting.resolve_code("http/no-foo"), "http/no-foo");
    }

    #[test]
    fn test_conflict_still_resolves_non_conflicting_libraries() {
        let mixed = create_diagnostic_code_resolver(libs(&[
            ("@typespec/http", None),
            ("typespec-http", None),
            ("@typespec/openapi3", None),
        ]));
        assert_eq!(
            mixed.resolve_code("openapi3/no-foo"),
            "@typespec/openapi3/no-foo"
        );
    }

    // ========================================================================
    // getAmbiguousShortName
    // ========================================================================

    fn conflicting_resolver() -> DiagnosticCodeResolver {
        create_diagnostic_code_resolver(libs(&[
            ("@typespec/http", None),
            ("typespec-http", None),
            ("@typespec/openapi3", None),
        ]))
    }

    #[test]
    fn test_ambiguous_short_name_returns_candidates() {
        let conflicting = conflicting_resolver();
        assert_eq!(
            conflicting.get_ambiguous_short_name("http/no-foo"),
            Some(AmbiguousShortName {
                short_name: "http".to_string(),
                candidates: vec!["@typespec/http".to_string(), "typespec-http".to_string()],
            })
        );
    }

    #[test]
    fn test_ambiguous_short_name_none_for_unambiguous() {
        let conflicting = conflicting_resolver();
        assert_eq!(
            conflicting.get_ambiguous_short_name("openapi3/no-foo"),
            None
        );
    }

    #[test]
    fn test_ambiguous_short_name_none_for_full_code() {
        let conflicting = conflicting_resolver();
        assert_eq!(
            conflicting.get_ambiguous_short_name("@typespec/http/no-foo"),
            None
        );
    }

    #[test]
    fn test_ambiguous_short_name_none_for_unknown() {
        let conflicting = conflicting_resolver();
        assert_eq!(conflicting.get_ambiguous_short_name("unknown/no-foo"), None);
    }

    #[test]
    fn test_ambiguous_short_name_none_for_bare_code() {
        let conflicting = conflicting_resolver();
        assert_eq!(
            conflicting.get_ambiguous_short_name("unknown-identifier"),
            None
        );
    }

    #[test]
    fn test_format_short_name_candidates() {
        assert_eq!(
            format_short_name_candidates(&[
                "@typespec/http".to_string(),
                "typespec-http".to_string()
            ]),
            "\"@typespec/http\", \"typespec-http\""
        );
    }
}
