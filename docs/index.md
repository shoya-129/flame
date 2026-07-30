# Flame Language Documentation

Welcome to the official documentation for the Flame programming language. Flame is a fast, safe, and modern systems language that directly understands and compiles Rust crates.

## Core Language

* **[Syntax and Control Flow](./syntax.md)**: Variables, mutability, `if`/`else`, `match`, loops, and defer.
* **[Types, Structs, and Traits](./types.md)**: Functions, custom types (`struct` and `enum`), `impl` blocks, and traits.
* **[Ownership and Borrowing](./ownership.md)**: Memory safety, `&T`, `&mut T`, and the borrow checker.
* **[Concurrency and Async](./async.md)**: Writing asynchronous code with `async`, `await`, and `spawn`.

## Ecosystem and Tooling

* **[Standard Library](./stdlib.md)**: Built-in functions, primitive types (Strings, Vectors, HashMaps), and standard utilities.
* **[Using Native Rust Crates](./native_crates.md)**: How to add Rust crates to `flame.toml`, how the AOT compiler statically links them, and how to use VS Code intellisense to discover APIs natively from docs.rs.
* **[Plugins & `@plugin`](./plugins.md)**: How to dynamically load native logic using the `@plugin` decorator.
* **[Modules and Imports](./modules.md)**: Organizing your Flame codebase with imports and exports.
