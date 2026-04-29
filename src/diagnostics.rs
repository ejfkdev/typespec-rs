//! Diagnostics module for TypeSpec-Rust
//!
//! Ported from TypeSpec compiler/src/core/diagnostics.ts
//! and TypeSpec compiler/src/core/diagnostic-creator.ts
//!
//! This module provides diagnostic collection, reporting, and creation utilities.

/// Diagnostic severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Information,
    Hint,
}

impl std::fmt::Display for DiagnosticSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiagnosticSeverity::Error => write!(f, "error"),
            DiagnosticSeverity::Warning => write!(f, "warning"),
            DiagnosticSeverity::Information => write!(f, "info"),
            DiagnosticSeverity::Hint => write!(f, "hint"),
        }
    }
}

/// A diagnostic message with location information
#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostic {
    /// Diagnostic code (e.g., "invalid-argument")
    pub code: String,
    /// Diagnostic severity
    pub severity: DiagnosticSeverity,
    /// The diagnostic message
    pub message: String,
    /// Optional URL for more information
    pub url: Option<String>,
    /// Source location
    pub location: Option<SourceLocation>,
    /// Related locations
    pub related: Vec<RelatedLocation>,
}

impl Diagnostic {
    /// Create a new diagnostic
    pub fn new(code: &str, message: &str) -> Self {
        Self {
            code: code.to_string(),
            severity: DiagnosticSeverity::Error,
            message: message.to_string(),
            url: None,
            location: None,
            related: Vec::new(),
        }
    }

    /// Create an error diagnostic
    pub fn error(code: &str, message: &str) -> Self {
        Self::new(code, message)
    }

    /// Create a warning diagnostic
    pub fn warning(code: &str, message: &str) -> Self {
        Self {
            code: code.to_string(),
            severity: DiagnosticSeverity::Warning,
            message: message.to_string(),
            url: None,
            location: None,
            related: Vec::new(),
        }
    }

    /// Check if this diagnostic is an error
    pub fn is_error(&self) -> bool {
        self.severity == DiagnosticSeverity::Error
    }
}

/// Source location for a diagnostic
#[derive(Debug, Clone, PartialEq)]
pub struct SourceLocation {
    /// Source file path
    pub file: String,
    /// Start position
    pub start: usize,
    /// End position
    pub end: usize,
    /// Whether this is a synthetic location
    pub is_synthetic: bool,
}

/// Related location for a diagnostic
#[derive(Debug, Clone, PartialEq)]
pub struct RelatedLocation {
    /// Location message
    pub message: String,
    /// The related location
    pub location: Option<SourceLocation>,
}

/// Diagnostic collector - collects diagnostics from operations
#[derive(Debug, Clone, Default)]
pub struct DiagnosticCollector {
    diagnostics: Vec<Diagnostic>,
}

impl DiagnosticCollector {
    /// Create a new diagnostic collector
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
        }
    }

    /// Add a diagnostic to the collector
    pub fn add(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    /// Get all collected diagnostics
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Clear all collected diagnostics
    pub fn clear(&mut self) {
        self.diagnostics.clear();
    }

    /// Check if there are any diagnostics
    pub fn has_diagnostics(&self) -> bool {
        !self.diagnostics.is_empty()
    }

    /// Check if any error-level diagnostics were reported
    pub fn has_error(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == DiagnosticSeverity::Error)
    }

    /// Pipe a result through the collector (for diagnostic accessor pattern)
    pub fn pipe<T>(&mut self, result: (T, Vec<Diagnostic>)) -> T {
        let (value, diags) = result;
        for diag in diags {
            self.add(diag);
        }
        value
    }
}

/// Ignore diagnostics and return just the value
pub fn ignore_diagnostics<T>(result: (T, Vec<Diagnostic>)) -> T {
    result.0
}

/// Create a synthetic source location
pub fn create_synthetic_source_location(loc: &str) -> SourceLocation {
    SourceLocation {
        file: loc.to_string(),
        start: 0,
        end: 0,
        is_synthetic: true,
    }
}

