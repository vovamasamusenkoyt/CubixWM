mod compositor;
mod output;
mod seat;
mod shm;
mod xdg_shell;

use compositor::CompositorProtocol;
use output::OutputProtocol;
use seat::SeatProtocol;
use shm::ShmProtocol;
use xdg_shell::XdgShellProtocol;

pub struct ProtocolState {
    compositor: CompositorProtocol,
    xdg_shell: XdgShellProtocol,
    seat: SeatProtocol,
    output: OutputProtocol,
    shm: ShmProtocol,
    initialized: bool,
}

impl ProtocolState {
    pub fn new() -> Self {
        Self {
            compositor: CompositorProtocol,
            xdg_shell: XdgShellProtocol,
            seat: SeatProtocol,
            output: OutputProtocol,
            shm: ShmProtocol,
            initialized: false,
        }
    }

    pub fn initialize(&mut self) {
        let _ = (
            &self.compositor,
            &self.xdg_shell,
            &self.seat,
            &self.output,
            &self.shm,
        );
        self.initialized = true;
    }
}

impl Default for ProtocolState {
    fn default() -> Self {
        Self::new()
    }
}
