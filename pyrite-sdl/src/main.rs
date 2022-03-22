use std::{
    collections::VecDeque,
    sync::{
        atomic::{self, AtomicU16, AtomicU32, AtomicU64},
        Arc, Mutex,
    },
    time::Instant,
};

use anyhow::{Context as _, Error};
use crossbeam::queue::SegQueue;
use gba::{memory::io::FifoChannel, Button, ButtonSet, Command};
use sdl2::{
    audio::{AudioCallback, AudioSpec, AudioSpecDesired},
    event::Event,
    keyboard::{Keycode, Mod},
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
    let audio_subsystem = sdl_context
        .audio()
        .map_err(Error::msg)
        .context("failed to initialize SDL audio")?;

    let desired_spec = AudioSpecDesired {
        freq: Some(32768),
        channels: Some(2),
        samples: None,
    };
    let audio_command_queue = Arc::new(SegQueue::new());
    let device = audio_subsystem
        .open_playback(None, &desired_spec, |spec| {
            GbaAudio::new(spec, audio_command_queue.clone())
        })
        .map_err(Error::msg)
        .context("failed to open audio playback")?;
    device.resume();

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
    let bios = config.gba.bios_path.and_then(|path| {
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

    let mut buttons = ButtonSet::default();
    let buttons_u16 = Arc::new(AtomicU16::new(buttons.into()));
    let buttons_u16_gba = buttons_u16.clone();

    let frame_duration_accumulate = Arc::new(AtomicU64::new(0));
    let frame_duration_count = Arc::new(AtomicU32::new(0));

    let frame_duration_accumulate_gba = frame_duration_accumulate.clone();
    let frame_duration_count_gba = frame_duration_count.clone();
    let screen = screen_buffer.clone();
    gba.on_frame(move |gba, state| {
        let buttons = buttons_u16_gba.load(atomic::Ordering::Acquire);
        gba.set_buttons(ButtonSet::from(buttons));

        if !state.paused {
            let mut screen = screen.lock().expect("failed to lock screen buffer");
            screen.copy_from_slice(gba.video().screen());

            let mut audio_commands = [Command::Wait(0); GbaAudio::COMMAND_CHUNK_SIZE];
            gba.audio()
                .commands()
                .chunks(GbaAudio::COMMAND_CHUNK_SIZE)
                .for_each(|chunk| {
                    (&mut audio_commands[0..chunk.len()]).copy_from_slice(chunk);
                    (&mut audio_commands[chunk.len()..]).fill(Command::Wait(0));
                    audio_command_queue.push(audio_commands);
                });
        }

        let frame_duration_us: u64 = state.frame_duration().as_micros().try_into().unwrap();
        frame_duration_accumulate_gba.fetch_add(frame_duration_us, atomic::Ordering::Release);
        frame_duration_count_gba.fetch_add(1, atomic::Ordering::Release);
    });
    gba.set_paused(false);

    let mut fps_counter = FPSCounter::default();

    log::trace!("starting SDL loop...");
    canvas.set_draw_color(Color::RGB(255, 0, 255));
    'main_loop: loop {
        let frame_start_time = Instant::now();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyUp {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    log::trace!("exiting SDL loop...");
                    break 'main_loop;
                }

                Event::KeyDown {
                    keycode: Some(keycode),
                    repeat,
                    keymod,
                    ..
                } => {
                    if let Some(button) = keycode_to_button(keycode) {
                        buttons.set_pressed(button, true)
                    }

                    if !repeat && keymod.intersects(Mod::LCTRLMOD | Mod::RCTRLMOD) {
                        match keycode {
                            Keycode::P => gba.after_frame(|_, ctx| ctx.paused = !ctx.paused),
                            Keycode::R => gba.after_frame(move |gba, _| gba.reset(boot_from_bios)),
                            _ => {}
                        }
                    }
                }

                Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => {
                    if let Some(button) = keycode_to_button(keycode) {
                        buttons.set_pressed(button, false)
                    }
                }

                _ => (),
            }
        }

        buttons_u16.store(buttons.into(), atomic::Ordering::Release);

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
            const GBA_TARGET_FRAME_DURATION_US: f64 = 1000000.0 / 60.0;

            let frame_duration_accumulate_value =
                frame_duration_accumulate.load(atomic::Ordering::Acquire);
            let frame_duration_count_value = frame_duration_count.load(atomic::Ordering::Acquire);
            let average_frame_duration_us =
                frame_duration_accumulate_value as f64 / frame_duration_count_value as f64;
            frame_duration_accumulate
                .fetch_sub(frame_duration_accumulate_value, atomic::Ordering::Release);
            frame_duration_count.fetch_sub(frame_duration_count_value, atomic::Ordering::Release);

            let gba_average_frame_duration = average_frame_duration_us / 1000.0;
            let gba_performance_percentage =
                (GBA_TARGET_FRAME_DURATION_US / average_frame_duration_us) * 100.0;

            let title = format!(
                "Pyrite ({:.1} FPS) (GBA: {:.1} ms | {:.1} %)",
                fps, gba_average_frame_duration, gba_performance_percentage
            );
            canvas
                .window_mut()
                .set_title(&title)
                .context("failed to set window title")?;
        }
    }

    device.pause();
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

