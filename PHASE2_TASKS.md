# Phase 2 Implementation Tasks

## Overview
Convert AST → IR with type resolution and cyclic dependency detection.

## Task 1: Add IR Data Structures

**File:** `src/lib.rs`

**What to add:**
```rust
// New enum for resolved types
pub enum IRType {
    Primitive(String),
    ResourceRef(usize),
    List(Box<IRType>),
}

// New structures
pub struct IRField {
    pub name: String,
    pub field_type: IRType,
    pub nullable: bool,
    pub optional: bool,
    pub default: Option<DefaultValue>,
    pub index: usize,
}

pub struct IRResource {
    pub name: String,
    pub fields: Vec<IRField>,
}

pub struct IRProgram {
    pub resources: Vec<IRResource>,
}
```

**Checklist:**
- [ ] Add `IRType` enum
- [ ] Add `IRField`, `IRResource`, `IRResource` structs
- [ ] Derive Debug, Clone, PartialEq on all
- [ ] Add helper methods if needed (e.g., `IRProgram::get_resource(name)`)

---

## Task 2: Implement TypeResolver

**File:** `src/lib.rs`

**What to implement:**

```rust
pub struct TypeResolver {
    resource_map: HashMap<String, usize>,
}

impl TypeResolver {
    /// Build a resolver from the AST program
    pub fn new(program: &Program) -> Result<Self, String> {
        // Build resource_map: name -> index
        // Validate no duplicate names (already done, but re-check)
    }
    
    /// Convert AST types to IR types
    fn resolve_type(&self, ast_type: &ASTType) -> Result<IRType, String> {
        match ast_type {
            ASTType::Primitive(p) => Ok(IRType::Primitive(p.clone())),
            ASTType::Named(name) => {
                // Check if name exists in resource_map
                // If yes: Ok(IRType::ResourceRef(index))
                // If no: Err(format!("Undefined type: {}", name))
            }
            ASTType::List(inner) => {
                // Recursively resolve inner type
                let resolved = self.resolve_type(inner)?;
                Ok(IRType::List(Box::new(resolved)))
            }
        }
    }
    
    /// Transform AST Program to IR Program
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
```

**Checklist:**
- [ ] Implement `TypeResolver::new()` - build resource_map
- [ ] Implement `resolve_type()` - recursive type resolution
- [ ] Implement `resolve()` - full AST → IR transformation
- [ ] Handle error case: undefined named type
- [ ] Test with valid types (primitives, named, lists, nested lists)
- [ ] Test with invalid types (undefined resource)

---

## Task 3: Implement Cycle Detection

**File:** `src/lib.rs`

**What to implement:**

```rust
pub struct CycleDetector {
    graph: Vec<Vec<usize>>, // adjacency list
}

impl CycleDetector {
    /// Build a dependency graph from IR
    pub fn build(ir: &IRProgram) -> Result<Self, String> {
        let mut graph = vec![Vec::new(); ir.resources.len()];
        
        // For each resource, for each field type, add edge to dependency
        for (res_idx, resource) in ir.resources.iter().enumerate() {
            for field in &resource.fields {
                Self::collect_refs(res_idx, &field.field_type, &mut graph);
            }
        }
        
        Ok(CycleDetector { graph })
    }
    
    /// Helper: collect all resource references from a type
    fn collect_refs(from_idx: usize, ir_type: &IRType, graph: &mut Vec<Vec<usize>>) {
        match ir_type {
            IRType::Primitive(_) => {},
            IRType::ResourceRef(to_idx) => {
                graph[from_idx].push(*to_idx);
            }
            IRType::List(inner) => {
                Self::collect_refs(from_idx, inner, graph);
            }
        }
    }
    
    /// Detect cycles using DFS
    pub fn detect(&self) -> Result<(), String> {
        let n = self.graph.len();
        let mut visited = vec![false; n];
        let mut rec_stack = vec![false; n];
        let mut path = Vec::new(); // for error reporting
        
        for i in 0..n {
            if !visited[i] {
                self.dfs(i, &mut visited, &mut rec_stack, &mut path)?;
            }
        }
        
        Ok(())
    }
    
    fn dfs(
        &self,
        node: usize,
        visited: &mut Vec<bool>,
        rec_stack: &mut Vec<bool>,
        path: &mut Vec<usize>,
    ) -> Result<(), String> {
        visited[node] = true;
        rec_stack[node] = true;
        path.push(node);
        
        for &neighbor in &self.graph[node] {
            if !visited[neighbor] {
                self.dfs(neighbor, visited, rec_stack, path)?;
            } else if rec_stack[neighbor] {
                // Found a cycle
                // Build error message showing the cycle
                let cycle_start = path.iter().position(|&n| n == neighbor).unwrap();
                let cycle_path = &path[cycle_start..];
                let cycle_names: Vec<_> = cycle_path.iter()
                    .map(|&idx| &self.resource_names[idx])
                    .collect();
                return Err(format!("Cyclic dependency detected: {}", 
                    cycle_names.join(" → ")));
            }
        }
        
        path.pop();
        rec_stack[node] = false;
        Ok(())
    }
}
```

