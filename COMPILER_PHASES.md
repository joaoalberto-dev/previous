# Previous Compiler Phases

## Current Status: Phase 1 Complete ✅
**AST Construction** - Lexer → Parser → AST

Delivers:
- Token stream
- Parsed AST with all syntax rules
- Basic validation (PascalCase, unique names)

---

## Phase 2: Type Resolution + IR (NEXT) ⏭️

### Goals
1. Convert AST → IR with validated type references
2. Resolve all `ASTType::Named(String)` to `IRType::ResourceRef(usize)`
3. Detect cyclic dependencies
4. Prepare field metadata for encoding

### IR Data Structures

```rust
pub struct IRProgram {
    pub resources: Vec<IRResource>,
    pub resource_index: HashMap<String, usize>, // for fast lookup
}

pub struct IRResource {
    pub name: String,
    pub fields: Vec<IRField>,
}

pub struct IRField {
    pub name: String,
    pub field_type: IRType,
    pub nullable: bool,
    pub optional: bool,
    pub default: Option<DefaultValue>,
    pub index: usize,
}

pub enum IRType {
    Primitive(String),           // "string", "number", "bool"
    ResourceRef(usize),          // index into IRProgram.resources
    List(Box<IRType>),
}
```

### Implementation Steps

#### Step 1: Add IR structures to lib.rs
- Define `IRProgram`, `IRResource`, `IRField`, `IRType`
- Derive Debug, Clone on all

#### Step 2: Create `TypeResolver` phase
```rust
pub struct TypeResolver {
    resource_map: HashMap<String, usize>,
}

impl TypeResolver {
    pub fn new(program: &Program) -> Result<Self, String>
    pub fn resolve(&self, program: Program) -> Result<IRProgram, String>
}
```

Key tasks:
- Build `resource_map: HashMap<name → index>`
- For each resource, for each field:
  - Resolve `ASTType::Named(name)` → verify name exists → convert to `ResourceRef(index)`
  - Recursively resolve List inner types
  - Preserve Primitive types as-is

#### Step 3: Detect cyclic dependencies
After type resolution, run cycle detection:

```rust
pub struct CycleDetector {
    graph: Vec<Vec<usize>>, // adj list: resource_index → resources it refs
}

impl CycleDetector {
    pub fn detect(&self) -> Result<(), String>
}
```

Algorithm:
- Build adjacency list from resolved types
- DFS from each unvisited node
- Track: visited, rec_stack
- If node in rec_stack → cycle found → error

#### Step 4: Update Compiler phase
```rust
pub struct Compiler {
    ast: Program,
}

impl Compiler {
    pub fn compile(&self) -> Result<CompiledOutput, String> {
        // 1. Validate AST (already done)
        Compiler::validate_ast(&self.ast)?;
        
        // 2. Type resolution
        let ir = TypeResolver::new(&self.ast)?
            .resolve(self.ast)?;
        
        // 3. Cycle detection
        CycleDetector::build(&ir)?.detect()?;
        
        // 4. Return compiled IR
        Ok(CompiledOutput { ir })
    }
}
```

### Tests to Add

```rust
#[test]
fn test_resolve_named_type() {
    // resource User { string name }
    // resource Profile { User user }
    // After resolution, Profile.fields[0].field_type should be ResourceRef(0)
}

#[test]
fn test_resolve_list_of_named_type() {
    // resource User { ... }
    // resource Users { list User users }
    // After resolution should be List(ResourceRef(0))
}

#[test]
fn test_undefined_named_type() {
    // resource User { Unknown x }
    // Should error: "Unknown type: Unknown"
}

#[test]
fn test_cyclic_dependency_simple() {
    // resource A { B b }
    // resource B { A a }
    // Should error: "Cyclic dependency detected: A → B → A"
}

#[test]
fn test_cyclic_dependency_self() {
    // resource A { list A items }
    // Should error: "Cyclic dependency detected: A → A"
}

#[test]
fn test_cyclic_dependency_deep() {
    // A → B → C → A
    // Should error
}

#[test]
fn test_no_cycle_in_dag() {
    // A → B → C (no back edge)
    // Should succeed
}
```

### CompiledOutput Changes

**Current:**
```rust
pub struct CompiledOutput {
    pub resources: Vec<Resource>, // still AST
}
```

**After Phase 2:**
```rust
pub struct CompiledOutput {
    pub ir: IRProgram,
}
```

Public API stays the same:
```rust
pub fn compile_schema(input: &str) -> Result<CompiledOutput, String>
```

---

## Phase 3: Binary Encoding Model

Define the encoding spec for each type.

Example rules:
- `string`: prefix with u32 length, then UTF-8 bytes
- `number`: i64 (8 bytes, little-endian)
- `bool`: 1 byte (0x00 or 0x01)
- `list`: prefix with u32 count, then each item encoded
- `nullable`: prefix with 1 byte (0x00=null, 0x01=present), then value if present
- `optional`: similar to nullable
- `resource`: each field in order (index is implicit)

---

## Phase 4: Code Generation

Generate TypeScript/Rust code:
- Client: deserializers, lazy field access
- Server: serializers, field builders

---

## Phase 5: CLI + File I/O

Handle:
- Reading .pr files
- `--out` directory
- Writing generated code

---

## Summary

| Phase | Status | Output |
|-------|--------|--------|
| 1. AST Construction | ✅ Done | AST |
| 2. Type Resolution + IR | ⏭️ Next | IR (validated, no cycles) |
| 3. Encoding Model | 📅 Planned | Binary spec |
| 4. Code Generation | 📅 Planned | TS/Rust code |
| 5. CLI | 📅 Planned | CLI tool |

**Start with Phase 2** → builds everything else.
