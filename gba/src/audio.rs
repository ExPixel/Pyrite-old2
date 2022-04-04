pub mod sampler;

use crate::{
    dma,
    memory::io::{
        DutyLenEnvelope, FifoChannel, FreqControl, IoRegisters, PSGChannel, Resolution,
        SweepControl, Timing,
    },
    Gba,
};

#[derive(Default)]
pub struct GbaAudio {
    commands: Vec<Command>,
    last_update_time: u64,
}

impl GbaAudio {
    pub fn clear(&mut self, now: u64) {
        self.commands.clear();
        self.last_update_time = now;
    }

    pub fn commands(&self) -> &[Command] {
        &self.commands
    }

    fn set_psg_sweep_control(&mut self, value: SweepControl, ioregs: &IoRegisters) {
        self.wait(ioregs.time);
        self.commands.push(Command::SetPSGSweepControl(value));
    }

    fn set_psg_duty_len_env(
        &mut self,
        chan: PSGChannel,
        value: DutyLenEnvelope,
        ioregs: &IoRegisters,
    ) {
        self.wait(ioregs.time);
        self.commands
            .push(Command::SetPSGDutyLenEnvelope(chan, value));
    }

    fn set_psg_freq_control(&mut self, chan: PSGChannel, value: FreqControl, ioregs: &IoRegisters) {
        self.wait(ioregs.time);
        self.commands.push(Command::SetPSGFreqControl(chan, value));
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
    gba.audio
        .set_psg_sweep_control(gba.mem.ioregs.sound1cnt_l, &gba.mem.ioregs);
}

pub fn psg_duty_len_env_changed<const PSG: u32>(gba: &mut Gba) {
    let (channel, reg) = match PSG {
        1 => (PSGChannel::Sound1, gba.mem.ioregs.sound1cnt_h),
        2 => (PSGChannel::Sound2, gba.mem.ioregs.sound2cnt_l),
        _ => unreachable!(),
    };
    gba.audio
        .set_psg_duty_len_env(channel, reg, &gba.mem.ioregs);
}

pub fn psg_freq_control_changed<const PSG: u32>(gba: &mut Gba) {
    let (channel, reg) = match PSG {
        1 => (PSGChannel::Sound1, gba.mem.ioregs.sound1cnt_x),
        2 => (PSGChannel::Sound2, gba.mem.ioregs.sound2cnt_h),
        _ => unreachable!(),
    };
    gba.audio
        .set_psg_freq_control(channel, reg, &gba.mem.ioregs);
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

    SetPSGSweepControl(SweepControl),
    SetPSGDutyLenEnvelope(PSGChannel, DutyLenEnvelope),
    SetPSGFreqControl(PSGChannel, FreqControl),
}
