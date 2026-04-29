//! Paging decorator implementations and data structures
//!
//! Ported from TypeSpec compiler/lib/std/decorators.tsp (paging section)
//! and compiler/lib/paging.ts
//!
//! Provides:
//! - Paging state keys (STATE_LIST, STATE_OFFSET, etc.)
//! - Paging decorator apply/is functions (@list, @offset, @pageIndex, etc.)
//! - Paging data structures (PagingProperty, PagingInput, PagingOutput, PagingOperation)
//! - Paging utility functions (get_paging_property_kind, has_multiple_paging_markers)

use crate::checker::types::TypeId;
use crate::state_accessors::StateAccessors;

// ============================================================================
// State keys for paging decorators
// ============================================================================

/// State key for @list decorator (paging)
pub const STATE_LIST: &str = "TypeSpec.list";
/// State key for @offset decorator (paging)
pub const STATE_OFFSET: &str = "TypeSpec.offset";
/// State key for @pageIndex decorator (paging)
pub const STATE_PAGE_INDEX: &str = "TypeSpec.pageIndex";
/// State key for @pageSize decorator (paging)
pub const STATE_PAGE_SIZE: &str = "TypeSpec.pageSize";
/// State key for @pageItems decorator (paging)
pub const STATE_PAGE_ITEMS: &str = "TypeSpec.pageItems";
/// State key for @continuationToken decorator (paging)
pub const STATE_CONTINUATION_TOKEN: &str = "TypeSpec.continuationToken";
/// State key for @nextLink decorator (paging)
pub const STATE_NEXT_LINK: &str = "TypeSpec.nextLink";
/// State key for @prevLink decorator (paging)
pub const STATE_PREV_LINK: &str = "TypeSpec.prevLink";
/// State key for @firstLink decorator (paging)
pub const STATE_FIRST_LINK: &str = "TypeSpec.firstLink";
/// State key for @lastLink decorator (paging)
pub const STATE_LAST_LINK: &str = "TypeSpec.lastLink";

// ============================================================================
// Paging decorator apply/is functions
// ============================================================================

flag_decorator!(apply_list, is_list, STATE_LIST);
flag_decorator!(apply_page_items, is_page_items, STATE_PAGE_ITEMS);
flag_decorator!(
    apply_continuation_token,
    is_continuation_token,
    STATE_CONTINUATION_TOKEN
);
flag_decorator!(apply_next_link, is_next_link, STATE_NEXT_LINK);
flag_decorator!(apply_prev_link, is_prev_link, STATE_PREV_LINK);
flag_decorator!(apply_first_link, is_first_link, STATE_FIRST_LINK);
flag_decorator!(apply_last_link, is_last_link, STATE_LAST_LINK);
flag_decorator!(apply_offset, is_offset, STATE_OFFSET);
flag_decorator!(apply_page_index, is_page_index, STATE_PAGE_INDEX);
flag_decorator!(apply_page_size, is_page_size, STATE_PAGE_SIZE);

// ============================================================================
// Paging data structures
// Ported from TS lib/paging.ts PagingProperty / PagingOperation
// ============================================================================

/// A paging property reference with path information.
/// Ported from TS PagingProperty interface.
#[derive(Debug, Clone)]
pub struct PagingProperty {
    /// The model property that holds the paging value
    pub property: TypeId,
    /// Path to the property in the model (for nested properties)
    pub path: Vec<TypeId>,
}

/// Input paging properties for a paging operation.
/// Ported from TS PagingOperation.input.
#[derive(Debug, Clone, Default)]
pub struct PagingInput {
    pub offset: Option<PagingProperty>,
    pub page_index: Option<PagingProperty>,
    pub page_size: Option<PagingProperty>,
    pub continuation_token: Option<PagingProperty>,
}

/// Output paging properties for a paging operation.
/// Ported from TS PagingOperation.output.
#[derive(Debug, Clone, Default)]
pub struct PagingOutput {
    pub page_items: Option<PagingProperty>,
    pub next_link: Option<PagingProperty>,
    pub prev_link: Option<PagingProperty>,
    pub first_link: Option<PagingProperty>,
    pub last_link: Option<PagingProperty>,
    pub continuation_token: Option<PagingProperty>,
}

/// Complete paging operation information.
/// Ported from TS PagingOperation interface.
#[derive(Debug, Clone)]
pub struct PagingOperation {
    pub input: PagingInput,
    pub output: PagingOutput,
}

/// Check if a property has any paging marker.
/// Returns the kind of paging property, or None if not a paging property.
pub fn get_paging_property_kind(state: &StateAccessors, target: TypeId) -> Option<&'static str> {
    if is_offset(state, target) {
        return Some("offset");
    }
    if is_page_index(state, target) {
        return Some("pageIndex");
    }
    if is_page_size(state, target) {
        return Some("pageSize");
    }
    if is_page_items(state, target) {
        return Some("pageItems");
    }
    if is_continuation_token(state, target) {
        return Some("continuationToken");
    }
    if is_next_link(state, target) {
        return Some("nextLink");
    }
    if is_prev_link(state, target) {
        return Some("prevLink");
    }
    if is_first_link(state, target) {
        return Some("firstLink");
    }
    if is_last_link(state, target) {
        return Some("lastLink");
    }
    None
}

/// Check if a property has multiple incompatible paging markers.
pub fn has_multiple_paging_markers(state: &StateAccessors, target: TypeId) -> bool {
    let count = [
        is_offset(state, target),
        is_page_index(state, target),
        is_page_size(state, target),
        is_page_items(state, target),
        is_continuation_token(state, target),
        is_next_link(state, target),
        is_prev_link(state, target),
        is_first_link(state, target),
        is_last_link(state, target),
    ]
    .iter()
    .filter(|&&b| b)
    .count();
    count > 1
}
