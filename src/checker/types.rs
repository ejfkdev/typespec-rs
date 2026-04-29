//! Checker Type System
//! Ported from TypeSpec compiler/src/core/types.ts
//!
//! Defines the Type enum, TypeId, and TypeStore that form the core
//! of the checker's type representation.

use crate::ast::node::NodeId;
use std::collections::HashMap;

/// Type mapper for template instantiation.
/// Ported from TS compiler TypeMapper interface.
///
/// Maps template parameters to their concrete types when a template is instantiated.
/// Template declarations have `template_mapper = None`; template instances have `Some`.
#[derive(Debug, Clone)]
pub struct TypeMapper {
    /// Whether this is a partial mapping
    pub partial: bool,
    /// The mapping from template parameter NodeIds to TypeIds
    pub map: HashMap<NodeId, TypeId>,
    /// Arguments used for instantiation (TypeIds)
    pub args: Vec<TypeId>,
    /// Source node used to create this mapper
    pub source_node: Option<NodeId>,
    /// Parent mapper if any
    pub parent_mapper: Option<Box<TypeMapper>>,
}

impl TypeMapper {
    pub fn new() -> Self {
        Self {
            partial: false,
            map: HashMap::new(),
            args: Vec::new(),
            source_node: None,
            parent_mapper: None,
        }
    }
}

impl Default for TypeMapper {
    fn default() -> Self {
        Self::new()
    }
}

/// Type ID - unique identifier for a type in the type store
pub type TypeId = u32;

/// Reserved TypeId for invalid/uninitialized type
pub const INVALID_TYPE_ID: TypeId = 0;

/// Result of inferring a scalar type for a primitive value.
/// Ported from TS checker.ts inferScalarForPrimitiveValue.
/// When multiple scalars match the same primitive value (e.g., int32 | int64),
/// `ambiguous` contains the diagnostic info and `scalar` is None.
#[derive(Debug, Clone)]
pub struct InferredScalar {
    /// The inferred scalar TypeId, if unambiguous
    pub scalar: Option<TypeId>,
    /// If ambiguous: (value_name, ambiguous_type_names, example_name)
    pub ambiguous: Option<(String, String, String)>,
}

impl InferredScalar {
    /// Create an unambiguous result with a single scalar
    pub fn single(scalar: TypeId) -> Self {
        Self {
            scalar: Some(scalar),
            ambiguous: None,
        }
    }

    /// Create a result with no match
    pub fn none() -> Self {
        Self {
            scalar: None,
            ambiguous: None,
        }
    }

    /// Create an ambiguous result
    pub fn ambiguous(value_name: String, type_names: String, example_name: String) -> Self {
        Self {
            scalar: None,
            ambiguous: Some((value_name, type_names, example_name)),
        }
    }
}

// ============================================================================
// Intrinsic Types
// ============================================================================

/// Intrinsic type names (void, never, unknown, null, ErrorType)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IntrinsicTypeName {
    ErrorType,
    Void,
    Never,
    Unknown,
    Null,
}

/// Intrinsic type representation
#[derive(Debug, Clone)]
pub struct IntrinsicType {
    pub id: TypeId,
    pub name: IntrinsicTypeName,
    pub node: Option<NodeId>,
    pub is_finished: bool,
}

// ============================================================================
// Literal Types
// ============================================================================

/// String literal type - e.g., "hello"
#[derive(Debug, Clone)]
pub struct StringType {
    pub id: TypeId,
    pub value: String,
    pub node: Option<NodeId>,
    pub is_finished: bool,
}

/// Numeric literal type - e.g., 42
#[derive(Debug, Clone)]
pub struct NumericType {
    pub id: TypeId,
    pub value: f64,
    pub value_as_string: String,
    pub node: Option<NodeId>,
    pub is_finished: bool,
}

/// Boolean literal type - true or false
#[derive(Debug, Clone)]
pub struct BooleanType {
    pub id: TypeId,
    pub value: bool,
    pub node: Option<NodeId>,
    pub is_finished: bool,
}

// ============================================================================
// Composite Types
// ============================================================================

/// How a source model was used in building a model type.
/// Ported from TS SourceModel.usage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SourceModelUsage {
    /// `model A is B`
    Is,
    /// `model A {...B}` (spread)
    Spread,
    /// `alias A = B & C` (intersection)
    Intersection,
}

impl SourceModelUsage {
    /// Get the string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            SourceModelUsage::Is => "is",
            SourceModelUsage::Spread => "spread",
            SourceModelUsage::Intersection => "intersection",
        }
    }
}

/// Information about a source model used to build a model type.
/// Ported from TS SourceModel interface.
#[derive(Debug, Clone)]
pub struct SourceModel {
    /// How was this model used (is, spread, or intersection)
    pub usage: SourceModelUsage,
    /// The source model TypeId
    pub model: TypeId,
    /// Node where this source model was referenced
    pub node: Option<NodeId>,
}

/// Model type - represents a TypeSpec model
#[derive(Debug, Clone)]
pub struct ModelType {
    pub id: TypeId,
    pub name: String,
    pub node: Option<NodeId>,
    pub namespace: Option<TypeId>,
    pub properties: HashMap<String, TypeId>,
    pub property_names: Vec<String>, // Ordered property names
    pub base_model: Option<TypeId>,
    pub derived_models: Vec<TypeId>,
    pub source_model: Option<TypeId>,
    /// Source models used to build this model, with usage information.
    /// Includes models referenced via `model is`, `...` spread, or intersection `&`.
    /// TS: ModelType.sourceModels (SourceModel[])
    pub source_models: Vec<SourceModel>,
    pub indexer: Option<(TypeId, TypeId)>, // (key_type, value_type)
    pub template_node: Option<NodeId>,
    /// Template mapper for template instances. None for template declarations and non-templated types.
    /// TS: TemplatedType.templateMapper
    pub template_mapper: Option<Box<TypeMapper>>,
    pub decorators: Vec<DecoratorApplication>,
    /// Documentation comment
    pub doc: Option<String>,
    /// Summary comment
    pub summary: Option<String>,
    pub is_finished: bool,
}

impl ModelType {
    /// Create a new ModelType with sensible defaults for all optional fields.
    pub fn new(id: TypeId, name: String, node: Option<NodeId>, namespace: Option<TypeId>) -> Self {
        Self {
            id,
            name,
            node,
            namespace,
            properties: HashMap::new(),
            property_names: Vec::new(),
            base_model: None,
            derived_models: Vec::new(),
            source_model: None,
            source_models: vec![],
            indexer: None,
            template_node: None,
            template_mapper: None,
            decorators: Vec::new(),
            doc: None,
            summary: None,
            is_finished: false,
        }
    }
}

/// Model property type
#[derive(Debug, Clone)]
pub struct ModelPropertyType {
    pub id: TypeId,
    pub name: String,
    pub node: Option<NodeId>,
    pub r#type: TypeId,
    pub optional: bool,
    pub default_value: Option<TypeId>,
    pub model: Option<TypeId>,
    /// For properties copied via `is`, points to the source property
    /// TS: ModelProperty.sourceProperty
    pub source_property: Option<TypeId>,
    pub decorators: Vec<DecoratorApplication>,
    pub is_finished: bool,
}

