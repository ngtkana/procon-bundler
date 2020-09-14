use regex::Regex;
use std::{convert, path};

mod data_structures;
use data_structures as ds;

pub(super) fn modules(path: &path::Path) {
    let modules = extract(&super::cat(&path.join("src/lib.rs")));

    modules.iter().for_each(|module| {
        println!(
            "vis:{}, name:{} ({})",
            if module.vis.0.is_empty() {
                "none"
            } else {
                module.vis.0.as_ref()
            },
            module.name.0,
            match module.location {
                ds::Location::Inline => "inline",
                ds::Location::External => "external",
            }
        );
    });
}

fn extract(content: &str) -> Vec<ds::Module> {
    // mod name の後ろにブロックのはじめのカッコが来る感じのものです。
    fn inline(line: &str) -> Option<ds::Module> {
        let re = Regex::new(
            r"(|pub|pub\(self\)|pub\(super\)|pub\(crate\))\s*mod\s*([a-z|A-Z][a-z}A-Z|0-9|_]*)\s*\{",
        )
        .unwrap();
        re.captures(line).map(|captures| ds::Module {
            vis: ds::Vis(captures[1].to_string()),
            name: ds::Segment(captures[2].to_string()),
            location: ds::Location::Inline,
        })
    }

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

    content
        .split('\n')
        .map(|s| inline(s).or(external(s)))
        .filter_map(convert::identity)
        .collect::<Vec<ds::Module>>()
}
