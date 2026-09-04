use crate::{JsonCompletion, JsonHover};
use regex::Regex;

const KEYWORDS: &[(&str, &str)] = &[
    ("let", "Declares a local variable. Example: `let x = 5`"),
    ("const", "Declares a constant. Example: `const x = 5`"),
    (
        "mut",
        "Keyword for mutable reference/variable. Example: `let mut x = 5`",
    ),
    ("fn", "Declares a function. Example: `fn do_something() {}`"),
    (
        "async",
        "Declares an asynchronous function. Example: `async fn do_something() {}`",
    ),
    (
        "struct",
        "Declares a struct. Example: `struct Point { x: Num, y: Num }`",
    ),
    (
        "enum",
        "Declares an enum. Example: `enum Color { Red, Green, Blue }`",
    ),
    (
        "trait",
        "Declares a trait (interface). Example: `trait Drawable { fn draw(); }`",
    ),
    (
        "impl",
        "Implements methods for a struct or enum. Example: `impl Point { fn new() -> Point {} }`",
    ),
    (
        "if",
        "Conditional execution. Example: `if condition { ... }`",
    ),
    (
        "else",
        "Alternative conditional execution. Example: `else { ... }`",
    ),
    (
        "and",
        "Logical AND operator. Evaluates to true only if both operands are true. Example: `if is_online and is_ready { ... }`",
    ),
    (
        "or",
        "Logical OR operator. Evaluates to true if either operand is true. Example: `if is_offline or is_faulted { ... }`",
    ),
    (
        "not",
        "Logical NOT keyword. Inverts a boolean expression. Example: `if not is_faulted { ... }`",
    ),
    (
        "?:",
        "Nil-coalescing operator (Elvish operator). Returns the left-hand side if non-nil, otherwise evaluates and returns the fallback right-hand expression. Example: `let speed = motor?.speed ?: 0`",
    ),
    (
        "?.",
        "Safe-navigation member access operator. Evaluates member access if receiver is non-nil, otherwise short-circuits to nil without panicking. Example: `let speed = motor?.speed`",
    ),
    (
        "++",
        "Increment operator (supports prefix `++var` and postfix `var++`). Example: `count++`",
    ),
    (
        "--",
        "Decrement operator (supports prefix `--var` and postfix `var--`). Example: `timeout--`",
    ),
    (
        "+=",
        "Compound addition assignment operator. Example: `speed += 10`",
    ),
    (
        "-=",
        "Compound subtraction assignment operator. Example: `speed -= 10`",
    ),
    (
        "while",
        "Looping construct based on condition. Example: `while condition { ... }`",
    ),
    (
        "loop",
        "Infinite looping construct. Example: `loop { ... }`",
    ),
    (
        "for",
        "Iterating construct. Example: `for i in 0..10 { ... }`",
    ),
    (
        "in",
        "Used in for-loops or containment checks. Example: `for x in list`",
    ),
    (
        "as",
        "Type casting or import alias keyword. Example: `let n = val as Int`",
    ),
    ("type", "Defines a type alias. Example: `type UserID = Int`"),
    (
        "where",
        "Generic type constraint clause. Example: `fn test<T>(x: T) where T: Debug {}`",
    ),
    (
        "match",
        "Pattern matching. Example: `match x { 1 => ..., _ => ... }`",
    ),
    ("return", "Returns from a function. Example: `return 5`"),
    ("break", "Breaks out of a loop. Example: `break`"),
    (
        "continue",
        "Skips to the next iteration of a loop. Example: `continue`",
    ),
    (
        "defer",
        "Defers execution until the end of the scope. Example: `defer file.close()`",
    ),
    ("import", "Imports a module. Example: `import std.fs`"),
    (
        "export",
        "Exports a function, struct, enum, or annotation for external modules. Example: `export fn process() {}`",
    ),
    (
        "package",
        "Declares the package namespace for the current file. Required for folder-based imports. Example: `package utils`",
    ),
    (
        "native",
        "Native dependencies import prefix. Example: `import native.mysql`",
    ),
    (
        "std",
        "Standard library import prefix. Example: `import std.math`",
    ),
    (
        "thread",
        "Spawns a new background thread block. Example: `thread { thread.sleep(3000) }`",
    ),
    (
        "await",
        "Waits for an asynchronous task or thread to complete. Example: `await task`",
    ),
    (
        "println",
        "```flame\nfn println(value: Any)\n```\nPrints a value to standard output followed by a newline.\n\n**Example:**\n```flame\nprintln(\"Hello, Flame!\")\n```",
    ),
    (
        "print",
        "```flame\nfn print(value: Any)\n```\nPrints a value to standard output without trailing newline.\n\n**Example:**\n```flame\nprint(\"Processing...\")\n```",
    ),
    (
        "eprint",
        "```flame\nfn eprint(value: Any)\n```\nPrints a value to standard error.\n\n**Example:**\n```flame\neprint(\"Error: operation failed\")\n```",
    ),
    (
        "assert",
        "```flame\nfn assert(condition: Bool, message: String = \"\")\n```\nAsserts that condition evaluates to `true`. Terminates execution with an error if false.\n\n**Example:**\n```flame\nassert(x > 0, \"x must be positive\")\n```",
    ),
    (
        "assertEq",
        "```flame\nfn assertEq(actual: Any, expected: Any, message: String = \"\")\n```\nAsserts that `actual` equals `expected`. Terminates execution with an error if values differ.\n\n**Example:**\n```flame\nassertEq(status_code, 200)\n```",
    ),
    (
        "assertNe",
        "```flame\nfn assertNe(actual: Any, unexpected: Any, message: String = \"\")\n```\nAsserts that `actual` does NOT equal `unexpected`.\n\n**Example:**\n```flame\nassertNe(result, nil)\n```",
    ),
    (
        "assertTrue",
        "```flame\nfn assertTrue(condition: Bool, message: String = \"\")\n```\nAsserts that `condition` is `true`.\n\n**Example:**\n```flame\nassertTrue(list.is_empty())\n```",
    ),
    (
        "assertFalse",
        "```flame\nfn assertFalse(condition: Bool, message: String = \"\")\n```\nAsserts that `condition` is `false`.\n\n**Example:**\n```flame\nassertFalse(file.exists())\n```",
    ),
    (
        "panic",
        "```flame\nfn panic(message: String)\n```\nTerminates program execution immediately with an unrecoverable error message and diagnostic line trace. When called inside an `@Test` function, it halts the individual test case and marks it as failed with the specified message while allowing remaining test suites to proceed.\n\n**Example:**\n```flame\nif is_online and is_faulted {\n    panic(\"Invalid state: cannot be both online and faulted\")\n}\n```",
    ),
    (
        "typeof",
        "```flame\nfn typeof(value: Any) -> String\n```\nReturns the runtime type name of `value` as a String (e.g. `\"Int\"`, `\"String\"`, `\"Formula\"`).\n\n**Example:**\n```flame\nlet t = typeof(42)\n```",
    ),
    (
        "range",
        "```flame\nfn range(start: Int, end: Int, step: Int = 1) -> Vec<Int>\n```\nGenerates a vector of integers from `start` up to `end`.\n\n**Example:**\n```flame\nfor i in range(0, 5) { print(i) }\n```",
    ),
    (
        "sleep",
        "```flame\nfn sleep(ms: Int)\n```\nSuspends current thread execution for the specified milliseconds.\n\n**Example:**\n```flame\nsleep(1000)\n```",
    ),
    (
        "mockData",
        "```flame\nfn mockData(schema: String = \"default\") -> Formula\n```\nGenerates mock object data for testing. Supported schemas: `\"user\"`, `\"post\"`, `\"product\"`.\n\n**Example:**\n```flame\nlet user = mockData(\"user\")\n```",
    ),
    (
        "mockApi",
        "```flame\nfn mockApi(url: String = \"*\", body: String = \"{}\", status: Int = 200) -> Formula\n```\nConfigures mock responses for API endpoints during tests.\n\n**Example:**\n```flame\nlet res = mockApi(\"/api/v1/users\", \"{\\\"id\\\": 1}\", 200)\n```",
    ),
    (
        "mockFunction",
        "```flame\nfn mockFunction(name: String, return_value: Any)\n```\nOverrides a named function in the current environment to return `return_value` during tests.\n\n**Example:**\n```flame\nmockFunction(\"fetch_user\", formula { name: \"Alex\" })\n```",
    ),
    (
        "formula",
        "Declares a static map/dictionary structure. Example: `formula { key: \"value\" }`",
    ),
    (
        "@Application",
        "```flame\nannotation @Application(features: [\"String\"])\n```\n**Application Entry Point**\n\nMarks this function as the application's entry point. The function is invoked automatically when the program starts. Configuration options such as `features` enable optional standard library modules and control application-wide compiler/runtime behavior.",
    ),
    (
        "@Test",
        "```flame\nannotation @Test(timeout: Int = 1000, skip: Bool = false)\n```\n**Unit Test**\n\nMarks this function as a test case. The compiler will aggregate all `@Test` functions and execute them in a secure test harness when you run `flame test`.\n\n**Parameters:**\n- `timeout: Int`: Timeout in milliseconds. Test fails if execution exceeds this.\n- `skip: Bool`: If true, skips executing this test.",
    ),
    (
        "@Embedded",
        "```flame\nannotation @Embedded(target: String)\n```\n**Embedded Target Definition**\n\nDirects the compiler to emit machine code tailored for a specific microcontroller architecture, such as `arduino-uno` or `rp2040`.\n\n**Parameters:**\n- `target: String`: The hardware architecture target name.",
    ),
    (
        "@Cli",
        "```flame\nannotation @Cli\n```\n**CLI Application**\n\nMarks the application as a Command Line Interface tool, enabling automatic parsing of command line arguments into structures.",
    ),
    (
        "@Platform",
        "```flame\nannotation @Platform(target: String)\n```\n**Conditional Compilation**\n\nConditionally compiles the annotated declaration only if the active build target matches the given substring.\n\n**Example:**\n```flame\n@Platform(\"windows\")\nfn get_os_name() -> String {\n    \"Windows\"\n}\n```",
    ),
    (
        "@Docs",
        "```flame\nannotation @Docs(String...)\n```\n**Documentation Provider**\n\nProvides rich IDE hover documentation for functions, structs, and enums, supporting markdown syntax.\n\n**Example:**\n```flame\n@Docs(\"Computes the sum of two numbers.\")\nfn sum(a: Int, b: Int) -> Int {\n    a + b\n}\n```",
    ),
    (
        "@Requires",
        "```flame\nannotation @Requires(String...)\n```\n**Dependency Requirement**\n\nSpecifies system, hardware, or module dependencies required by this function or module. The compiler makes the dependency visible inside the function scope without globally importing it. It is loaded when the function executes and safely unloaded afterwards.\n\n**Example:**\n```flame\n@Requires(\"std.fs\")\n```",
    ),
    (
        "@Permission",
        "```flame\nannotation @Permission(String...)\n```\n**Access Permission**\n\nRequests specific runtime permissions (e.g., `\"net\"`, `\"fs\"`, `\"env\"`).\n\n**Rules:**\n- If no `@Permission` is specified anywhere in the project, permissions are auto-allowed.\n- If specified on any function, the user must explicitly allow all mentioned permissions at runtime (via terminal prompt or `flame.toml`), otherwise execution stops immediately.\n- When used on an `@Test` function, permissions are automatically granted.\n\n**Example:**\n```flame\n@Permission(\"net\", \"fs\")\n```",
    ),
    (
        "@Command",
        "```flame\nannotation @Command\n```\n**CLI Command**\n\nRegisters a function as an executable command within a `@Cli` application. Associates the function with a specific command-line keyword.",
    ),
    (
        "@Suggestions",
        "```flame\nannotation @Suggestions([{name: String, kind: String}])\n```\n**Package Suggestions**\n\nProvides custom suggestions for IDE autocompletion when typing the package name (e.g. `mypackage.`). Used inside `PackageDecl` to suggest objects or functions exported by the package.\n\n**Parameters:**\n- `args: Array`: An array of objects, where each object has `name` (the property to suggest) and `kind` (the completion kind, such as `\"function\"`, `\"method\"`, `\"property\"`, or `\"object\"`).",
    ),
    (
        "features",
        "**features: String[]**\n\nEnables optional standard library capabilities for the application. Enabled features are available throughout the program and only the required runtime dependencies are included in AOT builds.",
    ),
    (
        "Formula",
        "Built-in Type: A map-like literal data structure.",
    ),
    ("Int", "Built-in Type: A 64-bit signed integer."),
    ("Float", "Built-in Type: A 64-bit floating point number."),
    ("String", "Built-in Type: A UTF-8 text string."),
    ("Bool", "Built-in Type: A boolean value (true or false)."),
    ("Nil", "Built-in Type: Represents the absence of a value."),
    ("Vec", "Built-in Type: A dynamically-sized array."),
    (
        "ThreadHandler",
        "Built-in Type: A handle to a spawned background thread.",
    ),
    (
        "input",
        "```flame\nfn input(prompt: String = \"\") -> String\n```\nTakes a line of text input from standard input.\n\nExample:\n```flame\nlet name = input(\"Name: \")\n```",
    ),
    (
        "push",
        "Built-in Method: Appends an element to the back of a collection. Example: `arr.push(100)`",
    ),
    (
        "pop",
        "Built-in Method: Removes the last element from a collection and returns it. Example: `let last = arr.pop()`",
    ),
    (
        "len",
        "Built-in Method: Returns the number of elements in the collection or string. Example: `let l = arr.len()`",
    ),
    (
        "isEmpty",
        "Built-in Method: Returns true if the collection or string contains no elements. Example: `if arr.is_empty() { ... }`",
    ),
    (
        "filter",
        "Built-in Method: Creates a new array containing elements that pass the provided test function. Example: `let filtered = arr.filter((x: Int) { return x > 10 })`",
    ),
    (
        "map",
        "Built-in Method: Creates a new array populated with the results of calling a provided function on every element. Example: `let mapped = arr.map((x: Int) { return x * 2 })`",
    ),
    (
        "insert",
        "Built-in Method: Inserts or updates a key-value pair in a Formula map. Example: `map.insert(\"role\", \"admin\")`",
    ),
    (
        "get",
        "Built-in Method: Retrieves a value by key from a Formula map. Example: `let val = map.get(\"key\")`",
    ),
    (
        "remove",
        "Built-in Method: Removes a key from a Formula map. Example: `map.remove(\"key\")`",
    ),
    (
        "clone",
        "Built-in Method: Explicitly copies a value to prevent ownership move. Example: `let copy = val.clone()`",
    ),
    (
        "contains",
        "Built-in Method: Checks if a string contains the given substring. Example: `str.contains(\"pattern\")`",
    ),
    (
        "startsWith",
        "Built-in Method: Checks if a string begins with the given prefix. Example: `str.starts_with(\"prefix\")`",
    ),
    (
        "endsWith",
        "Built-in Method: Checks if a string ends with the given suffix. Example: `str.endsWith(\"suffix\")`",
    ),
    (
        "replace",
        "Built-in Method: Replaces occurrences of a substring with another. Example: `str.replace(\"old\", \"new\")`",
    ),
    (
        "trim",
        "Built-in Method: Strips leading and trailing whitespace from a string. Example: `str.trim()`",
    ),
    (
        "toUpperCase",
        "Built-in Method: Converts a string to uppercase. Example: `str.toUpperCase()`",
    ),
    (
        "toLowerCase",
        "Built-in Method: Converts a string to lowercase. Example: `str.to_lowercase()`",
    ),
    (
        "annotation",
        "Declares a reusable custom annotation function. Example: `annotation Benchmark(name: String) -> Formula {}`",
    ),
    (
        "Cli",
        "```flame\nannotation @Cli\n```\n**CLI Application**\n\nMarks the application as a Command Line Interface tool, enabling automatic parsing of command line arguments into structures.",
    ),
    (
        "Command",
        "```flame\nannotation @Command\n```\n**CLI Command**\n\nRegisters a function as an executable command within a `@Cli` application. Associates the function with a specific command-line keyword.",
    ),
    (
        "Test",
        "```flame\nannotation @Test(timeout: Int = 1000, skip: Bool = false)\n```\n**Unit Test**\n\nMarks this function as a test case. The compiler will aggregate all `@Test` functions and execute them in a secure test harness when you run `flame test`.\n\n**Parameters:**\n- `timeout: Int`: Timeout in milliseconds. Test fails if execution exceeds this.\n- `skip: Bool`: If true, skips executing this test.",
    ),
    (
        "Setup",
        "```flame\nannotation @Setup\n```\n**Test Setup**\n\nRuns before every test in the module (equivalent to `beforeEach()`).\nIgnored during `flame run` and `flame build`.",
    ),
    (
        "Cleanup",
        "```flame\nannotation @Cleanup\n```\n**Test Cleanup**\n\nRuns after every test in the module (equivalent to `afterEach()`).\nIgnored during `flame run` and `flame build`.",
    ),
    (
        "BeforeAll",
        "```flame\nannotation @BeforeAll\n```\n**Module Initialization**\n\nRuns exactly once before any test executes (e.g., database connection setup).\nIgnored during `flame run` and `flame build`.",
    ),
    (
        "AfterAll",
        "```flame\nannotation @AfterAll\n```\n**Module Teardown**\n\nRuns exactly once after all tests complete (e.g., closing servers or cleaning temp files).\nIgnored during `flame run` and `flame build`.",
    ),
    (
        "Ignore",
        "```flame\nannotation @Ignore\n```\n**Skip Test**\n\nSkips test execution when running `flame test`.",
    ),
    (
        "Only",
        "```flame\nannotation @Only\n```\n**Focus Test**\n\nRestricts test execution to ONLY functions marked with `@Only` during `flame test`.",
    ),
    (
        "Parameterized",
        "```flame\nannotation @Parameterized(arguments: Vector<Tuple>)\n```\n**Parameterized Test**\n\nExpands a test into multiple independent test cases, passing each tuple element as parameters to the test function.",
    ),
    (
        "Embedded",
        "```flame\nannotation @Embedded(target: String)\n```\n**Embedded Target Definition**\n\nDirects the compiler to emit machine code tailored for a specific microcontroller architecture, such as `arduino-uno` or `rp2040`.\n\n**Parameters:**\n- `target: String`: The hardware architecture target name.",
    ),
    (
        "Benchmark",
        "```flame\nannotation @Benchmark\n```\n**Performance Benchmark**\n\nExecutes the function as a high-precision performance benchmark during `flame test`, reporting average, minimum, and maximum execution times.",
    ),
    (
        "Requires",
        "```flame\nannotation @Requires(String...)\n```\n**Dependency Requirement**\n\nSpecifies system, hardware, or module dependencies required by this function or module. The compiler makes the dependency visible inside the function scope without globally importing it. It is loaded when the function executes and safely unloaded afterwards.\n\n**Example:**\n```flame\n@Requires(\"std.fs\")\n```",
    ),
    (
        "Permission",
        "```flame\nannotation @Permission(String...)\n```\n**Access Permission**\n\nRequests specific runtime permissions (e.g., `\"net\"`, `\"fs\"`, `\"env\"`).\n\n**Rules:**\n- If no `@Permission` is specified anywhere in the project, permissions are auto-allowed.\n- If specified on any function, the user must explicitly allow all mentioned permissions at runtime (via terminal prompt or `flame.toml`), otherwise execution stops immediately.\n- When used on an `@Test` function, permissions are automatically granted.\n\n**Example:**\n```flame\n@Permission(\"net\", \"fs\")\n```",
    ),
    (
        "ExpectPanic",
        "```flame\nannotation @ExpectPanic\n```\n**Expected Failure**\n\nAsserts that the test function MUST terminate with a panic or error; fails if the function completes successfully.",
    ),
    (
        "toInt",
        "```flame\nfn toInt(radix: Int = 10) -> Int\n```\nConverts a String value to a signed integer. Throws a runtime error if digits are invalid.",
    ),
    (
        "tryInt",
        "```flame\nfn tryInt(radix: Int = 10) -> Int | Nil\n```\nAttempts to parse a String value to an integer, returning `nil` if parsing fails.",
    ),
    (
        "toFloat",
        "```flame\nfn toFloat() -> Float\n```\nConverts a String value to a floating-point number.",
    ),
    (
        "tryFloat",
        "```flame\nfn tryFloat() -> Float | Nil\n```\nAttempts to parse a String value to a float, returning `nil` if parsing fails.",
    ),
    (
        "toDouble",
        "```flame\nfn toDouble() -> Float\n```\nConverts a String value to a double-precision floating-point number.",
    ),
    (
        "tryDouble",
        "```flame\nfn tryDouble() -> Float | Nil\n```\nAttempts to parse a String value to a double, returning `nil` if parsing fails.",
    ),
    (
        "toBool",
        "```flame\nfn toBool() -> Bool\n```\nConverts a String (\"true\", \"false\", \"1\", \"0\", etc.) to a boolean value.",
    ),
    (
        "tryBool",
        "```flame\nfn tryBool() -> Bool | Nil\n```\nAttempts to convert a String to a boolean value, returning `nil` if unrecognized.",
    ),
    (
        "toChar",
        "```flame\nfn toChar() -> String\n```\nReturns a string consisting of the first character.",
    ),
    (
        "toByte",
        "```flame\nfn toByte() -> Byte\n```\nConverts a String or Int into a binary Byte or Byte array.",
    ),
    (
        "toString",
        "```flame\nfn toString(precision: Int = -1) -> String\n```\nConverts any value (integer, float, boolean, nil, byte array) into its String representation. For floats, specifying precision limits decimal digits.",
    ),
    (
        "toUtf8",
        "```flame\nfn toUtf8() -> String\n```\nDecodes a Byte array into a UTF-8 String. Panics if the bytes are not valid UTF-8.",
    ),
    (
        "tryUtf8",
        "```flame\nfn tryUtf8() -> String?\n```\nAttempts to decode a Byte array into a UTF-8 String. Returns `nil` if the bytes are not valid UTF-8.",
    ),
    (
        "writeBytes",
        "```flame\nfn writeBytes(path: String, bytes: Byte)\n```\nWrites a byte array to a file, overwriting if it exists.",
    ),
    (
        "readBytes",
        "```flame\nfn readBytes(path: String) -> Byte\n```\nReads the entire contents of a file as a byte array.",
    ),
    (
        "appendBytes",
        "```flame\nfn appendBytes(path: String, bytes: Byte)\n```\nAppends a byte array to the end of a file.",
    ),
    (
        "writeByte",
        "```flame\nfn writeByte(path: String, byte: Int | Byte)\n```\nWrites a single byte (0-255) to a file.",
    ),
    (
        "readByte",
        "```flame\nfn readByte(path: String) -> Byte\n```\nReads a single byte from a file.",
    ),
    (
        "appendByte",
        "```flame\nfn appendByte(path: String, byte: Int | Byte)\n```\nAppends a single byte (0-255) to a file.",
    ),
    (
        "writeByteAt",
        "```flame\nfn writeByteAt(path: String, offset: Int, byte: Int | Byte)\n```\nWrites a single byte to a file at a specific offset.",
    ),
    (
        "readByteAt",
        "```flame\nfn readByteAt(path: String, offset: Int) -> Byte\n```\nReads a single byte from a file at a specific offset.",
    ),
];