/// Interface type
#[derive(Debug, Clone)]
pub struct InterfaceType {
    pub id: TypeId,
    pub name: String,
    pub node: Option<NodeId>,
    pub namespace: Option<TypeId>,
    pub operations: HashMap<String, TypeId>,
    pub operation_names: Vec<String>,
    pub extends: Vec<TypeId>,
    /// TS: sourceInterfaces - interfaces that this interface extends
    pub source_interfaces: Vec<TypeId>,
    pub template_node: Option<NodeId>,
    /// Template mapper for template instances. None for template declarations and non-templated types.
    /// TS: TemplatedType.templateMapper
    pub template_mapper: Option<Box<TypeMapper>>,
    pub decorators: Vec<DecoratorApplication>,
    /// Documentation comment
    pub doc: Option<String>,
    /// Summary comment
    pub summary: Option<String>,
    pub is_finished: bool,
}

impl InterfaceType {
    pub fn new(id: TypeId, name: String, node: Option<NodeId>, namespace: Option<TypeId>) -> Self {
        Self {
            id,
            name,
            node,
            namespace,
            operations: HashMap::new(),
            operation_names: Vec::new(),
            extends: Vec::new(),
            source_interfaces: Vec::new(),
            template_node: None,
            template_mapper: None,
            decorators: Vec::new(),
            doc: None,
            summary: None,
            is_finished: false,
        }
    }
}

/// Operation type
#[derive(Debug, Clone)]
pub struct OperationType {
    pub id: TypeId,
    pub name: String,
    pub node: Option<NodeId>,
    pub namespace: Option<TypeId>,
    pub parameters: Option<TypeId>, // ModelType for params
    pub return_type: Option<TypeId>,
    pub source_operation: Option<TypeId>, // TS: sourceOperation - when `op foo is bar`
    /// The interface this operation belongs to, if any.
    /// TS: Operation.interface
    pub interface_: Option<TypeId>,
    pub template_node: Option<NodeId>,
    /// Template mapper for template instances. None for template declarations and non-templated types.
    /// TS: TemplatedType.templateMapper
    pub template_mapper: Option<Box<TypeMapper>>,
    pub decorators: Vec<DecoratorApplication>,
    /// Documentation comment
    pub doc: Option<String>,
    /// Summary comment
    pub summary: Option<String>,
    pub is_finished: bool,
}

impl OperationType {
    pub fn new(id: TypeId, name: String, node: Option<NodeId>, namespace: Option<TypeId>) -> Self {
        Self {
            id,
            name,
            node,
            namespace,
            parameters: None,
            return_type: None,
            source_operation: None,
            interface_: None,
            template_node: None,
            template_mapper: None,
            decorators: Vec::new(),
            doc: None,
            summary: None,
            is_finished: false,
        }
    }
}

/// Enum type
#[derive(Debug, Clone)]
pub struct EnumType {
    pub id: TypeId,
    pub name: String,
    pub node: Option<NodeId>,
    pub namespace: Option<TypeId>,
    pub members: HashMap<String, TypeId>,
    pub member_names: Vec<String>,
    pub decorators: Vec<DecoratorApplication>,
    /// Documentation comment
    pub doc: Option<String>,
    /// Summary comment
    pub summary: Option<String>,
    pub is_finished: bool,
}

impl EnumType {
    pub fn new(id: TypeId, name: String, node: Option<NodeId>, namespace: Option<TypeId>) -> Self {
        Self {
            id,
            name,
            node,
            namespace,
            members: HashMap::new(),
            member_names: Vec::new(),
            decorators: Vec::new(),
            doc: None,
            summary: None,
            is_finished: false,
        }
    }
}

/// Enum member type
#[derive(Debug, Clone)]
pub struct EnumMemberType {
    pub id: TypeId,
    pub name: String,
    pub node: Option<NodeId>,
    pub r#enum: Option<TypeId>,
    pub value: Option<TypeId>, // String or Numeric type
    /// TS: sourceMember - when spread operators make new enum members,
    /// this tracks the enum member we copied from
    pub source_member: Option<TypeId>,
    pub decorators: Vec<DecoratorApplication>,
    pub is_finished: bool,
}

/// Union type
#[derive(Debug, Clone)]
pub struct UnionType {
    pub id: TypeId,
    pub name: String,
    pub node: Option<NodeId>,
    pub namespace: Option<TypeId>,
    pub variants: HashMap<String, TypeId>,
    pub variant_names: Vec<String>,
    /// Whether this is an anonymous union expression (e.g., `string | int32`)
    /// vs a named union declaration.
    /// TS: Union.expression
    pub expression: bool,
    pub template_node: Option<NodeId>,
    /// Template mapper for template instances. None for template declarations and non-templated types.
    /// TS: TemplatedType.templateMapper
    pub template_mapper: Option<Box<TypeMapper>>,
    pub decorators: Vec<DecoratorApplication>,
    /// Documentation comment
    pub doc: Option<String>,
    /// Summary comment
    pub summary: Option<String>,
    pub is_finished: bool,
}

impl UnionType {
    pub fn new(
        id: TypeId,
        name: String,
        node: Option<NodeId>,
        namespace: Option<TypeId>,
        expression: bool,
    ) -> Self {
        Self {
            id,
            name,
            node,
            namespace,
            expression,
            variants: HashMap::new(),
            variant_names: Vec::new(),
            template_node: None,
            template_mapper: None,
            decorators: Vec::new(),
            doc: None,
            summary: None,
            is_finished: false,
        }
    }
}

/// Union variant type
#[derive(Debug, Clone)]
pub struct UnionVariantType {
    pub id: TypeId,
    pub name: String,
    pub node: Option<NodeId>,
    pub r#type: TypeId,
    pub union: Option<TypeId>,
    pub decorators: Vec<DecoratorApplication>,
    pub is_finished: bool,
}

/// Scalar type
#[derive(Debug, Clone)]
pub struct ScalarType {
    pub id: TypeId,
    pub name: String,
    pub node: Option<NodeId>,
    pub namespace: Option<TypeId>,
    pub base_scalar: Option<TypeId>,
    pub constructors: Vec<TypeId>,
    /// TS: derivedScalars - list of scalars that extend this one
    pub derived_scalars: Vec<TypeId>,
    pub template_node: Option<NodeId>,
    /// Template mapper for template instances. None for template declarations and non-templated types.
    /// TS: TemplatedType.templateMapper
    pub template_mapper: Option<Box<TypeMapper>>,
    pub decorators: Vec<DecoratorApplication>,
    /// Documentation comment
    pub doc: Option<String>,
    /// Summary comment
    pub summary: Option<String>,
    pub is_finished: bool,
}

