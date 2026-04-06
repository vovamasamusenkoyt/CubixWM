use crate::utils::{Error, Result};
use chrono::Local;
use smithay::{
    backend::{
        input::{InputEvent, KeyboardKeyEvent},
        renderer::{
            Color32F, Frame, Renderer,
            element::{
                Kind,
                surface::{WaylandSurfaceRenderElement, render_elements_from_surface_tree},
            },
            gles::GlesRenderer,
            utils::{draw_render_elements, on_commit_buffer_handler},
        },
        winit::{self, WinitEvent},
    },
    delegate_compositor, delegate_data_device, delegate_seat, delegate_shm, delegate_xdg_shell,
    input::{Seat, SeatHandler, SeatState, keyboard::FilterResult, pointer::CursorImageStatus},
    reexports::wayland_server::{
        Client, Display, ListeningSocket,
        backend::{ClientData, ClientId, DisconnectReason},
        protocol::{
            wl_buffer, wl_seat,
            wl_surface::{self, WlSurface},
        },
    },
    utils::{Rectangle, Serial, Transform},
    wayland::{
        buffer::BufferHandler,
        compositor::{
            CompositorClientState, CompositorHandler, CompositorState, SurfaceAttributes,
            TraversalAction, with_surface_tree_downward,
        },
        selection::{
            SelectionHandler,
            data_device::{
                ClientDndGrabHandler, DataDeviceHandler, DataDeviceState, ServerDndGrabHandler,
            },
        },
        shell::xdg::{
            PopupSurface, PositionerState, ToplevelSurface, XdgShellHandler, XdgShellState,
        },
        shm::{ShmHandler, ShmState},
    },
};
use std::{
    os::unix::io::OwnedFd,
    process::{Command, Stdio},
    sync::Arc,
    time::Instant,
};
use wayland_protocols::xdg::shell::server::xdg_toplevel;

pub struct SmithayBackend {
    initialized: bool,
    display_created: bool,
    event_loop_created: bool,
}

struct NestedCompositor {
    compositor_state: CompositorState,
    xdg_shell_state: XdgShellState,
    shm_state: ShmState,
    seat_state: SeatState<Self>,
    data_device_state: DataDeviceState,
}

#[derive(Default)]
struct ClientState {
    compositor_state: CompositorClientState,
}

impl BufferHandler for NestedCompositor {
    fn buffer_destroyed(&mut self, _buffer: &wl_buffer::WlBuffer) {}
}

impl XdgShellHandler for NestedCompositor {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        surface.with_pending_state(|state| {
            state.states.set(xdg_toplevel::State::Activated);
        });
        surface.send_configure();
    }

    fn new_popup(&mut self, _surface: PopupSurface, _positioner: PositionerState) {}

    fn grab(&mut self, _surface: PopupSurface, _seat: wl_seat::WlSeat, _serial: Serial) {}

    fn reposition_request(
        &mut self,
        _surface: PopupSurface,
        _positioner: PositionerState,
        _token: u32,
    ) {
    }
}

impl SelectionHandler for NestedCompositor {
    type SelectionUserData = ();
}

impl DataDeviceHandler for NestedCompositor {
    fn data_device_state(&self) -> &DataDeviceState {
        &self.data_device_state
    }
}

impl ClientDndGrabHandler for NestedCompositor {}

impl ServerDndGrabHandler for NestedCompositor {
    fn send(&mut self, _mime_type: String, _fd: OwnedFd, _seat: Seat<Self>) {}
}

impl CompositorHandler for NestedCompositor {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }

    fn client_compositor_state<'a>(&self, client: &'a Client) -> &'a CompositorClientState {
        &client
            .get_data::<ClientState>()
            .expect("missing client state")
            .compositor_state
    }

    fn commit(&mut self, surface: &WlSurface) {
        on_commit_buffer_handler::<Self>(surface);
    }
}

impl ShmHandler for NestedCompositor {
    fn shm_state(&self) -> &ShmState {
        &self.shm_state
    }
}

