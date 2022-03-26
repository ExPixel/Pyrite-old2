use anyhow::Context as _;
use gba::{Command, GbaAudioSampler};
use pyrite::GbaHandle;
use rodio::{OutputStream, Sink, Source};
use std::sync::Arc;
use util::spinlock::SpinLock;

pub fn run(gba: GbaHandle) -> anyhow::Result<OutputStream> {
    let (stream, stream_handle) =
        rodio::OutputStream::try_default().context("error while getting default output stream")?;
    let output = Sink::try_new(&stream_handle).context("failed to create audio sink")?;

    let commands_buffer: Arc<SpinLock<Vec<Command>>> = Arc::default();
    let sound_source = GbaSoundSource::new(Arc::clone(&commands_buffer));
    stream_handle
        .play_raw(sound_source)
        .context("error occurred while playing GBA sound source")?;

    gba.on_frame(move |gba, state| {
        let output_is_paused = output.is_paused();

        if state.paused {
            if !output_is_paused {
                output.pause();
            }
            return;
        }

        {
            let gba_audio_commands = gba.audio().commands();
            let mut commands_buffer = commands_buffer.lock();
            commands_buffer.clear();
            if gba_audio_commands.is_empty() {
                commands_buffer.push(Command::Wait(0));
            } else {
                commands_buffer.extend(gba_audio_commands.iter().copied().rev());
            }
        }

        if output_is_paused {
            output.play();
        }
    });
    Ok(stream)
}

struct GbaSoundSource {
    right_channel_sample: Option<f32>,
    sampler: GbaAudioSampler,
    commands: Vec<Command>,
    commands_buffer: Arc<SpinLock<Vec<Command>>>,
}

impl GbaSoundSource {
    fn new(commands_buffer: Arc<SpinLock<Vec<Command>>>) -> Self {
        GbaSoundSource {
            right_channel_sample: None,
            sampler: GbaAudioSampler::new(32768),
            commands: Vec::new(),
            commands_buffer,
        }
    }

    fn try_refill_buffer(&mut self) {
        if let Some(mut commands_buffer) = self.commands_buffer.try_lock() {
            if !commands_buffer.is_empty() {
                self.commands.extend(commands_buffer.drain(0..));
            }
        }
    }
}

impl Iterator for GbaSoundSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(right_sample) = self.right_channel_sample.take() {
            return Some(right_sample * 0.05);
        }

        if self.commands.is_empty() {
            self.try_refill_buffer();
        }

        while self.sampler.needs_commands() {
            if let Some(command) = self.commands.pop() {
                self.sampler.command(command);
            } else {
                break;
            }
        }

        let frame = self.sampler.frame();
        self.right_channel_sample = Some(frame.1);
        Some(frame.0 * 0.05)
    }
}

impl Source for GbaSoundSource {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        2
    }

    fn sample_rate(&self) -> u32 {
        32768
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}
