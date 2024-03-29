use crate::parse_line;
use std::mem::take;

use {
    crate::{ConfigToml, Crate, Module, Resolve, Span, TAB_LENGTH},
    parse_line::{
        parse_block_doc_comments_end, parse_block_doc_comments_start, parse_block_end,
        parse_cfg_test, parse_module_block_begin, parse_module_decl, parse_oneline_doc_comments,
        remove_indentation, substitute_path,
    },
    std::{
        io::BufRead,
        path::{Path, PathBuf},
    },
};

pub fn bundle_crate<R: Resolve>(crate_name: &str, resolver: R, config_toml: ConfigToml) -> Crate {
    CrateBundler::new(crate_name, resolver, config_toml).bundle_crate()
}

#[derive(Clone, Debug, Default, PartialEq)]
struct CrateBundler<'a, R> {
    crate_name: &'a str,
    resolver: R,
    config_toml: ConfigToml,
}

impl<'a, R: Resolve> CrateBundler<'a, R> {
    fn new(crate_name: &'a str, resolver: R, config_toml: ConfigToml) -> Self {
        Self {
            crate_name,
            resolver,
            config_toml,
        }
    }
    fn bundle_crate(&mut self) -> Crate {
        let reader = self.resolver.resolve(Path::new("."));
        Crate {
            name: self.crate_name.to_owned(),
            root: self.bundle_module(reader, PathBuf::from(".")),
        }
    }
    fn bundle_module(&mut self, reader: impl BufRead, mut current_module_path: PathBuf) -> Module {
        fn push_line_to_stack<R>(me: &CrateBundler<R>, stack: &mut [Module], line: &str) {
            let stack_len = stack.len();
            let spans = &mut stack.last_mut().unwrap().spans;
            if !matches!(spans.last(), Some(Span::Lines(_))) {
                spans.push(Span::Lines(Vec::new()));
            }
            match spans.last_mut().unwrap() {
                Span::Lines(ref mut lines) => {
                    lines.push(remove_indentation(
                        substitute_path(line, me.crate_name, &me.config_toml).as_ref(),
                        stack_len - 1,
                    ));
                }
                Span::Module(_) => unreachable!(),
            }
        }
        // 結果がモジュール別に格納されるスタック
        // （関数終了時には、要素数 1 になっているべきです。）
        let mut stack = vec![Module::new(current_module_path.clone())];
        // 未解決 #[cfg(test)] フラグ
        let mut unresolved_cfg_test = None::<UnresolvedCfgTest>;
        // doc comments 内フラグ
        let mut in_doc_comments = false;

        for line in reader.lines().map(|line| {
            line.unwrap_or_else(|e| {
                panic!(
                    "Couldn't read a new line because of an IO error. e = {:?}",
                    e
                )
            })
        }) {
            // この行を使う必要があるときに立てるフラグ
            let mut needs_current_line = false;

            // 正規表現によるパースをトライ
            //
            // どの `Case \d` にも合致しないときには、
            // `needs_current_line` フラグが立つので、
            // 直後に回収します。
            if in_doc_comments {
                // Case 1: ブロック doc comments の終了
                // NOTE: `*/` は通常のブロックコメントの終了にも使われるので、
                // フラグをチェックしています。
                if parse_block_doc_comments_end(&line) {
                    in_doc_comments = false;
                }
            } else if let Some(name) = parse_module_decl(&line) {
                // Case 2: モジュール宣言
                //
                // * モジュールパスを変更して再帰呼出し
                // * モジュールパスを戻す
                // * テストフラグが立っていればモジュールに反映
                //
                current_module_path.push(name);
                let mut module = self.bundle_module(
                    self.resolver.resolve(&current_module_path),
                    current_module_path.clone(),
                );
                module.is_test = take(&mut unresolved_cfg_test).is_some();
                stack
                    .last_mut()
                    .unwrap()
                    .spans
                    .push(Span::Module(Box::new(module)));
                current_module_path.pop();
            } else if let Some(name) = parse_module_block_begin(&line) {
                // Case 3: インラインモジュールの開始
                //
                // * モジュールパスを変更
                // * スタックに新しいモジュールを積む
                // * テストフラグが立っていればモジュールに反映
                //
                current_module_path.push(name);
                let mut module = Module::new(current_module_path.clone());
                module.is_test = take(&mut unresolved_cfg_test).is_some();
                stack.push(module);
            } else if let Some(space_count) = parse_block_end(&line) {
                if 2 <= stack.len() && space_count == (stack.len() - 2) * TAB_LENGTH {
                    // Case 4: インラインモジュールの終了
                    //
                    // * 終了したモジュールをスタックから取り出してスカッシュ
                    // * モジュールパスを戻す
                    //
                    current_module_path.pop();
                    let module = stack.pop().unwrap();
                    stack
                        .last_mut()
                        .unwrap()
                        .spans
                        .push(Span::Module(Box::new(module)));
                } else {
                    needs_current_line = true;
                }
            } else {
                // 前回ループの Case 5 で見た #[cfg(test)] が、この時点で解決していないならば、
                // モジュールではなかったので、遅ればせながらプッシュします。
                if let Some(UnresolvedCfgTest::Unknown(cfg_test)) = unresolved_cfg_test {
                    unresolved_cfg_test = Some(UnresolvedCfgTest::Module);
                    push_line_to_stack(self, &mut stack, &cfg_test);
                }
                if parse_cfg_test(&line) {
                    // Case 5: #[cfg(test)]
                    unresolved_cfg_test = Some(UnresolvedCfgTest::Unknown(line.to_owned()));
                } else if parse_block_doc_comments_start(&line) {
                    // Case 6: ブロック doc comments の開始
                    assert!(!in_doc_comments);
                    in_doc_comments = true;
                } else if parse_oneline_doc_comments(&line) || line.is_empty() {
                    // Case 7: oneline doc comments or 空行
                    //
                    // * なにもしません
                } else {
                    needs_current_line = true;
                }
            }

            // 「この行を使う必要があるときに立てるフラグ」回収です。
            if needs_current_line {
                push_line_to_stack(self, &mut stack, &line);
            }
        }
        let res = stack.pop().unwrap();
        assert!(stack.is_empty());
        res
    }
}

