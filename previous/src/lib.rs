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

pub fn run() {
    println!("Previous");
}
