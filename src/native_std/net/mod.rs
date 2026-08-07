pub mod tcp;
pub mod udp;
#[cfg(feature = "http")]
pub mod http;
#[cfg(feature = "ws")]
pub mod ws;
#[cfg(feature = "mqtt")]
pub mod mqtt;
pub mod dns;
pub mod url;
pub mod interface;

use crate::vm::Value;
use std::collections::HashMap;

pub fn init(submodule: &str) -> HashMap<String, Value> {
    match submodule {
        "tcp" => tcp::init(),
        "udp" => udp::init(),
        #[cfg(feature = "http")]
        "http" => http::init(),
        #[cfg(feature = "ws")]
        "ws" => ws::init(),
        #[cfg(feature = "mqtt")]
        "mqtt" => mqtt::init(),
        "dns" => dns::init(),
        "url" => url::init(),
        "interface" => interface::init(),
        _ => HashMap::new(),
    }
}
