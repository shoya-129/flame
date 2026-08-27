# Flame Toolchain & Language Changelog

All pre-release versions in the `0.x.x` series carry the official codename **Flame Spark**, reflecting the fast, evolving, and multithreaded foundation of the language toolchain. Upon reaching the stable `1.0.0` milestone, Flame will transition to its canonical **Final Spark** release codename.
---

## [0.3.9] - 2026-08-27 (Codename: *Third Spark*)

### ✨ Typechecker Enhancements & Bug Fixes
- **Auto-Referencing**: Added robust support for implicit references in method calls (e.g., safely passing a `Vector2` where a `&Vector2` is expected without strict syntax errors).
- **Enum Upcasting & Equality**: Fixed initialization of variables with Enum variants so they seamlessly upcast to the parent Enum. Added rules enabling variants to be accurately compared via `==`.
- **Closure Coercion**: Allowed empty tuple `()` arguments to automatically match empty closure types (`() -> Unknown`), restoring idiomatic callback shorthand for event handlers.
- **IDE Hover Resilience**: Repaired a glitch where the IDE's tooltip hover documentation would abruptly stop displaying midway through files. Variables now resist cascading into `Unknown` types on minor reference mismatches, retaining their strong typing and inline documentation throughout the entire execution context.

---
## [0.3.8] - 2026-08-26 (Codename: *Third Spark*)

### ✨ New Features & Enhancements
- **Automatic Structural Naming**: Rust structs used in FMI plugins now automatically inherit their `flame_name` from the Rust struct's identifier, eliminating the need for manual `flame_name` attributes in metadata.
- **Enhanced Test Engine**: Upgraded the test engine to support structural method invocation on native types, enabling AOT testing for complex plugin architectures.
- **Improved Debug Output**: Enhanced the test engine with structured logging for plugin loading and method binding, providing clearer diagnostics during development.

### 🐛 Bug Fixes
- **Native Plugin Integration Fix**: Resolved issues preventing native plugin methods from being discovered and bound by the AOT compiler, specifically addressing cases where `flame_name` attributes were missing.
- **Test Execution Stability**: Eliminated engine panics caused by unlinked native libraries by implementing proper linking in the test runner and AOT generation pipeline.

---

## [0.3.7] - 2026-08-25 (Codename: *Third Spark*)

### ✨ New Features & Fixes
- **Flame Binder API (`flame-binder`)**: Introduced a dedicated Rust crate that allows developers to seamlessly embed the Flame VM in their Rust projects. Developers can load Flame scripts and execute exported Flame functions directly from Rust using `Binder::load("path.fm")` and `binder.call("func", args)`.
- **Complete AOT VFS Isolation**: The AOT compiler now strictly bundles only source code files (`.fm`, `.flame`, and `.fmi`). The runtime VM (`runner.rs`) has been updated to bypass physical filesystem fallback when executing a compiled binary, ensuring that `.env` files, `tests/` directories, and other host files are completely isolated and not accidentally accessed by compiled applications.
- **Documentation Updates**: Added Google site verification metadata to the online docs and clarified that the package directory name must match the package identifier for VFS module resolution.

---

## [0.3.6] - 2026-08-24 (Codename: *Third Spark*)

### ✨ New Features & IDE Enhancements
- **Custom `@Suggestion` Autocompletion**: The IDE now supports defining custom autocomplete suggestions natively through `@Suggestion` annotations. By passing arrays of metadata such as `@Suggestion([["TestObj", "object"], ["hello", "function"]])`, packages can explicitly inject methods, objects, and configurations directly into the Language Server dropdown matrix.
- **`package` Declaration Keyword**: Introduced the formal `package` keyword modifier for delineating module boundary roots. This explicit syntax clarifies scoping semantics and triggers pristine `import package <name>` documentation blocks directly in the IDE hover window.
- **`import main` Auto-Resolution**: The top-level `src/` directory is now implicitly treated as the `main` package namespace. Files outside of `src/` (such as those in `tests/`) can securely execute `import main` to aggregate all exported symbols from the `src/` folder natively without forcing repetitive `package main` definitions per file.

### 🐛 Bug Fixes
- **Formula Return Bug**: Resolved a parser evaluation glitch where `formula { ... }` blocks failed to correctly route returned values or terminated prematurely during AST unwinding. Formulas now behave identically to native objects when mapping complex nested responses.
- **IDE Hover Isolation**: Corrected a flaw inside the Language Server where internal annotations (e.g., `@Suggestions`) and unrecognized meta-tags incorrectly bled into the global Markdown hover tooltips for decorated functions and packages. Hover strings are now pristine and specifically filtered.
- **VFS Runtime Prioritization**: Fixed a bug in `runner.rs` where runtime release builds were prioritizing the physical file system over the embedded Virtual File System (VFS), preventing packed applications from executing reliably if source files were modified or deleted after compilation.

---

## [0.3.5] - 2026-08-22 (Codename: *Third Spark*)

