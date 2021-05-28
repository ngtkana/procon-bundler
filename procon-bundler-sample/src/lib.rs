mod a;
pub mod small_module;

#[macro_export]
macro_rules! my_macro {
    () => {
        #[allow(unused_imports)]
        use $crate::A;
    };
}

#[allow(dead_code)]
enum A {}

#[allow(dead_code)]
fn f() {
    my_macro! {}
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
