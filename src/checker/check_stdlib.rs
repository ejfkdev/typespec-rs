//! Standard library initialization
//!
//! Ported from TypeSpec compiler stdlib initialization methods

use super::*;

impl Checker {
    /// Initialize standard TypeSpec types (string, int32, etc.)
    pub(crate) fn initialize_std_types(&mut self) {
        // Create TypeSpec namespace first (so scalars can reference it)
        let typespec_ns_id = self.create_type(Type::Namespace(Box::new(NamespaceType::new(
            self.next_type_id(),
            "TypeSpec".to_string(),
            None,
            None, // parent will be set in initialize_global_namespace
            true,
        ))));
        self.typespec_namespace_id = Some(typespec_ns_id);
        self.declared_types
            .insert("TypeSpec".to_string(), typespec_ns_id);
        self.std_types
            .insert("TypeSpec".to_string(), typespec_ns_id);

        // Create TypeSpec.Prototypes namespace
        let prototypes_ns_id = self.create_type(Type::Namespace(Box::new(NamespaceType::new(
            self.next_type_id(),
            "Prototypes".to_string(),
            None,
            Some(typespec_ns_id),
            true,
        ))));
        self.declared_types
            .insert("TypeSpec.Prototypes".to_string(), prototypes_ns_id);
        // Register Prototypes as sub-namespace of TypeSpec
        if let Some(t) = self.get_type_mut(typespec_ns_id)
            && let Type::Namespace(ns) = t
        {
            ns.namespaces
                .insert("Prototypes".to_string(), prototypes_ns_id);
            ns.namespace_names.push("Prototypes".to_string());
        }

        // Create TypeSpec.Reflection namespace
        let reflection_ns_id = self.create_type(Type::Namespace(Box::new(NamespaceType::new(
            self.next_type_id(),
            "Reflection".to_string(),
            None,
            Some(typespec_ns_id),
            true,
        ))));
        self.declared_types
            .insert("TypeSpec.Reflection".to_string(), reflection_ns_id);
        // Register Reflection as sub-namespace of TypeSpec
        if let Some(t) = self.get_type_mut(typespec_ns_id)
            && let Type::Namespace(ns) = t
        {
            ns.namespaces
                .insert("Reflection".to_string(), reflection_ns_id);
            ns.namespace_names.push("Reflection".to_string());
        }

        // Define base types first (order matters for base_scalar references)
        let base_types = [
            "string",
            "numeric",
            "integer",
            "float",
            "decimal",
            "boolean",
            "Unit",
            "duration",
            "plainDate",
            "plainTime",
        ];
        for name in &base_types {
            if !self.std_types.contains_key(*name) {
                let type_id = {
                    let mut s = ScalarType::new(
                        self.next_type_id(),
                        name.to_string(),
                        None,
                        Some(typespec_ns_id),
                        None,
                    );
                    s.is_finished = true;
                    self.create_type(Type::Scalar(s))
                };
                self.std_types.insert(name.to_string(), type_id);
                // Register in TypeSpec namespace
                if let Some(t) = self.get_type_mut(typespec_ns_id)
                    && let Type::Namespace(ns) = t
                {
                    ns.scalars.insert(name.to_string(), type_id);
                    ns.scalar_names.push(name.to_string());
                }
            }
        }

        // Define derived types with their base_scalar relationships
        // TS intrinsics.tsp: int8 extends int16 extends int32 extends int64 extends integer extends numeric
        // uint8 extends uint16 extends uint32 extends uint64 extends integer extends numeric
        // safeint extends int64, float32 extends float, float64 extends float, decimal128 extends decimal
        let derived_types: &[(&str, &str)] = &[
            ("int64", "integer"),
            ("int32", "int64"),
            ("int16", "int32"),
            ("int8", "int16"),
            ("uint64", "integer"),
            ("uint32", "uint64"),
            ("uint16", "uint32"),
            ("uint8", "uint16"),
            ("safeint", "int64"),
            ("float64", "float"),
            ("float32", "float64"),
            ("decimal128", "decimal"),
            ("url", "string"),
            ("bytes", "string"),
            ("utcDateTime", "string"),
            ("offsetDateTime", "string"),
            ("unixTimestamp32", "utcDateTime"),
        ];

        for &(name, base_name) in derived_types {
            if !self.std_types.contains_key(name) {
                let base_scalar = self.std_types.get(base_name).copied();
                let type_id = {
                    let mut s = ScalarType::new(
                        self.next_type_id(),
                        name.to_string(),
                        None,
                        Some(typespec_ns_id),
                        base_scalar,
                    );
                    s.is_finished = true;
                    self.create_type(Type::Scalar(s))
                };
                self.std_types.insert(name.to_string(), type_id);
                // Register in TypeSpec namespace
                if let Some(t) = self.get_type_mut(typespec_ns_id)
                    && let Type::Namespace(ns) = t
                {
                    ns.scalars.insert(name.to_string(), type_id);
                    ns.scalar_names.push(name.to_string());
                }

                // Register as derived scalar on the base
                if let Some(base_id) = base_scalar
                    && let Some(t) = self.get_type_mut(base_id)
                    && let Type::Scalar(s) = t
                {
                    s.derived_scalars.push(type_id);
                }
            }
        }

        // Set up the base_scalar chain for the base numeric types themselves
        // integer extends numeric, float extends numeric, decimal extends numeric
        let base_chains: &[(&str, &str)] = &[
            ("integer", "numeric"),
            ("float", "numeric"),
            ("decimal", "numeric"),
        ];
        for &(name, base_name) in base_chains {
            if let (Some(&type_id), Some(&base_id)) =
                (self.std_types.get(name), self.std_types.get(base_name))
            {
                if let Some(t) = self.get_type_mut(type_id)
                    && let Type::Scalar(s) = t
                    && s.base_scalar.is_none()
                {
                    s.base_scalar = Some(base_id);
                }
                if let Some(t) = self.get_type_mut(base_id)
                    && let Type::Scalar(s) = t
                {
                    s.derived_scalars.push(type_id);
                }
            }
        }

        // Register built-in model types
        // Array is a built-in template model with an integer indexer
        if !self.declared_types.contains_key("Array") {
            let integer_id = self.std_types.get("integer").copied();
            let array_id = {
                let mut m = ModelType::new(
                    self.next_type_id(),
                    "Array".to_string(),
                    None,
                    Some(typespec_ns_id),
                );
                m.indexer = integer_id.map(|id| (id, self.error_type));
                m.template_node = Some(0); // Mark as template
                m.is_finished = true;
                self.create_type(Type::Model(m))
            };
            self.declared_types.insert("Array".to_string(), array_id);
            // Register in TypeSpec namespace
            if let Some(t) = self.get_type_mut(typespec_ns_id)
                && let Type::Namespace(ns) = t
            {
                ns.models.insert("Array".to_string(), array_id);
                ns.model_names.push("Array".to_string());
            }
        }

        // Record is a built-in template model with a string indexer
        if !self.declared_types.contains_key("Record") {
            let string_id = self.std_types.get("string").copied();
            let record_id = {
                let mut m = ModelType::new(
                    self.next_type_id(),
                    "Record".to_string(),
                    None,
                    Some(typespec_ns_id),
                );
                m.indexer = string_id.map(|id| (id, self.error_type));
                m.template_node = Some(0); // Mark as template
                m.is_finished = true;
                self.create_type(Type::Model(m))
            };
            self.declared_types.insert("Record".to_string(), record_id);
            // Register in TypeSpec namespace
            if let Some(t) = self.get_type_mut(typespec_ns_id)
                && let Type::Namespace(ns) = t
            {
                ns.models.insert("Record".to_string(), record_id);
                ns.model_names.push("Record".to_string());
            }
        }
    }