### ✨ Standard Library & Naming Convention Overhaul
- **`camelCase` Standardization**: Successfully migrated the entire Flame ecosystem to universally enforce `camelCase` naming conventions for built-in methods across all standard library modules, native plugins, and execution environments. Legacy snake_case function calls (such as `assert_eq`, `to_string`, `read_bytes`) have been officially deprecated and refactored natively to `assertEq`, `toString`, `readBytes`, etc.

### 🐛 Bug Fixes & IDE Enhancements
- **Byte Array Indexing Support**: Patched a critical bug within the AST Engine (`Expr::Index`) where the interpreter failed to support zero-based indexing (`bytes[0]`) on raw Native Byte Arrays (`Value::Bytes(Vec<u8>)`). Indexing into a byte array now safely and correctly slices out a single `Byte` value without throwing a runtime trap.
- **Time API Alignment**: Corrected legacy tests and core native bindings for `std.time`. The `time.now()` interface now strictly returns a structured `Object` natively mapped with Unix Epoch `millis` and conversion primitives (`toSeconds`, `toString`) instead of ambiguously returning a raw scalar `Int`.
- **Language Server Import Bleed Prevention**: Solved an intrusive Language Server protocol (LSP) bug within the IDE `TypeChecker` where Markdown Hover documentation (`@Docs`) would incorrectly bleed into completely unrelated variables bearing the same name across separate projects upon triggering an `import` directive. An `is_importing` context flag now forcefully isolates namespace bindings.

---

## [0.3.4] - 2026-08-22 (Codename: *Third Spark*)

### ✨ New Features & Enhancements
- **Math Standard Library (`std.math`)**: Implemented a comprehensive `math` module bound to native system intrinsics (`math.sin`, `math.cos`, `math.sqrt`, `math.abs`, `math.min`, `math.max`, `math.pi`, `math.e`, `math.inf`).
- **Array & Tuple Indexing**: Added syntactic support for zero-based array and tuple indexing directly via bracket notation (e.g., `arr[0]`, `point[1]`), eliminating the need to rely strictly on iteration and destructuring.
- **Match Statement Auto-Formatting**: `flame fmt` now flawlessly supports AST-aware code formatting for `match` statements and their deeply nested interior `{ ... }` block arms.

### 🐛 Bug Fixes & Compiler Enhancements
- **Deep Tuple Type Parsing**: Fixed a critical parser bug in `src/typechecker.rs` where the compiler blindly split on commas across nested data structures. Complex type signatures like `[(Float, Int)]` are now correctly depth-parsed without prematurely splitting on the inner tuple's comma.
- **Formatter Brace & Grouping Fixes**: Resolved bugs inside the automated code formatting engine (`flame fmt`) related to parentheses/grouping depth tracking and brace block stacks, preventing formatting failures on complex conditional layouts.

### 📚 Documentation & Benchmarking
- **Destructuring & Complex Types**: Added comprehensive documentation for Tuple Destructuring and massively complex type extraction to the official `data-types.mdx` documentation.
- **`Unknown` Type Clarification**: Formally documented the `Unknown` fallback type and its type-safety trade-offs.
- **Native Benchmark Suite**: Created a pure-execution benchmark comparison between Python 3 and Flame's Native AOT execution targets, demonstrating Flame's sub-millisecond loop overheads.

---

## [0.3.3] - 2026-08-21 (Codename: *Third Spark*)


### ✨ New Features & Enhancements
- **`.env` Auto-loading**: Native standard library integration with `dotenvy`. The `std.env` module now automatically scans the workspace root for `.env` files and silently injects them during initialization. Calls to `env.get("FOO")` will effortlessly fetch variables from local environment configurations out of the box.

### 🐛 IDE & Tooling Fixes
- **Lexer UTF-8 Span Tracking**: Resolved a severe parsing bug inside the compiler's lexical analyzer where multibyte non-ASCII characters (e.g., emojis) in string literals caused token span offsets to wildly desynchronize. Spans are now accurately tracked by byte-index instead of character-index, completely stopping the `flame format` tool from irreversibly mangling source code.

---

## [0.3.2] - 2026-08-21 (Codename: *Third Spark*)

### ✨ New Features & Enhancements
- **`std.time` Plugin Expansion**: Expanded the standard library `time` module with comprehensive natively-bound date and time utilities including `time.now()`, `time.parse()`, and `time.fromMillis()`.
- **`std.fmt` Module**: Added a new native standard library module for advanced string and text formatting operations.

### 📚 Documentation
- **Telegram Bot Tutorial**: Added a comprehensive step-by-step guide (`docs/src/content/docs/getting-started/telegram-bot.mdx`) demonstrating how to build an asynchronous, long-polling Telegram bot using Flame's `flamer` web backend and `std.net.http`.

