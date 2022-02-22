use anyhow::{Context as _, Error};
use sdl2::{
    event::Event,
    pixels::{Color, PixelFormatEnum},
};

fn main() {
    if let Err(err) = run() {
        eprintln!("{:#}", err);
        std::process::exit(1);
    }
    std::process::exit(0)
}

fn run() -> anyhow::Result<()> {
    pretty_env_logger::formatted_builder()
        .parse_filters("trace")
        .try_init()
        .context("failed to initiaize logger")?;

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
    let mut canvas = window
        .into_canvas()
        .accelerated()
        .present_vsync()
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

    log::trace!("starting SDL loop...");
    canvas.set_draw_color(Color::RGB(255, 0, 255));
    'main_loop: loop {
        for event in event_pump.poll_iter() {
            if let Event::Quit { .. } = event {
                log::trace!("exiting SDL loop...");
                break 'main_loop;
            }
        }

        canvas.clear();

        // if gba_has_frame.load(std::sync::atomic::Ordering::Acquire) {
        //     pixel_buffer = pixel_buffer_swap.swap(pixel_buffer);

        //     gba_frame_texture
        //         .with_lock(None, |buffer: &mut [u8], pitch: usize| {
        //             for y in 0..160 {
        //                 for x in 0..240 {
        //                     let offset_src = y * 240 + x;
        //                     let offset_dst = y * pitch + x * 2;

        //                     buffer[offset_dst] = pixel_buffer[offset_src] as u8;
        //                     buffer[offset_dst + 1] = (pixel_buffer[offset_src] >> 8) as u8;
        //                 }
        //             }
        //         })
        //         .map_err(Error::msg)
        //         .context("failed to update GBA frame texture")?;

        //     canvas
        //         .copy(&gba_frame_texture, None, None)
        //         .map_err(Error::msg)
        //         .context("failed to copy GBA frame texture to canvas")?;
        // }

        canvas
            .copy(&gba_frame_texture, None, None)
            .map_err(Error::msg)
            .context("failed to copy GBA frame texture to canvas")?;

        canvas.present();
    }

    log::info!("exiting...");

    Ok(())
}
