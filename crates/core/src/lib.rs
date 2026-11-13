//! Pure maths for Penumbra: no I/O, builds for native and wasm32.

/// Placeholder until the event starts (pre-event rule: no lunar maths yet).
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
}
