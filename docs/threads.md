# Multithreading in Flame

Flame provides a high-performance, multithreaded concurrency architecture designed from the ground up to avoid the bottlenecks of simple event loops while ensuring complete memory safety, zero race conditions, and predictable state management.

---

## 1. True Multithreading vs. Single-Threaded Event Loops

Flame is **not** a bottlenecked, single-threaded Node.js-style event-loop language. Under the hood, Flame leverages Rust's native Operating System threading and multi-threaded **Tokio worker pools**. 
- **`async` / `await`** is used specifically to manage asynchronous, non-blocking I/O operations (such as HTTP networking, file systems, and database queries) across multi-core worker pools.
- **`thread { ... }`** blocks spawn independent, physical operating system compute threads that execute purely CPU-bound tasks in parallel across multiple CPU cores without starving high-concurrency network loops.

---

## 2. Architecture: Separating Execution Model from Concurrency Model

A naive concurrency implementation often attempts to simply "clone the entire VM/interpreter state per HTTP request or task." However, Flame explicitly rejects this approach due to severe architectural drawbacks:
1. **Performance Overhead**: Cloning an entire interpreter state per incoming HTTP request is computationally expensive and wastes massive amounts of memory.
2. **Global State Divergence**: Shared application state, mutable globals, native objects, and loaded modules inevitably diverge across disconnected runtime clones.
3. **Loss of Determinism**: It becomes impossible to reason about whether two simultaneous requests are interacting with an identical application runtime state.

Instead, Flame cleanly separates the **concurrency worker model** from the **deterministic execution model**.

### Visualizing Flame's Multithreaded Architecture

![Flame Multithreading Architecture](./assets/flame_multithreading_diagram.png)

### Architectural Flow Explained:
1. **Multi-Core Tokio Worker Pools (Top Layer)**: When handling concurrent networking (such as a local native Axum HTTP server plugin), Tokio worker threads listen for incoming network sockets and accept connections in parallel across all available physical CPU cores simultaneously.
2. **Atomic Payload Packaging**: When a network endpoint or asynchronous callback triggers (e.g., an incoming `POST /users` payload), the native Rust FFI bridge wraps the request bodies, HTTP headers, and struct types (`Request` and `Response`) into standardized, thread-safe memory structures (`CValue`).
3. **Lock-Free Channel Queue (Middle Boundary)**: Instead of locking mutexes or cloning interpreters, the packaged callback handler and arguments are transmitted to Flame's VM engine across a high-speed, atomic message-passing queue.
4. **Single Deterministic Runtime Engine (Bottom Layer)**: The Flame Runtime processes the callback using its unified, thread-safe VM state, guaranteeing zero state divergence and absolute memory safety without requiring expensive interpreter cloning or risking mutex deadlocks.

---

## 3. Spawning Dedicated Compute Threads (`thread { ... }`)

While Tokio worker pools process non-blocking asynchronous I/O, pure computational math or CPU-heavy workloads require standalone OS processing power. You can spawn dedicated background threads using the `thread` block syntax:

```flame
import std.thread

fn main() {
    // Inspect the current OS thread identity
    print("Main Thread ID: " + str(thread.id())) // e.g., outputs "ThreadId(1)"

    // Spawn an independent computational OS thread
    let handle = thread {
        // Executes in parallel on a separate physical CPU core!
        print("Worker Thread ID: " + str(thread.id())) // e.g., outputs "ThreadId(2)"
        
        // Perform CPU-intensive math or long calculations safely
        let mut sum = 0
        let mut i = 0
        while i < 50000000 {
            sum = sum + (i * 2)
            i = i + 1
        }
        
        // Return computed artifact back to parent thread
        return { status: "Done", total: sum }
    }

    // Continue executing concurrent tasks in the main thread without blocking...
    print("Main thread continuing work while worker computes...")

    // Synchronously join or asynchronously await the worker thread's completion
    let result = await handle 
    // OR via explicit blocking join: let result = handle.join()
    
    print("Received computation result from worker: " + str(result.total))
}
```

### How `thread` Execution Guarantee Safety (Memory & Environment Isolation)
When you invoke a `thread { ... }` block, Flame enforces memory safety through **Lexical Snapshot Isolation**:
- **Atomic Environment Snapshots**: When crossing an OS thread boundary, Flame does not share raw mutable pointers between threads. Instead, the runtime freezes an immutable snapshot of the active lexical scope (`Env`) inside an atomic reference arc (`Arc<Env>`). 
- **Zero Race Conditions**: Because background workers evaluate math against isolated immutable snapshots or local variables, data races and pointer invalidation become mathematically impossible at compilation and runtime.
- **Rendezvous Synchronization**: Calling `await handle` or `handle.join()` creates a synchronization rendezvous point. Once the worker thread concludes execution, its finalized return value is safely transferred back across the thread boundary into the parent execution scope.

---

## 4. Channels: `send` and `recv`

Flame channels provide simple thread-to-thread message passing.

```flame
import std.thread

fn main() {
    let (tx, rx) = thread.channel()

    let worker = thread {
        tx.send(formula {
            kind: "ready",
            count: 3
        })
    }

    let message = rx.recv()
    await worker

    message.kind.assert_eq("ready")
    message.count.assert_eq(3)
}
```

- `thread.channel()` returns `(Sender, Receiver)`.
- `tx.send(value)` pushes a Flame value into the channel.
- `rx.recv()` blocks until a value arrives.
- Messages can be primitives, formulas, tuples, and other Flame runtime values.

## 5. Resolution of Error Conditions and Edge Cases

Flame's multithreaded runtime is engineered to handle abnormal execution states and developer errors cleanly:

### 1. Thread Panics and Unwound Stacks
If a computational background thread or native FFI bridge encounters a panic or division-by-zero exception, the unwinding stack is trapped strictly inside the isolated worker thread boundary. The core Flame VM state remains completely intact. Invoking `await handle` or `.join()` on a crashed thread gracefully resolves into a evaluably handled runtime diagnostic rather than aborting your server process.

### 2. Orphaned Threads and Memory Leak Prevention
What happens if you spawn a thread and forget to invoke `await` or `.join()`? 
Flame guarantees zero resource leakage. Once a background thread concludes execution, Rust’s deterministic RAII destructors immediately drop all localized variable allocations, environment references, and OS handles from memory automatically.

### 3. Graceful Program Shutdown (`wait_for_all_threads`)
Right before a Flame executable completes execution, the runtime engine automatically invokes an internal `wait_for_all_threads()` safeguard routine. This system monitor holds open process tear-down until all active background worker threads resolve cleanly, ensuring that open disk files, network descriptors, and pending operations complete without truncation.

### 4. Daemon Persistence for Native Listeners
When running asynchronous networking servers (such as `await app.listen(3000)`), simple script interpreters normally exit upon reaching the last line of code. Flame automatically senses active Tokio socket bindings or multithreaded listening queues, engaging a **multi-threaded daemon lock**. This ensures your application remains persistent, responsive, and serving HTTP requests continuously until an explicit user termination signal (`Ctrl + C`) is received.
