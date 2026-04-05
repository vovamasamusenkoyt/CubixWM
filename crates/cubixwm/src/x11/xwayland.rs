#[derive(Debug, Default)]
pub struct X11State {
    initialized: bool,
}

impl X11State {
    pub fn new() -> Self {
        Self { initialized: false }
    }

    pub fn initialize(&mut self) {
        self.initialized = true;
    }
}
