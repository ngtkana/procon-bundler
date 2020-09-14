use regex::Regex;
use std::convert;

pub struct Made {
    pub name: String,
    pub deps: Vec<String>,
}

pub fn make(lines: &str) -> Made {
    let lines = lines
        .split('\n')
        .map(str::to_owned)
        .collect::<Vec<String>>();
    let name = name(&lines);
    let deps = dependencies_lines(&lines)
        .iter()
        .map(|s| parse_dependency_line(s))
        .filter_map(convert::identity)
        .collect::<Vec<_>>();
    Made { name, deps }
}

fn name(lines: &[String]) -> String {
    fn is_name(line: &str) -> Option<String> {
        let re = Regex::new(r#"name = "(.*)""#).unwrap();
        re.captures(line).map(|capture| capture[1].to_owned())
    }
    lines.iter().find_map(|line| is_name(line)).unwrap()
}

fn dependencies_lines(lines: &[String]) -> Vec<String> {
    fn is_start(line: &str) -> bool {
        line == "[dependencies]"
    }
    fn is_end(line: &str) -> bool {
        let re = Regex::new(r"\[.*\]").unwrap();
        re.is_match(line)
    }

    if let Some(start) = lines.iter().position(|line| is_start(&line)) {
        let start = start + 1;
        let end = lines[start..]
            .iter()
            .position(|line| is_end(&line))
            .map(|count| start + count)
            .unwrap_or(lines.len());
        lines[start..end].to_vec()
    } else {
        Vec::new()
    }
}

fn parse_dependency_line(line: &str) -> Option<String> {
    let mut iter = line.split('=');
    iter.next().and_then(|left| {
        if !left.is_empty() {
            Some(left.trim().to_owned())
        } else {
            None
        }
    })
}
