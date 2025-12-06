# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Previous** is a binary protocol and BFF (Backend For Frontend) framework built in Rust. It consists of:
- A schema definition language (.pr files)
- A compiler that transforms schemas into intermediate representation (IR)
- Client and server utilities for binary data exchange

The project is a compiler with multiple phases that converts schema definitions into validated IR, ready for binary encoding and code generation.

## Repository Structure

```
previous/
├── previous/           # Rust crate (the actual compiler)
│   ├── src/
│   │   ├── lib.rs     # Main compiler implementation (~1500 lines)
│   │   └── main.rs    # Demo runner with test cases
│   └── Cargo.toml
├── *.md               # Documentation files
└── PHASE2_*.md        # Phase 2 implementation documentation
```

**Important:** The Rust crate is located in the `previous/` subdirectory, not the repository root.

## Build and Test Commands

All commands must be run from the `previous/` directory (where Cargo.toml is located):

```bash
# Build the project
cargo build

# Run all tests (69 tests across all compiler phases)
cargo test

# Run the CLI tool (shows help or runs demo)
cargo run

# Run demo mode
cargo run -- demo

# Compile a schema file
cargo run -- examples/user.pr --out ./generated

# Compile with verbose output
cargo run -- examples/user.pr --out ./test_output --verbose

# Show help
cargo run -- --help

# Show version
cargo run -- version

# Run tests with output
cargo test -- --nocaptures

# Check code without building
cargo check
```

## Compiler Architecture

The compiler follows a complete 5-phase pipeline (all phases complete ✅):

### Phase 1: AST Construction ✅
- **Lexer**: Tokenizes input (.pr files)
- **Parser**: Builds abstract syntax tree (AST)
- **Validation**: Enforces PascalCase for resources, unique names
- **Output**: `Program` containing `Resource` nodes with `Field` definitions

### Phase 2: Type Resolution + IR ✅
- **TypeResolver**: Converts AST → IR with validated type references
  - Resolves `ASTType::Named(String)` → `IRType::ResourceRef(usize)`
  - Validates all type references exist
  - Recursively resolves list types
- **CycleDetector**: DFS-based cycle detection
  - Detects self-references: `A { list A }`
  - Detects simple cycles: `A { B } B { A }`
  - Detects deep cycles: `A → B → C → A`
  - Reports full cycle path in error messages
- **Output**: `IRProgram` with indexed resource references (no string lookups needed)

### Phase 3: Binary Encoding ✅
- **Value Types**: Runtime representation (`Value`, `FieldValue`)
- **BinaryEncoder**: Serializes values to binary format
  - **string**: u32 length (little-endian) + UTF-8 bytes
  - **number**: i64 (8 bytes, little-endian)
  - **bool**: 1 byte (0x00 = false, 0x01 = true)
  - **list**: u32 count + each item encoded recursively
  - **nullable**: 1 byte (0x00 = null, 0x01 = present) + value if present
  - **optional**: 1 byte (0x00 = absent, 0x01 = present) + value if present
  - **resource**: fields encoded in order (index is implicit)
- **Type Safety**: Runtime validation of values against IR types
- **Output**: Binary data (`Vec<u8>`) ready for transmission

### Phase 4: Code Generation ✅
- **CodeGenerator**: Transforms IR into production code
- **TypeScript Client**:
  - Interface definitions for each resource
  - BinaryReader utility for deserialization
  - Resource classes with lazy field access
  - Getter methods (getName(), getAge(), etc.)
  - toJSON() for debugging
  - Full type safety with proper TypeScript annotations
- **Rust Server**:
  - Type-safe struct definitions
  - Builder pattern for fluent construction
  - encode() method for binary serialization
  - to_value() for runtime conversion
  - Option<T> for nullable/optional fields
  - Debug and Clone derives
- **Output**: TypeScript client code + Rust server code ready for deployment

### Phase 5: CLI & File I/O ✅
- **Command-Line Tool**: Full-featured CLI with clap
- **Commands**:
  - `previouscc <file.pr> --out ./generated` - Compile schema file
  - `previouscc compile <file.pr>` - Explicit compile subcommand
  - `previouscc demo` - Run interactive demo
  - `previouscc version` - Show version info
- **File I/O**:
  - Read .pr files from filesystem
  - Write client.ts (TypeScript client)
  - Write server.rs (Rust server)
  - Auto-create output directories
- **Error Handling**: Clear messages with file context
- **Output**: Production-ready files on disk

## Key Data Structures

### AST Types (Phase 1)
```rust
Program           // Top-level: contains Vec<Resource>
Resource          // A schema resource with name and fields
Field             // name, field_type (ASTType), nullable, optional, default, index
ASTType           // Primitive(String) | Named(String) | List(Box<ASTType>)
```

