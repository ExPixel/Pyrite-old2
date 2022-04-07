pub mod sampler;

use crate::{
    dma,
    memory::io::{Direction, FifoChannel, IoRegisters, PSGChannel, Resolution, Timing},
    scheduler::{EventTag, Scheduler},
    Gba,
};

#[derive(Default)]
pub struct GbaAudio {
    scheduler: Scheduler,
    commands: Vec<Command>,
    last_update_time: u64,
    psg_envelope_volumes: [u16; 4],
}

impl GbaAudio {
    pub fn new(scheduler: Scheduler) -> GbaAudio {
        GbaAudio {
            scheduler,
            commands: Vec::with_capacity(1024),
            last_update_time: 0,
            psg_envelope_volumes: [0; 4],
        }
    }

    pub fn clear(&mut self, now: u64) {
        self.commands.clear();
        self.last_update_time = now;
    }

    pub fn commands(&self) -> &[Command] {
        &self.commands
    }

    fn stop_psg(&mut self, chan: PSGChannel, ioregs: &mut IoRegisters) {
        self.unschedule_psg_events(chan);
        ioregs.soundcnt_x.set_sound_on(chan, false);
        self.wait(ioregs.time);
        self.commands.push(Command::SetPSGEnabled(chan, false));
    }

    fn psg_length_end(&mut self, chan: PSGChannel, ioregs: &mut IoRegisters) {
        self.stop_psg(chan, ioregs);
    }

    fn psg_envelope_step(&mut self, chan: PSGChannel, ioregs: &IoRegisters) {
        use PSGChannel::*;

        let direction = match chan {
            Sound1 => ioregs.sound1cnt_h.envelope_direction(),
            Sound2 => ioregs.sound2cnt_l.envelope_direction(),
            Sound3 => unreachable!("sound 3 envelope step"),
            Sound4 => ioregs.sound4cnt_l.envelope_direction(),
        };
        let volume_index = u16::from(chan) as usize;

        let reschedule = if direction == Direction::Increasing {
            self.psg_envelope_volumes[volume_index] += 1;
            self.psg_envelope_volumes[volume_index] < 15
        } else {
            self.psg_envelope_volumes[volume_index] -= 1;
            self.psg_envelope_volumes[volume_index] > 0
        };

        self.wait(ioregs.time);
        self.commands.push(Command::SetPSGEnvelopeVolume(
            chan,
            self.psg_envelope_volumes[volume_index],
        ));

        if reschedule {
            self.schedule_psg_envelope_step(chan, ioregs);
        }
    }

    fn psg_sweep_step(&mut self, ioregs: &mut IoRegisters) {
        let delta = ioregs.sound1cnt_x.freq_setting() >> ioregs.sound1cnt_l.shifts();
        let frate = if ioregs.sound1cnt_l.direction() == Direction::Increasing {
            ioregs.sound1cnt_x.freq_setting().saturating_add(delta)
        } else {
            ioregs.sound1cnt_x.freq_setting().saturating_sub(delta)
        };

        if frate >= 2048 {
            self.stop_psg(PSGChannel::Sound1, ioregs);
        } else if (frate as i16) >= 0 && frate != ioregs.sound1cnt_x.freq_setting() {
            ioregs.sound1cnt_x.set_freq_setting(frate);
            self.wait(ioregs.time);
            self.commands
                .push(Command::SetSquareFrequencyRate(PSGChannel::Sound1, frate));
            self.schedule_psg_sweep_step(ioregs);
        }
    }

    fn set_psg_sweep_control(&mut self, ioregs: &IoRegisters) {
        if !ioregs.soundcnt_x.master_enable() {
            return;
        }
        self.wait(ioregs.time);
        // FIXME reimplement this
    }

    fn set_psg_noise_len_env(&mut self, _ioregs: &mut IoRegisters) {
        // FIXME for now this is a NOP but eventually length
        // should work like envelopes/sweeps and increment the sound
        // length in the register every 1/256s.
    }

    fn set_psg_wave_len(&mut self, _ioregs: &mut IoRegisters) {
        // FIXME for now this is a NOP but eventually length
        // should work like envelopes/sweeps and increment the sound
        // length in the register every 1/256s.
    }