/// Get related locations for a diagnostic (template instantiation trace).
/// Ported from TS getRelatedLocations().
pub fn get_related_locations(_diagnostic: &Diagnostic) -> Vec<RelatedLocation> {
    // TODO: Requires template mapper infrastructure to trace template instantiations.
    // Returns empty until checker template system supports instantiation tracing.
    Vec::new()
}

/// Get the source location for a diagnostic target.
/// Ported from TS getSourceLocation().
/// Currently simplified — returns the diagnostic's own location or a synthetic one.
pub fn get_source_location(diagnostic: &Diagnostic) -> Option<SourceLocation> {
    diagnostic.location.clone()
}

/// Get the template instantiation trace for a diagnostic target.
/// Ported from TS getDiagnosticTemplateInstantitationTrace().
/// Returns nodes involved in template instantiation chain.
/// Currently returns empty — requires template mapper infrastructure.
pub fn get_diagnostic_template_instantiation_trace(
    _diagnostic: &Diagnostic,
) -> Vec<SourceLocation> {
    // TODO: Requires template mapper to trace instantiation chain.
    // Will walk: target.templateMapper → source.node → source.mapper → ...
    Vec::new()
}

// ============================================================================
// Diagnostic Creator
// Ported from TypeSpec compiler/src/core/diagnostic-creator.ts
// ============================================================================

/// Definition of a single diagnostic message within a diagnostic code.
#[derive(Debug, Clone)]
pub struct DiagnosticMessageDefinition {
    /// The message text, optionally a format string with {key} placeholders
    pub text: String,
}

/// Definition of a diagnostic code with its severity and messages.
#[derive(Debug, Clone)]
pub struct DiagnosticDefinition {
    /// The severity of this diagnostic
    pub severity: DiagnosticSeverity,
    /// Map of message ID to message definition (at minimum "default")
    pub messages: Vec<(String, DiagnosticMessageDefinition)>,
    /// Optional URL for more information
    pub url: Option<String>,
}

impl DiagnosticDefinition {
    /// Create a simple error diagnostic definition with a default message
    pub fn error(text: &str) -> Self {
        Self {
            severity: DiagnosticSeverity::Error,
            messages: vec![(
                "default".to_string(),
                DiagnosticMessageDefinition {
                    text: text.to_string(),
                },
            )],
            url: None,
        }
    }

    /// Create a simple warning diagnostic definition with a default message
    pub fn warning(text: &str) -> Self {
        Self {
            severity: DiagnosticSeverity::Warning,
            messages: vec![(
                "default".to_string(),
                DiagnosticMessageDefinition {
                    text: text.to_string(),
                },
            )],
            url: None,
        }
    }

    /// Create an error diagnostic definition with multiple message variants.
    /// This matches the TS pattern where a diagnostic code can have
    /// multiple message IDs (e.g., "default", "addedAfter", "removedBefore").
    pub fn error_with_messages(messages: Vec<(&str, &str)>) -> Self {
        Self {
            severity: DiagnosticSeverity::Error,
            messages: messages
                .into_iter()
                .map(|(id, text)| {
                    (
                        id.to_string(),
                        DiagnosticMessageDefinition {
                            text: text.to_string(),
                        },
                    )
                })
                .collect(),
            url: None,
        }
    }

    /// Create a warning diagnostic definition with multiple message variants.
    pub fn warning_with_messages(messages: Vec<(&str, &str)>) -> Self {
        Self {
            severity: DiagnosticSeverity::Warning,
            messages: messages
                .into_iter()
                .map(|(id, text)| {
                    (
                        id.to_string(),
                        DiagnosticMessageDefinition {
                            text: text.to_string(),
                        },
                    )
                })
                .collect(),
            url: None,
        }
    }

    /// Get the message text for a given message ID
    pub fn get_message(&self, message_id: &str) -> Option<&str> {
        self.messages
            .iter()
            .find(|(id, _)| id == message_id)
            .map(|(_, def)| def.text.as_str())
    }
}

