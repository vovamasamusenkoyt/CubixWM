use crate::app::startup;
use crate::backend::Backend;
use crate::desktop::{Point, Rect, Size, WindowId, Workspace};
use crate::input::InputState;
use crate::protocol::ProtocolState;
use crate::render::Renderer;
use crate::utils::Result;
use crate::wm::WindowManager;
use crate::x11::X11State;

pub struct Application {
    backend: Backend,
    renderer: Renderer,
    protocols: ProtocolState,
    input: InputState,
    wm: WindowManager,
    x11: X11State,
}

impl Application {
    pub fn new() -> Self {
        Self {
            backend: Backend::new(),
            renderer: Renderer::new(),
            protocols: ProtocolState::new(),
            input: InputState::new(),
            wm: WindowManager::new(vec![Workspace::new("1")]),
            x11: X11State::new(),
        }
    }

    pub fn run(&mut self) -> Result<()> {
        let boot = startup::boot(
            &mut self.backend,
            &mut self.renderer,
            &mut self.protocols,
            &mut self.input,
            &mut self.x11,
        );

        println!(
            "cubixwm booted with backend={} renderer={}",
            boot.backend_name, boot.renderer_name
        );
        println!("wm ready; workspaces={}", self.wm.workspace_count());

        Ok(())
    }

    pub fn demo(&mut self) {
        let terminal = self.wm.create_window(
            "Terminal",
            Rect::new(40, 40, 960, 540),
            crate::desktop::WindowKind::Wayland,
        );
        let browser = self.wm.create_window(
            "Browser",
            Rect::new(120, 90, 1200, 800),
            crate::desktop::WindowKind::X11,
        );

        self.wm.focus(terminal);
        self.wm.raise(browser);
        self.wm.begin_move(browser, Point::new(120, 90));
        self.wm.update_cursor(Point::new(240, 160));
        self.wm.end_grab();

        self.wm.begin_resize(
            browser,
            crate::desktop::ResizeEdge::BottomRight,
            Point::new(0, 0),
        );
        self.wm.update_cursor(Point::new(150, 110));
        self.wm.end_grab();

        println!("stacking order: {:?}", self.wm.stacking());
        if let Some(window) = self.wm.window(browser) {
            println!(
                "browser => id={} title={} x={} y={} w={} h={}",
                window.id.0,
                window.title,
                window.rect.origin.x,
                window.rect.origin.y,
                window.rect.size.width,
                window.rect.size.height
            );
        }
    }
}

impl Default for Application {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
fn _unused_to_keep_imports(
    WindowId(_id): WindowId,
    Size {
        width: _,
        height: _,
    }: Size,
) {
}
