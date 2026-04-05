use crate::desktop::{Point, ResizeEdge, WindowId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowAction {
    Focus(WindowId),
    Raise(WindowId),
    StartMove(WindowId, Point),
    StartResize(WindowId, ResizeEdge, Point),
    EndGrab,
}
