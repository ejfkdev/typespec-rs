//! Compiler diagnostic message registry
//!
//! Ported from TypeSpec `compiler/src/core/messages.ts`.
//!
//! Currently only the set of diagnostic codes is ported (the
//! `compilerDiagnosticCodes` export), which is used by the unused-suppression
//! availability check. The full message-text registry (used upstream by
//! `createDiagnostic`/`reportDiagnostic`) is future porting work — the
//! checker currently reports diagnostics with inline messages.

use std::collections::HashSet;
use std::sync::OnceLock;

/// The set of all diagnostic codes defined by the compiler.
///
/// Ported from TS:
/// ```ts
/// export const compilerDiagnosticCodes = new Set(Object.keys(diagnostics));
/// ```
pub fn compiler_diagnostic_codes() -> &'static HashSet<&'static str> {
    static CODES: OnceLock<HashSet<&'static str>> = OnceLock::new();
    CODES.get_or_init(|| {
        HashSet::from([
            // Scanner errors
            "digit-expected",
            "hex-digit-expected",
            "binary-digit-expected",
            "unterminated",
            "creating-file",
            "invalid-escape-sequence",
            "no-new-line-start-triple-quote",
            "no-new-line-end-triple-quote",
            "triple-quote-indent",
            "invalid-character",
            "file-not-found",
            "file-load",
            "init-template-invalid-json",
            "init-template-download-failed",
            "multiple-blockless-namespace",
            "blockless-namespace-first",
            "import-first",
            "duplicate-import",
            "self-import",
            "token-expected",
            // Parser errors
            "unknown-directive",
            "augment-decorator-target",
            "duplicate-decorator",
            "decorator-conflict",
            "reserved-identifier",
            "invalid-directive-location",
            "invalid-decorator-location",
            "default-required",
            "invalid-template-argument-name",
            "invalid-template-default",
            "required-parameter-first",
            "rest-parameter-last",
            "rest-parameter-required",
            "doc-invalid-identifier",
            "experimental-feature",
            "auto-decorator-disabled",
            // Checker errors
            "using-invalid-ref",
            "invalid-type-ref",
            "invalid-template-args",
            "intersect-non-model",
            "intersect-invalid-index",
            "incompatible-indexer",
            "no-array-properties",
            "intersect-duplicate-property",
            "invalid-decorator",
            "invalid-ref",
            "duplicate-property",
            "override-property-mismatch",
            "extend-scalar",
            "extend-model",
            "is-model",
            "is-operation",
            "spread-model",
            "unsupported-default",
            "spread-object",
            "expect-value",
            "non-callable",
            "named-init-required",
            "invalid-primitive-init",
            "ambiguous-scalar-type",
            "unassignable",
            "property-unassignable",
            "property-required",
            "parameter-required",
            "value-in-type",
            "no-prop",
            "missing-index",
            "missing-property",
            "unexpected-property",
            "extends-interface",
            "extends-interface-duplicate",
            "interface-duplicate",
            "union-duplicate",
            "enum-member-duplicate",
            "constructor-duplicate",
            "spread-enum",
            "decorator-fail",
            "rest-parameter-array",
            "invalid-modifier",
            "function-return",
            "fn-in-union-expression",
            "missing-implementation",
            "missing-extern-declaration",
            "overload-same-parent",
            "shadow",
            "invalid-deprecation-argument",
            "duplicate-deprecation",
            "config-invalid-argument",
            "config-circular-variable",
            "config-path-absolute",
            "config-invalid-name",
            "path-unix-style",
            "config-path-not-found",
            "config-project-kind-filename",
            "config-project-only-option",
            "config-unknown-feature",
            "config-project-not-as-cli-config",
            "dynamic-import",
            "invalid-import",
            "invalid-main",
            "import-not-found",
            "library-invalid",
            "incompatible-library",
            "compiler-version-mismatch",
            "duplicate-symbol",
            "duplicate-suppression",
            "ambiguous-short-name",
            "decorator-decl-target",
            "mixed-string-template",
            "non-literal-string-template",
            "ambiguous-symbol",
            "duplicate-using",
            "on-validate-fail",
            "emitter-not-found",
            "invalid-emitter",
            "js-error",
            // Linter
            "missing-import",
            "invalid-rule-ref",
            "unknown-rule",
            "unknown-rule-set",
            "rule-enabled-disabled",
            "invalid-rule-options",
            "format-failed",
            "invalid-pattern-regex",
            // Decorator validation
            "decorator-wrong-target",
            "invalid-argument",
            "invalid-argument-count",
            "known-values-invalid-enum",
            "invalid-value",
            "deprecated",
            "no-optional-key",
            "invalid-discriminated-union",
            "invalid-discriminated-union-variant",
            "missing-discriminator-property",
            "invalid-discriminator-value",
            "invalid-encode",
            "invalid-mime-type",
            "no-mime-type-suffix",
            "encoded-name-conflict",
            "incompatible-paging-props",
            "invalid-paging-prop",
            "duplicate-paging-prop",
            "missing-paging-items",
            "service-decorator-duplicate",
            "list-type-not-model",
            "invalid-range",
            "add-response",
            "add-parameter",
            "add-model-property",
            "add-model-property-fail",
            "add-response-type",
            "circular-base-type",
            "circular-constraint",
            "circular-op-signature",
            "circular-alias-type",
            "circular-const",
            "circular-prop",
            "conflict-marker",
            "visibility-sealed",
            "default-visibility-not-member",
            "operation-visibility-constraint-empty",
            "no-compatible-vs-installed",
            "vs-extension-windows-only",
            "vsix-download-failed",
            "vscode-in-path",
            "invalid-option-flag",
            "cli-command-deprecated",
        ])
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compiler_diagnostic_codes_contains_known_codes() {
        let codes = compiler_diagnostic_codes();
        assert!(codes.contains("deprecated"));
        assert!(codes.contains("invalid-ref"));
        assert!(codes.contains("duplicate-symbol"));
        assert!(!codes.contains("not-a-real-code"));
    }
}
