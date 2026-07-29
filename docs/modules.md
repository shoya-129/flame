# Modules, Imports, and Native Rust Crates in Wren

Wren's module system allows you to organize your code into multiple files and leverage the entire Rust ecosystem directly in your Wren projects.

## Modules and Files

In Wren, every `.wren` file is a module. 
By default, everything you define in a module is private to that module. 
To make a function, struct, or variable accessible to other modules, you must prefix it with the `export` keyword.

```wren
// math.wren
export fn add(a: I32, b: I32) -> I32 {
    return a + b
}

export const PI: Float = 3.14159

// This function is private
fn helper() {
    print("Doing math...")
}
```

## Importing Local Modules

You can import elements from another module using the `import` keyword. You can import specific items or everything in a namespace.

```wren
// main.wren
import math.{add, PI}

let result = add(10, 20)
print($"Result is {result}, PI is {PI}")
```

## Native Rust Crates

One of Wren's most powerful features is its ability to directly import and use Rust crates. This works seamlessly because the Wren compiler generates native code and statically links Rust crates into your final executable.

### The `wren.toml` Configuration

To use a Rust crate, add it to the `[native-dependencies]` section in your `wren.toml` file. This tells the Wren package manager to download the crate using `cargo` and make its metadata available to your editor and compiler.

```toml
[package]
name = "my_app"
version = "0.1.0"

[native-dependencies]
uuid = "1.0"
axum = "0.7"
```

### Importing a Rust Crate

You import Rust crates just like any other module, but you prefix the crate name with `native.` to explicitly state that it's a Rust crate dependency.

```wren
// main.wren
import native.uuid

let my_id = uuid.new_v4()
print($"Generated ID: {my_id}")
```

### Under the Hood: `.wmeta` and Suggestions

When you build your project or run the language server, the Wren compiler automatically uses `rustdoc` to generate an Abstract Syntax Tree of the Rust crates you depend on. 

It generates `.wmeta` files (Wren Metadata) in the `.wren/pkg` directory. This is how Wren knows the signature of `uuid.new_v4()`.

**VS Code Extension:**
If you use the `wren-vscode-extension`, it reads these `.wmeta` files automatically. When you type `uuid.`, you will instantly get autocomplete suggestions for all public functions, structs, and methods exposed by the Rust crate, complete with their native Rust documentation! This allows you to explore Rust APIs effortlessly in your Wren code.
