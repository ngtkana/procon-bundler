use std::{fs::File, io::Read};

mod bundle_crate;
mod config_toml;
mod fmt;
mod resolver;
mod types;

pub use {
    bundle_crate::bundle_crate,
    config_toml::ConfigToml,
    fmt::CrateWriter,
    resolver::{CrateResolver, Resolve},
    std::path::Path,
    types::{Module, Span},
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

pub fn bundle_by_crate_path(path: &Path) -> Module {
    let cann = path
        .canonicalize()
        .unwrap_or_else(|_| panic!("絶対パスを取得できません。 path = {:?}", path));
    let name = cann
        .as_path()
        .file_stem()
        .unwrap_or_else(|| panic!("filestem がありません。 path = {:?}", path));
    let name = name
        .to_str()
        .unwrap_or_else(|| panic!("OsStr -> str の変換ができません。 path = {:?}", path));
    let resolver = CrateResolver::new(path.to_path_buf());
    let config_path = path.join("Config.toml");
    let mut buf = String::new();
    File::open(config_path.as_path())
        .unwrap_or_else(|_| {
            panic!(
                "Config.toml が見つかりません。config_toml_path = {:?}",
                config_path
            )
        })
        .read_to_string(&mut buf)
        .unwrap_or_else(|_| panic!("Config.toml の中身がよめません。"));
    let config = ConfigToml::new(&buf);
    bundle_crate(name, resolver, config)
}

fn main() {}
