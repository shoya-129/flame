# Native Plugins and Advanced FFI in Flame

Flame's native plugin ecosystem provides zero-overhead interoperability with Rust code, allowing you to seamlessly integrate complex asynchronous frameworks like **Tokio** and **Axum**, define custom struct types, and pass parameters directly into Flame callback handlers.

---

## Why Native Local Plugins?

When building high-performance networking servers, cryptographic tools, or specialized systems, writing custom native plugins allows you to leverage Rust's full speed while retaining Flame's rapid developer experience, dynamic script ergonomics, and automated IDE intellisense.

Unlike traditional foreign function interfaces (FFI) that rely on dynamic `.dll` / `.so` library loading, Flame bridges compile Ahead-of-Time (AOT) directly into a statically linked binary.

---

## Developing a Local Plugin: An Axum + Tokio Web Server

Let's build a fully functional asynchronous web server native plugin that exposes custom structs, route registers (`GET` and `POST`), and parameter passing into Flame callbacks.

### 1. Initialize the Plugin Architecture

Create a `native` Rust workspace inside your project directory (`./native/src/lib.rs` and `./native/Cargo.toml`):

```toml
# native/Cargo.toml
[package]
name = "server"
version = "0.1.0"
edition = "2021"

[dependencies]
axum = "0.7"
tokio = { version = "1.0", features = ["full"] }
```

### 2. Exporting Struct Types and Methods (`lib.rs`)

Flame automatically extracts public struct types, documentation comments, and function signatures to generate `.fmi` interface files for IDE hover types and type checking!

```rust
// native/src/lib.rs
use axum::{routing::{get, post}, Router};
use std::mem;
use std::net::SocketAddr;

/// Represents the core Flame web application server instance.
pub struct FlameServer {
    router: Router,
}

/// Represents an incoming HTTP network request payload.
#[derive(Debug, Clone)]
pub struct Request {
    pub body: String,
}

/// Represents an outgoing HTTP network response payload.
#[derive(Debug, Clone)]
pub struct Response {
    pub body: String,
}

/// Initialize a new FlameServer instance.
pub fn init() -> FlameServer {
    FlameServer {
        router: Router::new(),
    }
}

impl FlameServer {
    /// Register a GET route with a zero-argument Flame callback handler.
    pub fn get<H, T>(&mut self, path: &'static str, handler: H)
    where
        H: axum::handler::Handler<T, ()> + Clone + Send + Sync + 'static,
        T: 'static,
    {
        let router = mem::take(&mut self.router);
        self.router = router.route(path, get(handler));
    }

    /// Register a POST route taking a request body String argument.
    pub fn post<H, T>(&mut self, path: &'static str, handler: H)
    where
        H: axum::handler::Handler<T, ()> + Clone + Send + Sync + 'static,
        T: 'static,
    {
        let router = mem::take(&mut self.router);
        self.router = router.route(path, post(handler));
    }

    /// Bind to a TCP network socket and start the asynchronous Tokio daemon loop.
    pub async fn listen(self, port: u16) -> std::io::Result<()> {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, self.router).await.map_err(std::io::Error::other)
    }
}
```

---

## 3. Configuring the Flame Manifest (`flame.toml`)

Declare your local native folder under `[plugins]` or `[native-dependencies]`:

```toml
[package]
name = "automation"
version = "0.1.0"
entry = "src/main.fm"

[plugins]
server = "./native"
```

---

## 4. Using Your Server and Struct Types in Flame

When you build or check your project, Flame automatically generates a JSON-compatible `.fmi` file in `.flame/pkg/server/server.fmi`. Your IDE immediately reads this to provide complete type inference and inline documentation!

```flame
import native.server

// 'app' is accurately inferred and displayed on hover as struct type: FlameServer
let app = await server.init()

// Register multiple asynchronous routes
app.get("/", () {
    "Welcome to Flame Multithreaded Web Server!"
})

app.get("/about", () {
    "Built on top of Rust, Axum, and Tokio!"
})

// Register a POST handler receiving parameters directly from HTTP payloads!
fn create_user(body: String) -> String {
    print($"Received incoming POST data: {body}")
    $"Successfully created record for payload: {body}"
}
app.post("/users", create_user)

// Bind and engage multi-threaded background daemon listening on port 3000
await app.listen(3000)
```

---

## IDE Type Inference and `.fmi` Interfaces

Behind the scenes, Flame inspects your Rust plugin using `cargo rustdoc` to output standardized `.fmi` files. Unlike messy C header bindings, Flame's FMI files store elegant JSON representations of:
1. **Modules**: Namespaces and exported functions.
2. **Structs**: Custom types like `FlameServer`, `Request`, and `Response`.
3. **Docstrings**: Original comments formatted in markdown directly in VS Code hover modals!
4. **Signatures**: Exact parameter types and return values for every native function and struct method.

When hovering over `app` in the example above, VS Code natively displays:
```flame
app: FlameServer
```
And typing `app.` pops up completion items for `.get()`, `.post()`, and `.listen()` with their accurate parameter types and return signatures!
