pub fn add(left: usize, right: usize) -> usize {
    left + right
}
pub mod field;
pub mod finit_diff;
pub mod io;
pub mod operators;
pub mod params;
pub mod system;

pub use pdifflib_derive::{LoggingSchema, ParameterSchema};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