const EMBEDDED_LITERALS: &[(&str, &str, &str)] = &[
    (
        "arduino-uno",
        "Hardware Target (AVR ATmega328P)",
        "Zero-cost `#![no_std]` compiler target for classic Arduino Uno boards.",
    ),
    (
        "esp32",
        "Hardware Target (Xtensa/RISC-V)",
        "Zero-cost compiler target for ESP32 Wi-Fi & Bluetooth chips.",
    ),
    (
        "stm32",
        "Hardware Target (ARM Cortex-M)",
        "Zero-cost compiler target for STM32 32-bit microcontrollers.",
    ),
    (
        "rp2040",
        "Hardware Target (Raspberry Pi Silicon)",
        "Zero-cost compiler target for dual-core ARM Cortex-M0+ RP2040 chips.",
    ),
    (
        "atmega328p",
        "Hardware Target (Bare AVR IC)",
        "Direct chip compiler target for standalone ATmega328P microcontrollers.",
    ),
    (
        "mega",
        "Hardware Target (Arduino Mega 2560)",
        "Compiler target for ATmega2560 extended GPIO boards.",
    ),
    (
        "avr-nano",
        "Hardware Target (Arduino Nano)",
        "Compiler target for compact ATmega328P Nano breakout boards.",
    ),
    (
        "OUTPUT",
        "Pin Mode Parameter",
        "Configures digital pin as push-pull output driver.",
    ),
    (
        "INPUT",
        "Pin Mode Parameter",
        "Configures pin as high-impedance floating input.",
    ),
    (
        "INPUT_PULLUP",
        "Pin Mode Parameter",
        "Configures pin input with internal 20K pull-up resistor active.",
    ),
    (
        "PWM",
        "Pin Mode Parameter",
        "Configures pin for Pulse-Width Modulation wave output.",
    ),
    (
        "HIGH",
        "Digital Output Level",
        "Drives pin voltage to VCC (5V / 3.3V).",
    ),
    (
        "LOW",
        "Digital Output Level",
        "Drives pin voltage to Ground (0V).",
    ),
];

