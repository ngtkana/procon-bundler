use {
    crate::{ConfigToml, TAB_LENGTH},
    lazy_static::lazy_static,
    regex::Captures,
    regex::Regex,
    std::borrow::Cow,
};

pub fn parse_module_decl(line: &str) -> Option<String> {
    lazy_static! {
        static ref RE: Regex = Regex::new(concat!(
            "^",
            r#"\s*"#,                                // spaces
            r#"((pub|pub\s*\([^\)]+\))\s+)?"#,       // vis
            r#"mod"#,                                // mod
            r#"\s+"#,                                // space
            r#"(?P<name>[a-zA-Z_]([a-zA-Z_0-9]*))"#, // name <- capture here!
            r#"\s*"#,                                // spaces
            ";",                                     // semi
            r#"\s*$"#,                               // spaces
        ))
        .unwrap();
    }
    RE.captures(line)
        .map(|captures| captures.name("name").unwrap().as_str().to_owned())
}

pub fn parse_module_block_begin(line: &str) -> Option<String> {
    lazy_static! {
        static ref RE: Regex = Regex::new(concat!(
            "^",
            r#"\s*"#,                                // spaces
            r#"((pub|pub\s*\([^\)]+\))\s+)?"#,       // vis
            r#"mod"#,                                // mod
            r#"\s+"#,                                // space
            r#"(?P<name>[a-zA-Z_]([a-zA-Z_0-9]*))"#, // name <- capture here!
            r#"\s*"#,                                // spaces
            r#"\{"#,                                 // opening brace
            r#"\s*$"#,                               // spaces
        ))
        .unwrap();
    }
    RE.captures(line)
        .map(|captures| captures.name("name").unwrap().as_str().to_owned())
}

// Leading spaces の個数を返します。
pub fn parse_block_end(line: &str) -> Option<usize> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r#"^(?P<leading>\s*)\}\s*$"#).unwrap();
    }
    RE.captures(line)
        .map(|captures| captures.name("leading").unwrap().as_str().len())
}

// #[cfg(test)] であるかどうかを判定します。
pub fn parse_cfg_test(line: &str) -> bool {
    lazy_static! {
        static ref RE: Regex = Regex::new(r#"^\s*#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]\s*"#).unwrap();
    }
    RE.is_match(line)
}

// oneline doc_comments であるかを判定します。
pub fn parse_oneline_doc_comments(line: &str) -> bool {
    line.trim().starts_with("///") || line.trim().starts_with("//!")
}

// block doc_comments の開始であるかを判定します。
pub fn parse_block_doc_comments_start(line: &str) -> bool {
    line.trim().starts_with("/*!") || line.trim().starts_with("/**")
}

// block doc_comments の終了であるかを判定します。
pub fn parse_block_doc_comments_end(line: &str) -> bool {
    line.trim().ends_with("*/")
}

// たかだか指定個数のインデントを消します。
pub fn remove_indentation(line: &str, indent_level: usize) -> String {
    let mut chars = line.chars().peekable();
    let mut rest = indent_level * TAB_LENGTH;
    while let Some(c) = chars.peek() {
        match c {
            ' ' => {
                if rest == 0 {
                    break;
                } else {
                    rest -= 1;
                }
            }
            '\t' => {
                if rest < TAB_LENGTH {
                    break;
                } else {
                    rest -= TAB_LENGTH;
                }
            }
            _ => break,
        }
        chars.next();
    }
    chars.collect::<String>()
}

// パスの置換をします。
pub fn substitute_path<'a>(line: &'a str, crate_name: &str, config: &ConfigToml) -> String {
    pub fn crates(line: &str, crate_name: &str) -> String {
        lazy_static! {
            static ref RE: Regex = Regex::new(r#"([^\$])crate::"#).unwrap();
        }
        RE.replace_all(line, |caps: &Captures| {
            format!("{}crate::{}::", &caps[1], crate_name.replace('-', "_"))
        })
        .into_owned()
    }
    pub fn crate_macros(line: &str, crate_name: &str) -> String {
        line.replace(
            "$crate",
            &format!("$crate::{}", crate_name.replace('-', &"_")),
        )
    }
    pub fn non_macro<'a>(line: &'a str, config: &ConfigToml) -> Cow<'a, str> {
        fn replace(caps: &Captures, config: &ConfigToml) -> String {
            let name = caps.name("name").unwrap().as_str();
            if config.deps.contains_key(name) {
                format!("crate::{}::", name)
            } else {
                format!("{}::", name)
            }
        }
        lazy_static! {
            static ref RE: Regex =
                Regex::new(r#"(?P<name>[A-Za-z_][A-Za-z0-9_\-]*)(\u{3A}){2}"#).unwrap();
        }
        RE.replace_all(line, |caps: &Captures| replace(caps, config))
    }
    let line = crates(line, crate_name);
    let line = crate_macros(&line, crate_name);
    non_macro(&line, config).into_owned()
}

