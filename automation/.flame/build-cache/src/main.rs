#![allow(unused_variables, dead_code, unused_imports, non_snake_case)]
use flamelang::runner::{Runner, CValue};
use std::path::PathBuf;

use flamelang::vm;
// Wrapper for crate uuid
mod bridge_uuid {
    use super::*;
    pub fn Uuid_nil(_args: *const CValue, _len: usize) -> CValue {
        let mut c_args = unsafe { std::slice::from_raw_parts(_args, _len) };
        let res = uuid::Uuid::nil();
        let c_str = std::ffi::CString::new(res.to_string()).unwrap_or_default();
        let mut cv = CValue::null();
        cv.tag = flamelang::runner::CValueTag::String;
        cv.string_ptr = c_str.into_raw();
        cv
    }
    pub fn Uuid_max(_args: *const CValue, _len: usize) -> CValue {
        let mut c_args = unsafe { std::slice::from_raw_parts(_args, _len) };
        let res = uuid::Uuid::max();
        let c_str = std::ffi::CString::new(res.to_string()).unwrap_or_default();
        let mut cv = CValue::null();
        cv.tag = flamelang::runner::CValueTag::String;
        cv.string_ptr = c_str.into_raw();
        cv
    }
    pub fn Uuid_from_u128(_args: *const CValue, _len: usize) -> CValue {
        let mut c_args = unsafe { std::slice::from_raw_parts(_args, _len) };
        let arg0 = c_args[0].int_val as u128;
        let res = uuid::Uuid::from_u128(arg0);
        let c_str = std::ffi::CString::new(res.to_string()).unwrap_or_default();
        let mut cv = CValue::null();
        cv.tag = flamelang::runner::CValueTag::String;
        cv.string_ptr = c_str.into_raw();
        cv
    }
    pub fn Uuid_from_u128_le(_args: *const CValue, _len: usize) -> CValue {
        let mut c_args = unsafe { std::slice::from_raw_parts(_args, _len) };
        let arg0 = c_args[0].int_val as u128;
        let res = uuid::Uuid::from_u128_le(arg0);
        let c_str = std::ffi::CString::new(res.to_string()).unwrap_or_default();
        let mut cv = CValue::null();
        cv.tag = flamelang::runner::CValueTag::String;
        cv.string_ptr = c_str.into_raw();
        cv
    }
    pub fn Uuid_from_u64_pair(_args: *const CValue, _len: usize) -> CValue {
        let mut c_args = unsafe { std::slice::from_raw_parts(_args, _len) };
        let arg0 = c_args[0].int_val as u64;
        let arg1 = c_args[1].int_val as u64;
        let res = uuid::Uuid::from_u64_pair(arg0, arg1);
        let c_str = std::ffi::CString::new(res.to_string()).unwrap_or_default();
        let mut cv = CValue::null();
        cv.tag = flamelang::runner::CValueTag::String;
        cv.string_ptr = c_str.into_raw();
        cv
    }
    pub fn Uuid_as_hyphenated(_args: *const CValue, _len: usize) -> CValue {
        let mut c_args = unsafe { std::slice::from_raw_parts(_args, _len) };
        // Self is arg 0, cast from obj_ptr
        let obj = unsafe { &mut *(c_args[0].obj_ptr as *mut uuid::Uuid) };
        let res = obj.as_hyphenated();
        let boxed = Box::new(res);
        let ptr = Box::into_raw(boxed) as *mut std::ffi::c_void;
        let mut cv = CValue::null();
        cv.tag = flamelang::runner::CValueTag::NativeObject;
        cv.obj_ptr = ptr;
        cv
    }
    pub fn Uuid_as_simple(_args: *const CValue, _len: usize) -> CValue {
        let mut c_args = unsafe { std::slice::from_raw_parts(_args, _len) };
        // Self is arg 0, cast from obj_ptr
        let obj = unsafe { &mut *(c_args[0].obj_ptr as *mut uuid::Uuid) };
        let res = obj.as_simple();
        let boxed = Box::new(res);
        let ptr = Box::into_raw(boxed) as *mut std::ffi::c_void;
        let mut cv = CValue::null();
        cv.tag = flamelang::runner::CValueTag::NativeObject;
        cv.obj_ptr = ptr;
        cv
    }
    pub fn Uuid_as_urn(_args: *const CValue, _len: usize) -> CValue {
        let mut c_args = unsafe { std::slice::from_raw_parts(_args, _len) };
        // Self is arg 0, cast from obj_ptr
        let obj = unsafe { &mut *(c_args[0].obj_ptr as *mut uuid::Uuid) };
        let res = obj.as_urn();
        let boxed = Box::new(res);
        let ptr = Box::into_raw(boxed) as *mut std::ffi::c_void;
        let mut cv = CValue::null();
        cv.tag = flamelang::runner::CValueTag::NativeObject;
        cv.obj_ptr = ptr;
        cv
    }
    pub fn Uuid_as_braced(_args: *const CValue, _len: usize) -> CValue {
        let mut c_args = unsafe { std::slice::from_raw_parts(_args, _len) };
        // Self is arg 0, cast from obj_ptr
        let obj = unsafe { &mut *(c_args[0].obj_ptr as *mut uuid::Uuid) };
        let res = obj.as_braced();
        let boxed = Box::new(res);
        let ptr = Box::into_raw(boxed) as *mut std::ffi::c_void;
        let mut cv = CValue::null();
        cv.tag = flamelang::runner::CValueTag::NativeObject;
        cv.obj_ptr = ptr;
        cv
    }
    pub fn Uuid_new_v4(_args: *const CValue, _len: usize) -> CValue {
        let mut c_args = unsafe { std::slice::from_raw_parts(_args, _len) };
        let res = uuid::Uuid::new_v4();
        let c_str = std::ffi::CString::new(res.to_string()).unwrap_or_default();
        let mut cv = CValue::null();
        cv.tag = flamelang::runner::CValueTag::String;
        cv.string_ptr = c_str.into_raw();
        cv
    }
    pub fn Uuid_get_variant(_args: *const CValue, _len: usize) -> CValue {
        let mut c_args = unsafe { std::slice::from_raw_parts(_args, _len) };
        // Self is arg 0, cast from obj_ptr
        let obj = unsafe { &mut *(c_args[0].obj_ptr as *mut uuid::Uuid) };
        let res = obj.get_variant();
        let boxed = Box::new(res);
        let ptr = Box::into_raw(boxed) as *mut std::ffi::c_void;
        let mut cv = CValue::null();
        cv.tag = flamelang::runner::CValueTag::NativeObject;
        cv.obj_ptr = ptr;
        cv
    }
    pub fn Uuid_get_version_num(_args: *const CValue, _len: usize) -> CValue {
        let mut c_args = unsafe { std::slice::from_raw_parts(_args, _len) };
        // Self is arg 0, cast from obj_ptr
        let obj = unsafe { &mut *(c_args[0].obj_ptr as *mut uuid::Uuid) };
        let res = obj.get_version_num();
        let mut cv = CValue::null();
        cv.tag = flamelang::runner::CValueTag::Int;
        cv.int_val = res as i64;
        cv
    }
    pub fn Uuid_get_version(_args: *const CValue, _len: usize) -> CValue {
        let mut c_args = unsafe { std::slice::from_raw_parts(_args, _len) };
        // Self is arg 0, cast from obj_ptr
        let obj = unsafe { &mut *(c_args[0].obj_ptr as *mut uuid::Uuid) };
        let res = obj.get_version();
        let boxed = Box::new(res);
        let ptr = Box::into_raw(boxed) as *mut std::ffi::c_void;
        let mut cv = CValue::null();
        cv.tag = flamelang::runner::CValueTag::NativeObject;
        cv.obj_ptr = ptr;
        cv
    }
    pub fn Uuid_as_u128(_args: *const CValue, _len: usize) -> CValue {
        let mut c_args = unsafe { std::slice::from_raw_parts(_args, _len) };
        // Self is arg 0, cast from obj_ptr
        let obj = unsafe { &mut *(c_args[0].obj_ptr as *mut uuid::Uuid) };
        let res = obj.as_u128();
        let boxed = Box::new(res);
        let ptr = Box::into_raw(boxed) as *mut std::ffi::c_void;
        let mut cv = CValue::null();
        cv.tag = flamelang::runner::CValueTag::NativeObject;
        cv.obj_ptr = ptr;
        cv
    }
    pub fn Uuid_to_u128_le(_args: *const CValue, _len: usize) -> CValue {
        let mut c_args = unsafe { std::slice::from_raw_parts(_args, _len) };
        // Self is arg 0, cast from obj_ptr
        let obj = unsafe { &mut *(c_args[0].obj_ptr as *mut uuid::Uuid) };
        let res = obj.to_u128_le();
        let boxed = Box::new(res);
        let ptr = Box::into_raw(boxed) as *mut std::ffi::c_void;
        let mut cv = CValue::null();
        cv.tag = flamelang::runner::CValueTag::NativeObject;
        cv.obj_ptr = ptr;
        cv
    }
    pub fn Uuid_as_bytes(_args: *const CValue, _len: usize) -> CValue {
        let mut c_args = unsafe { std::slice::from_raw_parts(_args, _len) };
        // Self is arg 0, cast from obj_ptr
        let obj = unsafe { &mut *(c_args[0].obj_ptr as *mut uuid::Uuid) };
        let res = obj.as_bytes();
        let boxed = Box::new(res);
        let ptr = Box::into_raw(boxed) as *mut std::ffi::c_void;
        let mut cv = CValue::null();
        cv.tag = flamelang::runner::CValueTag::NativeObject;
        cv.obj_ptr = ptr;
        cv
    }
    pub fn Uuid_to_bytes_le(_args: *const CValue, _len: usize) -> CValue {
        let mut c_args = unsafe { std::slice::from_raw_parts(_args, _len) };
        // Self is arg 0, cast from obj_ptr
        let obj = unsafe { &mut *(c_args[0].obj_ptr as *mut uuid::Uuid) };
        let res = obj.to_bytes_le();
        let boxed = Box::new(res);
        let ptr = Box::into_raw(boxed) as *mut std::ffi::c_void;
        let mut cv = CValue::null();
        cv.tag = flamelang::runner::CValueTag::NativeObject;
        cv.obj_ptr = ptr;
        cv
    }
    pub fn Uuid_is_nil(_args: *const CValue, _len: usize) -> CValue {
        let mut c_args = unsafe { std::slice::from_raw_parts(_args, _len) };
        // Self is arg 0, cast from obj_ptr
        let obj = unsafe { &mut *(c_args[0].obj_ptr as *mut uuid::Uuid) };
        let res = obj.is_nil();
        let mut cv = CValue::null();
        cv.tag = flamelang::runner::CValueTag::Bool;
        cv.bool_val = res;
        cv
    }
    pub fn Uuid_is_max(_args: *const CValue, _len: usize) -> CValue {
        let mut c_args = unsafe { std::slice::from_raw_parts(_args, _len) };
        // Self is arg 0, cast from obj_ptr
        let obj = unsafe { &mut *(c_args[0].obj_ptr as *mut uuid::Uuid) };
        let res = obj.is_max();
        let mut cv = CValue::null();
        cv.tag = flamelang::runner::CValueTag::Bool;
        cv.bool_val = res;
        cv
    }
    pub fn Uuid_encode_buffer(_args: *const CValue, _len: usize) -> CValue {
        let mut c_args = unsafe { std::slice::from_raw_parts(_args, _len) };
        let res = uuid::Uuid::encode_buffer();
        let boxed = Box::new(res);
        let ptr = Box::into_raw(boxed) as *mut std::ffi::c_void;
        let mut cv = CValue::null();
        cv.tag = flamelang::runner::CValueTag::NativeObject;
        cv.obj_ptr = ptr;
        cv
    }
    pub fn Uuid_get_timestamp(_args: *const CValue, _len: usize) -> CValue {
        let mut c_args = unsafe { std::slice::from_raw_parts(_args, _len) };
        // Self is arg 0, cast from obj_ptr
        let obj = unsafe { &mut *(c_args[0].obj_ptr as *mut uuid::Uuid) };
        let res = obj.get_timestamp();
        let boxed = Box::new(res);
        let ptr = Box::into_raw(boxed) as *mut std::ffi::c_void;
        let mut cv = CValue::null();
        cv.tag = flamelang::runner::CValueTag::NativeObject;
        cv.obj_ptr = ptr;
        cv
    }
    pub fn Uuid_get_node_id(_args: *const CValue, _len: usize) -> CValue {
        let mut c_args = unsafe { std::slice::from_raw_parts(_args, _len) };
        // Self is arg 0, cast from obj_ptr
        let obj = unsafe { &mut *(c_args[0].obj_ptr as *mut uuid::Uuid) };
        let res = obj.get_node_id();
        let boxed = Box::new(res);
        let ptr = Box::into_raw(boxed) as *mut std::ffi::c_void;
        let mut cv = CValue::null();
        cv.tag = flamelang::runner::CValueTag::NativeObject;
        cv.obj_ptr = ptr;
        cv
    }
    pub fn Builder_from_u128(_args: *const CValue, _len: usize) -> CValue {
        let mut c_args = unsafe { std::slice::from_raw_parts(_args, _len) };
        let arg0 = c_args[0].int_val as u128;
        let res = uuid::Builder::from_u128(arg0);
        let boxed = Box::new(res);
        let ptr = Box::into_raw(boxed) as *mut std::ffi::c_void;
        let mut cv = CValue::null();
        cv.tag = flamelang::runner::CValueTag::NativeObject;
        cv.obj_ptr = ptr;
        cv
    }
    pub fn Builder_from_u128_le(_args: *const CValue, _len: usize) -> CValue {
        let mut c_args = unsafe { std::slice::from_raw_parts(_args, _len) };
        let arg0 = c_args[0].int_val as u128;
        let res = uuid::Builder::from_u128_le(arg0);
        let boxed = Box::new(res);
        let ptr = Box::into_raw(boxed) as *mut std::ffi::c_void;
        let mut cv = CValue::null();
        cv.tag = flamelang::runner::CValueTag::NativeObject;
        cv.obj_ptr = ptr;
        cv
    }
    pub fn Builder_nil(_args: *const CValue, _len: usize) -> CValue {
        let mut c_args = unsafe { std::slice::from_raw_parts(_args, _len) };
        let res = uuid::Builder::nil();
        let boxed = Box::new(res);
        let ptr = Box::into_raw(boxed) as *mut std::ffi::c_void;
        let mut cv = CValue::null();
        cv.tag = flamelang::runner::CValueTag::NativeObject;
        cv.obj_ptr = ptr;
        cv
    }
    pub fn Builder_as_uuid(_args: *const CValue, _len: usize) -> CValue {
        let mut c_args = unsafe { std::slice::from_raw_parts(_args, _len) };
        // Self is arg 0, cast from obj_ptr
        let obj = unsafe { &mut *(c_args[0].obj_ptr as *mut uuid::Builder) };
        let res = obj.as_uuid();
        let boxed = Box::new(res);
        let ptr = Box::into_raw(boxed) as *mut std::ffi::c_void;
        let mut cv = CValue::null();
        cv.tag = flamelang::runner::CValueTag::NativeObject;
        cv.obj_ptr = ptr;
        cv
    }
    pub fn Timestamp_from_gregorian_time(_args: *const CValue, _len: usize) -> CValue {
        let mut c_args = unsafe { std::slice::from_raw_parts(_args, _len) };
        let arg0 = c_args[0].int_val as u64;
        let arg1 = c_args[1].int_val as u16;
        let res = uuid::Timestamp::from_gregorian_time(arg0, arg1);
        let boxed = Box::new(res);
        let ptr = Box::into_raw(boxed) as *mut std::ffi::c_void;
        let mut cv = CValue::null();
        cv.tag = flamelang::runner::CValueTag::NativeObject;
        cv.obj_ptr = ptr;
        cv
    }
    pub fn Timestamp_from_unix_time(_args: *const CValue, _len: usize) -> CValue {
        let mut c_args = unsafe { std::slice::from_raw_parts(_args, _len) };
        let arg0 = c_args[0].int_val as u64;
        let arg1 = c_args[1].int_val as u32;
        let arg2 = c_args[2].int_val as u128;
        let arg3 = c_args[3].int_val as u8;
        let res = uuid::Timestamp::from_unix_time(arg0, arg1, arg2, arg3);
        let boxed = Box::new(res);
        let ptr = Box::into_raw(boxed) as *mut std::ffi::c_void;
        let mut cv = CValue::null();
        cv.tag = flamelang::runner::CValueTag::NativeObject;
        cv.obj_ptr = ptr;
        cv
    }
}