pub fn get_literal_completions(prefix: &str) -> Vec<JsonCompletion> {
    let clean = prefix.trim_matches('"').trim_matches('\'');
    EMBEDDED_LITERALS
        .iter()
        .filter(|(val, _, _)| val.starts_with(clean) || clean.is_empty())
        .map(|(val, kind, doc)| JsonCompletion {
            sort_text: None,
            label: format!("\"{}\"", val),
            kind: "value".to_string(),
            detail: kind.to_string(),
            documentation: Some(format!("```flame\n\"{}\"\n```\n**{}**\n{}", val, kind, doc)),
        })
        .collect()
}

pub fn get_keyword_completions(
    current_line: &str,
    raw_word: &str,
    prefix: &str,
    tc_opt: Option<&crate::typechecker::TypeChecker>,
) -> Vec<JsonCompletion> {
    let mut comps = Vec::new();

    if raw_word.starts_with('@') {
        let clean_prefix = prefix.trim_start_matches('@');

        // Workspace annotations first
        if let Some(tc) = tc_opt {
            for ann_name in &tc.annotations {
                if ann_name.starts_with(clean_prefix) || clean_prefix.is_empty() {
                    comps.push(JsonCompletion {
                        sort_text: Some("0_".to_string()),
                        label: format!("@{}", ann_name),
                        kind: "annotation".to_string(),
                        detail: "workspace annotation".to_string(),
                        documentation: tc
                            .hover_info
                            .values()
                            .find(|doc| doc.contains(&format!("annotation @{}", ann_name)))
                            .cloned(),
                    });
                }
            }
        }

        let annotations = [
            "@Application",
            "@Test",
            "@Embedded",
            "@Cli",
            "@Command",
            "@Requires",
            "@Permission",
            "@Suggestions",
            "@Docs",
            "@Platform",
        ];
        for ann in annotations {
            if ann.starts_with(raw_word) {
                let label = ann.to_string();
                let clean_label = ann.trim_start_matches('@');
                if clean_label.starts_with(clean_prefix) || clean_prefix.is_empty() {
                    comps.push(JsonCompletion {
                        sort_text: Some("1_".to_string()),
                        label,
                        kind: "annotation".to_string(),
                        detail: "built-in annotation".to_string(),
                        documentation: None,
                    });
                }
            }
        }
        return comps;
    }

    if current_line.contains("@Application")
        && current_line.contains("features")
        && current_line.contains('[')
    {
        let features = [
            "\"http\"", "\"tcp\"", "\"udp\"", "\"ws\"", "\"mqtt\"", "\"url\"",
        ];
        for feat in features {
            if feat.starts_with(prefix) || prefix.is_empty() || feat.contains(prefix) {
                // If prefix already has a quote, we don't want to insert double quotes.
                let label = if prefix.starts_with('"') {
                    feat.trim_start_matches('"').to_string()
                } else {
                    feat.to_string()
                };
                comps.push(JsonCompletion {
                    sort_text: None,
                    label,
                    kind: "value".to_string(),
                    detail: "feature module".to_string(),
                    documentation: None,
                });
            }
        }
        return comps;
    }

    comps.extend(
        KEYWORDS
            .iter()
            .filter(|(kw, _)| {
                let is_alphabetic = kw
                    .chars()
                    .all(|c| c.is_alphabetic() || c == '_' || c == '@');
                is_alphabetic && (kw.starts_with(prefix) || prefix.is_empty())
            })
            .map(|(kw, doc)| JsonCompletion {
                sort_text: if kw.starts_with('@') {
                    Some("2_".to_string())
                } else {
                    None
                },
                label: kw.to_string(),
                kind: if kw.starts_with('@') {
                    "annotation".to_string()
                } else {
                    "keyword".to_string()
                },
                detail: if kw.starts_with('@') {
                    "built-in annotation".to_string()
                } else {
                    "keyword".to_string()
                },
                documentation: Some(doc.to_string()),
            }),
    );

    if current_line.contains("target:") || current_line.contains("@Embedded") {
        let lits = get_literal_completions(prefix);
        for mut lit in lits {
            if prefix.starts_with('"') {
                lit.label = lit.label.trim_start_matches('"').to_string();
            }
            comps.push(lit);
        }
    }

    comps
}

pub fn get_keyword_hover(word: &str) -> Option<JsonHover> {
    let clean_word = word.trim_start_matches('@');
    let mut hover = KEYWORDS
        .iter()
        .find(|(kw, _)| *kw == word || kw.trim_start_matches('@') == clean_word)
        .map(|(kw, doc)| {
            let formatted_doc = if doc.starts_with("```") {
                doc.to_string()
            } else if let Some((desc, ex)) = doc.split_once("Example: `") {
                let clean_ex = ex.trim_end_matches('`');
                let kind = if desc.starts_with("Built-in Function:")
                    || desc.starts_with("Built-in Method:")
                {
                    "fn"
                } else {
                    "keyword"
                };
                format!(
                    "```flame\n{} {}\n```\n{}\n\n**Example:**\n```flame\n{}\n```",
                    kind,
                    kw,
                    desc.trim(),
                    clean_ex
                )
            } else if doc.starts_with("Built-in Type:") {
                format!("```flame\ntype {}\n```\n{}", kw, doc.trim())
            } else if doc.starts_with("Built-in Function:") || doc.starts_with("Built-in Method:") {
                format!("```flame\nfn {}\n```\n{}", kw, doc.trim())
            } else if kw.starts_with('@') {
                format!("```flame\nannotation {}\n```\n{}", kw, doc.trim())
            } else {
                format!("```flame\nkeyword {}\n```\n{}", kw, doc.trim())
            };

            JsonHover {
                label: kw.to_string(),
                documentation: Some(formatted_doc),
            }
        });

    if hover.is_none() {
        let clean_lit = word.trim_matches('"').trim_matches('\'');
        if let Some((val, kind, doc)) = EMBEDDED_LITERALS
            .iter()
            .find(|(val, _, _)| *val == clean_lit)
        {
            hover = Some(JsonHover {
                label: format!("\"{}\"", val),
                documentation: Some(format!("```flame\n\"{}\"\n```\n**{}**\n{}", val, kind, doc)),
            });
        }
    }
    hover
}

#[derive(Debug)]
pub struct ScannedVar {
    pub name: String,
    pub typ: Option<String>,
    pub doc: Option<String>,
}

#[derive(Debug)]
pub struct ScannedStruct {
    pub name: String,
    pub fields: Vec<(String, String)>,
    pub methods: Vec<String>,
}

fn extract_balanced_block(source: &str, open_brace_pos: usize) -> Option<&str> {
    let bytes = source.as_bytes();
    if bytes.get(open_brace_pos) != Some(&b'{') {
        return None;
    }
    let mut depth = 0;
    let mut in_string = false;
    let mut string_char = b'"';
    let mut escaped = false;
    let start_idx = open_brace_pos + 1;

    for i in open_brace_pos..bytes.len() {
        let b = bytes[i];
        if in_string {
            if escaped {
                escaped = false;
            } else if b == b'\\' {
                escaped = true;
            } else if b == string_char {
                in_string = false;
            }
            continue;
        }

        if b == b'"' || b == b'\'' {
            in_string = true;
            string_char = b;
            continue;
        }

        if b == b'{' {
            depth += 1;
        } else if b == b'}' {
            depth -= 1;
            if depth == 0 {
                return Some(&source[start_idx..i]);
            }
        }
    }
    None
}

