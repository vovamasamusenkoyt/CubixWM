mod cursor;
mod scene;

use cursor::CursorRenderer;
use scene::SceneRenderer;

pub struct Renderer {
    scene: SceneRenderer,
    cursor: CursorRenderer,
    initialized: bool,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            scene: SceneRenderer,
            cursor: CursorRenderer,
            initialized: false,
        }
    }

    pub fn initialize(&mut self) {
        let _ = (&self.scene, &self.cursor);
        self.initialized = true;
    }

    pub fn name(&self) -> &'static str {
        if self.initialized {
            "renderer-stub"
        } else {
            "renderer-stub(uninitialized)"
        }
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}
