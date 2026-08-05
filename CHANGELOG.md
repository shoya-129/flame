# Flame Toolchain & Language Changelog

All pre-release versions in the `0.x.x` series carry the official codename **Flame Spark**, reflecting the fast, evolving, and multithreaded foundation of the language toolchain. Upon reaching the stable `1.0.0` milestone, Flame will transition to its canonical **Final Spark** release codename.

---

## [0.1.5] - 2026-08-05 (Codename: *Flame Spark*) 

### 🧩 Language Grammar, Syntax & LSP Features
- **Native Rust Import Syntax**: Introduced `import native.<name>` to allow importing native Rust plugins in Flame code, enabling IDE hover text and autocompletion that distinguish plugins from regular modules.
- **Comprehensive Built-in Documentation**: Expanded the Flame Language Server Protocol (LSP) to automatically generate structured markdown documentation hover text for all core language keywords, built-in functions (`print`, `eprint`, `thread`), and standard library modules.
- **Enhanced Rust Type Display**: Updated the `check --line` command to display rich Rust type information for function parameters and return types (e.g., `std::vec::Vec<T>`, `u32`) for improved IDE type-hover accuracy.

### 🧪 Testing & Build System Enhancements
- **Automatic Test Discovery**: Integrated `flame test` to recursively scan `tests/`, `src/`, and workspace target folders for test files.
- **Testing Lifecycle Annotations**: Added `@BeforeAll`, `@AfterAll`, `@Setup`, `@Cleanup`, and `@Test` (with `timeout`, `skip`, `only`, `tags`, `Parameterized` options) to control test execution flow and filtering.
- **Zero-Cost Production Stripping**: Enabled automatic stripping of all test code, annotations, and benchmarks during production builds (`flame build --release`) to ensure zero runtime overhead.
- **Native Plugin Build Scoping**: Extended `flame native init` to support specifying custom plugin names via `--name` and refined build logic to automatically pass `--release` flags for native plugins, ensuring release builds use optimized Rust profiles.

---

## [0.1.4] - 2026-08-01 (Codename: *Flame Spark*) 

### 🧪 Native Testing & Lifecycle Framework (`flame test`)
- **Integrated Test Discovery Engine**: Built a zero-configuration test engine (`flame test`) with recursive folder scanning across workspace targets, `tests/`, and `src/` directories.
- **Structured Lifecycle Annotations**: Introduced built-in PascalCase testing decorators with deterministic lifecycle orchestration:
  - **`@BeforeAll` / `@AfterAll`**: Executes global initialization (e.g., database connection pooling) and teardown exactly once per test suite.
  - **`@Setup` / `@Cleanup`**: Automatically runs state reset and cleanup logic before and after every individual test function.
  - **`@Test` Configuration Parameters**: Advanced execution modifiers including millisecond execution timeouts (`@Test(timeout: 3000)`), skipping (`@Test(skip: true)` or `@Ignore`), isolation (`@Test(only: true)` or `@Only`), categorical filtering (`tags: ["db", "auth"]`), and multi-parameter unrolling via `@Parameterized`.

### 🚀 Zero-Cost Production Stripping
- **Automatic Dead-Code Elimination**: During production compilation (`flame build --release` and standard `flame run`), all test suites, lifecycle routines, and benchmark decorators (`@Test`, `@Setup`, `@Cleanup`, etc.) are automatically stripped from abstract syntax trees, ensuring zero runtime memory overhead and negligible release executable sizes.

### ✨ Custom Annotations & Advanced IDE Metadata
- **`annotation` Keyword**: Introduced a dedicated keyword for declaring custom, strongly-typed metadata routines that return structured payloads (e.g., `annotation Benchmark(name: String, iterations: Int) -> Formula`).
- **Semantic Red Highlighting & Precision Hover**: IDE language integrations now render built-in test decorators and custom annotation identifiers in distinct semantic red. Hovering displays complete function signatures including exact parameter names and data types (e.g., `fn about(name: String) -> String`) accompanied by detailed inline markdown documentation.

### 🔧 Universal Explicit Type Conversions & Syntax Enhancements
- **Explicit Conversion Methods**: Added robust universal methods across primitives, strings, and composite collections without implicit casting surprises:
  - **Numeric & Boolean Parsing**: `.toInt(radix: Int)`, `.tryInt(radix: Int)`, `.toFloat()`, `.tryFloat()`, `.toBool()`, and `.tryBool()`.
  - **Formatting & Byte Arrays**: `.toString(precision: Int)` (supporting exact floating-point decimal truncation such as `3.14159.toString(2)` -> `"3.14"`), `.toChar()`, and `.toBytes()`.
