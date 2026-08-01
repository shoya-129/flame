# Asynchronous Concurrency in Flame (`async` & `await`)

Flame provides a high-performance asynchronous execution runtime designed specifically for high-concurrency non-blocking I/O operations—such as handling thousands of incoming HTTP network requests, database queries, and file stream transfers—without sacrificing true multi-core OS thread utilization.

---

## 1. True Multi-Threaded Tokio Workers vs. Slow Event Loops

Unlike legacy runtimes such as Node.js that force asynchronous I/O onto a bottlenecked, single-threaded execution loop, Flame operates entirely on **Rust’s native multi-threaded Tokio worker pools**.
- **Multi-Core Concurrency**: Asynchronous operations are scheduled dynamically across all available physical and logical CPU cores simultaneously.
- **Deterministic Execution**: Instead of cloning entire VM interpreter states per network request (which leads to catastrophic memory overhead and state divergence), parallel worker threads handle incoming sockets simultaneously and deliver structured task envelopes over atomic memory channels to Flame's single deterministic execution engine.

---

## 2. What Exactly Happens Under the Hood on `async` and `await`?

When you mark a function or closure with `async`, you are creating a **Lazy Future Task Representation**:

```flame
async fn fetch_user_profile(user_id: Int) -> UserProfile {
    let response = await http.get("https://api.example.com/users/" + str(user_id))
    return response.json()
}
```

### The Anatomy of `.await` Execution
When Flame encounters an `await` instruction on an asynchronous operation:
1. **Task Suspension Without Thread Blocking**: Flame does **not** block or lock the operating system thread while waiting for network packets or I/O completions. Instead, the runtime suspends the state machine of the current future task only.
2. **Worker Hand-off**: The underlying OS worker thread immediately returns to the Tokio execution pool to process other concurrent network sockets, database connections, or compute scripts.
3. **Reactive Wakeup (Zero Polling)**: The system does not waste CPU cycles spinning in polling loops. When an OS network interrupt signals that the HTTP packets have arrived, the Tokio scheduler reactively re-wakes the task and resumes execution right after the `await` boundary with zero state overhead!

---

## 3. The "Missing Await" Problem: What If You Forget to Use `await`?

A common architectural pitfall when developing networking applications, HTTP servers, or long-running tasks is omitting the `await` keyword on an asynchronous operation. Because Flame is built on high-performance native futures, omitting `await` has profound runtime behavior consequences:

### Case 1: Unawaited HTTP / Network Clients (Lazy Futures)
In Flame and Rust, asynchronous functions return **lazy futures**—they do not perform any I/O work until explicitly polled and evaluated via `await`.

#### ❌ Incorrect (Missing `await` on HTTP Request):
```flame
// WARNING: This does not trigger an HTTP network connection!
let res = http.get("https://api.example.com/data")

// ERROR: Type mismatch / Runtime error!
// 'res' is an Unresolved Future Object (Future<Response>), NOT a resolved Response struct!
print(res.status_code) 
```
- **What Happens**: Because `http.get` returns an unresolved future, the network connection is **not initiated synchronously**. The variable `res` holds a pending task wrapper rather than the actual HTTP response payload. 
- **Consequences**: If you try to access member fields like `.status_code`, `.body`, or `.json()` on an unawaited variable, Flame's static type checker and runtime will trigger a diagnostic error. Worse, if `res` goes out of scope without ever being `.await`ed, the future is cleanly dropped and **the network request is silently abandoned before any packets are sent!**

#### ✅ Correct Usage:
```flame
// Successfully evaluates the future and extracts the resolved Response struct!
let res = await http.get("https://api.example.com/data")
print("HTTP Status: " + str(res.status_code))
```

---

### Case 2: Unawaited HTTP Server & Native Plugin Initialization
When initializing multi-threaded network listeners (such as an Axum native server plugin) or starting daemon bindings, skipping `await` destroys server availability.

#### ❌ Incorrect (Missing `await` on Server Init):
```flame
import native.server

// WARNING: Server initialization future created, but never awaited!
let app = server.init()

// Main script reaches EOF immediately and process shuts down!
```
- **What Happens**: Because `server.init()` is an asynchronous native FFI initialization call that sets up Tokio network sockets, failing to `await` it causes the execution engine to step right past the initialization without pausing.
- **Consequences**: The script immediately executes down to the EOF (End of File) without acquiring server daemon persistence locks. The operating system kills the process immediately before a single incoming HTTP connection can be acknowledged!

#### ✅ Correct Usage:
```flame
import native.server

// Await socket binding and initialize multithreaded server state
let app = await server.init()

// Register routes with typed Request and Response structs
app.get("/status", (req) {
    return { status: "Online", uptime: 99.99 }
})

// Await server loop engagement; runtime automatically engages persistent daemon mode!
await app.listen(8080)
```

---

### Case 3: Synchronous Long-Running Processes Inside Async Handlers (Thread Starvation)
While `async` / `await` handles millions of concurrent network wait tasks seamlessly, placing **heavy CPU-bound synchronous code** (such as cryptography hashes, massive array iterations, or blocking recursive loops) directly inside an `async` route without async yield points creates **Worker Starvation**.

#### ❌ Incorrect (CPU Blocking inside Network Route):
```flame
app.get("/generate-report", (req) {
    // CRITICAL WARNING: Running heavy synchronous loops blocks this Tokio worker thread!
    let mut i = 0
    while i < 1000000000 {
        i = i + 1
    }
    return { status: "Complete" }
})
```
- **What Happens**: Because the simple synchronous `while` loop does not contain an `await` instruction or an asynchronous yield point, it commandeers the entire operating system worker thread for several seconds without releasing control to the Tokio scheduler.
- **Consequences**: During this calculation window, any other concurrent incoming HTTP requests routed by the OS to this specific worker thread will be blocked in a waiting queue, degrading server throughput and responsiveness!

#### ✅ The Solution: Offloading Compute to Worker Threads (`thread { ... }`)
Flame cleanly separates **I/O Concurrency (`async` / `await`)** from **Computational Concurrency (`thread`)**. If an HTTP route needs to execute a massive computational or synchronous task, spawn a dedicated background OS compute thread:

```flame
app.get("/generate-report", async (req) {
    // Offload CPU-bound calculations to a parallel native compute worker thread
    let compute_task = thread {
        let mut sum = 0
        let mut i = 0
        while i < 1000000000 {
            sum = sum + i
            i = i + 1
        }
        return sum
    }

    // Await the compute thread's completion asynchronously without blocking network workers!
    let result = await compute_task.join()
    return { status: "Complete", calculation: result }
})
```

---

## 4. Summary Best Practices

| Task Type | Recommended Flame Mechanism | Why? |
| :--- | :--- | :--- |
| **HTTP Requests & Networking** | `async` / `await` | Non-blocking execution allows thousands of parallel connections per CPU core without thread overhead. |
| **Database & File I/O** | `async` / `await` | Releases OS worker threads instantly while disk or database engines process read/write buffers. |
| **Heavy Math & Data Processing** | `thread { ... }` blocks | Allocates independent physical OS compute threads that run in parallel without blocking server network loops. |
| **Server Daemon Loops** | `await app.listen(port)` | Automatically engages Flame's zero-polling multi-threaded daemon persistence until user interrupt (`Ctrl + C`). |