impl ScalarType {
    pub fn new(
        id: TypeId,
        name: String,
        node: Option<NodeId>,
        namespace: Option<TypeId>,
        base_scalar: Option<TypeId>,
    ) -> Self {
        Self {
            id,
            name,
            node,
            namespace,
            base_scalar,
            constructors: Vec::new(),
            derived_scalars: Vec::new(),
            template_node: None,
            template_mapper: None,
            decorators: Vec::new(),
            doc: None,
            summary: None,
            is_finished: false,
        }
    }
}

/// Scalar constructor type
#[derive(Debug, Clone)]
pub struct ScalarConstructorType {
    pub id: TypeId,
    pub name: String,
    pub node: Option<NodeId>,
    pub scalar: Option<TypeId>,
    pub parameters: Vec<TypeId>,
    pub is_finished: bool,
}

/// Template parameter type
#[derive(Debug, Clone)]
pub struct TemplateParameterType {
    pub id: TypeId,
    pub name: String,
    pub node: Option<NodeId>,
    pub constraint: Option<TypeId>,
    pub default: Option<TypeId>,
    pub is_finished: bool,
}

/// Tuple type
#[derive(Debug, Clone)]
pub struct TupleType {
    pub id: TypeId,
    pub node: Option<NodeId>,
    pub values: Vec<TypeId>,
    pub is_finished: bool,
}

/// Namespace type
#[derive(Debug, Clone)]
pub struct NamespaceType {
    pub id: TypeId,
    pub name: String,
    pub node: Option<NodeId>,
    pub namespace: Option<TypeId>,
    pub models: HashMap<String, TypeId>,
    pub model_names: Vec<String>, // Ordered model names
    pub scalars: HashMap<String, TypeId>,
    pub scalar_names: Vec<String>, // Ordered scalar names
    pub operations: HashMap<String, TypeId>,
    pub operation_names: Vec<String>, // Ordered operation names
    pub interfaces: HashMap<String, TypeId>,
    pub interface_names: Vec<String>, // Ordered interface names
    pub enums: HashMap<String, TypeId>,
    pub enum_names: Vec<String>, // Ordered enum names
    pub unions: HashMap<String, TypeId>,
    pub union_names: Vec<String>, // Ordered union names
    pub namespaces: HashMap<String, TypeId>,
    pub namespace_names: Vec<String>, // Ordered namespace names
    /// TS: decoratorDeclarations - decorator declarations in this namespace
    pub decorator_declarations: HashMap<String, TypeId>,
    pub decorator_declaration_names: Vec<String>, // Ordered decorator declaration names
    /// TS: functionDeclarations - function declarations in this namespace
    pub function_declarations: HashMap<String, TypeId>,
    pub function_declaration_names: Vec<String>, // Ordered function declaration names
    pub decorators: Vec<DecoratorApplication>,
    /// Documentation comment
    pub doc: Option<String>,
    /// Summary comment
    pub summary: Option<String>,
    pub is_finished: bool,
}

impl NamespaceType {
    /// Create a new NamespaceType with empty collections.
    /// Only `id`, `name`, `node`, `namespace`, and `is_finished` need to be specified.
    pub fn new(
        id: TypeId,
        name: String,
        node: Option<NodeId>,
        namespace: Option<TypeId>,
        is_finished: bool,
    ) -> Self {
        Self {
            id,
            name,
            node,
            namespace,
            models: HashMap::new(),
            model_names: Vec::new(),
            scalars: HashMap::new(),
            scalar_names: Vec::new(),
            operations: HashMap::new(),
            operation_names: Vec::new(),
            interfaces: HashMap::new(),
            interface_names: Vec::new(),
            enums: HashMap::new(),
            enum_names: Vec::new(),
            unions: HashMap::new(),
            union_names: Vec::new(),
            namespaces: HashMap::new(),
            namespace_names: Vec::new(),
            decorator_declarations: HashMap::new(),
            decorator_declaration_names: Vec::new(),
            function_declarations: HashMap::new(),
            function_declaration_names: Vec::new(),
            decorators: Vec::new(),
            doc: None,
            summary: None,
            is_finished,
        }
    }

    /// Look up a member by name across all member maps (namespaces, models, scalars, etc.).
    /// Returns the first match in order: namespaces, models, scalars, interfaces, enums, unions, operations, decorator_declarations.
    pub fn lookup_member(&self, name: &str) -> Option<TypeId> {
        self.namespaces
            .get(name)
            .copied()
            .or_else(|| self.models.get(name).copied())
            .or_else(|| self.scalars.get(name).copied())
            .or_else(|| self.interfaces.get(name).copied())
            .or_else(|| self.enums.get(name).copied())
            .or_else(|| self.unions.get(name).copied())
            .or_else(|| self.operations.get(name).copied())
            .or_else(|| self.decorator_declarations.get(name).copied())
    }
}

/// Decorator type
#[derive(Debug, Clone)]
pub struct DecoratorType {
    pub id: TypeId,
    pub name: String,
    pub node: Option<NodeId>,
    pub namespace: Option<TypeId>,
    pub target: Option<TypeId>,
    /// The target type constraint (e.g., "Model", "unknown") - describes what the decorator can be applied to
    pub target_type: String,
    pub parameters: Vec<FunctionParameterType>,
    pub is_finished: bool,
}

/// Function type
#[derive(Debug, Clone)]
pub struct FunctionTypeType {
    pub id: TypeId,
    pub name: String,
    pub node: Option<NodeId>,
    pub namespace: Option<TypeId>,
    pub parameters: Vec<TypeId>,
    pub return_type: Option<TypeId>,
    pub is_finished: bool,
}

/// Function parameter type
#[derive(Debug, Clone)]
pub struct FunctionParameterType {
    pub id: TypeId,
    pub name: String,
    pub node: Option<NodeId>,
    pub r#type: Option<TypeId>,
    pub optional: bool,
    pub rest: bool,
    pub is_finished: bool,
}

/// String template type
#[derive(Debug, Clone)]
pub struct StringTemplateType {
    pub id: TypeId,
    pub node: Option<NodeId>,
    pub spans: Vec<TypeId>,
    /// The computed string value if all interpolated elements are string-serializable.
    /// None if any element cannot be converted to a string.
    pub string_value: Option<String>,
    pub is_finished: bool,
}

/// String template span type
#[derive(Debug, Clone)]
pub struct StringTemplateSpanType {
    pub id: TypeId,
    pub node: Option<NodeId>,
    pub expression: Option<TypeId>,
    pub r#type: Option<TypeId>,
    pub is_finished: bool,
}

// ============================================================================
// Type Enum
// ============================================================================

/// The Type enum - represents all possible TypeSpec types.
/// Ported from TS `type Type = BooleanLiteral | Decorator | ...`
#[derive(Debug, Clone)]
pub enum Type {
    Intrinsic(IntrinsicType),
    Model(ModelType),
    ModelProperty(ModelPropertyType),
    Interface(InterfaceType),
    Operation(OperationType),
    Enum(EnumType),
    EnumMember(EnumMemberType),
    Union(UnionType),
    UnionVariant(UnionVariantType),
    Scalar(ScalarType),
    ScalarConstructor(ScalarConstructorType),
    TemplateParameter(TemplateParameterType),
    Tuple(TupleType),
    String(StringType),
    Number(NumericType),
    Boolean(BooleanType),
    Namespace(Box<NamespaceType>),
    Decorator(DecoratorType),
    FunctionType(FunctionTypeType),
    FunctionParameter(FunctionParameterType),
    StringTemplate(StringTemplateType),
    StringTemplateSpan(StringTemplateSpanType),
}

