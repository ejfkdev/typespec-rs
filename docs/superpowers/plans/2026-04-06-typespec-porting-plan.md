# TypeSpec Rust 移植计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将 TypeSpec 核心编译器、语言特性包、emitter-framework、asset-emitter 从 TypeScript 移植到 Rust

**Architecture:** 分阶段移植，采用 Rust 的模块系统组织代码。先移植核心基础设施（AST、类型系统、解析器），再移植语言特性包，最后移植发射器框架。每个阶段独立可测试。

**Tech Stack:** Rust (edition 2024), Cargo, proc-macro2, quote, syn

---

## 阶段概览

| 阶段 | 内容 | 任务数 |
|------|------|--------|
| Phase 1 | 核心基础设施 (AST, Types, Program) | 12 |
| Phase 2 | 解析器与扫描器 (Scanner, Parser) | 8 |
| Phase 3 | 编译器核心 (Binder, Checker, NameResolver) | 10 |
| Phase 4 | 标准库与工具 (lib/, std/, config, formatter) | 6 |
| Phase 5 | 语言特性包 (http, rest, versioning) | 8 |
| Phase 6 | 其他语言包 (openapi, json-schema, xml, protobuf) | 6 |
| Phase 7 | 发射器框架 (emitter-framework, asset-emitter) | 6 |

---

## Phase 1: 核心基础设施

### Task 1.1: 项目结构初始化

**Files:**
- Create: `Cargo.toml` (update existing)
- Create: `src/ast/mod.rs`
- Create: `src/ast/token.rs`
- Create: `src/ast/node.rs`
- Create: `src/types/mod.rs`
- Create: `src/program/mod.rs`

- [ ] **Step 1: 更新 Cargo.toml 添加必要依赖**

```toml
[package]
name = "typespec-rs"
version = "0.1.0"
edition = "2024"

[dependencies]
proc-macro2 = "1.0"
quote = "1.0"
syn = "2.0"
thiserror = "1.0"
anyhow = "1.0"
```

- [ ] **Step 2: 创建 AST 模块基础结构**

```rust
// src/ast/mod.rs
pub mod token;
pub mod node;

pub use token::*;
pub use node::*;
```

- [ ] **Step 3: 创建 Token 定义**

```rust
// src/ast/token.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    // Identifiers and literals
    Identifier,
    StringLiteral,
    NumericLiteral,
    // Keywords
    Model,
    Enum,
    Interface,
    Union,
    Namespace,
    Using,
    // ... more
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    pub value: String,
}

#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub line: u32,
    pub column: u32,
}
```

- [ ] **Step 4: 创建 Node 基类**

```rust
// src/ast/node.rs
use super::token::Span;

#[derive(Debug, Clone)]
pub struct Node {
    pub id: NodeId,
    pub kind: NodeKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum NodeKind {
    Identifier,
    TypeDeclaration,
    // ... more
}

pub type NodeId = u32;
```

- [ ] **Step 5: 创建 Types 模块基础**

```rust
// src/types/mod.rs
pub mod builder;
pub mod checker;

pub use builder::*;
pub use checker::*;
```

- [ ] **Step 6: 创建 Program 结构**

```rust
// src/program/mod.rs
use std::collections::HashMap;
use crate::ast::{Node, NodeId};

pub struct Program {
    pub nodes: HashMap<NodeId, Node>,
    pub source_files: HashMap<String, SourceFile>,
}

pub struct SourceFile {
    pub path: String,
    pub content: String,
    pub ast: NodeId,
}
```

- [ ] **Step 7: 验证编译**

Run: `cargo check`
Expected: PASS (warnings about unused code ok)

---

### Task 1.2: AST 完整定义

**Files:**
- Create: `src/ast/types.rs` (complete AST node types)
- Create: `src/ast/visitor.rs`

- [ ] **Step 1: 从 TypeSpec 文档和源码提取完整类型定义**

需要移植的 AST 节点类型：
- `Identifier`, `MemberExpression`
- `ModelDeclaration`, `ModelProperty`, `ModelIndexer`
- `EnumDeclaration`, `EnumMember`
- `InterfaceDeclaration`, `InterfaceMember`
- `UnionDeclaration`, `UnionVariant`
- `NamespaceDeclaration`, `UsingDeclaration`
- `OperationDeclaration`, `OperationSignature`
- `DecoratorDeclaration`, `DecoratorExpression`
- `ImportDeclaration`, `ExportDeclaration`

- [ ] **Step 2: 创建 Visitor trait**

```rust
// src/ast/visitor.rs
pub trait Visitor {
    fn visit_node(&mut self, node: &Node) {}
    fn visit_model(&mut self, model: &ModelDeclaration) {}
    fn visit_property(&mut self, property: &ModelProperty) {}
    // ... more visitor methods
}
```

- [ ] **Step 3: 创建测试文件**

```rust
// src/ast/tests.rs
#[cfg(test)]
mod tests {
    #[test]
    fn test_node_creation() {
        // TODO: Add tests
    }
}
```

---

### Task 1.3: Type 系统完整定义

**Files:**
- Create: `src/types/primitive.rs`
- Create: `src/types/model.rs`
- Create: `src/types/enum.rs`
- Create: `src/types/union.rs`
- Create: `src/types/interface.rs`
- Create: `src/types/namespace.rs`
- Create: `src/types/operation.rs`
- Create: `src/types/decorator.rs`

---

### Task 1.4: 标准库定义

**Files:**
- Create: `src/std/mod.rs`
- Create: `src/std/primitives.rs`
- Create: `src/std/helpers.rs`

---

## Phase 2: 解析器与扫描器

### Task 2.1: Scanner 实现

**Files:**
- Create: `src/scanner/mod.rs`
- Create: `src/scanner/lexer.rs`

### Task 2.2: Parser 实现

**Files:**
- Create: `src/parser/mod.rs`
- Create: `src/parser/ast_builder.rs`

### Task 2.3: Parser 测试

**Files:**
- Create: `src/parser/tests.rs`

---

## Phase 3: 编译器核心

### Task 3.1: Binder 实现

**Files:**
- Create: `src/binder/mod.rs`

### Task 3.2: Checker 实现

**Files:**
- Create: `src/checker/mod.rs`
- Create: `src/checker/type_relation.rs`

### Task 3.3: NameResolver 实现

**Files:**
- Create: `src/resolver/mod.rs`

---

## Phase 4-7: (详细任务待续)

后续阶段计划在完成 Phase 1-3 后继续详细规划。

---

## 依赖关系图

```
Phase 1 (基础设施)
    │
    ├─ AST ──────────────┐
    ├─ Types ────────────┤
    └─ Program ──────────┤
           │             │
           ▼             ▼
Phase 2 (解析器)     Phase 3 (编译器)
    │                    │
    └────────┬────────────┘
             │
             ▼
Phase 4 (标准库/工具)
             │
             ▼
Phase 5-6 (语言特性包)
             │
             ▼
Phase 7 (发射器框架)
```

---

**Plan Status:** Draft - 需要用户确认后开始执行