fn strip_comments_and_strings(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut lexer = crate::lexer::Lexer::new(source);
    let mut last_idx = 0;

    loop {
        let t = lexer.next_token();
        if t.kind == crate::lexer::TokenKind::EOF {
            if last_idx < source.len() {
                out.push_str(&source[last_idx..]);
            }
            break;
        }

        match t.kind {
            crate::lexer::TokenKind::Comment
            | crate::lexer::TokenKind::StringLiteral
            | crate::lexer::TokenKind::InterpolatedStringContent
            | crate::lexer::TokenKind::StringEnd => {
                if t.span.start > last_idx {
                    out.push_str(&source[last_idx..t.span.start]);
                }
                for ch in source[t.span.start..t.span.end].chars() {
                    if ch == '\n' || ch == '\r' {
                        out.push(ch);
                    } else {
                        // preserve byte length matching where possible for ASCII
                        // but if it's a multi-byte char, spaces will mess up byte offset
                        // We push a space for each char. If we have multi-byte chars,
                        // this might drift. It's better to just push spaces equal to the byte len
                        for _ in 0..ch.len_utf8() {
                            out.push(' ');
                        }
                    }
                }
                last_idx = t.span.end;
            }
            _ => {}
        }
    }

    out
}

pub fn scan_document(content: &str) -> (Vec<ScannedVar>, Vec<ScannedStruct>) {
    let stripped = strip_comments_and_strings(content);
    let content = &stripped;
    let mut vars = Vec::new();
    let mut structs = Vec::new();

    // Scan for structs: `struct Name { field: type, ... }`
    let struct_header_re = Regex::new(r"struct\s+([a-zA-Z_]\w*)\s*\{").unwrap();
    let field_re = Regex::new(r"([a-zA-Z_]\w*)\s*:\s*([a-zA-Z_]\w*)").unwrap();
    for cap in struct_header_re.captures_iter(content) {
        let name = cap[1].to_string();
        let match_obj = cap.get(0).unwrap();
        let open_brace_pos = match_obj.end() - 1;
        let mut fields = Vec::new();
        if let Some(body) = extract_balanced_block(content, open_brace_pos) {
            for field_cap in field_re.captures_iter(body) {
                fields.push((field_cap[1].to_string(), field_cap[2].to_string()));
            }
        }
        structs.push(ScannedStruct {
            name,
            fields,
            methods: vec![],
        });
    }

    // Scan for impls: `impl Name { fn method(...) { ... } }`
    let impl_header_re = Regex::new(r"impl\s+([a-zA-Z_]\w*)\s*\{").unwrap();
    let fn_re = Regex::new(r"fn\s+([a-zA-Z_]\w*)\s*\(").unwrap();
    for cap in impl_header_re.captures_iter(content) {
        let name = cap[1].to_string();
        let match_obj = cap.get(0).unwrap();
        let open_brace_pos = match_obj.end() - 1;
        if let Some(body) = extract_balanced_block(content, open_brace_pos) {
            let mut methods = Vec::new();
            for fn_cap in fn_re.captures_iter(body) {
                methods.push(fn_cap[1].to_string());
            }
            if let Some(s) = structs.iter_mut().find(|s| s.name == name) {
                s.methods.extend(methods);
            } else {
                structs.push(ScannedStruct {
                    name,
                    fields: vec![],
                    methods,
                });
            }
        }
    }

    // Scan for variables: `let x = StructName.new(...)`, `let x: StructName = ...`, `let x = StructName { ... }`
    let var_re =
        Regex::new(r"(?:let|const)(?:\s+mut)?\s+([a-zA-Z_]\w*)(?:\s*:\s*([a-zA-Z_]\w*))?(?:\s*=\s*(?:(await)\s+)?(?:[a-zA-Z_]\w*\.)*([a-zA-Z_]\w*)(?:\.new|\s*\{|\s*\()?)?").unwrap();
    for cap in var_re.captures_iter(content) {
        let name = cap[1].to_string();
        let typ = cap.get(2).map(|m| m.as_str().to_string()).or_else(|| {
            if cap.get(3).is_some() {
                Some("Promise".to_string())
            } else {
                cap.get(4).map(|m| m.as_str().to_string())
            }
        });
        vars.push(ScannedVar {
            name,
            typ,
            doc: None,
        });
    }

    // Scan for parameters in functions or closures (naive): `name: Type` where Type starts with uppercase
    let param_re = Regex::new(r"\b([a-z_]\w*)\s*:\s*([A-Z]\w*)").unwrap();
    for cap in param_re.captures_iter(content) {
        vars.push(ScannedVar {
            name: cap[1].to_string(),
            typ: Some(cap[2].to_string()),
            doc: None,
        });
    }

    // Scan for annotations to inject implicit variables
    let annotation_re = Regex::new(r"@([a-zA-Z_]\w*)\s*(?:\(|$)").unwrap();
    for cap in annotation_re.captures_iter(content) {
        let name = cap[1].to_string();
        let var_name = name.to_lowercase();
        // Set typ to a special marker so main.rs knows it's from an annotation
        vars.push(ScannedVar {
            name: var_name.clone(),
            typ: Some(format!("annotation_plugin:{}", var_name)),
            doc: None,
        });
    }

    // Scan for variables with formula bodies to extract fields
    let formula_header_re =
        Regex::new(r"(?:let|const)(?:\s+mut)?\s+([a-zA-Z_]\w*)\s*=\s*formula\s*\{").unwrap();
    for cap in formula_header_re.captures_iter(content) {
        let name = cap[1].to_string();
        let match_obj = cap.get(0).unwrap();
        let open_brace_pos = match_obj.end() - 1;
        if let Some(body) = extract_balanced_block(content, open_brace_pos) {
            let mut fields = Vec::new();
            let formula_field_re = Regex::new(r"([a-zA-Z_]\w*)\s*:").unwrap();
            for field_cap in formula_field_re.captures_iter(body) {
                fields.push((field_cap[1].to_string(), "Unknown".to_string()));
            }
            let synthetic_type = format!("__formula_{}", name);
            structs.push(ScannedStruct {
                name: synthetic_type.clone(),
                fields,
                methods: vec!["toString".to_string(), "toString".to_string()],
            });
            // Overwrite or add to vars at the beginning so it is found first
            vars.insert(
                0,
                ScannedVar {
                    name,
                    typ: Some(synthetic_type),
                    doc: None,
                },
            );
        }
    }

    // Scan for function and annotation decls: `fn name(a: Type, b: Type)` or `annotation name(...) -> Ret`
    let fn_decl_re = Regex::new(
        r"(?:export\s+)?(?:async\s+)?(fn|annotation)\s+([a-zA-Z_]\w*)\s*\(([^)]*)\)(?:\s*->\s*([a-zA-Z0-9_<>, \t]+))?",
    )
    .unwrap();

    let mut annotation_returns = std::collections::HashMap::new();
    for cap in fn_decl_re.captures_iter(content) {
        let kind_kw = &cap[1];
        let name_str = &cap[2];
        let params_str = cap[3].trim();
        let ret_str = cap.get(4).map_or("()", |m| m.as_str().trim());

        let sig = if kind_kw == "annotation" {
            annotation_returns.insert(name_str.to_string(), ret_str.to_string());
            if ret_str == "()" {
                format!("annotation @{}({})", name_str, params_str)
            } else {
                format!("annotation @{}({}) -> {}", name_str, params_str, ret_str)
            }
        } else {
            if ret_str == "()" {
                format!("fn {}({})", name_str, params_str)
            } else {
                format!("fn {}({}) -> {}", name_str, params_str, ret_str)
            }
        };

        vars.push(ScannedVar {
            name: name_str.to_string(),
            typ: Some(sig),
            doc: None,
        });
    }

    // Scan for annotation usages: `@Component` -> injects `component: ReturnType`
    let ann_usage_re = Regex::new(r"@([A-Z]\w*)").unwrap();
    for cap in ann_usage_re.captures_iter(content) {
        let ann_name = &cap[1];
        let mut c = ann_name.chars();
        let lower_name = match c.next() {
            None => String::new(),
            Some(f) => f.to_lowercase().collect::<String>() + c.as_str(),
        };

        let typ = annotation_returns
            .get(ann_name)
            .cloned()
            .unwrap_or_else(|| ann_name.to_string());

        vars.push(ScannedVar {
            name: lower_name,
            typ: Some(typ),
            doc: None,
        });
    }

    (vars, structs)
}

pub fn get_native_module_def(module: &str) -> Option<crate::vm::NativeModuleDef> {
    let defs = crate::native_std::get_module_defs();
    let search = if module.starts_with("std.") {
        module.to_string()
    } else {
        format!("std.{}", module)
    };
    defs.into_iter()
        .find(|d| d.name == search || d.name == module)
}

pub fn get_std_module_methods(module: &str) -> Option<Vec<String>> {
    let mut parts = module.split('.');
    let base = parts.next()?;

    let base_module = if base == "std" {
        match parts.next() {
            Some(m) if !m.is_empty() => m,
            _ => {
                // If it's just "std" or "std.", suggest the standard modules
                return Some(
                    vec![
                        "thread",
                        "process",
                        "fs",
                        "byte",
                        "net",
                        "json",
                        "math",
                        "time",
                        "fmt",
                        "os",
                        "hardware",
                        "desktop",
                        "env",
                        "hid",
                        "camera",
                        "bluetooth",
                        "serial",
                        "embedded",
                        "unit",
                    ]
                    .into_iter()
                    .map(String::from)
                    .collect(),
                );
            }
        }
    } else {
        base
    };

    let mut map = match base_module {
        "thread" => Some(crate::native_std::thread::init()),
        "process" => Some(crate::native_std::process::init()),
        "fs" => Some(crate::native_std::fs::init()),
        "byte" => Some(crate::native_std::byte::init()),
        "net" => Some(crate::native_std::net::init(&parts.next()?)),
        "json" => Some(crate::native_std::json::init()),
        "math" => Some(crate::native_std::math::init()),
        "time" => Some(crate::native_std::time::init()),
        "fmt" => Some(crate::native_std::fmt::init()),
        "os" => Some(crate::native_std::os::init()),
        "hardware" => Some(crate::native_std::hardware::init()),
        "desktop" => Some(crate::native_std::desktop::init()),
        "env" => Some(crate::native_std::env::init()),
        "hid" => Some(crate::native_std::hid::init()),
        "camera" => Some(crate::native_std::camera::init()),
        "bluetooth" => Some(crate::native_std::bluetooth::init()),
        "serial" => Some(crate::native_std::serial::init()),
        "embedded" => Some(crate::native_std::embedded::init()),
        "unit" => Some(crate::native_std::unit::init()),
        _ => None,
    }?;

    for part in parts {
        match map.get(part) {
            Some(crate::vm::Value::Object(inner)) | Some(crate::vm::Value::Formula(inner)) => {
                map = inner.clone();
            }
            _ => return None,
        }
    }

    Some(map.keys().cloned().collect())
}

#[derive(serde::Serialize)]
pub struct SemanticToken {
    pub line: usize,
    pub col: usize,
    pub length: usize,
    pub token_type: usize,
    pub token_modifiers: usize,
}

