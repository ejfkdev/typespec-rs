//! HTTP response resolution
//!
//! Ported from TypeSpec packages/http/src/responses.ts
//!
//! Provides:
//! - `is_plain_response_body` — checks if a type is a plain response body (no HTTP envelope)
//! - `resolve_response_variants` — recursively flattens union variants into plain/envelope groups
//! - `ResponseIndex` — deduplicates responses by status code

use crate::checker::types::{IntrinsicTypeName, Type, TypeId};
use crate::checker::Checker;
use crate::libs::http::operation::HttpOperationResponse;
use crate::libs::status_codes::StatusCodeEntry as HttpStatusCodesEntry;

// ============================================================================
// State key constants for HTTP response metadata
// ============================================================================

const STATE_STATUS_CODE: &str = "TypeSpec.Http.statusCode";
const STATE_BODY: &str = "TypeSpec.Http.body";
const STATE_BODY_ROOT: &str = "TypeSpec.Http.bodyRoot";
const STATE_HEADER: &str = "TypeSpec.Http.header";
const STATE_MULTIPART_BODY: &str = "TypeSpec.Http.multipartBody";

// ============================================================================
// AllowedSegmentSeparators — route path separators
// ============================================================================

/// Characters allowed as segment separators in URI path templates.
/// Ported from TS `AllowedSegmentSeparators = ["/", ":", "?"]`.
pub const ALLOWED_SEGMENT_SEPARATORS: &[char] = &['/', ':', '?'];

/// Check if a character is an allowed URI path segment separator.
pub fn is_allowed_segment_separator(c: char) -> bool {
    ALLOWED_SEGMENT_SEPARATORS.contains(&c)
}

// ============================================================================
// is_plain_response_body
// ============================================================================

/// Check if a type is a "plain" response body — one with no HTTP envelope metadata.
/// Ported from TS `isPlainResponseBody()`.
///
/// Returns `false` for:
/// - `void` types
/// - Models with `@statusCode` property
/// - Models with `@header` property
/// - Models with `@body` or `@bodyRoot` property
/// - Models with `@multipartBody` property
///
/// Returns `true` for simple data models with no HTTP-specific decorators.
pub fn is_plain_response_body(checker: &Checker, type_id: TypeId) -> bool {
    let resolved = checker.resolve_alias_chain(type_id);

    match checker.get_type(resolved) {
        Some(Type::Intrinsic(i)) => {
            !matches!(i.name, IntrinsicTypeName::Void | IntrinsicTypeName::ErrorType)
        }
        Some(Type::Model(m)) => {
            for name in &m.property_names {
                if let Some(&prop_id) = m.properties.get(name)
                    && has_http_metadata_decorators(checker, prop_id)
                {
                    return false;
                }
            }
            true
        }
        Some(Type::Union(_)) => true,
        Some(Type::Enum(_))
        | Some(Type::Scalar(_))
        | Some(Type::String(_))
        | Some(Type::Number(_))
        | Some(Type::Boolean(_))
        | Some(Type::Tuple(_)) => true,
        _ => false,
    }
}

/// Check if a model property has HTTP envelope metadata decorators.
fn has_http_metadata_decorators(checker: &Checker, prop_id: TypeId) -> bool {
    let state = &checker.state_accessors;
    state.get_state(STATE_STATUS_CODE, prop_id).is_some()
        || state.get_state(STATE_HEADER, prop_id).is_some()
        || state.get_state(STATE_BODY, prop_id).is_some()
        || state.get_state(STATE_BODY_ROOT, prop_id).is_some()
        || state.get_state(STATE_MULTIPART_BODY, prop_id).is_some()
}

// ============================================================================
// ResolvedResponseVariant
// ============================================================================

/// A resolved response variant after union flattening.
/// Ported from TS `ResolvedResponseVariant`.
#[derive(Debug, Clone)]
pub enum ResolvedResponseVariant {
    /// A plain body response (no HTTP envelope metadata)
    Plain { type_id: TypeId },
    /// A response envelope (has HTTP metadata like @statusCode, @header, etc.)
    Envelope { type_id: TypeId },
}

// ============================================================================
// resolveResponseVariants
// ============================================================================

