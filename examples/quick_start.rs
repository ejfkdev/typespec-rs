//! Quick Start - Convert TypeSpec to YAML/JSON in 5 lines
//!
//! The simplest way to use the emit API

use typespec_rs::emit::*;

fn main() {
    println!("=== Quick Start ===\n");

    let tsp = r#"
        model User {
            id: string;
            name: string;
            email: string;
        }
    "#;

    // Method 1: Convert to YAML
    let yaml = to_yaml(tsp).unwrap();
    println!("YAML output:\n{}", yaml.output);

    // Method 2: Convert to JSON
    let json = to_json(tsp).unwrap();
    println!("\nJSON output:\n{}", json.output);

    // Method 3: Using convert() with a specific emitter
    let result = convert(tsp, YamlEmitter::new()).unwrap();
    println!("\nYAML via convert():\n{}", result.output);
}
