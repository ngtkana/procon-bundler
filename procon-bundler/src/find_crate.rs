mod workspace_cargo_toml;

use std::path::{Path, PathBuf};

pub fn find(workspace_root: &Path, crate_name: &Path) -> PathBuf {
    workspace_root.join("libs").join(crate_name)
}
