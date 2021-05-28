#[macro_export]
macro_rules! my_macro2 {
    () => {
        #[allow(unused_imports)]
        use $crate::A;
    };
}

#[allow(dead_code)]
enum B {}

mod b {
    #[allow(unused_imports)]
    use crate::A;
}

#[allow(dead_code)]
fn g() {
    my_macro2! {}
}
