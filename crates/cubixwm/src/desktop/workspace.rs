#[derive(Debug, Clone)]
pub struct Workspace {
    pub name: String,
}

impl Workspace {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}
