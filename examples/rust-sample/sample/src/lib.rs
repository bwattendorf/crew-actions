//! Tiny library used only to exercise the rust-gate workflow.

/// Adds two numbers without overflowing.
pub fn add(a: u32, b: u32) -> Option<u32> {
    a.checked_add(b)
}

#[cfg(test)]
mod tests {
    use super::add;

    #[test]
    fn adds() {
        assert_eq!(add(2, 3), Some(5));
    }

    #[test]
    fn overflow_is_none() {
        assert_eq!(add(u32::MAX, 1), None);
    }
}
