# Plugins and the `@plugin` Decorator

Wren's philosophy emphasizes robust integration with the native ecosystem. Beyond static AOT-compiled Rust crates, Wren also supports dynamically loaded plugins via the `@plugin` decorator.

## What is a Plugin?

In Wren, a plugin is a module or library that provides native code which is loaded and executed at runtime. This allows you to extend your Wren programs with dynamic capabilities without needing to recompile the main executable.

Plugins are declared in the `wren.toml` file under the `[plugins]` section:

```toml
[plugins]
my_plugin = "1.0.0"
local_plugin = "./path/to/local"
```

## The `@plugin` Decorator

To use a plugin inside your Wren script, you use the `@plugin` decorator on a structural component, typically preceding an import or function declaration. This tells the compiler and language server that the ensuing logic relies on a dynamically loaded plugin.

### Syntax

```wren
@plugin "my_plugin"
import my_plugin

// Now you can use functions and structs exported by my_plugin
```

### How it Works

1. **Resolution**: The Wren package manager resolves plugins from `wren.toml`.
2. **Intellisense**: When you type `@plugin`, the VS Code extension automatically suggests all available plugins declared in your manifest.
3. **Runtime**: The native bridge securely loads the plugin into the VM environment, exposing its native functions directly into the Wren module namespace.

By leveraging `@plugin`, you can build extensible architectures where third-party contributors can write native modules that plug directly into your application!
