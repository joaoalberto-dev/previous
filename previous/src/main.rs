use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(name = "previouscc")]
#[command(version = "0.1.0")]
#[command(about = "Previous Schema Compiler - Binary protocol and BFF framework compiler", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Input schema file (.pr)
    #[arg(value_name = "FILE")]
    input: Option<PathBuf>,

    /// Output directory for generated code
    #[arg(short, long, value_name = "DIR", default_value = "./generated")]
    out: PathBuf,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile a schema file
    Compile {
        /// Input schema file (.pr)
        input: PathBuf,

        /// Output directory for generated code
        #[arg(short, long, value_name = "DIR", default_value = "./generated")]
        out: PathBuf,

        /// Enable verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    /// Show version information
    Version,
    /// Run demo examples
    Demo,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Compile { input, out, verbose }) => {
            compile_command(input, out, verbose);
        }
        Some(Commands::Version) => {
            println!("previouscc {}", env!("CARGO_PKG_VERSION"));
            println!("Previous Schema Compiler");
        }
        Some(Commands::Demo) => {
            run_demo();
        }
        None => {
            // Default behavior: compile if input file provided
            if let Some(input) = cli.input {
                compile_command(input, cli.out, cli.verbose);
            } else {
                // No input file, run demo
                run_demo();
            }
        }
    }
}

fn compile_command(input: PathBuf, out: PathBuf, verbose: bool) {
    let options = previous::CliOptions {
        input_file: input.clone(),
        output_dir: out.clone(),
        verbose,
    };

    println!("Previous Compiler v{}", env!("CARGO_PKG_VERSION"));
    println!();

    if verbose {
        println!("Input:  {}", input.display());
        println!("Output: {}", out.display());
        println!();
    }

    match previous::compile_file(&options) {
        Ok(_) => {
            println!("✓ Compilation successful!");
            println!();
            println!("Generated files:");
            println!("  {}/client.ts", out.display());
            println!("  {}/server.ts", out.display());
            println!();
            println!("Next steps:");
            println!("  - Client: Import client.ts for binary decoding");
            println!("  - Server: Import server.ts for binary encoding");
        }
        Err(e) => {
            eprintln!("✗ Compilation failed!");
            eprintln!();
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

fn run_demo() {
    println!("Previous Compiler v{}", env!("CARGO_PKG_VERSION"));
    println!();
    println!("=== Demo Mode ===");
    println!();

    // Test 1: Valid schema with no cycles + Code Generation
    println!("Test 1: Valid Schema (No Cycles)");
    println!("{}", "=".repeat(50));

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
            println!("✓ Compilation successful!");
            println!();
            println!("Resources compiled: {}", output.ir.resources.len());
            for resource in &output.ir.resources {
                println!("  - {} ({} fields)", resource.name, resource.fields.len());
            }
            println!();

            println!("Generated code:");
            println!("  TypeScript Client: {} lines", output.generated_code.typescript_client.lines().count());
            println!("  TypeScript Server: {} lines", output.generated_code.typescript_server.lines().count());
            println!();

            // Show sample TypeScript Client
            println!("TypeScript Client Sample (User interface):");
            println!("{}", "-".repeat(50));
            let ts_lines: Vec<&str> = output.generated_code.typescript_client.lines().collect();
            let user_start = ts_lines.iter().position(|l| l.contains("export interface IUser")).unwrap_or(0);
            for line in &ts_lines[user_start..user_start.min(ts_lines.len()).saturating_add(8).min(ts_lines.len())] {
                println!("{}", line);
            }
            println!();

            // Show sample TypeScript Server
            println!("TypeScript Server Sample (User class):");
            println!("{}", "-".repeat(50));
            let ts_server_lines: Vec<&str> = output.generated_code.typescript_server.lines().collect();
            let ts_server_user_start = ts_server_lines.iter().position(|l| l.contains("export class User")).unwrap_or(0);
            for line in &ts_server_lines[ts_server_user_start..ts_server_user_start.min(ts_server_lines.len()).saturating_add(10).min(ts_server_lines.len())] {
                println!("{}", line);
            }
            println!();

            // Binary Encoding Demo
            println!("Binary Encoding Demo:");
            println!("{}", "-".repeat(50));
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
                    println!("Encoded User to {} bytes", bytes.len());
                    println!("Hex: {}", bytes.iter().take(20).map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join(" "));
                    if bytes.len() > 20 {
                        println!("     ...");
                    }
                }
                Err(e) => eprintln!("Encoding error: {}", e),
            }
        }
        Err(e) => {
            eprintln!("✗ Compilation error: {}", e);
        }
    }

    println!();
    println!();

    // Test 2: Cycle Detection
    println!("Test 2: Cycle Detection (A ↔ B)");
    println!("{}", "=".repeat(50));

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
            eprintln!("✗ ERROR: Should have detected cycle!");
        }
        Err(e) => {
            println!("✓ Correctly detected cycle");
            println!("  Error: {}", e);
        }
    }

    println!();
    println!();

    // Test 3: Self-Reference Detection
    println!("Test 3: Self-Reference Detection (A → A)");
    println!("{}", "=".repeat(50));

    let self_ref_schema = r#"
        resource TreeNode {
            string value
            list TreeNode children
        }
    "#;

    match previous::compile_schema(self_ref_schema) {
        Ok(_) => {
            eprintln!("✗ ERROR: Should have detected self-reference!");
        }
        Err(e) => {
            println!("✓ Correctly detected self-reference");
            println!("  Error: {}", e);
        }
    }

    println!();
    println!();
    println!("Demo complete!");
    println!();
    println!("Try compiling a schema file:");
    println!("  previouscc <schema.pr> --out ./generated");
    println!();
    println!("Or use the compile subcommand:");
    println!("  previouscc compile <schema.pr> --out ./generated --verbose");
}
