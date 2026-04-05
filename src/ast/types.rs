//! Complete AST node type definitions for TypeSpec-Rust
//! Ported from TypeSpec compiler/src/core/types.ts

use super::node::NodeId;
use super::token::Span;
use std::collections::HashMap;

/// SyntaxKind enum - identifies the type of AST node
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SyntaxKind {
    // File structure
    TypeSpecScript,
    JsSourceFile,
    ImportStatement,

    // Identifiers and expressions
    Identifier,
    MemberExpression,
    StringLiteral,
    NumericLiteral,
    BooleanLiteral,

    // Model
    ModelStatement,
    ModelExpression,
    ModelProperty,
    ModelSpreadProperty,

    // Scalar
    ScalarStatement,
    ScalarConstructor,

    // Interface
    InterfaceStatement,

    // Union
    UnionStatement,
    UnionVariant,

    // Enum
    EnumStatement,
    EnumMember,
    EnumSpreadMember,

    // Namespace
    NamespaceStatement,
    UsingStatement,

    // Operation
    OperationStatement,
    OperationSignatureDeclaration,
    OperationSignatureReference,

    // Alias and const
    AliasStatement,
    ConstStatement,

    // Decorator
    DecoratorDeclarationStatement,
    DecoratorExpression,
    AugmentDecoratorStatement,

    // Expression types
    ArrayExpression,
    TupleExpression,
    UnionExpression,
    IntersectionExpression,
    TypeReference,
    CallExpression,
    ValueOfExpression,
    TypeOfExpression,

    // String template
    StringTemplateExpression,
    StringTemplateHead,
    StringTemplateMiddle,
    StringTemplateTail,
    StringTemplateSpan,

    // Object literal
    ObjectLiteral,
    ObjectLiteralProperty,
    ObjectLiteralSpreadProperty,
    ArrayLiteral,

    // Keywords
    ExternKeyword,
    InternalKeyword,
    VoidKeyword,
    NeverKeyword,
    UnknownKeyword,

    // Template
    TemplateParameterDeclaration,
    TemplateArgument,

    // Function
    FunctionDeclarationStatement,
    FunctionParameter,
    FunctionTypeExpression,

    // Directive
    DirectiveExpression,

    // Doc
    Doc,
    DocText,
    DocParamTag,
    DocPropTag,
    DocReturnsTag,
    DocErrorsTag,
    DocTemplateTag,
    DocUnknownTag,

    // Misc
    EmptyStatement,
    InvalidStatement,
    JsNamespaceDeclaration,

    // Modifier
    LineComment,
    BlockComment,
}

/// Node flags for tracking parse state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeFlags {
    None = 0,
    DescendantErrorsExamined = 1 << 0,
    ThisNodeHasError = 1 << 1,
    DescendantHasError = 1 << 2,
    Synthetic = 1 << 3,
}

/// Modifier flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModifierFlags {
    None = 0,
    Extern = 1 << 1,
    Internal = 1 << 2,
}

/// Base interface for all AST nodes
#[derive(Debug, Clone)]
pub struct BaseNode {
    pub id: NodeId,
    pub kind: SyntaxKind,
    pub flags: NodeFlags,
    pub span: Span,
    pub parent: Option<NodeId>,
}

// ============================================================================
// Expression Nodes
// ============================================================================

/// Identifier node - basic identifier like `Foo`
#[derive(Debug, Clone)]
pub struct Identifier {
    pub id: NodeId,
    pub span: Span,
    pub value: String,
}

