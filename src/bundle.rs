mod make;
mod remove;
mod wrap;

use std::path;

pub fn bundle(path: &path::Path) {
    let cargo_toml_content = super::cat(&path.join("Cargo.toml"));
    let make::Made { name, deps } = make::make(&cargo_toml_content);

    let lib_content = super::cat(&path.join("src/lib.rs"));
    let removed = remove::remove(&lib_content, &deps);
    let wrapped = wrap::wrap(&removed, &name);
    println!("{}", wrapped);
}