### 🐛 Bug Fixes
- **Tokio Async Panic in HTTP**: Patched a fatal runtime error (`Cannot drop a runtime in a context where blocking is not allowed`) when executing the `std.net.http` plugin (`http.get` and `http.post`) within asynchronous environments. The blocking I/O calls have been explicitly sandboxed onto dedicated OS threads (`std::thread::spawn`), making them perfectly safe to execute inside Tokio asynchronous tasks.
- **Primitive Methods Strip Bug (AOT)**: Fixed a compiler conditional logic bug (`#[cfg(feature = "base64")]`) that unintentionally stripped out the entire native method resolution chain (`Expr::Dot`) for `Value::Int`, `Value::Float`, and `Value::Bytes`. Core intrinsic methods like `abs()`, `floor()`, `concat()`, and `toBase64()` now reliably execute in both interpreted mode and AOT test modes.
- **Time Plugin Standardization**: Harmonized `std.time` output across the language implementation. Methods like `time.now()` and `time.fromMillis()` now return a standardized `Value::Int` epoch timestamp in both AOT compiler injection and the interpreted standard library, definitively resolving `expected Int, got Object` assertion failures when chaining methods like `.year()`.
- **AOT Testing Environment Generation**: Hardened `aot_compiler.rs` to automatically discover and enforce compilation flags (like `base64`) during isolated testing, guaranteeing 1:1 runtime parity with standard execution.

---

## [0.3.1] - 2026-08-17 (Codename: *Third Spark*)

### ✨ New Features & Enhancements
- **Refined Build CLI**: Overhauled `cargo build` logs and outputs during `flame build`. Suppressed redundant rustc outputs and integrated a clean, in-place animating spinner (White -> Yellow -> Flame Pink) utilizing true terminal escape sequences for smooth cross-platform compatibility.

### 🐛 Bug Fixes
- **`@Platform` Compilation Filtering**: Resolved an issue where conditional compilation tags (e.g. `@Platform("windows")`) incorrectly resolved to `Linux` on Windows platforms. The platform filtering logic was moved natively into the parser.
- **Strict Duplicate Function Checking**: The TypeChecker now accurately catches identically named functions declared in the same scope without a disambiguating `@Platform` tag, emitting a proper compiler error instead of silently passing duplicate Rust bindings.
- **`std.camera` Image Format Fix**: Patched a fatal runtime error (`The image format Jpeg is not supported`) when pulling frames from standard hardware interfaces by correctly bootstrapping image decoding features (`jpeg`, `png`, `bmp`) into the underlying Cargo crate dependencies.

## [0.3.0] - 2026-08-16 (Codename: *Second Spark*)

### ✨ New Features & Enhancements
- **`flame doctor` Command**: Added a new CLI command `flame doctor` to diagnose environment health, check toolchain installations, standard libraries, plugins, AOT compiler status, and platform configuration.
- **Reproducible Builds (`flame.lock`)**: Implemented `flame.lock` for the AOT compiler. The lockfile is correctly mirrored to the build cache to ensure perfectly reproducible builds across environments.
- **IDE Signature Help (`->`)**: Implemented native VS Code Signature Help! When typing `(` or `,`, developers will now see an interactive parameter hint popup natively rendering function signatures (e.g. `add(a: Int, b: Int) -> Int`), resolving beautifully for workspace functions, closures, and annotations.

### ⚡ Performance Optimizations
- **`.fmi` Caching (AOT Compiler)**: Drastically reduced subsequent compilation times. `flame build` now intelligently caches `.fmi` native dependency interface files in `.flame/pkg`, completely bypassing the heavy `rustdoc --output-format json` phase once generated.

### 🐛 IDE & Tooling Fixes
- **`@Platform` Host OS Fallback**: Fixed an issue where the IDE would hide OS-specific `@Platform` functions. If no explicit target is specified (like inside the language server), the compiler now correctly defaults to the current host OS, ensuring accurate completion suggestions.
- **Annotation Autocomplete Priority**: Upgraded IDE completion ordering so workspace-declared annotations are correctly prioritized and suggested before built-ins.
- **Custom Annotation Lints**: Introduced a `TypeChecker` lint rule enforcing that all custom user annotations must start with an uppercase letter to better distinguish them from regular functions.
- **Hover Docs for Annotations**: Fixed a bug where hover documentation (`@Docs`) was failing for annotations. Hovering over `@Annotation` will now perfectly render its rich markdown documentation.
- **`features` Keyword Scope**: Corrected the IDE to restrict the `features` keyword completion strictly to the interior body of the `@Application` annotation block.
- **`std.` Submodule Autocomplete**: Enhanced language server parsing so typing `std.` correctly displays available standard library submodules (`std.net`, `std.byte`, `std.json`, etc.) instantly.
- **Runtime Error Exits**: Standardized runtime execution errors to reliably and consistently exit with `process exited with code 1`.

### 📚 Documentation & Built-in Annotations
- **`@Requires` Annotation**: Implemented lexical dependency injection. The compiler dynamically makes the specified standard module (or native plugin) visible inside the annotated function scope without globally importing it. Safely unloaded from memory after execution to optimize resource usage. IDE autocomplete now automatically suggests standard library modules and plugins when typing inside `@Requires("...`.
- **`@Permission` Annotation**: Introduced a robust runtime capability model.
  - If used on a function, the user must explicitly allow the permissions via terminal prompt or `flame.toml`, otherwise execution stops instantly.
  - If no `@Permission` is specified project-wide, execution is auto-allowed.
  - Permissions are automatically granted when used on `@Test` functions.
