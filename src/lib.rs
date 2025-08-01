pub fn add(left: usize, right: usize) -> usize {
    left + right
}
pub mod io;
pub mod finit_diff;
pub mod field;
pub mod system;



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
