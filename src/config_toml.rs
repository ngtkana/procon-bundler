use {
    std::{collections::HashMap, path::PathBuf},
    toml::{from_str, Value},
};

pub struct ConfigToml {
    pub deps: HashMap<String, PathBuf>,
}

impl ConfigToml {
    pub fn new(file_content: &str) -> Self {
        // dependency の行の一つの、`=` よりも右側をパースします。
        fn from_resource(resource: &Value) -> Option<PathBuf> {
            match resource {
                Value::Table(resource) => resource.get("path").map(|path| {
                    PathBuf::from(
                        path.as_str()
                            .unwrap_or_else(|| {
                                panic!("path の値が string ではありません。path = {:?}", path)
                            })
                            .to_string(),
                    )
                }),
                _ => None,
            }
        }
        // [dependencies] セクションをパースします。
        fn from_deps(deps: &Value) -> HashMap<String, PathBuf> {
            match deps {
                Value::Table(deps) => deps
                    .iter()
                    .flat_map(|(name, dep)| {
                        from_resource(dep).map(|pathbuf| (name.to_string(), pathbuf))
                    })
                    .collect::<HashMap<_, _>>(),
                _ => panic!("dependencies が Table ではありませんでした。"),
            }
        }
        // ファイル全体をパースします。
        Self {
            deps: match from_str::<Value>(file_content).expect("TOML をパースできませんでした。")
            {
                Value::Table(table) => table
                    .get("dependencies")
                    .map(|deps| from_deps(deps))
                    .unwrap_or_else(Default::default),
                _ => panic!("TOML ファイルが Table ではありませんでした。"),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use {
        super::ConfigToml,
        std::{collections::HashMap, path::PathBuf},
    };

    #[test]
    fn test_only_path_dependencies() {
        let config = ConfigToml::new(
            r#"
            [dependencies]
            a = { path = "../path/to/a"}
            b = { path = "../path/to/b"}
            c = { path = "../path/to/c"}
        "#,
        );
        let mut expected = HashMap::new();
        expected.insert("a".to_string(), PathBuf::from("../path/to/a"));
        expected.insert("b".to_string(), PathBuf::from("../path/to/b"));
        expected.insert("c".to_string(), PathBuf::from("../path/to/c"));
        assert_eq!(config.deps, expected);
    }

    #[test]
    fn test_skips_git_and_crates_io_dependencies() {
        let config = ConfigToml::new(
            r#"
            [dependencies]
            a = { path = "../path/to/a"}
            toml = "0.5.8"
            b = { path = "../path/to/b"}
            dbg = { git = "https://github.com/ngtkana/ac-adapter-rs.git", optional = true }
            c = { path = "../path/to/c"}
        "#,
        );
        let mut expected = HashMap::new();
        expected.insert("a".to_string(), PathBuf::from("../path/to/a"));
        expected.insert("b".to_string(), PathBuf::from("../path/to/b"));
        expected.insert("c".to_string(), PathBuf::from("../path/to/c"));
        assert_eq!(config.deps, expected);
    }

    #[test]
    fn test_skips_dev_dependencies() {
        let config = ConfigToml::new(
            r#"
            [dependencies]
            a = { path = "../path/to/a"}

            [dev-dependencies]
            b = { path = "../path/to/b"}
            c = { path = "../path/to/c"}
        "#,
        );
        let mut expected = HashMap::new();
        expected.insert("a".to_string(), PathBuf::from("../path/to/a"));
        assert_eq!(config.deps, expected);
    }

    #[test]
    fn test_parse_typical_cargo_toml() {
        let config = ConfigToml::new(
            r#"
            [package]
            name = "hi!"
            version = "5.0.0"
            authors = ["me"]
            edition = "2018"

            # See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

            [dependencies]
            lazy_static = "1.4"
            here = { path = "../here" }
            itertools = "0.8"
            semver = "0.9.0"
            thiserror = "1.0.16"
            num-traits = "0.2"
            there = { path = "../there" }
            boolinator = "2.4.0"

            [dev-dependencies]
            assert_approx_eq = "1"
            hamcrest2 = "0.3.0"
            test-case = { version = "1", features = ["hamcrest_assertions"] }
        "#,
        );
        let mut expected = HashMap::new();
        expected.insert("here".to_string(), PathBuf::from("../here"));
        expected.insert("there".to_string(), PathBuf::from("../there"));
        assert_eq!(config.deps, expected);
    }
}
