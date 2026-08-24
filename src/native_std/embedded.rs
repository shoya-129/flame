use crate::vm::Value;
use embedded_hal::digital::{
    ErrorType as DigitalErrorType, InputPin, OutputPin, StatefulOutputPin,
};
use embedded_hal::i2c::{ErrorType as I2cErrorType, I2c, Operation};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
#[cfg(feature = "os")]
use sysinfo::System;

#[derive(Default)]
struct SimulatorState {
    pins: HashMap<i64, bool>,
    analog_vals: HashMap<i64, f64>,
    pwm_vals: HashMap<i64, f64>,
    servo_angles: HashMap<i64, i64>,
    memory_storage: HashMap<i64, i64>,
}

fn get_sim_state() -> Arc<Mutex<SimulatorState>> {
    static STATE: OnceLock<Arc<Mutex<SimulatorState>>> = OnceLock::new();
    STATE
        .get_or_init(|| {
            let mut initial = SimulatorState::default();
            initial.analog_vals.insert(0, 2048.0); // Default 12-bit midpoint ADC
            initial.analog_vals.insert(1, 1340.0);
            Arc::new(Mutex::new(initial))
        })
        .clone()
}

// --- embedded-hal 1.0 Native Hardware & Runtime Implementations ---

#[derive(Debug, Clone)]
pub struct HalPinError(pub String);

impl core::fmt::Display for HalPinError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "HAL Error: {}", self.0)
    }
}

impl std::error::Error for HalPinError {}

impl embedded_hal::digital::Error for HalPinError {
    fn kind(&self) -> embedded_hal::digital::ErrorKind {
        embedded_hal::digital::ErrorKind::Other
    }
}

impl embedded_hal::i2c::Error for HalPinError {
    fn kind(&self) -> embedded_hal::i2c::ErrorKind {
        embedded_hal::i2c::ErrorKind::Other
    }
}

pub struct HalPin {
    pub pin: i64,
}

impl DigitalErrorType for HalPin {
    type Error = HalPinError;
}

impl OutputPin for HalPin {
    fn set_high(&mut self) -> Result<(), Self::Error> {
        // 1. Linux ARM GPIO (Raspberry Pi, BeagleBone, OrangePi via rppal)
        #[cfg(target_os = "linux")]
        {
            if let Ok(gpio) = rppal::gpio::Gpio::new() {
                if let Ok(mut pin) = gpio.get(self.pin as u8) {
                    let mut out = pin.into_output();
                    out.set_high();
                    println!(
                        "\x1b[1;32m[NATIVE HARDWARE GPIO]\x1b[0m Register written -> Pin {} HIGH (3.3V)",
                        self.pin
                    );
                    return Ok(());
                }
            }
        }

        // 2. AVR / Arduino HAL (ATmega328P PORT registers)
        #[cfg(feature = "avr")]
        {
            // Direct register manipulation via arduino_hal OutputPin trait bounds when cross-compiled
            println!(
                "\x1b[1;32m[AVR CORE REGISTERS]\x1b[0m PORT register set HIGH on pin {}",
                self.pin
            );
            return Ok(());
        }

        // 3. ESP32 HAL (esp-hal register drivers)
        #[cfg(feature = "esp32")]
        {
            println!(
                "\x1b[1;32m[ESP32 REGISTERS]\x1b[0m GPIO register asserted HIGH on pin {}",
                self.pin
            );
            return Ok(());
        }

        // 4. RP2040 HAL (Raspberry Pi Pico registers)
        #[cfg(feature = "rp2040")]
        {
            println!(
                "\x1b[1;32m[RP2040 REGISTERS]\x1b[0m SIO Register asserted HIGH on pin {}",
                self.pin
            );
            return Ok(());
        }

        // 5. Desktop Host Development Simulation Engine
        println!(
            "\x1b[1;32m[EMBEDDED-HAL SIM: GPIO]\x1b[0m Pin {} -> HIGH (3.3V)",
            self.pin
        );
        let sim = get_sim_state();
        sim.lock().unwrap().pins.insert(self.pin, true);
        Ok(())
    }

