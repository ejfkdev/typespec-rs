//! Code fix utilities for TypeSpec-Rust
//!
//! Ported from TypeSpec compiler/src/core/code-fixes.ts
//!
//! This module provides utilities for applying code fixes and edits.

/// An edit to insert text
#[derive(Debug, Clone)]
pub struct InsertTextCodeFixEdit {
    /// Position to insert at
    pub pos: usize,
    /// Text to insert
    pub text: String,
}

/// An edit to replace text
#[derive(Debug, Clone)]
pub struct ReplaceTextCodeFixEdit {
    /// Start position
    pub pos: usize,
    /// End position
    pub end: usize,
    /// Replacement text
    pub text: String,
}

/// A code fix edit
#[derive(Debug, Clone)]
pub enum CodeFixEdit {
    InsertText(InsertTextCodeFixEdit),
    ReplaceText(ReplaceTextCodeFixEdit),
}

/// Apply code fix edits to text content
///
/// # Arguments
/// * `content` - The original text content
/// * `edits` - List of edits to apply, sorted by position
///
/// # Returns
/// The modified text content
pub fn apply_code_fixes_on_text(content: &str, edits: &[CodeFixEdit]) -> String {
    let mut segments: Vec<String> = Vec::new();
    let mut last: usize = 0;

    // Sort edits by position
    let mut sorted_edits: Vec<&CodeFixEdit> = edits.iter().collect();
    sorted_edits.sort_by_key(|e| match e {
        CodeFixEdit::InsertText(e) => e.pos,
        CodeFixEdit::ReplaceText(e) => e.pos,
    });

    for edit in sorted_edits {
        match edit {
            CodeFixEdit::InsertText(edit) => {
                segments.push(content[last..edit.pos].to_string());
                segments.push(edit.text.clone());
                last = edit.pos;
            }
            CodeFixEdit::ReplaceText(edit) => {
                segments.push(content[last..edit.pos].to_string());
                segments.push(edit.text.clone());
                last = edit.end;
            }
        }
    }

    segments.push(content[last..].to_string());
    segments.join("")
}

/// Create a code fix context for building edits
pub struct CodeFixContext {
    edits: Vec<CodeFixEdit>,
}

impl CodeFixContext {
    /// Create a new code fix context
    pub fn new() -> Self {
        Self { edits: Vec::new() }
    }

    /// Prepend text at a position
    pub fn prepend_text(&mut self, pos: usize, text: &str) -> CodeFixEdit {
        let edit = CodeFixEdit::InsertText(InsertTextCodeFixEdit {
            pos,
            text: text.to_string(),
        });
        self.edits.push(edit.clone());
        edit
    }

    /// Append text after a position
    pub fn append_text(&mut self, end: usize, text: &str) -> CodeFixEdit {
        let edit = CodeFixEdit::InsertText(InsertTextCodeFixEdit {
            pos: end,
            text: text.to_string(),
        });
        self.edits.push(edit.clone());
        edit
    }

    /// Replace text between positions
    pub fn replace_text(&mut self, pos: usize, end: usize, text: &str) -> CodeFixEdit {
        let edit = CodeFixEdit::ReplaceText(ReplaceTextCodeFixEdit {
            pos,
            end,
            text: text.to_string(),
        });
        self.edits.push(edit.clone());
        edit
    }

    /// Get all edits collected
    pub fn into_edits(self) -> Vec<CodeFixEdit> {
        self.edits
    }
}

