mod workspace_cargo_toml;

use {
    glob::glob,
    std::{
        fs::File,
        io::Read,
        path::{Path, PathBuf},
    },
    workspace_cargo_toml::WorkspaceCargoToml,
};

pub fn find(workspace_root: &Path, crate_name: &Path) -> PathBuf {
    let mut buf = String::new();
    File::open(workspace_root.join("Cargo.toml"))
        .unwrap_or_else(|_| panic!("ワークスペースルートに Cargo.toml がありません。"))
        .read_to_string(&mut buf)
        .unwrap_or_else(|_| panic!("ワークスペースルートの Cargo.toml が IO Error で読めません。"));
    WorkspaceCargoToml::new(&buf)
        .members
        .iter()
        .flat_map(|member| {
            glob(
                workspace_root
                    .join(member)
                    .to_str()
                    .expect("OsStr -> str の変換に失敗しました。"),
            )
            .unwrap_or_else(|_| panic!("glob に失敗しました。member = {:?}", member))
        })
        .map(|glob_result| glob_result.expect("glob の結果にエラーがありました。"))
        .find(|crate_path_buf| crate_path_buf.ends_with(crate_name))
        .unwrap_or_else(|| {
            panic!(
                "クレートが見つかりませんでした。 workspace_root = {:?}, crate_name = {:?}",
                workspace_root, crate_name,
            )
        })
}
