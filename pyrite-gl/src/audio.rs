use anyhow::Context as _;
use crossbeam::queue::SegQueue;
use gba::{Command, GbaAudioSampler};
use pyrite::GbaHandle;
use rodio::{OutputStream, Sink, Source};
use std::sync::Arc;

const COMMANDS_CHUNK_SIZE: usize = 16;

pub fn run(gba: GbaHandle) -> anyhow::Result<OutputStream> {
    let (stream, stream_handle) =
        rodio::OutputStream::try_default().context("error while getting default output stream")?;
    let output = Sink::try_new(&stream_handle).context("failed to create audio sink")?;

    let commands_buffer_queue: Arc<SegQueue<[Command; COMMANDS_CHUNK_SIZE]>> =
        Arc::new(SegQueue::new());

    let sound_source = GbaSoundSource::new(Arc::clone(&commands_buffer_queue));
    stream_handle
        .play_raw(sound_source)
        .context("error occurred while playing GBA sound source")?;

    let mut last_chunk_count = 0usize;
    gba.on_frame(move |gba, state| {
        let output_is_paused = output.is_paused();

        if state.paused {
            if !output_is_paused {
                output.pause();
            }
            return;
        }

        let gba_audio_commands = gba.audio().commands();

        // Check to make sure that we have at most 2 frames worth of commands
        // waiting to be processed. If there are already two, then we just miss this
        // frame of audio commands.
        if commands_buffer_queue.len() <= last_chunk_count {
            last_chunk_count = 0;
            gba_audio_commands
                .chunks(COMMANDS_CHUNK_SIZE)
                .for_each(|src_chunk| {
                    let mut chunk = [Command::Wait(0); COMMANDS_CHUNK_SIZE];
                    (&mut chunk[..src_chunk.len()]).copy_from_slice(src_chunk);
                    commands_buffer_queue.push(chunk);
                    last_chunk_count += 1;
                });
        } else {
            log::debug!("missed frame audio command queue");
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
    commands: Box<[Command; COMMANDS_CHUNK_SIZE]>,
    commands_idx: usize,
    commands_buffer: Arc<SegQueue<[Command; COMMANDS_CHUNK_SIZE]>>,
}

impl GbaSoundSource {
    fn new(commands_buffer: Arc<SegQueue<[Command; COMMANDS_CHUNK_SIZE]>>) -> Self {
        GbaSoundSource {
            right_channel_sample: None,
            sampler: GbaAudioSampler::new(32768),
            commands: Box::new([Command::Wait(0); COMMANDS_CHUNK_SIZE]),
            commands_idx: 0,
            commands_buffer,
        }
    }

    fn try_refill_buffer(&mut self) {
        if let Some(chunk) = self.commands_buffer.pop() {
            *self.commands = chunk;
            self.commands_idx = 0;
        }
    }

    fn next_command(&mut self) -> Option<Command> {
        if self.commands_idx >= COMMANDS_CHUNK_SIZE {
            self.try_refill_buffer();
        }

        if self.commands_idx >= COMMANDS_CHUNK_SIZE {
            None
        } else {
            let idx = self.commands_idx;
            self.commands_idx += 1;
            Some(self.commands[idx])
        }
    }
}

impl Iterator for GbaSoundSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(right_sample) = self.right_channel_sample.take() {
            return Some(right_sample * 0.05);
        }

        while self.sampler.needs_commands() {
            if let Some(command) = self.next_command() {
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
