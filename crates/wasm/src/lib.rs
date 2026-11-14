//! wasm-bindgen wrapper over penumbra-core. Logic stays in core.

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn add(a: i32, b: i32) -> i32 {
    penumbra_core::add(a, b)
}

/// Frozen API version (ARCHITECTURE §3.1); the adapter refuses a fixture or manifest that differs.
#[wasm_bindgen]
pub fn api_version() -> u32 {
    penumbra_core::API_VERSION
}
