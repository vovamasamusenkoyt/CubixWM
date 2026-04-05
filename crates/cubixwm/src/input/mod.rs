mod bindings;
mod keyboard;
mod pointer;

pub use bindings::Bindings;
pub use keyboard::KeyboardState;
pub use pointer::PointerState;

pub struct InputState {
    keyboard: KeyboardState,
    pointer: PointerState,
    bindings: Bindings,
    initialized: bool,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            keyboard: KeyboardState::new(),
            pointer: PointerState::new(),
            bindings: Bindings::default(),
            initialized: false,
        }
    }

    pub fn initialize(&mut self) {
        self.keyboard.initialize();
        self.pointer.initialize();
        let _ = &self.bindings;
        self.initialized = true;
    }
}

impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}
