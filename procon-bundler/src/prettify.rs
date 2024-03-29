use {
    crate::{Crate, Module, Span, TAB, TAB_LENGTH},
    std::fmt::{Display, Formatter, Result, Write},
};

const OPEN: &str = "{";
const CLOSE: &str = "}";

pub fn format_crate_to_string(my_crate: Crate) -> String {
    format!("{}", CrateFormatter(&my_crate))
}

struct CrateFormatter<'a>(&'a Crate);

impl Display for CrateFormatter<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let undered = self.0.name.replace('-', "_");
        writeln!(f, "// {undered} {OPEN}{OPEN}{OPEN}")?;
        writeln!(
            f,
            "// https://ngtkana.github.io/ac-adapter-rs/{undered}/index.html",
        )?;
        writeln!(f)?; // to avoid the buggy behavior of rustfmt
        writeln!(f, "#[allow(dead_code)]")?;
        fmt_dfs(f, &self.0.name, &self.0.root, 0)?;
        write!(f, "// {CLOSE}{CLOSE}{CLOSE}")?;
        Ok(())
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
                .to_owned()
        })
        .unwrap_or_else(|| crate_name.replace('-', "_"));
    let indent = " ".repeat(indent_level * TAB_LENGTH);
    writeln!(w, "{}mod {} {{", &indent, &name)?;
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
    use crate::format_crate_to_string;
    use {
        super::{Crate, Module, Span},
        std::path::PathBuf,
    };

    #[test]
    fn test_single_module() {
        let w = Crate {
            name: "holy_crate".to_owned(),
            root: Module {
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
        let result = format_crate_to_string(w);
        let expected = concat!(
            concat!("// holy_crate {", "{{\n"),
            "// https://ngtkana.github.io/ac-adapter-rs/holy_crate/index.html\n",
            "\n",
            "#[allow(dead_code)]\n",
            "mod holy_crate {\n",
            "    1\n",
            "    2\n",
            "    3\n",
            "    4\n",
            "}\n",
            concat!("// }", "}}"),
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn test_deep_modules() {
        let w = Crate {
            name: "holy_crate".to_owned(),
            root: Module {
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
        let result = format_crate_to_string(w);
        let expected = concat!(
            concat!("// holy_crate {", "{{\n"),
            "// https://ngtkana.github.io/ac-adapter-rs/holy_crate/index.html\n",
            "\n",
            "#[allow(dead_code)]\n",
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
            concat!("// }", "}}"),
        );
        assert_eq!(result, expected);
    }
}
