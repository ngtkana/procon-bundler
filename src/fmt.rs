use {
    crate::{Module, Span, TAB, TAB_LENGTH},
    std::{
        fmt::{Debug, Formatter, Result, Write},
        iter::repeat,
    },
};

#[derive(Clone, Hash, PartialEq, Copy)]
pub struct CrateWriter<'a> {
    name: &'a str,
    root: &'a Module,
}
impl<'a> Debug for CrateWriter<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        fmt_dfs(f, self.name, self.root, 0)
    }
}

pub fn fmt_dfs(
    w: &mut impl Write,
    crate_name: &str,
    module: &Module,
    indent_level: usize,
) -> Result {
    let name = module
        .path
        .as_path()
        .file_stem()
        .map(|s| {
            s.to_str()
                .unwrap_or_else(|| panic!("OsStr から str に変換できませんでした。"))
        })
        .unwrap_or(crate_name);
    let indent = repeat(' ')
        .take(indent_level * TAB_LENGTH)
        .collect::<String>();
    writeln!(w, "{}mod {} {{", &indent, name)?;
    for span in &module.spans {
        match span {
            Span::Lines(lines) => {
                for line in lines {
                    writeln!(w, "{}{}{}", &indent, &TAB, line)?;
                }
            }
            Span::Module(module) => {
                if !module.is_test {
                    fmt_dfs(w, crate_name, module, indent_level + 1)?
                }
            }
        }
    }
    writeln!(w, "{}}}", &indent)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use {
        super::{CrateWriter, Module, Span},
        std::path::PathBuf,
    };

    #[test]
    fn test_single_module() {
        let w = CrateWriter {
            name: "holy_crate",
            root: &Module {
                is_test: false,
                path: PathBuf::from("."),
                spans: vec![Span::Lines(vec![
                    "1".to_owned(),
                    "2".to_owned(),
                    "3".to_owned(),
                    "4".to_owned(),
                ])],
            },
        };
        let result = format!("{:?}", w);
        let expected = concat!(
            "mod holy_crate {\n",
            "    1\n",
            "    2\n",
            "    3\n",
            "    4\n",
            "}\n",
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn test_deep_modules() {
        let w = CrateWriter {
            name: "holy_crate",
            root: &Module {
                is_test: false,
                path: PathBuf::from("."),
                spans: vec![
                    Span::Lines(vec!["start root".to_owned()]),
                    Span::Module(Box::new(Module {
                        is_test: false,
                        path: PathBuf::from("./a"),
                        spans: vec![
                            Span::Lines(vec!["start a".to_owned()]),
                            Span::Module(Box::new(Module {
                                is_test: false,
                                path: PathBuf::from("./a/b"),
                                spans: vec![Span::Lines(vec!["in b".to_owned()])],
                            })),
                            Span::Lines(vec!["end a".to_owned()]),
                        ],
                    })),
                    Span::Lines(vec!["end root".to_owned()]),
                ],
            },
        };
        let result = format!("{:?}", w);
        let expected = concat!(
            "mod holy_crate {\n",
            "    start root\n",
            "    mod a {\n",
            "        start a\n",
            "        mod b {\n",
            "            in b\n",
            "        }\n",
            "        end a\n",
            "    }\n",
            "    end root\n",
            "}\n",
        );
        assert_eq!(result, expected);
    }
}
