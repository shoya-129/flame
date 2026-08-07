// This file is auto-generated to provide standard library documentation.

pub fn get_std_module_doc(module: &str) -> Option<&'static str> {
    match module {
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

let user = http.get(\"https://api.github.com/users/shoya-129\").json()
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

let listener = tcp.TcpListener.bind(\"0.0.0.0:3000\")
for client in listener {
    client.write(\"Hello\")
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

let udp = udp.UdpSocket.bind(\":9000\")
udp.send(\"Hello\", \"192.168.0.10:9000\")
let (msg, addr) = udp.recv()
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

let response = http.get(\"https://api.github.com\")
println(response.status)
println(response.text())
```
"),
        "net.ws" | "std.net.ws" => Some("# Module `net.ws`

The `ws` module provides a WebSocket client.

## Types
- **`WebSocket`**: A WebSocket connection to a server.

**Example**:
```flame
import std.net.ws

let ws = ws.WebSocket.connect(\"ws://localhost:8080/ws\")
ws.send(\"Move Forward\")
let msg = ws.recv()
```
"),
        "net.mqtt" | "std.net.mqtt" => Some("# Module `net.mqtt`

The `mqtt` module provides an MQTT client.

## Types
- **`Mqtt`**: An MQTT client connection.

**Example**:
```flame
import std.net.mqtt

let client = mqtt.Mqtt.connect(\"mqtt://broker.local\")
client.publish(\"robot/move\", \"forward\")
client.subscribe(\"sensor/temp\") |msg| {
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
out.status.code.assert_eq(0)
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
rx.recv().assert_eq(\"hello\")
```",
        ),
        ("process" | "std.process", "exec") => Some(
            "Executes a system command and waits for it to finish.

**Example**:
```flame
import std.process
let out = process.exec(\"echo\", [\"Hello\"])
out.status.code.assert_eq(0)
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
            "Creates an exclusive digital GPIO Pin capability object backed by native `embedded-hal` drivers.\n\n**Methods**:\n- `.mode(dir)`: Set directional mode (`\"Input\"` or `\"Output\"`).\n- `.high()`: Assert voltage HIGH (3.3V / 5V).\n- `.low()`: Clear voltage LOW (0.0V).\n- `.toggle()`: Flip the active digital logic state.\n- `.read()`: Read active logic pin level."
        ),
        ("embedded" | "std.embedded", "analog") => Some(
            "Creates an ADC (Analog-to-Digital Converter) capability object.\n\n**Methods**:\n- `.read()`: Sample raw 12-bit binary representation.\n- `.readVoltage()`: Convert raw reading to calibrated voltage float.\n- `.readPercent()`: Sample analog input as 0.0 - 100.0% ratio."
        ),
        ("embedded" | "std.embedded", "pwm") => Some("Creates a Pulse-Width Modulation (PWM) frequency output driver.\n\n**Methods**:\n- `.write(duty)`: Set duty cycle value.\n- `.enable()` / `.disable()`: Toggle hardware timer clock signal."),
        ("embedded" | "std.embedded", "dac") => Some("Creates a Digital-to-Analog Converter (DAC) object for accurate analog voltage generation.\n\n**Methods**:\n- `.write(voltage)`: Emit specified steady DC voltage onto pin."),
        ("embedded" | "std.embedded", "uart") => Some("Creates an asynchronous UART / RS-232 / RS-485 serial communication bus driver.\n\n**Methods**:\n- `.println(str)`: Transmit line over TX wire.\n- `.readLine()`: Receive line from RX buffer."),
        ("embedded" | "std.embedded", "i2c") => Some("Creates an I2C slave transaction driver object for two-wire synchronized bus communications.\n\n**Methods**:\n- `.write(data)`: Transact data stream to specified 7-bit / 10-bit slave address.\n- `.scan()`: Enumerate active devices on SDA/SCL lines."),
        ("embedded" | "std.embedded", "spi") => Some("Creates a high-speed synchronous SPI bus transaction driver.\n\n**Methods**:\n- `.transfer(bytes)`: Exposes synchronous duplex MOSI/MISO frame byte transfer."),
        ("embedded" | "std.embedded", "can") => Some("Creates a CAN Bus network driver for automotive, industrial, and drone communication networks.\n\n**Methods**:\n- `.send(frame)`: Broadcast standard arbitration frame across differential lines."),
        ("embedded" | "std.embedded", "servo") => Some("Creates an angle-controlled PWM actuator object for precision hobby servos.\n\n**Methods**:\n- `.angle(deg)`: Rotate horn directly to target angle in degrees.\n- `.rotate(deg)`: Sweep angle slowly.\n- `.stop()`: De-energize signal servo motor."),
        ("embedded" | "std.embedded", "motor") => Some("Creates an H-bridge DC motor actuator capability object.\n\n**Methods**:\n- `.forward()` / `.reverse()`: Set directional rotation.\n- `.speed(pct)`: Throttle motor power PWM.\n- `.stop()`: Coast or brake motor shaft."),
        ("embedded" | "std.embedded", "stepper") => Some("Creates a precision stepper motor controller via Step and Direction pins.\n\n**Methods**:\n- `.step(count)`: Emit precise step pulse train.\n- `.rotate(deg)`: Calculate pulses and rotate shaft by desired angle."),
        ("embedded" | "std.embedded", "encoder") => Some("Creates a hardware quadrature rotary encoder interface.\n\n**Methods**:\n- `.reset()`: Reset internal directional counter integer to zero."),
        ("embedded" | "std.embedded", "sensor") => Some("Creates an abstract IoT sensor interface (e.g. BME280, MPU6050, DHT22) returning live environmental readings.\n\n**Methods**:\n- `.read()`: Poll sensor registers over I2C/SPI and update `.temperature`, `.humidity`, and `.pressure` properties."),
        ("embedded" | "std.embedded", "display") => Some("Creates an SPI/I2C framebuffer graphic display controller for OLED and TFT screens.\n\n**Methods**:\n- `.clear()`: Wipe screen buffer to black.\n- `.text(str)`: Draw vector glyph text directly to buffer."),
        ("embedded" | "std.embedded", "diffDrive") => Some("Creates a two-wheel differential drive kinematics controller for autonomous rovers and robots.\n\n**Methods**:\n- `.forward()`: Energize both wheel motors in tandem.\n- `.rotate(deg)`: Perform opposite-axle pivoting turn.\n- `.stop()`: Halt all drivetrain actuators."),
        ("embedded" | "std.embedded", "pid") => Some("Creates a Proportional-Integral-Derivative (PID) closed-loop algorithmic controller for robotics.\n\n**Methods**:\n- `.update(sample)`: Compute feedback error against `.target` and return corrective actuation compensation ratio."),
        ("embedded" | "std.embedded", "imu") => Some("Creates a Inertial Measurement Unit (IMU) sensor reading real-time acceleration vectors (`.acceleration.x/y/z`) and compass orientation (`.heading`)."),
        ("embedded" | "std.embedded", "board") => Some("Inspects host and microcontroller system specs, yielding real detected CPU architecture, processor brand, OS kernel, and memory totals."),
        ("embedded" | "std.embedded", "watchdog") => Some("Hardware Watchdog Timer interface.\n\n**Methods**:\n- `.feed()`: Reset countdown timer to prevent automated system hard-reboot."),
        ("embedded" | "std.embedded", "flash") => Some("Non-volatile Flash memory storage controller.\n\n**Methods**:\n- `.write(addr, val)`: Store word persistent across reboots.\n- `.read(addr)`: Fetch value at byte offset."),
        ("embedded" | "std.embedded", "eeprom") => Some("EEPROM byte-level persistent storage driver.\n\n**Methods**:\n- `.write(addr, val)`: Program EEPROM cell."),
        ("embedded" | "std.embedded", "detect_ports") => Some("Enumerates physical USB hardware microcontrollers and standard COM serial interfaces attached to the system for firmware upload and debugging."),
        _ => None,
    }
}