    fn set_psg_square_duty_len_env(&mut self, chan: PSGChannel, ioregs: &IoRegisters) {
        if !ioregs.soundcnt_x.master_enable() {
            return;
        }
        self.wait(ioregs.time);

        let dle = match chan {
            PSGChannel::Sound1 => ioregs.sound1cnt_h,
            PSGChannel::Sound2 => ioregs.sound2cnt_l,
            _ => unreachable!(),
        };
        self.commands
            .push(Command::SetSquareDuty(chan, dle.wave_pattern_duty()));
    }

    fn unschedule_psg_events(&self, chan: PSGChannel) {
        use PSGChannel::*;

        let tag_envelope_tick = if chan == Sound3 {
            None
        } else {
            Some(EventTag::psg_envelope_tick(chan))
        };
        let tag_sweep_tick = if chan == Sound1 {
            Some(EventTag::SweepTickPSG1)
        } else {
            None
        };
        let tag_length_end = EventTag::psg_length_end(chan);
        self.scheduler.unschedule_matching(|event| {
            event.tag == tag_length_end
                || Some(event.tag) == tag_envelope_tick
                || Some(event.tag) == tag_sweep_tick
        });
    }

    fn schedule_psg_length_end(&mut self, chan: PSGChannel, ioregs: &IoRegisters) {
        use PSGChannel::*;

        const CYCLES_PER_STEP: u32 = Gba::CYCLES_PER_SECOND / 256;
        let length_cycles = match chan {
            Sound1 => (64 - ioregs.sound1cnt_h.length() as u32) * CYCLES_PER_STEP,
            Sound2 => (64 - ioregs.sound2cnt_l.length() as u32) * CYCLES_PER_STEP,
            Sound3 => (256 - ioregs.sound3cnt_h.length() as u32) * CYCLES_PER_STEP,
            Sound4 => (64 - ioregs.sound4cnt_l.length() as u32) * CYCLES_PER_STEP,
        };
        let callback: fn(&mut Gba) = match chan {
            Sound1 => |gba| gba.audio.psg_length_end(Sound1, &mut gba.mem.ioregs),
            Sound2 => |gba| gba.audio.psg_length_end(Sound2, &mut gba.mem.ioregs),
            Sound3 => |gba| gba.audio.psg_length_end(Sound3, &mut gba.mem.ioregs),
            Sound4 => |gba| gba.audio.psg_length_end(Sound4, &mut gba.mem.ioregs),
        };
        self.scheduler
            .schedule(callback, length_cycles, EventTag::psg_length_end(chan));
    }

    fn schedule_psg_envelope_step(&mut self, chan: PSGChannel, ioregs: &IoRegisters) {
        use PSGChannel::*;

        const CYCLES_PER_STEP: u32 = Gba::CYCLES_PER_SECOND / 64;
        let step_cycles = match chan {
            Sound1 => ioregs.sound1cnt_h.envelope_step_time() as u32 * CYCLES_PER_STEP,
            Sound2 => ioregs.sound2cnt_l.envelope_step_time() as u32 * CYCLES_PER_STEP,
            Sound3 => 0,
            Sound4 => ioregs.sound4cnt_l.envelope_step_time() as u32 * CYCLES_PER_STEP,
        };

        if step_cycles == 0 {
            return;
        }

        let callback: fn(&mut Gba) = match chan {
            Sound1 => |gba| gba.audio.psg_envelope_step(Sound1, &gba.mem.ioregs),
            Sound2 => |gba| gba.audio.psg_envelope_step(Sound2, &gba.mem.ioregs),
            Sound3 => panic!("invalid PSG for envelope tick"),
            Sound4 => |gba| gba.audio.psg_envelope_step(Sound4, &gba.mem.ioregs),
        };
        self.scheduler
            .schedule(callback, step_cycles, EventTag::psg_envelope_tick(chan));
    }

