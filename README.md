<h1 style="display:flex; align-items: center; gap: 8px;">
  <img src="./docs/public/flame.png" alt="Flame Logo" width="46" height="46" style="vertical-align: middle;" />
  Flame
</h1>

Flame is a statically typed, fast, and modern programming language built in Rust. It serves as a comprehensive toolchain containing a compiler, an ahead-of-time (AOT) builder, automated code formatting, and zero-overhead native FFI bridging capabilities.

---

## Why Flame?
Flame is designed to sit between high-level developer experience and
native systems programming.

Write simple Flame code while being able to use native Rust crates,
native plugins, hardware APIs, and compile everything into a standalone
native executable.

---

## Architecture: True Multithreading & High Concurrency
Flame avoids the slow, bottlenecked single-threaded event loops common in legacy runtimes like Node.js:
- **True Multithreaded Execution**: Powered by Rust's native OS threads and multi-core **Tokio worker pools**, background computation and network tasks execute concurrently across all available CPU cores.
- **Execution vs. Concurrency Separation**: Instead of expensively cloning entire VM interpreter states per network request (which leads to state divergence and memory overhead), parallel worker pools package incoming requests and communicate over atomic message queues to a single deterministic Flame runtime engine.
- **Persistent Server Daemons**: The Flame runtime automatically engages multi-threaded daemon persistence whenever network bindings or async listeners (like Axum/Tokio) are active, keeping your server running responsive without arbitrary loops.

---

## Core Features
- **Ahead-of-Time (AOT) Compilation**: Directly compile Flame scripts into ultra-fast, statically linked executables with zero external library overhead.
- **Native Dependencies & Local Plugins**: Add public Rust crates or build custom local Rust libraries (`./native`) directly into your project. Flame automatically inspects Rust interfaces to export struct types and signatures to JSON `.fmi` bindings!
- **World-Class IDE Intellisense**: The Flame VS Code Extension uses native `.fmi` definitions to provide rich type hover info, doc comments, and method autocompletion for standard and local native plugins.
- **Lightweight Package Manager**: Built-in Golang-style dependency management capable of resolving packages over HTTP without requiring `git`.

---

## Installation
Install the Flame toolchain directly using Cargo:

```bash
cargo install --force flamelang
```
or using npm

```bash
npm i -g flamelang
```
---

## CLI & Toolchain Reference

The `flame` (or `flamelang`) binary provides an all-in-one developer workspace toolkit:

### 1. Running & Building Code
- **`flame run <file.fm> [--local]`**: Run a Flame script directly using the interactive compiler engine. When `--local` is specified, local plugins and static native bridges are compiled and executed in real time.
- **`flame build [entry.fm]`**: Compile your Flame program and its native Rust bridges into a standalone, statically linked dev executable inside `target/dev/`.
- **`flame build --release`**: Produces an optimized, production-grade standalone executable inside `target/release/`. Configures LLVM for maximum performance with full optimization (`opt-level = 3`), fat Link Time Optimization (`lto = "fat"`), single code generation unit across all crates (`codegen-units = 1`), stripped symbol tables (`strip = true`), and zero-overhead aborting panics (`panic = "abort"`). All native plugins and standard bridges are simultaneously compiled with `--release` flags!
- **`flame <file.fm>`**: Quick-exec shorthand to parse and run any `.fm` source file.

### 2. Diagnostics & IDE Integration (`check --json`)
- **`flame check <file.fm> [--json] [--line N --col N]`**: Performs instantaneous syntactic analysis, type inference, and static diagnostics without compiling or linking binaries.
- **Structured JSON Output**: When invoked with `--json`, it emits a comprehensive machine-readable payload used directly by the Flame VS Code Extension and Language Server Protocol (LSP):
  - **Precision Hover Metadata**: Passing `--line N --col N` instructs the type checker to resolve the exact AST node or symbol under the cursor. It reports inferred primitive types, struct definitions from native `.fmi` bridges, and identifies native AOT packages as `plugin` (e.g., `server: plugin`) rather than regular standard library modules.
  - **Intellisense & Autocomplete**: Extracts available methods, parameters, docstrings, and struct signatures from both standard libraries (`std.*`) and compiled native plugin interfaces (`native.*`).
  - **Real-time Diagnostics**: Returns syntax errors, undefined references, and type mismatches with exact line, column, and severity mappings.

**Example Usage & JSON Payload:**
```bash
flame check automation/src/main.fm --json --line 47 --col 18
```

```json
{
  "file": "automation/src/main.fm",
  "diagnostics": [],
  "std_modules": ["thread", "process", "fs", "math", "time", "os", "env"],
  "native_modules": ["server"],
  "plugins": [
    {
      "name": "server",
      "source": "./native",
      "version": null,
      "is_local": true
    }
  ],
  "completions": [],
  "hover": {
    "label": "server",
    "documentation": "```flame\nserver: plugin\n```\nInferred type from AST"
  }
}
```

### 3. Code Formatting
- **`flame format <file.fm>`**: Automatically reformats your source code to adhere to Flame's canonical formatting rules (clean indentation, comment preservation, and exact operator spacing without extraneous padding around dot notation or module accesses like `thread.sleep()`).
- **`flame format --all`** (or `flame format` directly in a workspace): Formats all `.fm` and `.flame` source files across your repository.

### 4. Package & Plugin Management
- **`flame new <name>`**: Initialize a new Flame workspace directory with a ready-to-run manifest and directory structure.
- **`flame add <url_or_name>`**: Download and register external Flame modules or repositories.
- **`flame add --plugin <path_to_plugin> [--name <plugin_name>]`**: Register a local or external native Rust plugin inside your `flame.toml` manifest. If `--name` is omitted, Flame automatically discovers the plugin name by inspecting its `Cargo.toml` or extracting the directory name!
- **`flame native init [plugin_name]`**: Initialize a native Rust plugin workspace (`./native`) inside your current Flame project, generating a starter `Cargo.toml`, bridge code, and automatically registering the specified plugin name in your `flame.toml`.

---

## Documentation
Dive into the `docs` folder for detailed guides on language syntax, multithreaded ownership models, native crate integration, and plugin design:
- [Multithreaded Architecture & Safety](https://flamelang.vercel.app/concurrency/threads-and-channels/)
- [Asynchronous Concurrency & Network I/O](https://flamelang.vercel.app/concurrency/async-await/)
- [Developing Local Plugins & FFI](https://flamelang.vercel.app/packages-and-native/native-plugins/)
- [Using Native Rust Crates](https://flamelang.vercel.app/packages-and-native/native-rust-crates/)
- [Changelog & Version Codenames](./CHANGELOG.md)
  
## License
ISC
