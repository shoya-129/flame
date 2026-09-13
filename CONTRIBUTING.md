# Contributing to Flame

Welcome to the Flame programming language! We are excited to have you contribute. This guide will walk you through the codebase architecture, how the modular components interact, where everything is located, and step-by-step instructions on how to add features, write tests, and submit contributions.

---

## Table of Contents

1. [Architecture & Codebase Tour](#architecture--codebase-tour)
   - [Where Everything Lives](#where-everything-lives)
   - [Detailed Component Breakdown](#detailed-component-breakdown)
2. [Getting Started as a Contributor](#getting-started-as-a-contributor)
   - [Prerequisites](#prerequisites)
   - [Building and Running Locally](#building-and-running-locally)
   - [Running the Test Suite](#running-the-test-suite)
3. [How to Add Features: Step-by-Step Guides](#how-to-add-features-step-by-step-guides)
   - [Guide 1: Adding a New Syntax Element or Keyword](#guide-1-adding-a-new-syntax-element-or-keyword)
   - [Guide 2: Adding a New Standard Library (`std`) Module](#guide-2-adding-a-new-standard-library-std-module)
   - [Guide 3: Adding Built-in Annotations](#guide-3-adding-built-in-annotations)
   - [Guide 4: Adding or Modifying a CLI Subcommand](#guide-4-adding-or-modifying-a-cli-subcommand)
   - [Guide 5: Adding Common Utility Functions](#guide-5-adding-common-utility-functions)
4. [Testing & Quality Standards](#testing--quality-standards)
5. [Submitting a Pull Request](#submitting-a-pull-request)

---

## Architecture & Codebase Tour

Flame is written in Rust (2024 edition). The repository is structured into modular, component-based directories so that each phase of the language pipeline has clear boundaries and responsibilities.

### Where Everything Lives

flame/
├── Cargo.toml               # Package configuration and feature flags
├── src/
│   ├── lib.rs               # Library root re-exporting core modules
│   ├── main.rs              # CLI entry point and top-level argument dispatcher
│   ├── lexer.rs             # Lexical analysis: token scanner and span tracking
│   ├── parser/              # Syntax analysis: AST definitions and recursive-descent parser
│   ├── typechecker/         # Semantic analysis: static typing, scopes, inference
│   ├── runner/              # Tree-walking runtime: expression evaluator, statement execution
│   ├── vm.rs                # Runtime value models (Value, Env, CValue, FFI)
│   ├── stdlib.rs            # Built-in global functions and stdlib loader
│   ├── native_std/          # Native Rust standard library implementations (fs, net, math, etc.)
│   ├── package_manager/     # Package manager: dependency resolution, downloading, FMI generation
│   ├── ide/                 # IDE intelligence: keyword documentation, definition finder, tokens
│   ├── cli/                 # CLI subcommand implementations and JSON language server engine
│   ├── utils/               # Centralized shared utilities (manifests, formatting, fs, text)
│   ├── compiler.rs          # AOT native binary compilation pipeline
│   ├── blaze.rs             # Blaze build system integration
│   ├── embedded/            # Bare-metal microcontroller code generation and flashing
│   ├── formatter.rs         # Flame code formatter
│   └── test_engine.rs       # Integrated test harness execution engine
├── macro/                   # Procedural macros for native plugins (#[flame_export], etc.)
├── ide/                     # Visual Studio Code extension
└── examples/                # Example Flame programs, CLI tools, and plugins

---

### Detailed Component Breakdown

#### 1. `src/parser/` — Syntax & AST

- **`ast.rs`**: Definitions for all Abstract Syntax Tree (AST) nodes: `Expr`, `Stmt`, `BinaryOp`, `UnaryOp`, `LiteralValue`, `Param`, `Annotation`, `MatchArm`, and span methods.
- **`parser.rs`**: Recursive-descent parser that consumes a token stream and produces AST statements (`Vec<Stmt>`).
- **`utils.rs`**: Indentation stripping (`strip_common_indentation`), test annotation detection, and platform conditional filtering (`filter_platform_stmts`).
- **`mod.rs`**: Re-exports all AST nodes and parser interfaces.

#### 2. `src/typechecker/` — Semantic Analysis & Type System

- **`types.rs`**: Core type definitions (`Type`, `VarInfo`, `ParamInfo`, `FunctionSig`, `StructInfo`, `EnumInfo`, `CommandInfo`).
- **`checker.rs`**: `TypeChecker` struct, `new()`, `check_program()`, hover information registration.
- **`builtins.rs`**: Registration of built-in type methods (String, Array, Map, Num, Int, etc.) and `get_std_module_type()`.
- **`stmts.rs`**: Top-level declaration collection and statement type checking (`check_stmt()`).
- **`exprs.rs`**: Expression type inference (`infer_expr_type()`, `infer_binary_type()`, calls, indexing, member access).
- **`helpers.rs`**: Type compatibility rules (`is_compatible()`), assignment validation, type formatting, and scope management (`push_scope()`, `pop_scope()`, `define_var()`).
- **`tests.rs`**: Unit tests for typechecker rules.

#### 3. `src/runner/` — Execution Engine & Runtime

- **`core.rs`**: `Runner` struct initialization (`new()`), execution entry point (`run()`), and thread cloning.
- **`callbacks.rs`**: Asynchronous callback queue processing and native callback invocation.
- **`stmts.rs`**: Statement execution (`execute_statement()` for loops, conditionals, assignments, match).
- **`exprs.rs`**: Expression evaluator (`eval_expr()`) handling literals, operators, function calls, closures, formula maps, etc.
- **`target.rs`**: Reference path resolution, mutation target lookup, and write-back semantics.
- **`plugins.rs`**: Dynamic plugin loading, native C ABI method calling, CLI argument binding, and dispatch.
- **`tests.rs`**: Runtime execution unit tests.

#### 4. `src/package_manager/` — Package Management & FMI

- **`meta.rs`**: Flame metadata structures (`FlameMeta`, `FlameFunctionMeta`, `FlameStructMeta`, `PluginSpec`).
- **`manifest.rs`**: `flame.toml` plugin entry parser and plugin listing.
- **`downloader.rs`**: Package archive streaming downloader with terminal progress bar.
- **`ops.rs`**: High-level package commands (`add_package`, `remove_package`, `install_all_packages`, `ensure_dependencies_installed`).
- **`native.rs`**: Native plugin inspection, building dependency crates, and `.fmi` interface generation.
- **`rustdoc.rs`**: `syn` AST parser and `rustdoc` JSON inspector to extract exported signatures from Rust crates.

#### 5. `src/ide/` — Language Intelligence

- **`keywords.rs`**: Keyword table with hover documentation, keyword completions, and literal autocompletions.
- **`scanner.rs`**: Fast document scanning for variables, types, functions, and import aliases.
- **`modules.rs`**: Standard library method discovery and native module definitions.
- **`semantic_tokens.rs`**: Semantic token extraction for syntax highlighting.
- **`definition.rs`**: Symbol definition locator across source files, Blaze modules, and standard libraries.

#### 6. `src/cli/` — Command Line Interface

- **`commands/build.rs`**: `flame build` implementation, snapshot tracking, and runtime rebuild checks.
- **`commands/run.rs`**: `flame run` and `flame run --watch` file execution.
- **`commands/test.rs`**: `flame test` test discovery, filtering, and test execution.
- **`commands/project.rs`**: `flame new`, `flame native init`, `flame package`, `flame flash`, `flame monitor`.
- **`commands/system.rs`**: `flame doctor`, `flame update`, `flame uninstall`.
- **`commands/help.rs`**: CLI help menu and usage text.
- **`ide.rs`**: LSP/JSON backend (`flame check --json`, `flame definition --json`, `analyze_file_for_json`).

#### 7. `src/utils/` — Shared Utilities

- **`manifest.rs`**: `find_manifest_root()`, `parse_manifest_section()`, `parse_manifest_permissions()`.
- **`format.rs`**: `format_byte_size()`, `format_transfer_speed()`, `clean_table_borders()`.
- **`fs.rs`**: `copy_dir_all()`.
- **`text.rs`**: `strip_comments_and_strings()`, `extract_balanced_block()`.

---

## Getting Started as a Contributor

### Prerequisites

- [Rust](https://rustup.rs/) (version 1.80+ or latest stable)
- `git`

### Building and Running Locally

Clone the repository:

```bash
git clone https://github.com/shoya-129/flame.git
cd flame
```

Compile the project:

```bash
cargo check
cargo build
```

Run the Flame CLI:

```bash
cargo run -- --help
cargo run -- doctor
cargo run -- run examples/src/main.fm
```

### Running the Test Suite

Always run the full test suite before committing:

```bash
cargo test
```

You can also run tests for a specific module:

```bash
cargo test typechecker
cargo test runner
cargo test ide
```

---

## How to Add Features: Step-by-Step Guides

### Guide 1: Adding a New Syntax Element or Keyword

When adding a new keyword or syntax construct (e.g. a `repeat` loop, a `typeof` operator, or a new binary operator):

1. **Tokenize in `src/lexer.rs`**:
   - Add the new token kind to `TokenKind` (e.g., `TokenKind::Repeat`).
   - In `Lexer::next_token()`, recognize the symbol or add it to the keyword lookup table.

2. **Define AST Node in `src/parser/ast.rs`**:
   - Add a new variant to `Expr` or `Stmt` (e.g., `Stmt::RepeatStmt { count: Expr, body: Vec<Stmt>, span: Span }`).
   - Implement the `span()` method for the new variant.

3. **Parse in `src/parser/parser.rs`**:
   - In `Parser::parse_statement()` or `Parser::parse_expression()`, add parsing logic when encountering the token.
   - Emit clear diagnostics with `Diagnostic::error()` if syntax is invalid.

4. **Typecheck in `src/typechecker/`**:
   - If it is an expression: add a handler in `src/typechecker/exprs.rs` (`infer_expr_type`).
   - If it is a statement: add a handler in `src/typechecker/stmts.rs` (`check_stmt`).
   - Enforce type constraints using `self.expect_assignable(...)` or `self.is_compatible(...)`.

5. **Execute in `src/runner/`**:
   - If expression: handle in `src/runner/exprs.rs` (`eval_expr`).
   - If statement: handle in `src/runner/stmts.rs` (`execute_statement`).

6. **IDE Support in `src/ide/keywords.rs`**:
   - Add the keyword, description, and code snippet to `const KEYWORDS` so users get completions and hover help.

7. **Add Tests**:
   - Add a typechecking test in `src/typechecker/tests.rs`.
   - Add a runtime test in `src/runner/tests.rs`.

---

### Guide 2: Adding a New Standard Library (`std`) Module

To add a new standard library module (e.g., `std.crypto` or `std.archive`):

1. **Create the native module in `src/native_std/<module_name>.rs`**:

   ```rust
   use crate::vm::Value;
   use std::collections::HashMap;

   pub fn init() -> HashMap<String, Value> {
       let mut m = HashMap::new();
       m.insert(
           "hash".to_string(),
           Value::NativeCallback(|args| {
               let input = match args.get(0) {
                   Some(Value::String(s)) => s,
                   _ => return Err("Expected string argument".to_string()),
               };
               // Implementation...
               Ok(Value::String(format!("hashed_{}", input)))
           }),
       );
       m
   }
   ```

2. **Register module in `src/native_std/mod.rs`**:
   - Add `pub mod <module_name>;`.
   - Update `get_module_defs()` if exposing metadata.

3. **Register in `src/stdlib.rs`**:
   - Add module mapping in `register_std_module()`:

     ```rust
     "std.<module_name>" => Some(crate::native_std::<module_name>::init()),
     ```

4. **Register types in `src/typechecker/builtins.rs`**:
   - In `get_std_module_type()`, define the module's exported functions, parameters, and return types.

5. **Register in IDE Support (`src/ide/modules.rs` & `src/cli/ide.rs`)**:
   - Add the module to `list_std_modules()` in `src/cli/ide.rs`.
   - Add method suggestions in `get_std_module_methods()` in `src/ide/modules.rs`.

---

### Guide 3: Adding Built-in Annotations

Annotations provide metadata for functions, structs, or CLI commands (e.g., `@Permission`, `@Test`, `@Platform`):

1. **Define / Recognize Annotation**:
   - If special handling is needed during parsing, check `src/parser/utils.rs` (`is_test_annotation`, etc.).
   - Annotation structure is `Annotation { name, args, span, name_span }` in `src/parser/ast.rs`.

2. **Validate in `src/typechecker/checker.rs` or `stmts.rs`**:
   - In `TypeChecker::process_annotations()`, check annotation arguments and enforce constraints.

3. **Runtime Hook in `src/runner/`**:
   - For execution lifecycle: handle in `src/runner/stmts.rs` or `src/runner/callbacks.rs`.
   - Read arguments via `anno.args`.

4. **Document in `src/ide/keywords.rs`**:
   - Add documentation block in `KEYWORDS` (e.g. `"@MyAnnotation"`, `"markdown documentation and example"`).

---

### Guide 4: Adding or Modifying a CLI Subcommand

Flame's CLI dispatcher is in `src/main.rs`, and commands are implemented in `src/cli/commands/`:

1. **Create/Update command file in `src/cli/commands/<command>.rs`**:

   ```rust
   pub fn run_my_command(args: &[String]) {
       println!("Executing custom command!");
   }
   ```

2. **Export in `src/cli/commands/mod.rs`**:

   ```rust
   pub mod my_command;
   pub use my_command::*;
   ```

3. **Add dispatch in `src/main.rs`**:

   ```rust
   "my-command" => {
       run_my_command(&args[2..]);
   }
   ```

4. **Update Help Menu in `src/cli/commands/help.rs`**:
   - Add command usage and description to `print_help()`.

---

### Guide 5: Adding Common Utility Functions

To keep the codebase DRY and component-friendly:

- Do **not** duplicate file discovery, TOML parsing, string formatting, or directory copying across modules.
- Place shared functions in `src/utils/`:
  - `src/utils/manifest.rs` for project root detection and TOML section reading.
  - `src/utils/format.rs` for string, speed, byte, and markdown table formatters.
  - `src/utils/fs.rs` for recursive file operations.
  - `src/utils/text.rs` for code string parsing and balanced bracket extractors.
- Re-export them in `src/utils/mod.rs` so any module can access them via `crate::utils::*`.

---

## Testing & Quality Standards

1. **Zero Compiler Warnings**: All code should compile cleanly under `cargo check`.
2. **Preserve Compatibility**: Public APIs in `mod.rs` files should maintain backward compatibility.
3. **Write Unit Tests**: For any new language feature or bug fix:
   - Add parser/typechecker test cases in `src/typechecker/tests.rs`.
   - Add execution test cases in `src/runner/tests.rs`.
4. **Run Regression Tests**:

   ```bash
   cargo test
   ```

---

## Submitting a Pull Request

1. **Fork & Branch**: Create a feature branch with a descriptive name (`git checkout -b feature/my-feature`).
2. **Verify Locally**:

   ```bash
   cargo check
   cargo test
   ```

3. **Commit Messages**: Use clear, conventional commit messages (e.g., `feat: add repeat loop syntax`, `fix: correct member context extraction in IDE`).
4. **Open PR**: Submit your pull request to the `main` branch with a description of the problem solved and test coverage added.

Thank you for helping build Flame! 🔥
