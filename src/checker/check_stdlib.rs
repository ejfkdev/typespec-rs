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
