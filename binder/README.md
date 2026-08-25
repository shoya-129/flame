# flame_binder

`flame_binder` is the official Rust integration crate for the **Flame** programming language. It provides a simple, native API to seamlessly embed the Flame VM within any Rust project, load Flame source files, and directly execute exported Flame functions from Rust.

## Usage

Add `flame_binder` to your Rust project's dependencies:

```toml
[dependencies]
flame_binder = { path = "path/to/flame_binder" }
flamelang = { path = "path/to/flamelang" }
```

### Loading and Executing Flame Code

Here is a quick example showing how to initialize the Flame environment and execute an exported function from a `.fm` file.

**`script.fm`** (Flame Source Code)
```flame
export fn hello(name: String) -> String {
    return $"Hello, {name}!";
}
```

**`main.rs`** (Rust Code)
```rust
use flame_binder::Binder;
use flamelang::vm::Value;

fn main() {
    // 1. Initialize the Binder with the path to your Flame script
    let mut binder = Binder::load("script.fm").expect("Failed to load Flame file");

    // 2. Prepare arguments to pass to the Flame function
    let args = vec![Value::String("World".to_string())];

    // 3. Call the exported Flame function natively
    let result = binder.call("hello", args).expect("Failed to execute function");

    // 4. Match and extract the returned Flame `Value`
    match result {
        Value::String(s) => println!("{}", s), // Outputs: Hello, World!
        _ => println!("Unexpected return type"),
    }
}
```

## Features

- **Native Rust Interop**: Seamlessly translate `flamelang::vm::Value` types between Rust and Flame.
- **Isolated Execution**: `Binder::load()` initializes an independent Flame `Runner` instance and handles lexical parsing, evaluation, and execution autonomously.
- **Lightweight**: Provides a minimal API surface focusing exclusively on executing Flame logic within Rust host applications.

## License

ISC License