#[allow(dead_code)]
pub struct GbaAudio {
    spec: AudioSpec,
    commands: VecDeque<Command>,
    command_buffer_queue: Arc<SegQueue<[Command; Self::COMMAND_CHUNK_SIZE]>>,
    fifo_a: f32,
    fifo_b: f32,

    /// The number of samples to output before processing the next command.
    wait_frames: usize,
}

impl GbaAudio {
    pub const COMMAND_CHUNK_SIZE: usize = 128;

    fn new(spec: AudioSpec, queue: Arc<SegQueue<[Command; Self::COMMAND_CHUNK_SIZE]>>) -> Self {
        log::debug!("audio.frequency = {}hz", spec.freq);
        log::debug!(" audio.channels = {}", spec.channels);

        GbaAudio {
            spec,
            commands: VecDeque::with_capacity(64),
            command_buffer_queue: queue,
            fifo_a: 0.0,
            fifo_b: 0.0,
            wait_frames: 0,
        }
    }

    fn next_samples(&mut self) -> (f32, f32) {
        (self.fifo_a, self.fifo_b)
    }

    fn run_commands(&mut self) {
        while let Some(command) = self.next_command() {
            match command {
                Command::Wait(cycles) if cycles != 0 => {
                    self.wait_cycles(cycles);
                    break;
                }
                Command::Wait(_) => { /* 0 cycles = NOP */ }

                Command::PlaySample(fifo, sample) => {
                    const CONVERT: f32 = 1.0 / 128.0;
                    let sample: f32 = (sample as f32 * CONVERT).clamp(-1.0, 1.0);
                    if fifo == FifoChannel::A {
                        self.fifo_a = sample;
                    } else {
                        self.fifo_b = sample;
                    }
                }
            }
        }
    }

    fn wait_cycles(&mut self, cycles: u32) {
        const GBA_FREQ_RECIP: f64 = 1.0 / (16.0 * 1024.0 * 1024.0);
        let freq = self.spec.freq as f64;
        let cycles = cycles as f64;
        self.wait_frames = (cycles * freq * GBA_FREQ_RECIP) as usize;
    }

    fn next_command(&mut self) -> Option<Command> {
        if self.commands.is_empty() {
            if let Some(new_commands) = self.command_buffer_queue.pop() {
                self.commands.extend(new_commands);
            }
        }
        self.commands.pop_front()
    }
}

impl AudioCallback for GbaAudio {
    type Channel = f32;

    fn callback(&mut self, out: &mut [Self::Channel]) {
        out.chunks_exact_mut(2).for_each(|frame| {
            if self.wait_frames == 0 {
                self.run_commands();
            }
            self.wait_frames = self.wait_frames.saturating_sub(1);

            let (left, right) = self.next_samples();
            frame[0] = left * 0.02;
            frame[1] = right * 0.02;
        });
    }
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

fn keycode_to_button(keycode: sdl2::keyboard::Keycode) -> Option<Button> {
    match keycode {
        sdl2::keyboard::Keycode::Z => Some(Button::A),
        sdl2::keyboard::Keycode::X => Some(Button::B),
        sdl2::keyboard::Keycode::Left => Some(Button::Left),
        sdl2::keyboard::Keycode::Right => Some(Button::Right),
        sdl2::keyboard::Keycode::Up => Some(Button::Up),
        sdl2::keyboard::Keycode::Down => Some(Button::Down),
        sdl2::keyboard::Keycode::A => Some(Button::L),
        sdl2::keyboard::Keycode::S => Some(Button::R),
        sdl2::keyboard::Keycode::Return => Some(Button::Start),
        sdl2::keyboard::Keycode::Backspace => Some(Button::Select),
        _ => None,
    }
}
