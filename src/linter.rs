//! Linter system for TypeSpec-Rust
//!
//! Ported from TypeSpec compiler/src/core/linter.ts
//!
//! Provides the core linter data structures and rule evaluation framework.
//! Note: The async/dynamic library loading portions of the TS linter are not
//! ported, as Rust doesn't have the same dynamic loading model.

use crate::diagnostics::{Diagnostic, DiagnosticSeverity};
use std::collections::HashMap;

/// Reference to a linter rule: "libraryName/ruleName"
pub type RuleRef = String;

/// A linter rule definition
#[derive(Debug, Clone)]
pub struct LinterRule {
    /// Fully qualified rule ID (e.g., "@typespec/compiler/unused-using")
    pub id: String,
    /// Rule name within the library
    pub name: String,
    /// Rule description
    pub description: String,
    /// Default severity
    pub severity: DiagnosticSeverity,
    /// Optional URL for documentation
    pub url: Option<String>,
    /// Whether this rule is async (not applicable in Rust - always sync)
    pub async_flag: bool,
    /// Messages for this rule
    pub messages: HashMap<String, String>,
}

impl LinterRule {
    /// Create a new linter rule
    pub fn new(name: &str, description: &str, severity: DiagnosticSeverity) -> Self {
        Self {
            id: name.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            severity,
            url: None,
            async_flag: false,
            messages: HashMap::new(),
        }
    }

    /// Create a linter rule with a fully qualified ID
    pub fn with_id(
        lib_name: &str,
        name: &str,
        description: &str,
        severity: DiagnosticSeverity,
    ) -> Self {
        Self {
            id: format!("{}/{}", lib_name, name),
            name: name.to_string(),
            description: description.to_string(),
            severity,
            url: None,
            async_flag: false,
            messages: HashMap::new(),
        }
    }

    /// Add a message to this rule
    pub fn add_message(&mut self, id: &str, text: &str) {
        self.messages.insert(id.to_string(), text.to_string());
    }

    /// Get a message by ID
    pub fn get_message(&self, id: &str) -> Option<&str> {
        self.messages.get(id).map(|s| s.as_str())
    }
}

/// A resolved linter definition with rules and rule sets
#[derive(Debug, Clone, Default)]
pub struct LinterResolvedDefinition {
    /// All rules in this linter
    pub rules: Vec<LinterRule>,
    /// Named rule sets (presets)
    pub rule_sets: HashMap<String, LinterRuleSet>,
}

/// A set of linter rules (enable/disable configuration)
#[derive(Debug, Clone, Default)]
pub struct LinterRuleSet {
    /// Rule sets this one extends from
    pub extends: Option<Vec<String>>,
    /// Rules to enable (name → enabled)
    pub enable: HashMap<String, bool>,
    /// Rules to disable
    pub disable: HashMap<String, bool>,
}

/// Parse a rule reference into (library_name, rule_name).
/// Rule references have the format "libraryName/ruleName".
/// Ported from TS parseRuleReference()
pub fn parse_rule_reference(ref_str: &str) -> Result<(String, String), Box<Diagnostic>> {
    let segments: Vec<&str> = ref_str.split('/').collect();
    let name = segments.last().copied();
    let library_name: String = segments[..segments.len().saturating_sub(1)].join("/");

    if library_name.is_empty() || name.is_none_or(|n| n.is_empty()) {
        return Err(Box::new(Diagnostic::error(
            "invalid-rule-ref",
            &format!(
                "Invalid rule reference: '{}'. Must be in format 'libraryName/ruleName'",
                ref_str
            ),
        )));
    }
    Ok((
        library_name,
        name.expect("name validated as non-empty above").to_string(),
    ))
}

