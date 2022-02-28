use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

use anyhow::{Context as _, Error};
use sdl2::{
    event::Event,
    pixels::{Color, PixelFormatEnum},
};

const LOCK_FPS: bool = false;

fn main() {
    if let Err(err) = run() {
        eprintln!("{:#}", err);
        std::process::exit(1);
    }
    std::process::exit(0)
}

fn run() -> anyhow::Result<()> {
    pretty_env_logger::formatted_builder()
        .parse_filters(&std::env::var("RUST_LOG").unwrap_or_else(|_| String::from("info")))
        .try_init()
        .context("failed to initiaize logger")?;

    let config = pyrite::config::from_toml_path("pyrite.toml")
        .map_err(|err| {
            let err = anyhow::Error::from(err);
            log::error!("{:#}", err);
        })
        .unwrap_or_default();

    let frame_delay_target = std::time::Duration::from_secs_f64(1.0 / 60.0);

    let sdl_context = sdl2::init()
        .map_err(Error::msg)
        .context("failed to initialize SDL")?;
    let video_subsystem = sdl_context
        .video()
        .map_err(Error::msg)
        .context("failed to initialize SDL video")?;
    let window = video_subsystem
        .window("Pyrite", 480, 320)
        .position_centered()
        .resizable()
        .build()
        .context("failed to create SDL window")?;
    let mut event_pump = sdl_context
        .event_pump()
        .map_err(Error::msg)
        .context("failed to initialize SDL event pump")?;
    let mut canvas_builder = window.into_canvas().accelerated();

    if config.graphics.vsync.unwrap_or(true) {
        canvas_builder = canvas_builder.present_vsync();
    }

    let mut canvas = canvas_builder
        .build()
        .context("failed to initialize SDL canvas")?;
    let texture_creator = canvas.texture_creator();

    let mut gba_frame_texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::BGR555, 240, 160)
        .context("failed to create GBA frame texture")?;
    gba_frame_texture
        .with_lock(None, |buffer: &mut [u8], _: usize| buffer.fill(0))
        .map_err(Error::msg)
        .context("failed to update GBA frame texture")?;
    let screen_buffer = Arc::new(Mutex::new([0u16; gba::SCREEN_PIXEL_COUNT]));

    let gba = pyrite::GbaHandle::new();

    let rom = get_rom_from_args().context("error occurred retrieving ROM from args")?;
    gba.after_frame_wait(move |gba, _| {
        gba.set_gamepak(rom);
        gba.reset();
    });

    let screen = screen_buffer.clone();
    gba.on_frame(move |gba, _| {
        let mut screen = screen.lock().expect("failed to lock screen buffer");
        screen.copy_from_slice(gba.video().buffer());
    });
    gba.set_paused(false);

    let mut fps_counter = FPSCounter::default();

    log::trace!("starting SDL loop...");
    canvas.set_draw_color(Color::RGB(255, 0, 255));
    'main_loop: loop {
        let frame_start_time = Instant::now();

        for event in event_pump.poll_iter() {
            if let Event::Quit { .. } = event {
                log::trace!("exiting SDL loop...");
                break 'main_loop;
            }
        }

        gba_frame_texture
            .with_lock(None, |buffer: &mut [u8], pitch: usize| {
                let screen = screen_buffer.lock().expect("failed to lock screen buffer");
                for y in 0..160 {
                    for x in 0..240 {
                        let offset_src = y * 240 + x;
                        let offset_dst = y * pitch + x * 2;

                        buffer[offset_dst] = screen[offset_src] as u8;
                        buffer[offset_dst + 1] = (screen[offset_src] >> 8) as u8;
                    }
                }
            })
            .map_err(Error::msg)
            .context("failed to update GBA frame texture")?;

        canvas.clear();
        canvas
            .copy(&gba_frame_texture, None, None)
            .map_err(Error::msg)
            .context("failed to copy GBA frame texture to canvas")?;
        canvas.present();

        let frame_end_time = Instant::now();
        let frame_duration = frame_end_time.duration_since(frame_start_time);
        if frame_duration < frame_delay_target && LOCK_FPS {
            let grace = std::time::Duration::from_millis(1);
            std::thread::sleep((frame_delay_target - frame_duration).saturating_sub(grace));
        }

        if let Some(fps) = fps_counter.count(frame_end_time) {
            let title = format!("Pyrite ({:.1} FPS)", fps);
            canvas
                .window_mut()
                .set_title(&title)
                .context("failed to set window title")?;
        }
    }

    gba.set_paused(true);
    gba.after_frame_wait(|gba, _| {
        log::debug!(
            "end address: 0x{:08X} (THUMB: {})",
            gba.cpu().next_exec_pc(),
            gba.cpu().registers.getf_t()
        );
    });

    log::info!("exiting...");

    Ok(())
}

fn get_rom_from_args() -> anyhow::Result<Vec<u8>> {
    let rom_path = std::env::args()
        .nth(1)
        .map(std::path::PathBuf::from)
        .ok_or_else(|| anyhow::anyhow!("expected ROM path as first argument"))?;
    std::fs::read(&rom_path)
        .with_context(|| format!("failed to read ROM path `{}`", rom_path.display()))
}

#[derive(Default)]
struct FPSCounter {
    start_time: Option<std::time::Instant>,
    frames: u32,
}

impl FPSCounter {
    pub fn count(&mut self, now: std::time::Instant) -> Option<f64> {
        self.frames += 1;
        if self.start_time.is_none() {
            self.start_time = Some(now);
            return None;
        }

        let elapsed = now.duration_since(self.start_time.unwrap());
        if elapsed < std::time::Duration::from_secs(1) {
            return None;
        }
        self.start_time = Some(now);

        let seconds = elapsed.as_secs_f64();
        let fps = self.frames as f64 / seconds;
        self.frames = 0;

        Some(fps)
    }
}