    /// Initialize standard TypeSpec decorator declarations
    /// Ported from TS lib/std/decorators.tsp and lib/intrinsic/tsp-index.ts
    pub(crate) fn initialize_std_decorators(&mut self) {
        use crate::std::decorator_registry::STD_DECORATORS;

        for def in STD_DECORATORS {
            // Find or create the namespace
            let ns_id = self.declared_types.get(def.namespace).copied();
            let ns_id = match ns_id {
                Some(id) => id,
                None => continue, // namespace not found, skip
            };

            // Create the decorator type
            let type_id = self.create_type(Type::Decorator(DecoratorType {
                id: self.next_type_id(),
                name: def.name.to_string(),
                node: None,
                namespace: Some(ns_id),
                target: None,
                target_type: "unknown".to_string(),
                parameters: Vec::new(),
                is_finished: true,
            }));

            // Register in namespace's decorator_declarations
            if let Some(t) = self.get_type_mut(ns_id)
                && let Type::Namespace(ns) = t
            {
                ns.decorator_declarations
                    .insert(def.name.to_string(), type_id);
                ns.decorator_declaration_names.push(def.name.to_string());
            }

            // Mark internal decorators
            if def.is_internal {
                self.internal_declarations.insert(type_id);
            }
        }
    }

    /// Initialize custom decorators registered via `register_decorator()`.
    ///
    /// This runs after `initialize_std_types()` so the global namespace exists.
    /// If a decorator's namespace doesn't exist yet, it is created as a
    /// sub-namespace of the global namespace.
    pub(crate) fn initialize_custom_decorators(&mut self) {
        let custom = std::mem::take(&mut self.custom_decorators);

        for def in custom {
            // Find or create the namespace, supporting dotted names like "AnyUse.CLI"
            let ns_id = self.ensure_decorator_namespace(&def.namespace);

            // Check if decorator already exists in this namespace
            let already_exists = if let Some(Type::Namespace(ns)) = self.get_type(ns_id) {
                ns.decorator_declarations.contains_key(&def.name)
            } else {
                false
            };

            if already_exists {
                continue;
            }

            // Create the decorator type
            // Convert DecoratorParamDef to FunctionParameterType
            let params: Vec<FunctionParameterType> = def
                .parameters
                .iter()
                .map(|p| {
                    let param_type = if p.type_name == "unknown" || p.type_name.is_empty() {
                        None
                    } else {
                        self.resolve_type_by_name(&p.type_name)
                    };
                    FunctionParameterType {
                        id: self.next_type_id(),
                        name: p.name.clone(),
                        node: None,
                        r#type: param_type,
                        optional: p.optional,
                        rest: p.rest,
                        is_finished: true,
                    }
                })
                .collect();

            let type_id = self.create_type(Type::Decorator(DecoratorType {
                id: self.next_type_id(),
                name: def.name.clone(),
                node: None,
                namespace: Some(ns_id),
                target: None,
                target_type: def.target_type.clone(),
                parameters: params,
                is_finished: true,
            }));

            // Register in namespace's decorator_declarations
            if let Some(t) = self.get_type_mut(ns_id)
                && let Type::Namespace(ns) = t
            {
                ns.decorator_declarations.insert(def.name.clone(), type_id);
                ns.decorator_declaration_names.push(def.name);
            }
        }
    }

