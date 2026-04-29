//! Program flow checking
//!
//! Ported from TypeSpec compiler program checking methods

use super::*;

impl Checker {
    /// Check the entire program
    pub fn check_program(&mut self) {
        // 1. Initialize standard types
        self.initialize_std_types();

        // 2. Initialize standard library decorators
        self.initialize_std_decorators();

        // 2b. Initialize standard enums, models, and functions
        self.initialize_std_enums_and_models();

        // 3. Create global namespace
        self.initialize_global_namespace();

        // 4. Check all source files
        self.check_source_file(self.root_id);

        // 5. Validate internal decorator usage
        self.internal_decorator_validation();

        // 6. Report unused using declarations
        self.report_unused_usings();
    }

    /// Check a single source file (TypeSpecScript node)
    pub fn check_source_file(&mut self, node_id: NodeId) {
        let ast = require_ast_or!(self);

        let node = match ast.id_to_node(node_id) {
            Some(n) => n.clone(),
            None => return,
        };

        let statements = match &node {
            AstNode::TypeSpecScript(script) => &script.statements,
            _ => return,
        };

        // Pre-registration pass: register all top-level declaration names
        // so that forward references can be resolved during the main checking pass.
        // TS: The binder pre-binds all declarations before the checker runs.
        self.pre_register_declarations(&ast, statements);

        let ctx = CheckContext::new();

        for &stmt_id in statements {
            self.check_node(&ctx, stmt_id);
        }

        // Finish the global namespace - populate its type collections
        if let Some(ns_id) = self.global_namespace_type {
            self.populate_namespace(ns_id);
            self.finish_type(ns_id);
        }
    }

    /// Pre-register all top-level declaration names so forward references work.
    /// Also recurses into namespace bodies to pre-register their declarations.
    pub(crate) fn pre_register_declarations(&mut self, ast: &AstBuilder, statements: &[NodeId]) {
        for &stmt_id in statements {
            let node = match ast.id_to_node(stmt_id) {
                Some(n) => n.clone(),
                None => continue,
            };

            // Extract (name_node_id, shell_type, child_statements) from each declaration
            let (name_node, shell, child_statements): (NodeId, TypeId, Option<Vec<NodeId>>) =
                match &node {
                    AstNode::ModelDeclaration(decl) => {
                        (decl.name, self.create_shell_type("Model", stmt_id), None)
                    }
                    AstNode::ScalarDeclaration(decl) => {
                        (decl.name, self.create_shell_type("Scalar", stmt_id), None)
                    }
                    AstNode::InterfaceDeclaration(decl) => (
                        decl.name,
                        self.create_shell_type("Interface", stmt_id),
                        None,
                    ),
                    AstNode::EnumDeclaration(decl) => {
                        (decl.name, self.create_shell_type("Enum", stmt_id), None)
                    }
                    AstNode::UnionDeclaration(decl) => {
                        (decl.name, self.create_shell_type("Union", stmt_id), None)
                    }
                    AstNode::AliasStatement(decl) => {
                        (decl.name, self.create_shell_type("Alias", stmt_id), None)
                    }
                    AstNode::OperationDeclaration(decl) => (
                        decl.name,
                        self.create_shell_type("Operation", stmt_id),
                        None,
                    ),
                    AstNode::NamespaceDeclaration(decl) => {
                        let name = Self::get_identifier_name(ast, decl.name);
                        if name.is_empty() {
                            continue;
                        }
                        let child_stmts = decl.statements.clone();
                        // Namespace merging: special handling
                        if let Some(&existing_id) = self.declared_types.get(&name) {
                            let existing_is_ns =
                                matches!(self.get_type(existing_id), Some(Type::Namespace(_)));
                            if existing_is_ns {
                                self.node_type_map.insert(stmt_id, existing_id);
                                let prev_ns = self.current_namespace;
                                self.current_namespace = Some(existing_id);
                                self.pre_register_declarations(ast, &child_stmts);
                                self.current_namespace = prev_ns;
                                continue;
                            }
                            self.error(
                                "duplicate-symbol",
                                &format!("Duplicate symbol: '{}'", name),
                            );
                            continue;
                        }
                        let type_id =
                            self.create_type(Type::Namespace(Box::new(NamespaceType::new(
                                self.next_type_id(),
                                name.clone(),
                                Some(stmt_id),
                                self.current_namespace,
                                false,
                            ))));
                        self.node_type_map.insert(stmt_id, type_id);
                        self.declared_types.insert(name, type_id);
                        let prev_ns = self.current_namespace;
                        self.current_namespace = Some(type_id);
                        self.pre_register_declarations(ast, &child_stmts);
                        self.current_namespace = prev_ns;
                        continue;
                    }
                    _ => continue,
                };

            // Common path: check name, check duplicate, register
            let name = Self::get_identifier_name(ast, name_node);
            if name.is_empty() {
                continue;
            }
            if self.declared_types.contains_key(&name) {
                self.error("duplicate-symbol", &format!("Duplicate symbol: '{}'", name));
                continue;
            }

            // Set the name on the shell type
            if let Some(t) = self.get_type_mut(shell) {
                t.set_name(name.clone());
            }

            self.node_type_map.insert(stmt_id, shell);
            self.declared_types.insert(name, shell);

            // Recurse into namespace body (not applicable here, kept for symmetry)
            if let Some(child_stmts) = child_statements {
                let prev_ns = self.current_namespace;
                self.current_namespace = Some(shell);
                self.pre_register_declarations(ast, &child_stmts);
                self.current_namespace = prev_ns;
            }
        }
    }

    /// Create a shell (unfinished) type for pre-registration.
    /// The name will be set later after duplicate checking.
    pub(crate) fn create_shell_type(&mut self, kind: &str, stmt_id: NodeId) -> TypeId {
        let ns = self.current_namespace;
        match kind {
            "Model" => self.create_type(Type::Model(ModelType::new(
                self.next_type_id(),
                String::new(),
                Some(stmt_id),
                ns,
            ))),
            "Scalar" | "Alias" => self.create_type(Type::Scalar(ScalarType::new(
                self.next_type_id(),
                String::new(),
                Some(stmt_id),
                ns,
                None,
            ))),
            "Interface" => self.create_type(Type::Interface(InterfaceType::new(
                self.next_type_id(),
                String::new(),
                Some(stmt_id),
                ns,
            ))),
            "Enum" => self.create_type(Type::Enum(EnumType::new(
                self.next_type_id(),
                String::new(),
                Some(stmt_id),
                ns,
            ))),
            "Union" => self.create_type(Type::Union(UnionType::new(
                self.next_type_id(),
                String::new(),
                Some(stmt_id),
                ns,
                false,
            ))),
            "Operation" => self.create_type(Type::Operation(OperationType::new(
                self.next_type_id(),
                String::new(),
                Some(stmt_id),
                ns,
            ))),
            _ => self.error_type,
        }
    }
}
