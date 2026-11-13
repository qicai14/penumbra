//! wasm-bindgen wrapper over penumbra-core. Logic stays in core.

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn add(a: i32, b: i32) -> i32 {
    penumbra_core::add(a, b)
}
