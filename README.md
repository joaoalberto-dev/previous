# Previous

Is a binary protocol and BFF framework.

Previous is made of a Schema definition, a compiler, client and server utils to help you create complete solutions with one tool.

## Compiler Status

| Phase | Status | Docs |
|-------|--------|------|
| 1. AST Construction | ✅ Complete | [README.md](README.md) |
| 2. Type Resolution + IR | ✅ Complete | [PHASE2_COMPLETE.md](PHASE2_COMPLETE.md) |
| 3. Binary Encoding | ✅ Complete | [PHASE3_COMPLETE.md](PHASE3_COMPLETE.md) |
| 4. Code Generation | ✅ Complete | [PHASE4_COMPLETE.md](PHASE4_COMPLETE.md) |
| 5. CLI & File I/O | ✅ Complete | [PHASE5_COMPLETE.md](PHASE5_COMPLETE.md) |

**🎉 PROJECT COMPLETE!** All 5 phases implemented. The Previous compiler is production-ready!

## Quick Start

```bash
# Compile a schema file
cargo run -- examples/user.pr --out ./generated

# Run demo
cargo run -- demo

# Show help
cargo run -- --help
```

## Design

Previous is built around `Resource`'s.

Resources are a way to describe the data your server and client can interchange.

### Anatomy of a Resource schema

You can create your Resources by defining its shape with field types and names.

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

This will be compiled to TypeScript and Rust code using the Previous compiler:

```bash
# Compile the schema
previouscc schema.pr --out ./generated
```

**Generated TypeScript Client:**
```typescript
import { User } from './generated/client';

const response = await fetch('/api/user/1');
const buffer = await response.arrayBuffer();
const user = new User(new Uint8Array(buffer));

console.log(user.getName());    // Lazy parsed
console.log(user.getEmail());   // Lazy parsed
console.log(user.toJSON());     // Full JSON conversion
```

**Generated Rust Server:**
```rust
use generated::User;

// Build user with builder pattern
let user = User::new()
    .name("Jhon".to_string())
    .email("jhon@email.com".to_string())
    .age(Some(30))
    .active(false);

// Encode to binary
let binary_data = user.encode(&ir_program)?;

// Send to client
Ok(Response::new(binary_data))
```

The Previous compiler handles all the serialization/deserialization automatically. The binary protocol is more efficient than JSON while maintaining type safety on both client and server.