#[derive(Debug, Default)]
pub struct KeyboardState {
    initialized: bool,
}

impl KeyboardState {
    pub fn new() -> Self {
        Self { initialized: false }
    }

    pub fn initialize(&mut self) {
        self.initialized = true;
    }
}