/// Resolve a linter definition, adding a default "all" rule set if missing.
/// Ported from TS resolveLinterDefinition()
pub fn resolve_linter_definition(
    lib_name: &str,
    rules: Vec<LinterRule>,
) -> LinterResolvedDefinition {
    // Assign fully qualified IDs
    let rules: Vec<LinterRule> = rules
        .into_iter()
        .map(|mut rule| {
            rule.id = format!("{}/{}", lib_name, rule.name);
            rule
        })
        .collect();

    // If there are rules but no "all" rule set, create one
    let mut rule_sets = HashMap::new();
    if !rules.is_empty() {
        let all_enable: HashMap<String, bool> =
            rules.iter().map(|r| (r.id.clone(), true)).collect();
        rule_sets.insert(
            "all".to_string(),
            LinterRuleSet {
                extends: None,
                enable: all_enable,
                disable: HashMap::new(),
            },
        );
    }

    LinterResolvedDefinition { rules, rule_sets }
}

/// Result of linting
#[derive(Debug, Clone, Default)]
pub struct LinterResult {
    /// Diagnostics produced by the linter
    pub diagnostics: Vec<Diagnostic>,
    /// Stats about linter execution
    pub stats: crate::stats::LinterRunStats,
}

/// Context provided to a linter rule during execution.
/// Ported from TS LinterRuleContext.
pub struct LinterRuleContext<'a> {
    /// The checker being linted
    pub checker: &'a crate::checker::Checker,
    /// Diagnostics collected so far by this rule
    diagnostics: Vec<Diagnostic>,
}

impl<'a> LinterRuleContext<'a> {
    /// Create a new linter rule context
    pub fn new(checker: &'a crate::checker::Checker) -> Self {
        Self {
            checker,
            diagnostics: Vec::new(),
        }
    }

    /// Report a diagnostic from this rule
    pub fn report_diagnostic(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    /// Consume the context and return collected diagnostics
    pub fn into_diagnostics(self) -> Vec<Diagnostic> {
        self.diagnostics
    }
}

/// A linter rule callback that checks for issues.
/// Receives a mutable LinterRuleContext and produces diagnostics.
pub type LinterRuleCallback = fn(&mut LinterRuleContext);

/// A linter rule with an optional execution callback.
/// Ported from TS createLinterRule() pattern.
#[derive(Debug, Clone)]
pub struct LinterRuleWithCallback {
    /// The rule definition
    pub rule: LinterRule,
    /// The execution callback (None for rules that need JS runtime)
    pub callback: Option<LinterRuleCallback>,
}

/// A simple linter that can evaluate registered rules.
pub struct Linter {
    /// All registered rules (by ID)
    rule_map: HashMap<String, LinterRule>,
    /// Currently enabled rules (by ID)
    enabled_rules: HashMap<String, LinterRule>,
    /// Registered linter libraries
    libraries: HashMap<String, LinterResolvedDefinition>,
    /// Rule callbacks (by rule ID)
    callbacks: HashMap<String, LinterRuleCallback>,
}

impl Linter {
    /// Create a new linter
    pub fn new() -> Self {
        Self {
            rule_map: HashMap::new(),
            enabled_rules: HashMap::new(),
            libraries: HashMap::new(),
            callbacks: HashMap::new(),
        }
    }

    /// Register a linter library
    pub fn register_library(&mut self, name: &str, definition: LinterResolvedDefinition) {
        for rule in &definition.rules {
            self.rule_map.insert(rule.id.clone(), rule.clone());
        }
        self.libraries.insert(name.to_string(), definition);
    }

    /// Extend the enabled rule set
    pub fn extend_rule_set(&mut self, rule_set: &LinterRuleSet) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Process extends - collect extending sets first to avoid borrow issues
        if let Some(ref extends) = rule_set.extends {
            let mut extending_sets: Vec<LinterRuleSet> = Vec::new();
            for extending_ref in extends {
                match parse_rule_reference(extending_ref) {
                    Ok((lib_name, set_name)) => {
                        if let Some(lib) = self.libraries.get(&lib_name) {
                            if let Some(extending_set) = lib.rule_sets.get(&set_name) {
                                extending_sets.push(extending_set.clone());
                            } else {
                                diagnostics.push(Diagnostic::error(
                                    "unknown-rule-set",
                                    &format!(
                                        "Unknown rule set '{}' in library '{}'",
                                        set_name, lib_name
                                    ),
                                ));
                            }
                        }
                    }
                    Err(diag) => diagnostics.push(*diag),
                }
            }
            // Now apply extending sets
            for ext_set in extending_sets {
                let ext_diags = self.extend_rule_set(&ext_set);
                diagnostics.extend(ext_diags);
            }
        }