/// A map of diagnostic codes to their definitions.
/// Key is the diagnostic code string.
pub type DiagnosticMap = std::collections::HashMap<String, DiagnosticDefinition>;

/// Diagnostic creator that creates typed diagnostics from a diagnostic map.
/// Ported from TS createDiagnosticCreator()
#[derive(Debug, Clone)]
pub struct DiagnosticCreator {
    /// The diagnostic map this creator is based on
    diagnostics: DiagnosticMap,
    /// Optional library name for namespacing diagnostic codes
    library_name: Option<String>,
}

impl DiagnosticCreator {
    /// Create a new diagnostic creator from a diagnostic map and optional library name.
    pub fn new(diagnostics: DiagnosticMap, library_name: Option<String>) -> Self {
        Self {
            diagnostics,
            library_name,
        }
    }

    /// Create a diagnostic from a code and optional format values.
    ///
    /// # Arguments
    /// * `code` - The diagnostic code
    /// * `message_id` - Optional message ID (defaults to "default")
    /// * `format` - Optional format values for message interpolation
    ///
    /// # Panics
    /// Panics if the code or message_id is not found in the diagnostic map.
    pub fn create_diagnostic(
        &self,
        code: &str,
        message_id: Option<&str>,
        format: &[(String, String)],
    ) -> Diagnostic {
        let def = match self.diagnostics.get(code) {
            Some(d) => d,
            None => {
                let codes: String = self
                    .diagnostics
                    .keys()
                    .map(|c| format!(" - {}", c))
                    .collect::<Vec<_>>()
                    .join("\n");
                let error_msg = match &self.library_name {
                    Some(lib) => format!(
                        "Unexpected diagnostic code '{}'. It must match one of the code defined in the library '{}'. Defined codes:\n{}",
                        code, lib, codes
                    ),
                    None => format!(
                        "Unexpected diagnostic code '{}'. It must match one of the code defined in the compiler. Defined codes:\n{}",
                        code, codes
                    ),
                };
                panic!("{}", error_msg);
            }
        };

        let msg_id = message_id.unwrap_or("default");
        let message_text = def.get_message(msg_id).unwrap_or_else(|| {
            let msgs: String = def.messages.iter().map(|(id, _)| format!(" - {}", id)).collect::<Vec<_>>().join("\n");
            let error_msg = match &self.library_name {
                Some(lib) => format!("Unexpected message id '{}' for code '{}' in library '{}'. Defined messages:\n{}", msg_id, code, lib, msgs),
                None => format!("Unexpected message id '{}' for code '{}'. Defined messages:\n{}", msg_id, code, msgs),
            };
            panic!("{}", error_msg);
        });

        // Interpolate format values into the message
        let mut message = message_text.to_string();
        for (key, value) in format {
            message = message.replace(&format!("{{{}}}", key), value);
        }

        let full_code = match &self.library_name {
            Some(lib) => format!("{}/{}", lib, code),
            None => code.to_string(),
        };

        let mut diag = Diagnostic::new(&full_code, &message);
        diag.severity = def.severity;
        if let Some(url) = &def.url {
            diag.url = Some(url.clone());
        }
        diag
    }

    /// Get the diagnostic map
    pub fn diagnostics(&self) -> &DiagnosticMap {
        &self.diagnostics
    }

    /// Get the number of diagnostic definitions
    pub fn len(&self) -> usize {
        self.diagnostics.len()
    }

    /// Check if the diagnostic map is empty
    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }

    /// Get the library name
    pub fn library_name(&self) -> Option<&str> {
        self.library_name.as_deref()
    }
}

// ============================================================================
// Compiler Options
// Ported from TypeSpec compiler/src/core/options.ts
// ============================================================================

