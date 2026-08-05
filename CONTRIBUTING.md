# Contributing to Flame

Welcome to the Flame language project! This guide contains everything you need
to know about contributing to Flame, extending its compiler and runtime, adding
standard library modules, implementing built-in and custom annotations, and
developing native plugins.

---

## Table of Contents

1. [Project Overview & Architecture](#project-overview--architecture)
2. [How to Add a New Standard Library (Std Lib) Module](#how-to-add-a-new-standard-library-std-lib-module)
3. [How to Add Built-In Annotations in Flame](#how-to-add-built-in-annotations-in-flame)
4. [Writing Custom Annotations in Flame (`annotation/` folder)](#writing-custom-annotations-in-flame-annotation-folder)
5. [Native Plugins & `flame-macro`](#native-plugins--flame-macro)
6. [IDE Extension (`flame-ide`) Integration](#ide-extension-flame-ide-integration)
7. [Running Tests & Validating Changes](#running-tests--validating-changes)

---

## Project Overview & Architecture

The Flame repository is organized into several key components:

- **`src/`**: The core Flame compiler, tree-walking interpreter, typechecker,
  and AOT compiler.
  - `lexer.rs`: Tokenizes Flame source code with span tracking.
  - `parser.rs`: Recursive-descent parser producing AST statements and
    expressions.
  - `typechecker.rs`: Static type-checker and symbol table inference.
  - `runner.rs`: Execution engine evaluating AST nodes, environments, closures,
    and annotations.
  - `vm.rs`: Value representation (`Value`), `CValue` FFI bindings, callback
    registration, and thread coordination.
  - `stdlib.rs`: Standard library resolution and builtin functions (`print`,
    `assert`, `type_of`, etc.).
  - `native_std/`: Native standard library modules (`thread`, `process`, `fs`,
    `math`, `time`, `os`, `net`, etc.).
  - `ide.rs` & `std_docs.rs`: Autocompletion, hover docs, and symbol scanning
    for IDE tooling.
  - `aot_compiler.rs`: Generates native Rust code, links native object files,
    and produces standalone binaries.
  - `main.rs`: CLI commands (`run`, `build`, `check`, `format`, `new`, `add`,
    `remove`, `list-plugins`).
- **`flame-macro/`**: Attribute procedural macros (`#[flame]`,
  `#[flame_export]`, `#[flame(daemon)]`, etc.) used by native Rust plugins.
- **`flame-ide/`**: VS Code extension providing language server features
  (diagnostics, completions, hover, syntax highlighting, formatting).
- **`examples/`**: Real-world sample projects, CLI tools, and native plugins
  (e.g. Axum web server).
- **`docs/`**: Comprehensive guides and reference manuals.

---

## How to Add a New Standard Library (Std Lib) Module

Adding a new standard library module to Flame (e.g., `std.crypto`, `std.json`,
`std.path`) involves the following steps:

### Step 1: Implement the Rust

Create or update the module implementation in `src/native_std/`:

```rust
// In src/native_std/pi.rs
use crate::vm::Value;
use std::collections::HashMap;

pub fn init() -> HashMap<String, Value> {
    let mut m = HashMap::new();

    m.insert(
        "pi".to_string(),
        Value::NativeCallback(|_args| Ok(Value::Float(std::f64::consts::PI))),
    );
}
```

### Step 2: Register in `src/stdlib.rs`

1. Add the module name to the standard modules list in `fn register_std_module`.
2. Add the expected function signatures and types to the type environment when
   imported:

```rust
"std.pi" => Some(crate::native_std::pi::init()),
```

### Step 3: Register in `src/main.rs` & `src/ide.rs` (IDE completions & hover)

1. Add the module to `list_std_modules` in `src/main.rs`:

2. Add module functions and documentation to `src/ide.rs` and `src/std_docs.rs`
   so developers get instant autocomplete and hover documentation in VS Code!

---

## How to Add Built-In Annotations in Flame

Built-in annotations (like `@Cli`, `@Command`, `@Test`, `@Benchmark`, `@Logger`,
`@Setup`, `@Cleanup`) provide declarative compile-time and runtime metadata.

### Step 1: Update the Lexer & Parser

Ensure `src/parser.rs` parses the annotation. `Parser::parse_annotation` parses
`@Identifier(key: val, ...)` or bare `@Identifier`.

### Step 2: Register in `src/typechecker.rs`

Add validation logic inside `TypeChecker::check_annotation`:

```rust
match annotation.name.as_str() {
    "Cli" | "Command" | "Test" | "Benchmark" | "MyNewAnnotation" => {
        // Validate argument types and allowed targets (functions, structs, etc.)
    }
    _ => {
        // Custom user-defined annotation fallback
    }
}
```

### Step 3: Implement Runtime Handling in `src/runner.rs`

1. If the annotation transforms execution before or after a function runs, hook
   it in `Runner::invoke_callback_value` or `Runner::eval_stmt`.
2. Built-in annotations can inspect arguments, inject behavior, or configure
   CLI/test runners:

```rust
for anno in annotations {
    match anno.name.as_str() {
        "MyNewAnnotation" => {
            // Read arguments and apply runtime behavior
        }
        _ => {}
    }
}
```

### Step 4: Add IDE Tooling & Documentation

Add documentation for the annotation in `src/ide.rs` in
`get_keyword_completions` or `hover_info` so users see syntax help and code
examples on hover.

---

## Writing Custom Annotations in Flame (`annotation/` folder)

Flame allows you to define custom annotations directly in Flame code!

### 1. Define the Annotation in `src/annotation/<name>.fm`

```flame
// src/annotation/logger.fm
annotation Logger(prefix: String = "APP") {
    print($"[LOGGER INIT] Prefix configured: {prefix}")
    return prefix
}
```

### 2. Import and Use the Annotation

```flame
// src/main.fm
import annotation.logger

@Logger(prefix: "flame-cli")
fn main() {
    print("Application running!")
}
```

When `main()` is invoked, the Flame runtime dynamically looks up the `Logger`
annotation function, evaluates its arguments, and executes the annotation logic
before running the target function.

---

## Native Plugins & `flame-macro`

Flame allows seamless integration with native Rust crates using `flame-macro`.

### 1. In your native crate's `Cargo.toml`:

```toml
[dependencies]
flame-macro = { path = "../../flame-macro" }
axum = "0.7"
tokio = { version = "1.0", features = ["full"] }
```

### 2. Annotate Rust structs and methods:

```rust
use flame_macro::flame;

pub struct FlameServer {
    router: axum::Router,
}

impl FlameServer {
    #[flame(daemon)]
    pub async fn listen(self, port: u16) -> std::io::Result<()> {
        let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, self.router).await.map_err(std::io::Error::other)
    }
}
```

### Available `flame` attributes:

- `#[flame(daemon)]` / `#[flame(runtime)]`: Keeps the runtime event loop alive
  for long-running servers and background tasks.
- `#[flame(constructor)]`: Exposes a constructor method for creating struct
  instances.
- `#[flame(skip)]`: Omits internal methods from the generated `.fmi` interface.
- `#[flame(rename = "custom_name")]`: Renames the symbol exported to Flame.

---

## IDE Extension (`flame-ide`) Integration

The `flame-ide` extension communicates with the `flame` binary using JSON mode:

- `flame check <file> --json --line <L> --col <C>`: Returns diagnostics,
  autocomplete items, and hover information for the exact cursor position.
- `flame format <file> --stdout`: Formats Flame code and outputs the result.

When modifying IDE features, update:

- `flame-ide/extension.js`: VS Code extension entry point (CommonJS).
- `src/main.rs`: `analyze_file_for_json` and `run_check_command`.
- `src/ide.rs`: Completion items, keyword lists, and struct member extractors.

---

## Running Tests & Validating Changes

Run all automated test suites across the workspace:

```bash
# Run core test suite
cargo test

# Check a Flame file via the CLI
cargo run -- check examples/src/main.fm --json

# Build the example project with native plugins
cargo run --manifest-path Cargo.toml -- build --manifest-path examples/Cargo.toml
```
