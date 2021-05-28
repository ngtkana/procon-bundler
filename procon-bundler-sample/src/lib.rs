//! outer doc-comments です。
//!
//! これに限らず、doc-comments 系は消去されます。
//!
//! てことでこれで説明書きますね。（はい？）
//!
//!
mod a;
pub mod small_module;

/// inner doc-comments です。
/// これも消えます。
#[macro_export]
macro_rules! my_macro {
    () => {
        /// 中途半端なところに書いても消えますよ。
        #[allow(unused_imports)]
        /// `$crate` は、`$crate::procon_bundler_sample` などに置換されます。
        use $crate::A;
    };
}

/**
outer doc-comment block です。
*/
#[allow(dead_code)]
enum A {}

/**
#[cfg(test)] のついたアイテムは、消えていただきたいところですが、
アイテムをパースするのが大変なので現状残してあります。
*/
#[cfg(test)]
#[allow(dead_code)]
enum OnlyForTest {}

#[allow(dead_code)]
fn f() {
    my_macro! {}
}

/// #[cfg(test)] のついたモジュールは消えます。
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