/// Compiler options for TypeSpec compilation.
/// Ported from TS CompilerOptions interface.
#[derive(Debug, Clone, Default)]
pub struct CompilerOptions {
    /// Miscellaneous options
    pub misc_options: std::collections::HashMap<String, String>,
    /// Default output directory used by emitters (default: ./tsp-output)
    pub output_dir: Option<String>,
    /// Path to config YAML file
    pub config: Option<String>,
    /// List of emitters to use
    pub emit: Option<Vec<String>>,
    /// List emitted outputs and their paths
    pub list_files: bool,
    /// Emitter options, keyed by emitter name
    pub options: std::collections::HashMap<String, std::collections::HashMap<String, String>>,
    /// Suppress all deprecated warnings
    pub ignore_deprecated: bool,
    /// Don't load the standard library
    pub nostdlib: bool,
    /// Do not run emitters (same as emit: [])
    pub no_emit: bool,
    /// Runs emitters but do not write output
    pub dry_run: bool,
    /// Additional imports
    pub additional_imports: Option<Vec<String>>,
    /// Treat warnings as errors
    pub warning_as_error: bool,
    /// Indicates compilation for live analysis in language server
    pub design_time_build: bool,
    /// Trace areas to enable
    pub trace: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_diagnostic_new() {
        let diag = Diagnostic::new("test-code", "Test message");
        assert_eq!(diag.code, "test-code");
        assert_eq!(diag.message, "Test message");
        assert_eq!(diag.severity, DiagnosticSeverity::Error);
    }

    #[test]
    fn test_diagnostic_error() {
        let diag = Diagnostic::error("err-code", "Error message");
        assert_eq!(diag.severity, DiagnosticSeverity::Error);
    }

    #[test]
    fn test_diagnostic_warning() {
        let diag = Diagnostic::warning("warn-code", "Warning message");
        assert_eq!(diag.severity, DiagnosticSeverity::Warning);
    }

    #[test]
    fn test_diagnostic_collector_new() {
        let collector = DiagnosticCollector::new();
        assert!(!collector.has_diagnostics());
    }

    #[test]
    fn test_diagnostic_collector_add() {
        let mut collector = DiagnosticCollector::new();
        collector.add(Diagnostic::error("code", "message"));
        assert!(collector.has_diagnostics());
        assert_eq!(collector.diagnostics().len(), 1);
    }

    #[test]
    fn test_diagnostic_collector_clear() {
        let mut collector = DiagnosticCollector::new();
        collector.add(Diagnostic::error("code", "message"));
        collector.clear();
        assert!(!collector.has_diagnostics());
    }

    #[test]
    fn test_diagnostic_collector_pipe() {
        let mut collector = DiagnosticCollector::new();
        let result = collector.pipe((42, vec![Diagnostic::error("code", "msg")]));
        assert_eq!(result, 42);
        assert_eq!(collector.diagnostics().len(), 1);
    }

    #[test]
    fn test_ignore_diagnostics() {
        let result = ignore_diagnostics((42, vec![Diagnostic::error("code", "msg")]));
        assert_eq!(result, 42);
    }

    #[test]
    fn test_source_location() {
        let loc = SourceLocation {
            file: "test.tsp".to_string(),
            start: 10,
            end: 20,
            is_synthetic: false,
        };
        assert_eq!(loc.file, "test.tsp");
        assert_eq!(loc.start, 10);
        assert_eq!(loc.end, 20);
    }

    // ============================================================================
    // DiagnosticSeverity Display tests
    // ============================================================================

    #[test]
    fn test_severity_display() {
        assert_eq!(format!("{}", DiagnosticSeverity::Error), "error");
        assert_eq!(format!("{}", DiagnosticSeverity::Warning), "warning");
        assert_eq!(format!("{}", DiagnosticSeverity::Information), "info");
        assert_eq!(format!("{}", DiagnosticSeverity::Hint), "hint");
    }

    // ============================================================================
    // DiagnosticDefinition tests
    // ============================================================================

    #[test]
    fn test_diagnostic_definition_error() {
        let def = DiagnosticDefinition::error("Something went wrong");
        assert_eq!(def.severity, DiagnosticSeverity::Error);
        assert_eq!(def.get_message("default"), Some("Something went wrong"));
        assert!(def.url.is_none());
    }

