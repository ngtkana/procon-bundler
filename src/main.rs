mod config_toml;
mod resolver;

pub use {
    config_toml::ConfigToml,
    resolver::{CrateResolver, Resolve},
};

#[allow(unused_macros)]
#[macro_export]
macro_rules! manual_resolver {
    (struct $resolver_name:ident {
        $($module_path:expr => $content:expr),* $(,)?
    }) => {
        struct $resolver_name {}
        impl $crate::resolver::Resolve for $resolver_name {
            type B = ::std::io::BufReader<&'static [u8]>;
            fn resolve(&self, module_path: &::std::path::Path) -> Self::B {
                ::std::io::BufReader::new(
                    match module_path.to_str().unwrap() {
                        $(
                            $module_path => $content,
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
