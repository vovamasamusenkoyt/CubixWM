use crate::utils::{Error, Result};
use drm::{
    buffer::{Buffer, DrmFourcc},
    control::{connector, crtc, Device as ControlDevice},
    Device as BasicDevice,
};
use smithay::backend::{
    session::{libseat::LibSeatSession, Session},
    udev::primary_gpu,
};
use std::{
    fs::File,
    os::fd::{AsFd, BorrowedFd, OwnedFd},
    path::PathBuf,
    thread,
    time::Duration,
};

pub struct TtyBackend;

impl TtyBackend {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&mut self) -> Result<()> {
        let (mut session, _notifier) = LibSeatSession::new()
            .map_err(|error| Error::new(format!("failed to open libseat session: {error}")))?;
        let seat = session.seat();
        let gpu_path = primary_gpu(&seat)
            .map_err(|error| Error::new(format!("failed to query primary gpu for {seat}: {error}")))?
            .ok_or_else(|| Error::new(format!("no gpu found for seat {seat}")))?;

        let fd = session
            .open(&gpu_path, rustix::fs::OFlags::RDWR | rustix::fs::OFlags::CLOEXEC)
            .map_err(|error| Error::new(format!("failed to open gpu {}: {error:?}", gpu_path.display())))?;
        let card = Card::new(fd, gpu_path.clone());

        let output = choose_output(&card)
            .map_err(|error| Error::new(format!("failed to choose drm output on {}: {error}", gpu_path.display())))?;

        eprintln!(
            "cubixwm tty mode on {} connector={:?} crtc={:?} mode={}x{}",
            gpu_path.display(),
            output.connector.handle(),
            output.crtc.handle(),
            output.mode.size().0,
            output.mode.size().1
        );
        eprintln!("press Ctrl+C to exit");

        let restore = RestoreCrtc::capture(&card, &output);
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

        let mut phase: u8 = 0;
        loop {
            let mut mapping = card
                .map_dumb_buffer(&mut buffer)
                .map_err(|error| Error::new(format!("failed to map dumb buffer: {error}")))?;
            fill_gradient(
                mapping.as_mut(),
                output.mode.size().0 as usize,
                output.mode.size().1 as usize,
                buffer.pitch() as usize,
                phase,
            );
            phase = phase.wrapping_add(3);
            thread::sleep(Duration::from_millis(150));
        }

        #[allow(unreachable_code)]
        {
            drop(restore);
            Ok(())
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
    path: PathBuf,
}

impl Card {
    fn new(fd: OwnedFd, path: PathBuf) -> Self {
        Self {
            file: File::from(fd),
            path,
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
    framebuffer: Option<drm::control::framebuffer::Handle>,
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

fn fill_gradient(bytes: &mut [u8], width: usize, height: usize, pitch: usize, phase: u8) {
    for y in 0..height {
        let row = &mut bytes[y * pitch..(y * pitch) + (width * 4)];
        for x in 0..width {
            let offset = x * 4;
            row[offset] = (x as u8).wrapping_add(phase);
            row[offset + 1] = (y as u8).wrapping_add(phase.wrapping_mul(2));
            row[offset + 2] = phase.wrapping_mul(3);
            row[offset + 3] = 0;
        }
    }
}
