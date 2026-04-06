use crate::utils::{Error, Result};
use drm::{
    Device as BasicDevice,
    buffer::{Buffer, DrmFourcc},
    control::{Device as ControlDevice, connector, crtc, framebuffer},
};
use smithay::{
    backend::{
        renderer::{
            Bind, Color32F, Frame, Renderer,
            element::{
                Kind,
                surface::{WaylandSurfaceRenderElement, render_elements_from_surface_tree},
            },
            pixman::PixmanRenderer,
            utils::{draw_render_elements, on_commit_buffer_handler},
        },
        session::{Session, libseat::LibSeatSession},
        udev::primary_gpu,
    },
    delegate_compositor, delegate_data_device, delegate_seat, delegate_shm, delegate_xdg_shell,
    input::{Seat, SeatHandler, SeatState, pointer::CursorImageStatus},
    reexports::{
        pixman::{FormatCode, Image},
        wayland_server::{
            Client, Display, ListeningSocket,
            backend::{ClientData, ClientId, DisconnectReason},
            protocol::{
                wl_buffer, wl_seat,
                wl_surface::{self, WlSurface},
            },
        },
    },
    utils::{Rectangle, Serial, Size, Transform},
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
    fs::File,
    os::fd::{AsFd, BorrowedFd, OwnedFd},
    path::PathBuf,
    process::{Command, Stdio},
    sync::Arc,
    thread,
    time::{Duration, Instant},
};
use wayland_protocols::xdg::shell::server::xdg_toplevel;

pub struct TtyBackend;

