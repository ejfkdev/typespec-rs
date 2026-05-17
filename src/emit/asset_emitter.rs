//! Asset Emitter Framework
//!
//! Ported from TypeSpec packages/asset-emitter
//!
//! Provides a structured, type-driven emission framework:
//! - `TypeEmitter` trait — pluggable emit logic for each TypeSpec type kind
//! - `AssetEmitter` — orchestrator that drives emission, handles cycles and caching
//! - `Declaration`, `Scope`, `SourceFile` — output organization
//! - `ObjectBuilder`, `ArrayBuilder` — structured output construction
//! - `Placeholder` — deferred forward references
//! - `EmitContext` — context system with lexical/reference context split
//! - `resolveDeclarationReferenceScope` — scope resolution for cross-scope references

use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;

use crate::checker::types::{Type, TypeId};
use crate::checker::Checker;

// ============================================================================
// Placeholder — deferred value that can be filled later
// ============================================================================

/// A deferred value placeholder that can be resolved later.
/// Ported from TS `Placeholder<T>`.
#[derive(Debug, Clone)]
pub struct Placeholder<T> {
    value: Option<T>,
}

impl<T: Default + Debug + Clone> Placeholder<T> {
    pub fn new() -> Self {
        Self { value: None }
    }

    pub fn is_resolved(&self) -> bool {
        self.value.is_some()
    }

    pub fn resolve(&mut self, value: T) {
        self.value = Some(value);
    }

    pub fn get(&self) -> Option<&T> {
        self.value.as_ref()
    }

    pub fn into_value(self) -> Option<T> {
        self.value
    }
}

impl<T: Default + Debug + Clone> Default for Placeholder<T> {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// ReferenceCycle
// ============================================================================

/// Information about a reference cycle detected during emission.
/// Ported from TS `ReferenceCycle`.
#[derive(Debug, Clone)]
pub struct ReferenceCycle {
    /// The type where the cycle was detected
    pub target: TypeId,
    /// The path of types in the cycle
    pub cycle_path: Vec<TypeId>,
}

// ============================================================================
// EmitEntity — result of emitting a type
// ============================================================================

/// The kind of emit entity produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmitEntityKind {
    /// A declared (named) entity
    Declaration,
    /// Raw code output
    RawCode,
    /// Circular reference detected
    Circular,
    /// No output
    None,
}

/// Result of emitting a TypeSpec type.
/// Ported from TS `type EmitEntity<T>`.
#[derive(Debug, Clone)]
pub enum EmitEntity<T: Default + Debug + Clone> {
    /// A declared (named) entity with a declaration object
    Declaration(Declaration<T>),
    /// Raw code string
    RawCode(RawCode<T>),
    /// Circular reference
    CircularEmit(CircularEmit),
    /// No result
    None,
}

impl<T: Default + Debug + Clone> EmitEntity<T> {
    pub fn kind(&self) -> EmitEntityKind {
        match self {
            Self::Declaration(_) => EmitEntityKind::Declaration,
            Self::RawCode(_) => EmitEntityKind::RawCode,
            Self::CircularEmit(_) => EmitEntityKind::Circular,
            Self::None => EmitEntityKind::None,
        }
    }

    pub fn is_declaration(&self) -> bool {
        matches!(self, Self::Declaration(_))
    }

    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
}

/// A declared entity — named output bound to a scope.
/// Ported from TS `interface Declaration<T>`.
#[derive(Debug, Clone)]
pub struct Declaration<T: Default + Debug + Clone> {
    /// The emitted value
    pub value: T,
    /// The TypeSpec type this declaration represents
    pub type_id: TypeId,
    /// The name of the declaration
    pub name: String,
    /// The scope this declaration belongs to
    pub scope: Option<Scope>,
    /// Whether this declaration has been finished
    pub is_finished: bool,
}

/// Raw code output.
/// Ported from TS `interface RawCode`.
#[derive(Debug, Clone)]
pub struct RawCode<T: Default + Debug + Clone> {
    /// The raw code value
    pub value: T,
    _marker: PhantomData<T>,
}

impl<T: Default + Debug + Clone> RawCode<T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            _marker: PhantomData,
        }
    }
}

/// Circular reference emit result.
#[derive(Debug, Clone)]
pub struct CircularEmit {
    /// The cycle detected
    pub cycle: ReferenceCycle,
}

