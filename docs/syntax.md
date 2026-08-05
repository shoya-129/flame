# Flame Syntax and Language Guide

Flame is a modern, statically typed language that combines performance with
expressive, high-level abstractions. This document provides a comprehensive
guide to all built-in types, keywords, control flow mechanics, and concurrent
programming models available in Flame.

## Table of Contents

- [1. Data Types](#1-data-types)
  - [Primitive Types](#primitive-types) (`Int`, `Float`, `String`, `Bool`,
    `Nil`)
  - [Composite Types](#composite-types) (`Vec<T>`, `Tuples`, `formula`)
  - [References (Borrowing)](#references-borrowing) (`&T`, `&mut T`)
- [2. Variables and Constants](#2-variables-and-constants) (`let`, `mut`,
  `const`)
- [3. Functions](#3-functions) (`fn`, `async fn`)
- [4. Control Flow](#4-control-flow)
  - [Conditionals](#conditionals) (`if`, `else`, `match`)
  - [Loops](#loops) (`while`, `loop`, `for`, `in`)
  - [Flow Modifiers](#flow-modifiers) (`break`, `continue`, `return`, `defer`)
- [5. Object-Oriented and Custom Types](#5-object-oriented-and-custom-types)
  (`struct`, `impl`, `enum`, `trait`)
- [6. Concurrency (`thread`, `await`)](#6-concurrency-thread-await)
- [7. Modules and Standard Library](#7-modules-and-standard-library) (`import`)
- [8. Built-in I/O Functions](#8-built-in-io-functions) (`print`, `eprint`,
  `input`)
- [9. Explicit Type Conversion Methods](#9-explicit-type-conversion-methods)
  (`.toInt()`, `.toString()`, `.toFloat()`, etc.)
- [10. Annotated Functions and Testing Framework](#10-annotated-functions-and-testing-framework)
  (`@Test`, `@Setup`, `@Cleanup`, `@BeforeAll`, `@AfterAll`, `annotation`)

---

## 1. Data Types

Flame is statically typed. Types are either inferred or explicitly declared.

### Primitive Types

- **`Int`**: A 64-bit signed integer.
  - _Usage_: `let age: Int = 25`
- **`Float`**: A 64-bit floating-point number.
  - _Usage_: `let pi: Float = 3.1415`
- **`String`**: A UTF-8 encoded text string. Supports standard and interpolated
  strings.
  - _Usage_: `let name = "Alice"`
  - _Interpolation_: `let greeting = $"Hello, {name}!"`
- **`Bool`**: A boolean value (`true` or `false`).
  - _Usage_: `let is_active = true`
- **`Nil`**: Represents the absence of a value (similar to `null` or `void`).

### Composite Types

- **`Vec<T>`**: A dynamically-sized array (vector) containing elements of type
  `T`.
  - _Usage_: `let mut numbers: Vec<Int> = [1, 2, 3]`
  - _Methods_:
    - `push(value)`: Appends an element to the back of the collection.
    - `pop()`: Removes the last element from a vector and returns it.
    - `len()`: Returns the number of elements in the vector.
    - `map(closure)`: Transforms each element of the collection using the
      provided closure and returns a new collection. Example:
      `let mapped = numbers.map((x: Int) { return x * 2 })`
    - `filter(closure)`: Filters the collection using the provided closure,
      retaining only elements for which it returns true. Example:
      `let filtered = numbers.filter((x: Int) { return x > 1 })`
- **Tuples**: Fixed-size, ordered collections of potentially different types.
  - _Usage_: `let coordinates: (Int, String) = (10, "North")`
- **`formula` (Map/Dictionary)**: A structured map-like data literal, ideal for
  configuration, JSON-like objects, or named key-value collections. Keys must be
  identifiers, and values can be literals, lists, closures (anonymous
  functions), or nested formulas. Duplicate keys are supported; if a key is
  assigned multiple times within the same formula, the latter value overwrites
  the previous one without needing the `mut` keyword.
  - _Usage_:
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

*For more in-depth documentation on how Borrowing and Ownership works, please refer to the [Index Guide](README.md).*

---

## 2. Variables and Constants

- **`let`**: Declares a standard, immutable variable. Once assigned, it cannot
  be changed.
  ```flame
  let name = "John"
  // name = "Doe" // Error: cannot mutate immutable variable
  ```
- **`mut`**: Used in conjunction with `let` to declare a variable whose value
  can be mutated.
  ```flame
  let mut counter = 0
  counter = counter + 1
  ```
- **`const`**: Declares a compile-time constant. Must include an explicit type.
  ```flame
  const MAX_USERS: Int = 100
  ```

---

## 3. Functions

- **`fn`**: Defines a function. You must specify parameter types and optionally
  the return type.
  ```flame
  fn add(a: Int, b: Int) -> Int {
      return a + b
  }
  ```

- **`async fn`**: Defines an asynchronous function that returns a Future and
  does not block the thread.
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

- **`if` / `else`**: Standard conditional branching.
  ```flame
  if score > 90 {
      print("A")
  } else if score > 80 {
      print("B")
  } else {
      print("C")
  }
  ```

- **`match`**: Powerful pattern matching against variables or enums.
  ```flame
  let state = 1
  match state {
      0 => print("Stopped"),
      1 => print("Running"),
      _ => print("Unknown") // `_` is the default/fallback case
  }
  ```

### Loops

- **`while`**: Loops as long as a condition evaluates to `true`.
  ```flame
  let mut x = 0
  while x < 5 {
      x = x + 1
  }
  ```

- **`loop`**: Creates an infinite loop. Must be explicitly broken.
  ```flame
  loop {
      print("Running forever...")
      break // explicitly exit the loop
  }
  ```

- **`for` / `in`**: Iterates over a collection or range.
  ```flame
  let items = [1, 2, 3]
  for item in items {
      print(item)
  }
  ```

### Flow Modifiers

- **`break`**: Exits the innermost loop immediately.
- **`continue`**: Skips the rest of the current loop iteration and moves to the
  next.
- **`return`**: Returns a value from the current function and exits it.
- **`defer`**: Schedules a block of code to run at the exact moment the current
  scope closes. Useful for cleaning up resources, closing files, or unlocking
  mutexes.
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

- **`struct`**: Defines a custom data structure with named fields.
  ```flame
  struct User {
      name: String,
      age: Int
  }
  ```

- **`impl`**: Defines methods (functions attached to a type) for a `struct` or
  `enum`.
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

- **`enum`**: Defines a type that can be one of several named variants. Variants
  can optionally hold data.
  ```flame
  enum ConnectionState {
      Disconnected,
      Connecting(String), // holds an IP string
      Connected
  }
  ```

- **`trait`**: Defines a shared interface (a contract) that multiple structs or
  enums can implement.
  ```flame
  trait Drawable {
      fn draw(&self)
  }
  ```

---

## 6. Concurrency (`thread`, `await`)

Flame handles concurrency using a thread-based background task system.

- **`thread`**: Spawns a new background thread to run a block of code
  concurrently alongside the main program. It returns a thread handle.
  ```flame
  let handle = thread {
      print("Running in the background!")
      return 42
  }
  ```

- **`await`**: A prefix operator used to wait for an asynchronous task or a
  spawned thread to finish. It pauses the current execution context until the
  value is ready.
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
  _(Note: `await` is a prefix operator in flame, meaning it goes **before** the
  variable, unlike Rust's postfix `.await`)_.

---

## 7. Modules and Standard Library

- **`export`**: Exposes functions, structs, or constants so they can be imported
  and used by other files.
- **`import`**: Includes code from other modules or local files.
  - **Local Files**: You can import other `.fm` files in the same directory by
    simply using their filename (without the `.fm` extension).
  - **Cross-Folder Local Files**: You can import modules from other folders by
    their relative module path, such as `import src.exports` or
    `import tests.helpers`.
  - **`std`**: The standard library prefix (e.g., `import std.fs` for file
    system, `import std.thread`, `import std.math`).
    - _For a complete list of standard library modules and functions, please see
      the [Standard Library Documentation](stddocs.md)._
  - **`native`**: Used to import native Rust plugins/dependencies (e.g.,
    `import native.mysql`).

### Export and Import Example

**`hello.fm`**

```flame
export fn sayhello(user: formula) -> String {
    print($"hello {user.name}-{user.id}")
}
```

**`main.fm`**

```flame
import hello

fn main() {
    let my_user = formula { name: "Alice", id: 123 }
    hello.sayhello(my_user)
}
```

### Import Resolution Rules

Flame resolves imported modules from the current file, common project source
folders (`src/`, `tests/`, `examples/src/`, `examples/tests/`), and then the
project root. This means the following work as expected:

```flame
import exports
import src.exports
import tests.helpers
import utils.math
```

The compiler treats imported modules as namespaces. All exported symbols inside
the imported file become members of the namespace:

```flame
imports.sayHello()
imports.VERSION
imports.MyStruct
```

Exported annotations are also re-exported into the importing file, so you can
write:

```flame
import orm

@Entity("users")
formula User {
    id: Int
    name: String
}
```

The annotation does not require module qualification when imported.

- **`@plugin`**: A macro/decorator used in manifest or code configurations to
  declare external dependencies.

---

## 8. Built-in I/O Functions

- **`print`**: Prints a value to standard output, followed by a newline.
  ```flame
  print("Normal log message")
  ```
- **`eprint`**: Prints a value to standard error, followed by a newline. Useful
  for warnings and diagnostics.
  ```flame
  eprint("Failed to load file!")
  ```
- **`input`**: Prompts the user with a message and reads a line of text from
  standard input.
  ```flame
  let name = input("Enter your name: ")
  ```

---

## 9. Explicit Type Conversion Methods

Flame provides a robust set of universal type conversion methods on values,
ensuring precise control over data transformations without reliance on implicit
casting.

### String Conversions

- **`.toInt(radix: Int)`**: Parses a string into an integer using an optional
  radix between 2 and 36 (defaults to base 10). Throws a runtime error if
  parsing fails.
  ```flame
  let a = "42".toInt()         // 42
  let hex = "1A".toInt(16)     // 26
  let bin = "1010".toInt(2)    // 10
  ```
- **`.tryInt(radix: Int)`**: Non-panicking version of `.toInt()`. Returns `Nil`
  if the conversion fails instead of throwing an error.
  ```flame
  let valid = "100".tryInt()   // 100
  let invalid = "abc".tryInt() // Nil
  ```
- **`.toFloat()` / `.toDouble()`**: Converts a string representation of a
  decimal number into a `Float`.
  ```flame
  let pi = "3.14159".toFloat()
  ```
- **`.tryFloat()` / `.tryDouble()`**: Non-panicking version of `.toFloat()`.
  Returns `Nil` upon invalid formatting.
- **`.toBool()` / `.tryBool()`**: Case-insensitive boolean parsing for strings
  such as `"true"`, `"false"`, `"1"`, or `"0"`.
- **`.toBytes()`**: Converts a string into an array of UTF-8 byte integers
  (`Vec<Int>`).

### Universal & Primitive Conversions

- **`.toString(precision: Int)`**: Converts any value (primitives, structs,
  formulas, or vectors) into its human-readable String representation. When
  invoked on a `Float`, an optional `precision` parameter specifies exact
  decimal truncation.
  ```flame
  let pi_str = 3.14159265.toString(2)  // "3.14"
  let list_str = [1, 2, 3].toString()  // "[1, 2, 3]"
  ```
- **`.toChar()`**: Converts an integer ASCII/Unicode code point into its
  single-character string representation.

---

## 10. Annotated Functions and Testing Framework

Flame features a native, zero-cost testing and metadata system powered by
**Annotated Functions**. Annotations act as declarative metadata decorators
applied above function declarations. All built-in testing annotations utilize
PascalCase starting with a capital letter (e.g., `@Test`, `@Setup`, `@Cleanup`,
`@BeforeAll`, `@AfterAll`) and are highlighted in distinct semantic red within
IDEs.

### Built-in Testing Annotations

The integrated test runner (`flame test`) discovers and executes annotated test
suites automatically across project directories (`src/`, `tests/`).

- **`@Test`**: Marks a test function. Supports advanced configuration
  parameters:
  - `timeout`: Maximum permitted runtime in milliseconds before aborting (e.g.,
    `@Test(timeout: 3000)`).
  - `skip`: Disables execution (`@Test(skip: true)` or `@Ignore`).
  - `only`: Isolates test suite execution to marked functions only
    (`@Test(only: true)` or `@Only`).
  - `tags`: Categorical markers for filtering (`@Test(tags: ["db", "auth"])`).
- **`@Setup` / `@Cleanup`**: Executes lifecycle routines before and after
  _every_ individual test function in the scope (analogous to `beforeEach` and
  `afterEach`).
- **`@BeforeAll` / `@AfterAll`**: Executes global initialization (e.g., spawning
  test database servers) and teardown logic exactly _once_ per test suite run.
- **`@Parameterized`**: Automatically unrolls a single test function across an
  array of parameter sets.
  ```flame
  @Parameterized([
      [1, 2, 3],
      [5, 5, 10]
  ])
  fn test_addition(a: Int, b: Int, expected: Int) {
      assert_eq(a + b, expected)
  }
  ```

### Built-in Test Helpers

Flame includes a small suite of native assertion and mock helpers for tests and
application code.

- `assert(condition, message: String = "assertion failed")`
- `assert_true(condition, message: String = "assertion failed")`
- `assert_false(condition, message: String = "assertion failed")`
- `assert_eq(actual, expected, message: String = "")`
- `assert_ne(actual, unexpected, message: String = "")`
- `mock_data(schema: String) -> Formula`
- `mock_api(url: String, body: String = "{\"status\": \"ok\"}", status: Int = 200) -> Formula`
- `mock_function(function_name: String, return_value: Any)`

```flame
@Test
fn test_mock_and_assertions() {
    let user = mock_data("user")
    assert_eq(user.id, 1001)
    let response = mock_api("/ping")
    assert_true(response.ok)
    mock_function("fetch_user", formula { id: 1, name: "Test" })
}
```

### Zero-Cost Production Stripping

To ensure optimal performance and minimal memory footprints, all functions
marked with test annotations (`@Test`, `@Setup`, `@Cleanup`, `@BeforeAll`,
`@AfterAll`, `@Benchmark`, `@Parameterized`, `@Ignore`, `@Only`) are
**automatically stripped during production compilation**
(`flame build --release` and standard `flame run`). Test fixtures and
dependencies are included strictly when invoked via `flame test`.

### Custom Annotations & `annotation` Keyword

Developers can declare custom annotations using the dedicated **`annotation`**
keyword. Like functions, annotations accept strict typed parameters, return
composite metadata payloads such as formulas, and fully integrate with Flame's
ownership and borrow checker models.

```flame
annotation Benchmark(name: String, iterations: Int) -> Formula {
    return formula { name: name, count: iterations }
}

@Benchmark(name: "matrix_multiplication", iterations: 1000)
fn multiply_matrices() {
    // High-performance numerical routine
}
```

### Exported Annotations

Annotations can be exported from one module and imported into another. Because
annotations are first-class symbols, they are usable without module
qualification after import.

**`orm.fm`**

```flame
export annotation Entity(table: String) -> String {
    print("Registering entity: {table}")
    return table
}
```

**`models.fm`**

```flame
import orm

@Entity("users")
struct User {
    id: Int
    name: String
}
```

This means the hover information for imported annotations works just like local
annotations, showing parameter names, default values, and return metadata.

### CLI Annotations (Experimental)

Flame can also document CLI-oriented annotations in the same style:

```flame
@Cli(name: "flame", version: "0.1.0", description: "Flame toolchain")
fn main(cli: Cli) {
    match cli {
        build { release, target } => build(release, target)
        run { release, args } => run(release, args)
        help => print("help")
        _ => print("Unknown command")
    }
}

@Command(name: "build", about: "Compile the current project")
fn build(release: Bool = false, target: String?, output: String?) {
}

@Command(name: "run", about: "Run the current project")
fn run(release: Bool = false, args: Vec<String> = []) {
}
```

Recommended model:

- `@Cli(...)` declares the command root.
- `@Command(...)` declares typed subcommands.
- `match cli { ... }` destructures the parsed command tree.
- Bool parameters act like flags, while typed parameters act like options or
  positional arguments depending on defaults and ordering.

### Native Runtime Persistence

For native Rust async bridges, daemon-style runtime persistence is now intended
to be explicit instead of guessed from names like `run`, `serve`, or `listen`.
Mark only true long-lived listeners with a Flame attribute in Rust:

```rust
#[flame(runtime)]
pub async fn listen(...) { ... }

#[flame(daemon)]
pub async fn serve(...) { ... }
```

Only functions marked this way should keep the Flame runtime alive after the
main script finishes. Ordinary async helpers should exit normally.

### IDE Hover & Semantic Highlighting

In Flame IDE integrations:

- Annotated decorators (such as `@Test` and `@Setup`) and custom annotation
  definitions appear with striking red syntax highlighting.
- Hovering over decorated functions or annotation references displays complete
  function signatures including exact parameter names and types (e.g.,
  `fn about(name: String) -> String`), accompanied by comprehensive inline
  markdown documentation and execution characteristics.
- Built-in annotation hovers include parameter shapes for decorators such as
  `@Test(timeout: Int = 5000, skip: Bool = false, only: Bool = false, tags: Vector<String> = [])`,
  `@Cli(name: String, version: String = "0.1.0", description: String = "")`,
  and `@Command(name: String, about: String = "")`.