impl SeatHandler for NestedCompositor {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;
    type TouchFocus = WlSurface;

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.seat_state
    }

    fn focus_changed(&mut self, _seat: &Seat<Self>, _focused: Option<&WlSurface>) {}

    fn cursor_image(&mut self, _seat: &Seat<Self>, _image: CursorImageStatus) {}
}

impl ClientData for ClientState {
    fn initialized(&self, _client_id: ClientId) {}

    fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) {}
}

delegate_xdg_shell!(NestedCompositor);
delegate_compositor!(NestedCompositor);
delegate_shm!(NestedCompositor);
delegate_seat!(NestedCompositor);
delegate_data_device!(NestedCompositor);

impl SmithayBackend {
    pub fn new() -> Self {
        Self {
            initialized: false,
            display_created: false,
            event_loop_created: false,
        }
    }

    pub fn initialize(&mut self) {
        let _display = Display::<()>::new().expect("failed to create wayland display");
        let _event_loop = smithay::reexports::calloop::EventLoop::<()>::try_new()
            .expect("failed to create calloop event loop");

        self.display_created = true;
        self.event_loop_created = true;
        self.initialized = true;
    }

    pub fn name(&self) -> &'static str {
        if self.initialized {
            "smithay-bootstrap"
        } else {
            "smithay-bootstrap(uninitialized)"
        }
    }

    pub fn summary(&self) -> &'static str {
        match (
            self.initialized,
            self.display_created,
            self.event_loop_created,
        ) {
            (true, true, true) => "display+event-loop ready",
            _ => "not ready",
        }
    }

    pub fn run_nested(&mut self) -> Result<()> {
        let mut display: Display<NestedCompositor> = Display::new()
            .map_err(|error| Error::new(format!("failed to create display: {error}")))?;
        let mut dh = display.handle();

        let compositor_state = CompositorState::new::<NestedCompositor>(&dh);
        let shm_state = ShmState::new::<NestedCompositor>(&dh, vec![]);
        let mut seat_state = SeatState::new();
        let mut seat = seat_state.new_wl_seat(&dh, "cubixwm-winit");
        let keyboard = seat
            .add_keyboard(Default::default(), 200, 200)
            .map_err(|error| Error::new(format!("failed to initialize keyboard seat: {error}")))?;

        let mut state = NestedCompositor {
            compositor_state,
            xdg_shell_state: XdgShellState::new::<NestedCompositor>(&dh),
            shm_state,
            seat_state,
            data_device_state: DataDeviceState::new::<NestedCompositor>(&dh),
        };

        let listener = ListeningSocket::bind_auto("cubixwm-", 32..128)
            .map_err(|error| Error::new(format!("failed to bind wayland socket: {error}")))?;
        let socket_name = listener
            .socket_name()
            .expect("listener did not expose a socket name")
            .to_string_lossy()
            .into_owned();

        let attributes = smithay::reexports::winit::window::Window::default_attributes()
            .with_title(&window_title(&socket_name))
            .with_inner_size(smithay::reexports::winit::dpi::LogicalSize::new(
                1280.0, 800.0,
            ));

        let (mut backend, mut event_loop) = winit::init_from_attributes::<GlesRenderer>(attributes)
            .map_err(|error| {
                Error::new(format!(
                    "failed to initialize smithay winit backend: {error}"
                ))
            })?;

        spawn_demo_client(&socket_name);
        let start_time = Instant::now();
        let mut clients = Vec::new();

        loop {
            let status = event_loop.dispatch_new_events(|event| match event {
                WinitEvent::Resized { .. } | WinitEvent::Redraw | WinitEvent::Focus(_) => {}
                WinitEvent::Input(event) => match event {
                    InputEvent::Keyboard { event } => {
                        let _ = keyboard.input::<(), _>(
                            &mut state,
                            event.key_code(),
                            event.state(),
                            0.into(),
                            0,
                            |_, _, _| FilterResult::Forward,
                        );
                    }
                    InputEvent::PointerMotionAbsolute { .. } => {
                        if let Some(surface) = state
                            .xdg_shell_state
                            .toplevel_surfaces()
                            .iter()
                            .next()
                            .cloned()
                        {
                            keyboard.set_focus(
                                &mut state,
                                Some(surface.wl_surface().clone()),
                                0.into(),
                            );
                        }
                    }
                    _ => {}
                },
                WinitEvent::CloseRequested => {}
            });

            match status {
                smithay::reexports::winit::platform::pump_events::PumpStatus::Continue => {}
                smithay::reexports::winit::platform::pump_events::PumpStatus::Exit(_) => break,
            }

            let size = backend.window_size();
            let damage = Rectangle::from_size(size);
            {
                let (renderer, mut framebuffer) = backend
                    .bind()
                    .map_err(|error| Error::new(format!("failed to bind renderer: {error}")))?;

                let elements = state
                    .xdg_shell_state
                    .toplevel_surfaces()
                    .iter()
                    .flat_map(|surface| {
                        render_elements_from_surface_tree(
                            renderer,
                            surface.wl_surface(),
                            (0, 0),
                            1.0,
                            1.0,
                            Kind::Unspecified,
                        )
                    })
                    .collect::<Vec<WaylandSurfaceRenderElement<GlesRenderer>>>();

                let mut frame = renderer
                    .render(&mut framebuffer, size, Transform::Flipped180)
                    .map_err(|error| Error::new(format!("failed to start frame: {error}")))?;

                frame
                    .clear(Color32F::new(0.08, 0.09, 0.11, 1.0), &[damage])
                    .map_err(|error| Error::new(format!("failed to clear frame: {error}")))?;

                draw_render_elements(&mut frame, 1.0, &elements, &[damage])
                    .map_err(|error| Error::new(format!("failed to draw surface tree: {error}")))?;

                let _ = frame
                    .finish()
                    .map_err(|error| Error::new(format!("failed to finish frame: {error}")))?;

                for surface in state.xdg_shell_state.toplevel_surfaces() {
                    send_frames_surface_tree(
                        surface.wl_surface(),
                        start_time.elapsed().as_millis() as u32,
                    );
                }

                if let Some(stream) = listener
                    .accept()
                    .map_err(|error| Error::new(format!("failed to accept client: {error}")))?
                {
                    let client = dh
                        .insert_client(stream, Arc::new(ClientState::default()))
                        .map_err(|error| Error::new(format!("failed to insert client: {error}")))?;
                    clients.push(client);
                }

                display
                    .dispatch_clients(&mut state)
                    .map_err(|error| Error::new(format!("failed to dispatch clients: {error}")))?;
                display
                    .flush_clients()
                    .map_err(|error| Error::new(format!("failed to flush clients: {error}")))?;
            }

            backend
                .submit(Some(&[damage]))
                .map_err(|error| Error::new(format!("failed to submit frame: {error}")))?;
        }

        let _ = clients;
        Ok(())
    }
}

