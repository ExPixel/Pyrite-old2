mod audio;
mod debuggerui;
mod gbaui;
mod glutil;
mod pyrite_window;

use std::path::{Path, PathBuf};

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
    gba: Box<GbaWindow>,
    debugger: Option<Box<DebuggerWindow>>,
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
            gba: Box::new(
                GbaWindow::new(config, window, gba).context("error while creating GBA window")?,
            ),
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
            f(&mut *self.gba);
        } else if self.is_debugger(id) {
            if let Some(ref mut debugger) = self.debugger {
                f(&mut **debugger)
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
        self.debugger = Some(Box::new(
            DebuggerWindow::new(self.gba_handle.clone(), window)
                .context("error while creating debugger window")?,
        ));

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

    let args = parse_args().context("error occurred while parsing arguments")?;

    let gba = pyrite::GbaHandle::new();

    let mut skip_frames = args.skip_to_frame.unwrap_or(0);
    let skipping_frames = skip_frames > 0;
    if skip_frames > 0 {
        gba.after_frame_wait(|_gba, state| {
            state.target_fps = 10000.0;
        });

        gba.on_frame(move |gba, state| {
            skip_frames -= 1;

            let render_frame = skip_frames <= 1 || (state.frame_count() % 2 == 0);
            gba.video_mut().set_skip_render(!render_frame);

            if skip_frames == 0 {
                state.target_fps = 60.0;
                state.paused = args.pause_on_startup;
                state.remove_callback();
            }
        });
    }

    let rom = std::fs::read(&args.rom)
        .with_context(|| format!("failed to read ROM path `{}`", args.rom.display()))?;

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

    if skipping_frames || !args.pause_on_startup {
        windows.gba_handle.set_paused(false);
    }

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

fn parse_args() -> anyhow::Result<Args> {
    use clap::{Arg, Command};
    use std::str::FromStr;

    let rom_arg = Arg::new("ROM").takes_value(true).required(true).index(1);
    let skip_to_frame_arg = Arg::new("skip-to-frame")
        .short('F')
        .takes_value(true)
        .long("skip-to-frame")
        .help("Skip to the given frame on startup.");
    let pause_on_startup = Arg::new("pause-on-startup")
        .short('P')
        .takes_value(false)
        .long("pause-on-startup")
        .help("Pause the emulator on startup. If `skip to frame` is set, this will pause after skipping.");

    let matches = Command::new("pyrite")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(rom_arg)
        .arg(skip_to_frame_arg)
        .arg(pause_on_startup)
        .get_matches();

    let rom: PathBuf = if let Some(rom_path) = matches.value_of("ROM") {
        Path::new(rom_path).into()
    } else {
        unreachable!("no ROM path provided");
    };

    let skip_to_frame = matches
        .value_of("skip-to-frame")
        .map(u32::from_str)
        .transpose()
        .context("skip-to-frame must be a valid integer")?;
    let pause_on_startup = matches.is_present("pause-on-startup");

    Ok(Args {
        rom,
        skip_to_frame,
        pause_on_startup,
    })
}

struct Args {
    rom: PathBuf,
    skip_to_frame: Option<u32>,
    pause_on_startup: bool,
}