#[derive(Clone, Debug, Hash, PartialEq)]
enum UnresolvedCfgTest {
    Unknown(String), // まだ mod の次がモジュールかどうかわからない状態
    Module,          // モジュールと確定した状態
}

#[cfg(test)]
mod tests {
    use {
        super::{bundle_crate, Crate, Module, Span},
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
        let result = bundle_crate("my_crate", ManualResolver {}, ConfigToml::new(""));
        let expected = Crate {
            name: "my_crate".to_owned(),
            root: Module {
                is_test: false,
                path: PathBuf::from("."),
                spans: vec![Span::Lines(vec!["hi,".to_owned(), "hello!".to_owned()])],
            },
        };
        assert_eq!(result, expected);
    }

    #[test]
    fn test_skip_doc_comments_and_empty_line() {
        manual_resolver! {
            struct ManualResolver {
                "." => concat!(
                    "/*!\n",
                    "    comments\n",
                    "    comments\n",
                    "*/\n",
                    "start\n",
                    "\n",
                    "/* block comments */\n", // 普通のコメントは消えません。
                    "// line comments\n",     // 普通のコメントは消えません。
                    "start\n",
                    "/// doc comments\n",
                    "    /// doc comments with inentation\n",
                    "/// doc comments\n",
                    "\n",
                    "contents\n",
                    "/**\n",
                    "    comments\n",
                    "    comments\n",
                    "*/\n",
                    "contents\n",
                    "end\n",
                    "end\n",
                    "\n",
                ),
            }
        }
        let result = bundle_crate("my_crate", ManualResolver {}, ConfigToml::new(""));
        let expected = Crate {
            name: "my_crate".to_owned(),
            root: Module {
                is_test: false,
                path: PathBuf::from("."),
                spans: vec![Span::Lines(vec![
                    "start".to_owned(),
                    "/* block comments */".to_owned(),
                    "// line comments".to_owned(),
                    "start".to_owned(),
                    "contents".to_owned(),
                    "contents".to_owned(),
                    "end".to_owned(),
                    "end".to_owned(),
                ])],
            },
        };
        assert_eq!(result, expected);
    }

