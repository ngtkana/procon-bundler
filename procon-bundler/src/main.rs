mod bundle_crate;
mod config_toml;
mod error;
mod parse_line;
mod prettify;
mod resolver;
mod types;

pub use {
    bundle_crate::bundle_crate,
    config_toml::ConfigToml,
    error::{BundlerError, Result},
    prettify::format_crate_to_string,
    resolver::{CrateResolver, Resolve},
    std::path::{Path, PathBuf},
    types::{Crate, Module, Span},
};

use clap::{Parser, Subcommand};
use std::fs;

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
            fn resolve(&self, module_path: &::std::path::Path) -> $crate::Result<Self::B> {
                let path_str = module_path.to_str().ok_or_else(|| {
                    $crate::BundlerError::InvalidPathConversion {
                        path: module_path.to_path_buf(),
                    }
                })?;
                let content = match path_str {
                    $(
                        $module_path => $content,
                    )*
                    _ => return Err($crate::BundlerError::ModuleFileNotFound {
                        module_path: module_path.to_path_buf(),
                        file_path: module_path.to_path_buf(),
                        source: ::std::io::Error::new(
                            ::std::io::ErrorKind::NotFound,
                            format!("Mock resolver: path not found: {:?}", module_path)
                        ),
                    }),
                };
                Ok(::std::io::BufReader::new(content.as_bytes()))
            }
        }
    };
}

#[derive(Parser)]
#[command(name = "procon-bundler")]
#[command(version = "0.2.0")]
#[command(author = "Nagata Kana <ngtkana@gmail.com>")]
#[command(about = "Bundles a crate into a depth-1 module")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Bundle a crate
    Bundle {
        /// The path to the root of a crate to bundle
        crate_root: PathBuf,
    },
    /// Find and bundle a desired crate in a workspace
    Find {
        /// The path to the root of a workspace to search
        workspace_root: PathBuf,
        /// The name of a crate to bundle (either chain-case or snake_case is okay)
        crate_name: String,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match run(cli) {
        Ok(output) => output,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    println!("{}", result);
}

fn run(cli: Cli) -> Result<String> {
    let crate_root = match cli.command {
        Commands::Find {
            workspace_root,
            crate_name,
        } => workspace_root.join("libs").join(crate_name),
        Commands::Bundle { crate_root } => crate_root,
    };

    bundle_to_string(&crate_root)
}

fn bundle_to_string(path: &Path) -> Result<String> {
    let name = path
        .file_stem()
        .ok_or_else(|| BundlerError::InvalidFileStem {
            path: path.to_path_buf(),
        })?;
    let name = name
        .to_str()
        .ok_or_else(|| BundlerError::InvalidPathConversion {
            path: path.to_path_buf(),
        })?;
    
    let resolver = CrateResolver::new(path.to_path_buf());
    let config_path = path.join("Cargo.toml");
    
    let buf = fs::read_to_string(&config_path).map_err(|e| {
        BundlerError::CargoTomlReadError {
            path: config_path,
            source: e,
        }
    })?;
    
    let config = ConfigToml::new(&buf)?;
    let my_crate = bundle_crate(name, resolver, config)?;
    Ok(format_crate_to_string(my_crate))
}

#[cfg(test)]
mod tests {
    use {super::bundle_to_string, difference::assert_diff, std::path::Path};

    #[test]
    fn test_bundle_by_crate_path() {
        let result = bundle_to_string(Path::new("../procon-bundler-sample")).unwrap();
        let expected = include_str!("../../procon-bundler-sample-result/src/lib.rs");
        let result = result.as_ref();
        let expected = expected[..expected.len() - 1].as_ref();
        assert_diff!(result, expected, "\n", 0);
    }
}
