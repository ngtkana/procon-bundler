/*!
inner doc-comment block です。
これも消えます。
*/
#[macro_export]
macro_rules! my_macro2 {
    () => {
        /// ルートモジュール以外においてあっても大丈夫です。
        #[allow(unused_imports)]
        use $crate::A;
    };
}

#[allow(dead_code)]
enum B {}

/**
インラインモジュールはそのままです。
*/
mod b {
    #[allow(unused_imports)]
    /// `crate` は `crate::procon_bundler_sample` などに置換されます。
    use crate::A;
    /// インラインモジュール内のファイルモジュールも展開されます。
    mod c;
}

#[allow(dead_code)]
fn g() {
    my_macro2! {}
}
