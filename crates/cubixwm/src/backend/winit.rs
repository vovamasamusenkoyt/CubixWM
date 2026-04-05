pub struct WinitBackend {
    initialized: bool,
}

impl WinitBackend {
    pub fn new() -> Self {
        Self { initialized: false }
    }

    pub fn initialize(&mut self) {
        self.initialized = true;
    }

    pub fn name(&self) -> &'static str {
        if self.initialized {
            "winit-stub"
        } else {
            "winit-stub(uninitialized)"
        }
    }
}
