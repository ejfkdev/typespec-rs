//! Modifiers for TypeSpec-Rust
//!
//! Ported from TypeSpec compiler/src/core/modifiers.ts
//!
//! This module handles modifiers like `extern` and `internal` for declarations.

use crate::ast::types::{ModifierKind, SyntaxKind};

/// Modifier flags - bitflags for modifier types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModifierFlags(u32);

#[allow(non_upper_case_globals)]
impl ModifierFlags {
    /// No modifiers
    pub const None: Self = ModifierFlags(0);
    /// Extern modifier
    pub const Extern: Self = ModifierFlags(1 << 0);
    /// Internal modifier
    pub const Internal: Self = ModifierFlags(1 << 1);
    /// All modifiers (for checking)
    pub const All: Self = ModifierFlags(Self::Extern.0 | Self::Internal.0);

    pub fn bits(&self) -> u32 {
        self.0
    }

    pub fn from_bits(bits: u32) -> Self {
        Self(bits)
    }

    pub fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

impl std::ops::BitOr for ModifierFlags {
    type Output = Self;

    fn bitor(self, other: Self) -> Self::Output {
        Self(self.0 | other.0)
    }
}

impl std::ops::BitAnd for ModifierFlags {
    type Output = Self;

    fn bitand(self, other: Self) -> Self::Output {
        Self(self.0 & other.0)
    }
}

/// Check if modifiers are valid for a declaration node
/// Ported from TS checker.ts checkModifiers() / modifiers.ts
pub fn check_modifiers(
    modifier_flags: ModifierFlags,
    node_kind: SyntaxKind,
) -> ModifierCheckResult {
    let compatibility = get_modifier_compatibility(node_kind);

    let mut invalid_modifiers = Vec::new();
    let mut missing_modifiers = Vec::new();

    // Check for modifiers not allowed on this node type
    let invalid_flags = ModifierFlags(modifier_flags.bits() & !compatibility.allowed.bits());
    if invalid_flags.bits() != 0 {
        if invalid_flags.contains(ModifierFlags::Internal) {
            invalid_modifiers.push("internal".to_string());
        }
        if invalid_flags.contains(ModifierFlags::Extern) {
            invalid_modifiers.push("extern".to_string());
        }
    }

    // Check for missing required modifiers
    let missing_flags = ModifierFlags(compatibility.required.bits() & !modifier_flags.bits());
    if missing_flags.bits() != 0 {
        if missing_flags.contains(ModifierFlags::Extern) {
            missing_modifiers.push("extern".to_string());
        }
        if missing_flags.contains(ModifierFlags::Internal) {
            missing_modifiers.push("internal".to_string());
        }
    }

    let is_valid = invalid_modifiers.is_empty() && missing_modifiers.is_empty();

    ModifierCheckResult {
        is_valid,
        invalid_modifiers,
        missing_modifiers,
    }
}

/// Modifier compatibility for a declaration type
struct ModifierCompatibility {
    allowed: ModifierFlags,
    required: ModifierFlags,
}

/// Get the modifier compatibility for a declaration node kind
/// Ported from TS modifiers.ts SYNTAX_MODIFIERS
fn get_modifier_compatibility(kind: SyntaxKind) -> ModifierCompatibility {
    match kind {
        // Namespace: no modifiers allowed
        SyntaxKind::NamespaceStatement => ModifierCompatibility {
            allowed: ModifierFlags::None,
            required: ModifierFlags::None,
        },
        // Most declarations: internal allowed, none required
        SyntaxKind::ModelStatement
        | SyntaxKind::ScalarStatement
        | SyntaxKind::InterfaceStatement
        | SyntaxKind::UnionStatement
        | SyntaxKind::EnumStatement
        | SyntaxKind::AliasStatement
        | SyntaxKind::ConstStatement
        | SyntaxKind::OperationStatement => ModifierCompatibility {
            allowed: ModifierFlags::Internal,
            required: ModifierFlags::None,
        },
        // dec/fn: both extern and internal allowed, extern required
        SyntaxKind::DecoratorDeclarationStatement | SyntaxKind::FunctionDeclarationStatement => {
            ModifierCompatibility {
                allowed: ModifierFlags::All,
                required: ModifierFlags::Extern,
            }
        }
        _ => ModifierCompatibility {
            allowed: ModifierFlags::None,
            required: ModifierFlags::None,
        },
    }
}

/// Result of modifier checking
#[derive(Debug, Clone)]
pub struct ModifierCheckResult {
    pub is_valid: bool,
    pub invalid_modifiers: Vec<String>,
    pub missing_modifiers: Vec<String>,
}

/// Convert modifier kind to flag
pub fn modifier_to_flag(kind: ModifierKind) -> ModifierFlags {
    match kind {
        ModifierKind::Extern => ModifierFlags::Extern,
        ModifierKind::Internal => ModifierFlags::Internal,
    }
}

/// Get the text for a declaration kind
pub fn get_declaration_kind_text(kind: SyntaxKind) -> &'static str {
    match kind {
        SyntaxKind::NamespaceStatement => "namespace",
        SyntaxKind::OperationStatement => "op",
        SyntaxKind::ModelStatement => "model",
        SyntaxKind::ScalarStatement => "scalar",
        SyntaxKind::InterfaceStatement => "interface",
        SyntaxKind::UnionStatement => "union",
        SyntaxKind::EnumStatement => "enum",
        SyntaxKind::AliasStatement => "alias",
        SyntaxKind::DecoratorDeclarationStatement => "dec",
        SyntaxKind::FunctionDeclarationStatement => "function",
        SyntaxKind::ConstStatement => "const",
        _ => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modifier_flags_none() {
        assert_eq!(ModifierFlags::None.bits(), 0);
        assert!(!ModifierFlags::None.contains(ModifierFlags::Extern));
    }

    #[test]
    fn test_modifier_flags_extern() {
        assert!(ModifierFlags::Extern.contains(ModifierFlags::Extern));
        assert!(!ModifierFlags::Extern.contains(ModifierFlags::Internal));
    }

    #[test]
    fn test_modifier_flags_internal() {
        assert!(ModifierFlags::Internal.contains(ModifierFlags::Internal));
        assert!(!ModifierFlags::Internal.contains(ModifierFlags::Extern));
    }

    #[test]
    fn test_modifier_flags_all() {
        let all = ModifierFlags::All;
        assert!(all.contains(ModifierFlags::Extern));
        assert!(all.contains(ModifierFlags::Internal));
    }

    #[test]
    fn test_modifier_flags_bitor() {
        let combined = ModifierFlags::Extern | ModifierFlags::Internal;
        assert!(combined.contains(ModifierFlags::Extern));
        assert!(combined.contains(ModifierFlags::Internal));
    }

    #[test]
    fn test_modifier_to_flag() {
        assert_eq!(
            modifier_to_flag(ModifierKind::Extern),
            ModifierFlags::Extern
        );
        assert_eq!(
            modifier_to_flag(ModifierKind::Internal),
            ModifierFlags::Internal
        );
    }

    #[test]
    fn test_get_declaration_kind_text() {
        assert_eq!(
            get_declaration_kind_text(SyntaxKind::ModelStatement),
            "model"
        );
        assert_eq!(
            get_declaration_kind_text(SyntaxKind::NamespaceStatement),
            "namespace"
        );
        assert_eq!(
            get_declaration_kind_text(SyntaxKind::InterfaceStatement),
            "interface"
        );
        assert_eq!(get_declaration_kind_text(SyntaxKind::EnumStatement), "enum");
        assert_eq!(
            get_declaration_kind_text(SyntaxKind::UnionStatement),
            "union"
        );
    }

    #[test]
    fn test_check_modifiers_valid() {
        let result = check_modifiers(ModifierFlags::Internal, SyntaxKind::ModelStatement);
        assert!(result.is_valid);
    }
}
