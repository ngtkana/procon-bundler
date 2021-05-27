mod config_toml;
mod resolver;

pub use {
    config_toml::ConfigToml,
    resolver::{FsResolver, Resolve},
};

#[allow(unused_macros)]
#[macro_export]
macro_rules! manual_resolver {
    (enum $resolver_name:ident {
        $($path:expr => $content:expr),* $(,)?
    }) => {
        enum $resolver_name {}
        impl $crate::resolver::Resolve for $resolver_name {
            type B = ::std::io::BufReader<&'static [u8]>;
            fn resolve(path: &str) -> Self::B {
                ::std::io::BufReader::new(
                    match path {
                        $(
                            $path => $content,
                        )*
                        _ => unreachable!(),
                    }
                    .as_bytes(),
                )
            }
        }
    };
}

fn main() {}