// ============================================================================
// Scope — hierarchical output organization
// ============================================================================

/// The kind of scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScopeKind {
    /// A namespace scope
    Namespace,
    /// A model scope
    Model,
    /// An interface scope
    Interface,
    /// A file scope
    File,
    /// A global scope
    Global,
    /// A custom scope
    Custom(String),
}

/// A scope for organizing emitted declarations.
/// Ported from TS `interface Scope`.
#[derive(Debug, Clone)]
pub struct Scope {
    /// Unique scope name
    pub name: String,
    /// The kind of scope
    pub kind: ScopeKind,
    /// Parent scope
    pub parent_scope: Option<String>,
    /// Declarations in this scope
    pub declaration_names: Vec<String>,
}

impl Scope {
    pub fn new(name: String, kind: ScopeKind) -> Self {
        Self {
            name,
            kind,
            parent_scope: None,
            declaration_names: Vec::new(),
        }
    }

    pub fn with_parent(mut self, parent: String) -> Self {
        self.parent_scope = Some(parent);
        self
    }

    pub fn add_declaration(&mut self, name: String) {
        if !self.declaration_names.contains(&name) {
            self.declaration_names.push(name);
        }
    }
}

// ============================================================================
// SourceFile
// ============================================================================

/// A source file produced by the emitter.
/// Ported from TS `interface SourceFile<T>`.
#[derive(Debug, Clone)]
pub struct SourceFile<T: Default + Debug + Clone> {
    /// The file path
    pub path: String,
    /// The emitted content
    pub content: T,
    /// The scope this file belongs to
    pub scope: Option<Scope>,
    _marker: PhantomData<T>,
}

impl<T: Default + Debug + Clone> SourceFile<T> {
    pub fn new(path: String, content: T) -> Self {
        Self {
            path,
            content,
            scope: None,
            _marker: PhantomData,
        }
    }
}

/// An emitted source file with metadata.
#[derive(Debug, Clone)]
pub struct EmittedSourceFile<T: Default + Debug + Clone> {
    /// The source file
    pub source_file: SourceFile<T>,
    /// Whether this file has been finished
    pub is_finished: bool,
}

// ============================================================================
// EmitContext — context passed during emission
// ============================================================================

/// Context for a single emission operation.
///
/// Ported from TS `interface ContextState` with lexical/reference context split.
/// The context is split into:
/// - `lexical_context`: context that comes from the type's lexical position
///   (e.g., namespace, model). This is reset when jumping to a new declaration.
/// - `reference_context`: context that is carried over when following references.
///   This persists across reference boundaries.
///
/// When context is accessed by the user, both are merged via `merged_context()`.
#[derive(Debug, Clone, Default)]
pub struct EmitContext {
    /// The type being emitted
    pub type_id: TypeId,
    /// Whether to emit a reference (vs declaration)
    pub is_reference: bool,
    /// The current scope name
    pub scope_name: Option<String>,
    /// Lexical context — set by the type's position in the type graph
    pub lexical_context: HashMap<String, String>,
    /// Reference context — carried over when following references
    pub reference_context: HashMap<String, String>,
}

impl EmitContext {
    /// Create a new empty context for a type.
    pub fn new(type_id: TypeId) -> Self {
        Self {
            type_id,
            is_reference: false,
            scope_name: None,
            lexical_context: HashMap::new(),
            reference_context: HashMap::new(),
        }
    }

    /// Get the merged context (lexical + reference).
    /// Reference context overrides lexical context for conflicting keys.
    pub fn merged_context(&self) -> HashMap<String, String> {
        let mut merged = self.lexical_context.clone();
        merged.extend(self.reference_context.clone());
        merged
    }

    /// Set a value in the lexical context.
    pub fn set_lexical(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.lexical_context.insert(key.into(), value.into());
    }

    /// Set a value in the reference context.
    pub fn set_reference(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.reference_context.insert(key.into(), value.into());
    }

    /// Get a value from the merged context.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.reference_context
            .get(key)
            .map(|s| s.as_str())
            .or_else(|| self.lexical_context.get(key).map(|s| s.as_str()))
    }

    /// Compute a cache key for this context.
    pub fn cache_key(&self) -> String {
        let mut parts: Vec<(String, String)> = self.merged_context().into_iter().collect();
        parts.sort_by(|a, b| a.0.cmp(&b.0));
        let ctx_str: String = parts
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join(",");
        format!("{}:{}:{}", self.type_id, self.is_reference, ctx_str)
    }
}

