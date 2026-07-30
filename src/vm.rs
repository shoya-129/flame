use crate::parser::{Param, Stmt};
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex, OnceLock};
use std::thread::JoinHandle;

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Nil,
    Bytes(Vec<u8>),
    Tuple(Vec<Value>),
    Formula(HashMap<String, Value>),
    ThreadHandler(u64),
    Sender(u64),
    Receiver(u64),
    ChildProcess(u64),
    CommandBuilder {
        program: String,
        args: Vec<String>,
    },
    RustServer {
        port: u16,
    },
    StructConstructor {
        name: String,
        fields: Vec<(String, String)>,
    },
    Function {
        params: Vec<Param>,
        body: Vec<Stmt>,
        env: Arc<Mutex<Env>>,
    },
    Break,
    Return(Box<Value>),
    Moved(String),
    Ref(Box<Value>),
    RefPath(RefPath, bool),
    NativeObject {
        crate_name: String,
        type_name: String,
        ptr: usize,
    },
    Object(HashMap<String, Value>),
    NativeFunction(fn(*const CValue, usize) -> CValue),
    NativeCallback(fn(Vec<Value>) -> Result<Value, String>),
    Range(i64, i64),
    EnumMeta(String, Vec<crate::parser::EnumVariant>),
    EnumValue(String, String, EnumData),
    VariantConstructor(String, crate::parser::EnumVariant),
}

#[derive(Debug, Clone)]
pub enum RefPath {
    Var(String),
    Field { owner: String, member: String },
}

