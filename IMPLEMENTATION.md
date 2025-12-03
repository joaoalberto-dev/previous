# Previous Schema Language Compiler Implementation

## Overview

This is a complete implementation of the **Previous Schema Language (PSL)** compiler, a domain-specific language for defining Resources in the Previous BFF framework.

## Architecture

The compiler is structured in three main layers:

### 1. Lexer
Tokenizes the input schema string into a stream of tokens.
- Handles whitespace and comments
- Recognizes keywords, identifiers, literals, and symbols
- Single-pass scanning with position tracking

### 2. Parser
Recursive descent parser that builds an Abstract Syntax Tree (AST) following the BNF specification.
- Parses resources, fields, types, and attributes
- Validates PascalCase for resource names
- Proper error reporting with context

### 3. Compiler
Validates the AST and produces compiled output.
- Ensures uniqueness of resource names
- Ensures uniqueness of field names within each resource
- Preserves field ordering and indexes

## Supported Features

### Types
- **Primitives**: `string`, `number`, `bool`
- **Named**: References to other resources (PascalCase identifiers)
- **Generic**: `list <type>` for zero-or-more collections

### Field Attributes
- **`nullable`**: Field can contain null values
- **`optional`**: Field may be omitted
- **`default(value)`**: Provides a default value

### Syntax

```
resource User {
    string name
    string email
    optional number age
    bool active
}

resource Names {
    list string names
}

resource Users {
    list User users
}

resource Settings {
    nullable bool notifications
}

resource Config {
    default(10) number timeout
}
```

Note: Attributes come BEFORE the type in field declarations.

## API

### Public Functions

```rust
pub fn parse_schema(input: &str) -> Result<Program, String>
```
Parses a schema string and returns an AST or error.

```rust
pub fn compile_schema(input: &str) -> Result<CompiledOutput, String>
```
Parses and compiles a schema, validating all constraints.

### Public Types

- `Program`: Contains a list of resources
- `Resource`: Contains a name and list of fields
- `Field`: Contains name, type, attributes, and index
- `ASTType`: Represents a type (Primitive, Named, or List)

## Testing

The implementation includes 12 comprehensive tests covering:
- Simple resource parsing
- Multiple fields
- Optional and nullable fields
- Default values
- List types
- Named resource types
- Duplicate name detection
- Field indexing
- PascalCase validation
- Full compilation pipeline

Run tests with: `cargo test`

## Validation Rules

1. **Resource Names**: Must be PascalCase (start with uppercase letter)
2. **Identifiers**: Must start with letter or underscore, can contain alphanumerics and underscores
3. **Uniqueness**: Resource names must be unique within a program
4. **Field Uniqueness**: Field names must be unique within a resource
5. **Field Count**: Every field must have a name and type

## Known Limitations

- No support for imports (single file only)
- No support for cyclic dependency detection (marked as TODO)
- Comments are not supported (not in specification)
- Forward references to resources are allowed but not validated