/// Trait for types that support template parameters (Model, Interface, Operation, Union, Scalar).
/// Provides unified access to template-related fields, eliminating the need for
/// 5-arm match expressions in is_template_instance/is_template_declaration.
pub trait TemplateInfo {
    /// Whether a template mapper is set (true for template instances)
    fn has_template_mapper(&self) -> bool;
    fn template_node(&self) -> Option<NodeId>;
    fn node(&self) -> Option<NodeId>;
}

/// Macro to implement TemplateInfo for types that support templates.
macro_rules! impl_template_info {
    ($ty:ty) => {
        impl TemplateInfo for $ty {
            fn has_template_mapper(&self) -> bool {
                self.template_mapper.is_some()
            }
            fn template_node(&self) -> Option<NodeId> {
                self.template_node
            }
            fn node(&self) -> Option<NodeId> {
                self.node
            }
        }
    };
}

impl_template_info!(ModelType);
impl_template_info!(InterfaceType);
impl_template_info!(OperationType);
impl_template_info!(UnionType);
impl_template_info!(ScalarType);

/// Get template info for a Type, if it supports templates.
pub fn get_template_info(t: &Type) -> Option<&dyn TemplateInfo> {
    match t {
        Type::Model(m) => Some(m),
        Type::Interface(i) => Some(i),
        Type::Operation(o) => Some(o),
        Type::Union(u) => Some(u),
        Type::Scalar(s) => Some(s),
        _ => None,
    }
}

/// Macro to dispatch a method call across all Type variants.
/// Reduces boilerplate for methods that access a common field (id, node, is_finished).
macro_rules! type_dispatch {
    ($self:expr, $inner:ident, $body:expr) => {
        match $self {
            Type::Intrinsic($inner) => $body,
            Type::Model($inner) => $body,
            Type::ModelProperty($inner) => $body,
            Type::Interface($inner) => $body,
            Type::Operation($inner) => $body,
            Type::Enum($inner) => $body,
            Type::EnumMember($inner) => $body,
            Type::Union($inner) => $body,
            Type::UnionVariant($inner) => $body,
            Type::Scalar($inner) => $body,
            Type::ScalarConstructor($inner) => $body,
            Type::TemplateParameter($inner) => $body,
            Type::Tuple($inner) => $body,
            Type::String($inner) => $body,
            Type::Number($inner) => $body,
            Type::Boolean($inner) => $body,
            Type::Namespace($inner) => $body,
            Type::Decorator($inner) => $body,
            Type::FunctionType($inner) => $body,
            Type::FunctionParameter($inner) => $body,
            Type::StringTemplate($inner) => $body,
            Type::StringTemplateSpan($inner) => $body,
        }
    };
}

/// Macro for partial dispatch — only matches listed variants, returns None for others.
/// Use for methods that only apply to some Type variants (e.g., namespace, decorators).
macro_rules! type_dispatch_partial {
    // Form 1: body is a direct value (not Option), wraps in Some
    ($self:expr, [$($variant:ident),*], $inner:ident, $body:expr) => {
        match $self {
            $(Type::$variant($inner) => Some($body),)*
            _ => None,
        }
    };
    // Form 2: body already returns Option, no wrapping
    (opt $self:expr, [$($variant:ident),*], $inner:ident, $body:expr) => {
        match $self {
            $(Type::$variant($inner) => $body,)*
            _ => None,
        }
    };
}

/// Macro for Value enum dispatch — matches all 10 variants.
macro_rules! value_dispatch {
    ($self:expr, $inner:ident, $body:expr) => {
        match $self {
            Value::StringValue($inner) => $body,
            Value::NumericValue($inner) => $body,
            Value::BooleanValue($inner) => $body,
            Value::ObjectValue($inner) => $body,
            Value::ArrayValue($inner) => $body,
            Value::EnumValue($inner) => $body,
            Value::NullValue($inner) => $body,
            Value::ScalarValue($inner) => $body,
            Value::FunctionValue($inner) => $body,
            Value::TemplateValue($inner) => $body,
        }
    };
}

impl Type {
    /// Get the TypeId of this type
    pub fn id(&self) -> TypeId {
        type_dispatch!(self, t, t.id)
    }

