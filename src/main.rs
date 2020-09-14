mod bundle;
mod modules;
mod resolve;

use clap::{App, Arg};
use std::{fs, io::prelude::*, path};

fn cat(path: &path::Path) -> String {
    let mut f = fs::File::open(path)
        .unwrap_or_else(|e| panic!("ファイルがありませんよ: path = {:?}, e = {}", path, e));
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap_or_else(|e| {
        panic!(
            "Read::read_to_string で失敗するようですね。path = {:?}, e = {}",
            path, e
        )
    });
    s
}

fn main() {
    let matches = App::new("Procon bundler")
        .version("1.0")
        .author("Natgata Kanta <ngtkana@gmail.com>")
        .about("へへーん")
        .arg(
            Arg::with_name("repo")
                .short("r")
                .long("repo")
                .value_name("REPO")
                .help("レポジトリへのパス")
                .required(false)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("path")
                .short("p")
                .long("path")
                .value_name("PATH")
                .help("クレートルートへのパス")
                .required(false)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("name")
                .short("n")
                .long("name")
                .value_name("NAME")
                .help("クレートのおなまえ")
                .required(false)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("modules")
                .short("m")
                .long("modules")
                .help("モジュールツリーを作ります。")
                .required(false)
                .takes_value(false),
        )
        .get_matches();

    let crate_path = matches
        .value_of("path")
        .map(path::Path::new)
        .map(path::Path::to_owned)
        .or_else(|| {
            matches.value_of("name").and_then(|name| {
                matches
                    .value_of("repo")
                    .map(|repo| resolve::resolve(repo, name))
            })
        })
        .expect("path か name くらいは指定していただきたいものです。");

    if matches.is_present("modules") {
        modules::modules(&crate_path);
    } else {
        bundle::bundle(&crate_path);
    }
}
