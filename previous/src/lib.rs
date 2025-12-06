/*
   1. Overview
        Previous Schema Language (PSL) is a domain-specific language used to define Resources, the primary units of data exchanged between the server and client in the Previous BFF framework.
        A schema file (*.pr) describes:
            - the data structures (Resources)
            - fields and their types
            - attributes of these fields (e.g., repeated)
            - documentation for generated code

        Schemas are input to the previousc compiler, which generates code for server and client implementations.

    2. Compiler
        Only one input file is allowed
        Imports are not supported
        The compiler should garantee uniqueness in Resource names
        The compiler should garantee uniqueness within Resource field names
        The output directory can be specified with `--out` param
        Cyclic dependencies are not supported

        Example
        ```
        $ previouscc schema.pr
        ```

    3. Lexical structure
        3.1. White space
            Whitespace (space, tab, newline) is ignored except where required to separate tokens.
        3.2. Comments
            There is no specification for comments.
        3.3. Identifiers
            Identifiers must start with letters. eg.: [a-zA-Z_]
            Identifiers are case-sensitive
            Numbers are allowed but not in the beggining of the identifier
            Symbols or special characters are not allowed
            Resource identifiers must follow PascalCase
        3.4. Keywords
            `resource`
            `string`
            `number`
            `bool`
            `nullable`
            `optional`
            `default`
            `list`
        3.5. File structure
            Each file could contain one or more resources

            Examples:
            ```
            resource User {
                string   name
                string   email
                optional number age
                bool     active
            }

            resource Names {
                list string name
            }

            resource Users {
                list User
            }

            resource Settings {
                nullable bool notifications
            }

            resource Notification {
                number default(10) interval
            }
            ```
        3.6. Fields
            Each field must contain at least a type and a identifier
            Each field should in its own line, there is no statement delimiter
            The fields within a Resource are surrounded by curly braces `{}`
            The field can receive more attributes that change its behaviour:
                - `nullable`: Make the field accept null values
                - `optional`: Make the field optional
                - `default(value)`: Create a default value for the field. The default value must be the same type as the field
                - `list`: The field support zero or more items of the given type
    4. Binary Encoding Model
        4.1. Field Ordering
            Fields are encoded in the order they appear in the Resource.
        4.2. Field Indexes
            Index = position starting at 0.

        Example:
        ```
        resource User {
            string name   // index 0
            number age    // index 1
            bool   active // index 2
        }
        ```

    5. BNF
        <program> ::= <resource_list>

        <resource_list> ::= <resource>
            |  <resource> <resource_list>

        ;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
        ;; RESOURCE DECLARATIONS
        ;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

        <resource> ::= "resource" <resource_identifier> "{" <field_list> "}"

        <field_list> ::= <field>
            |  <field> <field_list>

        <field> ::= <attributes> <type> <identifier>
            | <type> <identifier>

        ;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
        ;; ATTRIBUTES
        ;; order: zero or more attributes BEFORE the type
        ;; examples:
        ;;   optional number age
        ;;   nullable list User
        ;;   number default(3) count
        ;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

        <attributes> ::= <attribute>
            |  <attribute> <attributes>

        <attribute> ::= "nullable"
            | "optional"
            | <default_attr>

        <default_attr> ::= "default" "(" <literal> ")"

        ;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
        ;; TYPES
        ;;
        ;; base types (keywords):
        ;;   string, number, bool
        ;;
        ;; generic types:
        ;;   list <type>
        ;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

        <type> ::= "string"
            | "number"
            | "bool"
            | "list" <type>
            | <resource_identifier>

        ;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
        ;; IDENTIFIERS
        ;;
        ;; Regular identifiers: start with letter or underscore, then letters/digits/_.
        ;; Resource identifiers (PascalCase): capital letter, then alphanumeric/_.
        ;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

        <identifier> ::= <letter_or_underscore> <identifier_rest>

        <identifier_rest> ::= <letter_or_digit_or_underscore>
            | <letter_or_digit_or_underscore> <identifier_rest>

        <resource_identifier> ::= <capital_letter> <identifier_rest_optional>

        <identifier_rest_optional> ::= <identifier_rest>
            | ε

        <letter_or_underscore> ::= <letter> | "_"

        <letter_or_digit_or_underscore> ::= <letter> | <digit> | "_"

        <letter> ::= "a" | "b" | ... | "z"
            | "A" | "B" | ... | "Z"

        <capital_letter> ::= "A" | "B" | ... | "Z"

        <digit> ::= "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9"

        ;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
        ;; LITERALS
        ;;
        ;; Only needed for default(value)
        ;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

        <literal> ::= <number_literal>
            | <string_literal>
            | <bool_literal>

        <number_literal> ::= <digit>
            | <digit> <number_literal>

        <string_literal> ::= "\"" <string_characters> "\""

        <string_characters> ::= <string_character>
            | <string_character> <string_characters>

        <string_character> ::= any character except quote or newline

        <bool_literal> ::= "true" | "false"
*/

// ============================================================================
// AST TYPES
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum ASTType {
    Primitive(String),
    Named(String),
    List(Box<ASTType>),
}

#[derive(Debug, Clone)]
pub enum Literal {
    String(String),
    Number(i64),
    Bool(bool),
}

#[derive(Debug, Clone)]
pub struct DefaultValue {
    pub value: Literal,
}

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub field_type: ASTType,
    pub nullable: bool,
    pub optional: bool,
    pub default: Option<DefaultValue>,
    pub index: usize,
}

#[derive(Debug, Clone)]
pub struct Resource {
    pub name: String,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub resources: Vec<Resource>,
}

// ============================================================================
// IR TYPES (Intermediate Representation)
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum IRType {
    /// Primitive type: "string", "number", "bool"
    Primitive(String),
    /// Reference to a resource by index in IRProgram.resources
    ResourceRef(usize),
    /// List of zero or more items of the inner type
    List(Box<IRType>),
}

#[derive(Debug, Clone)]
pub struct IRField {
    pub name: String,
    pub field_type: IRType,
    pub nullable: bool,
    pub optional: bool,
    pub default: Option<DefaultValue>,
    pub index: usize,
}

#[derive(Debug, Clone)]
pub struct IRResource {
    pub name: String,
    pub fields: Vec<IRField>,
}

#[derive(Debug, Clone)]
pub struct IRProgram {
    pub resources: Vec<IRResource>,
}

impl IRProgram {
    /// Find a resource by name, returning its index
    pub fn get_resource_index(&self, name: &str) -> Option<usize> {
        self.resources.iter().position(|r| r.name == name)
    }

    /// Find a resource by name
    pub fn get_resource(&self, name: &str) -> Option<&IRResource> {
        self.resources.iter().find(|r| r.name == name)
    }
}

// ============================================================================
// BINARY ENCODING MODEL (Phase 3)
// ============================================================================
//
// Binary Encoding Specification:
// - string:    u32 length (little-endian) + UTF-8 bytes
// - number:    i64 (8 bytes, little-endian)
// - bool:      1 byte (0x00 = false, 0x01 = true)
// - list:      u32 count (little-endian) + each item encoded recursively
// - nullable:  1 byte (0x00 = null, 0x01 = present) + value if present
// - optional:  1 byte (0x00 = absent, 0x01 = present) + value if present
// - resource:  fields encoded in order (field index is implicit)
//

/// Runtime value representation for encoding
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    String(String),
    Number(i64),
    Bool(bool),
    List(Vec<Value>),
    Resource(Vec<FieldValue>),
    Null,
    Absent,
}

/// Field value with optional/nullable handling
#[derive(Debug, Clone, PartialEq)]
pub struct FieldValue {
    pub name: String,
    pub value: Value,
    pub is_optional: bool,
    pub is_nullable: bool,
}

/// Binary encoder for Previous values
pub struct BinaryEncoder {
    buffer: Vec<u8>,
}

impl BinaryEncoder {
    pub fn new() -> Self {
        BinaryEncoder { buffer: Vec::new() }
    }

    /// Get the encoded bytes
    pub fn finish(self) -> Vec<u8> {
        self.buffer
    }

    /// Encode a value based on its type
    pub fn encode_value(&mut self, value: &Value, ir_type: &IRType, ir_program: &IRProgram) -> Result<(), String> {
        match (value, ir_type) {
            (Value::String(s), IRType::Primitive(p)) if p == "string" => {
                self.encode_string(s);
                Ok(())
            }
            (Value::Number(n), IRType::Primitive(p)) if p == "number" => {
                self.encode_number(*n);
                Ok(())
            }
            (Value::Bool(b), IRType::Primitive(p)) if p == "bool" => {
                self.encode_bool(*b);
                Ok(())
            }
            (Value::List(items), IRType::List(inner_type)) => {
                self.encode_list(items, inner_type, ir_program)
            }
            (Value::Resource(fields), IRType::ResourceRef(idx)) => {
                self.encode_resource(fields, *idx, ir_program)
            }
            (Value::Null, _) => {
                // Null should be handled by nullable wrapper
                Err("Cannot encode null value without nullable wrapper".to_string())
            }
            (Value::Absent, _) => {
                // Absent should be handled by optional wrapper
                Err("Cannot encode absent value without optional wrapper".to_string())
            }
            _ => Err(format!("Type mismatch: value {:?} does not match type {:?}", value, ir_type)),
        }
    }

    /// Encode a field with optional/nullable handling
    pub fn encode_field(&mut self, field_value: &FieldValue, ir_field: &IRField, ir_program: &IRProgram) -> Result<(), String> {
        // Handle optional fields
        if ir_field.optional {
            match &field_value.value {
                Value::Absent => {
                    self.buffer.push(0x00); // absent
                    return Ok(());
                }
                _ => {
                    self.buffer.push(0x01); // present
                }
            }
        }

        // Handle nullable fields
        if ir_field.nullable {
            match &field_value.value {
                Value::Null => {
                    self.buffer.push(0x00); // null
                    return Ok(());
                }
                _ => {
                    self.buffer.push(0x01); // not null
                }
            }
        }

        // Encode the actual value
        self.encode_value(&field_value.value, &ir_field.field_type, ir_program)
    }

    // Primitive encoders

    fn encode_string(&mut self, s: &str) {
        let bytes = s.as_bytes();
        let len = bytes.len() as u32;
        self.buffer.extend_from_slice(&len.to_le_bytes());
        self.buffer.extend_from_slice(bytes);
    }

