// This file is auto-generated to provide standard library documentation.

pub fn get_std_module_doc(module: &str) -> Option<&'static str> {
    match module {
        "unit" | "std.unit" => {
            let var_name = "# Module `unit`
        
        The `unit` module provides built-in support for dimensional analysis and physical units.
        
        ## Types
        - **`Unit`**: Represents a physical unit formula (e.g. `m/s`).
        - **`Quantity`**: Represents a value with a specific unit (e.g. `10 m/s`).
        
        ## Constants
        - **`meter`**: The SI base unit for length.
        - **`second`**: The SI base unit for time.
        - **`kilogram`**: The SI base unit for mass.
        
        ## Functions
        - **`Equation(kg, m, s)`**: Creates a new custom unit by specifying the exponents for kilograms (`kg`), meters (`m`), and seconds (`s`).
        
        **Example**:
        ```flame
        import std.unit
        
        let speed = unit.Equation(0, 1, -1)
        let a = 10 * unit.meter
        let b = 2 * unit.second
        let c = a / b
        assertEq(c, 5 * speed)
        ```
        ";
            Some(var_name)
        },
        "net" | "std.net" => Some("# Module `net`

The `net` module is a complete toolkit for networking, providing interfaces for TCP, UDP, HTTP, WebSockets, MQTT, DNS, URLs, and Network Interfaces.

## Sub-Modules
- **`std.net.tcp`**: TCP Sockets and Listeners.
- **`std.net.udp`**: UDP Sockets.
- **`std.net.http`**: HTTP Client for making requests.
- **`std.net.ws`**: WebSocket Client.
- **`std.net.mqtt`**: MQTT Client for publish/subscribe.
- **`std.net.dns`**: DNS resolution and reverse lookups.
- **`std.net.url`**: URL parsing and manipulation.
- **`std.net.interface`**: Querying host network interfaces.

**Example**:
```flame
import std.net.http

let response = await http.get(\"https://api.github.com/users/shoya-129\")
let user = await response.json()
```
"),
        "json" | "std.json" => Some("# Module `json`

The `json` module provides utilities for parsing and stringifying JSON data.

## Functions
- **`parse(string)`**: Parses a JSON string into a Flame object, array, string, number, boolean, or nil.
- **`stringify(value)`**: Serializes a Flame value into a JSON string.

**Example**:
```flame
import std.json

let data = json.parse(\"{\\\"key\\\": \\\"value\\\"}\")
println(data.key)

let str = json.stringify(data)
```
"),
        "net.tcp" | "std.net.tcp" => Some("# Module `net.tcp`

The `tcp` module provides `TcpListener` and `TcpSocket` for raw socket communication.

## Types
- **`TcpListener`**: A TCP socket server, listening for connections.
- **`TcpSocket`**: A TCP stream between a local and a remote socket.
- **`IpAddr`**: Parses an IP address.
- **`SocketAddr`**: Represents a socket address (IP + Port).

**Example**:
```flame
import std.net.tcp

let listener = await tcp.TcpListener.bind(\"0.0.0.0:3000\")
for client in listener {
    await client.write(\"Hello\")
}
```
"),
        "net.udp" | "std.net.udp" => Some("# Module `net.udp`

The `udp` module provides `UdpSocket` for connectionless UDP communication.

## Types
- **`UdpSocket`**: A UDP socket.

**Example**:
```flame
import std.net.udp

let udp = await udp.UdpSocket.bind(\":9000\")
await udp.send(\"Hello\", \"192.168.0.10:9000\")
let (msg, addr) = await udp.recv()
```
"),
        "net.http" | "std.net.http" => Some("# Module `net.http`

The `http` module provides a powerful HTTP client.

## Functions
- **`get()`**: Performs an HTTP GET request.
- **`post()`**: Performs an HTTP POST request.
- **`put()`**: Performs an HTTP PUT request.
- **`delete()`**: Performs an HTTP DELETE request.
- **`patch()`**: Performs an HTTP PATCH request.
- **`download()`**: Downloads a file to the local system.
- **`upload()`**: Uploads a file.

**Example**:
```flame
import std.net.http

let response = await http.get(\"https://api.github.com\")
println(response.status)
println(await response.text())
```
"),
        "net.ws" | "std.net.ws" => Some("# Module `net.ws`

The `ws` module provides a WebSocket client.

## Types
- **`WebSocket`**: A WebSocket connection to a server.

**Example**:
```flame
import std.net.ws

let ws = await ws.WebSocket.connect(\"ws://localhost:8080/ws\")
await ws.send(\"Move Forward\")
let msg = await ws.recv()
```
"),
        "net.mqtt" | "std.net.mqtt" => Some("# Module `net.mqtt`

The `mqtt` module provides an MQTT client.

## Types
- **`Mqtt`**: An MQTT client connection.

**Example**:
```flame
import std.net.mqtt

let client = await mqtt.Mqtt.connect(\"mqtt://broker.local\")
await client.publish(\"robot/move\", \"forward\")
await client.subscribe(\"sensor/temp\") |msg| {
    println(msg)
}
```
"),
        "net.dns" | "std.net.dns" => Some("# Module `net.dns`

The `dns` module provides DNS resolution.

## Functions
- **`lookup()`**: Looks up an IP address for a hostname.
- **`reverse()`**: Looks up a hostname for an IP address.
"),
        "net.url" | "std.net.url" => Some("# Module `net.url`

The `url` module provides URL parsing.

## Types
- **`Url`**: Represents a parsed URL.
- **`Url`**: Represents a parsed URL.
"),
        "net.interface" | "std.net.interface" => Some("# Module `net.interface`

The `interface` module provides access to host network interfaces.

## Functions
- **`interfaces()`**: Returns a list of active network interfaces.
"),
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
- **`exec()`**: Executes a system command and waits for it to finish.

**Example**:
```flame
import std.process
let out = process.exec(\"echo\", [\"Hello\"])
out.status.code.assertEq(0)
```
- **`spawn()`**: Spawns a background process asynchronously.

**Example**:
```flame
let p = process.spawn(\"git\", [\"--version\"])
let result = p.wait_with_output()
```
- **`cmd()`**: Starts a command-builder chain for richer process launching.

**Example**:
```flame
let child = process.cmd(\"git\")
    .args([\"--version\"])
    .spawn()
let result = child.wait_with_output()
```
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
- **`readDir()`**: Returns a list of files in a directory.
"),
        "math" | "std.math" => Some("# Module `math`

The `math` module provides standard mathematical functions, dimensional analysis utilities, and physical constants.

## Constants
- **`pi`**: Archimedes' constant $\\pi \\approx 3.141592653589793$.
- **`e`**: Euler's number $e \\approx 2.718281828459045$.
- **`inf`**: Positive floating-point infinity.

## Functions
- **`abs(x)`**: Returns the absolute value of a number or physical quantity.
- **`sqrt(x)`**: Computes square root, propagating physical dimensions (e.g. `math.sqrt(s^2)` -> `s`).
- **`pow(base, exp)`**: Raises a number, quantity, or unit to an exponent (e.g. `math.pow(3m, 3)` -> `27 m^3`).
- **`min(a, b)`**: Returns the minimum of two values with matching dimensions.
- **`max(a, b)`**: Returns the maximum of two values with matching dimensions.
- **`round(x)`**: Rounds to nearest integer, preserving units.
- **`floor(x)`**: Rounds down to nearest integer.
- **`ceil(x)`**: Rounds up to nearest integer.
- **`sin(rad)`**: Sine of a dimensionless angle in radians.
- **`cos(rad)`**: Cosine of a dimensionless angle in radians.

**Example**:
```flame
import std.unit
import std.math

let L = 2 * unit.meter
let g = 9.81 * unit.meter / unit.second^2
let T = 2 * math.pi * math.sqrt(L / g) // ~ 2.837 s
```
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
        "embedded" | "std.embedded" => Some("# Module `embedded`

The `std.embedded` ecosystem provides modern capability-based objects for GPIO hardware, buses, actuators, sensors, and robotics.

## Submodules
- **`io`**: `pin()`, `analog()`, `pwm()`, `dac()`
- **`comm`**: `uart()`, `spi()`, `i2c()`, `can()`
- **`devices`**: `servo()`, `motor()`, `stepper()`, `encoder()`, `sensor()`, `display()`
- **`robotics`**: `diffDrive()`, `pid()`, `imu()`
- **`system`**: `board`, `watchdog()`, `flash()`, `eeprom()`

**Example**:
```flame
import std.embedded
let led = embedded.pin(13)
led.high()
```
"),
        "byte" | "std.byte" => Some("# Module `byte`\n\nThe `byte` module provides tools to manipulate files at the byte level.\n\n## Functions\n- **`readBytes()`**: Reads all bytes from a file.\n- **`writeBytes()`**: Writes a byte array to a file.\n- **`appendBytes()`**: Appends bytes to a file.\n- **`readByte()`**: Reads a single byte.\n- **`writeByte()`**: Writes a single byte.\n- **`readByteAt()`**: Reads a byte at an offset.\n- **`writeByteAt()`**: Writes a byte at an offset."),
        _ => None,
    }
}

pub fn get_std_function_doc(module: &str, function: &str) -> Option<&'static str> {
    match (module, function) {
        ("thread" | "std.thread", "sleep") => Some(
            "Suspends the current thread for the specified number of milliseconds.

**Example**:
```flame
import std.thread

thread.sleep(1000) // Sleep for 1 second
```",
        ),
        ("thread" | "std.thread", "channel") => Some(
            "Creates a message channel and returns `(Sender, Receiver)`.

**Example**:
```flame
let (tx, rx) = thread.channel()
tx.send(\"hello\")
rx.recv().assertEq(\"hello\")
```",
        ),
        ("process" | "std.process", "exec") => Some(
            "Executes a system command and waits for it to finish.

**Example**:
```flame
import std.process
let out = process.exec(\"echo\", [\"Hello\"])
out.status.code.assertEq(0)
```",
        ),
        ("process" | "std.process", "spawn") => Some(
            "Spawns a background process asynchronously.

**Example**:
```flame
let p = process.spawn(\"git\", [\"--version\"])
let result = p.wait_with_output()
```",
        ),
        ("process" | "std.process", "cmd") => Some(
            "Creates a `CommandBuilder` for chained process configuration.

**Example**:
```flame
let child = process.cmd(\"git\")
    .args([\"--version\"])
    .spawn()
let result = child.wait_with_output()
```",
        ),
        ("fs" | "std.fs", "read") => Some(
            "Reads the entire contents of a file as a string.

**Example**:
```flame
let content = fs.read(\"data.txt\")
```",
        ),
        ("fs" | "std.fs", "write") => Some(
            "Writes string data to a file.

**Example**:
```flame
fs.write(\"data.txt\", \"Hello World\")
```",
        ),
        ("fs" | "std.fs", "append") => Some("Appends string data to the end of a file."),
        ("fs" | "std.fs", "open") => {
            Some("Opens a file and returns a File object with read/write/delete methods.")
        }
        ("fs" | "std.fs", "mkdir") => Some("Creates a new directory."),
        ("fs" | "std.fs", "mkdir_all") => {
            Some("Creates a new directory and all its parent directories.")
        }
        ("fs" | "std.fs", "copy") => Some("Copies a file or directory from source to destination."),
        ("fs" | "std.fs", "delete") => Some("Deletes a file or directory."),
        ("fs" | "std.fs", "remove") => Some("Deletes a file or empty directory."),
        ("fs" | "std.fs", "exists") => Some("Returns true if the file or directory exists."),
        ("fs" | "std.fs", "is_file") => Some("Returns true if the path points to a regular file."),
        ("fs" | "std.fs", "is_dir") => Some("Returns true if the path points to a directory."),
        ("fs" | "std.fs", "readDir") => Some("Returns a list of files in a directory."),
        ("fs" | "std.fs", "readBytes") => Some("Reads an entire file into a binary byte array."),
        ("fs" | "std.fs", "writeBytes") => Some("Writes raw binary bytes to a file."),
        ("fs" | "std.fs", "appendBytes") => Some("Appends raw binary bytes to the end of a file."),
        ("byte" | "std.byte", "readBytes") => {
            Some("Reads the entire contents of a file as a byte array.")
        }
        ("byte" | "std.byte", "writeBytes") => {
            Some("Writes a byte array to a file, overwriting if it exists.")
        }
        ("byte" | "std.byte", "appendBytes") => Some("Appends a byte array to the end of a file."),
        ("byte" | "std.byte", "writeByte") => Some("Writes a single byte (0-255) to a file."),
        ("byte" | "std.byte", "readByte") => Some("Reads a single byte from a file."),
        ("byte" | "std.byte", "appendByte") => Some("Appends a single byte (0-255) to a file."),
        ("byte" | "std.byte", "writeByteAt") => {
            Some("Writes a single byte to a file at a specific offset.")
        }
        ("byte" | "std.byte", "readByteAt") => {
            Some("Reads a single byte from a file at a specific offset.")
        }
        ("math" | "std.math", "abs") => Some("Returns the absolute value of a number."),
        ("math" | "std.math", "pow") => Some("Returns the base raised to the power exponent."),
        ("math" | "std.math", "sqrt") => Some("Returns the square root of a number."),
        ("math" | "std.math", "sin") => Some("Returns the sine of an angle (in radians)."),
        ("math" | "std.math", "cos") => Some("Returns the cosine of an angle (in radians)."),
        ("math" | "std.math", "tan") => Some("Returns the tangent of an angle (in radians)."),
        ("math" | "std.math", "floor") => Some("Rounds a number down to the nearest integer."),
        ("math" | "std.math", "ceil") => Some("Rounds a number up to the nearest integer."),
        ("math" | "std.math", "round") => Some("Rounds a number to the nearest integer."),
        ("math" | "std.math", "random") => {
            Some("Returns a random floating point number between 0 and 1.")
        }
        ("time" | "std.time", "now") => Some(
            "Returns the current Unix timestamp in milliseconds.

**Example**:
```flame
let t = time.now()
```",
        ),
        ("time" | "std.time", "format") => {
            Some("Formats a timestamp into a human readable string.")
        }
        ("os" | "std.os", "name") => {
            Some("Returns the name of the operating system (e.g., 'windows', 'linux', 'macos').")
        }
        ("os" | "std.os", "arch") => Some("Returns the architecture of the operating system."),
        ("os" | "std.os", "hostname") => Some("Returns the computer's hostname."),
        ("hardware" | "std.hardware", "cpu_usage") => {
            Some("Returns the current CPU usage percentage.")
        }
        ("hardware" | "std.hardware", "memory_usage") => Some("Returns the current memory usage."),
        ("hardware" | "std.hardware", "disk_space") => Some("Returns free and total disk space."),
        ("desktop.mouse" | "std.desktop.mouse", "move") => Some(
            "Moves the mouse cursor to absolute screen coordinates (x, y).

**Example**:
```flame
desktop.mouse.move(500, 500)
```",
        ),
        ("desktop.mouse" | "std.desktop.mouse", "click") => Some(
            "Simulates a mouse click. Can pass 'left', 'right', or 'middle'.

**Example**:
```flame
desktop.mouse.click(\"left\")
```",
        ),
        ("desktop.keyboard" | "std.desktop.keyboard", "write") => Some(
            "Types out the specified text string.

**Example**:
```flame
desktop.keyboard.write(\"Hello World\")
```",
        ),
        ("desktop.keyboard" | "std.desktop.keyboard", "key") => {
            Some("Presses a specific key (e.g., 'enter', 'esc').")
        }
        ("desktop.keyboard" | "std.desktop.keyboard", "hotkey") => Some(
            "Presses a combination of keys simultaneously.

**Example**:
```flame
desktop.keyboard.hotkey(\"ctrl\", \"c\")
```",
        ),
        ("env" | "std.env", "get") => Some(
            "Gets the value of an environment variable.

**Example**:
```flame
let path = env.get(\"PATH\")
```",
        ),
        ("env" | "std.env", "set") => Some("Sets the value of an environment variable."),
        ("hid" | "std.hid", "devices") => Some("Lists available HID devices."),
        ("hid" | "std.hid", "open") => {
            Some("Opens a connection to a specific HID device by VID and PID.")
        }
        ("camera" | "std.camera", "capture") => {
            Some("Captures a single frame from the camera as an image.")
        }
        ("camera" | "std.camera", "list") => Some("Lists available camera devices."),
        ("bluetooth" | "std.bluetooth", "scan") => Some("Scans for nearby Bluetooth devices."),
        ("bluetooth" | "std.bluetooth", "connect") => Some("Connects to a Bluetooth device."),
        ("serial" | "std.serial", "ports") => Some("Lists available serial ports."),
        ("serial" | "std.serial", "open") => Some(
            "Opens a serial port connection at a specific baud rate.

**Example**:
```flame
let port = serial.open(\"COM3\", 9600)
```",
        ),
        ("embedded" | "std.embedded", "pin") => Some(
            "Creates an exclusive digital GPIO Pin capability object backed by native `embedded-hal` drivers.\n\n**Methods**:\n- `.mode(dir)`: Set directional mode (`\"Input\"` or `\"Output\"`).\n- `.high()`: Assert voltage HIGH (3.3V / 5V).\n- `.low()`: Clear voltage LOW (0.0V).\n- `.toggle()`: Flip the active digital logic state.\n- `.read()`: Read active logic pin level.",
        ),
        ("embedded" | "std.embedded", "analog") => Some(
            "Creates an ADC (Analog-to-Digital Converter) capability object.\n\n**Methods**:\n- `.read()`: Sample raw 12-bit binary representation.\n- `.readVoltage()`: Convert raw reading to calibrated voltage float.\n- `.readPercent()`: Sample analog input as 0.0 - 100.0% ratio.",
        ),
        ("embedded" | "std.embedded", "pwm") => Some(
            "Creates a Pulse-Width Modulation (PWM) frequency output driver.\n\n**Methods**:\n- `.write(duty)`: Set duty cycle value.\n- `.enable()` / `.disable()`: Toggle hardware timer clock signal.",
        ),
        ("embedded" | "std.embedded", "dac") => Some(
            "Creates a Digital-to-Analog Converter (DAC) object for accurate analog voltage generation.\n\n**Methods**:\n- `.write(voltage)`: Emit specified steady DC voltage onto pin.",
        ),
        ("embedded" | "std.embedded", "uart") => Some(
            "Creates an asynchronous UART / RS-232 / RS-485 serial communication bus driver.\n\n**Methods**:\n- `.println(str)`: Transmit line over TX wire.\n- `.readLine()`: Receive line from RX buffer.",
        ),
        ("embedded" | "std.embedded", "i2c") => Some(
            "Creates an I2C slave transaction driver object for two-wire synchronized bus communications.\n\n**Methods**:\n- `.write(data)`: Transact data stream to specified 7-bit / 10-bit slave address.\n- `.scan()`: Enumerate active devices on SDA/SCL lines.",
        ),
        ("embedded" | "std.embedded", "spi") => Some(
            "Creates a high-speed synchronous SPI bus transaction driver.\n\n**Methods**:\n- `.transfer(bytes)`: Exposes synchronous duplex MOSI/MISO frame byte transfer.",
        ),
        ("embedded" | "std.embedded", "can") => Some(
            "Creates a CAN Bus network driver for automotive, industrial, and drone communication networks.\n\n**Methods**:\n- `.send(frame)`: Broadcast standard arbitration frame across differential lines.",
        ),
        ("embedded" | "std.embedded", "servo") => Some(
            "Creates an angle-controlled PWM actuator object for precision hobby servos.\n\n**Methods**:\n- `.angle(deg)`: Rotate horn directly to target angle in degrees.\n- `.rotate(deg)`: Sweep angle slowly.\n- `.stop()`: De-energize signal servo motor.",
        ),
        ("embedded" | "std.embedded", "motor") => Some(
            "Creates an H-bridge DC motor actuator capability object.\n\n**Methods**:\n- `.forward()` / `.reverse()`: Set directional rotation.\n- `.speed(pct)`: Throttle motor power PWM.\n- `.stop()`: Coast or brake motor shaft.",
        ),
        ("embedded" | "std.embedded", "stepper") => Some(
            "Creates a precision stepper motor controller via Step and Direction pins.\n\n**Methods**:\n- `.step(count)`: Emit precise step pulse train.\n- `.rotate(deg)`: Calculate pulses and rotate shaft by desired angle.",
        ),
        ("embedded" | "std.embedded", "encoder") => Some(
            "Creates a hardware quadrature rotary encoder interface.\n\n**Methods**:\n- `.reset()`: Reset internal directional counter integer to zero.",
        ),
        ("embedded" | "std.embedded", "sensor") => Some(
            "Creates an abstract IoT sensor interface (e.g. BME280, MPU6050, DHT22) returning live environmental readings.\n\n**Methods**:\n- `.read()`: Poll sensor registers over I2C/SPI and update `.temperature`, `.humidity`, and `.pressure` properties.",
        ),
        ("embedded" | "std.embedded", "display") => Some(
            "Creates an SPI/I2C framebuffer graphic display controller for OLED and TFT screens.\n\n**Methods**:\n- `.clear()`: Wipe screen buffer to black.\n- `.text(str)`: Draw vector glyph text directly to buffer.",
        ),
        ("embedded" | "std.embedded", "diffDrive") => Some(
            "Creates a two-wheel differential drive kinematics controller for autonomous rovers and robots.\n\n**Methods**:\n- `.forward()`: Energize both wheel motors in tandem.\n- `.rotate(deg)`: Perform opposite-axle pivoting turn.\n- `.stop()`: Halt all drivetrain actuators.",
        ),
        ("embedded" | "std.embedded", "pid") => Some(
            "Creates a Proportional-Integral-Derivative (PID) closed-loop algorithmic controller for robotics.\n\n**Methods**:\n- `.update(sample)`: Compute feedback error against `.target` and return corrective actuation compensation ratio.",
        ),
        ("embedded" | "std.embedded", "imu") => Some(
            "Creates a Inertial Measurement Unit (IMU) sensor reading real-time acceleration vectors (`.acceleration.x/y/z`) and compass orientation (`.heading`).",
        ),
        ("embedded" | "std.embedded", "board") => Some(
            "Inspects host and microcontroller system specs, yielding real detected CPU architecture, processor brand, OS kernel, and memory totals.",
        ),
        ("embedded" | "std.embedded", "watchdog") => Some(
            "Hardware Watchdog Timer interface.\n\n**Methods**:\n- `.feed()`: Reset countdown timer to prevent automated system hard-reboot.",
        ),
        ("embedded" | "std.embedded", "flash") => Some(
            "Non-volatile Flash memory storage controller.\n\n**Methods**:\n- `.write(addr, val)`: Store word persistent across reboots.\n- `.read(addr)`: Fetch value at byte offset.",
        ),
        ("embedded" | "std.embedded", "eeprom") => Some(
            "EEPROM byte-level persistent storage driver.\n\n**Methods**:\n- `.write(addr, val)`: Program EEPROM cell.",
        ),
        ("embedded" | "std.embedded", "detect_ports") => Some(
            "Enumerates physical USB hardware microcontrollers and standard COM serial interfaces attached to the system for firmware upload and debugging.",
        ),
        ("json" | "std.json", "parse") => Some(
            "Parses a JSON string into a Flame object or array.\n\n**Example**:\n```flame\nlet obj = json.parse(\"{\\\"name\\\": \\\"Flame\\\"}\")\n```",
        ),
        ("json" | "std.json", "stringify") => Some(
            "Converts a Flame object or value into a JSON string.\n\n**Example**:\n```flame\nlet str = json.stringify(formula { key: \"value\" })\n```",
        ),
        ("unit" | "std.unit", "Equation") => Some(
            "```flame\nfn Equation(kg: Int, m: Int, s: Int) -> Quantity\n```\nCreates a new custom unit by specifying the exponents for kilograms (`kg`), meters (`m`), and seconds (`s`).\n\n**Example**:\n```flame\nlet speed = unit.Equation(0, 1, -1)\n```",
        ),
        ("math" | "std.math", "pi") => Some(
            "```flame\nconst pi: Float = 3.141592653589793\n```\nMathematical constant Archimedes' constant $\\pi$, representing the ratio of a circle's circumference to its diameter.\n\n**Example**:\n```flame\nlet circumference = 2 * math.pi * r\n```",
        ),
        ("math" | "std.math", "e") => Some(
            "```flame\nconst e: Float = 2.718281828459045\n```\nMathematical constant Euler's number $e$, the base of the natural logarithm.\n\n**Example**:\n```flame\nlet growth = math.e\n```",
        ),
        ("math" | "std.math", "inf") => Some(
            "```flame\nfn inf() -> Float\n```\nReturns positive floating-point infinity.",
        ),
        ("math" | "std.math", "abs") => Some(
            "```flame\nfn abs(x: Float | Int | Quantity | Unit) -> Float | Int | Quantity | Unit\n```\nReturns the absolute magnitude of a number, quantity, or unit while strictly preserving physical unit dimensions.\n\n**Example**:\n```flame\nlet d = math.abs(-10.5 * unit.meter) // 10.5 m\n```",
        ),
        ("math" | "std.math", "sqrt") => Some(
            "```flame\nfn sqrt(x: Float | Int | Quantity | Unit) -> Float | Quantity | Unit\n```\nReturns the square root of a number, quantity, or unit. For quantities and units, every dimensional exponent must be even and is safely halved.\n\n**Example**:\n```flame\nlet t = math.sqrt(4 * unit.second^2) // 2 s\n```",
        ),
        ("math" | "std.math", "pow") => Some(
            "```flame\nfn pow(base: Float | Int | Quantity | Unit, exp: Int | Float) -> Float | Quantity | Unit\n```\nRaises a base number, quantity, or unit to the specified power exponent, correctly scaling both magnitude and dimensional powers.\n\n**Example**:\n```flame\nlet volume = math.pow(3 * unit.meter, 3) // 27 m^3\n```",
        ),
        ("math" | "std.math", "sin") => Some(
            "```flame\nfn sin(rad: Float | Int) -> Float\n```\nReturns the trigonometric sine of a dimensionless angle specified in radians.\n\n**Example**:\n```flame\nlet y = math.sin(math.pi / 2)\n```",
        ),
        ("math" | "std.math", "cos") => Some(
            "```flame\nfn cos(rad: Float | Int) -> Float\n```\nReturns the trigonometric cosine of a dimensionless angle specified in radians.\n\n**Example**:\n```flame\nlet x = math.cos(0.0)\n```",
        ),
        ("math" | "std.math", "min") => Some(
            "```flame\nfn min(a: Float | Int | Quantity | Unit, b: Float | Int | Quantity | Unit) -> Float | Int | Quantity | Unit\n```\nReturns the smaller of two numbers or quantities with matching physical dimensions.\n\n**Example**:\n```flame\nlet shortest = math.min(5 * unit.meter, 12 * unit.meter) // 5 m\n```",
        ),
        ("math" | "std.math", "max") => Some(
            "```flame\nfn max(a: Float | Int | Quantity | Unit, b: Float | Int | Quantity | Unit) -> Float | Int | Quantity | Unit\n```\nReturns the larger of two numbers or quantities with matching physical dimensions.\n\n**Example**:\n```flame\nlet longest = math.max(5 * unit.meter, 12 * unit.meter) // 12 m\n```",
        ),
        ("math" | "std.math", "round") => Some(
            "```flame\nfn round(x: Float | Int | Quantity | Unit) -> Float | Int | Quantity | Unit\n```\nReturns the nearest integer to a number or quantity, preserving physical dimensions.\n\n**Example**:\n```flame\nlet r = math.round(2.837 * unit.second) // 3 s\n```",
        ),
        ("math" | "std.math", "floor") => Some(
            "```flame\nfn floor(x: Float | Int | Quantity | Unit) -> Float | Int | Quantity | Unit\n```\nReturns the largest integer less than or equal to a number or quantity.\n\n**Example**:\n```flame\nlet f = math.floor(2.837 * unit.second) // 2 s\n```",
        ),
        ("math" | "std.math", "ceil") => Some(
            "```flame\nfn ceil(x: Float | Int | Quantity | Unit) -> Float | Int | Quantity | Unit\n```\nReturns the smallest integer greater than or equal to a number or quantity.\n\n**Example**:\n```flame\nlet c = math.ceil(2.1 * unit.second) // 3 s\n```",
        ),
        _ => None,
    }
}
