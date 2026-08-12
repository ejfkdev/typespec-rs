//! Modifiers for TypeSpec-RS
//!
//! Ported from TypeSpec compiler/src/core/modifiers.ts
//!
//! This module handles modifiers like `extern`, `internal`, and `auto` for
//! declarations.

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
    /// Auto modifier (auto decorators, microsoft/typespec#10197)
    pub const Auto: Self = ModifierFlags(1 << 2);
    /// All modifiers (for checking)
    pub const All: Self = ModifierFlags(Self::Extern.0 | Self::Internal.0 | Self::Auto.0);

    pub fn bits(&self) -> u32 {
        self.0
    }

    pub fn from_bits(bits: u32) -> Self {
        Self(bits)
    }

    pub fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    pub fn intersects(self, other: Self) -> bool {
        (self.0 & other.0) != 0
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

/// A single invalid-modifier problem found by [`check_modifiers`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModifierProblem {
    /// Modifier is not allowed on this declaration kind.
    NotAllowed { modifier: String },
    /// A single required modifier is missing.
    MissingRequired { modifier: String },
    /// None of the (alternatives of) required modifiers are present.
    MissingRequiredOneOf { modifiers: Vec<String> },
    /// Two mutually exclusive modifiers were combined.
    MutuallyExclusive {
        modifier_a: String,
        modifier_b: String,
    },
}

/// Check if modifiers are valid for a declaration node.
///
/// Ported from TS modifiers.ts `checkModifiers`. `required` means "at least
/// one of these flags must be present" (microsoft/typespec#10197 changed the
/// dec declaration to accept `extern` or `auto`).
pub fn check_modifiers(
    modifier_flags: ModifierFlags,
    node_kind: SyntaxKind,
) -> ModifierCheckResult {
    let compatibility = get_modifier_compatibility(node_kind);

    let mut problems = Vec::new();

    // Modifiers not allowed on this node type
    let invalid_flags = ModifierFlags(modifier_flags.bits() & !compatibility.allowed.bits());
    for name in get_names_of_modifier_flags(invalid_flags) {
        problems.push(ModifierProblem::NotAllowed { modifier: name });
    }

    // At least one of the required modifiers must be present.
    if compatibility.required.bits() != 0 && !modifier_flags.intersects(compatibility.required) {
        let names = get_names_of_modifier_flags(compatibility.required);
        if names.len() == 1 {
            problems.push(ModifierProblem::MissingRequired {
                modifier: names.into_iter().next().unwrap(),
            });
        } else {
            problems.push(ModifierProblem::MissingRequiredOneOf { modifiers: names });
        }
    }

    // Mutually exclusive modifier pairs.
    for (a, b) in &compatibility.mutually_exclusive {
        if modifier_flags.intersects(*a) && modifier_flags.intersects(*b) {
            problems.push(ModifierProblem::MutuallyExclusive {
                modifier_a: get_names_of_modifier_flags(*a)
                    .into_iter()
                    .next()
                    .unwrap_or_default(),
                modifier_b: get_names_of_modifier_flags(*b)
                    .into_iter()
                    .next()
                    .unwrap_or_default(),
            });
        }
    }

    ModifierCheckResult {
        is_valid: problems.is_empty(),
        problems,
    }
}

/// Modifier compatibility for a declaration type.
///
/// Ported from TS `ModifierCompatibility`.
struct ModifierCompatibility {
    /// Flags allowed on the node type.
    allowed: ModifierFlags,
    /// At least one of these flags must be present.
    required: ModifierFlags,
    /// Pairs of flags that cannot be used together.
    mutually_exclusive: Vec<(ModifierFlags, ModifierFlags)>,
}

/// Get the modifier compatibility for a declaration node kind
/// Ported from TS modifiers.ts SYNTAX_MODIFIERS
fn get_modifier_compatibility(kind: SyntaxKind) -> ModifierCompatibility {
    match kind {
        // Namespace: no modifiers allowed
        SyntaxKind::NamespaceStatement => ModifierCompatibility {
            allowed: ModifierFlags::None,
            required: ModifierFlags::None,
            mutually_exclusive: Vec::new(),
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
            mutually_exclusive: Vec::new(),
        },
        // dec: extern or auto required (mutually exclusive), internal allowed
        SyntaxKind::DecoratorDeclarationStatement => ModifierCompatibility {
            allowed: ModifierFlags::All,
            required: ModifierFlags::Extern | ModifierFlags::Auto,
            mutually_exclusive: vec![(ModifierFlags::Extern, ModifierFlags::Auto)],
        },
        // fn: extern required, internal allowed; auto not allowed
        SyntaxKind::FunctionDeclarationStatement => ModifierCompatibility {
            allowed: ModifierFlags::Extern | ModifierFlags::Internal,
            required: ModifierFlags::Extern,
            mutually_exclusive: Vec::new(),
        },
        _ => ModifierCompatibility {
            allowed: ModifierFlags::None,
            required: ModifierFlags::None,
            mutually_exclusive: Vec::new(),
        },
    }
}