// Wrapper for crate rand
mod bridge_rand {
    use super::*;
    pub fn random(_args: *const CValue, _len: usize) -> CValue {
        let mut c_args = unsafe { std::slice::from_raw_parts(_args, _len) };
        let generic_type_cstr = unsafe { std::ffi::CStr::from_ptr(c_args[0].string_ptr) };
        let generic_type = generic_type_cstr.to_str().unwrap_or_default();
        match generic_type {
            "u8" => {
                let res = rand::random::<u8>();
                let mut cv = CValue::null();
                cv.tag = flamelang::runner::CValueTag::Int;
                cv.int_val = res as i64;
                return cv;
            }
            "u16" => {
                let res = rand::random::<u16>();
                let mut cv = CValue::null();
                cv.tag = flamelang::runner::CValueTag::Int;
                cv.int_val = res as i64;
                return cv;
            }
            "u32" => {
                let res = rand::random::<u32>();
                let mut cv = CValue::null();
                cv.tag = flamelang::runner::CValueTag::Int;
                cv.int_val = res as i64;
                return cv;
            }
            "u64" => {
                let res = rand::random::<u64>();
                let mut cv = CValue::null();
                cv.tag = flamelang::runner::CValueTag::Int;
                cv.int_val = res as i64;
                return cv;
            }
            "i8" => {
                let res = rand::random::<i8>();
                let mut cv = CValue::null();
                cv.tag = flamelang::runner::CValueTag::Int;
                cv.int_val = res as i64;
                return cv;
            }
            "i16" => {
                let res = rand::random::<i16>();
                let mut cv = CValue::null();
                cv.tag = flamelang::runner::CValueTag::Int;
                cv.int_val = res as i64;
                return cv;
            }
            "i32" => {
                let res = rand::random::<i32>();
                let mut cv = CValue::null();
                cv.tag = flamelang::runner::CValueTag::Int;
                cv.int_val = res as i64;
                return cv;
            }
            "i64" => {
                let res = rand::random::<i64>();
                let mut cv = CValue::null();
                cv.tag = flamelang::runner::CValueTag::Int;
                cv.int_val = res as i64;
                return cv;
            }
            "f32" => {
                let res = rand::random::<f32>();
                let mut cv = CValue::null();
                cv.tag = flamelang::runner::CValueTag::Float;
                cv.float_val = res as f64;
                return cv;
            }
            "f64" => {
                let res = rand::random::<f64>();
                let mut cv = CValue::null();
                cv.tag = flamelang::runner::CValueTag::Float;
                cv.float_val = res as f64;
                return cv;
            }
            "bool" => {
                let res = rand::random::<bool>();
                let mut cv = CValue::null();
                cv.tag = flamelang::runner::CValueTag::Bool;
                cv.bool_val = res;
                return cv;
            }
            _ => return CValue::null(),
        }
    }
    pub fn random_ratio(_args: *const CValue, _len: usize) -> CValue {
        let mut c_args = unsafe { std::slice::from_raw_parts(_args, _len) };
        let arg0 = c_args[0].int_val as u32;
        let arg1 = c_args[1].int_val as u32;
        let res = rand::random_ratio(arg0, arg1);
        let mut cv = CValue::null();
        cv.tag = flamelang::runner::CValueTag::Bool;
        cv.bool_val = res;
        cv
    }
}

