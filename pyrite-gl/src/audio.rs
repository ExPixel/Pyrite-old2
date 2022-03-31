use anyhow::Context as _;
use cpal::{
    traits::{DeviceTrait as _, HostTrait as _, StreamTrait as _},
    Stream,
};
use crossbeam::queue::SegQueue;
use gba::{Command, GbaAudioSampler};
use pyrite::GbaHandle;
use std::sync::Arc;

const COMMANDS_CHUNK_SIZE: usize = 64;

pub fn run(gba: GbaHandle) -> anyhow::Result<Stream> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or_else(|| anyhow::Error::msg("no default output device"))?;
    let config = device
        .default_output_config()
        .context("error retrieving default output configuration")?;
    let commands_buffer_queue: Arc<SegQueue<[Command; COMMANDS_CHUNK_SIZE]>> =
        Arc::new(SegQueue::new());

    let mut sound_source =
        GbaSoundSource::new(config.sample_rate().0, Arc::clone(&commands_buffer_queue));
    let channels = config.channels() as usize;
    let stream = device
        .build_output_stream(
            &config.into(),
            move |data: &mut [f32], _| sound_source.output(channels, data),
            |err| {
                log::error!("audio stream error: {err}");
            },
        )
        .context("failed to build output stream")?;
    stream.play().context("failed to play audio stream")?;

    let mut last_chunk_count = 0usize;
    gba.on_frame(move |gba, _| {
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
    });
    Ok(stream)
}

struct GbaSoundSource {
    sampler: GbaAudioSampler,
    commands: Box<[Command; COMMANDS_CHUNK_SIZE]>,
    commands_idx: usize,
    commands_buffer: Arc<SegQueue<[Command; COMMANDS_CHUNK_SIZE]>>,
}

impl GbaSoundSource {
    fn new(frequency: u32, commands_buffer: Arc<SegQueue<[Command; COMMANDS_CHUNK_SIZE]>>) -> Self {
        GbaSoundSource {
            sampler: GbaAudioSampler::new(frequency),
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

    fn output(&mut self, channels: usize, samples: &mut [f32]) {
        samples.chunks_exact_mut(channels).for_each(|frame| {
            while self.sampler.needs_commands() {
                if let Some(command) = self.next_command() {
                    self.sampler.command(command);
                } else {
                    break;
                }
            }

            if channels >= 2 {
                (frame[0], frame[1]) = self.sampler.frame(0.15);
            } else {
                let (left, right) = self.sampler.frame(0.15);
                frame[0] = (left + right) / 2.0;
            }

            if channels > 2 {
                frame[2..].fill(0.0);
            }
        });
    }
}
