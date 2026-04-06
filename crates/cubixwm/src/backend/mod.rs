mod renderer;
mod smithay;
#[cfg(feature = "tty-backend")]
mod tty;
mod winit;

pub use renderer::BackendRenderer;
pub use smithay::SmithayBackend;
#[cfg(feature = "tty-backend")]
pub use tty::TtyBackend;
pub use winit::WinitBackend;

pub struct Backend {
    smithay: SmithayBackend,
    #[cfg(feature = "tty-backend")]
    tty: TtyBackend,
    winit: WinitBackend,
    initialized: bool,
}

impl Backend {
    pub fn new() -> Self {
        Self {
            smithay: SmithayBackend::new(),
            #[cfg(feature = "tty-backend")]
            tty: TtyBackend::new(),
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

    pub fn run_tty(&mut self) -> crate::utils::Result<()> {
        #[cfg(feature = "tty-backend")]
        {
            return self.tty.run();
        }

        #[cfg(not(feature = "tty-backend"))]
        {
            Err(crate::utils::Error::new(
                "tty backend is disabled in this build; rebuild with `--features tty-backend`",
            ))
        }
    }
}

impl Default for Backend {
    fn default() -> Self {
        Self::new()
    }
}