    #[test]
    fn test_diagnostic_definition_warning() {
        let def = DiagnosticDefinition::warning("Be careful");
        assert_eq!(def.severity, DiagnosticSeverity::Warning);
        assert_eq!(def.get_message("default"), Some("Be careful"));
    }

    #[test]
    fn test_diagnostic_definition_with_url() {
        let def = DiagnosticDefinition {
            severity: DiagnosticSeverity::Error,
            messages: vec![(
                "default".to_string(),
                DiagnosticMessageDefinition {
                    text: "Error".to_string(),
                },
            )],
            url: Some("https://example.com/docs".to_string()),
        };
        assert_eq!(def.url, Some("https://example.com/docs".to_string()));
    }

    #[test]
    fn test_diagnostic_definition_multiple_messages() {
        let def = DiagnosticDefinition {
            severity: DiagnosticSeverity::Error,
            messages: vec![
                (
                    "default".to_string(),
                    DiagnosticMessageDefinition {
                        text: "Default message".to_string(),
                    },
                ),
                (
                    "atPath".to_string(),
                    DiagnosticMessageDefinition {
                        text: "Error at path {path}".to_string(),
                    },
                ),
            ],
            url: None,
        };
        assert_eq!(def.get_message("default"), Some("Default message"));
        assert_eq!(def.get_message("atPath"), Some("Error at path {path}"));
        assert_eq!(def.get_message("nonexistent"), None);
    }

    // ============================================================================
    // DiagnosticCreator tests
    // ============================================================================

    #[test]
    fn test_diagnostic_creator_simple() {
        let creator = DiagnosticCreator::new(
            HashMap::from([(
                "invalid-argument".to_string(),
                DiagnosticDefinition::error("Invalid argument"),
            )]),
            None,
        );
        let diag = creator.create_diagnostic("invalid-argument", None, &[]);
        assert_eq!(diag.code, "invalid-argument");
        assert_eq!(diag.message, "Invalid argument");
        assert_eq!(diag.severity, DiagnosticSeverity::Error);
    }

    #[test]
    fn test_diagnostic_creator_with_library_name() {
        let creator = DiagnosticCreator::new(
            HashMap::from([(
                "invalid-argument".to_string(),
                DiagnosticDefinition::error("Invalid argument"),
            )]),
            Some("myLib".to_string()),
        );
        let diag = creator.create_diagnostic("invalid-argument", None, &[]);
        assert_eq!(diag.code, "myLib/invalid-argument");
    }

    #[test]
    fn test_diagnostic_creator_with_format() {
        let creator = DiagnosticCreator::new(
            HashMap::from([(
                "wrong-type".to_string(),
                DiagnosticDefinition::error("Expected {expected} but got {actual}"),
            )]),
            None,
        );
        let diag = creator.create_diagnostic(
            "wrong-type",
            None,
            &[
                ("expected".to_string(), "string".to_string()),
                ("actual".to_string(), "number".to_string()),
            ],
        );
        assert_eq!(diag.message, "Expected string but got number");
    }

    #[test]
    fn test_diagnostic_creator_warning() {
        let creator = DiagnosticCreator::new(
            HashMap::from([(
                "deprecated".to_string(),
                DiagnosticDefinition::warning("'{name}' is deprecated"),
            )]),
            None,
        );
        let diag = creator.create_diagnostic(
            "deprecated",
            None,
            &[("name".to_string(), "oldApi".to_string())],
        );
        assert_eq!(diag.severity, DiagnosticSeverity::Warning);
        assert_eq!(diag.message, "'oldApi' is deprecated");
    }

    #[test]
    fn test_diagnostic_creator_with_message_id() {
        let creator = DiagnosticCreator::new(
            HashMap::from([(
                "invalid-value".to_string(),
                DiagnosticDefinition {
                    severity: DiagnosticSeverity::Error,
                    messages: vec![
                        (
                            "default".to_string(),
                            DiagnosticMessageDefinition {
                                text: "Invalid value".to_string(),
                            },
                        ),
                        (
                            "atPath".to_string(),
                            DiagnosticMessageDefinition {
                                text: "Invalid value at path {path}".to_string(),
                            },
                        ),
                    ],
                    url: None,
                },
            )]),
            None,
        );
        let diag = creator.create_diagnostic(
            "invalid-value",
            Some("atPath"),
            &[("path".to_string(), "foo.bar".to_string())],
        );
        assert_eq!(diag.message, "Invalid value at path foo.bar");
    }