- **Enhanced IDE Annotations**:
  - Annotations now correctly display with the `@` prefix and the proper "annotation" icon in autocompletion, preventing them from being mistakenly suggested as variables.
  - Hovering over an `@Test` annotated function now properly highlights its test-case status in the IDE.
  - Hover text dynamically concatenates all annotations applied to a function for rich contextual documentation.

---

## [0.2.9] - 2026-08-15 (Codename: *Second Spark*)

### 🐛 IDE & Syntax Highlighting
- **Semantic Tokens Provider Override**: Fixed a severe bug in the Flame Language Server (in `src/ide.rs`) where the `annotation` keyword was mistakenly classified as an `annotation` semantic token type (mapped to a `function` fallback). It now correctly emits as a `keyword` token, restoring the standard red/pink syntax coloring across all VS Code themes.
- **TextMate Grammar Specificity**: Hardened the `flame-ide` TextMate grammar rules for the `annotation` keyword. To prevent Oniguruma regex group capturing bugs, the rule was split into two explicit, highly robust declarations for `export annotation` and `annotation`.

---

## [0.2.8] - 2026-08-15 (Codename: *Second Spark*)

### 🐛 Testing & Package Manager
- **Workspace Isolation in Test Runner**: Hardened the `flamelang test` runner (including the `--all` fallback) to strictly respect package boundaries. The test engine will no longer recursively traverse and execute `.fm` test files inside nested directories that contain their own `flame.toml` manifests. This completely prevents edge-cases where sub-packages containing Native Rust plugins were mistakenly executed in the root's interpreted mode, resulting in stubbed Native functions and false-positive assertion failures.
- **Native Plugin Typechecking Revert**: Reverted a bug where parsing experimental file-based standard libraries caused the compiler's native type inference signatures (like `assert_eq(actual: Any, expected: Any)`) to be overwritten and evaluated as struct references, generating erroneous `expected Any, found Int` type mismatch diagnostics in the IDE.

---

## [0.2.7] - 2026-08-14 (Codename: *Second Spark*)

### 🐛 Parsing & Syntax Fixes
- **Match Destructuring**: The Flame `match` expression parser has been completely overhauled to support dot-notation paths (e.g. `Result.Ok` or `Option.None`) inside pattern arms. 
- **Tuple Pattern Unpacking**: Added full parser syntax to cleanly unpack positional enum values using parenthesis `(val)`, allowing true destructuring (`Option.Some(value) => ...`).
- **Match Blocks**: You can now correctly write `{ ... }` scoped execution blocks directly inside the `match` arm expression bodies, drastically improving flow control aesthetics without causing token collisions.

### 📚 Documentation
- **Control Flow**: Added comprehensive documentation on using Enum Destructuring and `{ ... }` blocks within `match` expressions.

---

## [0.2.6] - 2026-08-14 (Codename: *Second Spark*)

### 🎨 IDE & Developer Experience
- **Rich Hover Documentation**: Upgraded the IDE Language Server's typechecker to display beautifully formatted function signatures and inject custom `@Docs(...)` markdown metadata whenever a function or standard Enum (`Result`, `Option`) is hovered in the editor.

### 🐛 Formatter Fixes
- **Generic vs Operator Heuristic**: Fixed a critical spacing bug where the Flame formatter blindly collapsed mathematical comparisons (e.g. `total>50`). Engineered a heuristic pre-pass to perfectly differentiate generic type parameters (`Result<Int, Error>`) from logical comparison operators (`a > b`), enforcing accurate spacing for both scenarios automatically.

### 📚 Documentation
- **Error Handling Details**: Added an explicit breakdown in `data-types.mdx` clarifying the architectural differences between the strict `Err` enum variant and the standardized `Error` struct, explaining why structural errors are explicitly wrapped.

### 🧪 Testing & AOT
- **Native Test Execution (`@Test`)**: `flame test` now fully supports executing user-defined `@Test` blocks via the AOT compiler engine for projects containing native dependencies, dynamically marshalling parameters and results (`Option`, `Result`, and Arrays) across the Rust/Flame FFI boundaries seamlessly.
- **Production Build Isolation**: Hardened the compiler pipeline to ensure that any code inside `@Test` functions is completely pruned and excluded from both standard interpretation and compiled binary builds (`flame run`, `flame build`), ensuring zero footprint in production deployments.

---

## [0.2.5] - 2026-08-11 (Codename: *Second Spark*)

### 🔧 Language & Standard Library Updates
- **Byte Type Unification**: Replaced the separate `Bytes` and `Byte` types with a unified singular `Byte` type. Whether you are dealing with a single byte or a byte array, the `.type()` will correctly return `"Byte"`.
- **camelCase Standardization**: Refactored all byte operations and `std.byte`/`std.fs` methods to use standard `camelCase`. Functions like `to_bytes`, `to_utf8`, `write_bytes`, and `read_bytes` have been upgraded to `.toByte()`, `.toUtf8()`, `.writeBytes()`, and `.readBytes()` respectively.
- **Hover Docs**: Implemented rich, in-editor Markdown hover documentation for the new `Byte` functions (e.g., `toByte`, `toUtf8`, `writeBytes`) directly into the IDE language server (`flamelang check`).

