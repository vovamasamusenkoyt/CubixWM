#[derive(Debug, Default)]
pub struct PointerState {
    initialized: bool,
}

impl PointerState {
    pub fn new() -> Self {
        Self { initialized: false }
    }

    pub fn initialize(&mut self) {
        self.initialized = true;
    }
}
