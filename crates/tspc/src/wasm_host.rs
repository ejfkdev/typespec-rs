//! WASM extension host integration
//!
//! Loads WASM extensions that provide custom decorators and output formats.
//! Uses wasmtime as the WASM runtime.

use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

use anyhow::{anyhow, bail, Result};
use serde::Deserialize;
use wasmtime::*;

use typespec_rs::checker::types::TypeId;
use typespec_rs::state_accessors::StateAccessors;

// ============================================================================
// Data structures
// ============================================================================

/// Mutable host state shared between all host functions in a Store.
struct HostState {
    /// Shared reference to StateAccessors (Rc<RefCell<>> allows host functions to mutate).
    state: Rc<RefCell<StateAccessors>>,
    /// Diagnostics collected during extension execution.
    diagnostics: Vec<ExtensionDiagnostic>,
    /// Buffer for the last state_get result (guest reads via state_get_len + state_get_read).
    last_state_get_result: Option<String>,
}

/// A diagnostic reported by a WASM extension.
#[derive(Debug, Clone)]
pub struct ExtensionDiagnostic {
    pub severity: i32,
    pub code: String,
    pub message: String,
}

/// Extension manifest deserialized from JSON.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ExtensionManifest {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub formats: Vec<String>,
    #[serde(default)]
    pub decorators: Vec<DecoratorDefinition>,
}

/// A decorator declared by an extension.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct DecoratorDefinition {
    pub name: String,
    #[serde(default)]
    pub namespace: String,
    #[serde(default)]
    pub target_types: Vec<String>,
    #[serde(default)]
    pub parameters: Vec<DecoratorParameter>,
}

/// A parameter of a decorator declaration.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct DecoratorParameter {
    pub name: String,
    #[serde(rename = "type", default)]
    pub param_type: String,
}

/// A loaded WASM extension with its manifest and wasmtime handles.
pub struct WasmExtension {
    pub manifest: ExtensionManifest,
    instance: Instance,
    store: Store<HostState>,
    func_allocate: TypedFunc<i32, i32>,
    func_deallocate: TypedFunc<(i32, i32), ()>,
    func_manifest: TypedFunc<(), i32>,
    func_manifest_len: TypedFunc<(), i32>,
    func_init: TypedFunc<(i32, i32), i32>,
    func_handle_decorator: TypedFunc<(i32, i32), i32>,
    func_emit: TypedFunc<(i32, i32), i32>,
    func_emit_len: TypedFunc<(), i32>,
}

/// WASM host that manages extension loading and execution.
pub struct WasmHost {
    engine: Engine,
}

impl WasmHost {
    /// Create a new WASM host.
    pub fn new() -> Result<Self> {
        let engine = Engine::default();
        Ok(Self { engine })
    }

    /// Extract only the manifest from a .wasm file, without full initialization.
    /// Used during the pre-parse phase to register decorator declarations.
    pub fn extract_manifest(&mut self, path: &Path) -> Result<ExtensionManifest> {
        let wasm_bytes = std::fs::read(path)
            .map_err(|e| anyhow!("Failed to read '{}': {}", path.display(), e))?;
        let module = Module::new(&self.engine, &wasm_bytes)?;

        let host_state = HostState {
            state: Rc::new(RefCell::new(StateAccessors::new())),
            diagnostics: Vec::new(),
            last_state_get_result: None,
        };
        let mut store = Store::new(&self.engine, host_state);
        let linker = Self::create_linker(&self.engine)?;
        let instance = linker.instantiate(&mut store, &module)?;

        let func_manifest = instance
            .get_typed_func::<(), i32>(&mut store, "tsp_ext_manifest")
            .map_err(|_| anyhow!("guest must export 'tsp_ext_manifest'"))?;
        let func_manifest_len = instance
            .get_typed_func::<(), i32>(&mut store, "tsp_ext_manifest_len")
            .map_err(|_| anyhow!("guest must export 'tsp_ext_manifest_len'"))?;

        let ptr = func_manifest.call(&mut store, ())?;
        let len = func_manifest_len.call(&mut store, ())?;

        let memory = instance
            .get_memory(&mut store, "memory")
            .ok_or_else(|| anyhow!("guest must export 'memory'"))?;
        let json = read_guest_string(&memory, &store, ptr, len)?;

        serde_json::from_str(&json).map_err(Into::into)
    }

