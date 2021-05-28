use std::{fs::File, io::Read};

mod bundle_crate;
mod config_toml;
mod fmt;
mod resolver;
mod types;

pub use {
    bundle_crate::bundle_crate,
    config_toml::ConfigToml,
    fmt::format_crate_to_string,
    resolver::{CrateResolver, Resolve},
    std::path::Path,
    types::{Crate, Module, Span},
};

const TAB: &str = "    ";
const TAB_LENGTH: usize = TAB.len();

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
                        _ => panic!("Received an illegal module path `{:?}`", module_path),
                    }
                    .as_bytes(),
                )
            }
        }
    };
}

pub fn bundle_by_crate_path(path: &Path) -> Crate {
    let name = path
        .file_stem()
        .unwrap_or_else(|| panic!("filestem がありません。 path = {:?}", path));
    let name = name
        .to_str()
        .unwrap_or_else(|| panic!("OsStr -> str の変換ができません。 path = {:?}", path));
    let resolver = CrateResolver::new(path.to_path_buf());
    let config_path = path.join("Cargo.toml");
    let mut buf = String::new();
    File::open(config_path.as_path())
        .unwrap_or_else(|_| {
            panic!(
                "Cargo.toml が見つかりません。config_path = {:?}",
                config_path
            )
        })
        .read_to_string(&mut buf)
        .unwrap_or_else(|_| panic!("Cargo.toml の中身がよめません。"));
    let config = ConfigToml::new(&buf);
    bundle_crate(name, resolver, config)
}

fn main() {}

#[cfg(test)]
mod tests {
    use {
        super::{bundle_by_crate_path, format_crate_to_string},
        difference::assert_diff,
        std::{fs::File, io::Read as _, path::Path},
    };

    #[test]
    fn test_bundle_by_crate_path() {
        let result =
            format_crate_to_string(bundle_by_crate_path(Path::new("../procon-bundler-sample")));
        let mut expected = String::new();
        File::open("../procon-bundler-sample-result/src/lib.rs")
            .unwrap()
            .read_to_string(&mut expected)
            .unwrap();
        let result = result.as_ref();
        let expected = expected.as_ref();
        assert_diff!(result, expected, "\n", 0);
    }
}