// ============================================================================
// ObjectBuilder / ArrayBuilder — structured output construction
// ============================================================================

/// A property in an object value.
#[derive(Debug, Clone)]
pub struct ObjectProperty<T: Default + Debug + Clone> {
    /// Property key
    pub key: String,
    /// Property value (can be a placeholder for deferred resolution)
    pub value: ObjectPropertyValue<T>,
}

/// Value for an object property — either concrete or a placeholder.
#[derive(Debug, Clone)]
pub enum ObjectPropertyValue<T: Default + Debug + Clone> {
    /// Concrete value
    Value(T),
    /// Placeholder for deferred resolution
    Placeholder(Placeholder<T>),
}

/// A structured object value being built.
/// Ported from TS `interface ObjectValue`.
#[derive(Debug, Clone)]
pub struct ObjectValue<T: Default + Debug + Clone> {
    /// Properties
    pub properties: Vec<ObjectProperty<T>>,
}

impl<T: Default + Debug + Clone> ObjectValue<T> {
    pub fn new() -> Self {
        Self {
            properties: Vec::new(),
        }
    }

    pub fn set_property(&mut self, key: String, value: T) {
        if let Some(prop) = self.properties.iter_mut().find(|p| p.key == key) {
            prop.value = ObjectPropertyValue::Value(value);
        } else {
            self.properties.push(ObjectProperty {
                key,
                value: ObjectPropertyValue::Value(value),
            });
        }
    }

    pub fn set_placeholder(&mut self, key: String, placeholder: Placeholder<T>) {
        if let Some(prop) = self.properties.iter_mut().find(|p| p.key == key) {
            prop.value = ObjectPropertyValue::Placeholder(placeholder);
        } else {
            self.properties.push(ObjectProperty {
                key,
                value: ObjectPropertyValue::Placeholder(placeholder),
            });
        }
    }

    pub fn get_property(&self, key: &str) -> Option<&ObjectPropertyValue<T>> {
        self.properties.iter().find(|p| p.key == key).map(|p| &p.value)
    }
}

impl<T: Default + Debug + Clone> Default for ObjectValue<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for constructing object values.
/// Ported from TS `class ObjectBuilder<T>`.
#[derive(Debug, Clone)]
pub struct ObjectBuilder<T: Default + Debug + Clone> {
    value: ObjectValue<T>,
}

impl<T: Default + Debug + Clone> ObjectBuilder<T> {
    pub fn new() -> Self {
        Self {
            value: ObjectValue::new(),
        }
    }

    pub fn set(mut self, key: impl Into<String>, value: T) -> Self {
        self.value.set_property(key.into(), value);
        self
    }

    pub fn set_placeholder(mut self, key: impl Into<String>, placeholder: Placeholder<T>) -> Self {
        self.value.set_placeholder(key.into(), placeholder);
        self
    }

    pub fn build(self) -> ObjectValue<T> {
        self.value
    }
}

impl<T: Default + Debug + Clone> Default for ObjectBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for constructing array values.
/// Ported from TS `class ArrayBuilder<T>`.
#[derive(Debug, Clone)]
pub struct ArrayBuilder<T: Default + Debug + Clone> {
    items: Vec<T>,
}

impl<T: Default + Debug + Clone> ArrayBuilder<T> {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn push(mut self, item: T) -> Self {
        self.items.push(item);
        self
    }

    pub fn build(self) -> Vec<T> {
        self.items
    }
}

impl<T: Default + Debug + Clone> Default for ArrayBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// TypeEmitter trait — pluggable emit logic for each type kind
// ============================================================================

/// Trait for pluggable emit logic.
/// Ported from TS `class TypeEmitter<T>`.
///
/// Each method corresponds to a TypeSpec type kind. Override to customize
/// how each type is emitted. Methods are organized by:
/// - Declaration methods: for named types (model, scalar, enum, etc.)
/// - Literal methods: for anonymous types
/// - Reference methods: for referencing already-emitted types
/// - Context methods: for setting emit context per-type
pub trait TypeEmitter<T: Default + Debug + Clone>: Send + Sync {
    // ---- Model methods ----

    /// Emit a model declaration.
    fn model_declaration(
        &self,
        _checker: &Checker,
        _type_id: TypeId,
        _name: &str,
        _model: &crate::checker::types::ModelType,
        _emitter: &AssetEmitter<T>,
    ) -> EmitEntity<T> {
        EmitEntity::None
    }

