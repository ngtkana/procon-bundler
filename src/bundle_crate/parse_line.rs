use {lazy_static::lazy_static, regex::Regex};

pub fn parse_module_decl(line: &str) -> Option<String> {
    lazy_static! {
        static ref RE: Regex = Regex::new(concat!(
            "^",
            r#"\s*"#,                               // spaces
            r#"((pub|pub\s*\([^\)]+\))\s+)?"#,      // vis (capture 1, 2)
            r#"mod"#,                               // mod
            r#"\s+"#,                               // space
            r#"([a-z|A-Z|_]([a-z|A-Z|_|0-9]*))"#,   // name (capture 3, 4)
            r#"\s*"#,                               // spaces
            ";",                                    // semi
            r#"\s*$"#,                              // spaces
        ))
        .unwrap();
    }
    RE.captures(line).map(|captures| captures[3].to_owned())
}

pub fn parse_module_block_begin(line: &str) -> Option<String> {
    lazy_static! {
        static ref RE: Regex = Regex::new(concat!(
            "^",
            r#"\s*"#,                               // spaces
            r#"((pub|pub\s*\([^\)]+\))\s+)?"#,      // vis (capture 1, 2)
            r#"mod"#,                               // mod
            r#"\s+"#,                               // space
            r#"([a-z|A-Z|_]([a-z|A-Z|_|0-9]*))"#,   // name (capture 3, 4)
            r#"\s*"#,                               // spaces
            r#"\{"#,                                // opening brace
            r#"\s*$"#,                              // spaces
        ))
        .unwrap();
    }
    RE.captures(line).map(|captures| captures[3].to_owned())
}

// Leading spaces の個数を返します。
pub fn parse_block_end(line: &str) -> Option<usize> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r#"^(\s*)\}\s*$"#).unwrap();
    }
    RE.captures(line).map(|captures| captures[1].len())
}

// #[cfg(test)] であるかどうかを判定します。
pub fn parse_cfg_test(line: &str) -> bool {
    lazy_static! {
        static ref RE: Regex = Regex::new(r#"\s*#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]\s*"#).unwrap();
    }
    RE.is_match(line)
}

#[cfg(test)]
mod tests {
    use {
        super::{parse_block_end, parse_cfg_test, parse_module_block_begin, parse_module_decl},
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
}