    /// Get the kind name of this type
    pub fn kind_name(&self) -> &'static str {
        match self {
            Type::Intrinsic(_) => "Intrinsic",
            Type::Model(_) => "Model",
            Type::ModelProperty(_) => "ModelProperty",
            Type::Interface(_) => "Interface",
            Type::Operation(_) => "Operation",
            Type::Enum(_) => "Enum",
            Type::EnumMember(_) => "EnumMember",
            Type::Union(_) => "Union",
            Type::UnionVariant(_) => "UnionVariant",
            Type::Scalar(_) => "Scalar",
            Type::ScalarConstructor(_) => "ScalarConstructor",
            Type::TemplateParameter(_) => "TemplateParameter",
            Type::Tuple(_) => "Tuple",
            Type::String(_) => "String",
            Type::Number(_) => "Number",
            Type::Boolean(_) => "Boolean",
            Type::Namespace(_) => "Namespace",
            Type::Decorator(_) => "Decorator",
            Type::FunctionType(_) => "FunctionType",
            Type::FunctionParameter(_) => "FunctionParameter",
            Type::StringTemplate(_) => "StringTemplate",
            Type::StringTemplateSpan(_) => "StringTemplateSpan",
        }
    }

    /// Get the name of this type (if it has one)
    pub fn name(&self) -> Option<&str> {
        match self {
            Type::Intrinsic(t) => Some(match t.name {
                IntrinsicTypeName::ErrorType => "ErrorType",
                IntrinsicTypeName::Void => "void",
                IntrinsicTypeName::Never => "never",
                IntrinsicTypeName::Unknown => "unknown",
                IntrinsicTypeName::Null => "null",
            }),
            Type::Model(t) => Some(&t.name),
            Type::ModelProperty(t) => Some(&t.name),
            Type::Interface(t) => Some(&t.name),
            Type::Operation(t) => Some(&t.name),
            Type::Enum(t) => Some(&t.name),
            Type::EnumMember(t) => Some(&t.name),
            Type::Union(t) => Some(&t.name),
            Type::UnionVariant(t) => Some(&t.name),
            Type::Scalar(t) => Some(&t.name),
            Type::TemplateParameter(t) => Some(&t.name),
            Type::Namespace(t) => Some(&t.name),
            Type::Decorator(t) => Some(&t.name),
            Type::FunctionType(t) => Some(&t.name),
            Type::FunctionParameter(t) => Some(&t.name),
            Type::String(t) => Some(&t.value),
            Type::Number(t) => Some(&t.value_as_string),
            Type::Boolean(t) => Some(if t.value { "true" } else { "false" }),
            _ => None,
        }
    }

    /// Set the name on this type (only works for named variants)
    pub fn set_name(&mut self, new_name: String) {
        match self {
            Type::Model(t) => t.name = new_name,
            Type::Scalar(t) => t.name = new_name,
            Type::Interface(t) => t.name = new_name,
            Type::Enum(t) => t.name = new_name,
            Type::Union(t) => t.name = new_name,
            Type::Operation(t) => t.name = new_name,
            Type::Namespace(t) => t.name = new_name,
            Type::Decorator(t) => t.name = new_name,
            _ => {}
        }
    }

    /// Check if this is an error intrinsic type.
    ///
    /// Note: This only checks the type structure, not whether the TypeId matches
    /// `checker.error_type`. For the full check, use `typekit::type_kind::is_error()`.
    pub fn is_error(&self) -> bool {
        matches!(self, Type::Intrinsic(t) if t.name == IntrinsicTypeName::ErrorType)
    }

    /// Get the node ID from this type, if it has one
    pub fn node_id_from_type(&self) -> Option<NodeId> {
        type_dispatch!(self, t, t.node)
    }

    /// Get the template_node for types that support it
    pub fn template_node(&self) -> Option<NodeId> {
        type_dispatch_partial!(opt self, [Model, Interface, Union, Scalar, Operation], t, t.template_node)
    }

    /// Get the template_mapper for types that support it.
    /// Returns Some if this type is a template instance (was instantiated from a template).
    /// Returns None for template declarations and non-templated types.
    pub fn template_mapper(&self) -> Option<&TypeMapper> {
        type_dispatch_partial!(
            self,
            [Model, Interface, Union, Scalar, Operation],
            t,
            t.template_mapper.as_deref()?
        )
    }

    /// Set the template mapper if not already set. Returns true if set.
    pub fn set_template_mapper_if_none(&mut self, mapper: Box<TypeMapper>) -> bool {
        match self {
            Type::Model(t) if t.template_mapper.is_none() => {
                t.template_mapper = Some(mapper);
                true
            }
            Type::Interface(t) if t.template_mapper.is_none() => {
                t.template_mapper = Some(mapper);
                true
            }
            Type::Union(t) if t.template_mapper.is_none() => {
                t.template_mapper = Some(mapper);
                true
            }
            Type::Scalar(t) if t.template_mapper.is_none() => {
                t.template_mapper = Some(mapper);
                true
            }
            Type::Operation(t) if t.template_mapper.is_none() => {
                t.template_mapper = Some(mapper);
                true
            }
            _ => false,
        }
    }

    /// Get the namespace TypeId for types that belong to a namespace
    pub fn namespace(&self) -> Option<TypeId> {
        type_dispatch_partial!(opt self, [Model, Interface, Operation, Enum, Union, Scalar, Namespace, Decorator, FunctionType], t, t.namespace)
    }

    /// Get the doc comment for this type, if any
    pub fn doc(&self) -> Option<&str> {
        type_dispatch_partial!(opt self, [Model, Interface, Operation, Enum, Scalar, Union, Namespace], t, t.doc.as_deref())
    }

    /// Get the summary comment for this type, if any
    pub fn summary(&self) -> Option<&str> {
        type_dispatch_partial!(opt self, [Model, Interface, Operation, Enum, Scalar, Union], t, t.summary.as_deref())
    }

    /// Check if this type is finished
    pub fn is_finished(&self) -> bool {
        type_dispatch!(self, t, t.is_finished)
    }

    /// Mark this type as finished
    pub fn set_finished(&mut self, value: bool) {
        type_dispatch!(self, t, t.is_finished = value)
    }

    /// Get an immutable reference to this type's decorators, if it has any.
    /// Returns None for types that don't support decorators (Intrinsic, literals, etc.)
    pub fn decorators(&self) -> Option<&Vec<DecoratorApplication>> {
        type_dispatch_partial!(
            self,
            [
                Model,
                ModelProperty,
                Interface,
                Operation,
                Enum,
                EnumMember,
                Union,
                UnionVariant,
                Scalar,
                Namespace
            ],
            t,
            &t.decorators
        )
    }

    /// Get a mutable reference to this type's decorators, if it has any.
    /// Returns None for types that don't support decorators (Intrinsic, literals, etc.)
    pub fn decorators_mut(&mut self) -> Option<&mut Vec<DecoratorApplication>> {
        match self {
            Type::Model(t) => Some(&mut t.decorators),
            Type::ModelProperty(t) => Some(&mut t.decorators),
            Type::Interface(t) => Some(&mut t.decorators),
            Type::Operation(t) => Some(&mut t.decorators),
            Type::Enum(t) => Some(&mut t.decorators),
            Type::EnumMember(t) => Some(&mut t.decorators),
            Type::Union(t) => Some(&mut t.decorators),
            Type::UnionVariant(t) => Some(&mut t.decorators),
            Type::Scalar(t) => Some(&mut t.decorators),
            Type::Namespace(t) => Some(&mut t.decorators),
            _ => None,
        }
    }

    /// Set the type id - used by TypeStore::add to auto-correct id
    pub fn set_id(&mut self, id: TypeId) {
        type_dispatch!(self, t, t.id = id)
    }
}

// ============================================================================
// Type Store
// ============================================================================

/// TypeStore - stores all created types
#[derive(Debug, Clone)]
pub struct TypeStore {
    types: Vec<Type>,
}

impl TypeStore {
    pub fn new() -> Self {
        TypeStore { types: Vec::new() }
    }

    /// Add a type to the store and return its TypeId.
    /// Automatically corrects the type's id to match its position in the store.
    pub fn add(&mut self, mut t: Type) -> TypeId {
        let id = self.types.len() as TypeId;
        t.set_id(id);
        self.types.push(t);
        id
    }

    /// Get a type by its TypeId
    pub fn get(&self, id: TypeId) -> Option<&Type> {
        self.types.get(id as usize)
    }

    /// Get a mutable reference to a type by its TypeId
    pub fn get_mut(&mut self, id: TypeId) -> Option<&mut Type> {
        self.types.get_mut(id as usize)
    }

    /// Get the number of types in the store
    pub fn len(&self) -> usize {
        self.types.len()
    }

    /// Check if the store is empty
    pub fn is_empty(&self) -> bool {
        self.types.is_empty()
    }

    /// Allocate the next TypeId
    pub fn next_type_id(&self) -> TypeId {
        self.types.len() as TypeId
    }
}