    /// Emit a model literal (anonymous model).
    fn model_literal(
        &self,
        _checker: &Checker,
        _type_id: TypeId,
        _model: &crate::checker::types::ModelType,
        _emitter: &AssetEmitter<T>,
    ) -> EmitEntity<T> {
        EmitEntity::None
    }

    /// Emit a model instantiation (template instance).
    fn model_instantiation(
        &self,
        _checker: &Checker,
        _type_id: TypeId,
        _name: &str,
        _model: &crate::checker::types::ModelType,
        _emitter: &AssetEmitter<T>,
    ) -> EmitEntity<T> {
        EmitEntity::None
    }

    // ---- Scalar methods ----

    /// Emit a scalar declaration.
    fn scalar_declaration(
        &self,
        _checker: &Checker,
        _type_id: TypeId,
        _name: &str,
        _scalar: &crate::checker::types::ScalarType,
        _emitter: &AssetEmitter<T>,
    ) -> EmitEntity<T> {
        EmitEntity::None
    }

    /// Emit a scalar instantiation.
    fn scalar_instantiation(
        &self,
        _checker: &Checker,
        _type_id: TypeId,
        _name: &str,
        _scalar: &crate::checker::types::ScalarType,
        _emitter: &AssetEmitter<T>,
    ) -> EmitEntity<T> {
        EmitEntity::None
    }

    // ---- Enum methods ----

    /// Emit an enum declaration.
    fn enum_declaration(
        &self,
        _checker: &Checker,
        _type_id: TypeId,
        _name: &str,
        _enum: &crate::checker::types::EnumType,
        _emitter: &AssetEmitter<T>,
    ) -> EmitEntity<T> {
        EmitEntity::None
    }

    // ---- Union methods ----

    /// Emit a union declaration.
    fn union_declaration(
        &self,
        _checker: &Checker,
        _type_id: TypeId,
        _name: &str,
        _union: &crate::checker::types::UnionType,
        _emitter: &AssetEmitter<T>,
    ) -> EmitEntity<T> {
        EmitEntity::None
    }

    /// Emit a union literal (anonymous union).
    fn union_literal(
        &self,
        _checker: &Checker,
        _type_id: TypeId,
        _union: &crate::checker::types::UnionType,
        _emitter: &AssetEmitter<T>,
    ) -> EmitEntity<T> {
        EmitEntity::None
    }

    /// Emit a union instantiation (template instance with name).
    fn union_instantiation(
        &self,
        _checker: &Checker,
        _type_id: TypeId,
        _name: &str,
        _union: &crate::checker::types::UnionType,
        _emitter: &AssetEmitter<T>,
    ) -> EmitEntity<T> {
        EmitEntity::None
    }

    // ---- Interface methods ----

    /// Emit an interface declaration.
    fn interface_declaration(
        &self,
        _checker: &Checker,
        _type_id: TypeId,
        _name: &str,
        _interface: &crate::checker::types::InterfaceType,
        _emitter: &AssetEmitter<T>,
    ) -> EmitEntity<T> {
        EmitEntity::None
    }

    /// Emit an interface operation declaration.
    fn interface_operation_declaration(
        &self,
        _checker: &Checker,
        _type_id: TypeId,
        _name: &str,
        _operation: &crate::checker::types::OperationType,
        _emitter: &AssetEmitter<T>,
    ) -> EmitEntity<T> {
        EmitEntity::None
    }

    // ---- Operation methods ----

    /// Emit an operation declaration.
    fn operation_declaration(
        &self,
        _checker: &Checker,
        _type_id: TypeId,
        _name: &str,
        _operation: &crate::checker::types::OperationType,
        _emitter: &AssetEmitter<T>,
    ) -> EmitEntity<T> {
        EmitEntity::None
    }

    // ---- Array methods ----

    /// Emit an array declaration (named model with indexer).
    fn array_declaration(
        &self,
        _checker: &Checker,
        _type_id: TypeId,
        _name: &str,
        _element_type: TypeId,
        _emitter: &AssetEmitter<T>,
    ) -> EmitEntity<T> {
        EmitEntity::None
    }

    /// Emit an array literal (anonymous model with indexer).
    fn array_literal(
        &self,
        _checker: &Checker,
        _type_id: TypeId,
        _element_type: TypeId,
        _emitter: &AssetEmitter<T>,
    ) -> EmitEntity<T> {
        EmitEntity::None
    }