        // Process enable - collect enabled rules first
        let mut enabled_in_this_set = std::collections::HashSet::new();
        if !rule_set.enable.is_empty() {
            let mut rules_to_enable: Vec<(String, LinterRule)> = Vec::new();
            for (rule_name, enabled) in &rule_set.enable {
                if !enabled {
                    continue;
                }
                if let Some(rule) = self.rule_map.get(rule_name) {
                    enabled_in_this_set.insert(rule_name.clone());
                    rules_to_enable.push((rule_name.clone(), rule.clone()));
                } else {
                    diagnostics.push(Diagnostic::error(
                        "unknown-rule",
                        &format!("Unknown rule: '{}'", rule_name),
                    ));
                }
            }
            for (name, rule) in rules_to_enable {
                self.enabled_rules.insert(name, rule);
            }
        }

        // Process disable
        for rule_name in rule_set.disable.keys() {
            if enabled_in_this_set.contains(rule_name) {
                diagnostics.push(Diagnostic::error(
                    "rule-enabled-disabled",
                    &format!("Rule '{}' is both enabled and disabled", rule_name),
                ));
            }
            self.enabled_rules.remove(rule_name);
        }

        diagnostics
    }

    /// Get the currently enabled rules
    pub fn enabled_rules(&self) -> &HashMap<String, LinterRule> {
        &self.enabled_rules
    }

    /// Get all registered rules
    pub fn all_rules(&self) -> &HashMap<String, LinterRule> {
        &self.rule_map
    }

    /// Register a callback for a linter rule.
    /// The callback will be invoked when `lint()` is called and the rule is enabled.
    pub fn register_callback(&mut self, rule_id: &str, callback: LinterRuleCallback) {
        self.callbacks.insert(rule_id.to_string(), callback);
    }

    /// Execute all enabled linter rules against the given checker.
    /// Returns a LinterResult with collected diagnostics.
    /// Ported from TS lint() function.
    pub fn lint(&mut self, checker: &crate::checker::Checker) -> LinterResult {
        let mut all_diagnostics = Vec::new();

        for rule_id in self.enabled_rules.keys() {
            if let Some(&callback) = self.callbacks.get(rule_id) {
                let mut ctx = LinterRuleContext::new(checker);
                callback(&mut ctx);
                all_diagnostics.extend(ctx.into_diagnostics());
            }
        }

        LinterResult {
            diagnostics: all_diagnostics,
            stats: crate::stats::LinterRunStats::default(),
        }
    }
}

impl Default for Linter {
    fn default() -> Self {
        Self::new()
    }
}

/// Built-in linter library name
pub const BUILT_IN_LINTER_LIBRARY_NAME: &str = "@typespec/compiler";

/// Create the built-in linter library
pub fn create_built_in_linter_library() -> LinterResolvedDefinition {
    let mut unused_using_rule = LinterRule::with_id(
        BUILT_IN_LINTER_LIBRARY_NAME,
        "unused-using",
        "Using statement is not used",
        DiagnosticSeverity::Warning,
    );
    unused_using_rule.add_message("default", "Using statement is not used.");

    let mut unused_template_param_rule = LinterRule::with_id(
        BUILT_IN_LINTER_LIBRARY_NAME,
        "unused-template-parameter",
        "Template parameter is not used",
        DiagnosticSeverity::Warning,
    );
    unused_template_param_rule.add_message("default", "Template parameter '{name}' is not used.");
    unused_template_param_rule.add_message(
        "at",
        "Template parameter '{name}' declared here is not used.",
    );

    resolve_linter_definition(
        BUILT_IN_LINTER_LIBRARY_NAME,
        vec![unused_using_rule, unused_template_param_rule],
    )
}

// ============================================================================
// Built-in linter rule implementations
// ============================================================================

