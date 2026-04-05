use crate::desktop::{
    FocusStack, Point, Rect, ResizeEdge, StackingOrder, Window, WindowId, WindowKind, Workspace,
};
use crate::wm::GrabState;
use std::collections::HashMap;

pub struct WindowManager {
    next_id: u64,
    windows: HashMap<WindowId, Window>,
    stacking: StackingOrder,
    focus: FocusStack,
    grab: Option<GrabState>,
    workspaces: Vec<Workspace>,
}

impl WindowManager {
    pub fn new(workspaces: Vec<Workspace>) -> Self {
        Self {
            next_id: 1,
            windows: HashMap::new(),
            stacking: StackingOrder::default(),
            focus: FocusStack::default(),
            grab: None,
            workspaces,
        }
    }

    pub fn workspace_count(&self) -> usize {
        self.workspaces.len()
    }

    pub fn create_window(&mut self, title: &str, rect: Rect, kind: WindowKind) -> WindowId {
        let id = WindowId(self.next_id);
        self.next_id += 1;

        let window = Window::new(id, title, kind, rect);
        self.windows.insert(id, window);
        self.stacking.push(id);
        self.focus.focus(id);
        id
    }

    pub fn focus(&mut self, id: WindowId) {
        if self.windows.contains_key(&id) {
            self.focus.focus(id);
        }
    }

    pub fn focused(&self) -> Option<WindowId> {
        self.focus.current()
    }

    pub fn raise(&mut self, id: WindowId) {
        if self.windows.contains_key(&id) {
            self.stacking.raise(id);
        }
    }

    pub fn begin_move(&mut self, id: WindowId, cursor: Point) {
        if let Some(window) = self.windows.get(&id) {
            self.grab = Some(GrabState::Move {
                window: id,
                start_cursor: cursor,
                start_rect: window.rect,
            });
        }
    }

    pub fn begin_resize(&mut self, id: WindowId, edge: ResizeEdge, cursor: Point) {
        if let Some(window) = self.windows.get(&id) {
            self.grab = Some(GrabState::Resize {
                window: id,
                edge,
                start_cursor: cursor,
                start_rect: window.rect,
            });
        }
    }

    pub fn update_cursor(&mut self, cursor: Point) {
        match self.grab {
            Some(GrabState::Move {
                window,
                start_cursor,
                start_rect,
            }) => {
                if let Some(target) = self.windows.get_mut(&window) {
                    let dx = cursor.x - start_cursor.x;
                    let dy = cursor.y - start_cursor.y;
                    target.rect.origin.x = start_rect.origin.x + dx;
                    target.rect.origin.y = start_rect.origin.y + dy;
                }
            }
            Some(GrabState::Resize {
                window,
                edge,
                start_cursor,
                start_rect,
            }) => {
                if let Some(target) = self.windows.get_mut(&window) {
                    let dx = cursor.x - start_cursor.x;
                    let dy = cursor.y - start_cursor.y;
                    apply_resize(target, edge, start_rect, dx, dy);
                }
            }
            None => {}
        }
    }

    pub fn end_grab(&mut self) {
        self.grab = None;
    }

    pub fn stacking(&self) -> Vec<WindowId> {
        self.stacking.all().to_vec()
    }

    pub fn window(&self, id: WindowId) -> Option<&Window> {
        self.windows.get(&id)
    }
}

fn apply_resize(window: &mut Window, edge: ResizeEdge, start_rect: Rect, dx: i32, dy: i32) {
    const MIN_WIDTH: i32 = 100;
    const MIN_HEIGHT: i32 = 80;

    let mut x = start_rect.origin.x;
    let mut y = start_rect.origin.y;
    let mut width = start_rect.size.width;
    let mut height = start_rect.size.height;

    match edge {
        ResizeEdge::Top => {
            y += dy;
            height -= dy;
        }
        ResizeEdge::Bottom => {
            height += dy;
        }
        ResizeEdge::Left => {
            x += dx;
            width -= dx;
        }
        ResizeEdge::Right => {
            width += dx;
        }
        ResizeEdge::TopLeft => {
            x += dx;
            width -= dx;
            y += dy;
            height -= dy;
        }
        ResizeEdge::TopRight => {
            width += dx;
            y += dy;
            height -= dy;
        }
        ResizeEdge::BottomLeft => {
            x += dx;
            width -= dx;
            height += dy;
        }
        ResizeEdge::BottomRight => {
            width += dx;
            height += dy;
        }
    }

    if width < MIN_WIDTH {
        width = MIN_WIDTH;
        if matches!(
            edge,
            ResizeEdge::Left | ResizeEdge::TopLeft | ResizeEdge::BottomLeft
        ) {
            x = start_rect.origin.x + (start_rect.size.width - MIN_WIDTH);
        }
    }

    if height < MIN_HEIGHT {
        height = MIN_HEIGHT;
        if matches!(
            edge,
            ResizeEdge::Top | ResizeEdge::TopLeft | ResizeEdge::TopRight
        ) {
            y = start_rect.origin.y + (start_rect.size.height - MIN_HEIGHT);
        }
    }

    window.rect = Rect::new(x, y, width, height);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_updates_window_position() {
        let mut wm = WindowManager::new(vec![Workspace::new("1")]);
        let id = wm.create_window("demo", Rect::new(10, 20, 300, 200), WindowKind::Wayland);

        wm.begin_move(id, Point::new(10, 10));
        wm.update_cursor(Point::new(40, 50));
        wm.end_grab();

        let window = wm.window(id).unwrap();
        assert_eq!(window.rect.origin, Point::new(40, 60));
    }

    #[test]
    fn resize_respects_minimum_size() {
        let mut wm = WindowManager::new(vec![Workspace::new("1")]);
        let id = wm.create_window("demo", Rect::new(0, 0, 300, 200), WindowKind::Wayland);

        wm.begin_resize(id, ResizeEdge::TopLeft, Point::new(0, 0));
        wm.update_cursor(Point::new(500, 500));
        wm.end_grab();

        let window = wm.window(id).unwrap();
        assert_eq!(window.rect.size.width, 100);
        assert_eq!(window.rect.size.height, 80);
    }
}