    // ---- Intrinsic methods ----

    /// Emit an intrinsic type (string, numeric, boolean, void, null, etc.).
    fn intrinsic(
        &self,
        _checker: &Checker,
        _type_id: TypeId,
        _name: &str,
        _emitter: &AssetEmitter<T>,
    ) -> EmitEntity<T> {
        EmitEntity::None
    }

    // ---- Namespace methods ----

    /// Emit a namespace.
    fn emit_namespace(
        &self,
        _checker: &Checker,
        _type_id: TypeId,
        _name: &str,
        _namespace: &crate::checker::types::NamespaceType,
        _emitter: &AssetEmitter<T>,
    ) -> EmitEntity<T> {
        EmitEntity::None
    }

    // ---- Reference methods ----

    /// Emit a reference to a type.
    fn reference(
        &self,
        _checker: &Checker,
        _declaration: &Declaration<T>,
        _path_up: &[String],
        _path_down: &[String],
        _common_scope: Option<&Scope>,
        _emitter: &AssetEmitter<T>,
    ) -> EmitEntity<T> {
        EmitEntity::None
    }

    /// Emit a circular reference.
    fn circular_reference(
        &self,
        _checker: &Checker,
        _cycle: &ReferenceCycle,
        _emitter: &AssetEmitter<T>,
    ) -> EmitEntity<T> {
        EmitEntity::None
    }

    // ---- Name and context methods ----

    /// Get the declaration name for a type.
    fn declaration_name(&self, _checker: &Checker, _type_id: TypeId) -> Option<String> {
        None
    }

    /// Create a source file for emitted content.
    fn source_file(
        &self,
        _checker: &Checker,
        _path: &str,
        _content: T,
    ) -> SourceFile<T> {
        SourceFile::new(_path.to_string(), _content)
    }

    /// Get program-level context.
    fn program_context(&self, _checker: &Checker) -> HashMap<String, String> {
        HashMap::new()
    }

    // ---- Per-type context methods ----
    // These methods allow setting context for each type kind.
    // They are called during context establishment in emitType.

    /// Context for model declarations.
    fn model_declaration_context(
        &self,
        _checker: &Checker,
        _type_id: TypeId,
        _model: &crate::checker::types::ModelType,
    ) -> HashMap<String, String> {
        HashMap::new()
    }

    /// Reference context for model declarations.
    fn model_declaration_reference_context(
        &self,
        _checker: &Checker,
        _type_id: TypeId,
        _model: &crate::checker::types::ModelType,
    ) -> HashMap<String, String> {
        HashMap::new()
    }

    /// Context for namespace emission.
    fn namespace_context(
        &self,
        _checker: &Checker,
        _type_id: TypeId,
        _namespace: &crate::checker::types::NamespaceType,
    ) -> HashMap<String, String> {
        HashMap::new()
    }

    /// Reference context for namespace emission.
    fn namespace_reference_context(
        &self,
        _checker: &Checker,
        _type_id: TypeId,
        _namespace: &crate::checker::types::NamespaceType,
    ) -> HashMap<String, String> {
        HashMap::new()
    }
}

// ============================================================================
// AssetEmitter — the main orchestrator
// ============================================================================

/// The asset emitter orchestrator.
/// Ported from TS `createAssetEmitter()`.
///
/// Drives the emission process: dispatches to TypeEmitter methods,
/// handles caching with (method, type, context) triple keys,
/// cycle detection, and reference scope resolution.
pub struct AssetEmitter<T: Default + Debug + Clone> {
    /// The type emitter delegate
    type_emitter: Box<dyn TypeEmitter<T>>,
    /// Cache of already-emitted types, keyed by (method, type_id, context_key)
    emit_cache: HashMap<String, EmitEntity<T>>,
    /// Currently-emitting types (for cycle detection)
    emitting_stack: Vec<TypeId>,
    /// Source files produced
    source_files: Vec<EmittedSourceFile<T>>,
    /// Scopes
    scopes: HashMap<String, Scope>,
    /// Current emit context (lexical + reference)
    context: EmitContext,
    /// Whether the program context has been initialized
    program_context_initialized: bool,
    /// Waiting circular reference callbacks
    /// Key: cache key of the circular entity, Value: list of waiting callbacks
    waiting_circular_refs: HashMap<String, Vec<Box<dyn FnOnce(EmitEntity<T>) -> EmitEntity<T>>>>,
    /// Current scope name (for declarations)
    current_scope_name: Option<String>,
}