    fn encode_number(&mut self, n: i64) {
        self.buffer.extend_from_slice(&n.to_le_bytes());
    }

    fn encode_bool(&mut self, b: bool) {
        self.buffer.push(if b { 0x01 } else { 0x00 });
    }

    fn encode_list(&mut self, items: &[Value], inner_type: &IRType, ir_program: &IRProgram) -> Result<(), String> {
        let count = items.len() as u32;
        self.buffer.extend_from_slice(&count.to_le_bytes());

        for item in items {
            self.encode_value(item, inner_type, ir_program)?;
        }

        Ok(())
    }

    fn encode_resource(&mut self, fields: &[FieldValue], resource_idx: usize, ir_program: &IRProgram) -> Result<(), String> {
        let ir_resource = &ir_program.resources.get(resource_idx)
            .ok_or_else(|| format!("Invalid resource index: {}", resource_idx))?;

        // Encode fields in order
        if fields.len() != ir_resource.fields.len() {
            return Err(format!(
                "Field count mismatch: expected {} fields for resource '{}', got {}",
                ir_resource.fields.len(),
                ir_resource.name,
                fields.len()
            ));
        }

        for (field_value, ir_field) in fields.iter().zip(ir_resource.fields.iter()) {
            self.encode_field(field_value, ir_field, ir_program)?;
        }

        Ok(())
    }
}

// ============================================================================
// CODE GENERATION (Phase 4)
// ============================================================================

/// Generated code output containing client and server code
#[derive(Debug, Clone)]
pub struct GeneratedCode {
    pub typescript_client: String,
    pub rust_server: String,
}

/// Code generator for TypeScript client and Rust server
pub struct CodeGenerator {
    ir: IRProgram,
}

impl CodeGenerator {
    pub fn new(ir: IRProgram) -> Self {
        CodeGenerator { ir }
    }

    /// Generate both client and server code
    pub fn generate(&self) -> GeneratedCode {
        GeneratedCode {
            typescript_client: self.generate_typescript_client(),
            rust_server: self.generate_rust_server(),
        }
    }

    // ========================================================================
    // TypeScript Client Generation
    // ========================================================================

    fn generate_typescript_client(&self) -> String {
        let mut code = String::new();

        // Header
        code.push_str("// Generated by Previous Compiler\n");
        code.push_str("// DO NOT EDIT - This file is auto-generated\n\n");

        // Binary reader utility class
        code.push_str(&self.generate_binary_reader());
        code.push_str("\n");

        // Generate each resource
        for resource in &self.ir.resources {
            code.push_str(&self.generate_ts_resource(resource));
            code.push_str("\n");
        }

        code
    }

    fn generate_binary_reader(&self) -> String {
        r#"class BinaryReader {
  private buffer: Uint8Array;
  private offset: number;

  constructor(buffer: Uint8Array) {
    this.buffer = buffer;
    this.offset = 0;
  }

  readString(): string {
    const length = this.readU32();
    const bytes = this.buffer.slice(this.offset, this.offset + length);
    this.offset += length;
    return new TextDecoder().decode(bytes);
  }

  readNumber(): number {
    const view = new DataView(this.buffer.buffer, this.offset, 8);
    const value = view.getBigInt64(0, true); // little-endian
    this.offset += 8;
    return Number(value);
  }

  readBool(): boolean {
    const value = this.buffer[this.offset];
    this.offset += 1;
    return value === 1;
  }

  readU32(): number {
    const view = new DataView(this.buffer.buffer, this.offset, 4);
    const value = view.getUint32(0, true); // little-endian
    this.offset += 4;
    return value;
  }

  readByte(): number {
    const value = this.buffer[this.offset];
    this.offset += 1;
    return value;
  }
}
"#.to_string()
    }

    fn generate_ts_resource(&self, resource: &IRResource) -> String {
        let mut code = String::new();

        // Interface for the resource
        code.push_str(&format!("export interface I{} {{\n", resource.name));
        for field in &resource.fields {
            let ts_type = self.ir_type_to_typescript(&field.field_type);
            let optional = if field.optional || field.nullable { "?" } else { "" };
            code.push_str(&format!("  {}{}: {};\n", field.name, optional, ts_type));
        }
        code.push_str("}\n\n");

        // Decoder class
        code.push_str(&format!("export class {} {{\n", resource.name));
        code.push_str("  private reader: BinaryReader;\n");
        code.push_str(&format!("  private data: I{};\n\n", resource.name));

        // Constructor
        code.push_str("  constructor(buffer: Uint8Array) {\n");
        code.push_str("    this.reader = new BinaryReader(buffer);\n");
        code.push_str(&format!("    this.data = {{}} as I{};\n", resource.name));
        code.push_str("    this.decode();\n");
        code.push_str("  }\n\n");

        // Decode method
        code.push_str("  private decode(): void {\n");
        for field in &resource.fields {
            code.push_str(&self.generate_ts_field_decode(field));
        }
        code.push_str("  }\n\n");

        // Getter methods
        for field in &resource.fields {
            let ts_type = self.ir_type_to_typescript(&field.field_type);
            let optional = if field.optional || field.nullable { " | null | undefined" } else { "" };
            code.push_str(&format!(
                "  get{}(): {}{} {{\n",
                self.capitalize_first(&field.name),
                ts_type,
                optional
            ));
            code.push_str(&format!("    return this.data.{};\n", field.name));
            code.push_str("  }\n\n");
        }

        // toJSON method
        code.push_str(&format!("  toJSON(): I{} {{\n", resource.name));
        code.push_str("    return this.data;\n");
        code.push_str("  }\n");

        code.push_str("}\n");
        code
    }

    fn generate_ts_field_decode(&self, field: &IRField) -> String {
        let mut code = String::new();

        // Handle optional
        if field.optional {
            code.push_str("    const isPresent = this.reader.readByte();\n");
            code.push_str("    if (isPresent === 0) {\n");
            code.push_str(&format!("      this.data.{} = undefined;\n", field.name));
            code.push_str("    } else {\n");
            code.push_str(&format!("      this.data.{} = {};\n",
                field.name,
                self.generate_ts_type_read(&field.field_type, "      ")));
            code.push_str("    }\n");
            return code;
        }

        // Handle nullable
        if field.nullable {
            code.push_str("    const isNull = this.reader.readByte();\n");
            code.push_str("    if (isNull === 0) {\n");
            code.push_str(&format!("      this.data.{} = null;\n", field.name));
            code.push_str("    } else {\n");
            code.push_str(&format!("      this.data.{} = {};\n",
                field.name,
                self.generate_ts_type_read(&field.field_type, "      ")));
            code.push_str("    }\n");
            return code;
        }

        // Regular field
        code.push_str(&format!("    this.data.{} = {};\n",
            field.name,
            self.generate_ts_type_read(&field.field_type, "    ")));
        code
    }

    fn generate_ts_type_read(&self, ir_type: &IRType, indent: &str) -> String {
        match ir_type {
            IRType::Primitive(p) => match p.as_str() {
                "string" => "this.reader.readString()".to_string(),
                "number" => "this.reader.readNumber()".to_string(),
                "bool" => "this.reader.readBool()".to_string(),
                _ => "null".to_string(),
            },
            IRType::List(inner) => {
                let inner_read = self.generate_ts_type_read(inner, indent);
                format!(
                    "(() => {{\n{}  const count = this.reader.readU32();\n{}  const items = [];\n{}  for (let i = 0; i < count; i++) {{\n{}    items.push({});\n{}  }}\n{}  return items;\n{}}})()",
                    indent, indent, indent, indent, inner_read, indent, indent, indent
                )
            }
            IRType::ResourceRef(idx) => {
                let resource = &self.ir.resources[*idx];
                format!("new {}(this.reader.buffer.slice(this.reader.offset))", resource.name)
            }
        }
    }

    fn ir_type_to_typescript(&self, ir_type: &IRType) -> String {
        match ir_type {
            IRType::Primitive(p) => match p.as_str() {
                "string" => "string".to_string(),
                "number" => "number".to_string(),
                "bool" => "boolean".to_string(),
                _ => "any".to_string(),
            },
            IRType::List(inner) => format!("{}[]", self.ir_type_to_typescript(inner)),
            IRType::ResourceRef(idx) => format!("I{}", self.ir.resources[*idx].name),
        }
    }

    // ========================================================================
    // Rust Server Generation
    // ========================================================================

    fn generate_rust_server(&self) -> String {
        let mut code = String::new();

        // Header
        code.push_str("// Generated by Previous Compiler\n");
        code.push_str("// DO NOT EDIT - This file is auto-generated\n\n");
        code.push_str("use previous::{Value, FieldValue, BinaryEncoder, IRType, IRProgram};\n\n");

        // Generate each resource
        for (idx, resource) in self.ir.resources.iter().enumerate() {
            code.push_str(&self.generate_rust_resource(resource, idx));
            code.push_str("\n");
        }

        code
    }

    fn generate_rust_resource(&self, resource: &IRResource, _idx: usize) -> String {
        let mut code = String::new();

        // Struct definition
        code.push_str(&format!("#[derive(Debug, Clone)]\n"));
        code.push_str(&format!("pub struct {} {{\n", resource.name));
        for field in &resource.fields {
            let rust_type = self.ir_type_to_rust(&field.field_type);
            let wrapped_type = if field.optional {
                format!("Option<{}>", rust_type)
            } else if field.nullable {
                format!("Option<{}>", rust_type)
            } else {
                rust_type
            };
            code.push_str(&format!("    pub {}: {},\n", field.name, wrapped_type));
        }
        code.push_str("}\n\n");

        // Implementation
        code.push_str(&format!("impl {} {{\n", resource.name));

        // Constructor
        code.push_str("    pub fn new() -> Self {\n");
        code.push_str(&format!("        {} {{\n", resource.name));
        for field in &resource.fields {
            let default = if field.optional || field.nullable {
                "None".to_string()
            } else {
                self.rust_default_value(&field.field_type)
            };
            code.push_str(&format!("            {}: {},\n", field.name, default));
        }
        code.push_str("        }\n");
        code.push_str("    }\n\n");

        // Setter methods (builder pattern)
        for field in &resource.fields {
            let rust_type = self.ir_type_to_rust(&field.field_type);
            let param_type = if field.optional || field.nullable {
                format!("Option<{}>", rust_type)
            } else {
                rust_type.clone()
            };

            code.push_str(&format!("    pub fn {}(mut self, value: {}) -> Self {{\n", field.name, param_type));
            code.push_str(&format!("        self.{} = value;\n", field.name));
            code.push_str("        self\n");
            code.push_str("    }\n\n");
        }

        // Encode method
        code.push_str("    pub fn encode(&self, ir_program: &IRProgram) -> Result<Vec<u8>, String> {\n");
        code.push_str("        let value = self.to_value();\n");
        code.push_str("        let mut encoder = BinaryEncoder::new();\n");
        code.push_str(&format!("        let resource_idx = ir_program.get_resource_index(\"{}\").unwrap();\n", resource.name));
        code.push_str("        encoder.encode_value(&value, &IRType::ResourceRef(resource_idx), ir_program)?;\n");
        code.push_str("        Ok(encoder.finish())\n");
        code.push_str("    }\n\n");

        // to_value method
        code.push_str("    fn to_value(&self) -> Value {\n");
        code.push_str("        Value::Resource(vec![\n");
        for field in &resource.fields {
            code.push_str(&format!("            FieldValue {{\n"));
            code.push_str(&format!("                name: \"{}\".to_string(),\n", field.name));
            code.push_str(&format!("                value: {},\n", self.generate_rust_value_conversion(field)));
            code.push_str(&format!("                is_optional: {},\n", field.optional));
            code.push_str(&format!("                is_nullable: {},\n", field.nullable));
            code.push_str("            },\n");
        }
        code.push_str("        ])\n");
        code.push_str("    }\n");

        code.push_str("}\n");
        code
    }

    fn generate_rust_value_conversion(&self, field: &IRField) -> String {
        let conversion = match &field.field_type {
            IRType::Primitive(p) => match p.as_str() {
                "string" => format!("Value::String(self.{}.clone())", field.name),
                "number" => format!("Value::Number(self.{})", field.name),
                "bool" => format!("Value::Bool(self.{})", field.name),
                _ => "Value::Null".to_string(),
            },
            IRType::List(inner) => {
                let inner_conv = self.generate_list_item_conversion(inner);
                format!("Value::List(self.{}.iter().map(|item| {}).collect())", field.name, inner_conv)
            }
            IRType::ResourceRef(_) => {
                format!("self.{}.to_value()", field.name)
            }
        };

        if field.optional {
            format!("self.{}.as_ref().map(|v| {}).unwrap_or(Value::Absent)", field.name, conversion.replace(&format!("self.{}", field.name), "v"))
        } else if field.nullable {
            format!("self.{}.as_ref().map(|v| {}).unwrap_or(Value::Null)", field.name, conversion.replace(&format!("self.{}", field.name), "v"))
        } else {
            conversion
        }
    }

    fn generate_list_item_conversion(&self, ir_type: &IRType) -> String {
        match ir_type {
            IRType::Primitive(p) => match p.as_str() {
                "string" => "Value::String(item.clone())".to_string(),
                "number" => "Value::Number(*item)".to_string(),
                "bool" => "Value::Bool(*item)".to_string(),
                _ => "Value::Null".to_string(),
            },
            IRType::List(_) => "item.clone()".to_string(),
            IRType::ResourceRef(_) => "item.to_value()".to_string(),
        }
    }

    fn ir_type_to_rust(&self, ir_type: &IRType) -> String {
        match ir_type {
            IRType::Primitive(p) => match p.as_str() {
                "string" => "String".to_string(),
                "number" => "i64".to_string(),
                "bool" => "bool".to_string(),
                _ => "()".to_string(),
            },
            IRType::List(inner) => format!("Vec<{}>", self.ir_type_to_rust(inner)),
            IRType::ResourceRef(idx) => self.ir.resources[*idx].name.clone(),
        }
    }

    fn rust_default_value(&self, ir_type: &IRType) -> String {
        match ir_type {
            IRType::Primitive(p) => match p.as_str() {
                "string" => "String::new()".to_string(),
                "number" => "0".to_string(),
                "bool" => "false".to_string(),
                _ => "()".to_string(),
            },
            IRType::List(_) => "Vec::new()".to_string(),
            IRType::ResourceRef(idx) => format!("{}::new()", self.ir.resources[*idx].name),
        }
    }

    fn capitalize_first(&self, s: &str) -> String {
        let mut chars = s.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        }
    }
}