impl Default for TypeStore {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Value Types
// ============================================================================

/// Value ID - unique identifier for a value in the value store
pub type ValueId = u32;

/// Value - represents a runtime value in TypeSpec
/// Ported from TS compiler/src/core/types.ts Value types
#[derive(Debug, Clone)]
pub enum Value {
    StringValue(StringValue),
    NumericValue(NumericValue),
    BooleanValue(BooleanValue),
    ObjectValue(ObjectValue),
    ArrayValue(ArrayValue),
    EnumValue(EnumValue),
    NullValue(NullValue),
    ScalarValue(ScalarValue),
    /// Function value - represents a function reference as a value.
    /// Ported from TS FunctionValue.
    FunctionValue(FunctionValueType),
    /// Template value - internal type representing a value in a template declaration.
    /// This should never be exposed on the type graph (unlike TemplateParameter).
    /// Ported from TS TemplateValue.
    TemplateValue(TemplateValue),
}

/// Base trait for all value types
impl Value {
    /// Get the storage type of this value
    pub fn value_type(&self) -> TypeId {
        value_dispatch!(self, v, v.type_id)
    }

    /// Set the storage type of this value.
    /// Used when a const has an explicit type annotation — the value's type
    /// becomes the declared type, not the inferred literal type.
    /// TS: copyValue(value, { type })
    pub fn set_value_type(&mut self, new_type: TypeId) {
        value_dispatch!(self, v, v.type_id = new_type)
    }

    /// Get the kind name of this value
    pub fn value_kind_name(&self) -> &'static str {
        match self {
            Value::StringValue(_) => "StringValue",
            Value::NumericValue(_) => "NumericValue",
            Value::BooleanValue(_) => "BooleanValue",
            Value::ObjectValue(_) => "ObjectValue",
            Value::ArrayValue(_) => "ArrayValue",
            Value::EnumValue(_) => "EnumValue",
            Value::NullValue(_) => "NullValue",
            Value::ScalarValue(_) => "ScalarValue",
            Value::FunctionValue(_) => "FunctionValue",
            Value::TemplateValue(_) => "TemplateValue",
        }
    }
}

/// String value - e.g., const x = "hello"
#[derive(Debug, Clone)]
pub struct StringValue {
    pub type_id: TypeId,
    pub value: String,
    pub scalar: Option<TypeId>,
    pub node: Option<NodeId>,
}

/// Numeric value - e.g., const x = 42
#[derive(Debug, Clone)]
pub struct NumericValue {
    pub type_id: TypeId,
    pub value: f64,
    pub scalar: Option<TypeId>,
    pub node: Option<NodeId>,
}

/// Boolean value - e.g., const x = true
#[derive(Debug, Clone)]
pub struct BooleanValue {
    pub type_id: TypeId,
    pub value: bool,
    pub scalar: Option<TypeId>,
    pub node: Option<NodeId>,
}

/// Object value - e.g., const x = #{ name: "foo" }
#[derive(Debug, Clone)]
pub struct ObjectValue {
    pub type_id: TypeId,
    pub properties: Vec<ObjectValueProperty>,
    pub node: Option<NodeId>,
}

/// Object value property descriptor
#[derive(Debug, Clone)]
pub struct ObjectValueProperty {
    pub name: String,
    pub value: ValueId,
}

/// Array value - e.g., const x = #[1, 2, 3]
#[derive(Debug, Clone)]
pub struct ArrayValue {
    pub type_id: TypeId,
    pub values: Vec<ValueId>,
    pub node: Option<NodeId>,
}

/// Enum value - e.g., Status.active
#[derive(Debug, Clone)]
pub struct EnumValue {
    pub type_id: TypeId,
    pub value: TypeId, // EnumMember TypeId
    pub node: Option<NodeId>,
}

/// Null value - e.g., const x = null
#[derive(Debug, Clone)]
pub struct NullValue {
    pub type_id: TypeId,
    pub node: Option<NodeId>,
}

/// Scalar value - constructed via scalar constructor
#[derive(Debug, Clone)]
pub struct ScalarValue {
    pub type_id: TypeId,
    pub scalar: TypeId,
    pub args: Vec<ValueId>,
    pub node: Option<NodeId>,
}

/// Function value - represents a function reference as a value.
/// Ported from TS FunctionValue interface.
#[derive(Debug, Clone)]
pub struct FunctionValueType {
    pub type_id: TypeId,
    /// Function name (None for anonymous functions)
    pub name: Option<String>,
    pub node: Option<NodeId>,
}

/// Template value - internal type representing a value while in a template declaration.
/// This should never be exposed on the type graph (unlike TemplateParameter).
/// Ported from TS TemplateValue.
#[derive(Debug, Clone)]
pub struct TemplateValue {
    pub type_id: TypeId,
}

// ============================================================================
// Value Store
// ============================================================================

/// ValueStore - stores all created values
#[derive(Debug, Clone, Default)]
pub struct ValueStore {
    values: Vec<Value>,
}

impl ValueStore {
    pub fn new() -> Self {
        ValueStore { values: Vec::new() }
    }

    /// Add a value to the store and return its ValueId
    pub fn add(&mut self, v: Value) -> ValueId {
        let id = self.values.len() as ValueId;
        self.values.push(v);
        id
    }

    /// Get a value by its ValueId
    pub fn get(&self, id: ValueId) -> Option<&Value> {
        self.values.get(id as usize)
    }

    /// Get a mutable reference to a value by its ValueId
    pub fn get_mut(&mut self, id: ValueId) -> Option<&mut Value> {
        self.values.get_mut(id as usize)
    }

    /// Get the number of values in the store
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Check if the store is empty
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Allocate the next ValueId
    pub fn next_value_id(&self) -> ValueId {
        self.values.len() as ValueId
    }
}

// ============================================================================
// Entity (Type | Value | MixedConstraint | Indeterminate)
// ============================================================================

/// Entity - the result of checking a node, which may be a Type, Value,
/// MixedParameterConstraint, or Indeterminate.
/// In TS, Entity = Type | Value | MixedParameterConstraint | IndeterminateEntity.
///
/// "Indeterminate" means the result could be either a type or a value
/// depending on context (e.g., a string literal like "hello" is both
/// the type `"hello"` and the value `"hello"`).
///
/// "MixedConstraint" represents a union expression constraint that includes
/// both type and value parts (e.g., `T extends string | valueof int32`).
#[derive(Debug, Clone)]
pub enum Entity {
    /// A type entity
    Type(TypeId),
    /// A value entity
    Value(ValueId),
    /// A mixed parameter constraint entity
    /// Used when a template parameter constraint includes both type and value parts
    MixedConstraint(MixedParameterConstraint),
    /// An indeterminate entity - could be type or value
    /// The inner TypeId represents the type, which can be converted
    /// to a value if the context requires it
    Indeterminate(TypeId),
}

impl Entity {
    /// Check if this entity is a type
    pub fn is_type(&self) -> bool {
        matches!(self, Entity::Type(_))
    }

    /// Check if this entity is a value
    pub fn is_value(&self) -> bool {
        matches!(self, Entity::Value(_))
    }

    /// Check if this entity is indeterminate
    pub fn is_indeterminate(&self) -> bool {
        matches!(self, Entity::Indeterminate(_))
    }

