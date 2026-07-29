# Wrenlang

Wrenlang is a statically typed, fast, and modern programming language built in Rust. It serves as a comprehensive toolchain containing a compiler, an ahead-of-time (AOT) builder, and native FFI bridging capabilities.

## Features
- **Ahead of Time (AOT) Compilation**: Compile Wren code and bridge natively with Rust.
- **Native Dependencies**: Directly import and statically link Rust crates via `[native-dependencies]`.
- **Package Manager**: Built-in lightweight, Golang-style package manager capable of fetching dependencies over HTTP from git repositories without needing `git` installed.
- **Zero-Overhead FFI**: Call native Rust functions directly from your Wren scripts with ease.

## Installation
You can install Wrenlang using Cargo:

```bash
cargo install wrenlang
```

## Quick Start

1. **Initialize a new project**
```bash
wrenlang init my_project
cd my_project
```

2. **Add Native Dependencies**
Add dependencies from a local path or a GitHub repository:
```bash
wrenlang add https://github.com/user/my_plugin
```
Or for local native development:
```bash
wrenlang native init
wrenlang add --native bridge ./native
```

3. **Build & Run**
```bash
wrenlang run src/main.wren
```
Alternatively, build a standalone statically-linked executable:
```bash
wrenlang build
```

## Documentation
Please refer to the `docs` folder for extensive documentation on plugins, syntax, and AOT architecture.

## License
ISC
