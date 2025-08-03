use {
    crate::{BundlerError, Result},
    std::{
        fs::File,
        io::{BufRead, BufReader},
        path::{Path, PathBuf},
    },
};

pub trait Resolve {
    type B: BufRead;
    fn resolve(&self, module_path: &Path) -> Result<Self::B>;
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
    fn resolve(&self, module_path: &Path) -> Result<Self::B> {
        let mut buf = self.root.clone();
        let is_root = module_path
            .to_str()
            .ok_or_else(|| BundlerError::InvalidPathConversion {
                path: module_path.to_path_buf(),
            })?
            == ".";
        buf.push("src");
        buf.push(if is_root {
            Path::new("lib")
        } else {
            module_path
        });
        buf.set_extension("rs");
        
        let file = File::open(&buf).map_err(|e| BundlerError::ModuleFileNotFound {
            module_path: module_path.to_path_buf(),
            file_path: buf,
            source: e,
        })?;
        
        Ok(BufReader::new(file))
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
            .unwrap()
            .read_to_string(&mut s)
            .unwrap();
        assert_eq!(s.as_str(), "content of a");
        s.clear();
        resolver
            .resolve(Path::new("b"))
            .unwrap()
            .read_to_string(&mut s)
            .unwrap();
        assert_eq!(s.as_str(), "content of b");
    }

    #[test]
    fn test_resolve_depth_1() {
        let mut s = String::new();

        let crate_resolver = CrateResolver::new(PathBuf::from("../procon-bundler-sample"));
        crate_resolver
            .resolve(Path::new("small_module"))
            .unwrap()
            .read_to_string(&mut s)
            .unwrap();
        assert_eq!(
            s.as_str(),
            concat!("#[allow(dead_code)]\n", "pub type A = ();\n",)
        );
    }
}