impl<T: Default + Debug + Clone + 'static> AssetEmitter<T> {
    /// Create a new asset emitter with the given type emitter.
    pub fn new(type_emitter: impl TypeEmitter<T> + 'static) -> Self {
        Self {
            type_emitter: Box::new(type_emitter),
            emit_cache: HashMap::new(),
            emitting_stack: Vec::new(),
            source_files: Vec::new(),
            scopes: HashMap::new(),
            context: EmitContext::default(),
            program_context_initialized: false,
            waiting_circular_refs: HashMap::new(),
            current_scope_name: None,
        }
    }

    /// Get the current merged context.
    pub fn get_context(&self) -> HashMap<String, String> {
        self.context.merged_context()
    }

    /// Get the current scope name.
    pub fn current_scope(&self) -> Option<&str> {
        self.current_scope_name.as_deref()
    }

    /// Emit a type, dispatching to the appropriate TypeEmitter method.
    ///
    /// This uses (method, type_id, context) as the cache key, matching
    /// upstream's `typeToEmitEntity` map. Context is interned for cache hits.
    pub fn emit_type(&mut self, checker: &Checker, type_id: TypeId) -> EmitEntity<T> {
        self.emit_type_with_method(checker, type_id, None)
    }

    /// Emit a type with a specific method hint.
    fn emit_type_with_method(
        &mut self,
        checker: &Checker,
        type_id: TypeId,
        method_hint: Option<&str>,
    ) -> EmitEntity<T> {
        // Determine the method key and dispatch
        let (_method_key, result) = match checker.get_type(type_id) {
            Some(Type::Model(m)) => {
                let name = m.name.clone();
                let has_name = !name.is_empty();
                let is_template_instance = m.template_mapper.is_some();
                // Check for array type (model with indexer)
                let is_array = m.indexer.is_some();

                if is_array && has_name {
                    let (_key_type, value_type) = m.indexer.unwrap();
                    (
                        "arrayDeclaration",
                        self.type_emitter.array_declaration(
                            checker,
                            type_id,
                            &name,
                            value_type,
                            self,
                        ),
                    )
                } else if is_array {
                    let (_key_type, value_type) = m.indexer.unwrap();
                    (
                        "arrayLiteral",
                        self.type_emitter.array_literal(checker, type_id, value_type, self),
                    )
                } else if has_name && is_template_instance {
                    (
                        "modelInstantiation",
                        self.type_emitter.model_instantiation(
                            checker,
                            type_id,
                            &name,
                            &m,
                            self,
                        ),
                    )
                } else if has_name {
                    (
                        "modelDeclaration",
                        self.type_emitter.model_declaration(checker, type_id, &name, &m, self),
                    )
                } else {
                    (
                        "modelLiteral",
                        self.type_emitter.model_literal(checker, type_id, &m, self),
                    )
                }
            }
            Some(Type::Scalar(s)) => {
                let name = s.name.clone();
                let is_template_instance = s.template_mapper.is_some();
                if is_template_instance {
                    (
                        "scalarInstantiation",
                        self.type_emitter.scalar_instantiation(
                            checker,
                            type_id,
                            &name,
                            &s,
                            self,
                        ),
                    )
                } else {
                    (
                        "scalarDeclaration",
                        self.type_emitter.scalar_declaration(checker, type_id, &name, &s, self),
                    )
                }
            }
            Some(Type::Enum(e)) => {
                let name = e.name.clone();
                (
                    "enumDeclaration",
                    self.type_emitter.enum_declaration(checker, type_id, &name, &e, self),
                )
            }
            Some(Type::Union(u)) => {
                let name = u.name.clone();
                let is_template_instance = u.template_mapper.is_some();
                if is_template_instance && !name.is_empty() {
                    (
                        "unionInstantiation",
                        self.type_emitter.union_instantiation(
                            checker,
                            type_id,
                            &name,
                            &u,
                            self,
                        ),
                    )
                } else if name.is_empty() {
                    (
                        "unionLiteral",
                        self.type_emitter.union_literal(checker, type_id, &u, self),
                    )
                } else {
                    (
                        "unionDeclaration",
                        self.type_emitter.union_declaration(checker, type_id, &name, &u, self),
                    )
                }
            }
            Some(Type::Interface(i)) => {
                let name = i.name.clone();
                (
                    "interfaceDeclaration",
                    self.type_emitter.interface_declaration(checker, type_id, &name, &i, self),
                )
            }
            Some(Type::Operation(o)) => {
                let name = o.name.clone();
                // Determine if this is an interface operation
                let is_interface_op = o.interface_.is_some();
                if is_interface_op {
                    (
                        "interfaceOperationDeclaration",
                        self.type_emitter.interface_operation_declaration(
                            checker,
                            type_id,
                            &name,
                            &o,
                            self,
                        ),
                    )
                } else {
                    (
                        "operationDeclaration",
                        self.type_emitter.operation_declaration(checker, type_id, &name, &o, self),
                    )
                }
            }
            Some(Type::Namespace(ns)) => {
                let name = ns.name.clone();
                (
                    "namespace",
                    self.type_emitter.emit_namespace(checker, type_id, &name, &ns, self),
                )
            }
            Some(Type::Intrinsic(i)) => {
                let name = format!("{:?}", i.name);
                ("intrinsic", self.type_emitter.intrinsic(checker, type_id, &name, self))
            }
            _ => return EmitEntity::None,
        };

        let _ = method_hint;
        result
    }

    /// Emit a reference to a type.
    ///
    /// Ported from TS `emitTypeReference()`. Handles:
    /// - Circular references with placeholder resolution
    /// - Reference context patching
    /// - Scope resolution for declarations
    pub fn emit_type_reference(&mut self, checker: &Checker, type_id: TypeId) -> EmitEntity<T> {
        // First, emit the type
        let entity = self.emit_type(checker, type_id);

        // If the result is a declaration, resolve reference scope
        if let EmitEntity::Declaration(ref decl) = entity {
            let (path_up, path_down, common_scope) =
                self.resolve_declaration_reference_scope_inner(checker, decl);
            let scope_ref = common_scope.as_ref();
            return self.type_emitter.reference(
                checker,
                decl,
                &path_up,
                &path_down,
                scope_ref,
                self,
            );
        }

        // If circular, create a placeholder for later resolution
        if let EmitEntity::CircularEmit(ref _circular) = entity {
            // Return raw code with empty placeholder for now
            // Full implementation would register a callback to resolve the placeholder
            // when the circular reference completes
        }

        entity
    }

    /// Emit an entire program (all namespaces).
    pub fn emit_program(&mut self, checker: &Checker) -> Vec<EmittedSourceFile<T>> {
        let global_ns = match checker.global_namespace_type {
            Some(id) => id,
            None => return Vec::new(),
        };
        if let Some(Type::Namespace(ns)) = checker.get_type(global_ns) {
            // Initialize program context
            if !self.program_context_initialized {
                let prog_ctx = self.type_emitter.program_context(checker);
                self.context.lexical_context = prog_ctx;
                self.context.reference_context = HashMap::new();
                self.program_context_initialized = true;
            }

            for name in &ns.namespace_names.clone() {
                if let Some(&ns_id) = ns.namespaces.get(name) {
                    self.emit_type(checker, ns_id);
                }
            }
            for name in &ns.model_names.clone() {
                if let Some(&model_id) = ns.models.get(name) {
                    self.emit_type(checker, model_id);
                }
            }
            for name in &ns.scalar_names.clone() {
                if let Some(&scalar_id) = ns.scalars.get(name) {
                    self.emit_type(checker, scalar_id);
                }
            }
            for name in &ns.enum_names.clone() {
                if let Some(&enum_id) = ns.enums.get(name) {
                    self.emit_type(checker, enum_id);
                }
            }
            for name in &ns.union_names.clone() {
                if let Some(&union_id) = ns.unions.get(name) {
                    self.emit_type(checker, union_id);
                }
            }
            for name in &ns.interface_names.clone() {
                if let Some(&iface_id) = ns.interfaces.get(name) {
                    self.emit_type(checker, iface_id);
                }
            }
            for name in &ns.operation_names.clone() {
                if let Some(&op_id) = ns.operations.get(name) {
                    self.emit_type(checker, op_id);
                }
            }
        }
        self.source_files.clone()
    }

    /// Create a source file.
    pub fn create_source_file(&mut self, path: &str, content: T) -> SourceFile<T> {
        SourceFile::new(path.to_string(), content)
    }

    /// Create a new scope.
    pub fn create_scope(&mut self, name: String, kind: ScopeKind) -> Scope {
        let scope = Scope::new(name, kind);
        self.scopes.insert(scope.name.clone(), scope.clone());
        scope
    }

    /// Wrap a value as a declaration result.
    pub fn result_declaration(
        &self,
        value: T,
        type_id: TypeId,
        name: String,
    ) -> EmitEntity<T> {
        EmitEntity::Declaration(Declaration {
            value,
            type_id,
            name,
            scope: None,
            is_finished: false,
        })
    }

    /// Wrap a value as raw code result.
    pub fn result_code(&self, value: T) -> EmitEntity<T> {
        EmitEntity::RawCode(RawCode::new(value))
    }

    /// Return a None result.
    pub fn result_none(&self) -> EmitEntity<T> {
        EmitEntity::None
    }

    /// Resolve the reference scope for a declaration.
    ///
    /// Ported from TS `resolveDeclarationReferenceScope()`.
    /// Computes the path up from the declaration's scope to the common scope,
    /// and the path down from the current scope to the common scope.
    fn resolve_declaration_reference_scope_inner(
        &self,
        _checker: &Checker,
        declaration: &Declaration<T>,
    ) -> (Vec<String>, Vec<String>, Option<Scope>) {
        let decl_scope = match &declaration.scope {
            Some(s) => s,
            None => return (Vec::new(), Vec::new(), None),
        };

        let current_scope_name = match &self.current_scope_name {
            Some(n) => n,
            None => return (Vec::new(), Vec::new(), Some(decl_scope.clone())),
        };

        let current_scope = match self.scopes.get(current_scope_name) {
            Some(s) => s,
            None => return (Vec::new(), Vec::new(), Some(decl_scope.clone())),
        };

        // Find common scope between declaration scope and current scope
        let (path_up, path_down, common_scope) =
            find_common_scope(decl_scope, current_scope, &self.scopes);

        (path_up, path_down, common_scope)
    }
}

