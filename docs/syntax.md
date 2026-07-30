# Flame Syntax and Language Guide

Flame is a modern, statically typed language that combines performance with expressive, high-level abstractions. This document provides a comprehensive guide to all built-in types, keywords, control flow mechanics, and concurrent programming models available in Flame.

---

## 1. Data Types

Flame is statically typed. Types are either inferred or explicitly declared.

### Primitive Types
* **`Int`**: A 64-bit signed integer. 
  * *Usage*: `let age: Int = 25`
* **`Float`**: A 64-bit floating-point number.
  * *Usage*: `let pi: Float = 3.1415`
* **`String`**: A UTF-8 encoded text string. Supports standard and interpolated strings.
  * *Usage*: `let name = "Alice"`
  * *Interpolation*: `let greeting = $"Hello, {name}!"`
* **`Bool`**: A boolean value (`true` or `false`).
  * *Usage*: `let is_active = true`
* **`Nil`**: Represents the absence of a value (similar to `null` or `void`).

### Composite Types
* **`Vec<T>`**: A dynamically-sized array (vector) containing elements of type `T`.
  * *Usage*: `let numbers: Vec<Int> = [1, 2, 3]`
* **Tuples**: Fixed-size, ordered collections of potentially different types.
  * *Usage*: `let coordinates: (Int, String) = (10, "North")`
* **`formula` (Map/Dictionary)**: A structured map-like data literal, ideal for configuration, JSON-like objects, or named key-value collections. Keys must be identifiers, and values can be literals, lists, closures (anonymous functions), or nested formulas. Duplicate keys are supported; if a key is assigned multiple times within the same formula, the latter value overwrites the previous one without needing the `mut` keyword.
  * *Usage*:
    ```flame
    let config = formula {
        host: "localhost",
        port: 8080,
        options: {
            secure: true
        },
        nodes: [1, 2, 3],
        // Duplicate keys overwrite previous ones seamlessly!
        port: 8081,
        // Formulas can also store closures/anonymous functions
        on_ready: () {
            print("Config is ready!")
        }
    }
    ```

### References (Borrowing)
Flame uses references to borrow data without copying it.
* **`&T`**: An immutable reference. Read-only access to a value.
  * *Usage*: `let ref = &coordinates`
* **`&mut T`**: A mutable reference. Allows modifying the borrowed value.
  * *Usage*: `let ref = &mut coordinates`

---

## 2. Variables and Constants

* **`let`**: Declares a standard, immutable variable. Once assigned, it cannot be changed.
  ```flame
  let name = "John"
  // name = "Doe" // Error: cannot mutate immutable variable
  ```
* **`mut`**: Used in conjunction with `let` to declare a variable whose value can be mutated.
  ```flame
  let mut counter = 0
  counter = counter + 1
  ```
* **`const`**: Declares a compile-time constant. Must include an explicit type.
  ```flame
  const MAX_USERS: Int = 100
  ```

---

## 3. Functions

* **`fn`**: Defines a function. You must specify parameter types and optionally the return type.
  ```flame
  fn add(a: Int, b: Int) -> Int {
      return a + b
  }
  ```

* **`async fn`**: Defines an asynchronous function that returns a Future and does not block the thread.
  ```flame
  async fn fetch_data(url: String) -> String {
      // background work...
      return "data"
  }
  ```

---

## 4. Control Flow

Flame provides a rich set of control flow keywords.

### Conditionals
* **`if` / `else`**: Standard conditional branching.
  ```flame
  if score > 90 {
      print("A")
  } else if score > 80 {
      print("B")
  } else {
      print("C")
  }
  ```

* **`match`**: Powerful pattern matching against variables or enums.
  ```flame
  let state = 1
  match state {
      0 => print("Stopped"),
      1 => print("Running"),
      _ => print("Unknown") // `_` is the default/fallback case
  }
  ```

### Loops
* **`while`**: Loops as long as a condition evaluates to `true`.
  ```flame
  let mut x = 0
  while x < 5 {
      x = x + 1
  }
  ```

* **`loop`**: Creates an infinite loop. Must be explicitly broken.
  ```flame
  loop {
      print("Running forever...")
      break // explicitly exit the loop
  }
  ```

* **`for` / `in`**: Iterates over a collection or range.
  ```flame
  let items = [1, 2, 3]
  for item in items {
      print(item)
  }
  ```

### Flow Modifiers
* **`break`**: Exits the innermost loop immediately.
* **`continue`**: Skips the rest of the current loop iteration and moves to the next.
* **`return`**: Returns a value from the current function and exits it.
* **`defer`**: Schedules a block of code to run at the exact moment the current scope closes. Useful for cleaning up resources, closing files, or unlocking mutexes.
  ```flame
  fn process_file() {
      let file = open("data.txt")
      defer file.close() // Will automatically run when process_file finishes
      // ... read file ...
  }
  ```

---

## 5. Object-Oriented and Custom Types

flame separates data layout (`struct`) from behavior (`impl`).

* **`struct`**: Defines a custom data structure with named fields.
  ```flame
  struct User {
      name: String,
      age: Int
  }
  ```

* **`impl`**: Defines methods (functions attached to a type) for a `struct` or `enum`.
  ```flame
  impl User {
      // Associated function (like a static method)
      fn new(name: String) -> User {
          return User { name: name, age: 0 }
      }

      // Method (takes `self` or `&self` or `&mut self`)
      fn birthday(&mut self) {
          self.age = self.age + 1
      }
  }
  ```

* **`enum`**: Defines a type that can be one of several named variants. Variants can optionally hold data.
  ```flame
  enum ConnectionState {
      Disconnected,
      Connecting(String), // holds an IP string
      Connected
  }
  ```

* **`trait`**: Defines a shared interface (a contract) that multiple structs or enums can implement.
  ```flame
  trait Drawable {
      fn draw(&self)
  }
  ```

---

## 6. Concurrency (`thread`, `await`)

Flame handles concurrency using a thread-based background task system.

* **`thread`**: Spawns a new background thread to run a block of code concurrently alongside the main program. It returns a thread handle.
  ```flame
  let handle = thread {
      print("Running in the background!")
      return 42
  }
  ```

* **`await`**: A prefix operator used to wait for an asynchronous task or a spawned thread to finish. It pauses the current execution context until the value is ready.
  ```flame
  async fn do_work() {
      let handle = thread {
          // heavy computation
          return 100
      }
      
      // The current thread waits here until the background thread finishes
      let result = await handle 
      print($"Result was {result}")
  }
  ```
  *(Note: `await` is a prefix operator in flame, meaning it goes **before** the variable, unlike Rust's postfix `.await`)*.

---

## 7. Modules and Standard Library

* **`import`**: Includes code from other modules.
  * **`std`**: The standard library prefix (e.g., `import std.fs` for file system, `import std.thread`, `import std.math`).
  * **`native`**: Used to import native Rust plugins/dependencies (e.g., `import native.mysql`).

  ```flame
  import std.fs
  
  fn read_config() {
      let data = fs.read_to_string("config.json")
      print(data)
  }
  ```

* **`@plugin`**: A macro/decorator used in manifest or code configurations to declare external dependencies.

---

## 8. Built-in I/O Functions

* **`print`**: Prints a value to standard output, followed by a newline.
  ```flame
  print("Normal log message")
  ```
* **`eprint`**: Prints a value to standard error, followed by a newline. Useful for warnings and diagnostics.
  ```flame
  eprint("Failed to load file!")
  ```
