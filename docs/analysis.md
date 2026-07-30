# Flame: Architecture and Analysis

This document provides an in-depth analysis of the Flame programming language, covering its type system, memory management, performance characteristics, and execution model.

## Is Flame a Programming Language or a Scripting Language?

Flame is a **statically-typed, compiled programming language**, though it has been designed with the ergonomics of a scripting language in mind. You can use it to build full standalone applications (using `flame build`) or run files quickly as if they were scripts (`flame run`). Its underlying machinery, however, treats it as a full-fledged compiled language.

## Compilation vs. Interpretation

Flame is **not an interpreter-based language**. It is a **compiled language** that uses **Ahead-of-Time (AOT) compilation**. 

When you execute Flame code, it is compiled down and can bridge natively with Rust. The compiler produces a standalone, statically-linked native executable binary that interacts directly with the operating system, without requiring a virtual machine or interpreter at runtime.

## Type Safety

Flame is **statically typed**, meaning that type checking occurs at compile time. This ensures that a large class of bugs, such as passing the wrong data type to a function or performing invalid operations on objects, are caught before the program is ever run. Because it integrates deeply with Rust, it benefits from strict type-checking rules, resulting in highly reliable and predictable code execution.

## Memory Management

Flame manages memory using a **Rust-like Ownership and Borrowing model**. 
It **does not use a Garbage Collector (GC)**.

The memory model guarantees memory safety at compile time by enforcing strict rules:
1. **Ownership**: Each value has a single owner. When the owner goes out of scope, the memory is automatically freed.
2. **Borrowing**: Values can be borrowed immutably (`&T`) or mutably (`&mut T`), but you cannot have mutable and immutable references to the same data at the same time.

This model completely eliminates memory leaks, double-frees, dangling pointers, and data races in concurrent contexts, all while maintaining zero runtime overhead.

## Interoperability with Rust

Flame is built in Rust and is designed to have **zero-overhead Foreign Function Interface (FFI)** with it.
- **Native Bridging**: You can call native Rust functions directly from Flame.
- **Native Dependencies**: You can directly import and statically link Rust crates by defining them in a `[native-dependencies]` block.
Because Flame compiles to native code compatible with Rust's ABI, there is no serialization or context-switching penalty when Flame code calls Rust code, or vice versa.

## Performance Characteristics

Because Flame uses AOT compilation, has zero-overhead Rust FFI, and lacks a garbage collector, its performance is on par with C, C++, and Rust.

**Performance Stats / Benefits:**
- **Zero GC Pauses**: Since memory is managed at compile time via ownership, there are no unpredictable stutters or pauses during execution.
- **Fast Startup**: A statically-linked native binary has practically zero startup time compared to interpreted languages or languages running on a JVM/CLR.
- **Minimal Memory Footprint**: Without a runtime environment or garbage collector, Flame binaries consume only the memory explicitly allocated by the program.
- **Optimized Execution**: Leveraging Rust's underlying toolchains means Flame binaries benefit from advanced LLVM optimizations.

## Where Can Flame Be Used?

Given its speed, memory management, and ergonomics, Flame is highly versatile:
- **Systems Programming**: Writing CLIs, daemons, and system utilities where low-level control and high performance are mandatory.
- **Performance-Critical Components**: Modules that require heavy computation (e.g., parsers, data processors, game logic) that can be seamlessly bridged with Rust.
- **Scripting Replacements**: Automating tasks that are too slow or memory-heavy in Python or Node.js. `flame run` allows for quick iteration while still providing native performance.
- **Embedded or Resource-Constrained Environments**: Due to its small memory footprint and lack of a runtime environment, Flame is well-suited for environments where every megabyte counts.