// Wrapper for crate bridge
mod bridge_bridge {
    use super::*;
    pub fn rust_add(_args: *const CValue, _len: usize) -> CValue {
        let mut c_args = unsafe { std::slice::from_raw_parts(_args, _len) };
        let arg0 = c_args[0].int_val as i64;
        let arg1 = c_args[1].int_val as i64;
        let res = bridge::rust_add(arg0, arg1);
        let mut cv = CValue::null();
        cv.tag = flamelang::runner::CValueTag::Int;
        cv.int_val = res as i64;
        cv
    }
}

fn main() {
    let mut runner = Runner::new(PathBuf::from("src/main.fm"));
    runner.native_methods.insert("flame_uuid_Uuid_nil".to_string(), bridge_uuid::Uuid_nil as fn(*const CValue, usize) -> CValue);
    runner.native_methods.insert("flame_uuid_nil".to_string(), bridge_uuid::Uuid_nil as fn(*const CValue, usize) -> CValue);
    runner.native_methods.insert("flame_uuid_Uuid_max".to_string(), bridge_uuid::Uuid_max as fn(*const CValue, usize) -> CValue);
    runner.native_methods.insert("flame_uuid_max".to_string(), bridge_uuid::Uuid_max as fn(*const CValue, usize) -> CValue);
    runner.native_methods.insert("flame_uuid_Uuid_from_u128".to_string(), bridge_uuid::Uuid_from_u128 as fn(*const CValue, usize) -> CValue);
    runner.native_methods.insert("flame_uuid_from_u128".to_string(), bridge_uuid::Uuid_from_u128 as fn(*const CValue, usize) -> CValue);
    runner.native_methods.insert("flame_uuid_Uuid_from_u128_le".to_string(), bridge_uuid::Uuid_from_u128_le as fn(*const CValue, usize) -> CValue);
    runner.native_methods.insert("flame_uuid_from_u128_le".to_string(), bridge_uuid::Uuid_from_u128_le as fn(*const CValue, usize) -> CValue);
    runner.native_methods.insert("flame_uuid_Uuid_from_u64_pair".to_string(), bridge_uuid::Uuid_from_u64_pair as fn(*const CValue, usize) -> CValue);
    runner.native_methods.insert("flame_uuid_from_u64_pair".to_string(), bridge_uuid::Uuid_from_u64_pair as fn(*const CValue, usize) -> CValue);
    runner.native_methods.insert("flame_uuid_Uuid_as_hyphenated".to_string(), bridge_uuid::Uuid_as_hyphenated as fn(*const CValue, usize) -> CValue);
    runner.native_methods.insert("flame_uuid_as_hyphenated".to_string(), bridge_uuid::Uuid_as_hyphenated as fn(*const CValue, usize) -> CValue);
    runner.native_methods.insert("flame_uuid_Uuid_as_simple".to_string(), bridge_uuid::Uuid_as_simple as fn(*const CValue, usize) -> CValue);
    runner.native_methods.insert("flame_uuid_as_simple".to_string(), bridge_uuid::Uuid_as_simple as fn(*const CValue, usize) -> CValue);
    runner.native_methods.insert("flame_uuid_Uuid_as_urn".to_string(), bridge_uuid::Uuid_as_urn as fn(*const CValue, usize) -> CValue);
    runner.native_methods.insert("flame_uuid_as_urn".to_string(), bridge_uuid::Uuid_as_urn as fn(*const CValue, usize) -> CValue);
    runner.native_methods.insert("flame_uuid_Uuid_as_braced".to_string(), bridge_uuid::Uuid_as_braced as fn(*const CValue, usize) -> CValue);
    runner.native_methods.insert("flame_uuid_as_braced".to_string(), bridge_uuid::Uuid_as_braced as fn(*const CValue, usize) -> CValue);
    runner.native_methods.insert("flame_uuid_Uuid_new_v4".to_string(), bridge_uuid::Uuid_new_v4 as fn(*const CValue, usize) -> CValue);
    runner.native_methods.insert("flame_uuid_new_v4".to_string(), bridge_uuid::Uuid_new_v4 as fn(*const CValue, usize) -> CValue);
    runner.native_methods.insert("flame_uuid_Uuid_get_variant".to_string(), bridge_uuid::Uuid_get_variant as fn(*const CValue, usize) -> CValue);
    runner.native_methods.insert("flame_uuid_get_variant".to_string(), bridge_uuid::Uuid_get_variant as fn(*const CValue, usize) -> CValue);
    runner.native_methods.insert("flame_uuid_Uuid_get_version_num".to_string(), bridge_uuid::Uuid_get_version_num as fn(*const CValue, usize) -> CValue);
    runner.native_methods.insert("flame_uuid_get_version_num".to_string(), bridge_uuid::Uuid_get_version_num as fn(*const CValue, usize) -> CValue);
    runner.native_methods.insert("flame_uuid_Uuid_get_version".to_string(), bridge_uuid::Uuid_get_version as fn(*const CValue, usize) -> CValue);
    runner.native_methods.insert("flame_uuid_get_version".to_string(), bridge_uuid::Uuid_get_version as fn(*const CValue, usize) -> CValue);
    runner.native_methods.insert("flame_uuid_Uuid_as_u128".to_string(), bridge_uuid::Uuid_as_u128 as fn(*const CValue, usize) -> CValue);
    runner.native_methods.insert("flame_uuid_as_u128".to_string(), bridge_uuid::Uuid_as_u128 as fn(*const CValue, usize) -> CValue);
    runner.native_methods.insert("flame_uuid_Uuid_to_u128_le".to_string(), bridge_uuid::Uuid_to_u128_le as fn(*const CValue, usize) -> CValue);
    runner.native_methods.insert("flame_uuid_to_u128_le".to_string(), bridge_uuid::Uuid_to_u128_le as fn(*const CValue, usize) -> CValue);
    runner.native_methods.insert("flame_uuid_Uuid_as_bytes".to_string(), bridge_uuid::Uuid_as_bytes as fn(*const CValue, usize) -> CValue);
    runner.native_methods.insert("flame_uuid_as_bytes".to_string(), bridge_uuid::Uuid_as_bytes as fn(*const CValue, usize) -> CValue);
    runner.native_methods.insert("flame_uuid_Uuid_to_bytes_le".to_string(), bridge_uuid::Uuid_to_bytes_le as fn(*const CValue, usize) -> CValue);
    runner.native_methods.insert("flame_uuid_to_bytes_le".to_string(), bridge_uuid::Uuid_to_bytes_le as fn(*const CValue, usize) -> CValue);
    runner.native_methods.insert("flame_uuid_Uuid_is_nil".to_string(), bridge_uuid::Uuid_is_nil as fn(*const CValue, usize) -> CValue);
    runner.native_methods.insert("flame_uuid_is_nil".to_string(), bridge_uuid::Uuid_is_nil as fn(*const CValue, usize) -> CValue);
    runner.native_methods.insert("flame_uuid_Uuid_is_max".to_string(), bridge_uuid::Uuid_is_max as fn(*const CValue, usize) -> CValue);
    runner.native_methods.insert("flame_uuid_is_max".to_string(), bridge_uuid::Uuid_is_max as fn(*const CValue, usize) -> CValue);
    runner.native_methods.insert("flame_uuid_Uuid_encode_buffer".to_string(), bridge_uuid::Uuid_encode_buffer as fn(*const CValue, usize) -> CValue);
    runner.native_methods.insert("flame_uuid_encode_buffer".to_string(), bridge_uuid::Uuid_encode_buffer as fn(*const CValue, usize) -> CValue);
    runner.native_methods.insert("flame_uuid_Uuid_get_timestamp".to_string(), bridge_uuid::Uuid_get_timestamp as fn(*const CValue, usize) -> CValue);
    runner.native_methods.insert("flame_uuid_get_timestamp".to_string(), bridge_uuid::Uuid_get_timestamp as fn(*const CValue, usize) -> CValue);
    runner.native_methods.insert("flame_uuid_Uuid_get_node_id".to_string(), bridge_uuid::Uuid_get_node_id as fn(*const CValue, usize) -> CValue);
    runner.native_methods.insert("flame_uuid_get_node_id".to_string(), bridge_uuid::Uuid_get_node_id as fn(*const CValue, usize) -> CValue);
    runner.native_methods.insert("flame_uuid_Builder_from_u128".to_string(), bridge_uuid::Builder_from_u128 as fn(*const CValue, usize) -> CValue);
    runner.native_methods.insert("flame_uuid_Builder_from_u128_le".to_string(), bridge_uuid::Builder_from_u128_le as fn(*const CValue, usize) -> CValue);
    runner.native_methods.insert("flame_uuid_Builder_nil".to_string(), bridge_uuid::Builder_nil as fn(*const CValue, usize) -> CValue);
    runner.native_methods.insert("flame_uuid_Builder_as_uuid".to_string(), bridge_uuid::Builder_as_uuid as fn(*const CValue, usize) -> CValue);
    runner.native_methods.insert("flame_uuid_Timestamp_from_gregorian_time".to_string(), bridge_uuid::Timestamp_from_gregorian_time as fn(*const CValue, usize) -> CValue);
    runner.native_methods.insert("flame_uuid_Timestamp_from_unix_time".to_string(), bridge_uuid::Timestamp_from_unix_time as fn(*const CValue, usize) -> CValue);
    runner.native_methods.insert("flame_rand_random".to_string(), bridge_rand::random as fn(*const CValue, usize) -> CValue);
    runner.native_methods.insert("flame_rand_random_ratio".to_string(), bridge_rand::random_ratio as fn(*const CValue, usize) -> CValue);
    runner.native_methods.insert("flame_bridge_rust_add".to_string(), bridge_bridge::rust_add as fn(*const CValue, usize) -> CValue);
    // Since execute_source does not exist, we just run_file from main.rs if we had it, but here we can just parse and run
    // Read the package's source at runtime from current working directory
    let src = std::fs::read_to_string("src/main.fm").unwrap_or_default();
    let mut lexer = flamelang::lexer::Lexer::new(&src);
    let mut tokens = Vec::new();
    loop {
        let tok = lexer.next_token();
        let is_eof = tok.kind == flamelang::lexer::TokenKind::EOF;
        tokens.push(tok);
        if is_eof { break; }
    }
    let mut parser = flamelang::parser::Parser::new(tokens, "src/main.fm".to_string());
    match parser.parse() {
        Ok(stmts) => {
            let result = runner.run(&stmts);
            vm::wait_for_all_threads();
            if let Err(e) = result {
                eprintln!("\x1b[1;31mRuntime error:\x1b[0m {}", e);
            }
        }
        Err(diag) => {
            eprintln!("Parse error: {}", diag.message);
        }
    }
}