    fn schedule_psg_sweep_step(&mut self, ioregs: &IoRegisters) {
        const CYCLES_PER_STEP: u32 = Gba::CYCLES_PER_SECOND / 128;
        let step_cycles = ioregs.sound1cnt_l.sweep_time() as u32 * CYCLES_PER_STEP;
        let callback = |gba: &mut Gba| gba.audio.psg_sweep_step(&mut gba.mem.ioregs);
        self.scheduler
            .schedule(callback, step_cycles, EventTag::SweepTickPSG1);
    }

    fn set_psg_noise_freq_control(&mut self, ioregs: &mut IoRegisters) {
        use PSGChannel::*;

        if !ioregs.soundcnt_x.master_enable() {
            return;
        }

        // Because initial is always set to false here placing this code
        // outside of the conditional below allows these registers to be modified while
        // the sound is running.
        self.wait(ioregs.time);
        self.commands.push(Command::SetNoiseFrequencyParams {
            r: ioregs.sound4cnt_h.r() as u8,
            s: ioregs.sound4cnt_h.s() as u8,
        });
        self.commands
            .push(Command::SetNoiseCounterWidth(ioregs.sound4cnt_h.width()));

        if ioregs.sound4cnt_h.initial() {
            self.commands.push(Command::SetPSGEnvelopeVolume(
                Sound4,
                ioregs.sound4cnt_l.initial_envelope_volume(),
            ));
            self.psg_envelope_volumes[u16::from(Sound4) as usize] =
                ioregs.sound4cnt_l.initial_envelope_volume();
            self.commands.push(Command::SetPSGEnabled(Sound4, true));
            ioregs.soundcnt_x.set_sound_on(Sound4, true);
            ioregs.sound4cnt_h.set_initial(false);
            self.unschedule_psg_events(Sound4);

            if ioregs.sound4cnt_h.length_flag() {
                self.schedule_psg_length_end(Sound4, ioregs);
            }

            if ioregs.sound4cnt_l.envelope_step_time() > 0
                && ioregs.sound4cnt_l.initial_envelope_volume() > 0
            {
                self.schedule_psg_envelope_step(Sound4, ioregs);
            }
        }
    }

    fn set_psg_wave_freq_control(&mut self, ioregs: &mut IoRegisters) {
        if !ioregs.soundcnt_x.master_enable() {
            return;
        }
        log::debug!("set_psg_wave_freq_control");
    }

    fn set_psg_square_freq_control(&mut self, chan: PSGChannel, ioregs: &mut IoRegisters) {
        use PSGChannel::*;

        if !ioregs.soundcnt_x.master_enable() {
            return;
        }

        let (ctl, dle) = match chan {
            Sound1 => (&mut ioregs.sound1cnt_x, ioregs.sound1cnt_h),
            Sound2 => (&mut ioregs.sound2cnt_h, ioregs.sound2cnt_l),
            _ => unreachable!(),
        };

        if ctl.initial() {
            self.wait(ioregs.time);
            self.commands
                .push(Command::SetSquareFrequencyRate(chan, ctl.freq_setting()));
            self.commands.push(Command::SetPSGEnvelopeVolume(
                chan,
                dle.initial_envelope_volume(),
            ));
            self.psg_envelope_volumes[u16::from(chan) as usize] = dle.initial_envelope_volume();
            self.commands.push(Command::SetPSGEnabled(chan, true));
            ioregs.soundcnt_x.set_sound_on(chan, true);
            ctl.set_initial(false);
            self.unschedule_psg_events(chan);

            if ctl.length_flag() {
                self.schedule_psg_length_end(chan, ioregs);
            }

            if dle.envelope_step_time() > 0 && dle.initial_envelope_volume() > 0 {
                self.schedule_psg_envelope_step(chan, ioregs);
            }

            if chan == Sound1
                && ioregs.sound1cnt_l.sweep_time() > 0
                && ioregs.sound1cnt_l.shifts() > 0
            {
                self.schedule_psg_sweep_step(ioregs)
            }
        }
    }

    fn set_resolution(&mut self, resolution: Resolution, ioregs: &IoRegisters) {
        self.wait(ioregs.time);
        self.commands.push(Command::SetResolution(resolution));
    }

    fn set_bias(&mut self, bias: u16, ioregs: &IoRegisters) {
        self.wait(ioregs.time);
        self.commands.push(Command::SetBias(bias));
    }

