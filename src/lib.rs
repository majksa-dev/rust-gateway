//! Example of a simple library
//!
//! # Examples
//!
//! ```
//! let result = add(2, 2);
//! assert_eq!(result, 4);
//! ```

pub mod http;

/// Adds two numbers
/// ```
/// let result = add(2, 2);
/// assert_eq!(result, 4);
/// ```
pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        essentials::install();
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
