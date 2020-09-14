use regex::Regex;
use std::{borrow, mem};

pub fn remove(file_content: &str, deps: &[String]) -> String {
    let mut lines = file_content
        .split('\n')
        .map(str::to_owned)
        .collect::<Vec<String>>();
    remove_doc_comments(&mut lines);
    remove_tests(&mut lines);
    replace_deps(&mut lines, deps);
    remove_trailing_empty_lines(&mut lines);
    lines.join("\n")
}

fn remove_doc_comments(lines: &mut Vec<String>) {
    fn is_doc_comment(line: &str) -> bool {
        let re: Regex = Regex::new(r"//[/|!]").unwrap();
        re.is_match(line)
    }
    lines.retain(|line| !is_doc_comment(line));
}

fn remove_tests(lines: &mut Vec<String>) {
    fn is_cfg_test(line: &str) -> bool {
        line == "#[cfg(test)]"
    }
    fn is_start(line: &str) -> bool {
        line == "mod tests {"
    }
    fn is_end(line: &str) -> bool {
        line == "}"
    }
    if let Some((start, _)) = lines.iter().enumerate().find(|&(_, line)| is_start(line)) {
        assert!(is_cfg_test(&lines[start - 1]));
        let start = start - 1;
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

fn replace_deps(lines: &mut Vec<String>, deps: &[String]) {
    fn replace(line: &mut String, dep: &str) {
        let s = format!("{}::", dep);
        let re = Regex::new(&s).unwrap();
        match re.replace(&line, "crate::constant::") {
            borrow::Cow::Owned(s) => *line = s,
            borrow::Cow::Borrowed(_) => (),
        }
    }
    for line in lines.iter_mut() {
        deps.iter().for_each(|dep| replace(line, dep));
    }
}

fn remove_trailing_empty_lines(lines: &mut Vec<String>) {
    while let Some(line) = lines.pop() {
        if !line.is_empty() {
            lines.push(line);
            break;
        }
    }
}