    #[test]
    fn test_simple_external_module() {
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
        let result = bundle_crate("my_crate", ManualResolver {}, ConfigToml::new(""));
        let expected = Crate {
            name: "my_crate".to_owned(),
            root: Module {
                is_test: false,
                path: PathBuf::from("."),
                spans: vec![
                    Span::Lines(vec!["hi,".to_owned()]),
                    Span::Module(Box::new(Module {
                        is_test: false,
                        path: PathBuf::from("./a"),
                        spans: vec![Span::Lines(vec![
                            "a also says: hi,".to_owned(),
                            "a also says: hello!".to_owned(),
                        ])],
                    })),
                    Span::Lines(vec!["hello!".to_owned()]),
                ],
            },
        };
        assert_eq!(result, expected);
    }

    #[test]
    fn test_test_external_module() {
        manual_resolver! {
            struct ManualResolver {
                "." => concat!(
                    "hi,\n",
                    "#[cfg(test)]\n",
                    "mod a;\n",
                    "hello!\n",
                ),
                "./a" => concat!(
                    "a also says: hi,\n",
                    "a also says: hello!\n",
                ),
            }
        }
        let result = bundle_crate("my_crate", ManualResolver {}, ConfigToml::new(""));
        let expected = Crate {
            name: "my_crate".to_owned(),
            root: Module {
                is_test: false,
                path: PathBuf::from("."),
                spans: vec![
                    Span::Lines(vec!["hi,".to_owned()]),
                    Span::Module(Box::new(Module {
                        is_test: true,
                        path: PathBuf::from("./a"),
                        spans: vec![Span::Lines(vec![
                            "a also says: hi,".to_owned(),
                            "a also says: hello!".to_owned(),
                        ])],
                    })),
                    Span::Lines(vec!["hello!".to_owned()]),
                ],
            },
        };
        assert_eq!(result, expected);
    }

    #[test]
    fn test_inline_module() {
        manual_resolver! {
            struct ManualResolver {
                "." => concat!(
                    "hi,\n",
                    "mod a {\n",
                    "    hey\n",
                    "   shallow\n",
                    "     deep\n",
                    "}\n",
                    "hello!\n",
                ),
            }
        }
        let result = bundle_crate("my_crate", ManualResolver {}, ConfigToml::new(""));
        let expected = Crate {
            name: "my_crate".to_owned(),
            root: Module {
                is_test: false,
                path: PathBuf::from("."),
                spans: vec![
                    Span::Lines(vec!["hi,".to_owned()]),
                    Span::Module(Box::new(Module {
                        is_test: false,
                        path: PathBuf::from("./a"),
                        spans: vec![Span::Lines(vec![
                            "hey".to_owned(),
                            "shallow".to_owned(),
                            " deep".to_owned(),
                        ])],
                    })),
                    Span::Lines(vec!["hello!".to_owned()]),
                ],
            },
        };
        assert_eq!(result, expected);
    }

