//! Parse and Inspect Example
//!
//! This example demonstrates how to:
//! 1. Parse a TypeSpec code string into an AST
//! 2. Inspect the parsed AST structure
//! 3. Walk through the nodes

use typespec_rs::ast::types::SyntaxKind;
use typespec_rs::parser::{ParseOptions, Parser};

/// Parse TypeSpec code and return the parser result
fn parse_typespec(code: &str) -> typespec_rs::parser::ParseResult {
    let options = ParseOptions::default();
    let parser = Parser::new(code, options);
    parser.parse()
}

fn main() {
    println!("=== TypeSpec Parse and Inspect Example ===\n");

    // Example 1: Parse a simple model
    let code = r#"
        model Pet {
            name: string;
            age: int32;
        }
    "#;

    println!("Parsing code:\n{}", code);
    println!("---\n");

    let result = parse_typespec(code);

    println!("Parse diagnostics: {:?}", result.diagnostics);
    println!("Root ID: {}", result.root_id);
    println!("AST Builder has {} nodes", result.builder.nodes.len());

    // Example 2: Parse a more complex example with interface
    let complex_code = r#"
        alias uuid = string;

        model Pet {
            id: uuid;
            name: string;
            age: int32;
            type: "dog" | "cat" | "bird" | "fish";
        }

        interface PetStore {
            listPets(): Pet[];
            getPet(id: uuid): Pet;
            createPet(pet: Pet): Pet;
        }
    "#;

    println!("\nParsing complex code...");
    let result = parse_typespec(complex_code);

    println!("Diagnostics: {:?}", result.diagnostics.len());
    if result.diagnostics.is_empty() {
        println!("No parse errors!");
    }

    // Example 3: Show SyntaxKind values
    println!("\n=== Available SyntaxKind values ===\n");
    println!("ModelStatement: {:?}", SyntaxKind::ModelStatement as u32);
    println!(
        "InterfaceStatement: {:?}",
        SyntaxKind::InterfaceStatement as u32
    );
    println!("EnumStatement: {:?}", SyntaxKind::EnumStatement as u32);
    println!("UnionStatement: {:?}", SyntaxKind::UnionStatement as u32);

    println!("\n=== Example Complete ===");
}
