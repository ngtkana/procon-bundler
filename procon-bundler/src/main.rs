mod bundle_crate;
mod config_toml;
mod fmt;
mod parse_line;
mod resolver;
mod types;

pub use {
    bundle_crate::bundle_crate,
    clap::{load_yaml, App},
    config_toml::ConfigToml,
    fmt::format_crate_to_string,
    resolver::{CrateResolver, Resolve},
    std::{
        fs::File,
        io::Read,
        path::{Path, PathBuf},
    },
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

fn main() {
    let yaml = clap::load_yaml!("cli.yml");
    let app = App::from_yaml(yaml);
    let mut orig_app = app.clone();
    let matches = app.get_matches();

    let crate_root = match matches.subcommand() {
        ("find", Some(matches)) => Path::new(matches.value_of("WORKSPACE_ROOT").unwrap())
            .join("libs")
            .join(matches.value_of("CRATE_NAME").unwrap()),
        ("bundle", Some(matches)) => PathBuf::from(matches.value_of("CRATE_ROOT").unwrap()),
        _ => {
            orig_app
                .print_help()
                .expect("ヘルプを印刷できませんでした。");
            panic!();
        }
    };

    let result = bundle_to_string(crate_root.as_path());
    println!("{}", result);
}

fn bundle_to_string(path: &Path) -> String {
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
    let my_crate = bundle_crate(name, resolver, config);
    format_crate_to_string(my_crate)
}

#[cfg(test)]
mod tests {
    use {super::bundle_to_string, difference::assert_diff, std::path::Path};

    #[test]
    fn test_bundle_by_crate_path() {
        let result = bundle_to_string(Path::new("../procon-bundler-sample"));
        let expected = include_str!("../../procon-bundler-sample-result/src/lib.rs");
        let result = result.as_ref();
        let expected = expected[..expected.len() - 1].as_ref();
        assert_diff!(result, expected, "\n", 0);
    }
}
