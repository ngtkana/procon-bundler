mod parse_line;

use {
    super::Resolve,
    crate::ConfigToml,
    parse_line::parse_module_decl,
    std::{
        io::BufRead,
        mem::take,
        path::{Path, PathBuf},
    },
};

pub fn bundle_crate<R: Resolve>(resolver: R, config_toml: ConfigToml) -> Module {
    CrateBundler::new(resolver, config_toml).bundle_crate()
}

#[derive(Clone, Debug, Default, PartialEq)]
struct CrateBundler<R> {
    resolver: R,
    config_toml: ConfigToml,
}

impl<R: Resolve> CrateBundler<R> {
    fn new(resolver: R, config_toml: ConfigToml) -> Self {
        Self {
            resolver,
            config_toml,
        }
    }
    fn bundle_crate(&mut self) -> Module {
        let reader = self.resolver.resolve(Path::new("."));
        self.bundle_module(reader, PathBuf::from("."))
    }
    fn bundle_module(&mut self, reader: impl BufRead, mut current_module_path: PathBuf) -> Module {
        let mut spans = Vec::new();
        let mut lines = Vec::new();
        for line in reader.lines().map(|line| {
            line.unwrap_or_else(|e| {
                panic!(
                    "Couldn't read a new line because of an IO error. e = {:?}",
                    e
                )
            })
        }) {
            if let Some(name) = parse_module_decl(&line) {
                spans.push(Span::RawLines(take(&mut lines)));
                current_module_path.push(name);
                spans.push(Span::Module(Box::new(self.bundle_module(
                    self.resolver.resolve(&current_module_path),
                    current_module_path.clone(),
                ))));
                current_module_path.pop();
            } else {
                lines.push(line);
            }
        }
        if !lines.is_empty() {
            spans.push(Span::RawLines(lines));
        }
        Module {
            spans,
            path: current_module_path,
        }
    }
}

#[derive(Clone, Debug, Default, Hash, PartialEq)]
pub struct Module {
    path: PathBuf,
    spans: Vec<Span>,
}
#[derive(Clone, Debug, Hash, PartialEq)]
pub enum Span {
    RawLines(Vec<String>),
    Module(Box<Module>),
}

#[cfg(test)]
mod tests {
    use {
        super::{bundle_crate, Module, Span},
        crate::{manual_resolver, ConfigToml},
        std::path::PathBuf,
    };

    #[test]
    fn test_bundle_single_module() {
        manual_resolver! {
            struct ManualResolver {
                "." => concat!(
                    "hi,\n",
                    "hello!\n",
                ),
            }
        }
        let result = bundle_crate(ManualResolver {}, ConfigToml::new(""));
        let expected = Module {
            path: PathBuf::from("."),
            spans: vec![Span::RawLines(vec!["hi,".to_owned(), "hello!".to_owned()])],
        };
        assert_eq!(result, expected);
    }

    #[test]
    fn test_bundle_one_child_module() {
        manual_resolver! {
            struct ManualResolver {
                "." => concat!(
                    "hi,\n",
                    "mod a;\n",
                    "hello!\n",
                ),
                "./a" => concat!(
                    "a also says: hi,\n",
                    "a also says: hello!\n",
                ),
            }
        }
        let result = bundle_crate(ManualResolver {}, ConfigToml::new(""));
        let expected = Module {
            path: PathBuf::from("."),
            spans: vec![
                Span::RawLines(vec!["hi,".to_owned()]),
                Span::Module(Box::new(Module {
                    path: PathBuf::from("./a"),
                    spans: vec![Span::RawLines(vec![
                        "a also says: hi,".to_owned(),
                        "a also says: hello!".to_owned(),
                    ])],
                })),
                Span::RawLines(vec!["hello!".to_owned()]),
            ],
        };
        assert_eq!(result, expected);
    }
}