    /// Load a WASM extension from a file.
    pub fn load_extension(
        &mut self,
        path: &Path,
        state: Rc<RefCell<StateAccessors>>,
    ) -> Result<WasmExtension> {
        let wasm_bytes = std::fs::read(path)
            .map_err(|e| anyhow!("Failed to read '{}': {}", path.display(), e))?;
        let module = Module::new(&self.engine, &wasm_bytes)?;

        let host_state = HostState {
            state: state.clone(),
            diagnostics: Vec::new(),
            last_state_get_result: None,
        };
        let mut store = Store::new(&self.engine, host_state);
        let linker = Self::create_linker(&self.engine)?;
        let instance = linker.instantiate(&mut store, &module)?;

        // Cache required guest exports
        let func_allocate = instance
            .get_typed_func::<i32, i32>(&mut store, "allocate")
            .map_err(|_| anyhow!("guest must export 'allocate'"))?;
        let func_deallocate = instance
            .get_typed_func::<(i32, i32), ()>(&mut store, "deallocate")
            .map_err(|_| anyhow!("guest must export 'deallocate'"))?;
        let func_manifest = instance
            .get_typed_func::<(), i32>(&mut store, "tsp_ext_manifest")
            .map_err(|_| anyhow!("guest must export 'tsp_ext_manifest'"))?;
        let func_manifest_len = instance
            .get_typed_func::<(), i32>(&mut store, "tsp_ext_manifest_len")
            .map_err(|_| anyhow!("guest must export 'tsp_ext_manifest_len'"))?;
        let func_init = instance
            .get_typed_func::<(i32, i32), i32>(&mut store, "tsp_ext_init")
            .map_err(|_| anyhow!("guest must export 'tsp_ext_init'"))?;
        let func_handle_decorator = instance
            .get_typed_func::<(i32, i32), i32>(&mut store, "tsp_ext_handle_decorator")
            .map_err(|_| anyhow!("guest must export 'tsp_ext_handle_decorator'"))?;
        let func_emit = instance
            .get_typed_func::<(i32, i32), i32>(&mut store, "tsp_ext_emit")
            .map_err(|_| anyhow!("guest must export 'tsp_ext_emit'"))?;
        let func_emit_len = instance
            .get_typed_func::<(), i32>(&mut store, "tsp_ext_emit_len")
            .map_err(|_| anyhow!("guest must export 'tsp_ext_emit_len'"))?;

        let mut ext = WasmExtension {
            manifest: ExtensionManifest {
                name: String::new(),
                version: String::new(),
                formats: Vec::new(),
                decorators: Vec::new(),
            },
            instance,
            store,
            func_allocate,
            func_deallocate,
            func_manifest,
            func_manifest_len,
            func_init,
            func_handle_decorator,
            func_emit,
            func_emit_len,
        };

        // Read manifest
        let ptr = ext.func_manifest.call(&mut ext.store, ())?;
        let len = ext.func_manifest_len.call(&mut ext.store, ())?;
        let memory = ext
            .instance
            .get_memory(&mut ext.store, "memory")
            .ok_or_else(|| anyhow!("guest must export 'memory'"))?;
        let json = read_guest_string(&memory, &ext.store, ptr, len)?;
        ext.manifest = serde_json::from_str(&json)?;

        // Call tsp_ext_init with empty options
        let (opts_ptr, opts_len) = ext.write_guest_string("{}")?;
        let init_result = ext.func_init.call(&mut ext.store, (opts_ptr, opts_len))?;
        ext.free_guest_string(opts_ptr, opts_len)?;
        if init_result != 0 {
            bail!("tsp_ext_init returned error code: {}", init_result);
        }

        Ok(ext)
    }