### 📚 Documentation
- Updated `byte.mdx` and `filesystem.mdx` on the official documentation website to reflect the new `camelCase` standard and unified `Byte` type.

---

## [0.2.4] - 2026-08-10 (Codename: *Second Spark*)

### 🐛 Bug Fixes & VM Enhancements
- **Thread Return Payload Unwrapping**: Fixed a critical AST propagation bug in `Expr::ThreadSpawn` where a `return` statement evaluated inside a `thread { ... }` block would yield a `Value::Return` wrapper instead of the underlying payload. Doing `await handle` now correctly strips the internal wrapper so properties (like `Battery.percent`) are directly accessible on structured objects.
- **Block Statement Halting**: Overhauled `Expr::Block` resolution to guarantee that `return` and `break` statements properly halt block-level loop execution and bubble the value all the way out of the evaluation chain.

### 📚 Documentation & Ecosystem Tooling
- **Timestamp Extensions (`std.time`)**: Documented Flame's native integer-based timestamp capabilities in `threading-and-time.mdx`. Exposed detailed API descriptions for `.year()`, `.month()`, `.day()`, `.addDays(days)`, and `.addHours(hours)`.
- **Sidebar Configurations**: Fixed Astro Starlight sidebar configurations to explicitly render `std.math` inside the primary left-hand navigation pane for easier standard library discoverability.

---

## [0.2.3] - 2026-08-10 (Codename: *Second Spark*)

### 🐛 Bug Fixes & AOT Enhancements
- **Filesystem Write Append Bug**: Fixed an internal state corruption in `std.fs` where invoking `.write()` on an active file instance created via `fs.open()` would serialize and dump the `HashMap` object metadata into the file payload instead of the requested string/bytes.
- **Await Synchronization Fallback**: Ensured `await` execution behaves identically to JavaScript. Calling `await` on a synchronous primitive or object strictly bypasses thread evaluation and resolves the inner value immediately without throwing an invalid thread crash.

### 📚 Documentation & Ecosystem Tooling
- **`std.math` Architecture Migration**: Decoupled math documentation globally. Abstracted `math` interfaces away from `threading-and-time.mdx` and `overview.mdx` into a brand new standalone `math.mdx` documentation page. 
- **`std.fs` IDE Support**: Pushed expansive IDE hover docs to `std.fs` methods, injecting Markdown definitions for `open`, `delete`, `mkdir`, `mkdir_all`, and `copy` directly into the typechecker memory mapping.

### 🏗️ Object Architecture & Primitives
- **Data Conversion Macros**: Injected `.toHex()`, `.toBase64()`, and `.concat()` routines universally across the virtual machine's `Expr::Call` bridge. Byte-array data representations can now be natively re-encoded, merged, and transmitted strictly off the `Value::Bytes` AST prototype without requiring external packages.

---

## [0.2.2] - 2026-08-08 (Codename: *Second Spark*)

### 🌐 Network Toolkit & Standard Library (`std.net`, `std.json`)
- **Native Async Iteration**: Upgraded the `TcpListener` to natively support asynchronous loop evaluation. Developers can now utilize `for client in listener` to blockingly stream incoming clients through Tokio without requiring manual loop abstractions.
- **HTTP Post & Auto-JSON Serialization**: Implemented `http.post(url, body)`. Dramatically improved standard library ergonomics by automatically serializing Flame `formula { ... }` blocks and structured `Object`s into optimized JSON strings when passed as payloads to HTTP APIs.

### 🎨 IDE Intellisense & Developer Experience
- **`await` Type Inference (`Promise`)**: Overhauled the internal typechecker resolution engine. The IDE language server now intelligently intercepts `await` assignment expressions and correctly resolves the intermediate type as a `Promise` in hover tooltips instead of incorrectly mapping it to the `await` token keyword.
- **Verbose Log Silencing**: Cleaned up standard output during network testing by silencing the internal `[DEBUG Expr::Call NativeCallback]` VM logs.

### 📚 Documentation & Ecosystem Tooling
- **Starlight Docs Overhaul**: Completely updated the Astro Starlight documentation (`docs/`). Added dedicated index cards for `Network & Web (std.net, std.json)`, fixed async bindings in standard library examples, and documented the new `http.post` and `formula` JSON pipeline.
- **Object vs Formula Documentation**: Added comprehensive documentation distinguishing `Object` (`{ ... }`) from `Formula` (`formula { ... }`) to clarify their distinct syntax and structural advantages.
- **Example Hardening**: Stabilized the `flame-macro` dependency paths in `examples/ex/native` plugins to reference canonical registry versions (`0.1.0`) instead of local monorepo paths, streamlining developer testing workflows.