    fn fifo_play(&mut self, channel: FifoChannel, ioregs: &mut IoRegisters) {
        self.wait(ioregs.time);
        let sample = if channel == FifoChannel::A {
            ioregs.fifo_a.pop_sample()
        } else {
            ioregs.fifo_b.pop_sample()
        } as i8;
        self.commands.push(Command::PlaySample { channel, sample });
    }

    fn wait(&mut self, now: u64) {
        if now > self.last_update_time {
            let elapsed = (now - self.last_update_time) as u32;
            if let Some(Command::Wait(ref mut cycles)) = self.commands.last_mut() {
                *cycles += elapsed;
            } else {
                self.commands.push(Command::Wait(elapsed));
            }
        }
        self.last_update_time = now;
    }
}

pub fn psg_sweep_changed(gba: &mut Gba) {
    gba.audio.set_psg_sweep_control(&gba.mem.ioregs);
}

pub fn psg_nosie_len_env_changed(gba: &mut Gba) {
    gba.audio.set_psg_noise_len_env(&mut gba.mem.ioregs);
}

pub fn psg_wave_len_env_changed(gba: &mut Gba) {
    gba.audio.set_psg_wave_len(&mut gba.mem.ioregs);
}

pub fn psg_duty_len_env_changed<const PSG: u32>(gba: &mut Gba) {
    let channel = match PSG {
        1 => PSGChannel::Sound1,
        2 => PSGChannel::Sound2,
        _ => unreachable!(),
    };
    gba.audio
        .set_psg_square_duty_len_env(channel, &gba.mem.ioregs);
}

pub fn psg_freq_control_changed<const PSG: u32>(gba: &mut Gba) {
    use PSGChannel::*;

    match PSG {
        1 => gba
            .audio
            .set_psg_square_freq_control(Sound1, &mut gba.mem.ioregs),
        2 => gba
            .audio
            .set_psg_square_freq_control(Sound2, &mut gba.mem.ioregs),
        3 => gba.audio.set_psg_wave_freq_control(&mut gba.mem.ioregs),
        4 => gba.audio.set_psg_noise_freq_control(&mut gba.mem.ioregs),
        _ => unreachable!(),
    }
}

pub fn resolution_changed(gba: &mut Gba) {
    gba.audio
        .set_resolution(gba.mem.ioregs.soundbias.resolution(), &gba.mem.ioregs);
}

pub fn bias_changed(gba: &mut Gba) {
    gba.audio
        .set_bias(gba.mem.ioregs.soundbias.bias(), &gba.mem.ioregs);
}

pub fn check_fifo_timer_overflow(timer: usize, gba: &mut Gba) {
    if gba.mem.ioregs.soundcnt_h.dma_enable(FifoChannel::A)
        && gba.mem.ioregs.soundcnt_h.dma_timer_select(FifoChannel::A) == timer
    {
        gba.audio.fifo_play(FifoChannel::A, &mut gba.mem.ioregs);

        if gba.mem.ioregs.fifo_a.len() <= 16 {
            dma::dma_on_timing(gba, Timing::SoundFifo);
        }
    }

    if gba.mem.ioregs.soundcnt_h.dma_enable(FifoChannel::B)
        && gba.mem.ioregs.soundcnt_h.dma_timer_select(FifoChannel::B) == timer
    {
        gba.audio.fifo_play(FifoChannel::B, &mut gba.mem.ioregs);

        if gba.mem.ioregs.fifo_b.len() <= 16 {
            dma::dma_on_timing(gba, Timing::SoundFifo);
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Command {
    Wait(u32),
    PlaySample { channel: FifoChannel, sample: i8 },
    SetResolution(Resolution),
    SetBias(u16),

    SetNoiseFrequencyParams { r: u8, s: u8 },
    SetNoiseCounterWidth(u16),
    SetPSGEnabled(PSGChannel, bool),
    SetSquareFrequencyRate(PSGChannel, u16),
    SetSquareDuty(PSGChannel, u16),
    SetPSGEnvelopeVolume(PSGChannel, u16),
}
