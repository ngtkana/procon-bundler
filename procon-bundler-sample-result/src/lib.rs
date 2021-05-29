// procon_bundler_sample {{{
#[allow(dead_code)]
mod procon_bundler_sample {
    mod a {
        #[macro_export]
        macro_rules! my_macro2 {
            () => {
                #[allow(unused_imports)]
                use $crate::procon_bundler_sample::A;
            };
        }
        #[allow(dead_code)]
        enum B {}
        mod b {
            #[allow(unused_imports)]
            use crate::procon_bundler_sample::A;
            mod c {
                #[allow(dead_code)]
                type C = ();
            }
        }
        #[allow(dead_code)]
        fn g() {
            my_macro2! {}
        }
    }
    mod small_module {
        #[allow(dead_code)]
        pub type A = ();
    }
    #[macro_export]
    macro_rules! my_macro {
        () => {
            #[allow(unused_imports)]
            use $crate::procon_bundler_sample::A;
        };
    }
    #[allow(dead_code)]
    enum A {}
    #[cfg(test)]
    #[allow(dead_code)]
    enum OnlyForTest {}
    #[allow(dead_code)]
    fn f() {
        my_macro! {}
    }
}
// }}}
