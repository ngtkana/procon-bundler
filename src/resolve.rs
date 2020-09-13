use std::path;

pub fn resolve(repo: &str, name: &str) -> path::PathBuf {
    let res = glob::glob(&format!("{}/**/{}/Cargo.toml", repo, name))
        .expect("glob パターンが読めておらずでしょうか。");
    let res = res
        .map(|glob_result| glob_result.expect("GlobResult のエラー側にいるようです。"))
        .collect::<Vec<path::PathBuf>>();
    assert!(
        !res.is_empty(),
        "見つかりませんでした。: repo = {}, name = {}",
        repo,
        name
    );
    assert!(
        res.len() == 1,
        "一意的ではありません。 repo = {}, name = {}",
        repo,
        name
    );
    let mut res = (*res[0]).to_path_buf();
    res.pop();
    res
}
