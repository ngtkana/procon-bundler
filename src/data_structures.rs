#[derive(Debug, Clone)]
pub struct Module {
    pub vis: Vis,
    pub name: Segment,
    pub location: Location,
}

#[derive(Debug, Clone)]
pub struct Vis(pub String);

#[derive(Debug, Clone)]
pub struct ModulePath(pub Vec<Segment>);

#[derive(Debug, Clone)]
pub struct Segment(pub String);

#[derive(Debug, Clone)]
pub enum Location {
    Inline,
    External,
}
