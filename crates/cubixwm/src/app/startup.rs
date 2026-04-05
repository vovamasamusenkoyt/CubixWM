use crate::backend::Backend;
use crate::input::InputState;
use crate::protocol::ProtocolState;
use crate::render::Renderer;
use crate::x11::X11State;

pub struct BootSummary {
    pub backend_name: &'static str,
    pub renderer_name: &'static str,
}

pub fn boot(
    backend: &mut Backend,
    renderer: &mut Renderer,
    protocols: &mut ProtocolState,
    input: &mut InputState,
    x11: &mut X11State,
) -> BootSummary {
    backend.initialize();
    renderer.initialize();
    protocols.initialize();
    input.initialize();
    x11.initialize();

    BootSummary {
        backend_name: backend.name(),
        renderer_name: renderer.name(),
    }
}
