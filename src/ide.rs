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
    (
        "type",
        "Defines a type alias. Example: `type UserID = Int`",
    ),
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
    ("export", "Exports a function, struct, enum, or annotation for external modules. Example: `export fn process() {}`"),
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
        "assert_eq",
        "```flame\nfn assert_eq(actual: Any, expected: Any, message: String = \"\")\n```\nAsserts that `actual` equals `expected`. Terminates execution with an error if values differ.\n\n**Example:**\n```flame\nassert_eq(status_code, 200)\n```",
    ),
    (
        "assert_ne",
        "```flame\nfn assert_ne(actual: Any, unexpected: Any, message: String = \"\")\n```\nAsserts that `actual` does NOT equal `unexpected`.\n\n**Example:**\n```flame\nassert_ne(result, nil)\n```",
    ),
    (
        "assert_true",
        "```flame\nfn assert_true(condition: Bool, message: String = \"\")\n```\nAsserts that `condition` is `true`.\n\n**Example:**\n```flame\nassert_true(list.is_empty())\n```",
    ),
    (
        "assert_false",
        "```flame\nfn assert_false(condition: Bool, message: String = \"\")\n```\nAsserts that `condition` is `false`.\n\n**Example:**\n```flame\nassert_false(file.exists())\n```",
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
        "mock_data",
        "```flame\nfn mock_data(schema: String = \"default\") -> Formula\n```\nGenerates mock object data for testing. Supported schemas: `\"user\"`, `\"post\"`, `\"product\"`.\n\n**Example:**\n```flame\nlet user = mock_data(\"user\")\n```",
    ),
    (
        "mock_api",
        "```flame\nfn mock_api(url: String = \"*\", body: String = \"{}\", status: Int = 200) -> Formula\n```\nConfigures mock responses for API endpoints during tests.\n\n**Example:**\n```flame\nlet res = mock_api(\"/api/v1/users\", \"{\\\"id\\\": 1}\", 200)\n```",
    ),
    (
        "mock_function",
        "```flame\nfn mock_function(name: String, return_value: Any)\n```\nOverrides a named function in the current environment to return `return_value` during tests.\n\n**Example:**\n```flame\nmock_function(\"fetch_user\", formula { name: \"Alex\" })\n```",
    ),
    (
        "formula",
        "Declares a static map/dictionary structure. Example: `formula { key: \"value\" }`",
    ),
    (
        "@Application",
        "**Application Entry Point**\n\nMarks this function as the application's entry point. The function is invoked automatically when the program starts. Configuration options such as `features` enable optional standard library modules and control application-wide compiler/runtime behavior.",
    ),
    (
        "@Test",
        "**Unit Test**\n\nMarks this function as a test case. The compiler will aggregate all `@Test` functions and execute them in a secure test harness when you run `flame test`.\n\n**Parameters:**\n- `timeout: Int`: Timeout in milliseconds. Test fails if execution exceeds this.\n- `skip: Bool`: If true, skips executing this test.",
    ),
    (
        "@Embedded",
        "**Embedded Target Definition**\n\nDirects the compiler to emit machine code tailored for a specific microcontroller architecture, such as `arduino-uno` or `rp2040`.\n\n**Parameters:**\n- `target: String`: The hardware architecture target name.",
    ),
    (
        "@Cli",
        "**CLI Application**\n\nMarks the application as a Command Line Interface tool, enabling automatic parsing of command line arguments into structures.",
    ),
    (
        "@Command",
        "**CLI Command**\n\nRegisters a function as an executable command within a `@Cli` application. Associates the function with a specific command-line keyword.",
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
        "is_empty",
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
        "starts_with",
        "Built-in Method: Checks if a string begins with the given prefix. Example: `str.starts_with(\"prefix\")`",
    ),
    (
        "ends_with",
        "Built-in Method: Checks if a string ends with the given suffix. Example: `str.ends_with(\"suffix\")`",
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
        "to_uppercase",
        "Built-in Method: Converts a string to uppercase. Example: `str.to_uppercase()`",
    ),
    (
        "to_lowercase",
        "Built-in Method: Converts a string to lowercase. Example: `str.to_lowercase()`",
    ),
    (
        "annotation",
        "Declares a reusable custom annotation function. Example: `annotation Benchmark(name: String) -> Formula {}`",
    ),
    (
        "Cli",
        "```flame\n@Cli(name: String, version: String = \"0.1.0\", description: String = \"\")\n```\n**CLI Annotation**\nMarks an entry function or module as a CLI root. Use with `@Command`-annotated functions to describe subcommands, flags, and positional arguments for IDE/documentation tooling.",
    ),
    (
        "Command",
        "```flame\n@Command(name: String, about: String = \"\")\n```\n**CLI Annotation**\nMarks a function as a CLI subcommand handler. Parameters describe positional arguments and options; Bool parameters map naturally to flags.",
    ),
    (
        "Test",
        "```flame\n@Test(timeout: Int = 5000, skip: Bool = false, only: Bool = false, tags: Vector<String> = [])\n```\n**Annotated Function**\nMarks a function as a test.\nRuns only when `flame test` executes.\nIgnored during `flame run` and `flame build`.",
    ),
    (
        "Setup",
        "```flame\n@Setup\n```\n**Annotated Function**\nRuns before every test in the module (equivalent to `beforeEach()`).\nIgnored during `flame run` and `flame build`.",
    ),
    (
        "Cleanup",
        "```flame\n@Cleanup\n```\n**Annotated Function**\nRuns after every test in the module (equivalent to `afterEach()`).\nIgnored during `flame run` and `flame build`.",
    ),
    (
        "BeforeAll",
        "```flame\n@BeforeAll\n```\n**Annotated Function**\nRuns exactly once before any test executes (e.g., database connection setup).\nIgnored during `flame run` and `flame build`.",
    ),
    (
        "AfterAll",
        "```flame\n@AfterAll\n```\n**Annotated Function**\nRuns exactly once after all tests complete (e.g., closing servers or cleaning temp files).\nIgnored during `flame run` and `flame build`.",
    ),
    (
        "Ignore",
        "```flame\n@Ignore\n```\n**Annotated Function**\nSkips test execution when running `flame test`.",
    ),
    (
        "Only",
        "```flame\n@Only\n```\n**Annotated Function**\nRestricts test execution to ONLY functions marked with `@Only` during `flame test`.",
    ),
    (
        "Parameterized",
        "```flame\n@Parameterized(arguments: Vector<Tuple>)\n```\n**Annotated Function**\nExpands a test into multiple independent test cases, passing each tuple element as parameters to the test function.",
    ),
    (
        "Embedded",
        "```flame\n@Embedded(target: String = \"arduino-uno\", baud: Int = 115200)\n```\n**Embedded Hardware Loop Annotation**\nMarks a function as a continuous hardware execution loop (equivalent to Arduino's `void loop()`).\nDuring `flame build`, the compiler automatically extracts the target device (e.g., `\"arduino-uno\"`, `\"esp32\"`, `\"stm32\"`, `\"rp2040\"`) and transpiles the AST directly to freestanding zero-cost `#![no_std]` Rust firmware.",
    ),
    (
        "Benchmark",
        "```flame\n@Benchmark\n```\n**Annotated Function**\nExecutes the function as a high-precision performance benchmark during `flame test`, reporting average, minimum, and maximum execution times.",
    ),
    (
        "ExpectPanic",
        "```flame\n@ExpectPanic\n```\n**Annotated Function**\nAsserts that the test function MUST terminate with a panic or error; fails if the function completes successfully.",
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
    ("arduino-uno", "Hardware Target (AVR ATmega328P)", "Zero-cost `#![no_std]` compiler target for classic Arduino Uno boards."),
    ("esp32", "Hardware Target (Xtensa/RISC-V)", "Zero-cost compiler target for ESP32 Wi-Fi & Bluetooth chips."),
    ("stm32", "Hardware Target (ARM Cortex-M)", "Zero-cost compiler target for STM32 32-bit microcontrollers."),
    ("rp2040", "Hardware Target (Raspberry Pi Silicon)", "Zero-cost compiler target for dual-core ARM Cortex-M0+ RP2040 chips."),
    ("atmega328p", "Hardware Target (Bare AVR IC)", "Direct chip compiler target for standalone ATmega328P microcontrollers."),
    ("mega", "Hardware Target (Arduino Mega 2560)", "Compiler target for ATmega2560 extended GPIO boards."),
    ("avr-nano", "Hardware Target (Arduino Nano)", "Compiler target for compact ATmega328P Nano breakout boards."),
    ("OUTPUT", "Pin Mode Parameter", "Configures digital pin as push-pull output driver."),
    ("INPUT", "Pin Mode Parameter", "Configures pin as high-impedance floating input."),
    ("INPUT_PULLUP", "Pin Mode Parameter", "Configures pin input with internal 20K pull-up resistor active."),
    ("PWM", "Pin Mode Parameter", "Configures pin for Pulse-Width Modulation wave output."),
    ("HIGH", "Digital Output Level", "Drives pin voltage to VCC (5V / 3.3V)."),
    ("LOW", "Digital Output Level", "Drives pin voltage to Ground (0V)."),
];

