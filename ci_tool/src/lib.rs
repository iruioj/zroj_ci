pub mod gitutils;
pub mod public_api;
pub mod commit_parse;

pub fn sum(a: i32, b: i32) -> i32 {
    a + b
}

pub fn sum_generic<T: std::ops::Add<Output = T>>(a: T, b: T) -> T {
    a + b
}

fn private_noop() {}

pub fn get_vec() -> Vec<i32> {
    private_noop();
    Vec::new()
}