pub fn get_semantic_tokens(source: &str) -> Vec<SemanticToken> {
    let mut tokens = Vec::new();
    let mut lexer = crate::lexer::Lexer::new(source);

    loop {
        let t = lexer.next_token();
        if t.kind == crate::lexer::TokenKind::EOF {
            break;
        }

        let mut token_type = None;
        let modifiers = 0;

        match t.kind {
            crate::lexer::TokenKind::Comment => {
                token_type = Some(3); // comment
            }
            crate::lexer::TokenKind::StringLiteral
            | crate::lexer::TokenKind::InterpolatedStringContent
            | crate::lexer::TokenKind::StringEnd => {
                token_type = Some(4); // string
            }
            crate::lexer::TokenKind::Annotation => {
                token_type = Some(0); // keyword
            }
            crate::lexer::TokenKind::Fn => {
                token_type = Some(0); // keyword
            }
            crate::lexer::TokenKind::Let
            | crate::lexer::TokenKind::Const
            | crate::lexer::TokenKind::Struct
            | crate::lexer::TokenKind::Enum
            | crate::lexer::TokenKind::Trait
            | crate::lexer::TokenKind::Impl
            | crate::lexer::TokenKind::Export
            | crate::lexer::TokenKind::Import
            | crate::lexer::TokenKind::Mut
            | crate::lexer::TokenKind::As
            | crate::lexer::TokenKind::Type
            | crate::lexer::TokenKind::Where
            | crate::lexer::TokenKind::Formula
            | crate::lexer::TokenKind::If
            | crate::lexer::TokenKind::Else
            | crate::lexer::TokenKind::Match
            | crate::lexer::TokenKind::For
            | crate::lexer::TokenKind::In
            | crate::lexer::TokenKind::While
            | crate::lexer::TokenKind::Loop
            | crate::lexer::TokenKind::Break
            | crate::lexer::TokenKind::Continue
            | crate::lexer::TokenKind::Defer
            | crate::lexer::TokenKind::Return
            | crate::lexer::TokenKind::Yield
            | crate::lexer::TokenKind::Await
            | crate::lexer::TokenKind::Async
            | crate::lexer::TokenKind::Thread
            | crate::lexer::TokenKind::Ampersand2
            | crate::lexer::TokenKind::Pipe2
            | crate::lexer::TokenKind::Exclamation
            | crate::lexer::TokenKind::True
            | crate::lexer::TokenKind::False
            | crate::lexer::TokenKind::Nil => {
                token_type = Some(0); // keyword
            }
            _ => {
                if t.kind == crate::lexer::TokenKind::Identifier
                    && (t.lexeme == "self" || t.lexeme == "Self")
                {
                    token_type = Some(0); // keyword
                }
            }
        }

        if let Some(ty) = token_type {
            tokens.push(SemanticToken {
                line: t.span.line.saturating_sub(1),
                col: t.span.col.saturating_sub(1),
                length: t.span.end.saturating_sub(t.span.start),
                token_type: ty,
                token_modifiers: modifiers,
            });
        }
    }

    tokens
}

#[derive(serde::Serialize, Clone, Debug, PartialEq, Eq)]
pub struct JsonDefinition {
    pub file: String,
    pub line: usize,
    pub column: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_column: Option<usize>,
}

pub fn locate_blaze_dir() -> Option<std::path::PathBuf> {
    use std::path::PathBuf;

    // 1. Environment variables
    if let Ok(env_path) = std::env::var("FLAME_BLAZE_DIR") {
        let p = PathBuf::from(env_path);
        if p.join("std").exists() {
            return Some(p);
        }
        if p.exists() {
            return Some(p);
        }
    }
    if let Ok(env_path) = std::env::var("BLAZE_HOME") {
        let p = PathBuf::from(env_path);
        if p.join("std").exists() {
            return Some(p);
        }
        if p.exists() {
            return Some(p);
        }
    }
    if let Ok(env_path) = std::env::var("FLAME_HOME") {
        let p = PathBuf::from(env_path).join("Blaze");
        if p.join("std").exists() {
            return Some(p);
        }
    }

    // 2. Current working directory and parent hierarchies (development / repo root)
    if let Ok(cwd) = std::env::current_dir() {
        let mut curr = Some(cwd.as_path());
        while let Some(dir) = curr {
            let blaze = dir.join("Blaze");
            if blaze.join("std").exists() {
                return Some(blaze);
            }
            let blaze_lower = dir.join("blaze");
            if blaze_lower.join("std").exists() {
                return Some(blaze_lower);
            }
            curr = dir.parent();
        }
    }

    // 3. Executable-relative
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let candidates = [
                exe_dir.join("Blaze"),
                exe_dir.join("blaze"),
                exe_dir.join("..").join("Blaze"),
                exe_dir.join("..").join("blaze"),
                exe_dir.join("..").join("share").join("blaze"),
                exe_dir.join("..").join("lib").join("blaze"),
            ];
            for cand in candidates {
                if cand.join("std").exists() {
                    return Some(cand);
                }
            }
        }
    }

    // 4. User and platform-standard directories
    #[cfg(target_os = "windows")]
    {
        if let Ok(prog_files) = std::env::var("ProgramFiles") {
            let p = PathBuf::from(prog_files).join("Blaze");
            if p.join("std").exists() {
                return Some(p);
            }
        }
        if let Ok(prog_files_x86) = std::env::var("ProgramFiles(x86)") {
            let p = PathBuf::from(prog_files_x86).join("Blaze");
            if p.join("std").exists() {
                return Some(p);
            }
        }
        if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
            let p = PathBuf::from(local_app_data).join("Blaze");
            if p.join("std").exists() {
                return Some(p);
            }
        }
        if let Ok(user_profile) = std::env::var("USERPROFILE") {
            let p = PathBuf::from(user_profile).join(".blaze");
            if p.join("std").exists() {
                return Some(p);
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(home) = std::env::var("HOME") {
            let p = PathBuf::from(home).join(".blaze");
            if p.join("std").exists() {
                return Some(p);
            }
        }
        let sys_candidates = [
            PathBuf::from("/usr/local/share/blaze"),
            PathBuf::from("/usr/share/blaze"),
            PathBuf::from("/opt/blaze"),
        ];
        for p in sys_candidates {
            if p.join("std").exists() {
                return Some(p);
            }
        }
    }

    // 5. Automatic self-healing: if no definition directory exists yet, populate default
    crate::blaze::ensure_blaze_definitions_installed()
}

pub fn find_declaration_in_stmts(
    stmts: &[crate::parser::Stmt],
    symbol: &str,
    filepath: &str,
) -> Option<JsonDefinition> {
    let clean = symbol.trim_start_matches('@');
    for stmt in stmts {
        match stmt {
            crate::parser::Stmt::FuncDecl {
                name, name_span, ..
            } if name == clean => {
                return Some(JsonDefinition {
                    file: filepath.to_string(),
                    line: name_span.line,
                    column: name_span.col,
                    end_line: Some(name_span.line),
                    end_column: Some(name_span.col + name.len()),
                });
            }
            crate::parser::Stmt::AnnotationDecl {
                name, name_span, ..
            } if name == clean => {
                return Some(JsonDefinition {
                    file: filepath.to_string(),
                    line: name_span.line,
                    column: name_span.col,
                    end_line: Some(name_span.line),
                    end_column: Some(name_span.col + name.len()),
                });
            }
            crate::parser::Stmt::StructDecl {
                name, name_span, ..
            } if name == clean => {
                return Some(JsonDefinition {
                    file: filepath.to_string(),
                    line: name_span.line,
                    column: name_span.col,
                    end_line: Some(name_span.line),
                    end_column: Some(name_span.col + name.len()),
                });
            }
            crate::parser::Stmt::EnumDecl {
                name, name_span, ..
            } if name == clean => {
                return Some(JsonDefinition {
                    file: filepath.to_string(),
                    line: name_span.line,
                    column: name_span.col,
                    end_line: Some(name_span.line),
                    end_column: Some(name_span.col + name.len()),
                });
            }
            crate::parser::Stmt::ConstDecl {
                name, name_span, ..
            } if name == clean => {
                return Some(JsonDefinition {
                    file: filepath.to_string(),
                    line: name_span.line,
                    column: name_span.col,
                    end_line: Some(name_span.line),
                    end_column: Some(name_span.col + name.len()),
                });
            }
            crate::parser::Stmt::LetDecl {
                name, name_span, ..
            } if name == clean => {
                return Some(JsonDefinition {
                    file: filepath.to_string(),
                    line: name_span.line,
                    column: name_span.col,
                    end_line: Some(name_span.line),
                    end_column: Some(name_span.col + name.len()),
                });
            }
            crate::parser::Stmt::ExportDecl(inner, _) => {
                if let Some(def) = find_declaration_in_stmts(&[(**inner).clone()], symbol, filepath)
                {
                    return Some(def);
                }
            }
            crate::parser::Stmt::ImplDecl { methods, .. } => {
                for m in methods {
                    if let crate::parser::Stmt::FuncDecl {
                        name, name_span, ..
                    } = m
                    {
                        if name == clean {
                            return Some(JsonDefinition {
                                file: filepath.to_string(),
                                line: name_span.line,
                                column: name_span.col,
                                end_line: Some(name_span.line),
                                end_column: Some(name_span.col + name.len()),
                            });
                        }
                    }
                }
            }
            _ => {}
        }
    }
    None
}

fn search_body_for_local_symbol(
    body: &[crate::parser::Stmt],
    clean: &str,
    cursor_line: usize,
    filepath: &str,
) -> Option<JsonDefinition> {
    for b_stmt in body {
        match b_stmt {
            crate::parser::Stmt::LetDecl {
                name, name_span, ..
            } => {
                if name == clean && name_span.line <= cursor_line {
                    return Some(JsonDefinition {
                        file: filepath.to_string(),
                        line: name_span.line,
                        column: name_span.col,
                        end_line: Some(name_span.line),
                        end_column: Some(name_span.col + name.len()),
                    });
                }
            }
            crate::parser::Stmt::ConstDecl {
                name, name_span, ..
            } => {
                if name == clean {
                    return Some(JsonDefinition {
                        file: filepath.to_string(),
                        line: name_span.line,
                        column: name_span.col,
                        end_line: Some(name_span.line),
                        end_column: Some(name_span.col + name.len()),
                    });
                }
            }
            crate::parser::Stmt::FuncDecl {
                name, name_span, ..
            } => {
                if name == clean {
                    return Some(JsonDefinition {
                        file: filepath.to_string(),
                        line: name_span.line,
                        column: name_span.col,
                        end_line: Some(name_span.line),
                        end_column: Some(name_span.col + name.len()),
                    });
                }
            }
            _ => {}
        }
    }
    None
}

