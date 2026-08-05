# Flame Native Macros (`flame-macro`) [Beta]

`flame-macro` is an attribute procedural macro crate designed for native Rust plugins in the Flame ecosystem.

---

## Installation

In your native plugin's `Cargo.toml`:

```toml
[dependencies]
flame-macro = { path = "../flame-macro" }
```

---

## Macro Attributes

### `#[flame(daemon)]` / `#[flame(runtime)]`
Marks an asynchronous function or listener as a long-running service (such as an Axum web server, WebSocket server, or message broker). This informs the Flame runtime to maintain the event loop and thread pool while the daemon is active.

```rust
use flame_macro::flame;

impl FlameServer {
    #[flame(daemon)]
    pub async fn listen(self, port: u16) -> std::io::Result<()> {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, self.router).await.map_err(std::io::Error::other)
    }
}
```

---

### `#[flame(constructor)]`
Marks an associated method as the default constructor when instantiated from Flame code.

```rust
use flame_macro::flame;

pub struct DatabasePool { ... }

impl DatabasePool {
    #[flame(constructor)]
    pub fn connect(url: &str) -> Self {
        ...
    }
}
```

---

### `#[flame(skip)]`
Hides internal helper methods, fields, or functions from being exposed to the Flame interface (`.fmi`) and IDE autocomplete.

```rust
use flame_macro::flame;

impl MyPlugin {
    #[flame(skip)]
    pub fn internal_helper(&self) {
        // Not visible to Flame code
    }
}
```

---

### `#[flame(rename = "name")]`
Customizes the identifier exposed to Flame scripts.

```rust
use flame_macro::flame;

impl MyPlugin {
    #[flame(rename = "fetch_data")]
    pub fn rust_internal_fetch_routine(&self) -> String {
        "data".to_string()
    }
}
```
