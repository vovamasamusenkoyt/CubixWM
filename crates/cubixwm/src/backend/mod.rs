mod renderer;
mod winit;

pub use renderer::BackendRenderer;
pub use winit::WinitBackend;

pub struct Backend {
    winit: WinitBackend,
    initialized: bool,
}

impl Backend {
    pub fn new() -> Self {
        Self {
            winit: WinitBackend::new(),
            initialized: false,
        }
    }

    pub fn initialize(&mut self) {
        self.winit.initialize();
        self.initialized = true;
    }

    pub fn name(&self) -> &'static str {
        if self.initialized {
            self.winit.name()
        } else {
            "uninitialized"
        }
    }
}

impl Default for Backend {
    fn default() -> Self {
        Self::new()
    }
}
