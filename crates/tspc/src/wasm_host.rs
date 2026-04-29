//! WASM extension host integration
//!
//! Loads WASM extensions that provide custom decorators and output formats.
//! Uses wasmtime as the WASM runtime.

use std::path::Path;
use typespec_rs::checker::Checker;

/// A loaded WASM extension with its manifest
pub struct WasmExtension {
    /// Extension name
    pub name: String,
    /// Extension version
    pub version: String,
    /// Output formats this extension provides
    pub formats: Vec<String>,
    /// Decorator names this extension registers
    pub decorator_names: Vec<String>,
}

/// WASM host that manages extension loading and execution
pub struct WasmHost<'a> {
    _checker: &'a Checker,
}

impl<'a> WasmHost<'a> {
    /// Create a new WASM host
    pub fn new(checker: &'a Checker) -> Result<Self, String> {
        Ok(Self { _checker: checker })
    }

    /// Load a WASM extension from a file
    pub fn load_extension(&mut self, _path: &Path) -> Result<WasmExtension, String> {
        // TODO: Implement with wasmtime
        // 1. Read .wasm file
        // 2. Compile and instantiate with wasmtime
        // 3. Call tsp_ext_manifest to get extension info
        // 4. Call tsp_ext_init to initialize
        Err("WASM extension loading not yet implemented".to_string())
    }

    /// Emit output using a WASM extension
    pub fn emit(
        &mut self,
        _extension: &WasmExtension,
        _graph_json: &str,
    ) -> Result<String, String> {
        // TODO: Implement with wasmtime
        // 1. Call tsp_ext_emit with the type graph JSON
        // 2. Read the output string from guest memory
        Err("WASM emission not yet implemented".to_string())
    }
}
