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
    ///
    /// Lookup order:
    /// 1. Direct lookup in declared_types (top-level names)
    /// 2. Current namespace's decorator_declarations
    /// 3. Using-imported namespaces (mirrors check_identifier_inner resolve_via_using)
    /// 4. Recursive search of all sub-namespaces under the global namespace
    /// 5. Dotted name walk: "TypeSpec.indexer", "TypeSpec.Prototypes.getter"
    pub(crate) fn resolve_decorator_by_name(&self, name: &str) -> Option<TypeId> {
        // 1. Try direct lookup first (FQN-aware, simple names like "doc")
        if let Some(id) = self.resolve_declared_name(name) {
            return Some(id);
        }

        // 2. Check current namespace's decorator_declarations
        if let Some(ns_id) = self.current_namespace
            && let Some(Type::Namespace(ns)) = self.get_type(ns_id)
            && let Some(&dec_id) = ns.decorator_declarations.get(name)
        {
            return Some(dec_id);
        }

        // 3. Try using-imported namespaces
        if let Some(type_id) = self.resolve_decorator_via_using(name) {
            return Some(type_id);
        }

        // 3b. Recursive search of all sub-namespaces for the decorator.
        // This handles the case where a decorator is registered in a sub-namespace
        // (e.g., "Llm" or "AnyUse.CLI") but used without an explicit `using` declaration.
        // Mirrors how TypeSpec's name resolver implicitly finds symbols in parent scopes.
        if let Some(global_ns_id) = self.global_namespace_type
            && let Some(found) = self.find_decorator_in_namespace_tree(global_ns_id, name)
        {
            return Some(found);
        }

        // 4. Handle dotted names: "TypeSpec.indexer" or "TypeSpec.Prototypes.getter"
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
                    current_ns_id = self.resolve_declared_name(part);
                } else {
                    let ns_id = current_ns_id?;
                    let Some(Type::Namespace(ns)) = self.get_type(ns_id) else {
                        return None;
                    };
                    current_ns_id = ns.namespaces.get(*part).copied();
                }
            }
        }

        None
    }

    /// Resolve a decorator name via using declarations.
    /// Looks in each using'd namespace's decorator_declarations.
    fn resolve_decorator_via_using(&self, name: &str) -> Option<TypeId> {
        for (_, using_ns_name, resolved_ns) in &self.using_declarations {
            let ns_opt = resolved_ns.or_else(|| self.resolve_namespace_by_name(using_ns_name));
            if let Some(ns_id) = ns_opt
                && let Some(Type::Namespace(ns)) = self.get_type(ns_id)
                && let Some(&dec_id) = ns.decorator_declarations.get(name)
            {
                return Some(dec_id);
            }
        }
        None
    }

    /// Recursively search a namespace and all its sub-namespaces for a decorator by name.
    /// Used as a fallback when direct lookup, current namespace, and using imports fail.
    /// Uses a depth limit to guard against unexpected cycles in the namespace tree.
    fn find_decorator_in_namespace_tree(&self, ns_id: TypeId, name: &str) -> Option<TypeId> {
        self.find_decorator_in_namespace_tree_inner(ns_id, name, 0)
    }

    fn find_decorator_in_namespace_tree_inner(
        &self,
        ns_id: TypeId,
        name: &str,
        depth: u32,
    ) -> Option<TypeId> {
        if depth > 50 {
            return None;
        }
        if let Some(Type::Namespace(ns)) = self.get_type(ns_id) {
            // Check this namespace's decorator_declarations
            if let Some(&dec_id) = ns.decorator_declarations.get(name) {
                return Some(dec_id);
            }
            // Recurse into child namespaces
            for &child_ns_id in ns.namespaces.values() {
                if let Some(found) =
                    self.find_decorator_in_namespace_tree_inner(child_ns_id, name, depth + 1)
                {
                    return Some(found);
                }
            }
        }
        None
    }

    /// Resolve a (possibly dotted) namespace name to its TypeId.
    /// For "AnyUse.CLI", walks: declared_types["AnyUse"] → namespaces["CLI"].
    /// For simple names like "HTTP", looks up declared_types directly.
    pub(crate) fn resolve_namespace_by_name(&self, name: &str) -> Option<TypeId> {
        // Try direct lookup first (FQN-aware)
        if let Some(id) = self.resolve_declared_name(name) {
            return Some(id);
        }

        // Handle dotted names by walking the namespace hierarchy
        let parts: Vec<&str> = name.split('.').collect();
        if parts.len() < 2 {
            return None;
        }

        let mut current_id = self.resolve_declared_name(parts[0])?;
        for part in &parts[1..] {
            match self.get_type(current_id) {
                Some(Type::Namespace(ns)) => {
                    current_id = ns.namespaces.get(*part).copied()?;
                }
                _ => return None,
            }
        }
        Some(current_id)
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
