use std::{
    fs::File,
    io::{BufRead, BufReader},
};

pub trait Resolve {
    type B: BufRead;
    fn resolve(path: &str) -> Self::B;
}

pub enum FsResolver {}
impl Resolve for FsResolver {
    type B = BufReader<File>;
    fn resolve(path: &str) -> Self::B {
        BufReader::new(File::open(path).unwrap_or_else(|e| {
            panic!(
                "パスからファイルへの解決に失敗しました。path = {:?}, e = {:?}",
                path, e
            )
        }))
    }
}

#[allow(unused_macros)]
macro_rules! manual_resolver {
    (enum $resolver_name:ident {
        $($path:expr => $content:expr),* $(,)?
    }) => {
        enum $resolver_name {}
        impl $crate::resolver::Resolve for $resolver_name {
            type B = ::std::io::BufReader<&'static [u8]>;
            fn resolve(path: &str) -> Self::B {
                ::std::io::BufReader::new(
                    match path {
                        $(
                            $path => $content,
                        )*
                        _ => unreachable!(),
                    }
                    .as_bytes(),
                )
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::{FsResolver, Resolve};
    use std::io::Read;

    #[test]
    fn test_manual_resolver() {
        manual_resolver! {
            enum ManualResolver {
                "a" => "content of a",
                "b" => "content of b",
            }
        }

        let mut s = String::new();
        ManualResolver::resolve("a").read_to_string(&mut s).unwrap();
        assert_eq!(s.as_str(), "content of a");
        s.clear();
        ManualResolver::resolve("b").read_to_string(&mut s).unwrap();
        assert_eq!(s.as_str(), "content of b");
    }

    #[test]
    fn test_fs_resolver() {
        let mut s = String::new();

        FsResolver::resolve("testcase/test_fs_resolver.txt")
            .read_to_string(&mut s)
            .unwrap();
        assert_eq!(s.as_str(), "Hi!\n");
    }
}