### 🏗️ Object Architecture & Destructuring
- **Struct Instance Identity**: Re-architected `StructInit` behavior at the VM level. Struct initializations no longer collapse into `Value::Formula`. They now map to a distinct `Value::StructInstance` to strictly preserve struct identity, enabling perfect method resolution from `impl_<name>` environments.
- **Native Object Expressions**: Implemented the standalone `Expr::Object` AST node. Standard curly braces (`{ ... }`) natively parse as objects, completely separating them from the explicit `formula { ... }` syntax.
- **Object Destructuring**: Added robust destructuring support for Objects, Formulas, and StructInstances. You can now effortlessly unpack fields into local scope using `let { status, data } = obj`.

### 🎨 IDE Syntax Highlighting
- **Annotation Consistency**: Hardened the TextMate grammars in `flame-ide` and the documentation site. Both the `@` symbol and annotation names now flawlessly render as `keyword.declaration.flame` (matching `struct` and `formula`), ensuring they no longer inherit default function colors.

## [0.2.1] - 2026-08-07 (Codename: *Second Spark*)

### 🐛 Bug Fixes & AOT Compiler Enhancements
- **AOT Compiler Reqwest Linking Error (`E0433`)**: Fixed a critical compilation failure where generating an AOT binary (`flame build`) would fail with `use of unresolved module or unlinked crate reqwest`. Because AOT binaries rely on a stripped-down `flamelang` core (`default-features = false`), the compiler was still incorrectly attempting to compile the CLI package manager (`src/package_manager.rs`), which required the `reqwest` HTTP client.
- **CLI Feature Separation**: Introduced a dedicated `cli` feature flag in `Cargo.toml`. The `package_manager` and `aot_compiler` modules, along with their heavy dependencies (`reqwest`, `zip`), are now strictly isolated behind `#[cfg(feature = "cli")]`. This guarantees that AOT user binaries are incredibly lightweight and never accidentally compile the CLI toolchain internals.

---

## [0.2.0] - 2026-08-07 (Codename: *Second Spark*)

### 📦 Native Package Management & Git Resolution
- **GitHub Module Fetching**: Overhauled `flame add` to natively support remote GitHub repositories. Supplying schemas like `github.com/user/repo@v1.0.0` automatically resolves the remote, extracts the specific version/tag, and utilizes `git clone` to isolate the payload inside the `.flame/pkg/<name>` dependency cache, mirroring modern module systems like Go.
- **Distributable Package Bundling**: The `flame build` compiler command now acts as a dedicated bundler when detecting `type = "pkg"` inside a project's `flame.toml`. Flame generates a highly optimized distribution bundle in `target/<profile>/pkg/`, automatically mirroring the `src/` tree, `flame.toml` manifests, generated `.fmi` interface bindings, and natively compiled `.rlib` archives into a single, self-contained GitHub-ready payload.

### 🎨 IDE Intellisense & Testing Workflow
- **Test Directory Module Resolution**: Hardened the internal dependency lookup (`locate_import_file`) to aggressively resolve the active workspace root when the language server executes transient diagnostic checks (e.g., against `flame_check.fm`). This unlocks full IDE support—including native Rust hover docs, autocomplete matrices, and type checks—when utilizing `import main` from a package's `test/` directory.
- **Documentation**: Added an extensive new Astro documentation guide (`Creating Packages`) detailing Git dependency pulling, native Rust plugin embedding within packages, test configurations, and export directives.
- **Annotation Syntax Highlighting**: Updated the VS Code `flame-ide` grammar to elegantly distinguish decorators. The `@` symbol now renders cleanly in white, while the annotation name (`@Test`, `@Benchmark`) is color-mapped to match standard language keywords.
- **Networking Hover Docs**: Injected extensive Markdown hover documentation into `std_docs.rs` for the entire suite of newly scaffolded `std.net` interfaces, providing immediate examples for `http.get`, `WebSocket.connect`, and `Mqtt.publish`.

### 🌐 Network Toolkit Architecture (`std.net`)
- **Zero-Overhead Feature Gating**: Re-architected the `flamelang` `Cargo.toml` to ensure that heavy networking libraries (`tokio`, `reqwest`, `tungstenite`, `rumqttc`) are strictly optional. `tokio` was stripped down to its essential capabilities, and `reqwest` utilizes lightweight `rustls` instead of `native-tls`. A script that only blinks an LED will never link a TCP stack.
- **Dynamic AOT Feature Injection**: The AOT compiler (`flame build`) now analyzes user imports during the compilation phase. If it detects `import std.net.http` within the `.fm` source code, it dynamically resolves and injects the `"http"` Cargo feature flag into the generated native binary.
- **Split Submodule Routing**: Dismantled the monolithic `std.net` namespace. Networking is now routed efficiently through submodules: `std.net.tcp`, `std.net.udp`, `std.net.http`, `std.net.ws`, `std.net.mqtt`, `std.net.dns`, `std.net.url`, and `std.net.interface`.

