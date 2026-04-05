use crate::desktop::WindowId;

#[derive(Debug, Default)]
pub struct StackingOrder {
    order: Vec<WindowId>,
}

impl StackingOrder {
    pub fn push(&mut self, id: WindowId) {
        self.order.push(id);
    }

    pub fn raise(&mut self, id: WindowId) {
        self.order.retain(|existing| *existing != id);
        self.order.push(id);
    }

    pub fn all(&self) -> &[WindowId] {
        &self.order
    }
}