    /// Ensure the full namespace hierarchy for a (possibly dotted) namespace name.
    /// For "AnyUse.CLI", creates: global → AnyUse → CLI, returning the CLI TypeId.
    /// For simple names like "HTTP", creates: global → HTTP, returning the HTTP TypeId.
    fn ensure_decorator_namespace(&mut self, namespace: &str) -> TypeId {
        let parts: Vec<&str> = namespace.split('.').collect();

        if parts.len() == 1 {
            // Simple name — existing behavior
            return self.ensure_single_namespace(namespace);
        }

        // Dotted name — create hierarchy
        let mut parent_id = self.global_namespace_type;
        for part in parts {
            parent_id = Some(self.ensure_child_namespace(part, parent_id));
        }
        parent_id.expect("namespace hierarchy should produce a valid TypeId")
    }

    /// Ensure a single top-level namespace exists under the global namespace.
    fn ensure_single_namespace(&mut self, name: &str) -> TypeId {
        if let Some(&id) = self.declared_types.get(name) {
            return id;
        }

        let ns_id = self.create_type(Type::Namespace(Box::new(NamespaceType::new(
            self.next_type_id(),
            name.to_string(),
            None,
            self.global_namespace_type,
            false,
        ))));

        if let Some(global_id) = self.global_namespace_type
            && let Some(Type::Namespace(global_ns)) = self.get_type_mut(global_id)
        {
            global_ns.namespaces.insert(name.to_string(), ns_id);
            global_ns.namespace_names.push(name.to_string());
        }

        self.declared_types.insert(name.to_string(), ns_id);
        ns_id
    }