/// Result of modifier checking
#[derive(Debug, Clone)]
pub struct ModifierCheckResult {
    pub is_valid: bool,
    pub problems: Vec<ModifierProblem>,
}

/// Convert modifier kind to flag
pub fn modifier_to_flag(kind: ModifierKind) -> ModifierFlags {
    match kind {
        ModifierKind::Extern => ModifierFlags::Extern,
        ModifierKind::Internal => ModifierFlags::Internal,
        ModifierKind::Auto => ModifierFlags::Auto,
    }
}

/// Get the names of the modifiers represented by the given flags.
///
/// Ported from TS `getNamesOfModifierFlags`.
pub fn get_names_of_modifier_flags(flags: ModifierFlags) -> Vec<String> {
    let mut names = Vec::new();
    if flags.contains(ModifierFlags::Extern) {
        names.push("extern".to_string());
    }
    if flags.contains(ModifierFlags::Internal) {
        names.push("internal".to_string());
    }
    if flags.contains(ModifierFlags::Auto) {
        names.push("auto".to_string());
    }
    names
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
    fn test_modifier_flags_auto() {
        assert!(ModifierFlags::Auto.contains(ModifierFlags::Auto));
        assert!(ModifierFlags::All.contains(ModifierFlags::Auto));
    }

    #[test]
    fn test_modifier_flags_all() {
        let all = ModifierFlags::All;
        assert!(all.contains(ModifierFlags::Extern));
        assert!(all.contains(ModifierFlags::Internal));
        assert!(all.contains(ModifierFlags::Auto));
    }

    #[test]
    fn test_modifier_flags_bitor() {
        let combined = ModifierFlags::Extern | ModifierFlags::Internal;
        assert!(combined.contains(ModifierFlags::Extern));
        assert!(combined.contains(ModifierFlags::Internal));
    }

    #[test]
    fn test_dec_requires_extern_or_auto() {
        // No modifier: missing one of extern/auto
        let result = check_modifiers(
            ModifierFlags::None,
            SyntaxKind::DecoratorDeclarationStatement,
        );
        assert!(!result.is_valid);
        assert_eq!(
            result.problems,
            vec![ModifierProblem::MissingRequiredOneOf {
                modifiers: vec!["extern".to_string(), "auto".to_string()],
            }]
        );

        // extern alone is fine
        let result = check_modifiers(
            ModifierFlags::Extern,
            SyntaxKind::DecoratorDeclarationStatement,
        );
        assert!(result.is_valid, "problems: {:?}", result.problems);

        // auto alone is fine
        let result = check_modifiers(
            ModifierFlags::Auto,
            SyntaxKind::DecoratorDeclarationStatement,
        );
        assert!(result.is_valid, "problems: {:?}", result.problems);
    }

    #[test]
    fn test_dec_extern_auto_mutually_exclusive() {
        let result = check_modifiers(
            ModifierFlags::Extern | ModifierFlags::Auto,
            SyntaxKind::DecoratorDeclarationStatement,
        );
        assert!(!result.is_valid);
        assert!(
            result
                .problems
                .contains(&ModifierProblem::MutuallyExclusive {
                    modifier_a: "extern".to_string(),
                    modifier_b: "auto".to_string(),
                })
        );
    }

    #[test]
    fn test_auto_not_allowed_on_model() {
        let result = check_modifiers(ModifierFlags::Auto, SyntaxKind::ModelStatement);
        assert!(!result.is_valid);
        assert_eq!(
            result.problems,
            vec![ModifierProblem::NotAllowed {
                modifier: "auto".to_string(),
            }]
        );
    }

    #[test]
    fn test_auto_not_allowed_on_function() {
        let result = check_modifiers(
            ModifierFlags::Auto | ModifierFlags::Extern,
            SyntaxKind::FunctionDeclarationStatement,
        );
        assert!(!result.is_valid);
        assert!(result.problems.contains(&ModifierProblem::NotAllowed {
            modifier: "auto".to_string(),
        }));
    }

    #[test]
    fn test_fn_requires_extern() {
        let result = check_modifiers(
            ModifierFlags::None,
            SyntaxKind::FunctionDeclarationStatement,
        );
        assert!(!result.is_valid);
        assert_eq!(
            result.problems,
            vec![ModifierProblem::MissingRequired {
                modifier: "extern".to_string(),
            }]
        );
    }
}