pub fn find_local_symbol(
    stmts: &[crate::parser::Stmt],
    symbol: &str,
    filepath: &str,
    cursor_line: usize,
) -> Option<JsonDefinition> {
    let clean = symbol.trim_start_matches('@');

    // 1. Check enclosing function / impl method parameters and local declarations
    for stmt in stmts {
        if let crate::parser::Stmt::FuncDecl {
            params,
            body,
            span,
            ..
        } = stmt
        {
            if cursor_line >= span.line {
                for p in params {
                    if p.name == clean {
                        return Some(JsonDefinition {
                            file: filepath.to_string(),
                            line: span.line,
                            column: span.col,
                            end_line: Some(span.line),
                            end_column: Some(span.col + p.name.len()),
                        });
                    }
                }
                if let Some(inner_body) = body {
                    if let Some(def) = search_body_for_local_symbol(inner_body, clean, cursor_line, filepath) {
                        return Some(def);
                    }
                }
            }
        } else if let crate::parser::Stmt::ImplDecl { methods, .. } = stmt {
            for m in methods {
                if let crate::parser::Stmt::FuncDecl {
                    params,
                    body,
                    span,
                    ..
                } = m
                {
                    if cursor_line >= span.line {
                        for p in params {
                            if p.name == clean {
                                return Some(JsonDefinition {
                                    file: filepath.to_string(),
                                    line: span.line,
                                    column: span.col,
                                    end_line: Some(span.line),
                                    end_column: Some(span.col + p.name.len()),
                                });
                            }
                        }
                        if let Some(inner_body) = body {
                            if let Some(def) = search_body_for_local_symbol(inner_body, clean, cursor_line, filepath) {
                                return Some(def);
                            }
                        }
                    }
                }
            }
        }
    }

    // 2. Top-level declarations in current file
    find_declaration_in_stmts(stmts, symbol, filepath)
}

pub fn find_symbol_in_file(path: &std::path::Path, symbol: &str) -> Option<JsonDefinition> {
    let content = std::fs::read_to_string(path).ok()?;
    let path_str = path.to_string_lossy().to_string();

    let mut lexer = crate::lexer::Lexer::new(&content);
    let mut tokens = Vec::new();
    loop {
        let tok = lexer.next_token();
        let is_eof = tok.kind == crate::lexer::TokenKind::EOF;
        tokens.push(tok);
        if is_eof {
            break;
        }
    }
    let mut parser = crate::parser::Parser::new(tokens, path_str.clone());
    if let Ok(stmts) = parser.parse() {
        if let Some(def) = find_declaration_in_stmts(&stmts, symbol, &path_str) {
            return Some(def);
        }
    }

    // Fast fallback line-by-line pattern matching
    let clean = symbol.trim_start_matches('@');
    for (line_idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        let is_match = trimmed.starts_with(&format!("fn {}", clean))
            || trimmed.starts_with(&format!("export fn {}", clean))
            || trimmed.starts_with(&format!("annotation {}", clean))
            || trimmed.starts_with(&format!("export annotation {}", clean))
            || trimmed.starts_with(&format!("struct {}", clean))
            || trimmed.starts_with(&format!("export struct {}", clean))
            || trimmed.starts_with(&format!("enum {}", clean))
            || trimmed.starts_with(&format!("export enum {}", clean))
            || trimmed.starts_with(&format!("const {}:", clean))
            || trimmed.starts_with(&format!("export const {}:", clean))
            || trimmed.starts_with(&format!("let {}", clean))
            || trimmed.starts_with(&format!("export let {}", clean))
            || trimmed.contains(&format!("fn {}(", clean));

        if is_match {
            if let Some(col_idx) = line.find(clean) {
                return Some(JsonDefinition {
                    file: path_str,
                    line: line_idx + 1,
                    column: col_idx + 1,
                    end_line: Some(line_idx + 1),
                    end_column: Some(col_idx + 1 + clean.len()),
                });
            }
        }
    }

    None
}