    /// Ensure a child namespace named `name` exists under `parent_id`.
    /// If it already exists as a child, return its TypeId.
    /// Otherwise, create it, register it in declared_types and in parent's namespaces map.
    fn ensure_child_namespace(&mut self, name: &str, parent_id: Option<TypeId>) -> TypeId {
        // Check if already in declared_types (top-level)
        if let Some(&id) = self.declared_types.get(name)
            && let Some(Type::Namespace(ns)) = self.get_type(id)
            && ns.namespace == parent_id
        {
            return id;
        }

        // Check if it exists as a child of parent_id
        if let Some(pid) = parent_id
            && let Some(Type::Namespace(parent_ns)) = self.get_type(pid)
            && let Some(&child_id) = parent_ns.namespaces.get(name)
        {
            return child_id;
        }

        // Create the child namespace (is_finished = false so check_namespace can process it)
        let child_id = self.create_type(Type::Namespace(Box::new(NamespaceType::new(
            self.next_type_id(),
            name.to_string(),
            None,
            parent_id,
            false,
        ))));

        // Register in parent's namespaces map
        if let Some(pid) = parent_id
            && let Some(Type::Namespace(parent_ns)) = self.get_type_mut(pid)
        {
            if !parent_ns.namespaces.contains_key(name) {
                parent_ns.namespace_names.push(name.to_string());
            }
            parent_ns.namespaces.insert(name.to_string(), child_id);
        }

        // Register in declared_types for top-level lookup
        self.declared_types.insert(name.to_string(), child_id);

        child_id
    }

    /// Initialize standard enums and models from lib/std/
    /// Ported from TS lib/std/visibility.tsp and lib/std/decorators.tsp
    pub(crate) fn initialize_std_enums_and_models(&mut self) {
        let typespec_ns_id = match self.typespec_namespace_id {
            Some(id) => id,
            None => return,
        };

        // ===== Enums from lib/std/visibility.tsp =====

        // Lifecycle enum: Create, Read, Update, Delete, Query
        let lifecycle_id = self.register_std_enum(
            "Lifecycle",
            typespec_ns_id,
            &["Create", "Read", "Update", "Delete", "Query"],
        );
        if let Some(lifecycle_id) = lifecycle_id {
            self.std_types.insert("Lifecycle".to_string(), lifecycle_id);
        }

        // ===== Enums from lib/std/decorators.tsp =====

        self.register_std_enum(
            "DateTimeKnownEncoding",
            typespec_ns_id,
            &["rfc3339", "rfc7231", "unixTimestamp"],
        );

        self.register_std_enum(
            "DurationKnownEncoding",
            typespec_ns_id,
            &["ISO8601", "seconds", "milliseconds"],
        );

        self.register_std_enum(
            "BytesKnownEncoding",
            typespec_ns_id,
            &["base64", "base64url"],
        );

        self.register_std_enum(
            "ArrayEncoding",
            typespec_ns_id,
            &[
                "pipeDelimited",
                "spaceDelimited",
                "commaDelimited",
                "newlineDelimited",
            ],
        );

        // ===== Models from lib/std/visibility.tsp =====

        // VisibilityFilter model
        self.register_std_model("VisibilityFilter", typespec_ns_id);

        // ===== Models from lib/std/decorators.tsp =====

        self.register_std_model("ServiceOptions", typespec_ns_id);
        self.register_std_model("DiscriminatedOptions", typespec_ns_id);
        self.register_std_model("ExampleOptions", typespec_ns_id);
        self.register_std_model("OperationExample", typespec_ns_id);

        // ===== Reflection models from lib/std/reflection.tsp =====
        if let Some(reflection_ns_id) = self.declared_types.get("TypeSpec.Reflection").copied() {
            for name in &[
                "Enum",
                "EnumMember",
                "Interface",
                "Model",
                "ModelProperty",
                "Namespace",
                "Operation",
                "Scalar",
                "Union",
                "UnionVariant",
                "StringTemplate",
            ] {
                self.register_std_model(name, reflection_ns_id);
            }
        }

        // ===== Internal functions from lib/std/visibility.tsp =====
        // applyVisibilityFilter and applyLifecycleUpdate are internal extern fn
        // These are registered as function types for reference resolution
        for name in &["applyVisibilityFilter", "applyLifecycleUpdate"] {
            let type_id = self.create_type(Type::FunctionType(FunctionTypeType {
                id: self.next_type_id(),
                name: name.to_string(),
                node: None,
                namespace: Some(typespec_ns_id),
                parameters: Vec::new(),
                return_type: None,
                is_finished: true,
            }));
            self.internal_declarations.insert(type_id);
            self.declared_types.insert(name.to_string(), type_id);
            if let Some(t) = self.get_type_mut(typespec_ns_id)
                && let Type::Namespace(ns) = t
            {
                ns.function_declarations.insert(name.to_string(), type_id);
                ns.function_declaration_names.push(name.to_string());
            }
        }
    }