/// Unused using statement linter rule callback.
/// Extracts unused-using diagnostics from the checker's diagnostic list.
/// Ported from TS linter-rules/unused-using.rule.ts.
pub fn lint_unused_using(ctx: &mut LinterRuleContext) {
    // The checker already reports unused-using diagnostics during check_program().
    // Collect them from the checker's diagnostics so the linter can apply its own
    // severity/filtering rules.
    for diag in ctx.checker.diagnostics() {
        if diag.code == "unused-using" {
            ctx.report_diagnostic(diag.clone());
        }
    }
}

/// Unused template parameter linter rule callback.
/// Extracts unused-template-parameter diagnostics from the checker's diagnostic list.
/// Ported from TS linter-rules/unused-template-parameter.rule.ts.
pub fn lint_unused_template_parameter(ctx: &mut LinterRuleContext) {
    // The checker already reports unused-template-parameter diagnostics during check_program().
    // Collect them from the checker's diagnostics so the linter can apply its own
    // severity/filtering rules.
    for diag in ctx.checker.diagnostics() {
        if diag.code == "unused-template-parameter" {
            ctx.report_diagnostic(diag.clone());
        }
    }
}

/// Register all built-in linter rule callbacks with a linter instance.
pub fn register_builtin_linter_callbacks(linter: &mut Linter) {
    linter.register_callback(
        &format!("{}/unused-using", BUILT_IN_LINTER_LIBRARY_NAME),
        lint_unused_using,
    );
    linter.register_callback(
        &format!("{}/unused-template-parameter", BUILT_IN_LINTER_LIBRARY_NAME),
        lint_unused_template_parameter,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rule_reference_valid() {
        let (lib, name) = parse_rule_reference("@typespec/compiler/unused-using").unwrap();
        assert_eq!(lib, "@typespec/compiler");
        assert_eq!(name, "unused-using");
    }

    #[test]
    fn test_parse_rule_reference_nested() {
        let (lib, name) = parse_rule_reference("@scope/my-lib/my-rule").unwrap();
        assert_eq!(lib, "@scope/my-lib");
        assert_eq!(name, "my-rule");
    }

    #[test]
    fn test_parse_rule_reference_invalid_no_slash() {
        assert!(parse_rule_reference("norule").is_err());
    }

    #[test]
    fn test_parse_rule_reference_empty() {
        assert!(parse_rule_reference("").is_err());
    }

    #[test]
    fn test_parse_rule_reference_only_slash() {
        assert!(parse_rule_reference("/").is_err());
    }

    #[test]
    fn test_linter_rule_new() {
        let rule = LinterRule::new("my-rule", "My rule desc", DiagnosticSeverity::Warning);
        assert_eq!(rule.id, "my-rule");
        assert_eq!(rule.name, "my-rule");
        assert_eq!(rule.severity, DiagnosticSeverity::Warning);
    }

    #[test]
    fn test_linter_rule_with_id() {
        let rule = LinterRule::with_id("myLib", "my-rule", "desc", DiagnosticSeverity::Error);
        assert_eq!(rule.id, "myLib/my-rule");
        assert_eq!(rule.name, "my-rule");
    }

    #[test]
    fn test_linter_rule_messages() {
        let mut rule = LinterRule::new("test", "desc", DiagnosticSeverity::Warning);
        rule.add_message("default", "Something is wrong");
        rule.add_message("at", "At location {loc}");
        assert_eq!(rule.get_message("default"), Some("Something is wrong"));
        assert_eq!(rule.get_message("at"), Some("At location {loc}"));
        assert_eq!(rule.get_message("other"), None);
    }

    #[test]
    fn test_resolve_linter_definition() {
        let rules = vec![
            LinterRule::new("rule1", "desc1", DiagnosticSeverity::Warning),
            LinterRule::new("rule2", "desc2", DiagnosticSeverity::Error),
        ];
        let resolved = resolve_linter_definition("myLib", rules);
        assert_eq!(resolved.rules.len(), 2);
        assert_eq!(resolved.rules[0].id, "myLib/rule1");
        assert_eq!(resolved.rules[1].id, "myLib/rule2");
        // Should have "all" rule set
        assert!(resolved.rule_sets.contains_key("all"));
        let all_set = &resolved.rule_sets["all"];
        assert!(all_set.enable.get("myLib/rule1").copied().unwrap_or(false));
        assert!(all_set.enable.get("myLib/rule2").copied().unwrap_or(false));
    }

    #[test]
    fn test_resolve_linter_definition_empty() {
        let resolved = resolve_linter_definition("myLib", vec![]);
        assert!(resolved.rules.is_empty());
        // No "all" rule set for empty rules
        assert!(!resolved.rule_sets.contains_key("all"));
    }

    #[test]
    fn test_linter_register_library() {
        let mut linter = Linter::new();
        let resolved = resolve_linter_definition(
            "myLib",
            vec![LinterRule::new("r1", "desc1", DiagnosticSeverity::Warning)],
        );
        linter.register_library("myLib", resolved);
        assert!(linter.all_rules().contains_key("myLib/r1"));
    }

    #[test]
    fn test_linter_extend_rule_set() {
        let mut linter = Linter::new();
        let resolved = resolve_linter_definition(
            "myLib",
            vec![
                LinterRule::new("r1", "desc1", DiagnosticSeverity::Warning),
                LinterRule::new("r2", "desc2", DiagnosticSeverity::Error),
            ],
        );
        linter.register_library("myLib", resolved);

        // Enable r1
        let mut rule_set = LinterRuleSet::default();
        rule_set.enable.insert("myLib/r1".to_string(), true);
        let diags = linter.extend_rule_set(&rule_set);
        assert!(diags.is_empty());
        assert!(linter.enabled_rules().contains_key("myLib/r1"));
        assert!(!linter.enabled_rules().contains_key("myLib/r2"));
    }

    #[test]
    fn test_linter_extend_with_all_rule_set() {
        let mut linter = Linter::new();
        let resolved = resolve_linter_definition(
            "myLib",
            vec![LinterRule::new("r1", "desc1", DiagnosticSeverity::Warning)],
        );
        linter.register_library("myLib", resolved);

        // Use the "all" rule set
        let all_set = linter
            .libraries
            .get("myLib")
            .unwrap()
            .rule_sets
            .get("all")
            .cloned()
            .unwrap();
        let diags = linter.extend_rule_set(&all_set);
        assert!(diags.is_empty());
        assert!(linter.enabled_rules().contains_key("myLib/r1"));
    }

    #[test]
    fn test_linter_disable_rule() {
        let mut linter = Linter::new();
        let resolved = resolve_linter_definition(
            "myLib",
            vec![LinterRule::new("r1", "desc1", DiagnosticSeverity::Warning)],
        );
        linter.register_library("myLib", resolved);

        // Enable then disable
        let mut enable_set = LinterRuleSet::default();
        enable_set.enable.insert("myLib/r1".to_string(), true);
        linter.extend_rule_set(&enable_set);

        let mut disable_set = LinterRuleSet::default();
        disable_set.disable.insert("myLib/r1".to_string(), true);
        linter.extend_rule_set(&disable_set);
        assert!(!linter.enabled_rules().contains_key("myLib/r1"));
    }

    #[test]
    fn test_linter_unknown_rule() {
        let mut linter = Linter::new();
        let mut rule_set = LinterRuleSet::default();
        rule_set.enable.insert("nonexistent/rule".to_string(), true);
        let diags = linter.extend_rule_set(&rule_set);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, "unknown-rule");
    }

    #[test]
    fn test_linter_rule_enabled_and_disabled() {
        let mut linter = Linter::new();
        let resolved = resolve_linter_definition(
            "myLib",
            vec![LinterRule::new("r1", "desc1", DiagnosticSeverity::Warning)],
        );
        linter.register_library("myLib", resolved);

        let mut rule_set = LinterRuleSet::default();
        rule_set.enable.insert("myLib/r1".to_string(), true);
        rule_set.disable.insert("myLib/r1".to_string(), true);
        let diags = linter.extend_rule_set(&rule_set);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, "rule-enabled-disabled");
    }

    #[test]
    fn test_built_in_linter_library() {
        let lib = create_built_in_linter_library();
        assert_eq!(lib.rules.len(), 2);
        assert!(lib.rules.iter().any(|r| r.name == "unused-using"));
        assert!(
            lib.rules
                .iter()
                .any(|r| r.name == "unused-template-parameter")
        );
        assert!(lib.rule_sets.contains_key("all"));
    }

    // Additional tests ported from TS linter.test.ts

    #[test]
    fn test_registering_rule_does_not_enable_it() {
        // Ported from TS: "registering a rule doesn't enable it"
        let mut linter = Linter::new();
        let resolved = resolve_linter_definition(
            "myLib",
            vec![LinterRule::new("r1", "desc1", DiagnosticSeverity::Warning)],
        );
        linter.register_library("myLib", resolved);
        // Rule is registered but not enabled
        assert!(linter.all_rules().contains_key("myLib/r1"));
        assert!(!linter.enabled_rules().contains_key("myLib/r1"));
    }

    #[test]
    fn test_enabling_nonexistent_rule_set_emits_diagnostic() {
        // Ported from TS: "enabling a rule set that doesn't exists emit a diagnostic"
        let mut linter = Linter::new();
        let resolved = resolve_linter_definition(
            "myLib",
            vec![LinterRule::new("r1", "desc1", DiagnosticSeverity::Warning)],
        );
        linter.register_library("myLib", resolved);

        let rule_set = LinterRuleSet {
            extends: Some(vec!["myLib/nonexistent-set".to_string()]),
            ..Default::default()
        };
        let diags = linter.extend_rule_set(&rule_set);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, "unknown-rule-set");
    }

    #[test]
    fn test_extending_custom_rule_set() {
        // Ported from TS: "extending specific ruleset enable the rules inside"
        let mut linter = Linter::new();
        let mut resolved = resolve_linter_definition(
            "myLib",
            vec![
                LinterRule::new("r1", "desc1", DiagnosticSeverity::Warning),
                LinterRule::new("r2", "desc2", DiagnosticSeverity::Error),
            ],
        );
        // Add a custom rule set that only enables r1
        let mut custom_enable = HashMap::new();
        custom_enable.insert("myLib/r1".to_string(), true);
        resolved.rule_sets.insert(
            "custom".to_string(),
            LinterRuleSet {
                extends: None,
                enable: custom_enable,
                disable: HashMap::new(),
            },
        );
        linter.register_library("myLib", resolved);

        let rule_set = LinterRuleSet {
            extends: Some(vec!["myLib/custom".to_string()]),
            ..Default::default()
        };
        let diags = linter.extend_rule_set(&rule_set);
        assert!(diags.is_empty());
        assert!(linter.enabled_rules().contains_key("myLib/r1"));
        assert!(!linter.enabled_rules().contains_key("myLib/r2"));
    }

    #[test]
    fn test_multiple_libraries() {
        // Register rules from multiple libraries
        let mut linter = Linter::new();
        let resolved1 = resolve_linter_definition(
            "lib1",
            vec![LinterRule::new("r1", "desc1", DiagnosticSeverity::Warning)],
        );
        let resolved2 = resolve_linter_definition(
            "lib2",
            vec![LinterRule::new("r2", "desc2", DiagnosticSeverity::Error)],
        );
        linter.register_library("lib1", resolved1);
        linter.register_library("lib2", resolved2);

        assert!(linter.all_rules().contains_key("lib1/r1"));
        assert!(linter.all_rules().contains_key("lib2/r2"));

        // Enable one from each library
        let mut rule_set = LinterRuleSet::default();
        rule_set.enable.insert("lib1/r1".to_string(), true);
        rule_set.enable.insert("lib2/r2".to_string(), true);
        let diags = linter.extend_rule_set(&rule_set);
        assert!(diags.is_empty());
        assert!(linter.enabled_rules().contains_key("lib1/r1"));
        assert!(linter.enabled_rules().contains_key("lib2/r2"));
    }

    #[test]
    fn test_all_ruleset_includes_all_rules() {
        // Ported from TS: "/all ruleset is automatically provided and include all rules"
        let mut linter = Linter::new();
        let resolved = resolve_linter_definition(
            "myLib",
            vec![
                LinterRule::new("r1", "desc1", DiagnosticSeverity::Warning),
                LinterRule::new("r2", "desc2", DiagnosticSeverity::Error),
            ],
        );
        linter.register_library("myLib", resolved);

        // Use the "all" rule set
        let all_set = linter
            .libraries
            .get("myLib")
            .unwrap()
            .rule_sets
            .get("all")
            .cloned()
            .unwrap();
        let diags = linter.extend_rule_set(&all_set);
        assert!(diags.is_empty());
        assert!(linter.enabled_rules().contains_key("myLib/r1"));
        assert!(linter.enabled_rules().contains_key("myLib/r2"));
    }

    #[test]
    fn test_enabling_rule_from_unknown_library_emits_diagnostic() {
        // Ported from TS: "enabling a rule from a library that is not found emit a diagnostic"
        let mut linter = Linter::new();
        let resolved = resolve_linter_definition(
            "myLib",
            vec![LinterRule::new("r1", "desc1", DiagnosticSeverity::Warning)],
        );
        linter.register_library("myLib", resolved);

        let mut rule_set = LinterRuleSet::default();
        rule_set
            .enable
            .insert("other-lib/not-a-rule".to_string(), true);
        let diags = linter.extend_rule_set(&rule_set);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, "unknown-rule");
    }

    #[test]
    fn test_disable_only_removes_enabled_rule() {
        // Disabling a rule that was never enabled is a no-op
        let mut linter = Linter::new();
        let resolved = resolve_linter_definition(
            "myLib",
            vec![LinterRule::new("r1", "desc1", DiagnosticSeverity::Warning)],
        );
        linter.register_library("myLib", resolved);

        let mut disable_set = LinterRuleSet::default();
        disable_set.disable.insert("myLib/r1".to_string(), true);
        let diags = linter.extend_rule_set(&disable_set);
        assert!(diags.is_empty());
        assert!(!linter.enabled_rules().contains_key("myLib/r1"));
    }

    #[test]
    fn test_linter_rule_severity_preserved() {
        let rule_error = LinterRule::new("err-rule", "desc", DiagnosticSeverity::Error);
        let rule_warning = LinterRule::new("warn-rule", "desc", DiagnosticSeverity::Warning);
        assert_eq!(rule_error.severity, DiagnosticSeverity::Error);
        assert_eq!(rule_warning.severity, DiagnosticSeverity::Warning);
    }

    // ==================== Linter execution framework tests ====================

    #[test]
    fn test_linter_register_callback() {
        let mut linter = Linter::new();
        let resolved = resolve_linter_definition(
            "myLib",
            vec![LinterRule::new("r1", "desc1", DiagnosticSeverity::Warning)],
        );
        linter.register_library("myLib", resolved);

        fn test_callback(_ctx: &mut LinterRuleContext) {}
        linter.register_callback("myLib/r1", test_callback);
    }

    #[test]
    fn test_linter_rule_context_report_diagnostic() {
        let result = crate::parser::parse("");
        let mut checker = crate::checker::Checker::new();
        checker.set_parse_result(result.root_id, result.builder);
        checker.check_program();

        let mut ctx = LinterRuleContext::new(&checker);
        ctx.report_diagnostic(Diagnostic::warning("test-rule", "Test message"));
        ctx.report_diagnostic(Diagnostic::error("another-rule", "Another message"));

        let diags = ctx.into_diagnostics();
        assert_eq!(diags.len(), 2);
        assert_eq!(diags[0].code, "test-rule");
        assert_eq!(diags[1].code, "another-rule");
    }

    #[test]
    fn test_linter_rule_context_empty() {
        let result = crate::parser::parse("");
        let mut checker = crate::checker::Checker::new();
        checker.set_parse_result(result.root_id, result.builder);
        checker.check_program();

        let ctx = LinterRuleContext::new(&checker);
        let diags = ctx.into_diagnostics();
        assert!(diags.is_empty());
    }

    #[test]
    fn test_linter_with_callback_no_enabled_rules() {
        // If no rules are enabled, lint should produce no diagnostics
        let result = crate::parser::parse("model Foo {}");
        let mut checker = crate::checker::Checker::new();
        checker.set_parse_result(result.root_id, result.builder);
        checker.check_program();

        let mut linter = Linter::new();
        fn test_callback(_ctx: &mut LinterRuleContext) {}
        linter.register_callback("myLib/r1", test_callback);

        let lint_result = linter.lint(&checker);
        assert!(lint_result.diagnostics.is_empty());
    }

    #[test]
    fn test_linter_with_enabled_rule_and_callback() {
        let result = crate::parser::parse("model Foo {}");
        let mut checker = crate::checker::Checker::new();
        checker.set_parse_result(result.root_id, result.builder);
        checker.check_program();

        let mut linter = Linter::new();
        let resolved = resolve_linter_definition(
            "myLib",
            vec![LinterRule::new("r1", "desc1", DiagnosticSeverity::Warning)],
        );
        linter.register_library("myLib", resolved);

        // Enable the rule
        let mut rule_set = LinterRuleSet::default();
        rule_set.enable.insert("myLib/r1".to_string(), true);
        linter.extend_rule_set(&rule_set);

        // Register a callback that does nothing
        fn noop_callback(_ctx: &mut LinterRuleContext) {}
        linter.register_callback("myLib/r1", noop_callback);

        let lint_result = linter.lint(&checker);
        assert!(lint_result.diagnostics.is_empty());
    }

    #[test]
    fn test_linter_with_callback_that_reports() {
        let result = crate::parser::parse("model Foo {}");
        let mut checker = crate::checker::Checker::new();
        checker.set_parse_result(result.root_id, result.builder);
        checker.check_program();

        let mut linter = Linter::new();
        let resolved = resolve_linter_definition(
            "myLib",
            vec![LinterRule::new("r1", "desc1", DiagnosticSeverity::Warning)],
        );
        linter.register_library("myLib", resolved);

        let mut rule_set = LinterRuleSet::default();
        rule_set.enable.insert("myLib/r1".to_string(), true);
        linter.extend_rule_set(&rule_set);

        fn report_callback(ctx: &mut LinterRuleContext) {
            ctx.report_diagnostic(Diagnostic::warning("custom-rule", "Custom issue found"));
        }
        linter.register_callback("myLib/r1", report_callback);

        let lint_result = linter.lint(&checker);
        assert_eq!(lint_result.diagnostics.len(), 1);
        assert_eq!(lint_result.diagnostics[0].code, "custom-rule");
    }

    #[test]
    fn test_register_builtin_linter_callbacks() {
        let mut linter = Linter::new();
        let resolved = create_built_in_linter_library();
        linter.register_library(BUILT_IN_LINTER_LIBRARY_NAME, resolved);
        register_builtin_linter_callbacks(&mut linter);

        // Enable all rules
        let all_set = linter
            .libraries
            .get(BUILT_IN_LINTER_LIBRARY_NAME)
            .unwrap()
            .rule_sets
            .get("all")
            .cloned()
            .unwrap();
        linter.extend_rule_set(&all_set);
        assert!(
            linter
                .enabled_rules()
                .contains_key(&format!("{}/unused-using", BUILT_IN_LINTER_LIBRARY_NAME))
        );
        assert!(linter.enabled_rules().contains_key(&format!(
            "{}/unused-template-parameter",
            BUILT_IN_LINTER_LIBRARY_NAME
        )));
    }

    #[test]
    fn test_linter_rule_with_callback_struct() {
        let rule = LinterRule::new("r1", "desc", DiagnosticSeverity::Warning);
        fn my_callback(_ctx: &mut LinterRuleContext) {}
        let with_cb = LinterRuleWithCallback {
            rule,
            callback: Some(my_callback),
        };
        assert_eq!(with_cb.rule.name, "r1");
        assert!(with_cb.callback.is_some());
    }

    #[test]
    fn test_linter_rule_with_callback_none() {
        let rule = LinterRule::new("r1", "desc", DiagnosticSeverity::Warning);
        let with_cb = LinterRuleWithCallback {
            rule,
            callback: None,
        };
        assert!(with_cb.callback.is_none());
    }
}