#[cfg(test)]
mod tests {
    use {
        super::{
            parse_block_doc_comments_end, parse_block_doc_comments_start, parse_block_end,
            parse_cfg_test, parse_module_block_begin, parse_module_decl,
            parse_oneline_doc_comments, substitute_path, ConfigToml,
        },
        test_case::test_case,
    };

    #[test_case("mod a;" => Some("a".to_owned()); "simple mod decl")]
    #[test_case("pub mod a;" => Some("a".to_owned()); "pub mod decl")]
    #[test_case("pub(crate) mod a;" => Some("a".to_owned()); "pub(crate) mod decl")]
    #[test_case("pub(super::super) mod a;" => Some("a".to_owned()); "pub(path) mod decl")]
    #[test_case("    mod a;" => Some("a".to_owned()); "with leading spaces")]
    #[test_case("use a;" => None; "fake(use decl)")]
    #[test_case("mod ab;" => Some("ab".to_owned()); "two-char name")]
    #[test_case("mod a1;" => Some("a1".to_owned()); "contains digit")]
    #[test_case("mod a_;" => Some("a_".to_owned()); "contains under")]
    #[test_case("mod _a;" => Some("_a".to_owned()); "starts with under")]
    #[test_case("pub ( path  ::hey )  mod  __my_42_fn  ; " => Some("__my_42_fn".to_owned()); "complicated")]
    fn test_parse_module_decl(line: &str) -> Option<String> {
        parse_module_decl(line)
    }

    #[test_case("mod a {" => Some("a".to_owned()); "simple mod decl")]
    #[test_case("    pub ( foo   ) mod  bar  { " => Some("bar".to_owned()); "complicated")]
    fn test_parse_module_block_begin(line: &str) -> Option<String> {
        parse_module_block_begin(line)
    }

    #[test_case("}" => Some(0); "simple block end")]
    #[test_case("}      " => Some(0); "block end with trailing spaces")]
    #[test_case("      }" => Some(6); "block end with leading spaces")]
    #[test_case("   }   " => Some(3); "block end with leading and trailing spaces")]
    fn test_parse_block_end(line: &str) -> Option<usize> {
        parse_block_end(line)
    }

    #[test_case("#[cfg(test)]" => true; "simple cfg(test)")]
    #[test_case("#  [  cfg  (  test  )  ]  " => true; "cfg(test) with may spaces")]
    fn test_parse_cfg_test(line: &str) -> bool {
        parse_cfg_test(line)
    }

    #[test_case("/// hi" => true; "outer doc comments")]
    #[test_case("//! hi" => true; "inner doc comments")]
    #[test_case("    /// hi" => true; "outer doc comments with leading spaces")]
    #[test_case("    //! hi" => true; "inner doc comments with leading spaces")]
    fn test_parse_oneline_doc_comments(line: &str) -> bool {
        parse_oneline_doc_comments(line)
    }

    #[test_case("    /** hi    " => true; "outer block doc comments start")]
    #[test_case("    /*! hi    " => true; "inner block doc comments start")]
    fn test_parse_block_doc_comments_start(line: &str) -> bool {
        parse_block_doc_comments_start(line)
    }

    #[test_case("    hi **/    " => true; "block doc comments end")]
    fn test_parse_block_doc_comments_end(line: &str) -> bool {
        parse_block_doc_comments_end(line)
    }

    #[test_case("use crate_a::f" => "use crate::crate_a::f".to_owned(); "simple qualified use")]
    #[test_case("use crate_a::f as _" => "use crate::crate_a::f as _".to_owned(); "qualified use as")]
    #[test_case("use crate_never::f" => "use crate_never::f".to_owned(); "not in deps")]
    #[test_case("let _: crate_a::Type = crate_a::Type::new()" => "let _: crate::crate_a::Type = crate::crate_a::Type::new()".to_owned(); "expand twice")]
    #[test_case("type X = (crate_a::A, crate_b::B);" => "type X = (crate::crate_a::A, crate::crate_b::B);".to_owned(); "expand two distinct crates")]
    fn test_substitute_non_macro_path(line: &str) -> String {
        substitute_path(line, "my_crate", &build_sample_config_toml())
    }

    #[test_case("$crate::a" => "$crate::my_crate::a".to_owned(); "simple $crate")]
    #[test_case("crate::a" => "crate::a".to_owned(); "not `$crate` but just `crate`")]
    fn test_substitute_macro_path(line: &str) -> String {
        substitute_path(line, "my_crate", &build_sample_config_toml())
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