// ============================================================================
// TOKEN TYPES
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Resource,
    String,
    Number,
    Bool,
    Nullable,
    Optional,
    Default,
    List,
    True,
    False,

    // Identifiers and literals
    Identifier(String),
    StringLiteral(String),
    NumberLiteral(i64),

    // Symbols
    LeftBrace,
    RightBrace,
    LeftParen,
    RightParen,

    // Special
    Eof,
}

// ============================================================================
// LEXER
// ============================================================================

#[derive(Debug)]
pub struct Lexer {
    input: Vec<char>,
    position: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            position: 0,
        }
    }

    fn current_char(&self) -> Option<char> {
        if self.position < self.input.len() {
            Some(self.input[self.position])
        } else {
            None
        }
    }

    #[allow(dead_code)]
    fn peek_char(&self, offset: usize) -> Option<char> {
        let pos = self.position + offset;
        if pos < self.input.len() {
            Some(self.input[pos])
        } else {
            None
        }
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.current_char();
        if ch.is_some() {
            self.position += 1;
        }
        ch
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn read_identifier(&mut self) -> String {
        let mut ident = String::new();
        while let Some(ch) = self.current_char() {
            if ch.is_alphanumeric() || ch == '_' {
                ident.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        ident
    }

    fn read_string(&mut self) -> String {
        let mut string = String::new();
        self.advance(); // skip opening quote
        while let Some(ch) = self.current_char() {
            if ch == '"' {
                self.advance();
                break;
            }
            string.push(ch);
            self.advance();
        }
        string
    }

    fn read_number(&mut self) -> i64 {
        let mut num_str = String::new();
        while let Some(ch) = self.current_char() {
            if ch.is_ascii_digit() {
                num_str.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        num_str.parse().unwrap_or(0)
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        match self.current_char() {
            None => Token::Eof,
            Some('{') => {
                self.advance();
                Token::LeftBrace
            }
            Some('}') => {
                self.advance();
                Token::RightBrace
            }
            Some('(') => {
                self.advance();
                Token::LeftParen
            }
            Some(')') => {
                self.advance();
                Token::RightParen
            }
            Some('"') => {
                let string = self.read_string();
                Token::StringLiteral(string)
            }
            Some(ch) if ch.is_ascii_digit() => {
                let num = self.read_number();
                Token::NumberLiteral(num)
            }
            Some(ch) if ch.is_alphabetic() || ch == '_' => {
                let ident = self.read_identifier();
                match ident.as_str() {
                    "resource" => Token::Resource,
                    "string" => Token::String,
                    "number" => Token::Number,
                    "bool" => Token::Bool,
                    "nullable" => Token::Nullable,
                    "optional" => Token::Optional,
                    "default" => Token::Default,
                    "list" => Token::List,
                    "true" => Token::True,
                    "false" => Token::False,
                    _ => Token::Identifier(ident),
                }
            }
            Some(_) => {
                self.advance();
                self.next_token()
            }
        }
    }
}

// ============================================================================
// PARSER
// ============================================================================

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new(input: &str) -> Self {
        let mut lexer = Lexer::new(input);
        let mut tokens = Vec::new();

        loop {
            let token = lexer.next_token();
            if token == Token::Eof {
                tokens.push(token);
                break;
            }
            tokens.push(token);
        }

        Parser {
            tokens,
            position: 0,
        }
    }

    fn current_token(&self) -> &Token {
        self.tokens.get(self.position).unwrap_or(&Token::Eof)
    }

    #[allow(dead_code)]
    fn peek_token(&self, offset: usize) -> &Token {
        self.tokens
            .get(self.position + offset)
            .unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) -> Token {
        let token = self.current_token().clone();
        if self.position < self.tokens.len() {
            self.position += 1;
        }
        token
    }

    fn expect(&mut self, expected: Token) -> Result<(), String> {
        let current = self.current_token();
        let matches = match (&expected, current) {
            (Token::Resource, Token::Resource) => true,
            (Token::LeftBrace, Token::LeftBrace) => true,
            (Token::RightBrace, Token::RightBrace) => true,
            (Token::LeftParen, Token::LeftParen) => true,
            (Token::RightParen, Token::RightParen) => true,
            _ => std::mem::discriminant(&expected) == std::mem::discriminant(current),
        };
        if matches {
            self.advance();
            Ok(())
        } else {
            Err(format!("Expected {:?}, got {:?}", expected, current))
        }
    }

    pub fn parse(&mut self) -> Result<Program, String> {
        let mut resources = Vec::new();

        while self.current_token() != &Token::Eof {
            let resource = self.parse_resource()?;
            resources.push(resource);
        }

        Ok(Program { resources })
    }

    fn parse_resource(&mut self) -> Result<Resource, String> {
        self.expect(Token::Resource)?;

        let name = match self.advance() {
            Token::Identifier(id) => id,
            _ => return Err("Expected resource name".to_string()),
        };

        // Validate PascalCase
        if !name.chars().next().unwrap().is_uppercase() {
            return Err(format!("Resource name must be PascalCase: {}", name));
        }

        self.expect(Token::LeftBrace)?;

        let mut fields = Vec::new();
        let mut index = 0;

        while self.current_token() != &Token::RightBrace && self.current_token() != &Token::Eof {
            let field = self.parse_field(index)?;
            fields.push(field);
            index += 1;
        }

        self.expect(Token::RightBrace)?;

        Ok(Resource { name, fields })
    }

    fn parse_field(&mut self, index: usize) -> Result<Field, String> {
        let mut nullable = false;
        let mut optional = false;
        let mut default = None;

        // Parse attributes
        loop {
            match self.current_token() {
                Token::Nullable => {
                    nullable = true;
                    self.advance();
                }
                Token::Optional => {
                    optional = true;
                    self.advance();
                }
                Token::Default => {
                    self.advance();
                    self.expect(Token::LeftParen)?;
                    let literal = self.parse_literal()?;
                    self.expect(Token::RightParen)?;
                    default = Some(DefaultValue { value: literal });
                }
                _ => break,
            }
        }

        // Parse type
        let field_type = self.parse_type()?;

        // Parse identifier
        let name = match self.advance() {
            Token::Identifier(id) => id,
            _ => return Err("Expected field name".to_string()),
        };

        Ok(Field {
            name,
            field_type,
            nullable,
            optional,
            default,
            index,
        })
    }

    fn parse_type(&mut self) -> Result<ASTType, String> {
        match self.current_token() {
            Token::String => {
                self.advance();
                Ok(ASTType::Primitive("string".to_string()))
            }
            Token::Number => {
                self.advance();
                Ok(ASTType::Primitive("number".to_string()))
            }
            Token::Bool => {
                self.advance();
                Ok(ASTType::Primitive("bool".to_string()))
            }
            Token::List => {
                self.advance();
                let inner_type = self.parse_type()?;
                Ok(ASTType::List(Box::new(inner_type)))
            }
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();
                Ok(ASTType::Named(name))
            }
            _ => Err(format!("Expected type, got {:?}", self.current_token())),
        }
    }

    fn parse_literal(&mut self) -> Result<Literal, String> {
        match self.current_token() {
            Token::StringLiteral(s) => {
                let s = s.clone();
                self.advance();
                Ok(Literal::String(s))
            }
            Token::NumberLiteral(n) => {
                let n = *n;
                self.advance();
                Ok(Literal::Number(n))
            }
            Token::True => {
                self.advance();
                Ok(Literal::Bool(true))
            }
            Token::False => {
                self.advance();
                Ok(Literal::Bool(false))
            }
            _ => Err(format!("Expected literal, got {:?}", self.current_token())),
        }
    }
}

// ============================================================================
// COMPILER
// ============================================================================

#[allow(dead_code)]
pub struct Compiler {
    program: Program,
}

impl Compiler {
    pub fn new(program: Program) -> Result<Self, String> {
        // Validate uniqueness of resource names
        let mut resource_names = std::collections::HashSet::new();
        for resource in &program.resources {
            if !resource_names.insert(resource.name.clone()) {
                return Err(format!("Duplicate resource name: {}", resource.name));
            }
        }

        // Validate uniqueness of field names within each resource
        for resource in &program.resources {
            let mut field_names = std::collections::HashSet::new();
            for field in &resource.fields {
                if !field_names.insert(field.name.clone()) {
                    return Err(format!(
                        "Duplicate field name in {}: {}",
                        resource.name, field.name
                    ));
                }
            }
        }

        Ok(Compiler { program })
    }

    pub fn compile(&self) -> Result<CompiledOutput, String> {
        // 1. Validate AST (already done in new())

        // 2. Type resolution
        let resolver = TypeResolver::new(&self.program)?;
        let ir = resolver.resolve(self.program.clone())?;

        // 3. Cycle detection
        let cycle_detector = CycleDetector::build(&ir)?;
        cycle_detector.detect()?;

        // 4. Code generation
        let code_generator = CodeGenerator::new(ir.clone());
        let generated_code = code_generator.generate();

        // 5. Return compiled output with IR and generated code
        Ok(CompiledOutput {
            ir,
            generated_code,
        })
    }
}

// ============================================================================
// TYPE RESOLVER
// ============================================================================

pub struct TypeResolver {
    resource_map: std::collections::HashMap<String, usize>,
}

impl TypeResolver {
    /// Build a type resolver from an AST program
    ///
    /// Creates a mapping of resource names to their indices for fast lookup
    /// during type resolution.
    pub fn new(program: &Program) -> Result<Self, String> {
        let mut resource_map = std::collections::HashMap::new();

        for (index, resource) in program.resources.iter().enumerate() {
            if resource_map.insert(resource.name.clone(), index).is_some() {
                // This shouldn't happen because Compiler::new validates uniqueness
                return Err(format!("Duplicate resource name: {}", resource.name));
            }
        }

        Ok(TypeResolver { resource_map })
    }

    /// Resolve a single AST type to an IR type
    ///
    /// Converts:
    /// - ASTType::Primitive(s) → IRType::Primitive(s)
    /// - ASTType::Named(s) → IRType::ResourceRef(index) or error
    /// - ASTType::List(inner) → IRType::List(resolved_inner)
    fn resolve_type(&self, ast_type: &ASTType) -> Result<IRType, String> {
        match ast_type {
            ASTType::Primitive(name) => {
                // Validate it's one of the three primitives
                match name.as_str() {
                    "string" | "number" | "bool" => Ok(IRType::Primitive(name.clone())),
                    _ => Err(format!("Invalid primitive type: {}", name)),
                }
            }
            ASTType::Named(name) => {
                // Look up the resource name
                match self.resource_map.get(name) {
                    Some(&index) => Ok(IRType::ResourceRef(index)),
                    None => Err(format!("Undefined type: {}", name)),
                }
            }
            ASTType::List(inner) => {
                // Recursively resolve the inner type
                let resolved_inner = self.resolve_type(inner)?;
                Ok(IRType::List(Box::new(resolved_inner)))
            }
        }
    }

    /// Transform an entire AST program to an IR program
    ///
    /// Converts all field types from AST to IR, preserving all field attributes.
    pub fn resolve(&self, program: Program) -> Result<IRProgram, String> {
        let mut ir_resources = Vec::new();

        for ast_resource in program.resources {
            let mut ir_fields = Vec::new();

            for ast_field in ast_resource.fields {
                let resolved_type = self.resolve_type(&ast_field.field_type)?;
                ir_fields.push(IRField {
                    name: ast_field.name,
                    field_type: resolved_type,
                    nullable: ast_field.nullable,
                    optional: ast_field.optional,
                    default: ast_field.default,
                    index: ast_field.index,
                });
            }

            ir_resources.push(IRResource {
                name: ast_resource.name,
                fields: ir_fields,
            });
        }

        Ok(IRProgram {
            resources: ir_resources,
        })
    }
}

// ============================================================================
// CYCLE DETECTOR
// ============================================================================

pub struct CycleDetector {
    graph: Vec<Vec<usize>>,
    resource_names: Vec<String>,
}

impl CycleDetector {
    /// Build a dependency graph from the IR program
    ///
    /// Creates an adjacency list where each node represents a resource
    /// and edges represent references to other resources.
    pub fn build(ir: &IRProgram) -> Result<Self, String> {
        let mut graph = vec![Vec::new(); ir.resources.len()];

        // For each resource and its fields, collect all resource references
        for (res_idx, resource) in ir.resources.iter().enumerate() {
            for field in &resource.fields {
                Self::collect_refs(res_idx, &field.field_type, &mut graph);
            }
        }

        // Extract resource names for error reporting
        let resource_names: Vec<String> = ir.resources.iter().map(|r| r.name.clone()).collect();

        Ok(CycleDetector {
            graph,
            resource_names,
        })
    }

    /// Helper: extract all resource references from a type recursively
    ///
    /// - Primitive types: no references
    /// - ResourceRef: add edge from current resource to referenced resource
    /// - List: recursively process inner type
    fn collect_refs(from_idx: usize, ir_type: &IRType, graph: &mut Vec<Vec<usize>>) {
        match ir_type {
            IRType::Primitive(_) => {
                // No resource references in primitive types
            }
            IRType::ResourceRef(to_idx) => {
                // Add edge: from_idx → to_idx
                graph[from_idx].push(*to_idx);
            }
            IRType::List(inner) => {
                // Recursively process list inner type
                Self::collect_refs(from_idx, inner, graph);
            }
        }
    }

    /// Detect cycles in the resource dependency graph
    ///
    /// Uses depth-first search with recursion stack tracking.
    /// If a node is encountered that's already in the current recursion stack,
    /// a cycle has been found.
    pub fn detect(&self) -> Result<(), String> {
        let n = self.graph.len();
        let mut visited = vec![false; n];
        let mut rec_stack = vec![false; n];
        let mut path = Vec::new();

        for i in 0..n {
            if !visited[i] {
                self.dfs(i, &mut visited, &mut rec_stack, &mut path)?;
            }
        }

        Ok(())
    }

    /// Depth-first search for cycle detection
    ///
    /// Maintains:
    /// - visited: tracks nodes we've processed
    /// - rec_stack: tracks nodes in the current path (to detect back edges)
    /// - path: tracks the current traversal path for error messages
    fn dfs(
        &self,
        node: usize,
        visited: &mut Vec<bool>,
        rec_stack: &mut Vec<bool>,
        path: &mut Vec<usize>,
    ) -> Result<(), String> {
        // Mark as visited and in current recursion path
        visited[node] = true;
        rec_stack[node] = true;
        path.push(node);

        // Visit all neighbors
        for &neighbor in &self.graph[node] {
            if !visited[neighbor] {
                // Unvisited neighbor: recurse
                self.dfs(neighbor, visited, rec_stack, path)?;
            } else if rec_stack[neighbor] {
                // Neighbor is in current path: found a cycle!
                // Extract the cycle from the path
                let cycle_start = path.iter().position(|&n| n == neighbor).unwrap();
                let cycle_path = &path[cycle_start..];

                // Convert node indices to names
                let cycle_names: Vec<String> = cycle_path
                    .iter()
                    .map(|&idx| self.resource_names[idx].clone())
                    .collect();

                // Format error message: A → B → C → A
                let mut msg = cycle_names.join(" → ");
                msg.push_str(" → ");
                msg.push_str(&self.resource_names[neighbor]);

                return Err(format!("Cyclic dependency detected: {}", msg));
            }
        }

        // Backtrack: remove from current path
        path.pop();
        rec_stack[node] = false;
        Ok(())
    }
}

#[derive(Debug)]
pub struct CompiledOutput {
    pub ir: IRProgram,
    pub generated_code: GeneratedCode,
}

impl CompiledOutput {
    pub fn new() -> Self {
        CompiledOutput {
            ir: IRProgram {
                resources: Vec::new(),
            },
            generated_code: GeneratedCode {
                typescript_client: String::new(),
                rust_server: String::new(),
            },
        }
    }
}

// ============================================================================
// PUBLIC API
// ============================================================================

pub fn run() {
    println!("Previous Compiler v0.1.0");
}

pub fn parse_schema(input: &str) -> Result<Program, String> {
    let mut parser = Parser::new(input);
    parser.parse()
}

pub fn compile_schema(input: &str) -> Result<CompiledOutput, String> {
    let program = parse_schema(input)?;
    let compiler = Compiler::new(program)?;
    compiler.compile()
}

// ============================================================================
// CLI & FILE I/O (Phase 5)
// ============================================================================

use std::fs;
use std::path::{Path, PathBuf};

/// CLI options for the Previous compiler
#[derive(Debug, Clone)]
pub struct CliOptions {
    pub input_file: PathBuf,
    pub output_dir: PathBuf,
    pub verbose: bool,
}

impl Default for CliOptions {
    fn default() -> Self {
        CliOptions {
            input_file: PathBuf::from("schema.pr"),
            output_dir: PathBuf::from("./generated"),
            verbose: false,
        }
    }
}

/// Compile a schema file and write generated code to files
pub fn compile_file(options: &CliOptions) -> Result<(), String> {
    // Read the input file
    let schema_content = fs::read_to_string(&options.input_file)
        .map_err(|e| format!("Failed to read input file '{}': {}", options.input_file.display(), e))?;

    if options.verbose {
        eprintln!("Reading schema from: {}", options.input_file.display());
    }

    // Compile the schema
    let output = compile_schema(&schema_content)?;

    if options.verbose {
        eprintln!("Compilation successful!");
        eprintln!("  Resources: {}", output.ir.resources.len());
        eprintln!("  TypeScript lines: {}", output.generated_code.typescript_client.lines().count());
        eprintln!("  Rust lines: {}", output.generated_code.rust_server.lines().count());
    }

    // Create output directory if it doesn't exist
    fs::create_dir_all(&options.output_dir)
        .map_err(|e| format!("Failed to create output directory '{}': {}", options.output_dir.display(), e))?;

    // Write TypeScript client
    let ts_path = options.output_dir.join("client.ts");
    fs::write(&ts_path, &output.generated_code.typescript_client)
        .map_err(|e| format!("Failed to write TypeScript file '{}': {}", ts_path.display(), e))?;

    if options.verbose {
        eprintln!("  Generated: {}", ts_path.display());
    }

    // Write Rust server
    let rust_path = options.output_dir.join("server.rs");
    fs::write(&rust_path, &output.generated_code.rust_server)
        .map_err(|e| format!("Failed to write Rust file '{}': {}", rust_path.display(), e))?;

    if options.verbose {
        eprintln!("  Generated: {}", rust_path.display());
    }

    Ok(())
}

/// Compile a schema file and return the output (for testing/library use)
pub fn compile_file_to_output(input_path: &Path) -> Result<CompiledOutput, String> {
    let schema_content = fs::read_to_string(input_path)
        .map_err(|e| format!("Failed to read input file '{}': {}", input_path.display(), e))?;

    compile_schema(&schema_content)
}

/// Write generated code to files
pub fn write_generated_code(
    generated_code: &GeneratedCode,
    output_dir: &Path,
) -> Result<(), String> {
    // Create output directory
    fs::create_dir_all(output_dir)
        .map_err(|e| format!("Failed to create output directory '{}': {}", output_dir.display(), e))?;

    // Write TypeScript
    let ts_path = output_dir.join("client.ts");
    fs::write(&ts_path, &generated_code.typescript_client)
        .map_err(|e| format!("Failed to write TypeScript file: {}", e))?;

    // Write Rust
    let rust_path = output_dir.join("server.rs");
    fs::write(&rust_path, &generated_code.rust_server)
        .map_err(|e| format!("Failed to write Rust file: {}", e))?;

    Ok(())
}

/// Enhanced error type with file location context
#[derive(Debug, Clone)]
pub struct CompileError {
    pub message: String,
    pub file: Option<PathBuf>,
    pub line: Option<usize>,
    pub column: Option<usize>,
}

impl CompileError {
    pub fn new(message: String) -> Self {
        CompileError {
            message,
            file: None,
            line: None,
            column: None,
        }
    }

    pub fn with_file(mut self, file: PathBuf) -> Self {
        self.file = Some(file);
        self
    }

    pub fn with_location(mut self, line: usize, column: usize) -> Self {
        self.line = Some(line);
        self.column = Some(column);
        self
    }

    pub fn format(&self) -> String {
        let mut msg = String::new();

        if let Some(file) = &self.file {
            msg.push_str(&format!("Error in {}", file.display()));
            if let (Some(line), Some(col)) = (self.line, self.column) {
                msg.push_str(&format!(" at line {}, column {}", line, col));
            } else if let Some(line) = self.line {
                msg.push_str(&format!(" at line {}", line));
            }
            msg.push_str(": ");
        } else {
            msg.push_str("Error: ");
        }

        msg.push_str(&self.message);
        msg
    }
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format())
    }
}

