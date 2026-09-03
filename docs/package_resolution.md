# Package Resolution & Application-Specific Native Runtimes in Flame

Flame provides a seamless dependency and native interop model. Rather than requiring a universal runtime that contains every package or forcing developers to transpile their entire application into Rust source, Flame constructs an **application-specific native runtime** tailored directly to the dependencies your application actually uses.

## How Packages Are Resolved

When you add a package to your Flame project (e.g., `flame add <package_name>` or declare it in `flame.toml`), run:

```bash
flame install
```

### Resolution Flow:
1. **Dependency Analysis**: Flame parses the dependencies declared in `flame.toml` and fetches remote packages into `.flame/pkg/<package>/`.
2. **Metadata Generation (`.fmi`)**: For packages with native Rust implementations, Flame inspects their public API (structs, methods, and functions) using `rustdoc` JSON and syn AST inspection to extract interface metadata.
3. **Interface Files (`.fmi`)**: Flame generates and caches `.fmi` interface files in `.flame/pkg/<package>/<package>.fmi`. These `.fmi` files provide compile-time type-checking, semantic analysis, and full IDE IntelliSense (autocomplete, hover) without requiring the application developer to have Rust plugin source code in their project.
4. **Environment Initialization**: The Flame runtime registers these `.fmi` interfaces, enabling Flame source files (`src/`) to call native functionality seamlessly.

---

## Application-Specific Native Runtime Construction

When you run `flame build` or `flame build --release`:

```text
Flame Source
     ↓
   Blaze
     ↓
Dependency Analysis
     ↓
Application-Specific Runtime
     ↓
Rust / Cargo
     ↓
LLVM
     ↓
Native Binary
```

1. **Import & Feature Scanning**: Blaze scans your `src/` files and dependencies to determine the exact set of imported standard modules (e.g., `std.net.http`, `std.json`) and active native plugins.
2. **Native Bridge Generation**: Flame generates C-FFI bridges (`bridge_<crate_name>`) for each public method required by the dependency graph, converting between Flame's memory layout (`CValue`) and native Rust types.
3. **Tailored Runtime Scaffold**: Flame constructs an application-specific Cargo configuration in `.flame/build-cache` with `default-features = false`, activating **only** the features and linking **only** the native plugins required by your application.
4. **Cargo & LLVM Native Build**: Cargo compiles this specialized runtime with LLVM optimizations into `target/dev/` or `target/release/`.
5. **Clean Filesystem Deployment**: In normal build mode, the compiled native binary executes directly on the host filesystem without VFS. (VFS is used only when explicitly building a self-contained single executable via `flame build --exe` or `flame build --vfs --release`).

---

## Remote Dependencies & Package Management

Flame mirrors the modern dependency model for fetching remote packages:
- Add GitHub dependencies: `flame add github.com/user/repo@v1.0.0`
- Flame fetches the repository into `.flame/pkg/<repo>`.
- Packages provide their Flame-side source (`.fm`) and interface metadata (`.fmi`).
- **Developers never manually copy Rust plugin source code, Cargo projects, or DLL source trees into their application.**

---

## Package Creation & Distribution

When configuring a library or package (`type = "pkg"` in `flame.toml`), running `flame build` bundles:
- `flame.toml` manifest.
- The `src/` directory containing exported Flame code.
- Generated `.fmi` interface files for bundled native plugins.
- Built native archives (`.rlib`).

Consumers can install your package via `flame install` and immediately benefit from your native plugins and full IDE autocompletion with zero manual FFI bridging.

---

## C-ABI Value Layout & FFI Boundary

Native objects cross the FFI boundary using `CValue`, an efficient memory union representing Flame values:
1. `Runner` identifies native object methods via `.fmi` signatures.
2. It looks up the associated FFI symbol in the native methods registry.
3. It passes arguments into the generated bridge function.
4. The bridge unpacks the `CValue` arguments, executes the native method, and packs the return value back into a `CValue`.
5. Because the specialized runtime is compiled with LTO and LLVM optimization, method calls occur with native CPU performance.
