//! Integration tests for TypeSpec-Rust
//!
//! These tests demonstrate how a developer would use this library
//! to define interfaces, parse TypeSpec code, and convert to YAML/JSON.

/// Helper function to parse TypeSpec code and get diagnostics
fn parse_typespec_code(code: &str) -> Vec<String> {
    use typespec_rs::checker::Checker;
    use typespec_rs::parser::parse;

    let result = parse(code);
    let mut diags: Vec<String> = result
        .diagnostics
        .iter()
        .map(|d| format!("{}: {}", d.code, d.message))
        .collect();
    let mut checker = Checker::new();
    checker.set_parse_result(result.root_id, result.builder);
    checker.check_program();
    for d in checker.diagnostics() {
        diags.push(format!("{}: {}", d.code, d.message));
    }
    diags
}

/// Example: Define a simple model
#[cfg(test)]
mod model_definition_tests {
    use super::*;

    #[test]
    fn test_define_simple_model() {
        // Developer perspective: Define a simple model
        let code = r#"
            model Pet {
                name: string;
                age: int32;
            }
        "#;

        let diagnostics = parse_typespec_code(code);
        assert!(
            diagnostics.is_empty(),
            "Expected no errors, got: {:?}",
            diagnostics
        );
    }

    #[test]
    fn test_define_model_with_optional_field() {
        let code = r#"
            model User {
                id: int32;
                email: string;
                nickname?: string;
            }
        "#;

        let diagnostics = parse_typespec_code(code);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_define_model_with_array_property() {
        let code = r#"
            model Pet {
                name: string;
                tags: string[];
            }
        "#;

        let diagnostics = parse_typespec_code(code);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_define_model_with_reference() {
        let code = r#"
            model Address {
                street: string;
                city: string;
            }

            model User {
                name: string;
                address: Address;
            }
        "#;

        let diagnostics = parse_typespec_code(code);
        assert!(diagnostics.is_empty());
    }
}

/// Example: Define interfaces and operations
#[cfg(test)]
mod interface_definition_tests {
    use super::*;

    #[test]
    fn test_define_interface() {
        let code = r#"
            model Pet { name: string; }
            interface PetStore {
                getPet(petId: int32): Pet;
            }
        "#;

        let diagnostics = parse_typespec_code(code);
        assert!(
            diagnostics.is_empty(),
            "Unexpected diagnostics: {:?}",
            diagnostics
        );
    }

    #[test]
    fn test_define_interface_with_multiple_operations() {
        let code = r#"
            model Pet { name: string; }
            interface PetStore {
                getPet(petId: int32): Pet;
                createPet(pet: Pet): Pet;
            }
        "#;

        let diagnostics = parse_typespec_code(code);
        assert!(
            diagnostics.is_empty(),
            "Unexpected diagnostics: {:?}",
            diagnostics
        );
    }
}

/// Example: Define enums
#[cfg(test)]
mod enum_definition_tests {
    use super::*;

    #[test]
    fn test_define_string_enum() {
        let code = r#"
            enum PetType {
                dog,
                cat,
                bird,
            }
        "#;

        let diagnostics = parse_typespec_code(code);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_define_int_enum() {
        let code = r#"
            enum Status {
                pending: 0,
                active: 1,
                completed: 2,
            }
        "#;

        let diagnostics = parse_typespec_code(code);
        assert!(diagnostics.is_empty());
    }
}

/// Example: Define unions
#[cfg(test)]
mod union_definition_tests {
    use super::*;

    #[test]
    fn test_define_simple_union() {
        let code = r#"
            alias PetType = "dog" | "cat" | "bird";
        "#;

        let diagnostics = parse_typespec_code(code);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_define_model_with_union_property() {
        let code = r#"
            model Response {
                status: "success" | "error";
                data?: unknown;
            }
        "#;

        let diagnostics = parse_typespec_code(code);
        assert!(diagnostics.is_empty());
    }
}

/// Example: Define scalars
#[cfg(test)]
mod scalar_definition_tests {
    use super::*;

    #[test]
    fn test_define_custom_scalar() {
        let code = r#"
            scalar uuid extends string;
            scalar email extends string;
        "#;

        let diagnostics = parse_typespec_code(code);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_define_scalar_with_decorators() {
        let code = r#"
            @format("uuid")
            scalar uuid extends string;
        "#;

        let diagnostics = parse_typespec_code(code);
        assert!(diagnostics.is_empty());
    }
}

/// Example: Using decorators
#[cfg(test)]
mod decorator_tests {
    use super::*;

    #[test]
    fn test_model_with_decorators() {
        let code = r#"
            @doc("A pet in the store")
            model Pet {
                @doc("The pet's name")
                name: string;

                @minimum(0)
                age: int32;
            }
        "#;

        let diagnostics = parse_typespec_code(code);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_model_with_deprecated() {
        let code = r#"
            @deprecated("Use PetV2 instead")
            model Pet {
                name: string;
            }
        "#;

        let diagnostics = parse_typespec_code(code);
        assert!(diagnostics.is_empty());
    }
}

/// Example: Namespace usage
#[cfg(test)]
mod namespace_tests {
    use super::*;

    #[test]
    fn test_simple_namespace() {
        let code = r#"
            namespace PetStore {
                model Pet {
                    name: string;
                }
            }
        "#;

        let diagnostics = parse_typespec_code(code);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_nested_namespace() {
        let code = r#"
            namespace PetStore.Models {
                model Pet {
                    name: string;
                }
            }
        "#;

        let diagnostics = parse_typespec_code(code);
        assert!(diagnostics.is_empty());
    }
}

/// Example: Using the path utilities
#[cfg(test)]
mod path_utils_usage_tests {
    use typespec_rs::path_utils::*;

    #[test]
    fn test_join_paths_for_file_paths() {
        // Developer perspective: joining paths for file operations
        let base = "/project/src";
        let path = "models/pet.ts";

        let result = join_paths(base, path);
        assert_eq!(result, "/project/src/models/pet.ts");
    }

    #[test]
    fn test_normalize_path_with_dots() {
        // Normalizing paths with . and ..
        assert_eq!(normalize_path("/path/to/./file"), "/path/to/file");
        assert_eq!(normalize_path("/path/to/../file"), "/path/file");
    }

    #[test]
    fn test_get_directory_and_file_name() {
        // Splitting a path into directory and filename
        let path = "/project/src/models/pet.ts";

        let dir = get_directory_path(path);
        let file = get_base_file_name(path);

        assert_eq!(dir, "/project/src/models");
        assert_eq!(file, "pet.ts");
    }

    #[test]
    fn test_get_file_extension() {
        assert_eq!(get_any_extension_from_path("pet.ts"), ".ts");
        assert_eq!(get_any_extension_from_path("config.json"), ".json");
        assert_eq!(get_any_extension_from_path("noextension"), "");
    }
}

/// Example: Using RekeyableMap for ordered mapping
#[cfg(test)]
mod rekeyable_map_usage_tests {
    use typespec_rs::utils::RekeyableMap;

    #[test]
    fn test_insertion_order_preserved() {
        // Developer perspective: using a map that maintains insertion order
        let mut map = RekeyableMap::new();
        map.insert("z".to_string(), 1);
        map.insert("a".to_string(), 2);
        map.insert("m".to_string(), 3);

        let keys: Vec<_> = map.keys().cloned().collect();
        assert_eq!(keys, vec!["z", "a", "m"]);
    }

    #[test]
    fn test_rekey_maintains_order() {
        // Rekeying an entry should maintain its position
        let mut map = RekeyableMap::new();
        map.insert("old_key".to_string(), 1);
        map.insert("a".to_string(), 2);
        map.insert("b".to_string(), 3);

        map.rekey(&"old_key".to_string(), "new_key".to_string());

        let keys: Vec<_> = map.keys().cloned().collect();
        assert_eq!(keys, vec!["new_key", "a", "b"]);
    }
}

/// Example: Using performance timing
#[cfg(test)]
mod perf_timing_tests {
    use typespec_rs::perf::{Timer, time};

    #[test]
    fn test_timer_basic() {
        let timer = Timer::new();
        std::thread::sleep(std::time::Duration::from_millis(1));
        let elapsed = timer.end();

        assert!(elapsed >= 0.0, "Elapsed time should be non-negative");
    }

    #[test]
    fn test_time_function() {
        let (elapsed, result) = time(|| {
            std::thread::sleep(std::time::Duration::from_millis(1));
            42
        });

        assert!(elapsed >= 0.0);
        assert_eq!(result, 42);
    }

    #[test]
    fn test_perf_reporter() {
        use typespec_rs::perf::PerfReporter;

        let reporter = PerfReporter::new();

        let result = reporter.time("computation", || {
            std::thread::sleep(std::time::Duration::from_millis(1));
            "done"
        });

        assert_eq!(result, "done");
        assert!(reporter.get_measure("computation").is_some());
    }
}

/// Example: Using MIME type parsing
#[cfg(test)]
mod mime_type_usage_tests {
    use typespec_rs::mime_type::parse_mime_type;

    #[test]
    fn test_parse_common_mime_types() {
        // Developer perspective: parsing MIME types from HTTP headers
        let json = parse_mime_type("application/json").unwrap();
        assert_eq!(json.mime_type, "application");
        assert_eq!(json.subtype, "json");

        let xml = parse_mime_type("text/xml").unwrap();
        assert_eq!(xml.mime_type, "text");
        assert_eq!(xml.subtype, "xml");
    }

    #[test]
    fn test_parse_mime_type_with_suffix() {
        // e.g., application/vnd.api+json
        let result = parse_mime_type("application/vnd.api+json").unwrap();
        assert_eq!(result.suffix, Some("json".to_string()));
    }

    #[test]
    fn test_parse_invalid_mime_type() {
        assert!(parse_mime_type("invalid").is_none());
        assert!(parse_mime_type("").is_none());
        assert!(parse_mime_type("text/").is_none());
    }
}

/// Example: Using code fix utilities
#[cfg(test)]
mod code_fix_usage_tests {
    use typespec_rs::code_fixes::{
        CodeFixEdit, InsertTextCodeFixEdit, ReplaceTextCodeFixEdit, apply_code_fixes_on_text,
    };

    #[test]
    fn test_insert_text() {
        let content = "Hello World";
        let edits = vec![CodeFixEdit::InsertText(InsertTextCodeFixEdit {
            pos: 5,
            text: " Beautiful".to_string(),
        })];

        let result = apply_code_fixes_on_text(content, &edits);
        assert_eq!(result, "Hello Beautiful World");
    }

    #[test]
    fn test_replace_text() {
        let content = "Hello World";
        let edits = vec![CodeFixEdit::ReplaceText(ReplaceTextCodeFixEdit {
            pos: 0,
            end: 5,
            text: "Hi".to_string(),
        })];

        let result = apply_code_fixes_on_text(content, &edits);
        assert_eq!(result, "Hi World");
    }

    #[test]
    fn test_multiple_edits_sorted_by_position() {
        let content = "Hello World";
        let edits = vec![
            CodeFixEdit::InsertText(InsertTextCodeFixEdit {
                pos: 5,
                text: " there".to_string(),
            }),
            CodeFixEdit::ReplaceText(ReplaceTextCodeFixEdit {
                pos: 0,
                end: 5,
                text: "Hi".to_string(),
            }),
        ];

        let result = apply_code_fixes_on_text(content, &edits);
        assert_eq!(result, "Hi there World");
    }
}

/// Example: Using source file utilities
#[cfg(test)]
mod source_file_usage_tests {
    use typespec_rs::source_file::SourceFile;

    #[test]
    fn test_source_file_creation() {
        let content = "line1\nline2\nline3".to_string();
        let file = SourceFile::new(content, "test.tsp".to_string());

        assert_eq!(file.path, "test.tsp");
    }

    #[test]
    fn test_get_line_and_character_from_position() {
        let content = "line1\nline2\nline3".to_string();
        let mut file = SourceFile::new(content, "test.tsp".to_string());

        // Position 0 is in line 0, character 0 (0-indexed)
        let pos = file.get_line_and_character_of_position(0);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.character, 0);

        // Position after first newline is in line 1, character 0 (0-indexed)
        let pos = file.get_line_and_character_of_position(6);
        assert_eq!(pos.line, 1);
        assert_eq!(pos.character, 0);
    }

    #[test]
    fn test_get_line_and_character_third_line() {
        let content = "line1\nline2\nline3".to_string();
        let mut file = SourceFile::new(content, "test.tsp".to_string());

        // Position after second newline is in line 2, character 0 (0-indexed)
        let pos = file.get_line_and_character_of_position(12);
        assert_eq!(pos.line, 2);
        assert_eq!(pos.character, 0);
    }
}

/// Example: Real-world scenario - API definition
#[cfg(test)]
mod api_definition_scenario_tests {
    use super::*;

    #[test]
    fn test_define_pet_store_api() {
        // A complete API definition example
        let code = r#"
            alias uuid = string;

            model Pet {
                id: uuid;
                name: string;
                @minimum(0)
                age: int32;
                type: "dog" | "cat" | "bird";
            }

            model Error {
                code: int32;
                message: string;
            }

            interface PetStore {
                @get
                listPets(): Pet[];

                @get
                getPet(id: uuid): Pet | Error;

                @post
                createPet(pet: Pet): Pet;

                @delete
                deletePet(id: uuid): void;
            }
        "#;

        let diagnostics = parse_typespec_code(code);
        assert!(
            diagnostics.is_empty(),
            "Expected no errors, got: {:?}",
            diagnostics
        );
    }

    #[test]
    fn test_define_http_service_with_routes() {
        let code = r#"
            @serviceTitle("User Management API")
            @serviceVersion("1.0.0")
            model User {
                id: string;
                email: string;
                name: string;
            }

            model Error {
                code: int32;
                message: string;
            }

            model CreateUserRequest {
                email: string;
                name: string;
            }

            model UpdateUserRequest {
                email?: string;
                name?: string;
            }

            interface UserService {
                createUser(user: CreateUserRequest): User;

                getUser(id: string): User | Error;

                updateUser(id: string, user: UpdateUserRequest): User;

                deleteUser(id: string): void;
            }
        "#;

        let diagnostics = parse_typespec_code(code);
        assert!(
            diagnostics.is_empty(),
            "Unexpected diagnostics: {:?}",
            diagnostics
        );
    }
}

/// Example: Using the param message system
#[cfg(test)]
mod param_message_usage_tests {
    use std::collections::HashMap;
    use typespec_rs::param_message::CallableMessage;

    #[test]
    fn test_interpolate_diagnostic_message() {
        // Developer perspective: creating diagnostic messages with interpolation
        let msg = CallableMessage::new(
            vec![
                "Expected ".to_string(),
                " but got ".to_string(),
                ".".to_string(),
            ],
            vec!["expected".to_string(), "actual".to_string()],
        );

        let mut values = HashMap::new();
        values.insert("expected".to_string(), "string".to_string());
        values.insert("actual".to_string(), "int32".to_string());

        let result = msg.invoke(&values);
        assert_eq!(result, "Expected string but got int32.");
    }

    #[test]
    fn test_interpolate_with_missing_value() {
        let msg = CallableMessage::new(
            vec!["File ".to_string(), " not found.".to_string()],
            vec!["filename".to_string()],
        );

        let mut values = HashMap::new();
        values.insert("other".to_string(), "test.txt".to_string());

        // Missing key - interpolation is skipped
        let result = msg.invoke(&values);
        assert_eq!(result, "File  not found.");
    }
}