### IR Types (Phase 2)
```rust
IRProgram         // Contains Vec<IRResource>
IRResource        // Resolved resource with name and Vec<IRField>
IRField           // name, field_type (IRType), attributes, index
IRType            // Primitive(String) | ResourceRef(usize) | List(Box<IRType>)
```

**Critical difference:** AST uses `Named(String)` for resource references, IR uses `ResourceRef(usize)` with validated indices.

### Value Types (Phase 3)
```rust
Value             // Runtime values: String(String) | Number(i64) | Bool(bool) |
                  // List(Vec<Value>) | Resource(Vec<FieldValue>) | Null | Absent
FieldValue        // name, value, is_optional, is_nullable
BinaryEncoder     // Encodes Value to Vec<u8> using IR types for validation
```

**Purpose:** `Value` types represent actual data at runtime, while AST/IR types represent the schema structure.

## Public API

```rust
// Parse schema into AST
pub fn parse_schema(input: &str) -> Result<Program, String>

// Compile schema into validated IR (main entry point)
pub fn compile_schema(input: &str) -> Result<CompiledOutput, String>

// Demo runner
pub fn run()
```

## Schema Language (.pr files)

Example schema:
```
resource User {
    string name
    string email
    optional number age
    bool active
}

resource Users {
    list User users
}

resource Settings {
    nullable bool notifications
}

resource Counter {
    default(10) number value
}
```

### Keywords
- Types: `string`, `number`, `bool`, `list`
- Attributes: `nullable`, `optional`, `default(value)`
- Declaration: `resource`

### Constraints
- Resource names must be PascalCase
- Resource names must be unique
- Field names must be unique within a resource
- No imports (single file only)
- No cyclic dependencies
- No comments supported

## Testing

**Test count:** 69 tests (all passing)
- 12 tests for Phase 1 (parsing, AST construction)
- 8 tests for IR structures
- 12 tests for TypeResolver
- 11 tests for CycleDetector
- 16 tests for BinaryEncoder (Phase 3)
- 12 tests for CodeGenerator (Phase 4)

### Test categories:
1. **Parse/AST tests**: Simple resources, multiple fields, attributes, lists
2. **IR structure tests**: Resource lookup, field indexing
3. **Type resolution tests**: Primitives, named types, lists, nested lists, undefined types
4. **Cycle detection tests**: Self-reference, simple cycles, deep cycles, DAG validation
5. **Binary encoding tests**: Primitives, lists, nullable, optional, resources, nested resources, type validation
6. **Code generation tests**: TypeScript interfaces/classes, Rust structs/impls, optional/nullable, lists, nested resources, builder pattern, imports/headers

## Development Notes

### Where to find things:
- **All compiler code**: `previous/src/lib.rs` (single file, ~2500 lines)
- **Lexer**: `Lexer` struct and `Token` enum
- **Parser**: `Parser` struct with recursive descent methods
- **AST**: `Program`, `Resource`, `Field`, `ASTType` structs
- **IR**: `IRProgram`, `IRResource`, `IRField`, `IRType` structs
- **Value types**: `Value`, `FieldValue` enums/structs (Phase 3)
- **Binary encoding**: `BinaryEncoder` struct (Phase 3)
- **Code generation**: `CodeGenerator`, `GeneratedCode` structs (Phase 4)
- **Type resolution**: `TypeResolver` struct
- **Cycle detection**: `CycleDetector` struct with DFS algorithm
- **Compiler orchestration**: `Compiler` struct and `compile_schema()` function
- **Tests**: `#[cfg(test)] mod tests` at bottom of lib.rs

### Code organization in lib.rs:
1. Language specification (comments at top)
2. AST type definitions
3. IR type definitions
4. Binary encoding model (Value, FieldValue, BinaryEncoder)
5. Code generation (CodeGenerator, TypeScript/Rust templates)
6. Lexer implementation
7. Parser implementation
8. TypeResolver implementation
9. CycleDetector implementation
10. Compiler implementation
11. Public API functions
12. Test module (69 tests)

### When modifying the compiler:
- The compilation pipeline is: CLI → File I/O → Lexer → Parser → AST Validation → TypeResolver → CycleDetector → IR → BinaryEncoder → CodeGenerator → File Writing
- All phases must complete successfully for compilation to succeed
- Error messages should be clear and include context (resource names, cycle paths, type mismatches, file paths, etc.)
- Always run `cargo test` to ensure all 69 tests pass
- The CLI in `main.rs` provides multiple modes: compile (reads/writes files), demo (in-memory examples), version, help
- File I/O functions are in lib.rs: `compile_file()`, `compile_file_to_output()`, `write_generated_code()`
- Binary encoding requires creating `Value` instances that match the IR types - type mismatches will error
- Code generation produces TypeScript client and Rust server code in `CompiledOutput.generated_code`
- Generated files are written to the output directory specified via --out flag (default: ./generated)
