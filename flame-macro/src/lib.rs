use proc_macro::TokenStream;
use syn::{parse_macro_input, Item};
use quote::quote;

/// Attribute macro for Flame native plugin items.
///
/// Supported options:
/// - `#[flame(daemon)]` or `#[flame(runtime)]`: Marks a method (like a server or listener) as a long-running daemon.
/// - `#[flame(constructor)]`: Marks an associated function or method as a struct constructor in Flame.
/// - `#[flame(skip)]`: Ignores this item from being exported into the Flame runtime / `.fmi` interface.
/// - `#[flame(rename = "custom_name")]`: Exposes this item to Flame under a custom name.
#[proc_macro_attribute]
pub fn flame(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Preserves the item directly for Rust compilation while allowing metadata extraction by Flame's toolchain
    item
}

/// Attribute macro to explicitly export a Rust item into the Flame interface.
#[proc_macro_attribute]
pub fn flame_export(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
