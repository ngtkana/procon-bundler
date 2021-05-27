use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

pub trait Resolve {
    type B: BufRead;
    fn resolve(&self, module_path: &Path) -> Self::B;
}

pub struct CrateResolver {
    root: PathBuf,
}
impl CrateResolver {
    pub fn new(path_to_crate_root: PathBuf) -> Self {
        Self {
            root: path_to_crate_root,
        }
    }
}
impl Resolve for CrateResolver {
    type B = BufReader<File>;
    // NOTE: mod.rs も探したい場合はここの実装も変えましょう！
    fn resolve(&self, module_path: &Path) -> Self::B {
        let mut buf = self.root.clone();
        buf.push("src");
        buf.push(module_path);
        buf.set_extension("rs");
        BufReader::new(File::open(buf).unwrap_or_else(|e| {
            panic!(
                concat!(
                    "モジュールパスからファイルへの解決に失敗しました。",
                    "module_path = {:?}, e = {:?}",
                ),
                module_path, e
            )
        }))
    }
}

#[cfg(test)]
mod tests {
    use {
        super::{CrateResolver, Resolve},
        crate::manual_resolver,
        std::{
            io::Read,
            path::{Path, PathBuf},
        },
    };

    #[test]
    fn test_manual_resolver() {
        manual_resolver! {
            struct ManualResolver {
                "a" => "content of a",
                "b" => "content of b",
            }
        }
        let resolver = ManualResolver {};

        let mut s = String::new();
        resolver
            .resolve(Path::new("a"))
            .read_to_string(&mut s)
            .unwrap();
        assert_eq!(s.as_str(), "content of a");
        s.clear();
        resolver
            .resolve(Path::new("b"))
            .read_to_string(&mut s)
            .unwrap();
        assert_eq!(s.as_str(), "content of b");
    }

    #[test]
    fn test_resolve_depth_1() {
        let mut s = String::new();

        let crate_resolver = CrateResolver::new(PathBuf::from("testcase/sample_crate"));
        crate_resolver
            .resolve(Path::new("a"))
            .read_to_string(&mut s)
            .unwrap();
        assert_eq!(s.as_str(), "Hi, I am a.rs!\n");
    }

    #[test]
    fn test_resolve_depth_2() {
        let mut s = String::new();

        let crate_resolver = CrateResolver::new(PathBuf::from("testcase/sample_crate"));
        crate_resolver
            .resolve(Path::new("b/b1"))
            .read_to_string(&mut s)
            .unwrap();
        assert_eq!(s.as_str(), "Hi, I am b1.rs!\n");
    }
}