    /// Check if this entity is a mixed parameter constraint
    pub fn is_mixed_constraint(&self) -> bool {
        matches!(self, Entity::MixedConstraint(_))
    }

    /// Get the TypeId if this is a Type entity
    pub fn as_type_id(&self) -> Option<TypeId> {
        match self {
            Entity::Type(id) => Some(*id),
            Entity::Indeterminate(id) => Some(*id),
            _ => None,
        }
    }

    /// Get the ValueId if this is a Value entity
    pub fn as_value_id(&self) -> Option<ValueId> {
        match self {
            Entity::Value(id) => Some(*id),
            _ => None,
        }
    }
}

// ============================================================================
// Mixed Parameter Constraint
// ============================================================================

/// Mixed parameter constraint - for template parameter constraints
/// that can include both type and value constraints (e.g., `T extends string | valueof int32`)
#[derive(Debug, Clone)]
pub struct MixedParameterConstraint {
    /// The node that defines this constraint
    pub node: Option<NodeId>,
    /// Type constraint (for `T extends SomeType`)
    pub type_constraint: Option<TypeId>,
    /// Value constraint (for `T extends valueof SomeType`)
    pub value_constraint: Option<TypeId>,
}

// ============================================================================
// Decorator Application Types
// ============================================================================

/// DecoratorArgument - a single argument passed to a decorator
#[derive(Debug, Clone)]
pub struct DecoratorArgument {
    /// The value (as a type or literal)
    pub value: NodeId,
    /// The JS-marshalled value for use in JavaScript interop
    pub js_value: Option<DecoratorMarshalledValue>,
    /// Node where this argument appears
    pub node: Option<NodeId>,
}

/// DecoratorMarshalledValue - JS-marshalled values
#[derive(Debug, Clone)]
pub enum DecoratorMarshalledValue {
    Type(NodeId),
    Value(NodeId),
    Record(HashMap<String, NodeId>),
    Array(Vec<NodeId>),
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
}

/// DecoratorApplication - a decorator applied to a declaration
#[derive(Debug, Clone)]
pub struct DecoratorApplication {
    /// The decorator definition (TypeId pointing to a DecoratorType)
    pub definition: Option<TypeId>,
    /// The decorator node ID
    pub decorator: NodeId,
    /// Arguments to the decorator
    pub args: Vec<DecoratorArgument>,
    /// The node where this decorator was applied
    pub node: Option<NodeId>,
}

impl DecoratorApplication {
    pub fn new(decorator: NodeId) -> Self {
        Self {
            definition: None,
            decorator,
            args: Vec::new(),
            node: None,
        }
    }

    pub fn with_args(mut self, args: Vec<DecoratorArgument>) -> Self {
        self.args = args;
        self
    }

    pub fn with_definition(mut self, definition: TypeId) -> Self {
        self.definition = Some(definition);
        self
    }

    pub fn with_node(mut self, node: NodeId) -> Self {
        self.node = Some(node);
        self
    }
}

// ============================================================================
// Type Mapper (template instantiation)
// ============================================================================

