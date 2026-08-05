# Flame Standard Library

> **Version:** Current Runtime Implementation

The Flame Standard Library provides native modules for filesystem access, process
management, automation, hardware inspection, environment variables, mathematics,
timing, threading, operating system information, and device integration.

Only APIs that currently exist in the runtime are documented below.

---

# Imports

```flame
import std.desktop
import std.fs
import std.hardware
import std.process
import std.env
import std.math
import std.time
import std.thread
import std.os

// Optional platform modules
import std.bluetooth
import std.camera
import std.hid
import std.serial

// Reserved
import std.net
```

## Module Status

Module Status

---

std.automation ✅ Implemented std.fs ✅ Implemented std.hardware ✅ Implemented
std.process ✅ Implemented std.env ✅ Implemented std.math ✅ Implemented
std.time ✅ Implemented std.thread ✅ Implemented std.os ✅ Implemented
std.bluetooth ✅ Implemented std.camera ✅ Implemented std.hid ✅ Implemented
std.serial ✅ Implemented std.net 🚧 Reserved

---

# std.desktop

Desktop keyboard and mouse automation.

## mouse.move(x, y)

Moves the cursor to an absolute screen position.

```flame
desktop.mouse.move(500, 300)
```

Parameters:

Name Type

---

x Int y Int

Returns: `Nil`

---

## mouse.click(button = "left")

Supported buttons:

- left
- right
- middle

```flame
desktop.mouse.click()
desktop.mouse.click("right")
```

Returns `Nil`.

---

## keyboard.write(text)

Types text into the focused application.

```flame
desktop.keyboard.write("Hello World")
```

Returns `Nil`.

---

## keyboard.key(keyName, action = "click") || keyboard.hotkey

Keyboard key events.

Actions:

- click
- press
- release

Examples:

```flame
desktop.keyboard.key("A")
desktop.keyboard.key("enter")
desktop.keyboard.key("f5")
desktop.keyboard.hotkey("win", "r")
desktop.keyboard.hotkey("ctrl","press")
desktop.keyboard.hotkey("ctrl","release")
```

Supports:

- Letters
- Numbers
- Enter
- Tab
- Escape
- Backspace
- Delete
- Arrow keys
- Home / End
- PageUp / PageDown
- F1-F12
- Ctrl / Shift / Alt / Command

---

# std.fs

Filesystem utilities.

Functions:

- read(path)
- write(path, content)
- exists(path)
- remove(path)
- mkdir(path)
- mkdir_all(path)
- copy(source, destination)

Examples:

```flame
fs.write("hello.txt","Hello")

print(fs.read("hello.txt"))

if (fs.exists("hello.txt")) {
    print("Exists")
}

fs.copy("hello.txt","backup.txt")

fs.mkdir("logs")
fs.mkdir_all("build/output")

fs.remove("backup.txt")
```

---

# std.hardware

## memory()

Returns

    {
        total
        used
        free
        available
    }

## cpu()

Returns a list containing

    {
        name
        brand
        usage
        frequency
    }

## discover()

Returns a human-readable operating system description.

---

# std.process

## exec(command,args)

Runs synchronously.

Returns

    {
        stdout
        stderr
        status {
            code
        }
    }

Example

```flame
let result = process.exec("git", ["--version"])

print(result.stdout)
```

## spawn(command,args)

Starts a background process.

Returns a ChildProcess handle.

## cwd()

Returns current working directory.

## set_cwd(path)

Changes working directory.

## pid()

Returns current process ID.

---

# std.env

Functions

- get(key)
- set(key,value)
- remove(key)
- vars()
- temp()

Example

```flame
env.set("MODE","debug")

print(env.get("MODE"))

print(env.temp())
```

---

# std.math

Constants

- pi()
- e()

Functions

- abs(number)
- sin(number)
- cos(number)
- sqrt(number)

Example

```flame
print(math.pi())
print(math.sqrt(81))
```

---

# std.time

## now()

Milliseconds since Unix Epoch.

## timestamp()

Seconds since Unix Epoch.

---

# std.thread

Functions

- sleep(milliseconds)
- spawn(function)
- id()
- yield()

Example

```flame
thread.sleep(1000)

print(thread.id())
```

```flame
// create another thread
let t = thread {
    print("worker start")

    thread.sleep(3000)

    print("worker end")
}
t.join()
```

---

# std.os

Functions

- name()
- arch()
- family()

Example

```flame
print(os.name())
print(os.arch())
print(os.family())
```

---

# std.bluetooth

## supported()

Returns whether Bluetooth is available.

## adapters()

Returns adapter list.

## scan()

Scans nearby Bluetooth Low Energy devices.

Each device contains

    {
        name
        address
        connected
    }

---

# std.camera

## devices()

Lists available cameras.

Returns

    {
        index
        name
        description
    }

## capture(index,path)

Captures a photo.

```flame
camera.capture(0,"photo.png")
```

Returns `true` on success.

---

# std.hid

## devices()

Lists connected HID devices.

Each device contains

    {
        vendor
        product
        manufacturer
        product_name
        path
    }

---

# std.serial

## ports()

Returns available serial ports.

    {
        name
    }

---

# std.net

Reserved for future networking APIs.

Planned:

- HTTP
- TCP
- UDP
- TLS
- DNS
- WebSocket

---

# Complete Example

```flame
import std.fs
import std.hardware
import std.process
import std.env
import std.math
import std.time
import std.thread

print(hardware.memory())

fs.write("hello.txt","Hello")

print(fs.read("hello.txt"))

let result = process.exec("git",["--version"])

print(result.stdout)

print(math.pi())
print(time.timestamp())

thread.sleep(100)

print("Done")
```
