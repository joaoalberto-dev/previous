fn main() {
    previous::run();

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
        }
        Err(e) => eprintln!("Compilation error: {}", e),
    }
}
