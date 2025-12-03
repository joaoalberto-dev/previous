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

        // 3. TODO: Cycle detection in Task 3

        // 4. Return compiled IR
        Ok(CompiledOutput { ir })
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

#[derive(Debug)]
pub struct CompiledOutput {
    pub ir: IRProgram,
}

impl CompiledOutput {
    pub fn new() -> Self {
        CompiledOutput {
            ir: IRProgram {
                resources: Vec::new(),
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
}
