# Using Native Rust Crates in Flame

Flame's philosophy is: **"Don't reinvent the wheel; just use Rust's."** 

Flame doesn't just provide an FFI (Foreign Function Interface) to C or Rust. Instead, the Flame compiler treats Rust crates as a **second source language**. When you build a Flame project, the compiler resolves the Rust dependencies, generates static bridges, and statically compiles them directly into your final executable. No dynamic loading, no `.dll` files, and no wrapper boilerplate.

## 1. Adding a Crate

To use a Rust crate, add it to your `flame.toml` file under `[native-dependencies]`. This is analogous to a `Cargo.toml` dependency block.

```toml
[package]
name = "my_awesome_app"
version = "0.1.0"

[native-dependencies]
uuid = "1.0"
axum = "0.7"
regex = "1.10"
```

## 2. Importing the Crate

In your `.flame` source files, import the crate using the `native.` prefix.

```flame
import native.uuid
import native.regex

let id = uuid.new_v4()
print($"Generated ID: {id}")
```

## 3. How to Know Which Functions to Use

Because flame interacts natively with Rust, **the API you use in flame is exactly the same as the Rust API**. 

There are two primary ways to discover what functions you can call on a crate:

### A. Using `docs.rs`

Since you are calling Rust code directly, you can read the official Rust documentation for the crate on [docs.rs](https://docs.rs). 

For example, if you look up the `uuid` crate on docs.rs, you will see a function called `new_v4`. In flame, you call it exactly the same way: `uuid.new_v4()`. If you see a struct method, you call it the same way.

### B. The VS Code Extension and `.fmi` (Intellisense)

Flame provides a world-class developer experience through its VS Code Extension. You don't have to guess or memorize APIs!

When you run `flame build` or `flame run`, the flame compiler uses `rustdoc` behind the scenes to generate metadata about the crate. It saves this metadata as `.fmi` files in your `.flame/pkg` directory.

The **flame VS Code Extension** reads these `.fmi` files automatically. 

1. Type the crate name and a dot (e.g., `uuid.`).
2. The extension will provide **autocomplete suggestions** showing every available function, struct, and constant from that Rust crate.
3. It will even show you the **original Rust documentation and signatures** inline in your editor, exactly as the crate author wrote them!

## 4. Creating Custom Native Plugins

In addition to downloading public crates from crates.io, you can write **your own custom Rust code** and call it directly from flame. This is the recommended approach if you need extreme performance or want to bind to a specific Rust library with custom logic.

To do this:

1. **Initialize a Native Project**: Inside your flame project, create a new Rust library by running `cargo init --lib native` (or use a dedicated `flame native init` command if provided by the toolchain). This creates a `native` folder with a `Cargo.toml` and `src/lib.rs`.
2. **Write Rust Code**: Add your high-performance or custom Rust functions inside `native/src/lib.rs`.
    ```rust
    // native/src/lib.rs
    pub fn heavy_computation(x: i64) -> i64 {
        x * x * 42
    }
    ```
3. **Add as a Dependency**: Open your `flame.toml` and add your local `native` folder as a local path dependency under `[native-dependencies]`.
    ```toml
    [native-dependencies]
    native_plugin = { path = "./native" }
    ```
4. **Use in flame**: Finally, import your custom Rust code as a native plugin in your flame files and use it instantly!
    ```flame
    import native.native_plugin
    
    let result = native_plugin.heavy_computation(10)
    print(result) // Outputs: 4200
    ```
    
The compiler will automatically detect the local Rust code, build it alongside your flame script, and instantly provide autocomplete suggestions for your custom functions!

## 5. How it Works Under the Hood (For the Curious)

1. **Resolution**: `flame run` reads your `flame.toml` and discovers your native Rust dependencies (both external crates and local folders).
2. **Metadata Generation**: It runs `cargo rustdoc` to generate JSON metadata representing the Abstract Syntax Tree (AST) of the crates.
3. **Bridge Generation**: flame's AOT (Ahead-of-Time) compiler generates a safe Rust wrapper around the crate's public APIs, automatically mapping data types (Strings, booleans, integers, floats). Generic functions (like `random<T>`) are instantiated dynamically for supported primitive types.
4. **Static Compilation**: flame generates a `Cargo` workspace inside the `.flame/build-cache` directory, compiles everything statically into a single executable, and maps the Rust functions directly into the flame VM's memory.
5. **Execution**: The resulting binary is ultra-fast, statically linked, and memory safe!
