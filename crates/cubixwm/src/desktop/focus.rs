use crate::desktop::WindowId;

#[derive(Debug, Default)]
pub struct FocusStack {
    current: Option<WindowId>,
}

impl FocusStack {
    pub fn focus(&mut self, window: WindowId) {
        self.current = Some(window);
    }

    pub fn current(&self) -> Option<WindowId> {
        self.current
    }
}