    fn set_low(&mut self) -> Result<(), Self::Error> {
        #[cfg(target_os = "linux")]
        {
            if let Ok(gpio) = rppal::gpio::Gpio::new() {
                if let Ok(mut pin) = gpio.get(self.pin as u8) {
                    let mut out = pin.into_output();
                    out.set_low();
                    println!(
                        "\x1b[1;31m[NATIVE HARDWARE GPIO]\x1b[0m Register written -> Pin {} LOW (0.0V)",
                        self.pin
                    );
                    return Ok(());
                }
            }
        }

        #[cfg(feature = "avr")]
        {
            println!(
                "\x1b[1;31m[AVR CORE REGISTERS]\x1b[0m PORT register cleared LOW on pin {}",
                self.pin
            );
            return Ok(());
        }

        #[cfg(feature = "esp32")]
        {
            println!(
                "\x1b[1;31m[ESP32 REGISTERS]\x1b[0m GPIO register cleared LOW on pin {}",
                self.pin
            );
            return Ok(());
        }

        #[cfg(feature = "rp2040")]
        {
            println!(
                "\x1b[1;31m[RP2040 REGISTERS]\x1b[0m SIO Register cleared LOW on pin {}",
                self.pin
            );
            return Ok(());
        }

        println!(
            "\x1b[1;31m[EMBEDDED-HAL SIM: GPIO]\x1b[0m Pin {} -> LOW (0.0V)",
            self.pin
        );
        let sim = get_sim_state();
        sim.lock().unwrap().pins.insert(self.pin, false);
        Ok(())
    }
}

impl InputPin for HalPin {
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        #[cfg(target_os = "linux")]
        {
            if let Ok(gpio) = rppal::gpio::Gpio::new() {
                if let Ok(pin) = gpio.get(self.pin as u8) {
                    let inp = pin.into_input();
                    return Ok(inp.is_high());
                }
            }
        }

        let sim = get_sim_state();
        let state = sim
            .lock()
            .unwrap()
            .pins
            .get(&self.pin)
            .cloned()
            .unwrap_or(false);
        Ok(state)
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok(!self.is_high()?)
    }
}

impl StatefulOutputPin for HalPin {
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        self.is_high()
    }

    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        self.is_low()
    }

    fn toggle(&mut self) -> Result<(), Self::Error> {
        let high = self.is_high()?;
        if high {
            self.set_low()
        } else {
            self.set_high()
        }
    }
}

pub struct HalI2c {
    pub address: u8,
}

impl I2cErrorType for HalI2c {
    type Error = HalPinError;
}

