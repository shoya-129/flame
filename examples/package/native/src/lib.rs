pub fn rust_add(a: i64, b: i64) -> i64 {
    a + b
}

pub fn rust_is_positive(value: i64) -> bool {
    value > 0
}

pub fn rust_bool_to_string(value: bool) -> String {
    if value {
        "true".to_string()
    } else {
        "false".to_string()
    }
}

pub fn rust_string_length(value: String) -> i64 {
    value.len() as i64
}

pub fn rust_double(value: f64) -> f64 {
    value * 2.0
}

pub fn rust_bytes_sum(value: Vec<u8>) -> i64 {
    value.iter().map(|v| *v as i64).sum()
}

pub fn rust_maybe_positive(value: i64) -> Option<i64> {
    if value > 0 {
        Some(value)
    } else {
        None
    }
}

pub fn rust_checked_add(a: i64, b: i64) -> Result<i64, String> {
    a.checked_add(b)
        .ok_or_else(|| "integer overflow".to_string())
}

pub struct Counter {
    pub value: i64,
}

impl Counter {
    pub fn new(value: i64) -> Counter {
        Counter { value }
    }

    pub fn increment(&mut self) {
        self.value += 1;
    }

    pub fn get(&self) -> i64 {
        self.value
    }
}