pub fn find_symbol_in_rust_file(
    path: &std::path::Path,
    struct_name: Option<&str>,
    symbol: &str,
) -> Option<JsonDefinition> {
    let content = std::fs::read_to_string(path).ok()?;
    let path_str = path.to_string_lossy().to_string();

    // 1. If struct_name is provided, search inside `impl ... StructName` block
    if let Some(target_struct) = struct_name {
        let impl_pattern = format!(
            r"impl(?:\s*<[^>]+>)?\s+(?:[a-zA-Z_]\w*::)*{}\b",
            regex::escape(target_struct)
        );
        if let Ok(re_impl) = regex::Regex::new(&impl_pattern) {
            for m in re_impl.find_iter(&content) {
                if let Some(brace_offset) = content[m.end()..].find('{') {
                    let open_brace = m.end() + brace_offset;
                    if let Some(impl_body) = extract_balanced_block(&content, open_brace) {
                        let base_offset = open_brace + 1;
                        let fn_pattern = format!(
                            r"(?:pub(?:\s*\([^)]*\))?\s+)?(?:async\s+)?fn\s+{}\b",
                            regex::escape(symbol)
                        );
                        if let Ok(re_fn) = regex::Regex::new(&fn_pattern) {
                            if let Some(fn_match) = re_fn.find(impl_body) {
                                let sub = &impl_body[fn_match.start()..fn_match.end()];
                                let sym_offset = sub.find(symbol).unwrap_or(0);
                                let abs_offset = base_offset + fn_match.start() + sym_offset;
                                let line_idx = content[..abs_offset].chars().filter(|&c| c == '\n').count() + 1;
                                let last_line_start = content[..abs_offset].rfind('\n').map(|i| i + 1).unwrap_or(0);
                                let col_idx = content[last_line_start..abs_offset].chars().count() + 1;
                                return Some(JsonDefinition {
                                    file: path_str.clone(),
                                    line: line_idx,
                                    column: col_idx,
                                    end_line: Some(line_idx),
                                    end_column: Some(col_idx + symbol.len()),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    // 2. Search for fn symbol (free function or method in file)
    let fn_pattern = format!(
        r"(?:pub(?:\s*\([^)]*\))?\s+)?(?:async\s+)?fn\s+{}\b",
        regex::escape(symbol)
    );
    if let Ok(re_fn) = regex::Regex::new(&fn_pattern) {
        if let Some(fn_match) = re_fn.find(&content) {
            let sub = &content[fn_match.start()..fn_match.end()];
            let sym_offset = sub.find(symbol).unwrap_or(0);
            let abs_offset = fn_match.start() + sym_offset;
            let line_idx = content[..abs_offset].chars().filter(|&c| c == '\n').count() + 1;
            let last_line_start = content[..abs_offset].rfind('\n').map(|i| i + 1).unwrap_or(0);
            let col_idx = content[last_line_start..abs_offset].chars().count() + 1;
            return Some(JsonDefinition {
                file: path_str.clone(),
                line: line_idx,
                column: col_idx,
                end_line: Some(line_idx),
                end_column: Some(col_idx + symbol.len()),
            });
        }
    }

    // 3. Search for struct or enum: pub struct <symbol>, pub enum <symbol>
    let type_pattern = format!(
        r"(?:pub(?:\s*\([^)]*\))?\s+)?(?:struct|enum)\s+{}\b",
        regex::escape(symbol)
    );
    if let Ok(re_type) = regex::Regex::new(&type_pattern) {
        if let Some(type_match) = re_type.find(&content) {
            let sub = &content[type_match.start()..type_match.end()];
            let sym_offset = sub.find(symbol).unwrap_or(0);
            let abs_offset = type_match.start() + sym_offset;
            let line_idx = content[..abs_offset].chars().filter(|&c| c == '\n').count() + 1;
            let last_line_start = content[..abs_offset].rfind('\n').map(|i| i + 1).unwrap_or(0);
            let col_idx = content[last_line_start..abs_offset].chars().count() + 1;
            return Some(JsonDefinition {
                file: path_str,
                line: line_idx,
                column: col_idx,
                end_line: Some(line_idx),
                end_column: Some(col_idx + symbol.len()),
            });
        }
    }

    None
}

pub fn get_plugin_func_return_type(
    manifest_dir: &std::path::Path,
    plugin_name: &str,
    method_name: &str,
) -> Option<String> {
    // 1. Check .flame/pkg/<plugin_name>/<plugin_name>.fmi
    let fmi_path = manifest_dir
        .join(".flame")
        .join("pkg")
        .join(plugin_name)
        .join(format!("{}.fmi", plugin_name));
    if let Ok(content) = std::fs::read_to_string(&fmi_path) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(funcs) = json.get("functions").and_then(|f| f.as_array()) {
                for f in funcs {
                    if let (Some(name), Some(ret)) = (
                        f.get("flame_name")
                            .and_then(|n| n.as_str())
                            .or_else(|| f.get("name").and_then(|n| n.as_str())),
                        f.get("return_type").and_then(|r| r.as_str()),
                    ) {
                        if name == method_name {
                            return Some(ret.to_string());
                        }
                    }
                }
            }
        }
    }

    // 2. Check plugin source Rust files
    let toml_path = manifest_dir.join("flame.toml");
    if let Ok(content) = std::fs::read_to_string(&toml_path) {
        let entries = crate::package_manager::parse_section_entries(&content, "[plugins]");
        for (name, path_str) in entries {
            if name == plugin_name {
                let clean_path = path_str.trim().trim_matches('"').trim_matches('\'');
                let resolved = if clean_path.starts_with('.')
                    || clean_path.starts_with('/')
                    || clean_path.contains('\\')
                {
                    manifest_dir.join(clean_path)
                } else {
                    manifest_dir.join(".flame").join("pkg").join(&name)
                };
                let lib_rs = resolved.join("src").join("lib.rs");
                if let Ok(rs_content) = std::fs::read_to_string(&lib_rs) {
                    let fn_re = regex::Regex::new(&format!(
                        r"(?:pub(?:\s*\([^)]*\))?\s+)?(?:async\s+)?fn\s+{}\s*[^{{]*->\s*([a-zA-Z_]\w*)",
                        regex::escape(method_name)
                    ));
                    if let Ok(re) = fn_re {
                        if let Some(cap) = re.captures(&rs_content) {
                            return Some(cap[1].to_string());
                        }
                    }
                }
            }
        }
    }

    None
}

pub fn find_native_plugin_symbol(
    manifest_dir: &std::path::Path,
    plugin_name_or_alias: &str,
    struct_name: Option<&str>,
    symbol: &str,
) -> Option<JsonDefinition> {
    // 1. Locate plugin directory from flame.toml
    let toml_path = manifest_dir.join("flame.toml");
    let mut plugin_dir = None;
    if let Ok(content) = std::fs::read_to_string(&toml_path) {
        let entries = crate::package_manager::parse_section_entries(&content, "[plugins]");
        for (name, path_str) in entries {
            if name == plugin_name_or_alias {
                let clean_path = path_str.trim().trim_matches('"').trim_matches('\'');
                let resolved = if clean_path.starts_with('.')
                    || clean_path.starts_with('/')
                    || clean_path.contains('\\')
                {
                    manifest_dir.join(clean_path)
                } else {
                    manifest_dir.join(".flame").join("pkg").join(&name)
                };
                plugin_dir = Some(resolved);
                break;
            }
        }
    }

    if plugin_dir.is_none() {
        // Check .flame/pkg/<plugin_name_or_alias>
        let candidate = manifest_dir
            .join(".flame")
            .join("pkg")
            .join(plugin_name_or_alias);
        if candidate.exists() {
            plugin_dir = Some(candidate);
        }
    }

    if let Some(dir) = plugin_dir {
        let src_dir = dir.join("src");
        let lib_rs = src_dir.join("lib.rs");
        if lib_rs.is_file() {
            if let Some(def) = find_symbol_in_rust_file(&lib_rs, struct_name, symbol) {
                return Some(def);
            }
        }
        let main_rs = src_dir.join("main.rs");
        if main_rs.is_file() {
            if let Some(def) = find_symbol_in_rust_file(&main_rs, struct_name, symbol) {
                return Some(def);
            }
        }
        if let Ok(entries) = std::fs::read_dir(&src_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() && p.extension().and_then(|e| e.to_str()) == Some("rs") {
                    if p != lib_rs && p != main_rs {
                        if let Some(def) = find_symbol_in_rust_file(&p, struct_name, symbol) {
                            return Some(def);
                        }
                    }
                }
            }
        }

        // If symbol wasn't found in Rust source, check .fmi file if present
        let fmi_path = manifest_dir
            .join(".flame")
            .join("pkg")
            .join(plugin_name_or_alias)
            .join(format!("{}.fmi", plugin_name_or_alias));
        if fmi_path.is_file() {
            if let Some(def) = find_symbol_in_file(&fmi_path, symbol) {
                return Some(def);
            }
        }

        // If still looking for the plugin itself (e.g. symbol == plugin_name)
        if symbol == plugin_name_or_alias {
            if lib_rs.is_file() {
                return Some(JsonDefinition {
                    file: lib_rs.to_string_lossy().to_string(),
                    line: 1,
                    column: 1,
                    end_line: Some(1),
                    end_column: Some(1),
                });
            }
        }
    }

    None
}

pub fn infer_receiver_type(
    stmts: &[crate::parser::Stmt],
    content: &str,
    var_name: &str,
    manifest_dir: &std::path::Path,
) -> (Option<String>, Option<String>) {
    // 1. First check AST for LetDecl or ConstDecl matching var_name
    let mut ast_found = None;
    for stmt in stmts {
        match stmt {
            crate::parser::Stmt::LetDecl {
                name,
                type_ann,
                value,
                ..
            } if name == var_name => {
                ast_found = Some((type_ann.clone(), value.clone()));
                break;
            }
            crate::parser::Stmt::FuncDecl {
                body: Some(body), ..
            } => {
                for b in body {
                    if let crate::parser::Stmt::LetDecl {
                        name,
                        type_ann,
                        value,
                        ..
                    } = b
                    {
                        if name == var_name {
                            ast_found = Some((type_ann.clone(), value.clone()));
                            break;
                        }
                    }
                }
                if ast_found.is_some() {
                    break;
                }
            }
            _ => {}
        }
    }

    if let Some((type_ann, value)) = ast_found {
        if let Some(ann) = type_ann {
            let clean_ann = ann.split('.').last().unwrap_or(&ann).to_string();
            return (Some(clean_ann), None);
        }

        let mut curr_expr = &value;
        while let crate::parser::Expr::Await(inner, _)
        | crate::parser::Expr::Borrow(inner, _, _) = curr_expr
        {
            curr_expr = inner;
        }

        if let crate::parser::Expr::Call(callee, _, _) = curr_expr {
            if let crate::parser::Expr::Dot(receiver, method_name, _) = callee.as_ref() {
                if let crate::parser::Expr::Identifier(plugin_candidate, _) = receiver.as_ref() {
                    if let Some(ret_type) =
                        get_plugin_func_return_type(manifest_dir, plugin_candidate, method_name)
                    {
                        return (Some(ret_type), Some(plugin_candidate.clone()));
                    }
                }
            }
        }
    }

    // 2. Fallback regex on content
    let re = regex::Regex::new(&format!(
        r"(?:let|const)(?:\s+mut)?\s+{}\s*(?::\s*([a-zA-Z_]\w*))?\s*=\s*(?:await\s+)?([a-zA-Z_]\w*)\.([a-zA-Z_]\w*)",
        regex::escape(var_name)
    ));
    if let Ok(re) = re {
        if let Some(cap) = re.captures(content) {
            if let Some(ann) = cap.get(1) {
                return (Some(ann.as_str().to_string()), None);
            }
            let receiver = cap[2].to_string();
            let method = cap[3].to_string();
            if let Some(ret_type) =
                get_plugin_func_return_type(manifest_dir, &receiver, &method)
            {
                return (Some(ret_type), Some(receiver));
            }
        }
    }

    (None, None)
}

pub fn find_definition(
    file: &str,
    content: &str,
    line: usize,
    col: usize,
    stmts: &[crate::parser::Stmt],
    manifest_dir: &std::path::Path,
) -> Option<JsonDefinition> {
    let line_str = content.lines().nth(line.saturating_sub(1)).unwrap_or("");
    let col = col.min(line_str.len() + 1);

    // Check if cursor is on an import line
    let trimmed_line = line_str.trim();
    if trimmed_line.starts_with("import ") {
        let import_target = trimmed_line.trim_start_matches("import ").trim();
        let clean_import = import_target
            .split_whitespace()
            .next()
            .unwrap_or("")
            .trim_end_matches(';');

        if clean_import.starts_with("std.") || clean_import == "std" {
            let mod_name = clean_import.trim_start_matches("std.");
            if let Some(blaze) = locate_blaze_dir() {
                let std_file = blaze.join("std").join(format!("{}.fm", mod_name));
                if std_file.is_file() {
                    return Some(JsonDefinition {
                        file: std_file.to_string_lossy().to_string(),
                        line: 1,
                        column: 1,
                        end_line: Some(1),
                        end_column: Some(1),
                    });
                }
            }
        } else if clean_import.starts_with("native.") {
            let plugin_name = clean_import.trim_start_matches("native.");
            if let Some(def) =
                find_native_plugin_symbol(manifest_dir, plugin_name, None, plugin_name)
            {
                return Some(def);
            }
        } else {
            // Workspace or package import
            let path_parts: Vec<String> = clean_import.split('.').map(String::from).collect();
            if let Some(found_file) =
                crate::stdlib::locate_import_file(std::path::Path::new(file), &path_parts)
            {
                let target_path = if found_file.is_dir() {
                    if found_file.join("src").join("main.fm").is_file() {
                        found_file.join("src").join("main.fm")
                    } else if found_file.join("main.fm").is_file() {
                        found_file.join("main.fm")
                    } else if found_file.join("src").join("lib.rs").is_file() {
                        found_file.join("src").join("lib.rs")
                    } else if found_file.join("flame.toml").is_file() {
                        found_file.join("flame.toml")
                    } else {
                        found_file
                    }
                } else {
                    found_file
                };
                if target_path.is_file() {
                    return Some(JsonDefinition {
                        file: target_path.to_string_lossy().to_string(),
                        line: 1,
                        column: 1,
                        end_line: Some(1),
                        end_column: Some(1),
                    });
                }
            }
        }
    }

    // Extract word under cursor and dot context
    let mut start = col.saturating_sub(1);
    let chars: Vec<char> = line_str.chars().collect();

    while start > 0 {
        if let Some(&ch) = chars.get(start - 1) {
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '@' {
                start -= 1;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    let mut end = col.saturating_sub(1);
    while end < chars.len() {
        if let Some(&ch) = chars.get(end) {
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '@' {
                end += 1;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    let word = if start < end {
        chars[start..end].iter().collect::<String>()
    } else {
        String::new()
    };

    if word.is_empty() {
        return None;
    }

    // Check for member access prefix (e.g. math.sqrt or helper.do_work)
    let mut namespace = None;
    if start > 0 && chars.get(start - 1) == Some(&'.') {
        let mut ns_start = start - 1;
        while ns_start > 0 {
            if let Some(&ch) = chars.get(ns_start - 1) {
                if ch.is_ascii_alphanumeric() || ch == '_' {
                    ns_start -= 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        if ns_start < start - 1 {
            namespace = Some(chars[ns_start..start - 1].iter().collect::<String>());
        }
    }

    let clean_word = word.trim_start_matches('@');

    // 1. If no namespace prefix, check local symbols in the same file first
    if namespace.is_none() {
        if let Some(local_def) = find_local_symbol(stmts, &word, file, line) {
            return Some(local_def);
        }
        // Check if word matches a native plugin directly (e.g. server.init)
        if let Some(def) =
            find_native_plugin_symbol(manifest_dir, clean_word, None, clean_word)
        {
            return Some(def);
        }
    }

    // 2. If namespace prefix is present (e.g. math.sin or server.init or app.get)
    if let Some(ref ns) = namespace {
        // 2a. Standard library module
        let std_names = [
            "math", "fs", "json", "time", "byte", "net", "unit", "os", "hardware", "desktop",
            "env", "camera", "bluetooth", "serial", "embedded", "thread", "process", "hid",
        ];
        if std_names.contains(&ns.as_str()) {
            if let Some(blaze) = locate_blaze_dir() {
                let std_file = blaze.join("std").join(format!("{}.fm", ns));
                if std_file.is_file() {
                    if let Some(def) = find_symbol_in_file(&std_file, clean_word) {
                        return Some(def);
                    }
                    return Some(JsonDefinition {
                        file: std_file.to_string_lossy().to_string(),
                        line: 1,
                        column: 1,
                        end_line: Some(1),
                        end_column: Some(1),
                    });
                }
            }
        }

        // 2b. Direct native plugin member (e.g. server.init)
        if let Some(def) = find_native_plugin_symbol(manifest_dir, ns, None, clean_word) {
            return Some(def);
        }

        // 2c. Receiver method call on a variable (e.g. app.get or app.post)
        let (inferred_struct, inferred_plugin) =
            infer_receiver_type(stmts, content, ns, manifest_dir);
        if let Some(ref st) = inferred_struct {
            if let Some(ref pl) = inferred_plugin {
                if let Some(def) =
                    find_native_plugin_symbol(manifest_dir, pl, Some(st), clean_word)
                {
                    return Some(def);
                }
            } else {
                let toml_path = manifest_dir.join("flame.toml");
                if let Ok(m_content) = std::fs::read_to_string(&toml_path) {
                    for (pl_name, _) in
                        crate::package_manager::parse_section_entries(&m_content, "[plugins]")
                    {
                        if let Some(def) =
                            find_native_plugin_symbol(manifest_dir, &pl_name, Some(st), clean_word)
                        {
                            return Some(def);
                        }
                    }
                }
            }
        }

        // 2d. Workspace module import
        let path_parts = vec![ns.clone()];
        if let Some(found_file) =
            crate::stdlib::locate_import_file(std::path::Path::new(file), &path_parts)
        {
            let target_path = if found_file.is_dir() {
                if found_file.join("src").join("main.fm").is_file() {
                    found_file.join("src").join("main.fm")
                } else if found_file.join("main.fm").is_file() {
                    found_file.join("main.fm")
                } else if found_file.join("src").join("lib.rs").is_file() {
                    found_file.join("src").join("lib.rs")
                } else if found_file.join("flame.toml").is_file() {
                    found_file.join("flame.toml")
                } else {
                    found_file
                }
            } else {
                found_file
            };
            if target_path.is_file() {
                if let Some(def) = find_symbol_in_file(&target_path, clean_word) {
                    return Some(def);
                }
                return Some(JsonDefinition {
                    file: target_path.to_string_lossy().to_string(),
                    line: 1,
                    column: 1,
                    end_line: Some(1),
                    end_column: Some(1),
                });
            }
        }

        // 2e. Check local struct methods matching namespace as variable
        let (scanned_vars, scanned_structs) = scan_document(content);
        if let Some(var) = scanned_vars.iter().find(|v| v.name == *ns) {
            if let Some(ref typ_name) = var.typ {
                if let Some(st) = scanned_structs.iter().find(|s| s.name == *typ_name) {
                    if st.methods.contains(&clean_word.to_string()) {
                        if let Some(def) = find_local_symbol(stmts, clean_word, file, line) {
                            return Some(def);
                        }
                    }
                }
            }
        }
    }

    // 3. Check imported modules (direct function / struct call from imported file)
    let import_re =
        regex::Regex::new(r"(?m)^import\s+([a-zA-Z_][\w]*(?:\.[a-zA-Z_][\w]*)*)").unwrap();
    for cap in import_re.captures_iter(content) {
        let mod_path = &cap[1];
        if mod_path.starts_with("std.") {
            let std_mod = mod_path.trim_start_matches("std.");
            if let Some(blaze) = locate_blaze_dir() {
                let std_file = blaze.join("std").join(format!("{}.fm", std_mod));
                if std_file.is_file() {
                    if let Some(def) = find_symbol_in_file(&std_file, clean_word) {
                        return Some(def);
                    }
                }
            }
        } else {
            let parts: Vec<String> = mod_path.split('.').map(String::from).collect();
            if let Some(found_file) =
                crate::stdlib::locate_import_file(std::path::Path::new(file), &parts)
            {
                let target_path = if found_file.is_dir() {
                    if found_file.join("src").join("main.fm").is_file() {
                        found_file.join("src").join("main.fm")
                    } else if found_file.join("main.fm").is_file() {
                        found_file.join("main.fm")
                    } else if found_file.join("src").join("lib.rs").is_file() {
                        found_file.join("src").join("lib.rs")
                    } else if found_file.join("flame.toml").is_file() {
                        found_file.join("flame.toml")
                    } else {
                        found_file
                    }
                } else {
                    found_file
                };
                if target_path.is_file() {
                    if let Some(def) = find_symbol_in_file(&target_path, clean_word) {
                        return Some(def);
                    }
                }
            }
        }
    }

    // 4. Workspace modules matching clean_word
    let candidate_sibling = std::path::Path::new(file)
        .parent()
        .map(|p| p.join(format!("{}.fm", clean_word)));
    if let Some(ref cand) = candidate_sibling {
        if cand.is_file() {
            return Some(JsonDefinition {
                file: cand.to_string_lossy().to_string(),
                line: 1,
                column: 1,
                end_line: Some(1),
                end_column: Some(1),
            });
        }
    }

    let candidate_src = manifest_dir.join("src").join(format!("{}.fm", clean_word));
    if candidate_src.is_file() {
        return Some(JsonDefinition {
            file: candidate_src.to_string_lossy().to_string(),
            line: 1,
            column: 1,
            end_line: Some(1),
            end_column: Some(1),
        });
    }

    // 5. Flame packages (.flame/pkg/<clean_word>)
    let pkg_root = manifest_dir.join(".flame").join("pkg").join(clean_word);
    let pkg_candidates = [
        pkg_root.join("src").join("main.fm"),
        pkg_root.join("main.fm"),
        pkg_root.join("src").join("lib.rs"),
        pkg_root.join(format!("{}.fmi", clean_word)),
        pkg_root.join("flame.toml"),
    ];
    for cand in &pkg_candidates {
        if cand.is_file() {
            return Some(JsonDefinition {
                file: cand.to_string_lossy().to_string(),
                line: 1,
                column: 1,
                end_line: Some(1),
                end_column: Some(1),
            });
        }
    }

    // 6. Standard library modules & symbols
    if let Some(blaze) = locate_blaze_dir() {
        let std_file = blaze.join("std").join(format!("{}.fm", clean_word));
        if std_file.is_file() {
            return Some(JsonDefinition {
                file: std_file.to_string_lossy().to_string(),
                line: 1,
                column: 1,
                end_line: Some(1),
                end_column: Some(1),
            });
        }

        // Search in Blaze/std/*.fm
        let std_dir = blaze.join("std");
        if let Ok(entries) = std::fs::read_dir(&std_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() && p.extension().and_then(|s| s.to_str()) == Some("fm") {
                    if let Some(def) = find_symbol_in_file(&p, clean_word) {
                        return Some(def);
                    }
                }
            }
        }
    }

    // 7. Compiler Built-ins and Annotations
    let builtins = [
        "println", "print", "eprint", "assert", "assertEq", "assertNe", "assertTrue",
        "assertFalse", "panic", "typeof", "range", "sleep", "mockData", "mockApi",
        "mockFunction", "input", "Result", "Option", "Error",
    ];
    if builtins.contains(&clean_word) {
        if let Some(blaze) = locate_blaze_dir() {
            let builtins_file = blaze.join("std").join("builtins.fm");
            if builtins_file.is_file() {
                if let Some(def) = find_symbol_in_file(&builtins_file, clean_word) {
                    return Some(def);
                }
                return Some(JsonDefinition {
                    file: builtins_file.to_string_lossy().to_string(),
                    line: 1,
                    column: 1,
                    end_line: Some(1),
                    end_column: Some(1),
                });
            }
        }
    }

    let annotations = [
        "Application", "Test", "Embedded", "Cli", "Command", "Platform", "Docs", "Requires",
        "Permission", "Suggestions", "Setup", "Cleanup", "BeforeAll", "AfterAll", "Ignore",
        "Only", "Parameterized", "Benchmark", "ExpectPanic",
    ];
    if annotations.contains(&clean_word) || word.starts_with('@') {
        if let Some(blaze) = locate_blaze_dir() {
            let ann_file = blaze.join("std").join("annotations.fm");
            if ann_file.is_file() {
                if let Some(def) = find_symbol_in_file(&ann_file, clean_word) {
                    return Some(def);
                }
                return Some(JsonDefinition {
                    file: ann_file.to_string_lossy().to_string(),
                    line: 1,
                    column: 1,
                    end_line: Some(1),
                    end_column: Some(1),
                });
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_locate_blaze_dir() {
        let blaze_dir = locate_blaze_dir();
        assert!(blaze_dir.is_some(), "Blaze directory must be located");
        let std_dir = blaze_dir.unwrap().join("std");
        assert!(std_dir.exists(), "Blaze/std directory must exist");
        assert!(std_dir.join("builtins.fm").exists(), "builtins.fm must exist");
        assert!(std_dir.join("annotations.fm").exists(), "annotations.fm must exist");
        assert!(std_dir.join("math.fm").exists(), "math.fm must exist");
    }

    #[test]
    fn test_find_local_definition() {
        let source = "fn calculate() -> Int {\n    return 42\n}\n\nfn main() {\n    let res = calculate()\n}";
        let mut lexer = crate::lexer::Lexer::new(source);
        let mut tokens = Vec::new();
        loop {
            let t = lexer.next_token();
            let is_eof = t.kind == crate::lexer::TokenKind::EOF;
            tokens.push(t);
            if is_eof {
                break;
            }
        }
        let mut parser = crate::parser::Parser::new(tokens, "test.fm".to_string());
        let stmts = parser.parse().unwrap();
        let manifest_dir = std::path::Path::new(".");

        // Hover over calculate() on line 6, col 18
        let def = find_definition("test.fm", source, 6, 18, &stmts, manifest_dir);
        assert!(def.is_some(), "Definition must be found for local function");
        let d = def.unwrap();
        assert_eq!(d.file, "test.fm");
        assert_eq!(d.line, 1);
        assert_eq!(d.column, 4);
    }

    #[test]
    fn test_find_builtin_definition() {
        let source = "fn main() {\n    println(\"hello\")\n}";
        let mut lexer = crate::lexer::Lexer::new(source);
        let mut tokens = Vec::new();
        loop {
            let t = lexer.next_token();
            let is_eof = t.kind == crate::lexer::TokenKind::EOF;
            tokens.push(t);
            if is_eof {
                break;
            }
        }
        let mut parser = crate::parser::Parser::new(tokens, "test.fm".to_string());
        let stmts = parser.parse().unwrap();
        let manifest_dir = std::path::Path::new(".");

        // Hover over println on line 2, col 6
        let def = find_definition("test.fm", source, 2, 6, &stmts, manifest_dir);
        assert!(def.is_some(), "Definition must be found for println");
        let d = def.unwrap();
        assert!(d.file.ends_with("builtins.fm"));
    }

    #[test]
    fn test_find_annotation_definition() {
        let source = "@Application\nfn main() {}";
        let mut lexer = crate::lexer::Lexer::new(source);
        let mut tokens = Vec::new();
        loop {
            let t = lexer.next_token();
            let is_eof = t.kind == crate::lexer::TokenKind::EOF;
            tokens.push(t);
            if is_eof {
                break;
            }
        }
        let mut parser = crate::parser::Parser::new(tokens, "test.fm".to_string());
        let stmts = parser.parse().unwrap();
        let manifest_dir = std::path::Path::new(".");

        // Hover over @Application on line 1, col 5
        let def = find_definition("test.fm", source, 1, 5, &stmts, manifest_dir);
        assert!(def.is_some(), "Definition must be found for @Application");
        let d = def.unwrap();
        assert!(d.file.ends_with("annotations.fm"));
    }

    #[test]
    fn test_find_stdlib_member_definition() {
        let source = "import std.math\nfn main() {\n    let s = math.sqrt(4.0)\n}";
        let mut lexer = crate::lexer::Lexer::new(source);
        let mut tokens = Vec::new();
        loop {
            let t = lexer.next_token();
            let is_eof = t.kind == crate::lexer::TokenKind::EOF;
            tokens.push(t);
            if is_eof {
                break;
            }
        }
        let mut parser = crate::parser::Parser::new(tokens, "test.fm".to_string());
        let stmts = parser.parse().unwrap();
        let manifest_dir = std::path::Path::new(".");

        // Hover over sqrt on line 3, col 20
        let def = find_definition("test.fm", source, 3, 20, &stmts, manifest_dir);
        assert!(def.is_some(), "Definition must be found for math.sqrt");
        let d = def.unwrap();
        assert!(d.file.ends_with("math.fm"));
    }
}

