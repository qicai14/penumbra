//! Pure maths for Penumbra: no I/O, builds for native and wasm32.

pub mod band;
pub mod dem;
pub mod eph;
pub mod geometry;
pub mod horizon;
pub mod pit;
pub mod power;
pub mod site;
pub mod timeline;

/// Version of the core → WASM API (ARCHITECTURE §3.1). The manifest and the test fixture (web/tests/fixture) carry the same number.
pub const API_VERSION: u32 = 4;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_version_is_frozen_at_four() {
        assert_eq!(API_VERSION, 4);
    }
}
