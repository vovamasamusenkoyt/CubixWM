mod renderer;
mod smithay;
mod winit;

pub use renderer::BackendRenderer;
pub use smithay::SmithayBackend;
pub use winit::WinitBackend;

pub struct Backend {
    smithay: SmithayBackend,
    winit: WinitBackend,
    initialized: bool,
}

impl Backend {
    pub fn new() -> Self {
        Self {
            smithay: SmithayBackend::new(),
            winit: WinitBackend::new(),
            initialized: false,
        }
    }

    pub fn initialize(&mut self) {
        self.smithay.initialize();
        self.winit.initialize();
        self.initialized = true;
    }

    pub fn name(&self) -> &'static str {
        if self.initialized {
            self.smithay.name()
        } else {
            "uninitialized"
        }
    }

    pub fn smithay_summary(&self) -> &'static str {
        self.smithay.summary()
    }

    pub fn run_nested(&mut self) -> crate::utils::Result<()> {
        self.smithay.run_nested()
    }
}

impl Default for Backend {
    fn default() -> Self {
        Self::new()
    }
}