    /// Create a Linker with all host function imports.
    fn create_linker(engine: &Engine) -> Result<Linker<HostState>> {
        let mut linker = Linker::new(engine);

        // tsp.log(ptr, len)
        linker.func_wrap("tsp", "log", host_log)?;

        // tsp.state_set(key_ptr, key_len, type_id, val_ptr, val_len)
        linker.func_wrap("tsp", "state_set", host_state_set)?;

        // tsp.state_get(key_ptr, key_len, type_id) -> i32
        linker.func_wrap("tsp", "state_get", host_state_get)?;

        // tsp.state_get_read(buf_ptr)
        linker.func_wrap("tsp", "state_get_read", host_state_get_read)?;

        // tsp.state_add(key_ptr, key_len, type_id)
        linker.func_wrap("tsp", "state_add", host_state_add)?;

        // tsp.state_has(key_ptr, key_len, type_id) -> i32
        linker.func_wrap("tsp", "state_has", host_state_has)?;

        // tsp.report_diagnostic(severity, code_ptr, code_len, msg_ptr, msg_len)
        linker.func_wrap("tsp", "report_diagnostic", host_report_diagnostic)?;

        Ok(linker)
    }
}

// ============================================================================
// Host function implementations
// ============================================================================

fn host_log(mut caller: Caller<'_, HostState>, ptr: i32, len: i32) {
    if let Some(msg) = read_caller_string(&mut caller, ptr, len) {
        eprintln!("[wasm-ext] {}", msg);
    }
}

fn host_state_set(
    mut caller: Caller<'_, HostState>,
    key_ptr: i32,
    key_len: i32,
    type_id: i32,
    val_ptr: i32,
    val_len: i32,
) {
    let key = read_caller_string(&mut caller, key_ptr, key_len).unwrap_or_default();
    let val = read_caller_string(&mut caller, val_ptr, val_len).unwrap_or_default();
    caller
        .data_mut()
        .state
        .borrow_mut()
        .set_state(&key, type_id as TypeId, val);
}

fn host_state_get(
    mut caller: Caller<'_, HostState>,
    key_ptr: i32,
    key_len: i32,
    type_id: i32,
) -> i32 {
    let key = read_caller_string(&mut caller, key_ptr, key_len).unwrap_or_default();
    let result = caller
        .data()
        .state
        .borrow()
        .get_state(&key, type_id as TypeId)
        .map(|s| s.to_string());
    let len = match &result {
        Some(s) => s.len() as i32,
        None => -1,
    };
    caller.data_mut().last_state_get_result = result;
    len
}

fn host_state_get_read(mut caller: Caller<'_, HostState>, buf_ptr: i32) {
    let val = caller.data().last_state_get_result.clone();
    if let Some(ref val) = val {
        let memory = caller.get_export("memory").and_then(|e| e.into_memory());
        if let Some(memory) = memory {
            let data = memory.data_mut(&mut caller);
            let start = buf_ptr as usize;
            let end = start + val.len();
            if end <= data.len() {
                data[start..end].copy_from_slice(val.as_bytes());
            }
        }
    }
}

fn host_state_add(
    mut caller: Caller<'_, HostState>,
    key_ptr: i32,
    key_len: i32,
    type_id: i32,
) {
    let key = read_caller_string(&mut caller, key_ptr, key_len).unwrap_or_default();
    caller
        .data_mut()
        .state
        .borrow_mut()
        .add_to_state(&key, type_id as TypeId);
}

fn host_state_has(
    mut caller: Caller<'_, HostState>,
    key_ptr: i32,
    key_len: i32,
    type_id: i32,
) -> i32 {
    let key = read_caller_string(&mut caller, key_ptr, key_len).unwrap_or_default();
    if caller
        .data()
        .state
        .borrow()
        .has_state(&key, type_id as TypeId)
    {
        1
    } else {
        0
    }
}

