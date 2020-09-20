mod make;

use itertools::Itertools;
use regex::{Captures, Regex};
use std::{borrow, mem, path};

use super::data_structures as ds;

const TAB: &str = "    ";

pub fn bundle(crate_root: &path::Path) {
    // Cargo.toml を読んで依存クレートを読みます。
    let cargo_toml_content = super::cat(&crate_root.join("Cargo.toml"));
    let make::Made { name, deps } = make::make(&cargo_toml_content);

    // lib.rs を読んで `Vec<String>` 型に変換です。
    let mut lines = super::cat(&crate_root.join("src").join("lib.rs"))
        .split('\n')
        .map(str::to_owned)
        .collect::<Vec<_>>();

    // ますはモジュールを展開してそれぞれ mod name { } でつつみます。
    expand(&mut lines, crate_root, &name);

    // #[allow(dead_code)] アトリビュートをつけます。
    allow_dead_code(&mut lines);

    // Vim の fold marker をつけます。
    fold_marker(&mut lines, &name);

    // Doc comments を削除します。
    remove_doc_comments(&mut lines);

    // 外部クレートを参照するパスを書き換えます。
    replace_deps(&mut lines, &deps);

    println!(
        "{}",
        lines
            .iter()
            .map(String::as_ref)
            .intersperse("\n")
            .collect::<String>()
    );
}

fn expand(lines: &mut Vec<String>, crate_root: &path::Path, name: &str) {
    // mod name; の形のものです。
    fn external(line: &str) -> Option<ds::Module> {
        let re = Regex::new(
            r"(|pub|pub\(self\)|pub\(super\)|pub\(crate\))\s*mod ([a-z|A-Z][a-z}A-Z|0-9|_]*);",
        )
        .unwrap();
        re.captures(line).map(|captures| ds::Module {
            vis: ds::Vis(captures[1].to_string()),
            name: ds::Segment(captures[2].to_string()),
            location: ds::Location::External,
        })
    }

    // まずはルートモジュールのぷりプロセスです。
    preprocess(lines);

    *lines = lines
        .iter()
        .map(|line| match external(line) {
            Some(module) => {
                let path = crate_root
                    .join("src")
                    .join(module.name.0.clone())
                    .with_extension("rs");
                let mut sublines = super::cat(&path)
                    .split('\n')
                    .map(str::to_owned)
                    .collect::<Vec<String>>();

                // サブモジュールのプリプロセスです。
                preprocess(&mut sublines);
                // サブモジュールのポストプロセスです。
                postprocess(&mut sublines, module.vis, &module.name.0);
                remove_trailing_empty_lines(&mut sublines);
                sublines
            }
            None => vec![line.to_owned()],
        })
        .flatten()
        .collect::<Vec<String>>();

    // 最後にルートモジュールのポストプロセスです。
    postprocess(lines, ds::Vis(String::new()), name);
}

// -- 展開前に適用するもの

fn preprocess(lines: &mut Vec<String>) {
    // テストモジュールを消します。
    remove_test_modules(lines);
}

// -- 展開後に適用するもの

fn postprocess(lines: &mut Vec<String>, vis: ds::Vis, name: &str) {
    // 最後の空行を消します。
    remove_trailing_empty_lines(lines);

    // mod name { } をつけます。
    mod_block(lines, vis, name);
}

fn mod_block(lines: &mut Vec<String>, vis: ds::Vis, name: &str) {
    lines.iter_mut().for_each(|x| {
        if !x.is_empty() {
            *x = format!("{}{}", &TAB, &x)
        }
    });
    lines.insert(
        0,
        format!(
            "{}mod {} {{",
            if vis.0.is_empty() {
                String::new()
            } else {
                format!("{} ", vis.0)
            },
            name
        ),
    );
    lines.push("}".to_owned());
}
fn remove_test_modules(lines: &mut Vec<String>) {
    fn is_cfg_test(line: &str) -> bool {
        Regex::new(r"^#\[cfg\(test\)\]").unwrap().is_match(line)
    }
    fn is_start(line: &str) -> bool {
        Regex::new(r"mod\s*[a-z|A-Z][a-z|A-Z|0-9|_]*\s*\{")
            .unwrap()
            .is_match(line)
    }
    fn is_end(line: &str) -> bool {
        Regex::new(r"^\}").unwrap().is_match(line)
    }
    while let Some(start) = (0..lines.len() - 1)
        .find(|&start| is_cfg_test(&lines[start]) && is_start(&lines[start + 1]))
    {
        let count = 1 + lines[start..]
            .iter()
            .enumerate()
            .find(|&(_, line)| is_end(line))
            .unwrap()
            .0;

        let (l, cr) = lines.split_at_mut(start);
        let (_, r) = cr.split_at_mut(count);
        let mut l = l.to_vec();
        l.extend_from_slice(r);
        mem::swap(lines, &mut l);
    }
}
fn remove_trailing_empty_lines(lines: &mut Vec<String>) {
    fn is_empty(line: &str) -> bool {
        Regex::new(r"^\s*$").unwrap().is_match(line)
    }
    while let Some(line) = lines.pop() {
        if !is_empty(&line) {
            lines.push(line);
            break;
        }
    }
}

// -- 展開後に適用するもの

fn allow_dead_code(lines: &mut Vec<String>) {
    lines.insert(0, "#[allow(dead_code)]".to_owned());
}

fn fold_marker(lines: &mut Vec<String>, name: &str) {
    lines.insert(0, format!("// {} {{{{{{", name));
    lines.push("// }}}".to_owned());
}

fn remove_doc_comments(lines: &mut Vec<String>) {
    fn is_doc_comment(line: &str) -> bool {
        let re: Regex = Regex::new(r"//[/|!]").unwrap();
        re.is_match(line)
    }
    lines.retain(|line| !is_doc_comment(line));
}

fn replace_deps(lines: &mut Vec<String>, deps: &[String]) {
    fn replace(line: &mut String, dep: &str) {
        let s = format!("{}::", dep);
        let re = Regex::new(&s).unwrap();
        match re.replace(&line, |caps: &Captures| format!("crate::{}", &caps[0])) {
            borrow::Cow::Owned(s) => *line = s,
            borrow::Cow::Borrowed(_) => (),
        }
    }
    for line in lines.iter_mut() {
        deps.iter().for_each(|dep| replace(line, dep));
    }
}