Wait—we need resource names in the detector for error messages. Let me refactor:

```rust
pub struct CycleDetector {
    graph: Vec<Vec<usize>>,
    resource_names: Vec<String>,
}

impl CycleDetector {
    pub fn build(ir: &IRProgram) -> Result<Self, String> {
        let mut graph = vec![Vec::new(); ir.resources.len()];
        
        for (res_idx, resource) in ir.resources.iter().enumerate() {
            for field in &resource.fields {
                Self::collect_refs(res_idx, &field.field_type, &mut graph);
            }
        }
        
        let resource_names: Vec<String> = ir.resources
            .iter()
            .map(|r| r.name.clone())
            .collect();
        
        Ok(CycleDetector { graph, resource_names })
    }
    
    // ... rest same
}
```

**Checklist:**
- [ ] Implement `CycleDetector::build()` - build adjacency list
- [ ] Implement `collect_refs()` - extract resource refs from types
- [ ] Implement `detect()` - top-level DFS loop
- [ ] Implement `dfs()` - recursive cycle detection with backtracking
- [ ] Format error messages with cycle path
- [ ] Test: self-referential (A → A)
- [ ] Test: simple cycle (A → B → A)
- [ ] Test: deep cycle (A → B → C → A)
- [ ] Test: no cycle (DAG)
- [ ] Test: multiple independent cycles (should catch first)

---

## Task 4: Update CompiledOutput

**File:** `src/lib.rs`

**Changes:**

```rust
// OLD:
pub struct CompiledOutput {
    pub resources: Vec<Resource>,
}

// NEW:
pub struct CompiledOutput {
    pub ir: IRProgram,
}

impl CompiledOutput {
    pub fn new() -> Self {
        CompiledOutput {
            ir: IRProgram {
                resources: Vec::new(),
            }
        }
    }
}
```

**Checklist:**
- [ ] Update struct
- [ ] Update `Compiler::compile()` to produce IR instead of AST
- [ ] Update main.rs demo to work with IR
- [ ] Update tests if needed

---

## Task 5: Update Compiler Phase

**File:** `src/lib.rs`

**Changes to `Compiler` impl:**

```rust
pub fn compile(&self) -> Result<CompiledOutput, String> {
    // 1. Validate AST
    Self::validate_ast(&self.program)?;
    
    // 2. Resolve types
    let resolver = TypeResolver::new(&self.program)?;
    let ir = resolver.resolve(self.program.clone())?;
    
    // 3. Detect cycles
    let detector = CycleDetector::build(&ir)?;
    detector.detect()?;
    
    // 4. Return compiled IR
    Ok(CompiledOutput { ir })
}

fn validate_ast(program: &Program) -> Result<(), String> {
    // Move existing validation here (unique names, PascalCase)
    Ok(())
}
```

**Checklist:**
- [ ] Extract validation to `validate_ast()`
- [ ] Add type resolution phase
- [ ] Add cycle detection phase
- [ ] Update return type
- [ ] All existing tests still pass

---

## Task 6: Update Tests

**File:** `src/lib.rs` tests section

Add new test cases:

```rust
#[test]
fn test_type_resolution_named() { ... }

#[test]
fn test_type_resolution_list_named() { ... }

#[test]
fn test_undefined_type_error() { ... }

#[test]
fn test_cycle_self_reference() { ... }

#[test]
fn test_cycle_simple() { ... }

#[test]
fn test_cycle_deep() { ... }

#[test]
fn test_no_cycle_dag() { ... }
```

**Checklist:**
- [ ] Add all cycle detection tests
- [ ] Add type resolution error tests
- [ ] Add valid IR tests
- [ ] Run `cargo test` - all pass

---

## Task 7: Update main.rs Demo

**File:** `src/main.rs`

Print IR instead of AST:

```rust
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
                    field.field_type,  // Now IRType
                    if field.nullable { " (nullable)" } else { "" },
                    if field.optional { " (optional)" } else { "" },
                    if field.default.is_some() { " (has default)" } else { "" }
                );
            }
        }
    }
    Err(e) => eprintln!("Compilation error: {}", e),
}
```

**Checklist:**
- [ ] Update output to show IR
- [ ] Test with valid schema
- [ ] Test with undefined type (error)
- [ ] Test with cyclic dependency (error)

---

## Implementation Order

1. **Task 1** - Add IR structures
2. **Task 2** - TypeResolver
3. **Task 3** - CycleDetector
4. **Task 4** - Update CompiledOutput
5. **Task 5** - Update Compiler
6. **Task 6** - Add tests
7. **Task 7** - Update demo

---

## Acceptance Criteria

- [ ] All 7 tasks complete
- [ ] `cargo test` passes (15+ tests)
- [ ] `cargo build` succeeds with no warnings
- [ ] `cargo run` shows valid IR or appropriate error
- [ ] Handles: primitives, named types, lists, nesting
- [ ] Detects: undefined types, cyclic dependencies
- [ ] Preserves: field attributes (nullable, optional, default)
