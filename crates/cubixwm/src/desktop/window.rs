use crate::desktop::Rect;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowKind {
    Wayland,
    X11,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResizeEdge {
    Top,
    Bottom,
    Left,
    Right,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

#[derive(Debug, Clone)]
pub struct Window {
    pub id: WindowId,
    pub title: String,
    pub kind: WindowKind,
    pub rect: Rect,
    pub mapped: bool,
    pub fullscreen: bool,
}

impl Window {
    pub fn new(id: WindowId, title: impl Into<String>, kind: WindowKind, rect: Rect) -> Self {
        Self {
            id,
            title: title.into(),
            kind,
            rect,
            mapped: true,
            fullscreen: false,
        }
    }
}