/// TemplatedType - a type that was created from a template instantiation.
/// In TS this is a type that has `templateMapper` and `templateNode` fields.
/// In Rust, we track this via the `template_node` field on individual type structs
/// and the TypeMapper stored separately in the Checker.
/// Ported from TS TemplatedType = Model | Interface | Operation | Union | Scalar.
pub type TemplatedTypeId = TypeId;

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // TypeStore tests
    // ========================================================================

    #[test]
    fn test_type_store_new() {
        let store = TypeStore::new();
        assert!(store.is_empty());
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn test_type_store_add_intrinsic() {
        let mut store = TypeStore::new();
        let id = store.add(Type::Intrinsic(IntrinsicType {
            id: INVALID_TYPE_ID,
            name: IntrinsicTypeName::Void,
            node: None,
            is_finished: true,
        }));
        assert_eq!(id, 0);
        assert_eq!(store.len(), 1);
        assert!(store.get(0).is_some());
    }

    #[test]
    fn test_type_store_add_multiple() {
        let mut store = TypeStore::new();
        let id1 = store.add(Type::Intrinsic(IntrinsicType {
            id: INVALID_TYPE_ID,
            name: IntrinsicTypeName::Void,
            node: None,
            is_finished: true,
        }));
        let id2 = store.add(Type::Intrinsic(IntrinsicType {
            id: INVALID_TYPE_ID,
            name: IntrinsicTypeName::Never,
            node: None,
            is_finished: true,
        }));
        let id3 = store.add(Type::Intrinsic(IntrinsicType {
            id: INVALID_TYPE_ID,
            name: IntrinsicTypeName::Unknown,
            node: None,
            is_finished: true,
        }));
        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
        assert_eq!(id3, 2);
        assert_eq!(store.len(), 3);
    }

    #[test]
    fn test_type_store_get() {
        let mut store = TypeStore::new();
        store.add(Type::Intrinsic(IntrinsicType {
            id: INVALID_TYPE_ID,
            name: IntrinsicTypeName::Void,
            node: None,
            is_finished: true,
        }));
        let t = store.get(0).unwrap();
        assert!(matches!(t, Type::Intrinsic(i) if i.name == IntrinsicTypeName::Void));
    }

    #[test]
    fn test_type_store_get_invalid() {
        let store = TypeStore::new();
        assert!(store.get(0).is_none());
        assert!(store.get(999).is_none());
    }

    #[test]
    fn test_type_store_next_type_id() {
        let mut store = TypeStore::new();
        assert_eq!(store.next_type_id(), 0);
        store.add(Type::Intrinsic(IntrinsicType {
            id: INVALID_TYPE_ID,
            name: IntrinsicTypeName::Void,
            node: None,
            is_finished: true,
        }));
        assert_eq!(store.next_type_id(), 1);
    }

    #[test]
    fn test_type_store_get_mut() {
        let mut store = TypeStore::new();
        store.add(Type::Model(ModelType {
            id: INVALID_TYPE_ID,
            name: "Foo".to_string(),
            node: None,
            properties: HashMap::new(),
            property_names: vec![],
            indexer: None,
            base_model: None,
            derived_models: vec![],
            source_model: None,
            source_models: vec![],
            namespace: None,
            decorators: vec![],
            template_node: None,
            template_mapper: None,
            doc: None,
            summary: None,
            is_finished: false,
        }));
        if let Some(Type::Model(m)) = store.get_mut(0) {
            m.is_finished = true;
        }
        let t = store.get(0).unwrap();
        if let Type::Model(m) = t {
            assert!(m.is_finished);
        } else {
            panic!("Expected Model type");
        }
    }

    // ========================================================================
    // Type method tests
    // ========================================================================

    #[test]
    fn test_type_id() {
        let t = Type::Intrinsic(IntrinsicType {
            id: 5,
            name: IntrinsicTypeName::Void,
            node: None,
            is_finished: true,
        });
        assert_eq!(t.id(), 5);
    }

    #[test]
    fn test_type_kind_name() {
        let t = Type::Intrinsic(IntrinsicType {
            id: 0,
            name: IntrinsicTypeName::Void,
            node: None,
            is_finished: true,
        });
        assert_eq!(t.kind_name(), "Intrinsic");

        let m = Type::Model(ModelType {
            id: 0,
            name: "Foo".to_string(),
            node: None,
            properties: HashMap::new(),
            property_names: vec![],
            indexer: None,
            base_model: None,
            derived_models: vec![],
            source_model: None,
            source_models: vec![],
            namespace: None,
            decorators: vec![],
            template_node: None,
            template_mapper: None,
            doc: None,
            summary: None,
            is_finished: true,
        });
        assert_eq!(m.kind_name(), "Model");
    }

    #[test]
    fn test_type_name() {
        let t = Type::Intrinsic(IntrinsicType {
            id: 0,
            name: IntrinsicTypeName::Void,
            node: None,
            is_finished: true,
        });
        assert_eq!(t.name(), Some("void"));

        let m = Type::Model(ModelType {
            id: 0,
            name: "Foo".to_string(),
            node: None,
            properties: HashMap::new(),
            property_names: vec![],
            indexer: None,
            base_model: None,
            derived_models: vec![],
            source_model: None,
            source_models: vec![],
            namespace: None,
            decorators: vec![],
            template_node: None,
            template_mapper: None,
            doc: None,
            summary: None,
            is_finished: true,
        });
        assert_eq!(m.name(), Some("Foo"));
    }

    #[test]
    fn test_type_is_error() {
        let err = Type::Intrinsic(IntrinsicType {
            id: 0,
            name: IntrinsicTypeName::ErrorType,
            node: None,
            is_finished: true,
        });
        assert!(err.is_error());

        let void = Type::Intrinsic(IntrinsicType {
            id: 0,
            name: IntrinsicTypeName::Void,
            node: None,
            is_finished: true,
        });
        assert!(!void.is_error());
    }

    #[test]
    fn test_type_is_finished() {
        let t = Type::Intrinsic(IntrinsicType {
            id: 0,
            name: IntrinsicTypeName::Void,
            node: None,
            is_finished: true,
        });
        assert!(t.is_finished());
    }

    // ========================================================================
    // Entity tests
    // ========================================================================

    #[test]
    fn test_entity_type() {
        let e = Entity::Type(5);
        assert!(e.is_type());
        assert!(!e.is_value());
        assert!(!e.is_indeterminate());
        assert_eq!(e.as_type_id(), Some(5));
        assert_eq!(e.as_value_id(), None);
    }

    #[test]
    fn test_entity_value() {
        let e = Entity::Value(10);
        assert!(!e.is_type());
        assert!(e.is_value());
        assert!(!e.is_indeterminate());
        assert_eq!(e.as_type_id(), None);
        assert_eq!(e.as_value_id(), Some(10));
    }

    #[test]
    fn test_entity_indeterminate() {
        let e = Entity::Indeterminate(3);
        assert!(!e.is_type());
        assert!(!e.is_value());
        assert!(e.is_indeterminate());
        assert_eq!(e.as_type_id(), Some(3));
        assert_eq!(e.as_value_id(), None);
    }

    // ========================================================================
    // ValueStore tests
    // ========================================================================

    #[test]
    fn test_value_store_new() {
        let store = ValueStore::new();
        assert!(store.is_empty());
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn test_value_store_add_string() {
        let mut store = ValueStore::new();
        let id = store.add(Value::StringValue(StringValue {
            type_id: 0,
            value: "hello".to_string(),
            scalar: None,
            node: None,
        }));
        assert_eq!(id, 0);
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn test_value_store_add_numeric() {
        let mut store = ValueStore::new();
        let id = store.add(Value::NumericValue(NumericValue {
            type_id: 0,
            value: 42.0,
            scalar: None,
            node: None,
        }));
        assert_eq!(id, 0);
    }

    #[test]
    fn test_value_store_add_boolean() {
        let mut store = ValueStore::new();
        let id = store.add(Value::BooleanValue(BooleanValue {
            type_id: 0,
            value: true,
            scalar: None,
            node: None,
        }));
        assert_eq!(id, 0);
    }

    #[test]
    fn test_value_store_add_null() {
        let mut store = ValueStore::new();
        let id = store.add(Value::NullValue(NullValue {
            type_id: 0,
            node: None,
        }));
        assert_eq!(id, 0);
    }

    #[test]
    fn test_value_type() {
        let v = Value::StringValue(StringValue {
            type_id: 5,
            value: "test".to_string(),
            scalar: None,
            node: None,
        });
        assert_eq!(v.value_type(), 5);
    }

    #[test]
    fn test_value_kind_name() {
        assert_eq!(
            Value::NullValue(NullValue {
                type_id: 0,
                node: None
            })
            .value_kind_name(),
            "NullValue"
        );
        assert_eq!(
            Value::BooleanValue(BooleanValue {
                type_id: 0,
                value: true,
                scalar: None,
                node: None
            })
            .value_kind_name(),
            "BooleanValue"
        );
    }

    // ========================================================================
    // DecoratorApplication tests
    // ========================================================================

    #[test]
    fn test_decorator_application_new() {
        let da = DecoratorApplication::new(1);
        assert_eq!(da.decorator, 1);
        assert!(da.definition.is_none());
        assert!(da.args.is_empty());
        assert!(da.node.is_none());
    }

    #[test]
    fn test_decorator_application_builder() {
        let da = DecoratorApplication::new(1).with_definition(2).with_node(3);
        assert_eq!(da.decorator, 1);
        assert_eq!(da.definition, Some(2));
        assert_eq!(da.node, Some(3));
    }

    // ========================================================================
    // IntrinsicTypeName tests
    // ========================================================================

    #[test]
    fn test_intrinsic_type_names() {
        assert_eq!(IntrinsicTypeName::ErrorType as u8, 0);
        assert_eq!(IntrinsicTypeName::Void as u8, 1);
        assert_eq!(IntrinsicTypeName::Never as u8, 2);
        assert_eq!(IntrinsicTypeName::Unknown as u8, 3);
        assert_eq!(IntrinsicTypeName::Null as u8, 4);
    }

    #[test]
    fn test_string_type() {
        let t = Type::String(StringType {
            id: 1,
            value: "hello".to_string(),
            node: None,
            is_finished: true,
        });
        assert_eq!(t.kind_name(), "String");
        assert_eq!(t.name(), Some("hello"));
    }

    #[test]
    fn test_numeric_type() {
        let t = Type::Number(NumericType {
            id: 2,
            value: 3.15,
            value_as_string: "3.15".to_string(),
            node: None,
            is_finished: true,
        });
        assert_eq!(t.kind_name(), "Number");
        assert_eq!(t.name(), Some("3.15"));
    }

    #[test]
    fn test_boolean_type() {
        let t = Type::Boolean(BooleanType {
            id: 3,
            value: true,
            node: None,
            is_finished: true,
        });
        assert_eq!(t.kind_name(), "Boolean");
        assert_eq!(t.name(), Some("true"));
    }
}
