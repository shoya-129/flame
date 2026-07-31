# Modules, Imports, and Native Rust Crates in Flame

Flame's module system allows you to organize your code into multiple files and leverage the entire Rust ecosystem directly in your Flame projects.

## Modules and Files

In Flame, every `.fm` file is a module. 
By default, everything you define in a module is private to that module. 
To make a function, struct, or variable accessible to other modules, you must prefix it with the `export` keyword.

```flame
// math.fm
export fn add(a: Int, b: Int) -> Int {
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

```flame
// main.fm
import math.{add, PI}

let result = add(10, 20)
print($"Result is {result}, PI is {PI}")
```

## Native Rust Crates

One of Flame's most powerful features is its ability to directly import and use Rust crates. This works seamlessly because the Flame compiler generates native code and statically links Rust crates into your final executable.

### The `flame.toml` Configuration

To use a Rust crate, add it to the `[native-dependencies]` section in your `flame.toml` file. This tells the Flame package manager to download the crate using `cargo` and make its metadata available to your editor and compiler.

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

```flame
// main.fm
import native.uuid

let my_id = uuid.new_v4()
print($"Generated ID: {my_id}")
```

### Under the Hood: `.fmi` and Suggestions

Flame you build your project or run the language server, the Flame compiler automatically uses `rustdoc` to generate an Abstract Syntax Tree of the Rust crates you depend on. 

It generates `.fmi` files (Flame Metadata) in the `.flame/pkg` directory. This is how Flame knows the signature of `uuid.new_v4()`.

**VS Code Extension:**
If you use the `flame-vscode-extension`, it reads these `.fmi` files automatically. When you type `uuid.`, you will instantly get autocomplete suggestions for all public functions, structs, and methods exposed by the Rust crate, complete with their native Rust documentation! This allows you to explore Rust APIs effortlessly in your Flame code.