#[derive(Debug, Clone)]
pub enum EnumData {
    Unit,
    Tuple(Vec<Value>),
    Struct(HashMap<String, Value>),
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CValueTag {
    Null,
    Int,
    Float,
    Bool,
    String,
    NativeObject,
    Range,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct CValue {
    pub tag: CValueTag,
    pub int_val: i64,
    pub int_val2: i64,
    pub float_val: f64,
    pub bool_val: bool,
    pub string_ptr: *mut std::os::raw::c_char,
    pub obj_ptr: *mut std::ffi::c_void,
}

impl CValue {
    pub fn null() -> Self {
        Self {
            tag: CValueTag::Null,
            int_val: 0,
            int_val2: 0,
            float_val: 0.0,
            bool_val: false,
            string_ptr: std::ptr::null_mut(),
            obj_ptr: std::ptr::null_mut(),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::NativeObject { type_name, .. } => write!(f, "<native object: {}>", type_name),
            Value::Moved(name) => write!(f, "<moved {}>", name),
            Value::Ref(inner) => inner.fmt(f),
            Value::Int(i) => write!(f, "{i}"),
            Value::Float(v) => write!(f, "{v}"),
            Value::String(s) => write!(f, "{s}"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Nil => write!(f, "nil"),
            Value::Bytes(bytes) => {
                let hex: Vec<String> = bytes.iter().map(|b| format!("{:02X}", b)).collect();

                write!(f, "bytes[{}]", hex.join(" "))
            }
            Value::Tuple(items) => {
                let s: Vec<String> = items.iter().map(|it| it.to_string()).collect();
                write!(f, "({})", s.join(", "))
            }
            Value::Formula(map) => {
                let s: Vec<String> = map.iter().map(|(k, v)| format!("{}: {}", k, v)).collect();
                write!(f, "formula {{ {} }}", s.join(", "))
            }
            Value::Object(map) => {
                write!(
                    f,
                    "<object [{}]>",
                    map.keys().cloned().collect::<Vec<_>>().join(", ")
                )
            }
            Value::Range(start, end) => write!(f, "{}..{}", start, end),
            Value::ThreadHandler(id) => write!(f, "ThreadHandler({})", id),
            Value::Sender(id) => write!(f, "Sender({})", id),
            Value::Receiver(id) => write!(f, "Receiver({})", id),
            Value::ChildProcess(id) => write!(f, "ChildProcess({})", id),
            Value::CommandBuilder { program, args } => {
                write!(f, "CommandBuilder({} {})", program, args.join(" "))
            }
            Value::RustServer { port } => write!(f, "RustServer(127.0.0.1:{})", port),
            Value::StructConstructor { name, .. } => write!(f, "<struct constructor: {}>", name),
            Value::Function { .. } => write!(f, "<function>"),
            Value::Break => write!(f, "<break>"),
            Value::Return(val) => write!(f, "<return {}>", val),
            Value::NativeFunction(_) => write!(f, "<native function>"),
            Value::NativeCallback(_) => write!(f, "<native callback>"),
            Value::EnumMeta(name, _) => write!(f, "<enum {}>", name),
            Value::EnumValue(enum_name, variant_name, data) => match data {
                EnumData::Unit => {
                    write!(f, "{}.{}", enum_name, variant_name)
                }

                EnumData::Tuple(items) => {
                    let s: Vec<String> = items.iter().map(|it| it.to_string()).collect();

                    write!(f, "{}.{}({})", enum_name, variant_name, s.join(", "))
                }

                EnumData::Struct(map) => {
                    let s: Vec<String> = map.iter().map(|(k, v)| format!("{k}: {v}")).collect();

                    write!(f, "{}.{} {{ {} }}", enum_name, variant_name, s.join(", "))
                }
            },
            Value::VariantConstructor(enum_name, var) => {
                let var_name = match var {
                    crate::parser::EnumVariant::Unit(n) => n,
                    crate::parser::EnumVariant::Tuple(n, _) => n,
                    crate::parser::EnumVariant::Struct(n, _) => n,
                };
                write!(f, "<enum constructor: {}.{}>", enum_name, var_name)
            }
            Value::RefPath(path, mutable) => match path {
                RefPath::Var(name) => {
                    if *mutable {
                        write!(f, "&mut {name}")
                    } else {
                        write!(f, "&{name}")
                    }
                }
                RefPath::Field { owner, member } => {
                    if *mutable {
                        write!(f, "&mut {owner}.{member}")
                    } else {
                        write!(f, "&{owner}.{member}")
                    }
                }
            },
        }
    }
}

impl Value {
    pub fn pack(&self) -> CValue {
        match self {
            Value::Int(i) => CValue {
                tag: CValueTag::Int,
                int_val: *i,
                int_val2: 0,
                float_val: 0.0,
                bool_val: false,
                string_ptr: std::ptr::null_mut(),
                obj_ptr: std::ptr::null_mut(),
            },
            Value::Float(f) => CValue {
                tag: CValueTag::Float,
                int_val: 0,
                int_val2: 0,
                float_val: *f,
                bool_val: false,
                string_ptr: std::ptr::null_mut(),
                obj_ptr: std::ptr::null_mut(),
            },
            Value::Bool(b) => CValue {
                tag: CValueTag::Bool,
                int_val: 0,
                int_val2: 0,
                float_val: 0.0,
                bool_val: *b,
                string_ptr: std::ptr::null_mut(),
                obj_ptr: std::ptr::null_mut(),
            },
            Value::Range(start, end) => CValue {
                tag: CValueTag::Range,
                int_val: *start,
                int_val2: *end,
                float_val: 0.0,
                bool_val: false,
                string_ptr: std::ptr::null_mut(),
                obj_ptr: std::ptr::null_mut(),
            },
            Value::String(s) => {
                let c_str = std::ffi::CString::new(s.clone()).unwrap_or_default();
                CValue {
                    tag: CValueTag::String,
                    int_val: 0,
                    int_val2: 0,
                    float_val: 0.0,
                    bool_val: false,
                    string_ptr: c_str.into_raw(),
                    obj_ptr: std::ptr::null_mut(),
                }
            }
            Value::NativeObject { ptr, .. } => CValue {
                tag: CValueTag::NativeObject,
                int_val: 0,
                int_val2: 0,
                float_val: 0.0,
                bool_val: false,
                string_ptr: std::ptr::null_mut(),
                obj_ptr: *ptr as *mut std::ffi::c_void,
            },
            Value::Return(inner) => inner.pack(),
            _ => CValue::null(),
        }
    }

    pub fn unpack(cval: CValue, crate_name: &str, type_name: &str) -> Self {
        match cval.tag {
            CValueTag::Null => Value::Nil,
            CValueTag::Int => Value::Int(cval.int_val),
            CValueTag::Float => Value::Float(cval.float_val),
            CValueTag::Bool => Value::Bool(cval.bool_val),
            CValueTag::Range => Value::Range(cval.int_val, cval.int_val2),
            CValueTag::String => {
                if cval.string_ptr.is_null() {
                    Value::String(String::new())
                } else {
                    let s = unsafe {
                        std::ffi::CStr::from_ptr(cval.string_ptr)
                            .to_string_lossy()
                            .into_owned()
                    };
                    unsafe {
                        let _ = std::ffi::CString::from_raw(cval.string_ptr);
                    }
                    Value::String(s)
                }
            }
            CValueTag::NativeObject => Value::NativeObject {
                crate_name: crate_name.to_string(),
                type_name: type_name.to_string(),
                ptr: cval.obj_ptr as usize,
            },
        }
    }

    pub fn as_int(&self) -> Result<i64, String> {
        match self {
            Value::Int(i) => Ok(*i),
            _ => Err("expected int".into()),
        }
    }

    pub fn as_bool(&self) -> Result<bool, String> {
        match self {
            Value::Bool(b) => Ok(*b),
            _ => Err("expected bool".into()),
        }
    }

    pub fn as_bytes(&self) -> Result<Vec<u8>, String> {
        match self {
            Value::Bytes(bytes) => Ok(bytes.clone()),

            Value::Tuple(values) => {
                let mut out = Vec::new();

                for value in values {
                    match value {
                        Value::Int(i) if *i >= 0 && *i <= 255 => {
                            out.push(*i as u8);
                        }
                        _ => {
                            return Err("expected tuple of integers between 0 and 255".into());
                        }
                    }
                }

                Ok(out)
            }

            _ => Err("expected bytes".into()),
        }
    }
}
pub static THREADS: OnceLock<Mutex<HashMap<u64, JoinHandle<Value>>>> = OnceLock::new();
pub static THREAD_COUNTER: OnceLock<Mutex<u64>> = OnceLock::new();
pub static CHANNELS: OnceLock<Mutex<HashMap<u64, std::sync::mpsc::Sender<Value>>>> =
    OnceLock::new();
pub static RECEIVERS: OnceLock<Mutex<HashMap<u64, Arc<Mutex<std::sync::mpsc::Receiver<Value>>>>>> =
    OnceLock::new();
pub static CHANNEL_COUNTER: OnceLock<Mutex<u64>> = OnceLock::new();

pub fn get_threads() -> &'static Mutex<HashMap<u64, JoinHandle<Value>>> {
    THREADS.get_or_init(|| Mutex::new(HashMap::new()))
}
pub fn get_thread_counter() -> &'static Mutex<u64> {
    THREAD_COUNTER.get_or_init(|| Mutex::new(0))
}
pub fn get_channels() -> &'static Mutex<HashMap<u64, std::sync::mpsc::Sender<Value>>> {
    CHANNELS.get_or_init(|| Mutex::new(HashMap::new()))
}
pub fn get_receivers() -> &'static Mutex<HashMap<u64, Arc<Mutex<std::sync::mpsc::Receiver<Value>>>>>
{
    RECEIVERS.get_or_init(|| Mutex::new(HashMap::new()))
}
pub fn get_channel_counter() -> &'static Mutex<u64> {
    CHANNEL_COUNTER.get_or_init(|| Mutex::new(0))
}