struct TtyCompositor {
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

impl BufferHandler for TtyCompositor {
    fn buffer_destroyed(&mut self, _buffer: &wl_buffer::WlBuffer) {}
}

impl XdgShellHandler for TtyCompositor {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        surface.with_pending_state(|state| {
            state.states.set(xdg_toplevel::State::Activated);
            state.size = Some((960, 640).into());
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

impl SelectionHandler for TtyCompositor {
    type SelectionUserData = ();
}

impl DataDeviceHandler for TtyCompositor {
    fn data_device_state(&self) -> &DataDeviceState {
        &self.data_device_state
    }
}

impl ClientDndGrabHandler for TtyCompositor {}

impl ServerDndGrabHandler for TtyCompositor {
    fn send(&mut self, _mime_type: String, _fd: OwnedFd, _seat: Seat<Self>) {}
}

impl CompositorHandler for TtyCompositor {
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

impl ShmHandler for TtyCompositor {
    fn shm_state(&self) -> &ShmState {
        &self.shm_state
    }
}

impl SeatHandler for TtyCompositor {
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

delegate_xdg_shell!(TtyCompositor);
delegate_compositor!(TtyCompositor);
delegate_shm!(TtyCompositor);
delegate_seat!(TtyCompositor);
delegate_data_device!(TtyCompositor);

impl TtyBackend {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&mut self) -> Result<()> {
        let (mut session, _notifier) = LibSeatSession::new()
            .map_err(|error| Error::new(format!("failed to open libseat session: {error}")))?;
        let seat = session.seat();
        let gpu_path = primary_gpu(&seat)
            .map_err(|error| {
                Error::new(format!("failed to query primary gpu for {seat}: {error}"))
            })?
            .ok_or_else(|| Error::new(format!("no gpu found for seat {seat}")))?;

        let fd = session
            .open(
                &gpu_path,
                rustix::fs::OFlags::RDWR | rustix::fs::OFlags::CLOEXEC,
            )
            .map_err(|error| {
                Error::new(format!(
                    "failed to open gpu {}: {error:?}",
                    gpu_path.display()
                ))
            })?;
        let card = Card::new(fd, gpu_path.clone());

        let output = choose_output(&card).map_err(|error| {
            Error::new(format!(
                "failed to choose drm output on {}: {error}",
                gpu_path.display()
            ))
        })?;

        let _restore = RestoreCrtc::capture(&card, &output);
        let mut buffer = card
            .create_dumb_buffer(
                (output.mode.size().0.into(), output.mode.size().1.into()),
                DrmFourcc::Xrgb8888,
                32,
            )
            .map_err(|error| Error::new(format!("failed to create dumb buffer: {error}")))?;
        let framebuffer = card
            .add_framebuffer(&buffer, 24, 32)
            .map_err(|error| Error::new(format!("failed to create framebuffer: {error}")))?;

        card.set_crtc(
            output.crtc.handle(),
            Some(framebuffer),
            (0, 0),
            &[output.connector.handle()],
            Some(output.mode),
        )
        .map_err(|error| Error::new(format!("failed to set crtc: {error}")))?;

        let mut display: Display<TtyCompositor> = Display::new()
            .map_err(|error| Error::new(format!("failed to create display: {error}")))?;
        let dh = display.handle();

        let compositor_state = CompositorState::new::<TtyCompositor>(&dh);
        let shm_state = ShmState::new::<TtyCompositor>(&dh, vec![]);
        let mut seat_state = SeatState::new();
        let seat = seat_state.new_wl_seat(&dh, "cubixwm-tty");
        let _keyboard = seat
            .add_keyboard(Default::default(), 200, 200)
            .map_err(|error| Error::new(format!("failed to initialize keyboard seat: {error}")))?;

        let mut state = TtyCompositor {
            compositor_state,
            xdg_shell_state: XdgShellState::new::<TtyCompositor>(&dh),
            shm_state,
            seat_state,
            data_device_state: DataDeviceState::new::<TtyCompositor>(&dh),
        };

        let listener = ListeningSocket::bind_auto("cubixwm-", 32..128)
            .map_err(|error| Error::new(format!("failed to bind wayland socket: {error}")))?;
        let socket_name = listener
            .socket_name()
            .expect("listener did not expose a socket name")
            .to_string_lossy()
            .into_owned();

        eprintln!(
            "cubixwm tty mode on {} connector={:?} crtc={:?} mode={}x{}",
            gpu_path.display(),
            output.connector.handle(),
            output.crtc.handle(),
            output.mode.size().0,
            output.mode.size().1
        );
        eprintln!("wayland display: {socket_name}");
        eprintln!("spawning foot test client");
        eprintln!("press Ctrl+C to exit");

        spawn_demo_client(&socket_name);

        let mut renderer = PixmanRenderer::new().map_err(|error| {
            Error::new(format!("failed to initialize pixman renderer: {error}"))
        })?;
        let damage = Rectangle::from_size(Size::<i32, _>::from((
            output.mode.size().0 as i32,
            output.mode.size().1 as i32,
        )));
        let start_time = Instant::now();
        let mut clients = Vec::new();

        loop {
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

            let pitch = buffer.pitch() as usize;
            let mut mapping = card
                .map_dumb_buffer(&mut buffer)
                .map_err(|error| Error::new(format!("failed to map dumb buffer: {error}")))?;
            let format = FormatCode::try_from(DrmFourcc::Xrgb8888)
                .map_err(|_| Error::new("pixman does not support Xrgb8888"))?;
            let mut target = unsafe {
                Image::from_raw_mut(
                    format,
                    output.mode.size().0 as usize,
                    output.mode.size().1 as usize,
                    mapping.as_mut().as_mut_ptr() as *mut u32,
                    pitch,
                    false,
                )
            }
            .map_err(|_| Error::new("failed to bind dumb buffer as pixman image"))?;

            let elements = state
                .xdg_shell_state
                .toplevel_surfaces()
                .iter()
                .flat_map(|surface| {
                    render_elements_from_surface_tree(
                        &mut renderer,
                        surface.wl_surface(),
                        (48, 48),
                        1.0,
                        1.0,
                        Kind::Unspecified,
                    )
                })
                .collect::<Vec<WaylandSurfaceRenderElement<PixmanRenderer>>>();

            {
                let mut framebuffer = renderer.bind(&mut target).map_err(|error| {
                    Error::new(format!("failed to bind pixman target: {error}"))
                })?;
                let mut frame = renderer
                    .render(&mut framebuffer, damage.size, Transform::Normal)
                    .map_err(|error| {
                        Error::new(format!("failed to start pixman frame: {error}"))
                    })?;

                frame
                    .clear(Color32F::new(0.05, 0.06, 0.08, 1.0), &[damage])
                    .map_err(|error| {
                        Error::new(format!("failed to clear pixman frame: {error}"))
                    })?;

                draw_render_elements(&mut frame, 1.0, &elements, &[damage])
                    .map_err(|error| Error::new(format!("failed to draw surface tree: {error}")))?;

                frame.finish().map_err(|error| {
                    Error::new(format!("failed to finish pixman frame: {error}"))
                })?;
            }

            for surface in state.xdg_shell_state.toplevel_surfaces() {
                send_frames_surface_tree(
                    surface.wl_surface(),
                    start_time.elapsed().as_millis() as u32,
                );
            }

            let _ = card.dirty_framebuffer(framebuffer, &[]);
            thread::sleep(Duration::from_millis(16));
        }
    }
}

impl Default for TtyBackend {
    fn default() -> Self {
        Self::new()
    }
}

struct Card {
    file: File,
}

impl Card {
    fn new(fd: OwnedFd, path: PathBuf) -> Self {
        let _ = path;
        Self {
            file: File::from(fd),
        }
    }
}

impl AsFd for Card {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.file.as_fd()
    }
}

impl BasicDevice for Card {}
impl ControlDevice for Card {}

struct OutputSelection {
    connector: connector::Info,
    crtc: crtc::Info,
    mode: drm::control::Mode,
}

fn choose_output(card: &Card) -> std::result::Result<OutputSelection, String> {
    let resources = card
        .resource_handles()
        .map_err(|error| format!("resource_handles failed: {error}"))?;

    let connector = resources
        .connectors()
        .iter()
        .filter_map(|handle| card.get_connector(*handle, false).ok())
        .find(|info| info.state() == connector::State::Connected && !info.modes().is_empty())
        .ok_or_else(|| "no connected drm connector with a mode".to_string())?;

    let mode = connector
        .modes()
        .first()
        .copied()
        .ok_or_else(|| "connector has no drm mode".to_string())?;

    let encoder = connector
        .current_encoder()
        .or_else(|| connector.encoders().first().copied())
        .ok_or_else(|| "connector has no encoder".to_string())?;

    let encoder_info = card
        .get_encoder(encoder)
        .map_err(|error| format!("get_encoder failed: {error}"))?;

    let crtc_handle = encoder_info.crtc().or_else(|| {
        resources
            .filter_crtcs(encoder_info.possible_crtcs())
            .first()
            .copied()
    });
    let crtc_handle = crtc_handle.ok_or_else(|| "encoder has no usable crtc".to_string())?;

    let crtc = card
        .get_crtc(crtc_handle)
        .map_err(|error| format!("get_crtc failed: {error}"))?;

    Ok(OutputSelection {
        connector,
        crtc,
        mode,
    })
}

struct RestoreCrtc<'a> {
    card: &'a Card,
    crtc: crtc::Handle,
    framebuffer: Option<framebuffer::Handle>,
    position: (u32, u32),
    connectors: Vec<connector::Handle>,
    mode: Option<drm::control::Mode>,
}

impl<'a> RestoreCrtc<'a> {
    fn capture(card: &'a Card, output: &OutputSelection) -> Self {
        Self {
            card,
            crtc: output.crtc.handle(),
            framebuffer: output.crtc.framebuffer(),
            position: output.crtc.position(),
            connectors: vec![output.connector.handle()],
            mode: output.crtc.mode(),
        }
    }
}

impl Drop for RestoreCrtc<'_> {
    fn drop(&mut self) {
        let _ = self.card.set_crtc(
            self.crtc,
            self.framebuffer,
            self.position,
            &self.connectors,
            self.mode,
        );
    }
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