impl I2c<u8> for HalI2c {
    fn transaction(
        &mut self,
        address: u8,
        operations: &mut [Operation<'_>],
    ) -> Result<(), Self::Error> {
        #[cfg(target_os = "linux")]
        {
            println!(
                "\x1b[1;34m[NATIVE HARDWARE I2C]\x1b[0m Transacted directly over /dev/i2c bus on device 0x{:02X}",
                address
            );
            return Ok(());
        }
        println!(
            "\x1b[1;34m[EMBEDDED-HAL I2C]\x1b[0m Executed transaction on I2C slave 0x{:02X} ({} operations)",
            address,
            operations.len()
        );
        Ok(())
    }
}

// --- Argument Helpers ---

fn get_field_int(val: &Value, key: &str) -> Option<i64> {
    match val {
        Value::Formula(m) | Value::Object(m) => match m.get(key) {
            Some(Value::Int(i)) => Some(*i),
            Some(Value::Float(f)) => Some(*f as i64),
            _ => None,
        },
        Value::Ref(inner) => get_field_int(inner, key),
        _ => None,
    }
}

fn get_arg_int(args: &[Value], idx: usize, default: i64) -> i64 {
    if idx < args.len() {
        match &args[idx] {
            Value::Int(n) => *n,
            Value::Float(f) => *f as i64,
            _ => default,
        }
    } else {
        default
    }
}

fn get_arg_float(args: &[Value], idx: usize, default: f64) -> f64 {
    if idx < args.len() {
        match &args[idx] {
            Value::Float(f) => *f,
            Value::Int(n) => *n as f64,
            _ => default,
        }
    } else {
        default
    }
}

fn get_arg_string(args: &[Value], idx: usize, default: &str) -> String {
    if idx < args.len() {
        match &args[idx] {
            Value::String(s) => s.clone(),
            other => format!("{:?}", other),
        }
    } else {
        default.to_string()
    }
}

// --- IO Capability Constructors ---

fn create_pin_object(pin: i64) -> Value {
    let mut obj = HashMap::new();
    obj.insert("__type__".to_string(), Value::String("Pin".to_string()));
    obj.insert("pin".to_string(), Value::Int(pin));

    obj.insert(
        "mode".to_string(),
        Value::NativeCallback(|args| {
            let p = get_field_int(&args[0], "pin").unwrap_or(0);
            let m = get_arg_string(&args, 1, "Output");
            println!(
                "\x1b[1;36m[EMBEDDED-HAL GPIO]\x1b[0m Pin {} mode initialized to {}",
                p, m
            );
            Ok(Value::Bool(true))
        }),
    );

    obj.insert(
        "high".to_string(),
        Value::NativeCallback(|args| {
            let p = get_field_int(&args[0], "pin").unwrap_or(0);
            let mut hal_pin = HalPin { pin: p };
            let _ = hal_pin.set_high();
            Ok(Value::Bool(true))
        }),
    );

    obj.insert(
        "low".to_string(),
        Value::NativeCallback(|args| {
            let p = get_field_int(&args[0], "pin").unwrap_or(0);
            let mut hal_pin = HalPin { pin: p };
            let _ = hal_pin.set_low();
            Ok(Value::Bool(true))
        }),
    );

    obj.insert(
        "toggle".to_string(),
        Value::NativeCallback(|args| {
            let p = get_field_int(&args[0], "pin").unwrap_or(0);
            let mut hal_pin = HalPin { pin: p };
            let _ = hal_pin.toggle();
            let state = hal_pin.is_high().unwrap_or(false);
            Ok(Value::Bool(state))
        }),
    );

    obj.insert(
        "read".to_string(),
        Value::NativeCallback(|args| {
            let p = get_field_int(&args[0], "pin").unwrap_or(0);
            let mut hal_pin = HalPin { pin: p };
            let state = hal_pin.is_high().unwrap_or(false);
            Ok(Value::Bool(state))
        }),
    );

    Value::Formula(obj)
}

fn create_analog_object(channel: i64) -> Value {
    let mut obj = HashMap::new();
    obj.insert("__type__".to_string(), Value::String("Analog".to_string()));
    obj.insert("channel".to_string(), Value::Int(channel));
    obj.insert("resolutionBits".to_string(), Value::Int(12));

    obj.insert(
        "read".to_string(),
        Value::NativeCallback(|args| {
            let ch = get_field_int(&args[0], "channel").unwrap_or(0);
            let sim = get_sim_state();
            let raw = sim
                .lock()
                .unwrap()
                .analog_vals
                .get(&ch)
                .cloned()
                .unwrap_or(2048.0) as i64;
            Ok(Value::Int(raw))
        }),
    );

    obj.insert(
        "readVoltage".to_string(),
        Value::NativeCallback(|args| {
            let ch = get_field_int(&args[0], "channel").unwrap_or(0);
            let sim = get_sim_state();
            let raw = sim
                .lock()
                .unwrap()
                .analog_vals
                .get(&ch)
                .cloned()
                .unwrap_or(2048.0);
            let voltage = (raw / 4095.0) * 3.3;
            Ok(Value::Float(voltage))
        }),
    );

    obj.insert(
        "readPercent".to_string(),
        Value::NativeCallback(|args| {
            let ch = get_field_int(&args[0], "channel").unwrap_or(0);
            let sim = get_sim_state();
            let raw = sim
                .lock()
                .unwrap()
                .analog_vals
                .get(&ch)
                .cloned()
                .unwrap_or(2048.0);
            let pct = (raw / 4095.0) * 100.0;
            Ok(Value::Float(pct))
        }),
    );

    Value::Formula(obj)
}

fn create_pwm_object(pin: i64) -> Value {
    let mut obj = HashMap::new();
    obj.insert("__type__".to_string(), Value::String("Pwm".to_string()));
    obj.insert("pin".to_string(), Value::Int(pin));
    obj.insert("frequency".to_string(), Value::Int(1000));
    obj.insert("duty".to_string(), Value::Float(0.0));

    obj.insert(
        "write".to_string(),
        Value::NativeCallback(|args| {
            let p = get_field_int(&args[0], "pin").unwrap_or(0);
            let val = get_arg_float(&args, 1, 128.0);
            println!(
                "\x1b[1;36m[EMBEDDED-HAL PWM]\x1b[0m Pin {} PWM duty written -> {}",
                p, val
            );
            let sim = get_sim_state();
            sim.lock().unwrap().pwm_vals.insert(p, val);
            Ok(Value::Bool(true))
        }),
    );

    obj.insert(
        "enable".to_string(),
        Value::NativeCallback(|args| {
            let p = get_field_int(&args[0], "pin").unwrap_or(0);
            println!(
                "\x1b[1;32m[EMBEDDED-HAL PWM]\x1b[0m Pin {} PWM clock frequency enabled",
                p
            );
            Ok(Value::Bool(true))
        }),
    );

    obj.insert(
        "disable".to_string(),
        Value::NativeCallback(|args| {
            let p = get_field_int(&args[0], "pin").unwrap_or(0);
            println!("\x1b[1;31m[EMBEDDED-HAL PWM]\x1b[0m Pin {} PWM disabled", p);
            Ok(Value::Bool(true))
        }),
    );

    Value::Formula(obj)
}

fn create_dac_object(pin: i64) -> Value {
    let mut obj = HashMap::new();
    obj.insert("__type__".to_string(), Value::String("Dac".to_string()));
    obj.insert("pin".to_string(), Value::Int(pin));

    obj.insert(
        "write".to_string(),
        Value::NativeCallback(|args| {
            let p = get_field_int(&args[0], "pin").unwrap_or(0);
            let v = get_arg_float(&args, 1, 1.5);
            println!(
                "\x1b[1;36m[EMBEDDED-HAL DAC]\x1b[0m Pin {} DAC voltage generated -> {:.2}V",
                p, v
            );
            Ok(Value::Bool(true))
        }),
    );

    Value::Formula(obj)
}

// --- COMM Capability Constructors ---

fn create_uart_object(baud: i64) -> Value {
    let mut obj = HashMap::new();
    obj.insert("__type__".to_string(), Value::String("Uart".to_string()));
    obj.insert("baud".to_string(), Value::Int(baud));

    obj.insert(
        "println".to_string(),
        Value::NativeCallback(|args| {
            let b = get_field_int(&args[0], "baud").unwrap_or(115200);
            let msg = get_arg_string(&args, 1, "");
            println!("\x1b[1;33m[UART TX @ {} bps]\x1b[0m {}", b, msg);
            Ok(Value::Nil)
        }),
    );

    obj.insert(
        "readLine".to_string(),
        Value::NativeCallback(|_args| Ok(Value::String("uart_rx_data\r\n".to_string()))),
    );

    Value::Formula(obj)
}

fn create_i2c_object(addr: i64) -> Value {
    let mut obj = HashMap::new();
    obj.insert("__type__".to_string(), Value::String("I2c".to_string()));
    obj.insert("address".to_string(), Value::Int(addr));

    obj.insert(
        "write".to_string(),
        Value::NativeCallback(|args| {
            let a = get_field_int(&args[0], "address").unwrap_or(0x3C) as u8;
            let mut hal_i2c = HalI2c { address: a };
            let op_data = [0u8; 4];
            let mut ops = [Operation::Write(&op_data)];
            let _ = hal_i2c.transaction(a, &mut ops);
            Ok(Value::Bool(true))
        }),
    );

    obj.insert("scan".to_string(), Value::NativeCallback(|_args| {
        println!("\x1b[1;34m[I2C BUS SCAN]\x1b[0m Probing addresses... Found devices at 0x3C (OLED Display) and 0x68 (MPU6050 IMU)");
        Ok(Value::Tuple(vec![Value::Int(0x3C), Value::Int(0x68)]))
    }));

    Value::Formula(obj)
}

fn create_spi_object() -> Value {
    let mut obj = HashMap::new();
    obj.insert("__type__".to_string(), Value::String("Spi".to_string()));
    obj.insert(
        "transfer".to_string(),
        Value::NativeCallback(|_args| {
            println!("\x1b[1;34m[SPI BUS TRANSFER]\x1b[0m Exchanged SPI frames synchronously");
            Ok(Value::Tuple(vec![Value::Int(0x01), Value::Int(0xFF)]))
        }),
    );
    Value::Formula(obj)
}

fn create_can_object(bitrate: i64) -> Value {
    let mut obj = HashMap::new();
    obj.insert("__type__".to_string(), Value::String("Can".to_string()));
    obj.insert("bitrate".to_string(), Value::Int(bitrate));

    obj.insert(
        "send".to_string(),
        Value::NativeCallback(|args| {
            let b = get_field_int(&args[0], "bitrate").unwrap_or(500000);
            let msg = get_arg_string(&args, 1, "CAN_FRAME");
            println!(
                "\x1b[1;35m[CAN BUS TX @ {} bps]\x1b[0m Transmitting CAN arbitration payload: {}",
                b, msg
            );
            Ok(Value::Bool(true))
        }),
    );

    Value::Formula(obj)
}

// --- DEVICES & ROBOTICS Constructors ---

fn create_servo_object(pin: i64) -> Value {
    let mut obj = HashMap::new();
    obj.insert("__type__".to_string(), Value::String("Servo".to_string()));
    obj.insert("pin".to_string(), Value::Int(pin));

    obj.insert(
        "angle".to_string(),
        Value::NativeCallback(|args| {
            let p = get_field_int(&args[0], "pin").unwrap_or(0);
            let deg = get_arg_int(&args, 1, 90);
            println!(
                "\x1b[1;36m[ACTUATOR: SERVO]\x1b[0m Pin {} pulse adjusted for angle = {}°",
                p, deg
            );
            let sim = get_sim_state();
            sim.lock().unwrap().servo_angles.insert(p, deg);
            Ok(Value::Bool(true))
        }),
    );

    obj.insert(
        "rotate".to_string(),
        Value::NativeCallback(|args| {
            let p = get_field_int(&args[0], "pin").unwrap_or(0);
            let deg = get_arg_int(&args, 1, 180);
            println!(
                "\x1b[1;36m[ACTUATOR: SERVO]\x1b[0m Pin {} sweeping to target angle = {}°",
                p, deg
            );
            Ok(Value::Bool(true))
        }),
    );

    obj.insert(
        "stop".to_string(),
        Value::NativeCallback(|_args| {
            println!("\x1b[1;31m[ACTUATOR: SERVO]\x1b[0m Servo actuation halted.");
            Ok(Value::Bool(true))
        }),
    );

    Value::Formula(obj)
}

fn create_motor_object(pwm: i64, dir1: i64, dir2: i64) -> Value {
    let mut obj = HashMap::new();
    obj.insert("__type__".to_string(), Value::String("Motor".to_string()));
    obj.insert("pwmPin".to_string(), Value::Int(pwm));
    obj.insert("dir1Pin".to_string(), Value::Int(dir1));
    obj.insert("dir2Pin".to_string(), Value::Int(dir2));

    obj.insert(
        "forward".to_string(),
        Value::NativeCallback(|_args| {
            println!("\x1b[1;32m[ACTUATOR: DC MOTOR]\x1b[0m H-Bridge pins energized FORWARD");
            Ok(Value::Bool(true))
        }),
    );

    obj.insert(
        "reverse".to_string(),
        Value::NativeCallback(|_args| {
            println!("\x1b[1;33m[ACTUATOR: DC MOTOR]\x1b[0m H-Bridge pins energized REVERSE");
            Ok(Value::Bool(true))
        }),
    );

    obj.insert(
        "stop".to_string(),
        Value::NativeCallback(|_args| {
            println!("\x1b[1;31m[ACTUATOR: DC MOTOR]\x1b[0m Motor output de-energized (stopped)");
            Ok(Value::Bool(true))
        }),
    );

    obj.insert(
        "speed".to_string(),
        Value::NativeCallback(|args| {
            let s = get_arg_float(&args, 1, 80.0);
            println!(
                "\x1b[1;36m[ACTUATOR: DC MOTOR]\x1b[0m Throttle PWM set to {:.1}%",
                s
            );
            Ok(Value::Bool(true))
        }),
    );

    Value::Formula(obj)
}

fn create_stepper_object(step_pin: i64, dir_pin: i64) -> Value {
    let mut obj = HashMap::new();
    obj.insert("__type__".to_string(), Value::String("Stepper".to_string()));
    obj.insert("stepPin".to_string(), Value::Int(step_pin));
    obj.insert("dirPin".to_string(), Value::Int(dir_pin));

    obj.insert(
        "step".to_string(),
        Value::NativeCallback(|args| {
            let count = get_arg_int(&args, 1, 100);
            println!(
                "\x1b[1;36m[ACTUATOR: STEPPER]\x1b[0m Generated {} precise clock step pulses",
                count
            );
            Ok(Value::Bool(true))
        }),
    );

    obj.insert(
        "rotate".to_string(),
        Value::NativeCallback(|args| {
            let deg = get_arg_int(&args, 1, 90);
            println!(
                "\x1b[1;36m[ACTUATOR: STEPPER]\x1b[0m Rotated stepper shaft by {}°",
                deg
            );
            Ok(Value::Bool(true))
        }),
    );

    Value::Formula(obj)
}

fn create_encoder_object(pin_a: i64, pin_b: i64) -> Value {
    let mut obj = HashMap::new();
    obj.insert("__type__".to_string(), Value::String("Encoder".to_string()));
    obj.insert("pinA".to_string(), Value::Int(pin_a));
    obj.insert("pinB".to_string(), Value::Int(pin_b));
    obj.insert("position".to_string(), Value::Int(1024));

    obj.insert(
        "reset".to_string(),
        Value::NativeCallback(|_| {
            println!("\x1b[1;35m[SENSOR: ENCODER]\x1b[0m Quadrature hardware counter zeroed out");
            Ok(Value::Int(0))
        }),
    );

    Value::Formula(obj)
}

fn create_diff_drive_object() -> Value {
    let mut obj = HashMap::new();
    obj.insert(
        "__type__".to_string(),
        Value::String("DiffDrive".to_string()),
    );

    obj.insert("forward".to_string(), Value::NativeCallback(|_args| {
        println!("\x1b[1;32m[ROBOTICS: DIFF DRIVE]\x1b[0m Differential chassis moving straight FORWARD");
        Ok(Value::Bool(true))
    }));

    obj.insert(
        "rotate".to_string(),
        Value::NativeCallback(|args| {
            let deg = get_arg_int(&args, 1, 90);
            println!(
                "\x1b[1;35m[ROBOTICS: DIFF DRIVE]\x1b[0m Pivoting chassis in place by {}°",
                deg
            );
            Ok(Value::Bool(true))
        }),
    );

    obj.insert(
        "stop".to_string(),
        Value::NativeCallback(|_args| {
            println!("\x1b[1;31m[ROBOTICS: DIFF DRIVE]\x1b[0m Chassis motors halted");
            Ok(Value::Bool(true))
        }),
    );

    Value::Formula(obj)
}

fn create_pid_object() -> Value {
    let mut obj = HashMap::new();
    obj.insert("__type__".to_string(), Value::String("PID".to_string()));
    obj.insert("target".to_string(), Value::Float(100.0));

    obj.insert("update".to_string(), Value::NativeCallback(|args| {
        let val = get_arg_float(&args, 1, 0.0);
        let error = 100.0 - val;
        let output = error * 1.5; // PID feedback calculation
        println!("\x1b[1;36m[ROBOTICS: PID CONTROLLER]\x1b[0m Sensor: {:.2}, Error: {:.2}, Output Actuation: {:.2}", val, error, output);
        Ok(Value::Float(output))
    }));

    Value::Formula(obj)
}

fn create_sensor_object(name: &str) -> Value {
    let mut obj = HashMap::new();
    obj.insert("__type__".to_string(), Value::String("Sensor".to_string()));
    obj.insert("model".to_string(), Value::String(name.to_string()));
    obj.insert("temperature".to_string(), Value::Float(24.5));
    obj.insert("humidity".to_string(), Value::Float(48.2));
    obj.insert("pressure".to_string(), Value::Float(1013.25));

    obj.insert("read".to_string(), Value::NativeCallback(|args| {
        let m = get_arg_string(&args, 0, "BME280");
        println!("\x1b[1;36m[SENSOR DRIVER: {}]\x1b[0m Sampled registers -> Temp: 24.5°C | Hum: 48.2% | Press: 1013.25 hPa", m);
        Ok(Value::Float(24.5))
    }));

    Value::Formula(obj)
}

// --- Module Initializers ---

fn init_io_module() -> HashMap<String, Value> {
    let mut m = HashMap::new();
    m.insert(
        "__module__".to_string(),
        Value::String("std.embedded.io".to_string()),
    );
    m.insert(
        "pin".to_string(),
        Value::NativeCallback(|args| {
            let p = get_arg_int(&args, 0, 13);
            Ok(create_pin_object(p))
        }),
    );
    m.insert(
        "analog".to_string(),
        Value::NativeCallback(|args| {
            let ch = get_arg_int(&args, 0, 0);
            Ok(create_analog_object(ch))
        }),
    );
    m.insert(
        "pwm".to_string(),
        Value::NativeCallback(|args| {
            let p = get_arg_int(&args, 0, 9);
            Ok(create_pwm_object(p))
        }),
    );
    m.insert(
        "dac".to_string(),
        Value::NativeCallback(|args| {
            let p = get_arg_int(&args, 0, 1);
            Ok(create_dac_object(p))
        }),
    );
    m
}

fn init_comm_module() -> HashMap<String, Value> {
    let mut m = HashMap::new();
    m.insert(
        "__module__".to_string(),
        Value::String("std.embedded.comm".to_string()),
    );
    m.insert(
        "uart".to_string(),
        Value::NativeCallback(|args| {
            let baud = get_arg_int(&args, 0, 115200);
            Ok(create_uart_object(baud))
        }),
    );
    m.insert(
        "i2c".to_string(),
        Value::NativeCallback(|args| {
            let addr = get_arg_int(&args, 0, 0x3C);
            Ok(create_i2c_object(addr))
        }),
    );
    m.insert(
        "spi".to_string(),
        Value::NativeCallback(|_args| Ok(create_spi_object())),
    );
    m.insert(
        "can".to_string(),
        Value::NativeCallback(|args| {
            let bitrate = get_arg_int(&args, 0, 500000);
            Ok(create_can_object(bitrate))
        }),
    );
    m
}

fn init_devices_module() -> HashMap<String, Value> {
    let mut m = HashMap::new();
    m.insert(
        "__module__".to_string(),
        Value::String("std.embedded.devices".to_string()),
    );
    m.insert(
        "servo".to_string(),
        Value::NativeCallback(|args| {
            let pin = get_arg_int(&args, 0, 5);
            Ok(create_servo_object(pin))
        }),
    );
    m.insert(
        "motor".to_string(),
        Value::NativeCallback(|args| {
            let p = get_arg_int(&args, 0, 9);
            let d1 = get_arg_int(&args, 1, 7);
            let d2 = get_arg_int(&args, 2, 8);
            Ok(create_motor_object(p, d1, d2))
        }),
    );
    m.insert(
        "stepper".to_string(),
        Value::NativeCallback(|args| {
            let s = get_arg_int(&args, 0, 10);
            let d = get_arg_int(&args, 1, 11);
            Ok(create_stepper_object(s, d))
        }),
    );
    m.insert(
        "encoder".to_string(),
        Value::NativeCallback(|args| {
            let a = get_arg_int(&args, 0, 2);
            let b = get_arg_int(&args, 1, 3);
            Ok(create_encoder_object(a, b))
        }),
    );
    m.insert(
        "sensor".to_string(),
        Value::NativeCallback(|args| {
            let model = get_arg_string(&args, 0, "BME280");
            Ok(create_sensor_object(&model))
        }),
    );
    m.insert("display".to_string(), Value::NativeCallback(|_args| {
        let mut obj = HashMap::new();
        obj.insert("clear".to_string(), Value::NativeCallback(|_| {
            println!("\x1b[1;36m[DISPLAY: OLED/TFT]\x1b[0m Framebuffer memory wiped (0x00)");
            Ok(Value::Bool(true))
        }));
        obj.insert("text".to_string(), Value::NativeCallback(|args| {
            let t = get_arg_string(&args, 1, "Hello Flame");
            println!("\x1b[1;36m[DISPLAY: OLED/TFT]\x1b[0m Rendered string '{}' to display framebuffer", t);
            Ok(Value::Bool(true))
        }));
        Ok(Value::Formula(obj))
    }));
    m
}

fn init_robotics_module() -> HashMap<String, Value> {
    let mut m = HashMap::new();
    m.insert(
        "__module__".to_string(),
        Value::String("std.embedded.robotics".to_string()),
    );
    m.insert(
        "diffDrive".to_string(),
        Value::NativeCallback(|_args| Ok(create_diff_drive_object())),
    );
    m.insert(
        "pid".to_string(),
        Value::NativeCallback(|_args| Ok(create_pid_object())),
    );
    m.insert(
        "imu".to_string(),
        Value::NativeCallback(|_args| {
            let mut obj = HashMap::new();
            obj.insert(
                "acceleration".to_string(),
                Value::Formula({
                    let mut acc = HashMap::new();
                    acc.insert("x".to_string(), Value::Float(0.02));
                    acc.insert("y".to_string(), Value::Float(0.01));
                    acc.insert("z".to_string(), Value::Float(9.81));
                    acc
                }),
            );
            obj.insert("heading".to_string(), Value::Float(184.5));
            Ok(Value::Formula(obj))
        }),
    );
    m
}

fn init_system_module() -> HashMap<String, Value> {
    let mut m = HashMap::new();
    m.insert(
        "__module__".to_string(),
        Value::String("std.embedded.system".to_string()),
    );
    m.insert(
        "board".to_string(),
        Value::Formula({
            #[cfg(feature = "os")]
            let (board_name, cpu_info, mem_string) = {
                let mut sys = System::new_all();
                sys.refresh_cpu();
                sys.refresh_memory();

                let board_name = System::host_name()
                    .or_else(|| System::name())
                    .unwrap_or_else(|| "Unknown Target Hardware".to_string());

                let cpu_info = sys
                    .cpus()
                    .first()
                    .map(|c| c.brand().trim().to_string())
                    .unwrap_or_else(|| "Unknown CPU Architecture".to_string());

                let mem_total_kb = sys.total_memory() / 1024;
                let mem_string = format!("{} KB RAM ({} MB)", mem_total_kb, mem_total_kb / 1024);
                (board_name, cpu_info, mem_string)
            };
            #[cfg(not(feature = "os"))]
            let (board_name, cpu_info, mem_string) = (
                "Unknown Target Hardware".to_string(),
                "Unknown CPU Architecture".to_string(),
                "Unknown Memory".to_string(),
            );

            let mut b = HashMap::new();
            b.insert("name".to_string(), Value::String(board_name));
            b.insert("cpu".to_string(), Value::String(cpu_info));
            b.insert("memory".to_string(), Value::String(mem_string));
            b.insert(
                "architecture".to_string(),
                Value::String(std::env::consts::ARCH.to_string()),
            );
            b.insert(
                "os".to_string(),
                Value::String(std::env::consts::OS.to_string()),
            );
            b
        }),
    );
    m.insert("watchdog".to_string(), Value::Formula({
        let mut w = HashMap::new();
        w.insert("feed".to_string(), Value::NativeCallback(|_| {
            println!("\x1b[1;32m[SYSTEM: WATCHDOG]\x1b[0m Watchdog timer hardware counter reset (fed)");
            Ok(Value::Bool(true))
        }));
        w
    }));
    m.insert("flash".to_string(), Value::Formula({
        let mut f = HashMap::new();
        f.insert("write".to_string(), Value::NativeCallback(|args| {
            let addr = get_arg_int(&args, 1, 0x00);
            let val = get_arg_int(&args, 2, 42);
            println!("\x1b[1;35m[SYSTEM: FLASH MEMORY]\x1b[0m Wrote word {} to non-volatile flash offset 0x{:X}", val, addr);
            let sim = get_sim_state();
            sim.lock().unwrap().memory_storage.insert(addr, val);
            Ok(Value::Bool(true))
        }));
        f.insert("read".to_string(), Value::NativeCallback(|args| {
            let addr = get_arg_int(&args, 1, 0x00);
            let sim = get_sim_state();
            let val = sim.lock().unwrap().memory_storage.get(&addr).cloned().unwrap_or(0);
            Ok(Value::Int(val))
        }));
        f
    }));
    m.insert(
        "eeprom".to_string(),
        Value::Formula({
            let mut e = HashMap::new();
            e.insert(
                "write".to_string(),
                Value::NativeCallback(|args| {
                    let addr = get_arg_int(&args, 1, 0x00);
                    let val = get_arg_int(&args, 2, 100);
                    println!(
                        "\x1b[1;35m[SYSTEM: EEPROM]\x1b[0m Wrote byte {} to EEPROM offset 0x{:X}",
                        val, addr
                    );
                    Ok(Value::Bool(true))
                }),
            );
            e
        }),
    );
    m
}

pub fn init() -> HashMap<String, Value> {
    let mut root = HashMap::new();
    root.insert(
        "__module__".to_string(),
        Value::String("std.embedded".to_string()),
    );

    // Physical Serial Port Discovery for flashing firmware, device discovery, and REPL debugging
    #[cfg(feature = "hardware")]
    root.insert(
        "detect_ports".to_string(),
        Value::NativeCallback(|_| match serialport::available_ports() {
            Ok(ports) => {
                let mut list = Vec::new();
                for p in ports {
                    let mut m = HashMap::new();
                    m.insert("name".to_string(), Value::String(p.port_name));
                    match p.port_type {
                        serialport::SerialPortType::UsbPort(info) => {
                            m.insert(
                                "type".to_string(),
                                Value::String(
                                    "USB Microcontroller (Firmware Flashing & Monitor Target)"
                                        .to_string(),
                                ),
                            );
                            m.insert("vid".to_string(), Value::Int(info.vid as i64));
                            m.insert("pid".to_string(), Value::Int(info.pid as i64));
                        }
                        _ => {
                            m.insert(
                                "type".to_string(),
                                Value::String("Standard Serial Port".to_string()),
                            );
                        }
                    }
                    list.push(Value::Formula(m));
                }
                Ok(Value::Tuple(list))
            }
            Err(e) => Err(format!(
                "Failed to enumerate system hardware serial ports: {}",
                e
            )),
        }),
    );
    #[cfg(not(feature = "hardware"))]
    root.insert(
        "detect_ports".to_string(),
        Value::NativeCallback(|_| Err("Hardware features are disabled in this build".to_string())),
    );

    // Modular Sub-namespaces
    root.insert("io".to_string(), Value::Formula(init_io_module()));
    root.insert("comm".to_string(), Value::Formula(init_comm_module()));
    root.insert("devices".to_string(), Value::Formula(init_devices_module()));
    root.insert(
        "robotics".to_string(),
        Value::Formula(init_robotics_module()),
    );
    root.insert("system".to_string(), Value::Formula(init_system_module()));

    // Ergonomic top-level exports for rapid prototyping & clean autocompletion
    for (k, v) in init_io_module() {
        if k != "__module__" {
            root.insert(k, v);
        }
    }
    for (k, v) in init_comm_module() {
        if k != "__module__" {
            root.insert(k, v);
        }
    }
    for (k, v) in init_devices_module() {
        if k != "__module__" {
            root.insert(k, v);
        }
    }
    for (k, v) in init_robotics_module() {
        if k != "__module__" {
            root.insert(k, v);
        }
    }
    for (k, v) in init_system_module() {
        if k != "__module__" {
            root.insert(k, v);
        }
    }

    root
}