fn host_report_diagnostic(
    mut caller: Caller<'_, HostState>,
    severity: i32,
    code_ptr: i32,
    code_len: i32,
    msg_ptr: i32,
    msg_len: i32,
) {
    let code = read_caller_string(&mut caller, code_ptr, code_len).unwrap_or_default();
    let message = read_caller_string(&mut caller, msg_ptr, msg_len).unwrap_or_default();
    caller.data_mut().diagnostics.push(ExtensionDiagnostic {
        severity,
        code,
        message,
    });
}

// ============================================================================
// Memory helpers
// ============================================================================

/// Read a string from guest memory at (ptr, len).
fn read_guest_string(
    memory: &Memory,
    store: &Store<HostState>,
    ptr: i32,
    len: i32,
) -> Result<String> {
    let data = memory.data(store);
    let start = ptr as usize;
    let end = start + len as usize;
    if end > data.len() {
        bail!(
            "string read out of bounds: ptr={}, len={}, mem_len={}",
            ptr,
            len,
            data.len()
        );
    }
    let s = std::str::from_utf8(&data[start..end])?.to_string();
    Ok(s)
}

/// Read a string from a Caller's guest memory at (ptr, len).
fn read_caller_string(caller: &mut Caller<'_, HostState>, ptr: i32, len: i32) -> Option<String> {
    let memory = caller.get_export("memory").and_then(|e| e.into_memory())?;
    let data = memory.data(caller);
    let start = ptr as usize;
    let end = start + len as usize;
    if end > data.len() {
        return None;
    }
    std::str::from_utf8(&data[start..end]).ok().map(|s| s.to_string())
}

impl WasmExtension {
    /// Write a string into guest memory by calling guest's `allocate`, then copying.
    fn write_guest_string(&mut self, s: &str) -> Result<(i32, i32)> {
        let len = s.len() as i32;
        let ptr = self.func_allocate.call(&mut self.store, len)?;
        let memory = self
            .instance
            .get_memory(&mut self.store, "memory")
            .ok_or_else(|| anyhow!("guest must export 'memory'"))?;
        let data = memory.data_mut(&mut self.store);
        let start = ptr as usize;
        let end = start + s.len();
        if end > data.len() {
            bail!("string write out of bounds");
        }
        data[start..end].copy_from_slice(s.as_bytes());
        Ok((ptr, len))
    }

    /// Free a guest-allocated string.
    fn free_guest_string(&mut self, ptr: i32, len: i32) -> Result<()> {
        self.func_deallocate.call(&mut self.store, (ptr, len))?;
        Ok(())
    }

    /// Handle a decorator application on a type.
    /// `input_json` is `{ "type_id": N, "decorator_name": "...", "args": [...] }`
    pub fn handle_decorator(&mut self, input_json: &str) -> Result<i32> {
        let (ptr, len) = self.write_guest_string(input_json)?;
        let result = self.func_handle_decorator.call(&mut self.store, (ptr, len))?;
        self.free_guest_string(ptr, len)?;
        Ok(result)
    }

    /// Emit output for the given format using this extension.
    pub fn emit(&mut self, graph_json: &str) -> Result<String> {
        let (ptr, len) = self.write_guest_string(graph_json)?;
        let result_ptr = self.func_emit.call(&mut self.store, (ptr, len))?;
        self.free_guest_string(ptr, len)?;

        if result_ptr == 0 {
            bail!("tsp_ext_emit returned 0 (failure)");
        }

        let result_len = self.func_emit_len.call(&mut self.store, ())?;
        let memory = self
            .instance
            .get_memory(&mut self.store, "memory")
            .ok_or_else(|| anyhow!("guest must export 'memory'"))?;
        let output = read_guest_string(&memory, &self.store, result_ptr, result_len)?;
        Ok(output)
    }

    /// Take the diagnostics collected during extension execution.
    pub fn take_diagnostics(&mut self) -> Vec<ExtensionDiagnostic> {
        std::mem::take(&mut self.store.data_mut().diagnostics)
    }
}
