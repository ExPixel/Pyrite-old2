mod audio;
mod debuggerui;
mod gbaui;
mod glutil;
mod pyrite_window;

use anyhow::Context as _;
use debuggerui::DebuggerWindow;
use gbaui::GbaWindow;
use glutin::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
    window::{WindowBuilder, WindowId},
    PossiblyCurrent, WindowedContext,
};
use pyrite::{config::Config, GbaHandle};
use pyrite_window::PyriteWindow;

struct Windows {
    gba_handle: GbaHandle,
    gba: GbaWindow,
    debugger: Option<DebuggerWindow>,
    config: std::sync::Arc<Config>,
}

impl Windows {
    fn new(
        config: std::sync::Arc<Config>,
        window: WindowedContext<PossiblyCurrent>,
        gba: GbaHandle,
    ) -> anyhow::Result<Windows> {
        Ok(Windows {
            config: config.clone(),
            gba_handle: gba.clone(),
            gba: GbaWindow::new(config, window, gba).context("error while creating GBA window")?,
            debugger: None,
        })
    }

    fn main_events_cleared(&mut self) {
        if self.gba.update() {
            self.gba.request_redraw();
        }

        if let Some(ref mut debugger) = self.debugger {
            if debugger.update() {
                debugger.request_redraw();
            }
        }
    }

    fn is_gba(&self, id: WindowId) -> bool {
        self.gba.window_id() == id
    }

    fn is_debugger(&self, id: WindowId) -> bool {
        self.debugger
            .as_ref()
            .filter(|d| d.window_id() == id)
            .is_some()
    }

    fn with_window<F>(&mut self, id: WindowId, f: F)
    where
        F: FnOnce(&mut dyn PyriteWindow),
    {
        if self.is_gba(id) {
            f(&mut self.gba);
        } else if self.is_debugger(id) {
            if let Some(ref mut debugger) = self.debugger {
                f(debugger)
            }
        }
    }

    fn create_debugger_window(&mut self, el: &EventLoopWindowTarget<()>) -> anyhow::Result<()> {
        if self.debugger.is_some() {
            return Ok(());
        }

        let window = WindowBuilder::new()
            .with_title("Pyrite Debugger")
            .with_inner_size(LogicalSize::new(640.0f32, 320.0));
        let window = unsafe {
            glutin::ContextBuilder::new()
                .with_vsync(self.config.graphics.vsync.unwrap_or(true))
                .build_windowed(window, el)
                .context("failed to create windowed context")?
                .make_current()
                .map_err(|err| anyhow::anyhow!("{:?}", err))
                .context("failed to make window current")?
        };
        self.debugger = Some(
            DebuggerWindow::new(self.gba_handle.clone(), window)
                .context("error while creating debugger window")?,
        );

        Ok(())
    }

    fn close_debugger(&mut self) {
        self.debugger = None;
    }
}

fn on_window_event(
    event: WindowEvent,
    el: &EventLoopWindowTarget<()>,
    id: WindowId,
    windows: &mut Windows,
    flow: &mut ControlFlow,
) -> anyhow::Result<()> {
    match event {
        WindowEvent::CloseRequested if windows.is_gba(id) => {
            windows.with_window(id, |window| window.process_window_event(event));
            *flow = ControlFlow::Exit;
        }

        WindowEvent::CloseRequested if windows.is_debugger(id) => {
            windows.with_window(id, |window| window.process_window_event(event));
            windows.close_debugger();
        }

        event => {
            windows.with_window(id, |window| window.process_window_event(event));

            if windows.is_gba(id) {
                if windows.gba.wants_exit() {
                    *flow = ControlFlow::Exit;
                }

                if windows.gba.wants_debugger() {
                    if let Err(err) = windows.create_debugger_window(el) {
                        log::error!("error while creating debugger window: {err:?}");
                    }
                }
            } else if windows.is_debugger(id)
                && windows.debugger.as_ref().map(|d| d.wants_close()) == Some(true)
            {
                windows.close_debugger();
            }
        }
    }
    Ok(())
}

fn on_event(
    windows: &mut Windows,
    event: Event<()>,
    el: &EventLoopWindowTarget<()>,
    flow: &mut ControlFlow,
) -> anyhow::Result<()> {
    *flow = ControlFlow::Wait;

    match event {
        Event::WindowEvent { event, window_id } => {
            on_window_event(event, el, window_id, windows, flow)?
        }
        Event::RedrawRequested(window_id) => {
            windows.with_window(window_id, |window| window.gl_render(flow));
        }
        Event::MainEventsCleared => windows.main_events_cleared(),
        _ => *flow = ControlFlow::Poll,
    }
    Ok(())
}

fn run(event_loop: EventLoop<()>) -> anyhow::Result<()> {
    let config = std::sync::Arc::new(
        pyrite::config::from_toml_path("pyrite.toml")
            .map_err(|err| {
                let err = anyhow::Error::from(err);
                log::error!("{:#}", err);
            })
            .unwrap_or_default(),
    );

    let gba = pyrite::GbaHandle::new();
    let rom = get_rom_from_args().context("error occurred retrieving ROM from args")?;
    let bios = config.gba.bios_path.as_ref().and_then(|path| {
        match std::fs::read(&path)
            .with_context(|| format!("failed to read path {}", path.display()))
        {
            Ok(data) => Some(data),
            Err(err) => {
                log::error!("error occurred while loading BIOS: {:?}", err);
                None
            }
        }
    });
    let boot_from_bios = config.gba.boot_from_bios.unwrap_or(true);
    gba.after_frame_wait(move |gba, _| {
        gba.set_gamepak(rom);
        gba.set_bios(bios);
        gba.reset(boot_from_bios);
    });
    let _stream = audio::run(gba.clone()).context("error while starting audio")?;

    let window = WindowBuilder::new()
        .with_title("Pyrite")
        .with_inner_size(LogicalSize::new(480.0f32, 320.0));
    let window = unsafe {
        glutin::ContextBuilder::new()
            .with_vsync(config.graphics.vsync.unwrap_or(true))
            .build_windowed(window, &event_loop)
            .context("failed to create windowed context")?
            .make_current()
            .map_err(|err| anyhow::anyhow!("{:?}", err))
            .context("failed to make window current")?
    };
    let mut windows = Windows::new(config, window, gba).context("error initializing windows")?;

    windows.gba_handle.set_paused(false);
    event_loop.run(move |event, el, control_flow| {
        if let Err(err) = on_event(&mut windows, event, el, control_flow) {
            log::error!("error occurred in event loop: {:?}", err);
            *control_flow = ControlFlow::Exit;
        }
    });
}

fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();
    let event_loop = EventLoop::new();

    run(event_loop).context("error occurred while running main loop")?;
    unreachable!("run should never return unless there is an error");
}

fn get_rom_from_args() -> anyhow::Result<Vec<u8>> {
    let rom_path = std::env::args()
        .nth(1)
        .map(std::path::PathBuf::from)
        .ok_or_else(|| anyhow::anyhow!("expected ROM path as first argument"))?;
    std::fs::read(&rom_path)
        .with_context(|| format!("failed to read ROM path `{}`", rom_path.display()))
}
