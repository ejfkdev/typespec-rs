//! Internal visibility and access control
//!
//! Ported from TypeSpec compiler visibility checking methods

use super::*;

impl Checker {
    /// Check if a type is marked as internal
    pub(crate) fn is_internal_type(&self, type_id: TypeId) -> bool {
        self.internal_declarations.contains(&type_id)
    }

    /// Check if access to an internal type is allowed based on LocationContext.
    /// Ported from TS checker.ts checkSymbolAccess.
    ///
    /// Access rules (aligned with TS upstream):
    /// - Synthetic/Compiler source: always allowed
    /// - Same project: allowed (both source and target are in the user's project)
    /// - Same library: allowed (both in the same imported library)
    /// - Cross-scope: denied, report `invalid-ref` with `internal` message
    pub(crate) fn check_internal_visibility(&mut self, type_id: TypeId) {
        if !self.is_internal_type(type_id) {
            return;
        }
        let name = self
            .get_type(type_id)
            .and_then(|t| t.name().map(|s| s.to_string()))
            .unwrap_or_else(|| "unknown".to_string());
        self.check_internal_visibility_for(type_id, &name);
    }

    /// Check internal visibility with a pre-known name (avoids double lookup)
    pub(crate) fn check_internal_visibility_for(&mut self, type_id: TypeId, name: &str) {
        use crate::helpers::location_context::is_access_allowed;

        if !self.is_internal_type(type_id) {
            return;
        }

        // Determine source location context (where the reference is made)
        let source_context = self.get_current_location_context();

        // Determine target location context (where the internal symbol is declared)
        let target_context = self.get_stdlib_location_context(type_id);

        if is_access_allowed(&source_context, &target_context) {
            return;
        }

        self.error("invalid-ref", &format!("Symbol '{}' is internal and can only be accessed from within its declaring package.", name));
    }

    /// Determine the LocationContext of the current checking context.
    /// If we're inside the TypeSpec namespace tree, we're in Compiler context.
    /// Otherwise, we're in Project context.
    pub(crate) fn get_current_location_context(
        &self,
    ) -> crate::helpers::location_context::LocationContext {
        use crate::helpers::location_context::LocationContext;
        match self.current_namespace {
            Some(ns_id) if self.is_typespec_namespace(ns_id) => LocationContext::Compiler,
            _ => LocationContext::Project,
        }
    }

    /// Quick check: is the current context Compiler (TypeSpec namespace)?
    pub(crate) fn is_current_context_compiler(&self) -> bool {
        self.current_namespace
            .is_some_and(|ns_id| self.is_typespec_namespace(ns_id))
    }

    /// Resolve a decorator by its name, supporting dotted names like "TypeSpec.indexer".
    /// First tries a direct lookup in declared_types, then walks the namespace chain.
    pub(crate) fn resolve_decorator_by_name(&self, name: &str) -> Option<TypeId> {
        // Try direct lookup first (simple names like "doc")
        if let Some(&id) = self.declared_types.get(name) {
            return Some(id);
        }

        // Handle dotted names: "TypeSpec.indexer" or "TypeSpec.Prototypes.getter"
        let parts: Vec<&str> = name.split('.').collect();
        if parts.len() < 2 {
            return None;
        }

        // Walk the namespace chain to find the decorator
        let mut current_ns_id: Option<TypeId> = None;
        for (i, part) in parts.iter().enumerate() {
            if i == parts.len() - 1 {
                // Last part: look in the current namespace's decorator_declarations
                if let Some(ns_id) = current_ns_id
                    && let Some(Type::Namespace(ns)) = self.get_type(ns_id)
                    && let Some(&dec_id) = ns.decorator_declarations.get(*part)
                {
                    return Some(dec_id);
                }
            } else {
                // Navigate into namespace
                if i == 0 {
                    current_ns_id = self.declared_types.get(*part).copied();
                } else if let Some(ns_id) = current_ns_id {
                    if let Some(Type::Namespace(ns)) = self.get_type(ns_id) {
                        current_ns_id = ns.namespaces.get(*part).copied();
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            }
        }

        None
    }

    /// Determine the LocationContext for a type based on whether it's from the stdlib.
    /// Types in the "TypeSpec" namespace tree are Compiler context.
    /// Types in imported libraries would be Library context (not yet implemented).
    /// All other types are Project context.
    pub(crate) fn get_stdlib_location_context(
        &self,
        type_id: TypeId,
    ) -> crate::helpers::location_context::LocationContext {
        use crate::helpers::location_context::LocationContext;

        // Walk up the namespace chain to check if any ancestor is "TypeSpec"
        let ns = self.get_type(type_id).and_then(|t| t.namespace());
        if let Some(ns_id) = ns
            && self.is_typespec_namespace(ns_id)
        {
            return LocationContext::Compiler;
        }
        LocationContext::Project
    }

    /// Check if a namespace (or any of its ancestors) is the "TypeSpec" stdlib namespace
    pub(crate) fn is_typespec_namespace(&self, ns_id: TypeId) -> bool {
        match self.get_type(ns_id) {
            Some(Type::Namespace(ns)) => {
                if ns.name == "TypeSpec" {
                    return true;
                }
                if let Some(parent) = ns.namespace {
                    self.is_typespec_namespace(parent)
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}