pub fn wait_for_all_threads() {
    loop {
        let handle = {
            let mut registry = get_threads().lock().unwrap();

            if registry.is_empty() {
                break;
            }

            let id = *registry.keys().next().unwrap();

            registry.remove(&id)
        };

        if let Some(handle) = handle {
            let _ = handle.join();
        }
    }
}

#[derive(Debug, Clone)]
pub struct VarEntry {
    pub value: Value,
    pub is_mut: bool,
}

#[derive(Debug, Clone)]
pub struct Env {
    pub variables: HashMap<String, VarEntry>,
    pub parent: Option<Arc<Mutex<Env>>>,
}

impl Env {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            parent: None,
        }
    }

    pub fn fork(&self) -> Self {
        Self {
            variables: self.variables.clone(),
            parent: None,
        }
    }

    pub fn new_child(parent: Arc<Mutex<Env>>) -> Self {
        Self {
            variables: HashMap::new(),
            parent: Some(parent),
        }
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(entry) = self.variables.get(name) {
            Some(entry.value.clone())
        } else if let Some(parent) = &self.parent {
            parent.lock().unwrap().get(name)
        } else {
            None
        }
    }

    pub fn define(&mut self, name: String, val: Value, is_mut: bool) {
        self.variables.insert(name, VarEntry { value: val, is_mut });
    }

    pub fn assign(&mut self, name: String, val: Value) -> Result<(), String> {
        if let Some(entry) = self.variables.get_mut(&name) {
            if !entry.is_mut {
                return Err(format!(
                    "cannot mutate immutable variable '{}'. Declare with 'let mut' or 'var' to allow reassignment.",
                    name
                ));
            }
            entry.value = val;
            Ok(())
        } else if let Some(parent) = &self.parent {
            parent.lock().unwrap().assign(name, val)
        } else {
            Err(format!(
                "cannot assign to undeclared variable '{}'. Declare it first using 'let mut' or 'var'.",
                name
            ))
        }
    }

    pub fn move_var(&mut self, name: &str) {
        if let Some(entry) = self.variables.get_mut(name) {
            entry.value = Value::Moved(name.to_string());
        } else if let Some(parent) = &self.parent {
            parent.lock().unwrap().move_var(name);
        }
    }

    pub fn to_formula_map(&self) -> HashMap<String, Value> {
        let mut map = HashMap::new();
        for (k, entry) in &self.variables {
            map.insert(k.clone(), entry.value.clone());
        }
        map
    }
}