/// Recursively flatten union variants and classify them as plain body or envelope.
/// Ported from TS `resolveResponseVariants()`.
///
/// This function handles union return types like:
/// ```typespec
/// op read(): Pet | { @statusCode code: 201; @body pet: Pet };
/// ```
///
/// The `Pet` is a plain body variant, while the `{ @statusCode ... }` is an envelope.
pub fn resolve_response_variants(
    checker: &Checker,
    type_id: TypeId,
) -> Vec<ResolvedResponseVariant> {
    let resolved = checker.resolve_alias_chain(type_id);
    resolve_response_variants_inner(checker, resolved, 0)
}

const MAX_RECURSION_DEPTH: usize = 10;

fn resolve_response_variants_inner(
    checker: &Checker,
    type_id: TypeId,
    depth: usize,
) -> Vec<ResolvedResponseVariant> {
    if depth > MAX_RECURSION_DEPTH {
        return vec![ResolvedResponseVariant::Plain { type_id }];
    }

    match checker.get_type(type_id) {
        Some(Type::Union(u)) => {
            let mut plain_variants = Vec::new();
            let mut envelope_variants = Vec::new();

            for name in &u.variant_names {
                if let Some(&variant_id) = u.variants.get(name) {
                    let inner_type_id = match checker.get_type(variant_id) {
                        Some(Type::UnionVariant(v)) => v.r#type,
                        _ => continue,
                    };

                    let inner_resolved = checker.resolve_alias_chain(inner_type_id);

                    // Skip null variants
                    if let Some(Type::Intrinsic(i)) = checker.get_type(inner_resolved)
                        && matches!(i.name, IntrinsicTypeName::Null)
                    {
                        continue;
                    }

                    // Recursively resolve inner variants
                    let inner_variants =
                        resolve_response_variants_inner(checker, inner_resolved, depth + 1);

                    for variant in inner_variants {
                        match variant {
                            ResolvedResponseVariant::Plain { type_id } => {
                                plain_variants.push(type_id);
                            }
                            ResolvedResponseVariant::Envelope { type_id } => {
                                envelope_variants.push(type_id);
                            }
                        }
                    }
                }
            }

            let mut result = Vec::new();

            // Merge plain variants into a single group
            if plain_variants.len() == 1 {
                result.push(ResolvedResponseVariant::Plain {
                    type_id: plain_variants[0],
                });
            } else if !plain_variants.is_empty() {
                // Multiple plain variants: reference the original union as the merged plain body
                result.push(ResolvedResponseVariant::Plain { type_id });
            }

            // Add envelope variants individually
            for env_id in envelope_variants {
                result.push(ResolvedResponseVariant::Envelope { type_id: env_id });
            }

            result
        }
        Some(Type::Intrinsic(i)) if matches!(i.name, IntrinsicTypeName::Null) => {
            // Skip null variants
            vec![]
        }
        _ => classify_variant(checker, type_id),
    }
}

/// Classify a single type as plain or envelope.
fn classify_variant(
    checker: &Checker,
    type_id: TypeId,
) -> Vec<ResolvedResponseVariant> {
    if is_plain_response_body(checker, type_id) {
        vec![ResolvedResponseVariant::Plain { type_id }]
    } else {
        vec![ResolvedResponseVariant::Envelope { type_id }]
    }
}

// ============================================================================
// ResponseIndex
// ============================================================================

/// Index for deduplicating responses by status code.
/// Ported from TS `class ResponseIndex`.
#[derive(Debug, Clone, Default)]
pub struct ResponseIndex {
    responses: std::collections::HashMap<String, HttpOperationResponse>,
}

impl ResponseIndex {
    /// Create a new empty response index.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get a response by status code.
    pub fn get(&self, status_code: &HttpStatusCodesEntry) -> Option<&HttpOperationResponse> {
        let key = self.index_key(status_code);
        self.responses.get(&key)
    }

    /// Insert a response, deduplicating by status code.
    pub fn set(&mut self, status_code: HttpStatusCodesEntry, response: HttpOperationResponse) {
        let key = self.index_key(&status_code);
        self.responses.insert(key, response);
    }

    /// Get all responses sorted by status code.
    pub fn values(&self) -> Vec<&HttpOperationResponse> {
        let mut vals: Vec<_> = self.responses.values().collect();
        vals.sort_by_key(|r| self.sort_key(&r.status_codes));
        vals
    }