    #[test]
    #[should_panic(expected = "Unexpected diagnostic code")]
    fn test_diagnostic_creator_unknown_code() {
        let creator = DiagnosticCreator::new(
            HashMap::from([(
                "known-code".to_string(),
                DiagnosticDefinition::error("Known"),
            )]),
            None,
        );
        creator.create_diagnostic("unknown-code", None, &[]);
    }

    #[test]
    fn test_diagnostic_creator_accessors() {
        let creator = DiagnosticCreator::new(
            HashMap::from([("test".to_string(), DiagnosticDefinition::error("Test"))]),
            Some("myLib".to_string()),
        );
        assert_eq!(creator.library_name(), Some("myLib"));
        assert_eq!(creator.diagnostics().len(), 1);
    }

    // ============================================================================
    // CompilerOptions tests
    // ============================================================================

    #[test]
    fn test_compiler_options_default() {
        let opts = CompilerOptions::default();
        assert!(opts.output_dir.is_none());
        assert!(opts.emit.is_none());
        assert!(!opts.list_files);
        assert!(!opts.nostdlib);
        assert!(!opts.no_emit);
        assert!(!opts.dry_run);
        assert!(!opts.ignore_deprecated);
        assert!(!opts.warning_as_error);
        assert!(!opts.design_time_build);
    }

    #[test]
    fn test_compiler_options_with_values() {
        let opts = CompilerOptions {
            output_dir: Some("./tsp-output".to_string()),
            emit: Some(vec!["@typespec/openapi".to_string()]),
            nostdlib: true,
            warning_as_error: true,
            ..Default::default()
        };
        assert_eq!(opts.output_dir.as_deref(), Some("./tsp-output"));
        assert_eq!(opts.emit.as_ref().map(|v| v.len()), Some(1));
        assert!(opts.nostdlib);
        assert!(opts.warning_as_error);
    }

    #[test]
    fn test_compiler_options_additional_imports() {
        let opts = CompilerOptions {
            additional_imports: Some(vec!["../common/main.tsp".to_string()]),
            ..Default::default()
        };
        assert_eq!(opts.additional_imports.as_ref().map(|v| v.len()), Some(1));
    }

    #[test]
    fn test_compiler_options_trace() {
        let opts = CompilerOptions {
            trace: Some(vec!["binder".to_string(), "checker".to_string()]),
            ..Default::default()
        };
        assert_eq!(opts.trace.as_ref().map(|v| v.len()), Some(2));
    }

    // ============================================================================
    // get_related_locations / get_source_location / get_diagnostic_template_instantiation_trace tests
    // ============================================================================

    #[test]
    fn test_get_related_locations_no_location() {
        let diag = Diagnostic::error("test", "msg");
        let related = get_related_locations(&diag);
        assert!(related.is_empty());
    }

    #[test]
    fn test_get_source_location_with_location() {
        let mut diag = Diagnostic::error("test", "msg");
        diag.location = Some(SourceLocation {
            file: "test.tsp".to_string(),
            start: 0,
            end: 10,
            is_synthetic: false,
        });
        let loc = get_source_location(&diag);
        assert!(loc.is_some());
        assert_eq!(loc.unwrap().file, "test.tsp");
    }

    #[test]
    fn test_get_source_location_without_location() {
        let diag = Diagnostic::error("test", "msg");
        let loc = get_source_location(&diag);
        assert!(loc.is_none());
    }

    #[test]
    fn test_get_diagnostic_template_instantiation_trace() {
        let diag = Diagnostic::error("test", "msg");
        let trace = get_diagnostic_template_instantiation_trace(&diag);
        // Currently returns empty — will be populated when template system is complete
        assert!(trace.is_empty());
    }
}