- **Inequality Operator (`!=`)**: Fully integrated `!=` (`TokenKind::ExclamationEqual`) across the compiler lexer, parser, AST evaluation engine, and whitespace formatter.
- **Syntax Documentation**: Updated [docs/syntax.md](./docs/syntax.md) with dedicated guides on Explicit Type Conversion Methods and the Annotated Functions Testing Framework.

---

## [0.1.3] - 2026-08-01 (Codename: *Flame Spark*)

### ⚡ Dramatic Build & Compiler Speed Optimizations
- **Dependency Graph Trimming**: Completely removed unneeded `tokio` dependencies from the core `flamelang` compiler executable and stripped expensive unused procedural macro features (`extra-traits`, `fold`, `visit`) from `syn`, cutting initial toolchain installation times by over half.
- **Optimized Dev Profiles**: Added `split-debuginfo = "unpacked"` for Windows debugging efficiency and set `[profile.dev.package."*"] opt-level = 2`, ensuring external third-party crates run at high speeds without impeding instant incremental compiler builds.

### 🚀 Production AOT Release Optimization (`flame build --release`)
- **Production Release Profiles**: Standalone executables generated via `flame build --release` and native plugins created via `flame native init` now inherit aggressive compiler optimizations:
  - **`opt-level = 3`**: Peak runtime CPU throughput and execution speed.
  - **`lto = "fat"`**: Full cross-crate Link Time Optimization across Flame's execution engine, standard library bridges, and native plugins for zero-overhead function inlining.
  - **`codegen-units = 1`**: Maximizes compiler optimization visibility across all translation units.
  - **`strip = true`**: Automatically strips debug symbol tables for compact executable footprints.
  - **`panic = "abort"`**: Removes stack-unwinding landing pads to minimize binary overhead.
- **Release Plugin Compilation**: Updated the dependency builder (`ensure_dependencies_installed`) to automatically pass `--release` flags when building standard library bridges (`std_bridge`) and local native plugins during production builds.

### 🔌 Intelligent CLI Package & Plugin Management
- **Auto-Discovery of Plugin Names**: When invoking `flame add --plugin <path>` (or `@plugin <path>`), if `--name <plugin_name>` is omitted, Flame automatically inspects `<path>/Cargo.toml` to read the package name or extracts the fallback directory folder name.
- **Customizable Native Workspace Init**: `flame native init [plugin_name]` now enables specifying custom names directly from the command line, generating properly scoped `native/Cargo.toml` files and registering the plugin cleanly in `flame.toml`.

### 🧠 Advanced IDE & LSP Integration (`check --json`)
- **Plugin vs. Module Distinction**: Refactored the semantic type checker to explicitly categorize native AOT Rust imports (`import native.<name>`) as `plugin` rather than `module`.
- **Precision Hover Metadata**: When hovering over native plugin structures in VS Code or querying via `--line N --col N`, the language server outputs exact `server: plugin` tagging alongside method parameters, return types, and struct signatures extracted directly from compiled `.fmi` interface files.
- **Intellisense Completion Autocomplete**: Autocompletion items generate cleanly with `kind: "plugin"` and `detail: "native plugin"`.

### 📚 Architecture & Concurrency Documentation
- **Asynchronous Concurrency Guide ([docs/async.md](./docs/async.md))**: Authored a comprehensive guide detailing Tokio multi-threaded worker pools, zero-polling reactive wakeup mechanics, and comprehensive solutions to the "Missing Await" architectural pitfall across network HTTP clients, server listeners, and CPU loops.
- **Multithreaded Execution Guide ([docs/threads.md](./docs/threads.md))**: Integrated a visual multithreading architectural diagram (`flame_multithreading_diagram.png`) and expanded explanations of lexical atomic environment snapshot isolation (`Arc<Env>`) during parallel compute thread execution (`thread { ... }`).

---

## [0.1.2] - 2026-07-31 (Codename: *Flame Spark*)

### Added
- Standard library native Rust bridge export macros (`#[flame_export]`).
- Automated code formatting engine (`flame format <file.fm>` and `--all`) with comment syntax preservation and whitespace standardization around operator dot notations (e.g. `thread.sleep()`).
- Multi-threaded daemon lock persistence for network server sockets (e.g. Axum/Tokio HTTP servers) to keep processes active until explicit user interrupts (`Ctrl + C`).
