//! Compiler statistics
//!
//! Ported from TypeSpec compiler/src/core/stats.ts
//!
//! Provides data structures for tracking compilation complexity and runtime metrics.

use std::collections::HashMap;

/// Overall compiler statistics
#[derive(Debug, Clone, Default)]
pub struct Stats {
    /// Type complexity statistics
    pub complexity: ComplexityStats,
    /// Runtime performance statistics
    pub runtime: RuntimeStats,
}

/// Type complexity statistics
#[derive(Debug, Clone, Default)]
pub struct ComplexityStats {
    /// Number of types created
    pub created_types: usize,
    /// Number of types that have been finished (fully resolved)
    pub finished_types: usize,
}

/// Runtime performance statistics
#[derive(Debug, Clone, Default)]
pub struct RuntimeStats {
    /// Total compilation time in microseconds
    pub total: u64,
    /// Time spent in the loader phase (microseconds)
    pub loader: u64,
    /// Time spent in the resolver phase (microseconds)
    pub resolver: u64,
    /// Time spent in the checker phase (microseconds)
    pub checker: u64,
    /// Validation statistics
    pub validation: ValidationStats,
    /// Linter statistics
    pub linter: LinterRunStats,
    /// Emitter statistics
    pub emit: EmitStats,
}

/// Validation runtime statistics
#[derive(Debug, Clone, Default)]
pub struct ValidationStats {
    /// Total validation time in microseconds
    pub total: u64,
    /// Per-validator time in microseconds
    pub validators: HashMap<String, u64>,
}

/// Linter runtime statistics
#[derive(Debug, Clone, Default)]
pub struct LinterRunStats {
    /// Total linter time in microseconds
    pub total: u64,
    /// Per-rule time in microseconds
    pub rules: HashMap<String, u64>,
}

/// Emitter runtime statistics
#[derive(Debug, Clone, Default)]
pub struct EmitStats {
    /// Total emit time in microseconds
    pub total: u64,
    /// Per-emitter statistics
    pub emitters: HashMap<String, EmitterRunStats>,
}

/// Per-emitter runtime statistics
#[derive(Debug, Clone, Default)]
pub struct EmitterRunStats {
    /// Total time for this emitter in microseconds
    pub total: u64,
    /// Per-step time in microseconds
    pub steps: HashMap<String, u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_default() {
        let stats = Stats::default();
        assert_eq!(stats.complexity.created_types, 0);
        assert_eq!(stats.complexity.finished_types, 0);
        assert_eq!(stats.runtime.total, 0);
    }

    #[test]
    fn test_complexity_stats() {
        let stats = ComplexityStats {
            created_types: 10,
            finished_types: 8,
        };
        assert_eq!(stats.created_types, 10);
        assert_eq!(stats.finished_types, 8);
    }

    #[test]
    fn test_runtime_stats_with_validation() {
        let mut stats = RuntimeStats {
            total: 1000,
            checker: 500,
            ..Default::default()
        };
        stats.validation.total = 200;
        stats
            .validation
            .validators
            .insert("myValidator".to_string(), 200);
        assert_eq!(stats.checker, 500);
        assert_eq!(stats.validation.validators["myValidator"], 200);
    }

    #[test]
    fn test_linter_run_stats() {
        let mut stats = LinterRunStats {
            total: 300,
            ..Default::default()
        };
        stats.rules.insert("unused-using".to_string(), 150);
        stats
            .rules
            .insert("unused-template-parameter".to_string(), 150);
        assert_eq!(stats.rules.len(), 2);
    }

    #[test]
    fn test_emit_stats() {
        let mut stats = EmitStats {
            total: 400,
            ..Default::default()
        };
        let mut emitter = EmitterRunStats {
            total: 400,
            ..Default::default()
        };
        emitter.steps.insert("emit-types".to_string(), 200);
        emitter.steps.insert("emit-operations".to_string(), 200);
        stats.emitters.insert("my-emitter".to_string(), emitter);
        assert_eq!(stats.emitters["my-emitter"].steps.len(), 2);
    }
}