pub fn get_literal_completions(prefix: &str) -> Vec<JsonCompletion> {
    let clean = prefix.trim_matches('"').trim_matches('\'');
    EMBEDDED_LITERALS
        .iter()
        .filter(|(val, _, _)| val.starts_with(clean) || clean.is_empty())
        .map(|(val, kind, doc)| JsonCompletion { sort_text: None,
            label: format!("\"{}\"", val),
            kind: "value".to_string(),
            detail: kind.to_string(),
            documentation: Some(format!("```flame\n\"{}\"\n```\n**{}**\n{}", val, kind, doc)),
        })
        .collect()
}

pub fn get_keyword_completions(current_line: &str, raw_word: &str, prefix: &str) -> Vec<JsonCompletion> {
    let mut comps = Vec::new();

    if raw_word.starts_with('@') {
        let annotations = ["@Application", "@Test", "@Embedded", "@Cli", "@Command"];
        for ann in annotations {
            if ann.starts_with(raw_word) {
                let label = ann.to_string();
                let clean_prefix = prefix.trim_start_matches('@');
                let clean_label = ann.trim_start_matches('@');
                if clean_label.starts_with(clean_prefix) || clean_prefix.is_empty() {
                    comps.push(JsonCompletion { sort_text: Some("1_".to_string()),
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

    if current_line.contains("features") && current_line.contains('[') {
        let features = ["\"http\"", "\"tcp\"", "\"udp\"", "\"ws\"", "\"mqtt\"", "\"url\""];
        for feat in features {
            if feat.starts_with(prefix) || prefix.is_empty() || feat.contains(prefix) {
                // If prefix already has a quote, we don't want to insert double quotes.
                let label = if prefix.starts_with('"') {
                    feat.trim_start_matches('"').to_string()
                } else {
                    feat.to_string()
                };
                comps.push(JsonCompletion { sort_text: None,
                    label,
                    kind: "value".to_string(),
                    detail: "feature module".to_string(),
                    documentation: None,
                });
            }
        }
        return comps;
    }

    comps.extend(KEYWORDS
        .iter()
        .filter(|(kw, _)| {
            let is_alphabetic = kw.chars().all(|c| c.is_alphabetic() || c == '_');
            is_alphabetic && (kw.starts_with(prefix) || prefix.is_empty())
        })
        .map(|(kw, doc)| JsonCompletion { sort_text: None,
            label: kw.to_string(),
            kind: "keyword".to_string(),
            detail: "keyword".to_string(),
            documentation: Some(doc.to_string()),
        }));

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
        .find(|(kw, _)| *kw == word || *kw == clean_word)
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
        if let Some((val, kind, doc)) = EMBEDDED_LITERALS.iter().find(|(val, _, _)| *val == clean_lit) {
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

pub fn scan_document(content: &str) -> (Vec<ScannedVar>, Vec<ScannedStruct>) {
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
        Regex::new(r"(?:let|const)(?:\s+mut)?\s+([a-zA-Z_]\w*)(?:\s*:\s*([a-zA-Z_]\w*))?(?:\s*=\s*(?:(await)\s+)?([a-zA-Z_]\w*)(?:\.new|\s*\{|\s*\()?)?").unwrap();
    for cap in var_re.captures_iter(content) {
        let name = cap[1].to_string();
        let typ = cap.get(2).map(|m| m.as_str().to_string()).or_else(|| {
            if cap.get(3).is_some() {
                Some("Promise".to_string())
            } else {
                cap.get(4).map(|m| m.as_str().to_string())
            }
        });
        vars.push(ScannedVar { name, typ });
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
                methods: vec!["toString".to_string(), "to_string".to_string()],
            });
            // Overwrite or add to vars at the beginning so it is found first
            vars.insert(
                0,
                ScannedVar {
                    name,
                    typ: Some(synthetic_type),
                },
            );
        }
    }

    // Scan for function and annotation decls: `fn name(a: Type, b: Type)` or `annotation name(...) -> Ret`
    let fn_decl_re = Regex::new(
        r"(?:export\s+)?(?:async\s+)?(fn|annotation)\s+([a-zA-Z_]\w*)\s*\(([^)]*)\)(?:\s*->\s*([a-zA-Z0-9_<>, \t]+))?",
    )
    .unwrap();
    let arg_re = Regex::new(r"([a-zA-Z_]\w*)\s*:\s*([a-zA-Z_]\w*)").unwrap();
    for cap in fn_decl_re.captures_iter(content) {
        let kind_kw = &cap[1];
        let name_str = &cap[2];
        let params_str = cap[3].trim();
        let ret_str = cap.get(4).map_or("()", |m| m.as_str().trim());

        let sig = if kind_kw == "annotation" {
            if ret_str == "()" {
                format!("annotation {}({})", name_str, params_str)
            } else {
                format!("annotation {}({}) -> {}", name_str, params_str, ret_str)
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
        });

        let args_body = &cap[3];
        for arg_cap in arg_re.captures_iter(args_body) {
            vars.push(ScannedVar {
                name: arg_cap[1].to_string(),
                typ: Some(arg_cap[2].to_string()),
            });
        }
    }

    (vars, structs)
}

pub fn get_std_module_methods(module: &str) -> Option<Vec<String>> {
    let mut parts = module.split('.');
    let base = parts.next()?;

    let base_module = if base == "std" { parts.next()? } else { base };

    let mut map = match base_module {
        "thread" => Some(crate::native_std::thread::init()),
        "process" => Some(crate::native_std::process::init()),
        "fs" => Some(crate::native_std::fs::init()),
        "byte" => Some(crate::native_std::byte::init()),
        "net" => Some(crate::native_std::net::init(&parts.next()?)),
        "json" => Some(crate::native_std::json::init()),
        "math" => Some(crate::native_std::math::init()),
        "time" => Some(crate::native_std::time::init()),
        "os" => Some(crate::native_std::os::init()),
        "hardware" => Some(crate::native_std::hardware::init()),
        "desktop" => Some(crate::native_std::desktop::init()),
        "env" => Some(crate::native_std::env::init()),
        "hid" => Some(crate::native_std::hid::init()),
        "camera" => Some(crate::native_std::camera::init()),
        "bluetooth" => Some(crate::native_std::bluetooth::init()),
        "serial" => Some(crate::native_std::serial::init()),
        "embedded" => Some(crate::native_std::embedded::init()),
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
