use std::path::PathBuf;

#[derive(Clone, Debug, Default, Hash, PartialEq)]
pub struct Module {
    pub path: PathBuf,
    pub spans: Vec<Span>,
    pub is_test: bool,
}
impl Module {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            spans: Vec::new(),
            is_test: false,
        }
    }
}
#[derive(Clone, Debug, Hash, PartialEq)]
pub enum Span {
    Lines(Vec<String>),
    Module(Box<Module>),
}