    #[test]
    fn test_test_inline_module() {
        manual_resolver! {
            struct ManualResolver {
                "." => concat!(
                    "hi,\n",
                    "#[cfg(test)]\n",
                    "mod a {\n",
                    "    hey\n",
                    "   shallow\n",
                    "     deep\n",
                    "}\n",
                    "hello!\n",
                ),
            }
        }
        let result = bundle_crate("my_crate", ManualResolver {}, ConfigToml::new(""));
        let expected = Crate {
            name: "my_crate".to_owned(),
            root: Module {
                is_test: false,
                path: PathBuf::from("."),
                spans: vec![
                    Span::Lines(vec!["hi,".to_owned()]),
                    Span::Module(Box::new(Module {
                        is_test: true,
                        path: PathBuf::from("./a"),
                        spans: vec![Span::Lines(vec![
                            "hey".to_owned(),
                            "shallow".to_owned(),
                            " deep".to_owned(),
                        ])],
                    })),
                    Span::Lines(vec!["hello!".to_owned()]),
                ],
            },
        };
        assert_eq!(result, expected);
    }

    #[test]
    fn test_deep_inline_module() {
        manual_resolver! {
            struct ManualResolver {
                "." => concat!(
                    "begin .\n",
                    "mod a {\n",
                    "    begin a\n",
                    "    mod b {\n",
                    "        begin b\n",
                    "        mod c {\n",
                    "            begin c\n",
                    "            mod d {\n",
                    "                begin d\n",
                    "                end d\n",
                    "            }\n",
                    "            end c\n",
                    "        }\n",
                    "        end b\n",
                    "    }\n",
                    "    end a\n",
                    "}\n",
                    "end .\n",
                ),
            }
        }
        let result = bundle_crate("my_crate", ManualResolver {}, ConfigToml::new(""));
        let expected = Crate {
            name: "my_crate".to_owned(),
            root: Module {
                is_test: false,
                path: PathBuf::from("."),
                spans: vec![
                    Span::Lines(vec!["begin .".to_owned()]),
                    Span::Module(Box::new(Module {
                        is_test: false,
                        path: PathBuf::from("./a"),
                        spans: vec![
                            Span::Lines(vec!["begin a".to_owned()]),
                            Span::Module(Box::new(Module {
                                is_test: false,
                                path: PathBuf::from("./a/b"),
                                spans: vec![
                                    Span::Lines(vec!["begin b".to_owned()]),
                                    Span::Module(Box::new(Module {
                                        is_test: false,
                                        path: PathBuf::from("./a/b/c"),
                                        spans: vec![
                                            Span::Lines(vec!["begin c".to_owned()]),
                                            Span::Module(Box::new(Module {
                                                is_test: false,
                                                path: PathBuf::from("./a/b/c/d"),
                                                spans: vec![Span::Lines(vec![
                                                    "begin d".to_owned(),
                                                    "end d".to_owned(),
                                                ])],
                                            })),
                                            Span::Lines(vec!["end c".to_owned()]),
                                        ],
                                    })),
                                    Span::Lines(vec!["end b".to_owned()]),
                                ],
                            })),
                            Span::Lines(vec!["end a".to_owned()]),
                        ],
                    })),
                    Span::Lines(vec!["end .".to_owned()]),
                ],
            },
        };
        assert_eq!(result, expected);
    }

