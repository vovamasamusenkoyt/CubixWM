use crate::desktop::{Point, Rect, ResizeEdge, WindowId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrabState {
    Move {
        window: WindowId,
        start_cursor: Point,
        start_rect: Rect,
    },
    Resize {
        window: WindowId,
        edge: ResizeEdge,
        start_cursor: Point,
        start_rect: Rect,
    },
}
