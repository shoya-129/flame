# Flame Toolchain & Language Changelog

All pre-release versions in the `0.x.x` series carry the official codename **Flame Spark**, reflecting the fast, evolving, and multithreaded foundation of the language toolchain. Upon reaching the stable `1.0.0` milestone, Flame will transition to its canonical **Final Spark** release codename.

---

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