    /// Helper: register a standard enum type
    fn register_std_enum(
        &mut self,
        name: &str,
        namespace_id: TypeId,
        members: &[&str],
    ) -> Option<TypeId> {
        if self.declared_types.contains_key(name) {
            return self.declared_types.get(name).copied();
        }

        let mut members_map = HashMap::new();
        let mut member_names = Vec::new();
        for (i, &member_name) in members.iter().enumerate() {
            let value_type_id = self.create_type(Type::Number(NumericType {
                id: self.next_type_id(),
                value: i as f64,
                value_as_string: format!("{}", i),
                node: None,
                is_finished: true,
            }));
            let member_id = self.create_type(Type::EnumMember(EnumMemberType {
                id: self.next_type_id(),
                name: member_name.to_string(),
                node: None,
                r#enum: None, // will be set below
                value: Some(value_type_id),
                source_member: None,
                decorators: Vec::new(),
                doc: None,
                summary: None,
                is_finished: true,
            }));
            members_map.insert(member_name.to_string(), member_id);
            member_names.push(member_name.to_string());
        }

        let type_id = self.create_type(Type::Enum(EnumType {
            id: self.next_type_id(),
            name: name.to_string(),
            node: None,
            namespace: Some(namespace_id),
            members: members_map,
            member_names,
            decorators: Vec::new(),
            doc: None,
            summary: None,
            is_finished: true,
        }));

        // Set back-reference on members
        let member_ids: Vec<TypeId> = self
            .get_type(type_id)
            .and_then(|t| {
                if let Type::Enum(e) = t {
                    Some(e.members.values().copied().collect())
                } else {
                    None
                }
            })
            .unwrap_or_default();
        for member_id in member_ids {
            if let Some(Type::EnumMember(m)) = self.get_type_mut(member_id) {
                m.r#enum = Some(type_id);
            }
        }

        self.declared_types.insert(name.to_string(), type_id);
        if let Some(t) = self.get_type_mut(namespace_id)
            && let Type::Namespace(ns) = t
        {
            ns.enums.insert(name.to_string(), type_id);
            ns.enum_names.push(name.to_string());
        }

        Some(type_id)
    }

    /// Helper: register a standard model type (empty placeholder)
    fn register_std_model(&mut self, name: &str, namespace_id: TypeId) -> Option<TypeId> {
        if self.declared_types.contains_key(name) {
            return self.declared_types.get(name).copied();
        }

        let type_id = self.create_type(Type::Model(ModelType::new(
            self.next_type_id(),
            name.to_string(),
            None,
            Some(namespace_id),
        )));

        self.declared_types.insert(name.to_string(), type_id);
        if let Some(t) = self.get_type_mut(namespace_id)
            && let Type::Namespace(ns) = t
        {
            ns.models.insert(name.to_string(), type_id);
            ns.model_names.push(name.to_string());
        }

        Some(type_id)
    }

    /// Resolve a type by its name (e.g. "string", "int32", "boolean").
    /// Used during custom decorator parameter resolution to find std types.
    fn resolve_type_by_name(&self, name: &str) -> Option<TypeId> {
        self.std_types.get(name).copied()
    }

