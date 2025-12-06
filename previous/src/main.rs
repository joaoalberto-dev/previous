fn main() {
    previous::run();

    // Test 1: Valid schema with no cycles + Binary Encoding Demo
    println!("\n=== Test 1: Valid Schema (No Cycles) ===");
    let schema = r#"
        resource User {
            string name
            string email
            optional number age
            bool active
        }

        resource Names {
            list string name
        }

        resource Users {
            list User users
        }

        resource Settings {
            nullable bool notifications
        }

        resource Notification {
            default(10) number interval
        }
    "#;

    match previous::compile_schema(schema) {
        Ok(output) => {
            println!("\nCompilation successful!");
            println!("Resources compiled: {}", output.ir.resources.len());
            for resource in &output.ir.resources {
                println!("\nResource: {}", resource.name);
                for field in &resource.fields {
                    println!(
                        "  [{}] {} {:?}{}{}{}",
                        field.index,
                        field.name,
                        field.field_type,
                        if field.nullable { " (nullable)" } else { "" },
                        if field.optional { " (optional)" } else { "" },
                        if field.default.is_some() {
                            " (has default)"
                        } else {
                            ""
                        }
                    );
                }
            }

            // Code Generation Demo
            println!("\n--- Code Generation Demo ---");
            println!("Generated {} TypeScript lines", output.generated_code.typescript_client.lines().count());
            println!("Generated {} Rust lines\n", output.generated_code.rust_server.lines().count());

            // Show sample of TypeScript client
            println!("TypeScript Client Sample (User resource):");
            let ts_lines: Vec<&str> = output.generated_code.typescript_client.lines().collect();
            let user_start = ts_lines.iter().position(|l| l.contains("export interface IUser")).unwrap_or(0);
            for line in &ts_lines[user_start..user_start.min(ts_lines.len()).saturating_add(15).min(ts_lines.len())] {
                println!("{}", line);
            }

            // Show sample of Rust server
            println!("\nRust Server Sample (User resource):");
            let rust_lines: Vec<&str> = output.generated_code.rust_server.lines().collect();
            let rust_user_start = rust_lines.iter().position(|l| l.contains("pub struct User")).unwrap_or(0);
            for line in &rust_lines[rust_user_start..rust_user_start.min(rust_lines.len()).saturating_add(20).min(rust_lines.len())] {
                println!("{}", line);
            }

            // Binary Encoding Demo
            println!("\n--- Binary Encoding Demo ---");
            let user_value = previous::Value::Resource(vec![
                previous::FieldValue {
                    name: "name".to_string(),
                    value: previous::Value::String("Alice".to_string()),
                    is_optional: false,
                    is_nullable: false,
                },
                previous::FieldValue {
                    name: "email".to_string(),
                    value: previous::Value::String("alice@example.com".to_string()),
                    is_optional: false,
                    is_nullable: false,
                },
                previous::FieldValue {
                    name: "age".to_string(),
                    value: previous::Value::Number(30),
                    is_optional: true,
                    is_nullable: false,
                },
                previous::FieldValue {
                    name: "active".to_string(),
                    value: previous::Value::Bool(true),
                    is_optional: false,
                    is_nullable: false,
                },
            ]);

            let mut encoder = previous::BinaryEncoder::new();
            match encoder.encode_value(&user_value, &previous::IRType::ResourceRef(0), &output.ir) {
                Ok(_) => {
                    let bytes = encoder.finish();
                    println!("Encoded User resource to {} bytes", bytes.len());
                    println!("Binary data (hex): {}", bytes.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join(" "));
                }
                Err(e) => eprintln!("Encoding error: {}", e),
            }
        }
        Err(e) => eprintln!("Compilation error: {}", e),
    }

    // Test 2: Schema with a cycle (A → B → A)
    println!("\n=== Test 2: Schema with Cycle (A ↔ B) ===");
    let cyclic_schema = r#"
        resource A {
            string name
            B reference
        }
        
        resource B {
            string title
            A parent
        }
    "#;

    match previous::compile_schema(cyclic_schema) {
        Ok(_) => {
            eprintln!("ERROR: Should have detected cycle!");
        }
        Err(e) => {
            println!("✓ Correctly detected cycle:");
            println!("  Error: {}", e);
        }
    }

    // Test 3: Schema with self-reference
    println!("\n=== Test 3: Schema with Self-Reference (A → A) ===");
    let self_ref_schema = r#"
        resource TreeNode {
            string value
            list TreeNode children
        }
    "#;

    match previous::compile_schema(self_ref_schema) {
        Ok(_) => {
            eprintln!("ERROR: Should have detected self-reference!");
        }
        Err(e) => {
            println!("✓ Correctly detected self-reference:");
            println!("  Error: {}", e);
        }
    }
}