### ⚙️ Runtime & Lifecycle Management
- **Graceful Process Shutdowns**: Intercepted the `SIGINT` (`Ctrl+C`) signal via `ctrlc::set_handler()` at the root process level. Terminating background server scripts manually in the terminal now gracefully exits with code `0`, eliminating the aggressive `STATUS_CONTROL_C_EXIT (0xc000013a)` operating system panic.

---

## [0.1.9] - 2026-08-06 (Codename: *Flame Spark*)

### ⚡ Embedded Toolchain Enhancements
- **Native `thread.sleep` Translation**: The zero-cost bare-metal compiler now safely transpiles native `thread.sleep()` routines directly into architecture-specific hardware delay instructions (e.g., `arduino_hal::delay_ms()`), eliminating the need for standalone global sleep functions in firmware code.

### 🎨 IDE Intellisense & Developer Experience
- **Dynamic Hardware Methods**: The Flame language server now proactively injects native embedded capabilities (`mode()`, `high()`, `low()`, `read()`, `angle()`, `speed()`) into the property autocomplete matrix. This allows instant hardware method suggestions on variables like `led.` or `sensor.` even before full global type inference resolves their underlying physical hardware struct.
- **Method String Literal Completions**: Upgraded the internal cursor extraction algorithm to intelligently preserve quotation marks (`"`), enabling the language server to trigger accurate completion dropdowns for hardware configurations. Typing `sensor.mode("` now instantly suggests hardware-aware constants like `"INPUT_PULLUP"`, `"OUTPUT"`, and `"PWM"` complete with markdown documentation.
- **Documentation Overhaul**: Added comprehensive documentation for the `@Embedded` architecture directive inside the built-in annotations guide.


## [0.1.8] - 2026-08-06 (Codename: *Flame Spark*)

### 🎨 IDE Syntax Highlighting & Developer Experience (`flame-ide`)
- **String Interpolation Embedded Scoping**: Resolved a visual limitation in the VS Code syntax extension where expressions within interpolated string templates (e.g., `$"System RAM: {board.memory}"`) received monolithic blue string coloring. Injected authoritative TextMate embedded source language boundaries (`contentName: source.flame.embedded` and `meta.embedded.line.flame`), enabling expressions within braces `{...}` to render in vivid code colors without string token bleed.
- **Dotted Property Highlighting**: Added dedicated `variable.other.property.flame` TextMate grammatical matchers to clearly highlight accessed attributes and structural properties in bright editor accents.
- **Exhaustive Embedded Intellisense Hovers**: Upgraded the internal language server and documentation engine (`std_docs.rs`) to display rich Markdown method prototypes, hardware usage guidelines, and parameter summaries when hovering over any of the 22 embedded drivers and constructors.

---

## [0.1.7] - 2026-08-06 (Codename: *Flame Spark*)