    #[test]
    fn test_deep_external_module() {
        manual_resolver! {
            struct ManualResolver {
                "." => concat!(
                    "begin .\n",
                    "mod a;\n",
                    "end .\n",
                ),
                "./a" => concat!(
                    "begin a\n",
                    "mod b;\n",
                    "end a\n",
                ),
                "./a/b" => concat!(
                    "begin b\n",
                    "mod c;\n",
                    "end b\n",
                ),
                "./a/b/c" => concat!(
                    "begin c\n",
                    "mod d;\n",
                    "end c\n",
                ),
                "./a/b/c/d" => concat!(
                    "begin d\n",
                    "end d\n",
                ),
            }
        }
        let result = bundle_crate("my_crate", ManualResolver {}, ConfigToml::new(""));
        let expected = Crate {
            name: "my_crate".to_owned(),
            root: Module {
                is_test: false,
                path: PathBuf::from("."),
                spans: vec![
                    Span::Lines(vec!["begin .".to_owned()]),
                    Span::Module(Box::new(Module {
                        is_test: false,
                        path: PathBuf::from("./a"),
                        spans: vec![
                            Span::Lines(vec!["begin a".to_owned()]),
                            Span::Module(Box::new(Module {
                                is_test: false,
                                path: PathBuf::from("./a/b"),
                                spans: vec![
                                    Span::Lines(vec!["begin b".to_owned()]),
                                    Span::Module(Box::new(Module {
                                        is_test: false,
                                        path: PathBuf::from("./a/b/c"),
                                        spans: vec![
                                            Span::Lines(vec!["begin c".to_owned()]),
                                            Span::Module(Box::new(Module {
                                                is_test: false,
                                                path: PathBuf::from("./a/b/c/d"),
                                                spans: vec![Span::Lines(vec![
                                                    "begin d".to_owned(),
                                                    "end d".to_owned(),
                                                ])],
                                            })),
                                            Span::Lines(vec!["end c".to_owned()]),
                                        ],
                                    })),
                                    Span::Lines(vec!["end b".to_owned()]),
                                ],
                            })),
                            Span::Lines(vec!["end a".to_owned()]),
                        ],
                    })),
                    Span::Lines(vec!["end .".to_owned()]),
                ],
            },
        };
        assert_eq!(result, expected);
    }

    #[test]
    fn test_many_inline_module() {
        manual_resolver! {
            struct ManualResolver {
                "." => concat!(
                    "begin .\n",
                    "mod a {\n",
                    "    begin a\n",
                    "    end a\n",
                    "}\n",
                    "between a and b\n",
                    "between a and b\n",
                    "between a and b\n",
                    "mod b {\n",
                    "    begin b\n",
                    "    end b\n",
                    "}\n",
                    "between b and c\n",
                    "between b and c\n",
                    "between b and c\n",
                    "mod c {\n",
                    "    begin c\n",
                    "    end c\n",
                    "}\n",
                    "end .\n",
                ),
            }
        }
        let result = bundle_crate("my_crate", ManualResolver {}, ConfigToml::new(""));
        let expected = Crate {
            name: "my_crate".to_owned(),
            root: Module {
                is_test: false,
                path: PathBuf::from("."),
                spans: vec![
                    Span::Lines(vec!["begin .".to_owned()]),
                    Span::Module(Box::new(Module {
                        is_test: false,
                        path: PathBuf::from("./a"),
                        spans: vec![Span::Lines(vec!["begin a".to_owned(), "end a".to_owned()])],
                    })),
                    Span::Lines(vec![
                        "between a and b".to_owned(),
                        "between a and b".to_owned(),
                        "between a and b".to_owned(),
                    ]),
                    Span::Module(Box::new(Module {
                        is_test: false,
                        path: PathBuf::from("./b"),
                        spans: vec![Span::Lines(vec!["begin b".to_owned(), "end b".to_owned()])],
                    })),
                    Span::Lines(vec![
                        "between b and c".to_owned(),
                        "between b and c".to_owned(),
                        "between b and c".to_owned(),
                    ]),
                    Span::Module(Box::new(Module {
                        is_test: false,
                        path: PathBuf::from("./c"),
                        spans: vec![Span::Lines(vec!["begin c".to_owned(), "end c".to_owned()])],
                    })),
                    Span::Lines(vec!["end .".to_owned()]),
                ],
            },
        };
        assert_eq!(result, expected);
    }

