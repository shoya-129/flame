
// This file is auto-generated to provide standard library documentation.

pub fn get_std_module_doc(module: &str) -> Option<&'static str> {
    match module {
        "thread" | "std.thread" => Some("# Module `thread`

The `thread` module allows you to spawn background threads and manage concurrent execution.

## Functions
- **`sleep()`**: Suspends the current thread for the specified number of milliseconds.

**Example**:
```flame
import std.thread

thread.sleep(1000) // Sleep for 1 second
```
"),
        "process" | "std.process" => Some("# Module `process`

The `process` module provides utilities to spawn new processes, execute commands, and manage system processes.

## Functions
- **`run()`**: Executes a system command and waits for it to finish.

**Example**:
```flame
import std.process
process.run(\"echo Hello\")
```
- **`spawn()`**: Spawns a background process asynchronously.

**Example**:
```flame
let p = process.spawn(\"sleep\", [\"10\"])
```
- **`kill()`**: Kills a running process by its PID.
"),
        "fs" | "std.fs" => Some("# Module `fs`

The `fs` module provides tools to interact with the file system (read, write, delete files and directories).

## Functions
- **`read()`**: Reads the entire contents of a file as a string.

**Example**:
```flame
let content = fs.read(\"data.txt\")
```
- **`write()`**: Writes string data to a file.

**Example**:
```flame
fs.write(\"data.txt\", \"Hello World\")
```
- **`append()`**: Appends string data to the end of a file.
- **`remove()`**: Deletes a file or empty directory.
- **`exists()`**: Returns true if the file or directory exists.
- **`is_file()`**: Returns true if the path points to a regular file.
- **`is_dir()`**: Returns true if the path points to a directory.
- **`read_dir()`**: Returns a list of files in a directory.
"),
        "math" | "std.math" => Some("# Module `math`

The `math` module provides common mathematical functions and constants.

## Functions
- **`abs()`**: Returns the absolute value of a number.
- **`pow()`**: Returns the base raised to the power exponent.
- **`sqrt()`**: Returns the square root of a number.
- **`sin()`**: Returns the sine of an angle (in radians).
- **`cos()`**: Returns the cosine of an angle (in radians).
- **`tan()`**: Returns the tangent of an angle (in radians).
- **`floor()`**: Rounds a number down to the nearest integer.
- **`ceil()`**: Rounds a number up to the nearest integer.
- **`round()`**: Rounds a number to the nearest integer.
- **`random()`**: Returns a random floating point number between 0 and 1.
"),
        "time" | "std.time" => Some("# Module `time`

The `time` module provides utilities for tracking and formatting time.

## Functions
- **`now()`**: Returns the current Unix timestamp in milliseconds.

**Example**:
```flame
let t = time.now()
```
- **`format()`**: Formats a timestamp into a human readable string.
"),
        "os" | "std.os" => Some("# Module `os`

The `os` module provides operating system specific information and interactions.

## Functions
- **`name()`**: Returns the name of the operating system (e.g., 'windows', 'linux', 'macos').
- **`arch()`**: Returns the architecture of the operating system.
- **`hostname()`**: Returns the computer's hostname.
"),
        "hardware" | "std.hardware" => Some("# Module `hardware`

The `hardware` module provides access to hardware sensors and diagnostic information.

## Functions
- **`cpu_usage()`**: Returns the current CPU usage percentage.
- **`memory_usage()`**: Returns the current memory usage.
- **`disk_space()`**: Returns free and total disk space.
"),
        "desktop.mouse" | "std.desktop.mouse" => Some("# Module `desktop.mouse`

The `desktop.mouse` module allows you to programmatically control the mouse cursor and clicks.

## Functions
- **`move()`**: Moves the mouse cursor to absolute screen coordinates (x, y).

**Example**:
```flame
desktop.mouse.move(500, 500)
```
- **`click()`**: Simulates a mouse click. Can pass 'left', 'right', or 'middle'.

**Example**:
```flame
desktop.mouse.click(\"left\")
```
"),
        "desktop.keyboard" | "std.desktop.keyboard" => Some("# Module `desktop.keyboard`

The `desktop.keyboard` module allows you to programmatically type text and press hotkeys.

## Functions
- **`write()`**: Types out the specified text string.

**Example**:
```flame
desktop.keyboard.write(\"Hello World\")
```
- **`key()`**: Presses a specific key (e.g., 'enter', 'esc').
- **`hotkey()`**: Presses a combination of keys simultaneously.

**Example**:
```flame
desktop.keyboard.hotkey(\"ctrl\", \"c\")
```
"),
        "desktop" | "std.desktop" => Some("# Module `desktop`

The `desktop` module provides namespaces for interacting with the UI (`desktop.mouse`, `desktop.keyboard`).
"),
        "env" | "std.env" => Some("# Module `env`

The `env` module provides access to environment variables.

## Functions
- **`get()`**: Gets the value of an environment variable.

**Example**:
```flame
let path = env.get(\"PATH\")
```
- **`set()`**: Sets the value of an environment variable.
"),
        "hid" | "std.hid" => Some("# Module `hid`

The `hid` module allows you to connect to and interact with Human Interface Devices (e.g. custom controllers, stream decks).

## Functions
- **`devices()`**: Lists available HID devices.
- **`open()`**: Opens a connection to a specific HID device by VID and PID.
"),
        "camera" | "std.camera" => Some("# Module `camera`

The `camera` module provides access to connected webcams and cameras.

## Functions
- **`capture()`**: Captures a single frame from the camera as an image.
- **`list()`**: Lists available camera devices.
"),
        "bluetooth" | "std.bluetooth" => Some("# Module `bluetooth`

The `bluetooth` module provides access to Bluetooth devices.

## Functions
- **`scan()`**: Scans for nearby Bluetooth devices.
- **`connect()`**: Connects to a Bluetooth device.
"),
        "serial" | "std.serial" => Some("# Module `serial`

The `serial` module provides RS-232 serial port communication (useful for Arduino, Raspberry Pi, etc).

## Functions
- **`ports()`**: Lists available serial ports.
- **`open()`**: Opens a serial port connection at a specific baud rate.

**Example**:
```flame
let port = serial.open(\"COM3\", 9600)
```
"),
        _ => None,
    }
}

pub fn get_std_function_doc(module: &str, function: &str) -> Option<&'static str> {
    match (module, function) {
        ("thread" | "std.thread", "sleep") => Some("Suspends the current thread for the specified number of milliseconds.

**Example**:
```flame
import std.thread

thread.sleep(1000) // Sleep for 1 second
```"),
        ("process" | "std.process", "run") => Some("Executes a system command and waits for it to finish.

**Example**:
```flame
import std.process
process.run(\"echo Hello\")
```"),
        ("process" | "std.process", "spawn") => Some("Spawns a background process asynchronously.

**Example**:
```flame
let p = process.spawn(\"sleep\", [\"10\"])
```"),
        ("process" | "std.process", "kill") => Some("Kills a running process by its PID."),
        ("fs" | "std.fs", "read") => Some("Reads the entire contents of a file as a string.

**Example**:
```flame
let content = fs.read(\"data.txt\")
```"),
        ("fs" | "std.fs", "write") => Some("Writes string data to a file.

**Example**:
```flame
fs.write(\"data.txt\", \"Hello World\")
```"),
        ("fs" | "std.fs", "append") => Some("Appends string data to the end of a file."),
        ("fs" | "std.fs", "remove") => Some("Deletes a file or empty directory."),
        ("fs" | "std.fs", "exists") => Some("Returns true if the file or directory exists."),
        ("fs" | "std.fs", "is_file") => Some("Returns true if the path points to a regular file."),
        ("fs" | "std.fs", "is_dir") => Some("Returns true if the path points to a directory."),
        ("fs" | "std.fs", "read_dir") => Some("Returns a list of files in a directory."),
        ("math" | "std.math", "abs") => Some("Returns the absolute value of a number."),
        ("math" | "std.math", "pow") => Some("Returns the base raised to the power exponent."),
        ("math" | "std.math", "sqrt") => Some("Returns the square root of a number."),
        ("math" | "std.math", "sin") => Some("Returns the sine of an angle (in radians)."),
        ("math" | "std.math", "cos") => Some("Returns the cosine of an angle (in radians)."),
        ("math" | "std.math", "tan") => Some("Returns the tangent of an angle (in radians)."),
        ("math" | "std.math", "floor") => Some("Rounds a number down to the nearest integer."),
        ("math" | "std.math", "ceil") => Some("Rounds a number up to the nearest integer."),
        ("math" | "std.math", "round") => Some("Rounds a number to the nearest integer."),
        ("math" | "std.math", "random") => Some("Returns a random floating point number between 0 and 1."),
        ("time" | "std.time", "now") => Some("Returns the current Unix timestamp in milliseconds.

**Example**:
```flame
let t = time.now()
```"),
        ("time" | "std.time", "format") => Some("Formats a timestamp into a human readable string."),
        ("os" | "std.os", "name") => Some("Returns the name of the operating system (e.g., 'windows', 'linux', 'macos')."),
        ("os" | "std.os", "arch") => Some("Returns the architecture of the operating system."),
        ("os" | "std.os", "hostname") => Some("Returns the computer's hostname."),
        ("hardware" | "std.hardware", "cpu_usage") => Some("Returns the current CPU usage percentage."),
        ("hardware" | "std.hardware", "memory_usage") => Some("Returns the current memory usage."),
        ("hardware" | "std.hardware", "disk_space") => Some("Returns free and total disk space."),
        ("desktop.mouse" | "std.desktop.mouse", "move") => Some("Moves the mouse cursor to absolute screen coordinates (x, y).

**Example**:
```flame
desktop.mouse.move(500, 500)
```"),
        ("desktop.mouse" | "std.desktop.mouse", "click") => Some("Simulates a mouse click. Can pass 'left', 'right', or 'middle'.

**Example**:
```flame
desktop.mouse.click(\"left\")
```"),
        ("desktop.keyboard" | "std.desktop.keyboard", "write") => Some("Types out the specified text string.

**Example**:
```flame
desktop.keyboard.write(\"Hello World\")
```"),
        ("desktop.keyboard" | "std.desktop.keyboard", "key") => Some("Presses a specific key (e.g., 'enter', 'esc')."),
        ("desktop.keyboard" | "std.desktop.keyboard", "hotkey") => Some("Presses a combination of keys simultaneously.

**Example**:
```flame
desktop.keyboard.hotkey(\"ctrl\", \"c\")
```"),
        ("env" | "std.env", "get") => Some("Gets the value of an environment variable.

**Example**:
```flame
let path = env.get(\"PATH\")
```"),
        ("env" | "std.env", "set") => Some("Sets the value of an environment variable."),
        ("hid" | "std.hid", "devices") => Some("Lists available HID devices."),
        ("hid" | "std.hid", "open") => Some("Opens a connection to a specific HID device by VID and PID."),
        ("camera" | "std.camera", "capture") => Some("Captures a single frame from the camera as an image."),
        ("camera" | "std.camera", "list") => Some("Lists available camera devices."),
        ("bluetooth" | "std.bluetooth", "scan") => Some("Scans for nearby Bluetooth devices."),
        ("bluetooth" | "std.bluetooth", "connect") => Some("Connects to a Bluetooth device."),
        ("serial" | "std.serial", "ports") => Some("Lists available serial ports."),
        ("serial" | "std.serial", "open") => Some("Opens a serial port connection at a specific baud rate.

**Example**:
```flame
let port = serial.open(\"COM3\", 9600)
```"),
        _ => None,
    }
}