### 🤖 Native Embedded Systems Runtime & HAL (`std.embedded`)
- **Industry-Standard Rust HAL Architecture**: Transformed Flame's embedded library from a simulated serial messaging prototype into a legitimate hardware abstraction layer (**HAL**) backed by Rust's standard `embedded-hal 1.0`, `embedded-io`, and `embedded-storage` trait architectures.
- **Capability-Based Resource Ownership**: Eliminated unsafe, globally exposed procedures (like Arduino's C-style `digitalWrite`) in favor of exclusive capability objects with strongly typed methods:
  - **Digital GPIO & Analog ADCs**: Construct pin capabilities (`embedded.pin(13)`) with directional mode setting, logic toggling (`.high()`, `.low()`, `.toggle()`), and calibrated 12-bit ADC voltage sampling (`embedded.analog(0).readVoltage()`).
  - **Actuators & Robotics Kinematics**: Precision hobby servo horn angle sweeping (`embedded.servo(5).angle(120)`), dual-channel H-bridge DC motor throttling (`embedded.motor(9, 7, 8).speed(85.0)`), and two-wheel rover differential drive kinematics (`embedded.diffDrive(m1, m2)`).
  - **Hardware Buses & Displays**: Direct synchronous communications over `I2C`, `SPI`, and automotive `CAN` buses, along with real-time matrix framebuffer rendering for OLED/TFT displays (`embedded.display.text("Telemetry Active")`).
  - **Persistent Memory & Watchdogs**: Store non-volatile configuration words in Flash and EEPROM across reboots, and feed hardware Watchdog countdown timers (`embedded.watchdog.feed()`) to guard against mission critical deadlocks.
- **Cross-Compilation Feature Gates**: Configured conditional compiler workspace target features across `avr`, `esp32`, `stm32`, and `rp2040` architectures, paired with direct Linux userland memory-mapped GPIO drivers (`rppal`) for Single-Board Computers (Raspberry Pi / BeagleBone).
- **Real Host Hardware Discovery**: Integrated `sysinfo` within `embedded.board` to dynamically probe and expose genuine host CPU core brands, clock architecture, memory statistics, and kernel specifications.
- **Zero Boilerplate Firmware**: Removed traditional mandatory C++ micro-controller lifecycles (`void setup()` and `void loop()`), empowering engineers to write concise, script-level hardware logic from line one.

### 📚 Interactive Web Hardware Simulation Lab & Documentation
- **Live Virtual Circuit Workstation (`FlameSimulator.astro`)**: Designed and launched a stunning, high-performance virtual hardware simulation lab directly within the Flame Starlight documentation portal.
- **Intention-Driven Monaco Editor**: Integrated a responsive code editor widget complete with line numbering, syntax coloring, and an interactive **Flame Intellisense Autocompletion Matrix** offering instant suggestions for keywords and all `std.embedded` hardware capabilities.
- **Animated Hardware Telemetry Canvas**: Constructed an interactive visualization bench featuring real-time RGB glowing LED logic indicators, rotating servo motor horns displaying exact degree angles, spinning DC motor rotors with PWM duty percentage readouts, a graphic I2C/SPI OLED display screen framebuffer, and a live Logic Analyzer oscilloscope waveform trace.
- **Comprehensive Starlight Guide**: Published the complete **Embedded Ecosystem (std.embedded)** engineering tutorial and reference manual ([embedded.mdx](docs/src/content/docs/standard-library/embedded.mdx)), showcasing best practices in modern firmware and robotics development.

---

## [0.1.6] - 2026-08-05 (Codename: *Flame Spark*)

### 🤖 Explicit Systems Syntax & Nil-Safety Foundations
- **Keyword Logical Operators**: Added robust support for readable keyword logical operators (`and`, `or`, and `not`) alongside classic symbolic forms (`&&`, `||`, `!`). Designed explicitly for robotics, automation, and embedded systems to eliminate visual ambiguity in complex control assertions.
- **Dedicated Nil-Safety Architecture**: Enforced non-nullable types by default with seamless optional declarations (`Type?`), removing undefined pointer traps from mission-critical control loops:
  - **Safe Member Navigation (`?.`)**: Short-circuits property accesses and method calls on optional receivers without risking runtime panics.
  - **Nil Coalescing Operator (`?:`)**: Ergonomic Elvish operator syntax to assign fail-safe default fallback values in a single evaluation expression.
  - **Non-Null Assertion (`!`)**: Explicit runtime assertion unwraps verified system state with safe line-precise runtime diagnostics on nil encounters.
- **Increment, Decrement & Compound Assignments**: Fully functional native runtime support for prefix and postfix increment and decrement operators (`++var`, `var++`, `--var`, `var--`) and compound arithmetic assignments (`+=`, `-=`, `*=`, `/=`, `%=`) for high-frequency counter loops and sensor data accumulation.
- **Universal Precision Conversions**: Enhanced `.toString(precision: Int)` method dispatch across the evaluation engine to cleanly format floating-point telemetry strings (e.g., `3.14159.toString(2)` -> `"3.14"`).
- **CLI Compiler Version Querying**: Added native CLI flags `flame --v`, `flame -v`, and `flame --version` to instantly query and output the installed compiler version and codename (`Flame 0.1.6 (Codename: Flame Spark)`).

### ⚡ IDE & Semantic Token Enhancements (`flame-ide`)
- **Complete Comment & String Insulation**: Upgraded the VS Code language semantic token provider to emit authoritative `comment` and `string` semantic token ranges over all line comments (`// ...`) and block comments (`/* ... */`). This permanently prevents bracket pair colorizer bleeding and false annotation syntax highlighting on commented-out code (such as `// @Test()` or `//(old_expr)`).
- **Logical Keyword Highlighting & Hover**: Updated static TextMate grammar (`flame.tmLanguage.json`) and active semantic tokens to highlight `and`, `or`, and `not` as dedicated logical keywords. Hovering over operators and keywords now displays structured markdown documentation with robotics usage examples.
- **Intelligent Timestamp-Based Compiler Discovery**: Refinanced the extension compiler binary resolution logic to inspect filesystem modification times (`mtime`), ensuring VS Code always selects the most recently rebuilt `flamelang.exe` binary across debug and release build targets.

### 📚 Comprehensive Documentation Suite Upgrade
- **Dedicated Nil Safety Page**: Launched a standalone **Nil Safety & Optionals** guide on the Flame web documentation portal ([docs](docs)), detailing the mission-critical importance of null safety in hardware automation and robotics engineering.
- **Updated Operators Guide**: Revamped the **Operators & Expressions** guide with full increment, decrement, compound assignment, bitwise register manipulations, and a complete operator precedence table.
- **Testing Invariants with `panic`**: Added dedicated test failure and invariant assertions guidance to the Native Testing Framework documentation ([testing-framework.mdx](docs/src/content/docs/annotations-and-testing/testing-framework.mdx)), demonstrating how `panic(message)` terminates individual test cases cleanly without halting overarching test suite execution.
- **Declarative CLI Application Building**: Added extensive systems documentation and practical real-world guides for constructing declarative command-line utilities using built-in `@Cli` root annotations, `@Command` subcommand handlers, automatic typed parameter flag parsing, and structural subcommand matching ([builtin-annotations.mdx](docs/src/content/docs/annotations-and-testing/builtin-annotations.mdx)).

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