impl Default for SmithayBackend {
    fn default() -> Self {
        Self::new()
    }
}

fn window_title(socket_name: &str) -> String {
    format!(
        "CubixWM | {} | {}",
        socket_name,
        Local::now().format("%H:%M:%S")
    )
}

fn spawn_demo_client(socket_name: &str) {
    let script = "while true; do clear; date; sleep 1; done";
    let _ = Command::new("foot")
        .env("WAYLAND_DISPLAY", socket_name)
        .env(
            "XDG_RUNTIME_DIR",
            std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| format!("/run/user/{}", nix_uid())),
        )
        .arg("-T")
        .arg("CubixWM Clock")
        .arg("-e")
        .arg("sh")
        .arg("-lc")
        .arg(script)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
}

fn send_frames_surface_tree(surface: &wl_surface::WlSurface, time: u32) {
    with_surface_tree_downward(
        surface,
        (),
        |_, _, &()| TraversalAction::DoChildren(()),
        |_surface, states, &()| {
            for callback in states
                .cached_state
                .get::<SurfaceAttributes>()
                .current()
                .frame_callbacks
                .drain(..)
            {
                callback.done(time);
            }
        },
        |_, _, &()| true,
    );
}

fn nix_uid() -> u32 {
    std::process::Command::new("id")
        .arg("-u")
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .and_then(|value| value.trim().parse::<u32>().ok())
        .unwrap_or(1000)
}
