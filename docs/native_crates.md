# Using Native Rust Crates in Wren

Wren's philosophy is: **"Don't reinvent the wheel; just use Rust's."** 

Wren doesn't just provide an FFI (Foreign Function Interface) to C or Rust. Instead, the Wren compiler treats Rust crates as a **second source language**. When you build a Wren project, the compiler resolves the Rust dependencies, generates static bridges, and statically compiles them directly into your final executable. No dynamic loading, no `.dll` files, and no wrapper boilerplate.

## 1. Adding a Crate

To use a Rust crate, add it to your `wren.toml` file under `[native-dependencies]`. This is analogous to a `Cargo.toml` dependency block.

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

In your `.wren` source files, import the crate using the `native.` prefix.

```wren
import native.uuid
import native.regex

let id = uuid.new_v4()
print($"Generated ID: {id}")
```

## 3. How to Know Which Functions to Use

Because Wren interacts natively with Rust, **the API you use in Wren is exactly the same as the Rust API**. 

There are two primary ways to discover what functions you can call on a crate:

### A. Using `docs.rs`

Since you are calling Rust code directly, you can read the official Rust documentation for the crate on [docs.rs](https://docs.rs). 

For example, if you look up the `uuid` crate on docs.rs, you will see a function called `new_v4`. In Wren, you call it exactly the same way: `uuid.new_v4()`. If you see a struct method, you call it the same way.

### B. The VS Code Extension and `.wmeta` (Intellisense)

Wren provides a world-class developer experience through its VS Code Extension. You don't have to guess or memorize APIs!

When you run `wren build` or `wren run`, the Wren compiler uses `rustdoc` behind the scenes to generate metadata about the crate. It saves this metadata as `.wmeta` files in your `.wren/pkg` directory.

The **Wren VS Code Extension** reads these `.wmeta` files automatically. 

1. Type the crate name and a dot (e.g., `uuid.`).
2. The extension will provide **autocomplete suggestions** showing every available function, struct, and constant from that Rust crate.
3. It will even show you the **original Rust documentation and signatures** inline in your editor, exactly as the crate author wrote them!

## 4. How it Works Under the Hood (For the Curious)

1. **Resolution**: `wren run` reads your `wren.toml` and discovers your native Rust dependencies.
2. **Metadata Generation**: It runs `cargo rustdoc` to generate JSON metadata representing the Abstract Syntax Tree (AST) of the crates.
3. **Bridge Generation**: Wren's AOT (Ahead-of-Time) compiler generates a safe Rust wrapper around the crate's public APIs.
4. **Static Compilation**: Wren generates a `Cargo` workspace inside the `.wren/build-cache` directory, compiles everything statically into a single executable, and maps the Rust functions directly into the Wren VM's memory.
5. **Execution**: The resulting binary is ultra-fast, statically linked, and memory safe!
