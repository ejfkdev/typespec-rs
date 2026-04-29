//! Model definition examples
//!
//! Demonstrates how to parse and emit Model type definitions

use typespec_rs::emit::{to_json, to_yaml};

fn main() {
    println!("=== Model Examples ===\n");

    // Model with full property types
    let user_model = r#"
        model User {
            // Basic fields
            id: string;
            name: string;
            email: string;

            // Optional properties
            age?: int32;
            phone?: string;

            // Union type
            role: "admin" | "user" | "guest";

            // Array type
            tags: string[];

            // Nested model
            address: Address;
        }

        model Address {
            street: string;
            city: string;
            country: string;
            zipCode: string;
        }
    "#;

    println!("Input TypeSpec:");
    println!("{}", user_model);

    println!("\n--- YAML Output ---");
    let yaml = to_yaml(user_model).unwrap();
    println!("{}", yaml.output);

    println!("\n--- JSON Output ---");
    let json = to_json(user_model).unwrap();
    println!("{}", json.output);
}