impl From<String> for CompileError {
    fn from(message: String) -> Self {
        CompileError::new(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_resource() {
        let schema = "resource User { string name }";
        let result = parse_schema(schema);
        assert!(result.is_ok());
        let program = result.unwrap();
        assert_eq!(program.resources.len(), 1);
        assert_eq!(program.resources[0].name, "User");
        assert_eq!(program.resources[0].fields.len(), 1);
        assert_eq!(program.resources[0].fields[0].name, "name");
    }

    #[test]
    fn test_parse_multiple_fields() {
        let schema = r#"
            resource User {
                string name
                string email
                number age
                bool active
            }
        "#;
        let result = parse_schema(schema);
        assert!(result.is_ok());
        let program = result.unwrap();
        assert_eq!(program.resources[0].fields.len(), 4);
    }

    #[test]
    fn test_parse_optional_field() {
        let schema = "resource User { optional string name }";
        let result = parse_schema(schema);
        assert!(result.is_ok());
        let program = result.unwrap();
        assert!(program.resources[0].fields[0].optional);
        assert!(!program.resources[0].fields[0].nullable);
    }

    #[test]
    fn test_parse_nullable_field() {
        let schema = "resource Settings { nullable bool notifications }";
        let result = parse_schema(schema);
        assert!(result.is_ok());
        let program = result.unwrap();
        assert!(program.resources[0].fields[0].nullable);
        assert!(!program.resources[0].fields[0].optional);
    }

    #[test]
    fn test_parse_default_value() {
        let schema = "resource Config { default(10) number timeout }";
        let result = parse_schema(schema);
        assert!(result.is_ok());
        let program = result.unwrap();
        assert!(program.resources[0].fields[0].default.is_some());
    }

    #[test]
    fn test_parse_list_type() {
        let schema = "resource Names { list string names }";
        let result = parse_schema(schema);
        assert!(result.is_ok());
        let program = result.unwrap();
        match &program.resources[0].fields[0].field_type {
            ASTType::List(inner) => {
                assert_eq!(**inner, ASTType::Primitive("string".to_string()));
            }
            _ => panic!("Expected list type"),
        }
    }

    #[test]
    fn test_parse_named_type() {
        let schema = r#"
            resource User { string name }
            resource Users { list User users }
        "#;
        let result = parse_schema(schema);
        assert!(result.is_ok());
        let program = result.unwrap();
        match &program.resources[1].fields[0].field_type {
            ASTType::List(inner) => {
                assert_eq!(**inner, ASTType::Named("User".to_string()));
            }
            _ => panic!("Expected list of named type"),
        }
    }

    #[test]
    fn test_duplicate_resource_names() {
        let schema = r#"
            resource User { string name }
            resource User { string email }
        "#;
        let program = parse_schema(schema).unwrap();
        let result = Compiler::new(program);
        assert!(result.is_err());
    }

    #[test]
    fn test_duplicate_field_names() {
        let schema = "resource User { string name string name }";
        let program = parse_schema(schema).unwrap();
        let result = Compiler::new(program);
        assert!(result.is_err());
    }

    #[test]
    fn test_field_indexing() {
        let schema = r#"
            resource User {
                string name
                string email
                number age
            }
        "#;
        let result = parse_schema(schema);
        let program = result.unwrap();
        for (i, field) in program.resources[0].fields.iter().enumerate() {
            assert_eq!(field.index, i);
        }
    }

    #[test]
    fn test_pascal_case_validation() {
        let schema = "resource user { string name }";
        let result = parse_schema(schema);
        assert!(result.is_err());
    }

    #[test]
    fn test_compile_schema() {
        let schema = r#"
            resource User {
                string name
                optional number age
            }
        "#;
        let result = compile_schema(schema);
        assert!(result.is_ok());
    }

    // ========================================================================
    // IR STRUCTURE TESTS
    // ========================================================================

    #[test]
    fn test_ir_type_primitive() {
        let ir_type = IRType::Primitive("string".to_string());
        match ir_type {
            IRType::Primitive(s) => assert_eq!(s, "string"),
            _ => panic!("Expected primitive"),
        }
    }

    #[test]
    fn test_ir_type_resource_ref() {
        let ir_type = IRType::ResourceRef(0);
        match ir_type {
            IRType::ResourceRef(idx) => assert_eq!(idx, 0),
            _ => panic!("Expected resource ref"),
        }
    }

    #[test]
    fn test_ir_type_list() {
        let ir_type = IRType::List(Box::new(IRType::Primitive("string".to_string())));
        match ir_type {
            IRType::List(inner) => match *inner {
                IRType::Primitive(ref s) => assert_eq!(s, "string"),
                _ => panic!("Expected primitive inner type"),
            },
            _ => panic!("Expected list"),
        }
    }

    #[test]
    fn test_ir_type_equality() {
        let t1 = IRType::Primitive("string".to_string());
        let t2 = IRType::Primitive("string".to_string());
        assert_eq!(t1, t2);
    }

    #[test]
    fn test_ir_program_get_resource_index() {
        let ir = IRProgram {
            resources: vec![
                IRResource {
                    name: "User".to_string(),
                    fields: vec![],
                },
                IRResource {
                    name: "Post".to_string(),
                    fields: vec![],
                },
            ],
        };

        assert_eq!(ir.get_resource_index("User"), Some(0));
        assert_eq!(ir.get_resource_index("Post"), Some(1));
        assert_eq!(ir.get_resource_index("Unknown"), None);
    }

    #[test]
    fn test_ir_program_get_resource() {
        let ir = IRProgram {
            resources: vec![IRResource {
                name: "User".to_string(),
                fields: vec![],
            }],
        };

        assert!(ir.get_resource("User").is_some());
        assert!(ir.get_resource("Unknown").is_none());
    }

    #[test]
    fn test_ir_field_with_attributes() {
        let field = IRField {
            name: "age".to_string(),
            field_type: IRType::Primitive("number".to_string()),
            nullable: false,
            optional: true,
            default: None,
            index: 0,
        };

        assert_eq!(field.name, "age");
        assert!(field.optional);
        assert!(!field.nullable);
    }

    #[test]
    fn test_ir_field_with_default() {
        let field = IRField {
            name: "timeout".to_string(),
            field_type: IRType::Primitive("number".to_string()),
            nullable: false,
            optional: false,
            default: Some(DefaultValue {
                value: Literal::Number(10),
            }),
            index: 0,
        };

        assert!(field.default.is_some());
    }

    // ========================================================================
    // TYPE RESOLVER TESTS
    // ========================================================================

    #[test]
    fn test_type_resolver_new() {
        let schema = r#"
            resource User { string name }
            resource Post { string title }
        "#;
        let program = parse_schema(schema).unwrap();
        let resolver = TypeResolver::new(&program);

        assert!(resolver.is_ok());
        let resolver = resolver.unwrap();
        // Verify both resources are in the map
        assert!(resolver.resource_map.contains_key("User"));
        assert!(resolver.resource_map.contains_key("Post"));
    }

    #[test]
    fn test_resolve_primitive_types() {
        let schema = r#"
            resource Config {
                string name
                number timeout
                bool enabled
            }
        "#;
        let program = parse_schema(schema).unwrap();
        let resolver = TypeResolver::new(&program).unwrap();
        let ir = resolver.resolve(program).unwrap();

        assert_eq!(ir.resources.len(), 1);
        assert_eq!(ir.resources[0].fields.len(), 3);

        // Verify types are preserved
        match &ir.resources[0].fields[0].field_type {
            IRType::Primitive(s) => assert_eq!(s, "string"),
            _ => panic!("Expected primitive string"),
        }
        match &ir.resources[0].fields[1].field_type {
            IRType::Primitive(s) => assert_eq!(s, "number"),
            _ => panic!("Expected primitive number"),
        }
        match &ir.resources[0].fields[2].field_type {
            IRType::Primitive(s) => assert_eq!(s, "bool"),
            _ => panic!("Expected primitive bool"),
        }
    }

    #[test]
    fn test_resolve_named_type() {
        let schema = r#"
            resource User { string name }
            resource Profile { User user }
        "#;
        let program = parse_schema(schema).unwrap();
        let resolver = TypeResolver::new(&program).unwrap();
        let ir = resolver.resolve(program).unwrap();

        // Profile references User
        assert_eq!(ir.resources.len(), 2);
        assert_eq!(ir.resources[0].name, "User");
        assert_eq!(ir.resources[1].name, "Profile");

        // Check that the reference is resolved to index 0
        match &ir.resources[1].fields[0].field_type {
            IRType::ResourceRef(idx) => assert_eq!(*idx, 0),
            _ => panic!("Expected ResourceRef"),
        }
    }

    #[test]
    fn test_resolve_list_of_primitives() {
        let schema = r#"
            resource Names {
                list string names
            }
        "#;
        let program = parse_schema(schema).unwrap();
        let resolver = TypeResolver::new(&program).unwrap();
        let ir = resolver.resolve(program).unwrap();

        match &ir.resources[0].fields[0].field_type {
            IRType::List(inner) => match **inner {
                IRType::Primitive(ref s) => assert_eq!(s, "string"),
                _ => panic!("Expected primitive inner type"),
            },
            _ => panic!("Expected list type"),
        }
    }

    #[test]
    fn test_resolve_list_of_named_type() {
        let schema = r#"
            resource User { string name }
            resource Users { list User users }
        "#;
        let program = parse_schema(schema).unwrap();
        let resolver = TypeResolver::new(&program).unwrap();
        let ir = resolver.resolve(program).unwrap();

        // Check Users.users field
        match &ir.resources[1].fields[0].field_type {
            IRType::List(inner) => match **inner {
                IRType::ResourceRef(idx) => assert_eq!(idx, 0),
                _ => panic!("Expected ResourceRef inner type"),
            },
            _ => panic!("Expected list type"),
        }
    }

    #[test]
    fn test_resolve_nested_lists() {
        let schema = r#"
            resource Matrix {
                list list number values
            }
        "#;
        let program = parse_schema(schema).unwrap();
        let resolver = TypeResolver::new(&program).unwrap();
        let ir = resolver.resolve(program).unwrap();

        // Verify nested list structure: List(List(Primitive))
        match &ir.resources[0].fields[0].field_type {
            IRType::List(outer) => match **outer {
                IRType::List(ref inner) => match **inner {
                    IRType::Primitive(ref s) => assert_eq!(s, "number"),
                    _ => panic!("Expected primitive inner type"),
                },
                _ => panic!("Expected inner list"),
            },
            _ => panic!("Expected outer list"),
        }
    }

    #[test]
    fn test_resolve_preserves_field_attributes() {
        let schema = r#"
            resource Config {
                optional number age
                nullable bool enabled
                default(10) number timeout
            }
        "#;
        let program = parse_schema(schema).unwrap();
        let resolver = TypeResolver::new(&program).unwrap();
        let ir = resolver.resolve(program).unwrap();

        // Check first field (optional)
        assert!(ir.resources[0].fields[0].optional);
        assert!(!ir.resources[0].fields[0].nullable);

        // Check second field (nullable)
        assert!(ir.resources[0].fields[1].nullable);
        assert!(!ir.resources[0].fields[1].optional);

        // Check third field (default)
        assert!(ir.resources[0].fields[2].default.is_some());
    }

    #[test]
    fn test_resolve_undefined_type_error() {
        let schema = r#"
            resource User {
                Unknown unknownField
            }
        "#;
        let program = parse_schema(schema).unwrap();
        let resolver = TypeResolver::new(&program).unwrap();
        let result = resolver.resolve(program);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Undefined type"));
    }

    #[test]
    fn test_resolve_multiple_resources() {
        let schema = r#"
            resource User {
                string name
                string email
            }
            resource Post {
                string title
                User author
            }
            resource Blog {
                list Post posts
                User owner
            }
        "#;
        let program = parse_schema(schema).unwrap();
        let resolver = TypeResolver::new(&program).unwrap();
        let ir = resolver.resolve(program).unwrap();

        // Verify all resources are resolved
        assert_eq!(ir.resources.len(), 3);
        assert_eq!(ir.resources[0].name, "User");
        assert_eq!(ir.resources[1].name, "Post");
        assert_eq!(ir.resources[2].name, "Blog");

        // Verify references
        // Post.author should reference User (index 0)
        match &ir.resources[1].fields[1].field_type {
            IRType::ResourceRef(idx) => assert_eq!(*idx, 0),
            _ => panic!("Expected ResourceRef"),
        }

        // Blog.posts should be List(ResourceRef(1))
        match &ir.resources[2].fields[0].field_type {
            IRType::List(inner) => match **inner {
                IRType::ResourceRef(idx) => assert_eq!(idx, 1),
                _ => panic!("Expected ResourceRef"),
            },
            _ => panic!("Expected list"),
        }

        // Blog.owner should reference User (index 0)
        match &ir.resources[2].fields[1].field_type {
            IRType::ResourceRef(idx) => assert_eq!(*idx, 0),
            _ => panic!("Expected ResourceRef"),
        }
    }

    #[test]
    fn test_full_compilation_with_type_resolution() {
        let schema = r#"
            resource User { string name }
            resource Post { User author }
        "#;
        let result = compile_schema(schema);

        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.ir.resources.len(), 2);

        // Verify Post.author is resolved
        match &output.ir.resources[1].fields[0].field_type {
            IRType::ResourceRef(idx) => assert_eq!(*idx, 0),
            _ => panic!("Expected resolved type"),
        }
    }

    // ========================================================================
    // CYCLE DETECTOR TESTS
    // ========================================================================

    #[test]
    fn test_cycle_detector_no_cycles() {
        let schema = r#"
            resource User {
                string name
                string email
            }
            resource Post {
                string title
                User author
            }
        "#;
        let program = parse_schema(schema).unwrap();
        let resolver = TypeResolver::new(&program).unwrap();
        let ir = resolver.resolve(program).unwrap();

        let detector = CycleDetector::build(&ir).unwrap();
        let result = detector.detect();

        assert!(result.is_ok());
    }

    #[test]
    fn test_cycle_detector_self_reference() {
        let schema = r#"
            resource A {
                list A children
            }
        "#;
        let program = parse_schema(schema).unwrap();
        let resolver = TypeResolver::new(&program).unwrap();
        let ir = resolver.resolve(program).unwrap();

        let detector = CycleDetector::build(&ir).unwrap();
        let result = detector.detect();

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Cyclic dependency detected"));
        assert!(err.contains("A"));
    }

    #[test]
    fn test_cycle_detector_simple_cycle() {
        let schema = r#"
            resource A { B b }
            resource B { A a }
        "#;
        let program = parse_schema(schema).unwrap();
        let resolver = TypeResolver::new(&program).unwrap();
        let ir = resolver.resolve(program).unwrap();

        let detector = CycleDetector::build(&ir).unwrap();
        let result = detector.detect();

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Cyclic dependency detected"));
        assert!(err.contains("A"));
        assert!(err.contains("B"));
    }

    #[test]
    fn test_cycle_detector_three_way_cycle() {
        let schema = r#"
            resource A { B b }
            resource B { C c }
            resource C { A a }
        "#;
        let program = parse_schema(schema).unwrap();
        let resolver = TypeResolver::new(&program).unwrap();
        let ir = resolver.resolve(program).unwrap();

        let detector = CycleDetector::build(&ir).unwrap();
        let result = detector.detect();

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Cyclic dependency detected"));
    }

    #[test]
    fn test_cycle_detector_cycle_with_other_resources() {
        let schema = r#"
            resource A { B b }
            resource B { A a }
            resource C { string data }
            resource D { C ref }
        "#;
        let program = parse_schema(schema).unwrap();
        let resolver = TypeResolver::new(&program).unwrap();
        let ir = resolver.resolve(program).unwrap();

        let detector = CycleDetector::build(&ir).unwrap();
        let result = detector.detect();

        // Should detect the A ↔ B cycle
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Cyclic dependency detected"));
    }

    #[test]
    fn test_cycle_detector_list_in_cycle() {
        let schema = r#"
            resource A { list B items }
            resource B { A parent }
        "#;
        let program = parse_schema(schema).unwrap();
        let resolver = TypeResolver::new(&program).unwrap();
        let ir = resolver.resolve(program).unwrap();

        let detector = CycleDetector::build(&ir).unwrap();
        let result = detector.detect();

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Cyclic dependency detected"));
    }

    #[test]
    fn test_cycle_detector_nested_list_no_cycle() {
        let schema = r#"
            resource Item { string name }
            resource Collection { list list Item items }
        "#;
        let program = parse_schema(schema).unwrap();
        let resolver = TypeResolver::new(&program).unwrap();
        let ir = resolver.resolve(program).unwrap();

        let detector = CycleDetector::build(&ir).unwrap();
        let result = detector.detect();

        // Nested lists should not create cycles
        assert!(result.is_ok());
    }

    #[test]
    fn test_compile_schema_with_cycle_error() {
        let schema = r#"
            resource X { Y y }
            resource Y { X x }
        "#;
        let result = compile_schema(schema);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Cyclic dependency detected"));
    }

    #[test]
    fn test_compile_schema_without_cycle_success() {
        let schema = r#"
            resource User { string name }
            resource Post { User author }
            resource Blog { list Post posts }
        "#;
        let result = compile_schema(schema);

        assert!(result.is_ok());
    }

    #[test]
    fn test_cycle_error_message_format() {
        let schema = r#"
            resource A { B b }
            resource B { C c }
            resource C { A a }
        "#;
        let program = parse_schema(schema).unwrap();
        let resolver = TypeResolver::new(&program).unwrap();
        let ir = resolver.resolve(program).unwrap();

        let detector = CycleDetector::build(&ir).unwrap();
        let result = detector.detect();

        assert!(result.is_err());
        let err = result.unwrap_err();
        // Should show the cycle path with arrows
        assert!(err.contains(" → "));
    }

    #[test]
    fn test_cycle_detector_multiple_fields_with_cycle() {
        let schema = r#"
            resource A {
                string name
                B ref1
                B ref2
            }
            resource B {
                string title
                A parent
            }
        "#;
        let program = parse_schema(schema).unwrap();
        let resolver = TypeResolver::new(&program).unwrap();
        let ir = resolver.resolve(program).unwrap();

        let detector = CycleDetector::build(&ir).unwrap();
        let result = detector.detect();

        assert!(result.is_err());
    }

    // ========================================================================
    // BINARY ENCODING TESTS (Phase 3)
    // ========================================================================

    #[test]
    fn test_encode_string() {
        let mut encoder = BinaryEncoder::new();
        encoder.encode_string("hello");
        let bytes = encoder.finish();

        // Expected: [5, 0, 0, 0, 'h', 'e', 'l', 'l', 'o']
        // u32 length (5) in little-endian + UTF-8 bytes
        assert_eq!(bytes, vec![5, 0, 0, 0, b'h', b'e', b'l', b'l', b'o']);
    }

    #[test]
    fn test_encode_number() {
        let mut encoder = BinaryEncoder::new();
        encoder.encode_number(42);
        let bytes = encoder.finish();

        // Expected: i64(42) in little-endian (8 bytes)
        assert_eq!(bytes, vec![42, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_encode_bool_true() {
        let mut encoder = BinaryEncoder::new();
        encoder.encode_bool(true);
        let bytes = encoder.finish();

        assert_eq!(bytes, vec![0x01]);
    }

    #[test]
    fn test_encode_bool_false() {
        let mut encoder = BinaryEncoder::new();
        encoder.encode_bool(false);
        let bytes = encoder.finish();

        assert_eq!(bytes, vec![0x00]);
    }

    #[test]
    fn test_encode_primitive_value() {
        let schema = r#"
            resource User { string name }
        "#;
        let program = parse_schema(schema).unwrap();
        let resolver = TypeResolver::new(&program).unwrap();
        let ir = resolver.resolve(program).unwrap();

        let mut encoder = BinaryEncoder::new();
        let value = Value::String("test".to_string());
        let ir_type = IRType::Primitive("string".to_string());

        let result = encoder.encode_value(&value, &ir_type, &ir);
        assert!(result.is_ok());

        let bytes = encoder.finish();
        // u32(4) + "test"
        assert_eq!(bytes, vec![4, 0, 0, 0, b't', b'e', b's', b't']);
    }

    #[test]
    fn test_encode_list_of_primitives() {
        let schema = r#"
            resource Names { list string names }
        "#;
        let program = parse_schema(schema).unwrap();
        let resolver = TypeResolver::new(&program).unwrap();
        let ir = resolver.resolve(program).unwrap();

        let mut encoder = BinaryEncoder::new();
        let value = Value::List(vec![
            Value::String("a".to_string()),
            Value::String("b".to_string()),
        ]);
        let ir_type = IRType::List(Box::new(IRType::Primitive("string".to_string())));

        let result = encoder.encode_value(&value, &ir_type, &ir);
        assert!(result.is_ok());

        let bytes = encoder.finish();
        // u32(2) count + u32(1)+"a" + u32(1)+"b"
        assert_eq!(bytes, vec![
            2, 0, 0, 0,           // count = 2
            1, 0, 0, 0, b'a',     // "a"
            1, 0, 0, 0, b'b',     // "b"
        ]);
    }

    #[test]
    fn test_encode_list_of_numbers() {
        let schema = r#"
            resource Numbers { list number nums }
        "#;
        let program = parse_schema(schema).unwrap();
        let resolver = TypeResolver::new(&program).unwrap();
        let ir = resolver.resolve(program).unwrap();

        let mut encoder = BinaryEncoder::new();
        let value = Value::List(vec![
            Value::Number(10),
            Value::Number(20),
            Value::Number(30),
        ]);
        let ir_type = IRType::List(Box::new(IRType::Primitive("number".to_string())));

        let result = encoder.encode_value(&value, &ir_type, &ir);
        assert!(result.is_ok());

        let bytes = encoder.finish();
        // u32(3) + i64(10) + i64(20) + i64(30)
        assert_eq!(bytes.len(), 4 + 8 * 3);
        assert_eq!(&bytes[0..4], &[3, 0, 0, 0]); // count
    }

    #[test]
    fn test_encode_empty_list() {
        let schema = r#"
            resource Names { list string names }
        "#;
        let program = parse_schema(schema).unwrap();
        let resolver = TypeResolver::new(&program).unwrap();
        let ir = resolver.resolve(program).unwrap();

        let mut encoder = BinaryEncoder::new();
        let value = Value::List(vec![]);
        let ir_type = IRType::List(Box::new(IRType::Primitive("string".to_string())));

        let result = encoder.encode_value(&value, &ir_type, &ir);
        assert!(result.is_ok());

        let bytes = encoder.finish();
        // u32(0) count only
        assert_eq!(bytes, vec![0, 0, 0, 0]);
    }

    #[test]
    fn test_encode_nullable_null() {
        let schema = r#"
            resource Settings { nullable bool notifications }
        "#;
        let program = parse_schema(schema).unwrap();
        let resolver = TypeResolver::new(&program).unwrap();
        let ir = resolver.resolve(program).unwrap();

        let mut encoder = BinaryEncoder::new();
        let field_value = FieldValue {
            name: "notifications".to_string(),
            value: Value::Null,
            is_optional: false,
            is_nullable: true,
        };

        let result = encoder.encode_field(&field_value, &ir.resources[0].fields[0], &ir);
        assert!(result.is_ok());

        let bytes = encoder.finish();
        // 0x00 for null
        assert_eq!(bytes, vec![0x00]);
    }

    #[test]
    fn test_encode_nullable_present() {
        let schema = r#"
            resource Settings { nullable bool notifications }
        "#;
        let program = parse_schema(schema).unwrap();
        let resolver = TypeResolver::new(&program).unwrap();
        let ir = resolver.resolve(program).unwrap();

        let mut encoder = BinaryEncoder::new();
        let field_value = FieldValue {
            name: "notifications".to_string(),
            value: Value::Bool(true),
            is_optional: false,
            is_nullable: true,
        };

        let result = encoder.encode_field(&field_value, &ir.resources[0].fields[0], &ir);
        assert!(result.is_ok());

        let bytes = encoder.finish();
        // 0x01 for present + 0x01 for true
        assert_eq!(bytes, vec![0x01, 0x01]);
    }

    #[test]
    fn test_encode_optional_absent() {
        let schema = r#"
            resource User { optional number age }
        "#;
        let program = parse_schema(schema).unwrap();
        let resolver = TypeResolver::new(&program).unwrap();
        let ir = resolver.resolve(program).unwrap();

        let mut encoder = BinaryEncoder::new();
        let field_value = FieldValue {
            name: "age".to_string(),
            value: Value::Absent,
            is_optional: true,
            is_nullable: false,
        };

        let result = encoder.encode_field(&field_value, &ir.resources[0].fields[0], &ir);
        assert!(result.is_ok());

        let bytes = encoder.finish();
        // 0x00 for absent
        assert_eq!(bytes, vec![0x00]);
    }

    #[test]
    fn test_encode_optional_present() {
        let schema = r#"
            resource User { optional number age }
        "#;
        let program = parse_schema(schema).unwrap();
        let resolver = TypeResolver::new(&program).unwrap();
        let ir = resolver.resolve(program).unwrap();

        let mut encoder = BinaryEncoder::new();
        let field_value = FieldValue {
            name: "age".to_string(),
            value: Value::Number(30),
            is_optional: true,
            is_nullable: false,
        };

        let result = encoder.encode_field(&field_value, &ir.resources[0].fields[0], &ir);
        assert!(result.is_ok());

        let bytes = encoder.finish();
        // 0x01 for present + i64(30)
        assert_eq!(bytes.len(), 1 + 8);
        assert_eq!(bytes[0], 0x01);
    }

    #[test]
    fn test_encode_simple_resource() {
        let schema = r#"
            resource User {
                string name
                number age
                bool active
            }
        "#;
        let program = parse_schema(schema).unwrap();
        let resolver = TypeResolver::new(&program).unwrap();
        let ir = resolver.resolve(program).unwrap();

        let mut encoder = BinaryEncoder::new();
        let value = Value::Resource(vec![
            FieldValue {
                name: "name".to_string(),
                value: Value::String("Alice".to_string()),
                is_optional: false,
                is_nullable: false,
            },
            FieldValue {
                name: "age".to_string(),
                value: Value::Number(30),
                is_optional: false,
                is_nullable: false,
            },
            FieldValue {
                name: "active".to_string(),
                value: Value::Bool(true),
                is_optional: false,
                is_nullable: false,
            },
        ]);

        let result = encoder.encode_value(&value, &IRType::ResourceRef(0), &ir);
        assert!(result.is_ok());

        let bytes = encoder.finish();
        // u32(5) + "Alice" + i64(30) + bool(true)
        assert_eq!(&bytes[0..4], &[5, 0, 0, 0]); // length of "Alice"
        assert_eq!(&bytes[4..9], b"Alice");
        // age follows, then active
        assert!(bytes.len() > 9);
    }

    #[test]
    fn test_encode_nested_resource() {
        let schema = r#"
            resource User { string name }
            resource Profile { User user }
        "#;
        let program = parse_schema(schema).unwrap();
        let resolver = TypeResolver::new(&program).unwrap();
        let ir = resolver.resolve(program).unwrap();

        let mut encoder = BinaryEncoder::new();

        // Create nested User resource
        let user_value = Value::Resource(vec![
            FieldValue {
                name: "name".to_string(),
                value: Value::String("Bob".to_string()),
                is_optional: false,
                is_nullable: false,
            },
        ]);

        // Create Profile with User
        let profile_value = Value::Resource(vec![
            FieldValue {
                name: "user".to_string(),
                value: user_value,
                is_optional: false,
                is_nullable: false,
            },
        ]);

        let result = encoder.encode_value(&profile_value, &IRType::ResourceRef(1), &ir);
        assert!(result.is_ok());

        let bytes = encoder.finish();
        // u32(3) + "Bob"
        assert_eq!(&bytes[0..4], &[3, 0, 0, 0]); // length of "Bob"
        assert_eq!(&bytes[4..7], b"Bob");
    }

    #[test]
    fn test_encode_type_mismatch_error() {
        let schema = r#"
            resource User { string name }
        "#;
        let program = parse_schema(schema).unwrap();
        let resolver = TypeResolver::new(&program).unwrap();
        let ir = resolver.resolve(program).unwrap();

        let mut encoder = BinaryEncoder::new();
        let value = Value::Number(42); // Wrong type!
        let ir_type = IRType::Primitive("string".to_string());

        let result = encoder.encode_value(&value, &ir_type, &ir);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Type mismatch"));
    }

    #[test]
    fn test_encode_resource_field_count_mismatch() {
        let schema = r#"
            resource User {
                string name
                number age
            }
        "#;
        let program = parse_schema(schema).unwrap();
        let resolver = TypeResolver::new(&program).unwrap();
        let ir = resolver.resolve(program).unwrap();

        let mut encoder = BinaryEncoder::new();
        // Only provide 1 field when 2 are expected
        let value = Value::Resource(vec![
            FieldValue {
                name: "name".to_string(),
                value: Value::String("Alice".to_string()),
                is_optional: false,
                is_nullable: false,
            },
        ]);

        let result = encoder.encode_value(&value, &IRType::ResourceRef(0), &ir);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Field count mismatch"));
    }

    // ========================================================================
    // CODE GENERATION TESTS (Phase 4)
    // ========================================================================

    #[test]
    fn test_code_generation_simple_resource() {
        let schema = r#"
            resource User {
                string name
                number age
                bool active
            }
        "#;
        let output = compile_schema(schema).unwrap();

        // Check TypeScript client generation
        assert!(output.generated_code.typescript_client.contains("export interface IUser"));
        assert!(output.generated_code.typescript_client.contains("export class User"));
        assert!(output.generated_code.typescript_client.contains("BinaryReader"));
        assert!(output.generated_code.typescript_client.contains("readString()"));
        assert!(output.generated_code.typescript_client.contains("readNumber()"));
        assert!(output.generated_code.typescript_client.contains("readBool()"));
        assert!(output.generated_code.typescript_client.contains("getName()"));
        assert!(output.generated_code.typescript_client.contains("toJSON()"));

        // Check Rust server generation
        assert!(output.generated_code.rust_server.contains("pub struct User"));
        assert!(output.generated_code.rust_server.contains("pub name: String"));
        assert!(output.generated_code.rust_server.contains("pub age: i64"));
        assert!(output.generated_code.rust_server.contains("pub active: bool"));
        assert!(output.generated_code.rust_server.contains("pub fn new()"));
        assert!(output.generated_code.rust_server.contains("pub fn name(mut self"));
        assert!(output.generated_code.rust_server.contains("pub fn encode("));
    }

    #[test]
    fn test_code_generation_optional_fields() {
        let schema = r#"
            resource User {
                string name
                optional number age
            }
        "#;
        let output = compile_schema(schema).unwrap();

        // TypeScript should have optional field
        assert!(output.generated_code.typescript_client.contains("age?: number"));

        // Rust should use Option
        assert!(output.generated_code.rust_server.contains("pub age: Option<i64>"));
        assert!(output.generated_code.rust_server.contains("Value::Absent"));
    }

    #[test]
    fn test_code_generation_nullable_fields() {
        let schema = r#"
            resource Settings {
                nullable bool notifications
            }
        "#;
        let output = compile_schema(schema).unwrap();

        // TypeScript should have optional field
        assert!(output.generated_code.typescript_client.contains("notifications?: boolean"));

        // Rust should use Option
        assert!(output.generated_code.rust_server.contains("pub notifications: Option<bool>"));
        assert!(output.generated_code.rust_server.contains("Value::Null"));
    }

    #[test]
    fn test_code_generation_list_types() {
        let schema = r#"
            resource Names {
                list string names
            }
        "#;
        let output = compile_schema(schema).unwrap();

        // TypeScript array type
        assert!(output.generated_code.typescript_client.contains("names: string[]"));
        assert!(output.generated_code.typescript_client.contains("readU32()"));

        // Rust Vec type
        assert!(output.generated_code.rust_server.contains("pub names: Vec<String>"));
        assert!(output.generated_code.rust_server.contains("Value::List"));
    }

    #[test]
    fn test_code_generation_nested_resources() {
        let schema = r#"
            resource User { string name }
            resource Profile { User user }
        "#;
        let output = compile_schema(schema).unwrap();

        // TypeScript nested type
        assert!(output.generated_code.typescript_client.contains("export interface IUser"));
        assert!(output.generated_code.typescript_client.contains("export interface IProfile"));
        assert!(output.generated_code.typescript_client.contains("user: IUser"));

        // Rust nested type
        assert!(output.generated_code.rust_server.contains("pub struct User"));
        assert!(output.generated_code.rust_server.contains("pub struct Profile"));
        assert!(output.generated_code.rust_server.contains("pub user: User"));
    }

    #[test]
    fn test_code_generation_multiple_resources() {
        let schema = r#"
            resource User { string name }
            resource Post { string title }
            resource Comment { string text }
        "#;
        let output = compile_schema(schema).unwrap();

        // All resources should be generated
        assert!(output.generated_code.typescript_client.contains("export class User"));
        assert!(output.generated_code.typescript_client.contains("export class Post"));
        assert!(output.generated_code.typescript_client.contains("export class Comment"));

        assert!(output.generated_code.rust_server.contains("pub struct User"));
        assert!(output.generated_code.rust_server.contains("pub struct Post"));
        assert!(output.generated_code.rust_server.contains("pub struct Comment"));
    }

    #[test]
    fn test_typescript_getter_methods() {
        let schema = r#"
            resource User {
                string name
                number age
            }
        "#;
        let output = compile_schema(schema).unwrap();

        // Should have capitalized getter methods
        assert!(output.generated_code.typescript_client.contains("getName()"));
        assert!(output.generated_code.typescript_client.contains("getAge()"));
    }

    #[test]
    fn test_rust_builder_pattern() {
        let schema = r#"
            resource User {
                string name
                number age
            }
        "#;
        let output = compile_schema(schema).unwrap();

        // Should have builder-style setters
        assert!(output.generated_code.rust_server.contains("pub fn name(mut self, value: String) -> Self"));
        assert!(output.generated_code.rust_server.contains("pub fn age(mut self, value: i64) -> Self"));
        assert!(output.generated_code.rust_server.contains("self.name = value"));
        assert!(output.generated_code.rust_server.contains("self.age = value"));
        assert!(output.generated_code.rust_server.contains("self\n"));
    }

    #[test]
    fn test_generated_code_headers() {
        let schema = r#"
            resource User { string name }
        "#;
        let output = compile_schema(schema).unwrap();

        // Check headers
        assert!(output.generated_code.typescript_client.contains("Generated by Previous Compiler"));
        assert!(output.generated_code.typescript_client.contains("DO NOT EDIT"));
        assert!(output.generated_code.rust_server.contains("Generated by Previous Compiler"));
        assert!(output.generated_code.rust_server.contains("DO NOT EDIT"));
    }

    #[test]
    fn test_rust_imports() {
        let schema = r#"
            resource User { string name }
        "#;
        let output = compile_schema(schema).unwrap();

        // Should import necessary types
        assert!(output.generated_code.rust_server.contains("use previous::{Value, FieldValue, BinaryEncoder, IRType, IRProgram}"));
    }

    #[test]
    fn test_typescript_binary_reader_utility() {
        let schema = r#"
            resource User { string name }
        "#;
        let output = compile_schema(schema).unwrap();

        // Should have BinaryReader class
        assert!(output.generated_code.typescript_client.contains("class BinaryReader"));
        assert!(output.generated_code.typescript_client.contains("private buffer: Uint8Array"));
        assert!(output.generated_code.typescript_client.contains("private offset: number"));
        assert!(output.generated_code.typescript_client.contains("readString()"));
        assert!(output.generated_code.typescript_client.contains("readNumber()"));
        assert!(output.generated_code.typescript_client.contains("readBool()"));
        assert!(output.generated_code.typescript_client.contains("readU32()"));
        assert!(output.generated_code.typescript_client.contains("readByte()"));
    }

    #[test]
    fn test_rust_to_value_conversion() {
        let schema = r#"
            resource User {
                string name
                number age
            }
        "#;
        let output = compile_schema(schema).unwrap();

        // Should have to_value method
        assert!(output.generated_code.rust_server.contains("fn to_value(&self) -> Value"));
        assert!(output.generated_code.rust_server.contains("Value::Resource(vec!["));
        assert!(output.generated_code.rust_server.contains("FieldValue {"));
    }
}
