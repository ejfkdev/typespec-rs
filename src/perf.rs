//! Performance timing utilities for TypeSpec-Rust
//!
//! Ported from TypeSpec compiler/src/core/perf.ts
//!
//! This module provides performance measurement utilities.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Instant;

/// A timer that can be stopped to get elapsed time
#[derive(Debug, Clone)]
pub struct Timer {
    start: Instant,
}

impl Timer {
    /// Create a new timer that starts immediately
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    /// End the timer and return the elapsed time in milliseconds
    pub fn end(&self) -> f64 {
        let elapsed = self.start.elapsed();
        elapsed.as_secs_f64() * 1000.0
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}

/// Start a new timer
pub fn start_timer() -> Timer {
    Timer::new()
}

/// Time a synchronous function and return both the elapsed time and the function result
pub fn time<F, T>(f: F) -> (f64, T)
where
    F: FnOnce() -> T,
{
    let timer = start_timer();
    let result = f();
    let elapsed = timer.end();
    (elapsed, result)
}

/// Time an asynchronous function and return the elapsed time in milliseconds
/// Note: This is a synchronous wrapper for async functions
pub async fn time_async<F, Fut>(f: F) -> f64
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    let timer = start_timer();
    f().await;
    timer.end()
}

/// A timer that reports to a reporter when ended
#[derive(Debug)]
pub struct LabeledTimer {
    label: String,
    start: Instant,
    measures: Rc<RefCell<HashMap<String, f64>>>,
}

impl LabeledTimer {
    fn end(&self) -> f64 {
        let elapsed = self.start.elapsed();
        let ms = elapsed.as_secs_f64() * 1000.0;
        self.measures.borrow_mut().insert(self.label.clone(), ms);
        ms
    }
}

/// Performance reporter that tracks timing measurements
#[derive(Debug)]
pub struct PerfReporter {
    measures: Rc<RefCell<HashMap<String, f64>>>,
}

impl PerfReporter {
    /// Create a new performance reporter
    pub fn new() -> Self {
        Self {
            measures: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    /// Start a reporting timer with a label
    pub fn start_timer(&self) -> LabeledTimer {
        LabeledTimer {
            label: String::new(),
            start: Instant::now(),
            measures: Rc::clone(&self.measures),
        }
    }

    /// Start a reporting timer with a label
    pub fn start_timer_with_label(&self, label: &str) -> LabeledTimer {
        LabeledTimer {
            label: label.to_string(),
            start: Instant::now(),
            measures: Rc::clone(&self.measures),
        }
    }

    /// Time a synchronous function with a label
    pub fn time<T, F>(&self, label: &str, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        let timer = self.start_timer_with_label(label);
        let result = f();
        timer.end();
        result
    }

    /// Time an async function with a label
    pub async fn time_async<T, F, Fut>(&self, label: &str, f: F) -> T
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = T>,
    {
        let timer = self.start_timer_with_label(label);
        let result = f().await;
        timer.end();
        result
    }

    /// Report a manual measurement
    pub fn report(&self, label: &str, duration: f64) {
        self.measures
            .borrow_mut()
            .insert(label.to_string(), duration);
    }

    /// Get all recorded measures
    pub fn measures(&self) -> HashMap<String, f64> {
        self.measures.borrow().clone()
    }

    /// Get a specific measure
    pub fn get_measure(&self, label: &str) -> Option<f64> {
        self.measures.borrow().get(label).copied()
    }
}

impl Default for PerfReporter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer_basic() {
        let timer = Timer::new();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let elapsed = timer.end();
        assert!(elapsed >= 9.0); // Allow some tolerance
    }

    #[test]
    fn test_timer_default() {
        let _timer = Timer::default();
        let timer = Timer::new();
        assert!(timer.end() >= 0.0);
    }

    #[test]
    fn test_start_timer() {
        let timer = start_timer();
        let elapsed = timer.end();
        assert!(elapsed >= 0.0);
    }

    #[test]
    fn test_time_function() {
        let (elapsed, ()) = time(|| {
            std::thread::sleep(std::time::Duration::from_millis(5));
        });
        assert!(elapsed >= 4.0);
    }

    #[test]
    fn test_time_function_returns_value() {
        let (_elapsed, result) = time(|| 42);
        assert_eq!(result, 42);
    }

    #[test]
    fn test_perf_reporter_new() {
        let reporter = PerfReporter::new();
        assert!(reporter.measures().is_empty());
    }

    #[test]
    fn test_perf_reporter_time() {
        let reporter = PerfReporter::new();
        let result = reporter.time("test", || {
            std::thread::sleep(std::time::Duration::from_millis(5));
            42
        });
        assert_eq!(result, 42);
        assert!(reporter.get_measure("test").is_some());
    }

    #[test]
    fn test_perf_reporter_report() {
        let reporter = PerfReporter::new();
        reporter.report("manual", 123.45);
        assert_eq!(reporter.get_measure("manual"), Some(123.45));
    }

    #[test]
    fn test_perf_reporter_measures() {
        let reporter = PerfReporter::new();
        reporter.report("a", 1.0);
        reporter.report("b", 2.0);
        let measures = reporter.measures();
        assert_eq!(measures.len(), 2);
        assert_eq!(measures.get("a"), Some(&1.0));
        assert_eq!(measures.get("b"), Some(&2.0));
    }

    #[test]
    fn test_perf_module_functions() {
        let (elapsed, ()) = time(|| {
            std::thread::sleep(std::time::Duration::from_millis(5));
        });
        assert!(elapsed >= 4.0);
    }
}
