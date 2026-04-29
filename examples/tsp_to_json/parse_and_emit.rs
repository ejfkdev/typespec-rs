//! Parse TypeSpec and Emit JSON/YAML
//!
//! This example demonstrates how to:
//! 1. Parse TypeSpec code using the parser API
//! 2. Emit structured JSON and YAML output using the emit API

use typespec_rs::emit::{to_json, to_yaml};

const PETSTORE_TSP: &str = r#"
    alias uuid = string;

    model Pet {
        id: uuid;
        name: string;
        age: int32;
        type: "dog" | "cat" | "bird";
    }

    interface PetStore {
        listPets(): Pet[];
        getPet(id: uuid): Pet;
        createPet(pet: Pet): Pet;
    }
"#;

const BLOG_TSP: &str = r#"
    model User {
        id: string;
        email: string;
        name: string;
        status: "active" | "inactive" | "suspended";
    }

    model Post {
        id: string;
        title: string;
        content: string;
        authorId: string;
        tags: string[];
        publishedAt?: utcDateTime;
    }

    enum Category {
        Tech,
        Life,
        Travel,
        Food,
    }

    union SearchResult {
        User,
        Post,
        Category,
    }

    interface Blog {
        listPosts(): Post[];
        getPost(id: string): Post;
        createPost(post: Post): Post;
        search(query: string): SearchResult[];
    }
"#;

fn main() {
    println!("=== PetStore API ===\n");
    println!("Input:\n{}", PETSTORE_TSP);

    match to_yaml(PETSTORE_TSP) {
        Ok(result) => {
            println!("--- YAML Output ---");
            println!("{}", result.output);
        }
        Err(e) => eprintln!("YAML error: {}", e),
    }

    match to_json(PETSTORE_TSP) {
        Ok(result) => {
            println!("--- JSON Output ---");
            println!("{}", result.output);
        }
        Err(e) => eprintln!("JSON error: {}", e),
    }

    println!("\n=== Blog API ===\n");
    println!("Input:\n{}", BLOG_TSP);

    match to_yaml(BLOG_TSP) {
        Ok(result) => {
            println!("--- YAML Output ---");
            println!("{}", result.output);
        }
        Err(e) => eprintln!("YAML error: {}", e),
    }

    match to_json(BLOG_TSP) {
        Ok(result) => {
            println!("--- JSON Output ---");
            println!("{}", result.output);
        }
        Err(e) => eprintln!("JSON error: {}", e),
    }
}
