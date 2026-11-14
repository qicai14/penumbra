//! Pure maths for Penumbra: no I/O, builds for native and wasm32.

pub mod dem;
pub mod eph;
pub mod geometry;
pub mod horizon;
pub mod timeline;

/// Version of the core → WASM API (ARCHITECTURE §3.1). The fixture and the manifest carry the same number.
pub const API_VERSION: u32 = 1;

/// Placeholder kept for the template page until the first science ticket lands.
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_two_and_three() {
        assert_eq!(add(2, 3), 5);
    }

    #[test]
    fn api_version_is_frozen_at_one() {
        assert_eq!(API_VERSION, 1);
    }
}
