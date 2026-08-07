# Package Resolution & Native Interop in Flame

Flame provides a seamless interop story with native Rust crates and Flame packages. The package resolution system is designed to automatically detect and link dependencies, generating zero-overhead native bindings ahead-of-time (AOT).

## How Packages Are Resolved

When you add a package to your Flame project (e.g., `flame add <package_name>`), the `flame.toml` manifest is updated with the dependency path. 

### Resolution Flow:
1. **Dependency Analysis**: During compilation, Flame recursively analyzes all dependencies listed in `flame.toml`.
2. **Metadata Generation**: For native Rust dependencies, Flame automatically extracts the public API (structs, methods, and functions) using `rustdoc` JSON.
3. **Interface Files (`.fmi`)**: Flame generates a `.fmi` file for each native plugin. This `.fmi` file is automatically parsed during semantic analysis to provide IDE features (autocomplete, hover) and compile-time type-checking.
4. **Environment Initialization**: When executing a Flame script or compiling AOT, Flame automatically registers these `.fmi` modules into the global module registry. The parsed `.fmi` modules are cached inside `.flame/pkg/<package>/`.

## Native Plugin Linking (AOT Compilation)

When you run `flame run <file>` or `flame build`, Flame uses an advanced Ahead-Of-Time (AOT) compiler to generate a standalone Rust binary.

1. **AST Extraction**: The interpreter parses your Flame scripts and resolves module imports.
2. **Native Bridge Generation**: Flame generates raw C-FFI bridges for every public method of your native dependencies. It creates a `bridge_<crate_name>` wrapper that unpacks the Flame memory representation (`CValue`) into the native Rust types.
3. **Workspace Scaffold**: Flame scaffolds a temporary Rust workspace in `.flame/build-cache`. It creates a `Cargo.toml` that explicitly links the native dependencies you specified.
4. **Cargo Compilation**: It invokes `cargo build` on this cache. The AOT compiler links your native Rust plugins with the generated bridge and the Flame runtime library, resulting in a single executable (`<pkg_name>_aot.exe` or `lib<pkg_name>_aot.rlib`).
5. **Execution Integration**: Flame copies the final native executable to `target/dev/` (or `target/release/`) and spawns the process. 

## Git URLs and Remote Dependencies

Flame mirrors the Go dependency model for fetching remote packages natively:
- You can add GitHub dependencies natively by running: `flame add github.com/user/repo@v1.0.0`
- Flame automatically intercepts standard URL schemas and fetches the repository directly using `git clone`.
- The repository is cloned directly into the `.flame/pkg/<repo>` cache folder in your project.
- Remote Flame packages and plugins can specify branches or tags via the `@` symbol (e.g., `@main` or `@v0.2.1`).

## Package Generation

When you configure your project as a library (`type = "pkg"` in `flame.toml`), `flame build` does more than just AOT compile:
- It generates a full `pkg/` output directory in `target/<profile>/pkg/<pkg_name>`.
- The `src/` directory and `flame.toml` are mirrored to the output bundle.
- Any generated native bindings (`.fmi` interface files) and built native `.rlib` archives are included.
- This creates a fully self-contained package folder ready for distribution or direct GitHub linking!

## C-ABI Value Layout & Unpacking

Native objects cross the FFI boundary using the `CValue` struct, an unsafe, memory-efficient union that represents dynamic Flame values. 

When you call a native method:
1. `Runner` identifies the `Expr::Dot` receiver as a `Value::NativeObject`.
2. It looks up the associated FFI symbol inside the `native_methods` registry.
3. It packs the arguments into `CValue` instances and invokes the FFI bridge.
4. The bridge unpacks the `CValue` arguments, executes the native Rust method, and packs the return value back into a `CValue`.
5. The `Runner` unpacks the returned `CValue` and integrates it back into the Flame execution context.

## Summary

The entire process is transparent to the developer. You simply write Flame code and the compiler transparently detects, bridges, fetches Git dependencies, compiles, and statically links your native Rust plugins into a highly optimized binary—without requiring manual FFI setup.