    /// Initialize the global namespace
    pub(crate) fn initialize_global_namespace(&mut self) {
        let ns_id = self.create_type(Type::Namespace(Box::new(NamespaceType::new(
            self.next_type_id(),
            String::new(),
            None,
            None,
            false,
        ))));
        self.global_namespace_type = Some(ns_id);
        self.current_namespace = Some(ns_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;

    fn check(source: &str) -> Checker {
        let parse_result = parser::parse(source);
        let mut checker = Checker::new();
        checker.set_parse_result(parse_result.root_id, parse_result.builder.clone());
        checker.check_program();
        checker
    }

    #[test]
    fn test_register_decorator_basic() {
        let parse_result = parser::parse("model Pet { name: string; }");
        let mut checker = Checker::new();
        checker.set_parse_result(parse_result.root_id, parse_result.builder.clone());
        checker.register_decorator("command", "CLI", "unknown");
        checker.check_program();

        // Verify "CLI" namespace was created
        assert!(checker.declared_types.contains_key("CLI"));

        // Verify "command" decorator exists in CLI namespace
        let cli_id = checker.declared_types["CLI"];
        if let Some(Type::Namespace(ns)) = checker.get_type(cli_id) {
            assert!(ns.decorator_declarations.contains_key("command"));
        } else {
            panic!("CLI should be a namespace");
        }
    }

    #[test]
    fn test_register_multiple_decorators() {
        let parse_result = parser::parse("model Pet { name: string; }");
        let mut checker = Checker::new();
        checker.set_parse_result(parse_result.root_id, parse_result.builder.clone());
        checker.register_decorators(vec![
            ("command", "CLI", "unknown"),
            ("flag", "CLI", "unknown"),
            ("arg", "CLI", "Model"),
        ]);
        checker.check_program();

        let cli_id = checker.declared_types["CLI"];
        if let Some(Type::Namespace(ns)) = checker.get_type(cli_id) {
            assert!(ns.decorator_declarations.contains_key("command"));
            assert!(ns.decorator_declarations.contains_key("flag"));
            assert!(ns.decorator_declarations.contains_key("arg"));
        }
    }

    #[test]
    fn test_register_decorator_in_existing_namespace() {
        // Register a decorator in "TypeSpec" namespace which already exists
        let parse_result = parser::parse("model Pet { name: string; }");
        let mut checker = Checker::new();
        checker.set_parse_result(parse_result.root_id, parse_result.builder.clone());
        checker.register_decorator("myCustom", "TypeSpec", "unknown");
        checker.check_program();

        let ts_id = checker.declared_types["TypeSpec"];
        if let Some(Type::Namespace(ns)) = checker.get_type(ts_id) {
            assert!(ns.decorator_declarations.contains_key("myCustom"));
        }
    }

    #[test]
    fn test_register_decorator_no_duplicate() {
        // Register a decorator with same name as existing std decorator
        let parse_result = parser::parse("model Pet { name: string; }");
        let mut checker = Checker::new();
        checker.set_parse_result(parse_result.root_id, parse_result.builder.clone());
        // "doc" already exists in TypeSpec namespace
        checker.register_decorator("doc", "TypeSpec", "unknown");
        checker.check_program();

        // Should not create a duplicate
        let ts_id = checker.declared_types["TypeSpec"];
        if let Some(Type::Namespace(ns)) = checker.get_type(ts_id) {
            let count = ns
                .decorator_declaration_names
                .iter()
                .filter(|n| *n == "doc")
                .count();
            assert_eq!(count, 1, "should not duplicate existing decorator");
        }
    }

    #[test]
    fn test_custom_decorator_in_type_graph() {
        // Verify the decorator type was created correctly
        let parse_result = parser::parse("model Pet { name: string; }");
        let mut checker = Checker::new();
        checker.set_parse_result(parse_result.root_id, parse_result.builder.clone());
        checker.register_decorator("command", "CLI", "Operation");
        checker.check_program();

        let cli_id = checker.declared_types["CLI"];
        if let Some(Type::Namespace(ns)) = checker.get_type(cli_id) {
            let dec_id = ns.decorator_declarations["command"];
            if let Some(Type::Decorator(dt)) = checker.get_type(dec_id) {
                assert_eq!(dt.name, "command");
                assert_eq!(dt.target_type, "Operation");
                assert_eq!(dt.namespace, Some(cli_id));
            } else {
                panic!("command should be a decorator type");
            }
        }
    }

    #[test]
    fn test_decorator_with_keyword_name() {
        // "flag", "arg", "env" are reserved keywords in TypeSpec
        // They should work when registered programmatically
        let parse_result = parser::parse("model Config { name: string; }");
        let mut checker = Checker::new();
        checker.set_parse_result(parse_result.root_id, parse_result.builder.clone());
        checker.register_decorators(vec![
            ("flag", "CLI", "unknown"),
            ("arg", "CLI", "unknown"),
            ("env", "CLI", "unknown"),
        ]);
        checker.check_program();

        let cli_id = checker.declared_types["CLI"];
        if let Some(Type::Namespace(ns)) = checker.get_type(cli_id) {
            assert!(ns.decorator_declarations.contains_key("flag"));
            assert!(ns.decorator_declarations.contains_key("arg"));
            assert!(ns.decorator_declarations.contains_key("env"));
        }
    }

    #[test]
    fn test_std_decorators_still_work() {
        // Ensure std decorators are not affected by custom decorator registration
        let checker = check("model Pet { name: string; }");

        let ts_id = checker.declared_types["TypeSpec"];
        if let Some(Type::Namespace(ns)) = checker.get_type(ts_id) {
            assert!(ns.decorator_declarations.contains_key("doc"));
            assert!(ns.decorator_declarations.contains_key("tag"));
            assert!(ns.decorator_declarations.contains_key("error"));
        }
    }
}
