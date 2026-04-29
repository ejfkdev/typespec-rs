//! PetStore API Example - Real Parser
//!
//! Actually parses TypeSpec code and emits JSON using typespec-rs.

use typespec_rs::parser::{AstBuilder, AstNode, parse};

fn get_name(builder: &AstBuilder, node_id: u32) -> String {
    if let Some(AstNode::Identifier(id)) = builder.nodes.get(&node_id) {
        id.value.clone()
    } else {
        "<unknown>".to_string()
    }
}

fn get_type_string(builder: &AstBuilder, node_id: u32) -> String {
    match builder.nodes.get(&node_id) {
        Some(AstNode::Identifier(id)) => id.value.clone(),
        Some(AstNode::ArrayExpression(arr)) => {
            format!("{}[]", get_type_string(builder, arr.element_type))
        }
        Some(AstNode::TypeReference(tr)) => get_name(builder, tr.name),
        Some(AstNode::UnionExpression(u)) => {
            let opts: Vec<String> = u
                .options
                .iter()
                .map(|&opt| get_type_string(builder, opt))
                .collect();
            opts.join(" | ")
        }
        Some(AstNode::StringLiteral(s)) => format!("\"{}\"", s.value),
        _ => "<type>".to_string(),
    }
}

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║           PetStore API - Real Parser Output              ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let typespec_code = r#"
        alias uuid = string;

        scalar email extends string;

        enum PetStatus {
            available,
            pending,
            sold,
        }

        model Error {
            code: int32;
            message: string;
            details?: string;
        }

        model Pet {
            id: uuid;
            name: string;
            @minimum(0)
            age: int32;
            type: "dog" | "cat" | "bird" | "fish";
            status: PetStatus;
            tags?: string[];
            ownerId?: uuid;
        }

        model CreatePetRequest {
            name: string;
            type: "dog" | "cat" | "bird" | "fish";
            age: int32;
            ownerId?: uuid;
        }

        model UpdatePetRequest {
            name?: string;
            age?: int32;
            status?: PetStatus;
            tags?: string[];
        }

        model PetListResponse {
            items: Pet[];
            total: int32;
            offset: int32;
            limit: int32;
        }

        interface PetStore {
            @get
            listPets(limit?: int32, offset?: int32, status?: PetStatus): PetListResponse | Error;

            @get
            getPet(id: uuid): Pet | Error;

            @post
            createPet(pet: CreatePetRequest): Pet | Error;

            @put
            updatePet(id: uuid, pet: UpdatePetRequest): Pet | Error;

            @delete
            deletePet(id: uuid): void | Error;
        }
    "#;

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📥 Input TypeSpec:");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    println!("{}\n", typespec_code);

    // Parse
    let result = parse(typespec_code);

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📤 Parsed JSON Output:");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    println!("{{");
    println!("  \"source\": \"TypeSpec\",");
    println!("  \"diagnostics\": [],\n");

    // Extract and print declarations
    let mut models: Vec<(String, Vec<(String, String)>)> = Vec::new();
    #[allow(clippy::type_complexity)]
    let mut interfaces: Vec<(String, Vec<(String, String, Vec<String>)>)> = Vec::new();
    let mut enums: Vec<(String, Vec<String>)> = Vec::new();
    let mut aliases: Vec<String> = Vec::new();

    for node in result.builder.nodes.values() {
        match node {
            AstNode::ModelDeclaration(m) => {
                let name = get_name(&result.builder, m.name);
                let props: Vec<(String, String)> = m
                    .properties
                    .iter()
                    .filter_map(|&prop_id| {
                        if let Some(AstNode::ModelProperty(prop)) =
                            result.builder.nodes.get(&prop_id)
                        {
                            Some((
                                get_name(&result.builder, prop.name),
                                get_type_string(&result.builder, prop.value),
                            ))
                        } else {
                            None
                        }
                    })
                    .collect();
                models.push((name, props));
            }
            AstNode::InterfaceDeclaration(i) => {
                let name = get_name(&result.builder, i.name);
                let ops: Vec<(String, String, Vec<String>)> = i
                    .operations
                    .iter()
                    .filter_map(|&op_id| {
                        if let Some(AstNode::OperationDeclaration(op)) =
                            result.builder.nodes.get(&op_id)
                        {
                            let decorators: Vec<String> = op
                                .decorators
                                .iter()
                                .filter_map(|&d| {
                                    if let Some(AstNode::DecoratorExpression(dec)) =
                                        result.builder.nodes.get(&d)
                                    {
                                        Some(get_name(&result.builder, dec.target))
                                    } else {
                                        None
                                    }
                                })
                                .collect();
                            let return_type =
                                if let Some(AstNode::OperationSignatureDeclaration(sig)) =
                                    result.builder.nodes.get(&op.signature)
                                {
                                    get_type_string(&result.builder, sig.return_type)
                                } else {
                                    "<return>".to_string()
                                };
                            Some((get_name(&result.builder, op.name), return_type, decorators))
                        } else {
                            None
                        }
                    })
                    .collect();
                interfaces.push((name, ops));
            }
            AstNode::EnumDeclaration(e) => {
                let name = get_name(&result.builder, e.name);
                let members: Vec<String> = e
                    .members
                    .iter()
                    .filter_map(|&m| {
                        if let Some(AstNode::EnumMember(em)) = result.builder.nodes.get(&m) {
                            Some(get_name(&result.builder, em.name))
                        } else {
                            None
                        }
                    })
                    .collect();
                enums.push((name, members));
            }
            AstNode::AliasStatement(a) => {
                aliases.push(get_name(&result.builder, a.name));
            }
            _ => {}
        }
    }

    // Print models
    println!("  \"models\": {{");
    for (i, (name, props)) in models.iter().enumerate() {
        println!("    \"{}\": {{", name);
        for (j, (prop_name, prop_type)) in props.iter().enumerate() {
            let comma = if j < props.len() - 1 { "," } else { "" };
            println!("      \"{}\": \"{}\"{}", prop_name, prop_type, comma);
        }
        println!("    }}{}", if i < models.len() - 1 { "," } else { "" });
    }
    println!("  }},\n");

    // Print interfaces
    println!("  \"interfaces\": {{");
    for (i, (name, ops)) in interfaces.iter().enumerate() {
        println!("    \"{}\": {{", name);
        println!("      \"operations\": {{");
        for (j, (op_name, return_type, _decorators)) in ops.iter().enumerate() {
            let comma = if j < ops.len() - 1 { "," } else { "" };
            println!("        \"{}\": \"{}\"{}", op_name, return_type, comma);
        }
        println!("      }}");
        println!("    }}{}", if i < interfaces.len() - 1 { "," } else { "" });
    }
    println!("  }},\n");

    // Print enums
    println!("  \"enums\": {{");
    for (name, members) in enums.iter() {
        println!("    \"{}\": [{}]", name, members.join(", "));
    }
    println!("  }},\n");

    // Print aliases
    println!("  \"aliases\": [{}]", aliases.join(", "));

    println!("\n}}\n");

    // Summary
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📊 Summary:");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    println!("  Total nodes: {}", result.builder.nodes.len());
    println!("  Models: {}", models.len());
    println!("  Interfaces: {}", interfaces.len());
    println!("  Enums: {}", enums.len());
    println!("  Aliases: {}", aliases.len());

    if result.diagnostics.is_empty() {
        println!("\n  Status: ✅ No errors");
    } else {
        println!("\n  Status: ❌ {} errors", result.diagnostics.len());
    }
}
