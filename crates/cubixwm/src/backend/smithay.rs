use crate::utils::{Error, Result};
use smithay::{
    backend::{
        renderer::{Color32F, Frame, Renderer, gles::GlesRenderer},
        winit::{self, WinitEvent},
    },
    reexports::{
        calloop::EventLoop,
        wayland_server::Display,
        winit::{platform::pump_events::PumpStatus, window::Window},
    },
    utils::{Rectangle, Transform},
};

pub struct SmithayBackend {
    initialized: bool,
    display_created: bool,
    event_loop_created: bool,
}

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
        let _event_loop: EventLoop<'static, ()> =
            EventLoop::try_new().expect("failed to create calloop event loop");

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
        let attributes = Window::default_attributes()
            .with_title("CubixWM")
            .with_inner_size(smithay::reexports::winit::dpi::LogicalSize::new(
                1280.0, 800.0,
            ));

        let (mut backend, mut event_loop) = winit::init_from_attributes::<GlesRenderer>(attributes)
            .map_err(|error| {
                Error::new(format!(
                    "failed to initialize smithay winit backend: {error}"
                ))
            })?;

        loop {
            let status = event_loop.dispatch_new_events(|event| match event {
                WinitEvent::Resized { .. } | WinitEvent::Redraw | WinitEvent::Focus(_) => {}
                WinitEvent::Input(_) => {}
                WinitEvent::CloseRequested => {}
            });

            match status {
                PumpStatus::Continue => {}
                PumpStatus::Exit(_) => break,
            }

            let size = backend.window_size();
            let damage = Rectangle::from_size(size);
            {
                let (renderer, mut framebuffer) = backend
                    .bind()
                    .map_err(|error| Error::new(format!("failed to bind renderer: {error}")))?;

                let mut frame = renderer
                    .render(&mut framebuffer, size, Transform::Flipped180)
                    .map_err(|error| Error::new(format!("failed to start frame: {error}")))?;

                frame
                    .clear(Color32F::new(0.08, 0.09, 0.11, 1.0), &[damage])
                    .map_err(|error| Error::new(format!("failed to clear frame: {error}")))?;

                let _ = frame
                    .finish()
                    .map_err(|error| Error::new(format!("failed to finish frame: {error}")))?;
            }

            backend
                .submit(Some(&[damage]))
                .map_err(|error| Error::new(format!("failed to submit frame: {error}")))?;
        }

        Ok(())
    }
}

impl Default for SmithayBackend {
    fn default() -> Self {
        Self::new()
    }
}