    /// Compute the index key for a status code entry.
    fn index_key(&self, status_code: &HttpStatusCodesEntry) -> String {
        match status_code {
            HttpStatusCodesEntry::Code(code) => code.to_string(),
            HttpStatusCodesEntry::Range(range) => format!("{}-{}", range.start, range.end),
            HttpStatusCodesEntry::Wildcard => "*".to_string(),
        }
    }

    /// Compute a sort key for ordering responses.
    fn sort_key(&self, status_code: &HttpStatusCodesEntry) -> String {
        self.index_key(status_code)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::Checker;
    use crate::parser;

    fn check(source: &str) -> Checker {
        let parse_result = parser::parse(source);
        let mut checker = Checker::new();
        checker.set_parse_result(parse_result.root_id, parse_result.builder.clone());
        checker.check_program();
        checker
    }

    #[test]
    fn test_is_plain_response_body_simple_model() {
        let checker = check("model Pet { name: string; }");
        let pet_id = checker.declared_types["Pet"];
        assert!(
            is_plain_response_body(&checker, pet_id),
            "Simple model should be plain body"
        );
    }

    #[test]
    fn test_is_plain_response_body_void() {
        let checker = check("model M { x: int32; }");
        let void_type = checker.void_type;
        assert!(
            !is_plain_response_body(&checker, void_type),
            "void should NOT be plain body"
        );
    }

    #[test]
    fn test_is_plain_response_body_enum() {
        let checker = check("enum Color { red, green, blue }");
        let color_id = checker.declared_types["Color"];
        assert!(
            is_plain_response_body(&checker, color_id),
            "Enum should be plain body"
        );
    }

    #[test]
    fn test_allowed_segment_separators() {
        assert!(is_allowed_segment_separator('/'));
        assert!(is_allowed_segment_separator(':'));
        assert!(is_allowed_segment_separator('?'));
        assert!(!is_allowed_segment_separator('-'));
    }

    #[test]
    fn test_response_index_basic() {
        let mut index = ResponseIndex::new();
        let response = HttpOperationResponse {
            status_codes: HttpStatusCodesEntry::Code(200),
            response_type: 0,
            description: Some("OK".to_string()),
            responses: vec![],
        };
        index.set(HttpStatusCodesEntry::Code(200), response);
        assert!(index.get(&HttpStatusCodesEntry::Code(200)).is_some());
        assert!(index.get(&HttpStatusCodesEntry::Code(404)).is_none());
    }

    #[test]
    fn test_response_index_dedup() {
        let mut index = ResponseIndex::new();
        let r1 = HttpOperationResponse {
            status_codes: HttpStatusCodesEntry::Code(200),
            response_type: 0,
            description: Some("First".to_string()),
            responses: vec![],
        };
        let r2 = HttpOperationResponse {
            status_codes: HttpStatusCodesEntry::Code(200),
            response_type: 1,
            description: Some("Second".to_string()),
            responses: vec![],
        };
        index.set(HttpStatusCodesEntry::Code(200), r1);
        index.set(HttpStatusCodesEntry::Code(200), r2);
        assert_eq!(index.values().len(), 1);
        assert_eq!(index.values()[0].description, Some("Second".to_string()));
    }

    #[test]
    fn test_resolve_response_variants_simple_type() {
        let checker = check("model Pet { name: string; }");
        let pet_id = checker.declared_types["Pet"];
        let variants = resolve_response_variants(&checker, pet_id);
        assert_eq!(variants.len(), 1);
        assert!(matches!(variants[0], ResolvedResponseVariant::Plain { .. }));
    }

    #[test]
    fn test_resolve_response_variants_union() {
        let checker = check("union Shape { circle: string, square: int32 }");
        let shape_id = checker.declared_types["Shape"];
        let variants = resolve_response_variants(&checker, shape_id);
        // A non-discriminated union with all plain variants
        // should produce a single plain variant (the union itself)
        assert!(
            !variants.is_empty(),
            "Should produce at least one variant"
        );
        assert!(
            matches!(variants[0], ResolvedResponseVariant::Plain { .. }),
            "Union of plain types should be Plain"
        );
    }
}