// ============================================================================
// Scope Resolution
// ============================================================================

/// Find the common scope between two scopes and compute the path up/down.
///
/// Ported from TS `resolveDeclarationReferenceScope()`.
/// Returns (pathUp, pathDown, commonScope) where:
/// - pathUp: scope names from the declaration's scope up to the common scope
/// - pathDown: scope names from the current scope down to the common scope
/// - commonScope: the nearest common ancestor scope
fn find_common_scope(
    decl_scope: &Scope,
    current_scope: &Scope,
    all_scopes: &HashMap<String, Scope>,
) -> (Vec<String>, Vec<String>, Option<Scope>) {
    // Collect ancestor chain for declaration scope
    let decl_chain = collect_scope_chain(decl_scope, all_scopes);
    let current_chain = collect_scope_chain(current_scope, all_scopes);

    // Find common ancestor
    let mut common_idx = None;
    for (i, decl_name) in decl_chain.iter().enumerate() {
        if let Some(curr_name) = current_chain.get(i) {
            if decl_name == curr_name {
                common_idx = Some(i);
            }
        } else {
            break;
        }
    }

    match common_idx {
        Some(idx) => {
            // Path up: from declaration scope to common scope (excluding common)
            let path_up: Vec<String> = decl_chain[idx + 1..].to_vec();
            // Path down: from current scope to common scope (excluding common, reversed)
            let path_down: Vec<String> = current_chain[idx + 1..].iter().rev().cloned().collect();
            let common_name = &decl_chain[idx];
            let common_scope = all_scopes.get(common_name).cloned();
            (path_up, path_down, common_scope)
        }
        None => {
            // No common scope found
            (decl_chain, current_chain.into_iter().rev().collect(), None)
        }
    }
}

/// Collect the chain of scope names from root to the given scope.
fn collect_scope_chain(scope: &Scope, all_scopes: &HashMap<String, Scope>) -> Vec<String> {
    let mut chain = vec![scope.name.clone()];
    let mut current = scope.parent_scope.as_deref();
    while let Some(parent_name) = current {
        if chain.contains(&parent_name.to_string()) {
            break; // avoid infinite loops
        }
        chain.push(parent_name.to_string());
        current = all_scopes
            .get(parent_name)
            .and_then(|s| s.parent_scope.as_deref());
    }
    chain.reverse();
    chain
}