    #[test]
    fn test_many_moules_in_deep_module() {
        manual_resolver! {
            struct ManualResolver {
                "." => "mod a;\n",
                "./a" => concat!(
                    "mod b {\n",
                    "    mod c;\n",
                    "}\n",
                ),
                "./a/b/c" => "mod d;\n",
                "./a/b/c/d" => concat!(
                    "before e\n",
                    "before e\n",
                    "before e\n",
                    "mod e {\n",
                    "    mod f;\n",
                    "}\n",
                    "between e and g\n",
                    "between e and g\n",
                    "between e and g\n",
                    "mod g;\n",
                    "after g\n",
                    "after g\n",
                    "after g\n",
                ),
                "./a/b/c/d/e/f" => "in f",
                "./a/b/c/d/g" => "in g",
            }
        }
        let result = bundle_crate("my_crate", ManualResolver {}, ConfigToml::new(""));
        let expected = Crate {
            name: "my_crate".to_owned(),
            root: Module {
                is_test: false,
                path: PathBuf::from("."),
                spans: vec![Span::Module(Box::new(Module {
                    is_test: false,
                    path: PathBuf::from("./a"),
                    spans: vec![Span::Module(Box::new(Module {
                        is_test: false,
                        path: PathBuf::from("./a/b"),
                        spans: vec![Span::Module(Box::new(Module {
                            is_test: false,
                            path: PathBuf::from("./a/b/c"),
                            spans: vec![Span::Module(Box::new(Module {
                                is_test: false,
                                path: PathBuf::from("./a/b/c/d"),
                                spans: vec![
                                    Span::Lines(vec![
                                        "before e".to_owned(),
                                        "before e".to_owned(),
                                        "before e".to_owned(),
                                    ]),
                                    Span::Module(Box::new(Module {
                                        is_test: false,
                                        path: PathBuf::from("./a/b/c/d/e"),
                                        spans: vec![Span::Module(Box::new(Module {
                                            is_test: false,
                                            path: PathBuf::from("./a/b/c/d/e/f"),
                                            spans: vec![Span::Lines(vec!["in f".to_owned()])],
                                        }))],
                                    })),
                                    Span::Lines(vec![
                                        "between e and g".to_owned(),
                                        "between e and g".to_owned(),
                                        "between e and g".to_owned(),
                                    ]),
                                    Span::Module(Box::new(Module {
                                        is_test: false,
                                        path: PathBuf::from("./a/b/c/d/g"),
                                        spans: vec![Span::Lines(vec!["in g".to_owned()])],
                                    })),
                                    Span::Lines(vec![
                                        "after g".to_owned(),
                                        "after g".to_owned(),
                                        "after g".to_owned(),
                                    ]),
                                ],
                            }))],
                        }))],
                    }))],
                }))],
            },
        };
        assert_eq!(result, expected);
    }

    #[test]
    fn test_simple_substitution() {
        manual_resolver! {
            struct ManualResolver {
                "." => concat!(
                    "use crate_a::item_a;\n",
                ),
            }
        }
        let result = bundle_crate("my_crate", ManualResolver {}, build_sample_config_toml());
        let expected = Crate {
            name: "my_crate".to_owned(),
            root: Module {
                is_test: false,
                path: PathBuf::from("."),
                spans: vec![Span::Lines(vec!["use crate::crate_a::item_a;".to_owned()])],
            },
        };
        assert_eq!(result, expected);
    }

    #[test]
    fn test_substitute_at_deep_module() {
        manual_resolver! {
            struct ManualResolver {
                "." => concat!(
                    "mod a {\n",
                    "    use crate_a::item_a;\n",
                    "}\n",
                ),
            }
        }
        let result = bundle_crate("my_crate", ManualResolver {}, build_sample_config_toml());
        let expected = Crate {
            name: "my_crate".to_owned(),
            root: Module {
                is_test: false,
                path: PathBuf::from("."),
                spans: vec![Span::Module(Box::new(Module {
                    is_test: false,
                    path: PathBuf::from("./a"),
                    spans: vec![Span::Lines(vec!["use crate::crate_a::item_a;".to_owned()])],
                }))],
            },
        };
        assert_eq!(result, expected);
    }

    fn build_sample_config_toml() -> ConfigToml {
        ConfigToml::new(
            r#"
            [dependencies]
            crate_a = { path = "../crate_a" }
            crate_b = { path = "../crate_b" }
            crate_c = { path = "../crate_c" }
            crate_d = { path = "../crate_d" }
        "#,
        )
    }
}