/// Member expression - `foo.bar` or `foo::bar`
#[derive(Debug, Clone)]
pub struct MemberExpression {
    pub id: NodeId,
    pub span: Span,
    pub object: NodeId,       // Identifier or MemberExpression
    pub property: NodeId,     // Identifier
    pub selector: MemberSelector,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemberSelector {
    Dot,       // .
    DoubleColon, // ::
}

/// String literal - `"hello"`
#[derive(Debug, Clone)]
pub struct StringLiteral {
    pub id: NodeId,
    pub span: Span,
    pub value: String,
}

/// Numeric literal - `42`, `3.14`
#[derive(Debug, Clone)]
pub struct NumericLiteral {
    pub id: NodeId,
    pub span: Span,
    pub value: f64,
    pub value_as_string: String,
}

/// Boolean literal - `true`, `false`
#[derive(Debug, Clone)]
pub struct BooleanLiteral {
    pub id: NodeId,
    pub span: Span,
    pub value: bool,
}

/// Array expression - `string[]`
#[derive(Debug, Clone)]
pub struct ArrayExpression {
    pub id: NodeId,
    pub span: Span,
    pub element_type: NodeId,
}

/// Tuple expression - `[string, number]`
#[derive(Debug, Clone)]
pub struct TupleExpression {
    pub id: NodeId,
    pub span: Span,
    pub values: Vec<NodeId>,
}

/// Union expression - `string | null`
#[derive(Debug, Clone)]
pub struct UnionExpression {
    pub id: NodeId,
    pub span: Span,
    pub options: Vec<NodeId>,
}

/// Intersection expression - `Foo & Bar`
#[derive(Debug, Clone)]
pub struct IntersectionExpression {
    pub id: NodeId,
    pub span: Span,
    pub options: Vec<NodeId>,
}

/// Type reference - `MyType` or `Namespace.MyType`
#[derive(Debug, Clone)]
pub struct TypeReference {
    pub id: NodeId,
    pub span: Span,
    pub name: NodeId,         // Identifier or MemberExpression
    pub arguments: Vec<NodeId>,
}

/// Call expression - `foo(arg1, arg2)`
#[derive(Debug, Clone)]
pub struct CallExpression {
    pub id: NodeId,
    pub span: Span,
    pub target: NodeId,       // Identifier or MemberExpression
    pub arguments: Vec<NodeId>,
}

/// Value of expression - `valueof T`
#[derive(Debug, Clone)]
pub struct ValueOfExpression {
    pub id: NodeId,
    pub span: Span,
    pub target: NodeId,
}

/// Type of expression - `typeof expr`
#[derive(Debug, Clone)]
pub struct TypeOfExpression {
    pub id: NodeId,
    pub span: Span,
    pub target: NodeId,
}

// ============================================================================
// String Template
// ============================================================================

#[derive(Debug, Clone)]
pub struct StringTemplateExpression {
    pub id: NodeId,
    pub span: Span,
    pub head: NodeId,
    pub spans: Vec<NodeId>,
}

#[derive(Debug, Clone)]
pub struct StringTemplateHead {
    pub id: NodeId,
    pub span: Span,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct StringTemplateMiddle {
    pub id: NodeId,
    pub span: Span,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct StringTemplateTail {
    pub id: NodeId,
    pub span: Span,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct StringTemplateSpan {
    pub id: NodeId,
    pub span: Span,
    pub expression: NodeId,
    pub literal: NodeId,  // StringTemplateMiddle or StringTemplateTail
}

// ============================================================================
// Object Literal
// ============================================================================

#[derive(Debug, Clone)]
pub struct ObjectLiteral {
    pub id: NodeId,
    pub span: Span,
    pub properties: Vec<NodeId>,
}

#[derive(Debug, Clone)]
pub struct ObjectLiteralProperty {
    pub id: NodeId,
    pub span: Span,
    pub key: NodeId,
    pub value: NodeId,
}

#[derive(Debug, Clone)]
pub struct ObjectLiteralSpreadProperty {
    pub id: NodeId,
    pub span: Span,
    pub target: NodeId,
}

#[derive(Debug, Clone)]
pub struct ArrayLiteral {
    pub id: NodeId,
    pub span: Span,
    pub values: Vec<NodeId>,
}

// ============================================================================
// Keywords
// ============================================================================

#[derive(Debug, Clone)]
pub struct ExternKeyword {
    pub id: NodeId,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct InternalKeyword {
    pub id: NodeId,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct VoidKeyword {
    pub id: NodeId,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct NeverKeyword {
    pub id: NodeId,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct UnknownKeyword {
    pub id: NodeId,
    pub span: Span,
}

// ============================================================================
// Model
// ============================================================================

/// Model declaration - `model Foo { ... }`
#[derive(Debug, Clone)]
pub struct ModelDeclaration {
    pub id: NodeId,
    pub span: Span,
    pub name: NodeId,
    pub properties: Vec<NodeId>,
    pub extends: Option<NodeId>,     // Expression for extends clause
    pub is: Option<NodeId>,           // Expression for `model is` clause
    pub decorators: Vec<NodeId>,
    pub body_range: Span,
}

/// Model expression - inline model in type position
#[derive(Debug, Clone)]
pub struct ModelExpression {
    pub id: NodeId,
    pub span: Span,
    pub properties: Vec<NodeId>,
    pub body_range: Span,
}

/// Model property - `name: type`
#[derive(Debug, Clone)]
pub struct ModelProperty {
    pub id: NodeId,
    pub span: Span,
    pub name: NodeId,
    pub value: NodeId,               // Type expression
    pub decorators: Vec<NodeId>,
    pub optional: bool,
    pub default: Option<NodeId>,      // Default value expression
}

/// Model spread property - `...Target`
#[derive(Debug, Clone)]
pub struct ModelSpreadProperty {
    pub id: NodeId,
    pub span: Span,
    pub target: NodeId,
}

// ============================================================================
// Scalar
// ============================================================================

/// Scalar declaration - `scalar Foo extends string`
#[derive(Debug, Clone)]
pub struct ScalarDeclaration {
    pub id: NodeId,
    pub span: Span,
    pub name: NodeId,
    pub extends: Option<NodeId>,
    pub decorators: Vec<NodeId>,
    pub constructors: Vec<NodeId>,
    pub body_range: Span,
}

/// Scalar constructor - `constructor(...)`
#[derive(Debug, Clone)]
pub struct ScalarConstructor {
    pub id: NodeId,
    pub span: Span,
    pub name: NodeId,
    pub parameters: Vec<NodeId>,
}

// ============================================================================
// Interface
// ============================================================================

/// Interface declaration - `interface Foo { ... }`
#[derive(Debug, Clone)]
pub struct InterfaceDeclaration {
    pub id: NodeId,
    pub span: Span,
    pub name: NodeId,
    pub operations: Vec<NodeId>,
    pub extends: Vec<NodeId>,
    pub decorators: Vec<NodeId>,
    pub template_parameters: Vec<NodeId>,
    pub body_range: Span,
}

// ============================================================================
// Union
// ============================================================================

/// Union declaration - `union Foo { ... }`
#[derive(Debug, Clone)]
pub struct UnionDeclaration {
    pub id: NodeId,
    pub span: Span,
    pub name: NodeId,
    pub variants: Vec<NodeId>,
    pub decorators: Vec<NodeId>,
    pub template_parameters: Vec<NodeId>,
}

/// Union variant - individual variant in a union
#[derive(Debug, Clone)]
pub struct UnionVariant {
    pub id: NodeId,
    pub span: Span,
    pub name: Option<NodeId>,
    pub value: NodeId,
    pub decorators: Vec<NodeId>,
}

// ============================================================================
// Enum
// ============================================================================

/// Enum declaration - `enum Foo { ... }`
#[derive(Debug, Clone)]
pub struct EnumDeclaration {
    pub id: NodeId,
    pub span: Span,
    pub name: NodeId,
    pub members: Vec<NodeId>,
    pub decorators: Vec<NodeId>,
}

/// Enum member - individual member in an enum
#[derive(Debug, Clone)]
pub struct EnumMember {
    pub id: NodeId,
    pub span: Span,
    pub name: NodeId,
    pub value: Option<NodeId>,         // StringLiteral or NumericLiteral
    pub decorators: Vec<NodeId>,
}

/// Enum spread member - `...Target`
#[derive(Debug, Clone)]
pub struct EnumSpreadMember {
    pub id: NodeId,
    pub span: Span,
    pub target: NodeId,
}

// ============================================================================
// Namespace
// ============================================================================

/// Namespace declaration - `namespace Foo { ... }`
#[derive(Debug, Clone)]
pub struct NamespaceDeclaration {
    pub id: NodeId,
    pub span: Span,
    pub name: NodeId,
    pub statements: Vec<NodeId>,
    pub decorators: Vec<NodeId>,
    // Note: in TypeSpec, namespaces can be nested
}

/// Using declaration - `using Foo;`
#[derive(Debug, Clone)]
pub struct UsingDeclaration {
    pub id: NodeId,
    pub span: Span,
    pub name: NodeId,      // Identifier or MemberExpression
}

// ============================================================================
// Operation
// ============================================================================

/// Operation declaration - `op Foo(...): ReturnType;`
#[derive(Debug, Clone)]
pub struct OperationDeclaration {
    pub id: NodeId,
    pub span: Span,
    pub name: NodeId,
    pub signature: NodeId,         // OperationSignatureDeclaration or OperationSignatureReference
    pub decorators: Vec<NodeId>,
    pub template_parameters: Vec<NodeId>,
}

/// Operation signature declaration
#[derive(Debug, Clone)]
pub struct OperationSignatureDeclaration {
    pub id: NodeId,
    pub span: Span,
    pub parameters: NodeId,        // ModelExpression
    pub return_type: NodeId,
}

/// Operation signature reference - `op foo is Bar;`
#[derive(Debug, Clone)]
pub struct OperationSignatureReference {
    pub id: NodeId,
    pub span: Span,
    pub base_operation: NodeId,     // TypeReference
}

// ============================================================================
// Alias and Const
// ============================================================================

/// Alias statement - `alias Foo = Bar;`
#[derive(Debug, Clone)]
pub struct AliasStatement {
    pub id: NodeId,
    pub span: Span,
    pub name: NodeId,
    pub value: NodeId,
    pub template_parameters: Vec<NodeId>,
}

/// Const statement - `const foo = 42;`
#[derive(Debug, Clone)]
pub struct ConstStatement {
    pub id: NodeId,
    pub span: Span,
    pub name: NodeId,
    pub value: NodeId,
    pub type_annotation: Option<NodeId>,
}

// ============================================================================
// Decorator
// ============================================================================

/// Decorator declaration - `extern dec myDec(target: Type);`
#[derive(Debug, Clone)]
pub struct DecoratorDeclaration {
    pub id: NodeId,
    pub span: Span,
    pub name: NodeId,
    pub target: NodeId,            // FunctionParameter
    pub parameters: Vec<NodeId>,   // Additional parameters
}

/// Decorator expression - `@myDec(args)`
#[derive(Debug, Clone)]
pub struct DecoratorExpression {
    pub id: NodeId,
    pub span: Span,
    pub target: NodeId,            // Identifier or MemberExpression
    pub arguments: Vec<NodeId>,
}

/// Augment decorator statement - `@@myDec(args)` (at namespace level)
#[derive(Debug, Clone)]
pub struct AugmentDecoratorStatement {
    pub id: NodeId,
    pub span: Span,
    pub target: NodeId,            // Identifier or MemberExpression
    pub target_type: NodeId,       // TypeReference
    pub arguments: Vec<NodeId>,
}

// ============================================================================
// Function
// ============================================================================

/// Function declaration - `extern fn myFunc(...): ReturnType;`
#[derive(Debug, Clone)]
pub struct FunctionDeclaration {
    pub id: NodeId,
    pub span: Span,
    pub name: NodeId,
    pub parameters: Vec<NodeId>,
    pub return_type: Option<NodeId>,
}

/// Function parameter
#[derive(Debug, Clone)]
pub struct FunctionParameter {
    pub id: NodeId,
    pub span: Span,
    pub name: NodeId,
    pub type_annotation: Option<NodeId>,
    pub optional: bool,
    pub rest: bool,
}

/// Function type expression - `fn(...) => ...`
#[derive(Debug, Clone)]
pub struct FunctionTypeExpression {
    pub id: NodeId,
    pub span: Span,
    pub parameters: Vec<NodeId>,
    pub return_type: Option<NodeId>,
}

// ============================================================================
// Template
// ============================================================================

/// Template parameter declaration - `T` in `model Foo<T> { ... }`
#[derive(Debug, Clone)]
pub struct TemplateParameterDeclaration {
    pub id: NodeId,
    pub span: Span,
    pub name: NodeId,
    pub constraint: Option<NodeId>,
    pub default: Option<NodeId>,
}

/// Template argument - `string` in `Foo<string>`
#[derive(Debug, Clone)]
pub struct TemplateArgument {
    pub id: NodeId,
    pub span: Span,
    pub name: Option<NodeId>,      // Optional named argument
    pub argument: NodeId,
}

// ============================================================================
// Import / Export
// ============================================================================

/// Import statement - `import "./foo";`
#[derive(Debug, Clone)]
pub struct ImportStatement {
    pub id: NodeId,
    pub span: Span,
    pub path: NodeId,              // StringLiteral
}

// ============================================================================
// Directive
// ============================================================================

/// Directive expression - `#pragma ...`
#[derive(Debug, Clone)]
pub struct DirectiveExpression {
    pub id: NodeId,
    pub span: Span,
    pub target: NodeId,            // Identifier
    pub arguments: Vec<NodeId>,
}

// ============================================================================
// Doc Comments
// ============================================================================

#[derive(Debug, Clone)]
pub struct Doc {
    pub id: NodeId,
    pub span: Span,
    pub content: Vec<NodeId>,
    pub tags: Vec<NodeId>,
}

#[derive(Debug, Clone)]
pub struct DocText {
    pub id: NodeId,
    pub span: Span,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct DocParamTag {
    pub id: NodeId,
    pub span: Span,
    pub tag_name: NodeId,
    pub param_name: NodeId,
    pub content: Vec<NodeId>,
}

#[derive(Debug, Clone)]
pub struct DocPropTag {
    pub id: NodeId,
    pub span: Span,
    pub tag_name: NodeId,
    pub prop_name: NodeId,
    pub content: Vec<NodeId>,
}

#[derive(Debug, Clone)]
pub struct DocReturnsTag {
    pub id: NodeId,
    pub span: Span,
    pub tag_name: NodeId,
    pub content: Vec<NodeId>,
}

#[derive(Debug, Clone)]
pub struct DocErrorsTag {
    pub id: NodeId,
    pub span: Span,
    pub tag_name: NodeId,
    pub content: Vec<NodeId>,
}

#[derive(Debug, Clone)]
pub struct DocTemplateTag {
    pub id: NodeId,
    pub span: Span,
    pub tag_name: NodeId,
    pub param_name: NodeId,
    pub content: Vec<NodeId>,
}

#[derive(Debug, Clone)]
pub struct DocUnknownTag {
    pub id: NodeId,
    pub span: Span,
    pub tag_name: NodeId,
    pub content: Vec<NodeId>,
}

// ============================================================================
// Comments
// ============================================================================

#[derive(Debug, Clone)]
pub struct LineComment {
    pub id: NodeId,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct BlockComment {
    pub id: NodeId,
    pub span: Span,
    pub parsed_as_docs: bool,
}

// ============================================================================
// Miscellaneous
// ============================================================================

#[derive(Debug, Clone)]
pub struct EmptyStatement {
    pub id: NodeId,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct InvalidStatement {
    pub id: NodeId,
    pub span: Span,
    pub decorators: Vec<NodeId>,
}

#[derive(Debug, Clone)]
pub struct JsNamespaceDeclaration {
    pub id: NodeId,
    pub span: Span,
    pub name: NodeId,
}

// ============================================================================
// Root Node
// ============================================================================

/// TypeSpec script root node
#[derive(Debug, Clone)]
pub struct TypeSpecScript {
    pub id: NodeId,
    pub span: Span,
    pub statements: Vec<NodeId>,
    pub comments: Vec<NodeId>,
    pub parse_diagnostics: Vec<NodeId>,
}

// ============================================================================
// Modifier
// ============================================================================

#[derive(Debug, Clone)]
pub struct Modifier {
    pub id: NodeId,
    pub span: Span,
    pub kind: ModifierKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModifierKind {
    Extern,
    Internal,
}

// ============================================================================
// Declaration trait for named declarations
// ============================================================================

/// Trait for nodes that declare a name
pub trait Declaration {
    fn name(&self) -> NodeId;
    fn modifiers(&self) -> ModifierFlags;
}

// ============================================================================
// Template declaration (nodes that can have template parameters)
// ============================================================================

/// Trait for nodes that can have template parameters
pub trait TemplateDeclaration {
    fn template_parameters(&self) -> &[NodeId];
}
