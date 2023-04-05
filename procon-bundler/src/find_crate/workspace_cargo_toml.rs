use toml::{from_str, Value};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct WorkspaceCargoToml {
    pub members: Vec<String>,
}

impl WorkspaceCargoToml {
    pub fn new(file_content: &str) -> Self {
        // ファイル全体をパースします。
        Self {
            members: match from_str::<Value>(file_content)
                .expect("TOML をパースできませんでした。")
            {
                Value::Table(table) => table
                    .get("workspace")
                    .unwrap_or_else(|| {
                        panic!("ワークスペースの Cargo.toml に `workspace` がありません。")
                    })
                    .get("members")
                    .unwrap_or_else(|| {
                        panic!("ワークスペースの Cargo.toml の `workspace` に `members` がありません。")
                    })
                    .as_array()
                    .unwrap_or_else(|| {
                        panic!("ワークスペースの Cargo.toml の `workspace` の `members` が配列ではありません。")
                    })
                    .iter()
                    .map(|value| value.as_str()
                        .unwrap_or_else(|| {
                            panic!("ワークスペースの Cargo.toml の `workspace` の `members` の要素が文字列ではありません。")
                        })
                        .to_owned()
                    ).collect::<Vec<_>>(),
                _ => panic!("TOML ファイルが Table ではありませんでした。"),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::WorkspaceCargoToml;

    #[test]
    fn test_parse_typical_cargo_toml() {
        let config = WorkspaceCargoToml::new(
            r#"
            [workspace]
            members = [
                "a/b/*",
                "c",
            ]
        "#,
        );
        let expected = vec!["a/b/*".to_owned(), "c".to_owned()];
        assert_eq!(config.members, expected);
    }
}