impl Default for CodeFixContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_insert_text() {
        let content = "hello world";
        let edits = [CodeFixEdit::InsertText(InsertTextCodeFixEdit {
            pos: 5,
            text: " beautiful".to_string(),
        })];
        let result = apply_code_fixes_on_text(content, &edits);
        assert_eq!(result, "hello beautiful world");
    }

    #[test]
    fn test_apply_replace_text() {
        let content = "hello world";
        let edits = [CodeFixEdit::ReplaceText(ReplaceTextCodeFixEdit {
            pos: 0,
            end: 5,
            text: "hi".to_string(),
        })];
        let result = apply_code_fixes_on_text(content, &edits);
        assert_eq!(result, "hi world");
    }

    #[test]
    fn test_apply_multiple_edits() {
        let content = "hello world";
        // Edits are applied to original content in sorted order
        // Replace "hello" (0-5) with "hi", then insert " again" at original position 7
        // After Replace: "hi world", then content.slice(5, 7) = " w", so "hi" + " w" + " again" + "orld"
        let edits = [
            CodeFixEdit::ReplaceText(ReplaceTextCodeFixEdit {
                pos: 0,
                end: 5,
                text: "hi".to_string(),
            }),
            CodeFixEdit::InsertText(InsertTextCodeFixEdit {
                pos: 7,
                text: " again".to_string(),
            }),
        ];
        let result = apply_code_fixes_on_text(content, &edits);
        assert_eq!(result, "hi w againorld");
    }

    #[test]
    fn test_apply_empty_edits() {
        let content = "hello world";
        let edits: [CodeFixEdit; 0] = [];
        let result = apply_code_fixes_on_text(content, &edits);
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_apply_unsorted_edits() {
        let content = "hello world";
        // Edits are sorted by position and applied to original content
        // Replace "hello" (0-5) with "hi", then insert at original position 7
        let edits = [
            CodeFixEdit::InsertText(InsertTextCodeFixEdit {
                pos: 7,
                text: " beautiful".to_string(),
            }),
            CodeFixEdit::ReplaceText(ReplaceTextCodeFixEdit {
                pos: 0,
                end: 5,
                text: "hi".to_string(),
            }),
        ];
        let result = apply_code_fixes_on_text(content, &edits);
        // Result: "hi" + " w" + " beautiful" + "orld"
        assert_eq!(result, "hi w beautifulorld");
    }

    #[test]
    fn test_code_fix_context_new() {
        let ctx = CodeFixContext::new();
        assert!(ctx.edits.is_empty());
    }

    #[test]
    fn test_code_fix_context_prepend_text() {
        let mut ctx = CodeFixContext::new();
        let _ = ctx.prepend_text(5, " inserted");
        assert_eq!(ctx.edits.len(), 1);
    }

    #[test]
    fn test_code_fix_context_replace_text() {
        let mut ctx = CodeFixContext::new();
        let _ = ctx.replace_text(0, 5, "replaced");
        assert_eq!(ctx.edits.len(), 1);
    }

    #[test]
    fn test_code_fix_context_into_edits() {
        let mut ctx = CodeFixContext::new();
        ctx.prepend_text(5, "text");
        let edits = ctx.into_edits();
        assert_eq!(edits.len(), 1);
    }

    // ==================== Ported from TS codefixes.test.ts ====================

    #[test]
    fn test_prepend_at_pos() {
        // TS: "apply prepend fix at pos" — "abcdef" prepend "123" at pos 3 → "abc123def"
        let mut ctx = CodeFixContext::new();
        ctx.prepend_text(3, "123");
        let edits = ctx.into_edits();
        let result = apply_code_fixes_on_text("abcdef", &edits);
        assert_eq!(result, "abc123def");
    }

    #[test]
    fn test_replace_at_range() {
        // TS: "apply replace fix at pos" — "abcdef" replace (3,5) with "123" → "abc123f"
        let mut ctx = CodeFixContext::new();
        ctx.replace_text(3, 5, "123");
        let edits = ctx.into_edits();
        let result = apply_code_fixes_on_text("abcdef", &edits);
        assert_eq!(result, "abc123f");
    }

    #[test]
    fn test_prepend_multiple_items() {
        // TS: "prepend multiple items" — "abc" prepend "123" at 1, prepend "456" at 2 → "a123b456c"
        let mut ctx = CodeFixContext::new();
        ctx.prepend_text(1, "123");
        ctx.prepend_text(2, "456");
        let edits = ctx.into_edits();
        let result = apply_code_fixes_on_text("abc", &edits);
        assert_eq!(result, "a123b456c");
    }

    #[test]
    fn test_prepend_multiple_items_out_of_order() {
        // TS: "prepend multiple items out of order" — same result
        let mut ctx = CodeFixContext::new();
        ctx.prepend_text(2, "456");
        ctx.prepend_text(1, "123");
        let edits = ctx.into_edits();
        let result = apply_code_fixes_on_text("abc", &edits);
        assert_eq!(result, "a123b456c");
    }

    #[test]
    fn test_replace_multiple_items() {
        // TS: "replace multiple items" — "abc" replace (1,2)→"123", replace (2,3)→"456" → "a123456"
        let mut ctx = CodeFixContext::new();
        ctx.replace_text(1, 2, "123");
        ctx.replace_text(2, 3, "456");
        let edits = ctx.into_edits();
        let result = apply_code_fixes_on_text("abc", &edits);
        assert_eq!(result, "a123456");
    }

    #[test]
    fn test_replace_multiple_items_out_of_order() {
        // TS: "replace multiple items out of order" — same result
        let mut ctx = CodeFixContext::new();
        ctx.replace_text(2, 3, "456");
        ctx.replace_text(1, 2, "123");
        let edits = ctx.into_edits();
        let result = apply_code_fixes_on_text("abc", &edits);
        assert_eq!(result, "a123456");
    }
}
