use crate::{
    memory::io::{FifoChannel, IoRegisters},
    Gba,
};

#[derive(Default)]
pub struct GbaAudio {
    commands: Vec<Command>,
    last_update_time: u64,
}

impl GbaAudio {
    pub fn clear(&mut self) {
        self.commands.clear();
    }

    pub fn commands(&self) -> &[Command] {
        &self.commands
    }

    fn fifo_play(&mut self, chan: FifoChannel, ioregs: &mut IoRegisters) {
        self.wait(ioregs.time);
        let sample = if chan == FifoChannel::A {
            ioregs.fifo_a.pop_sample()
        } else {
            ioregs.fifo_b.pop_sample()
        };
        self.commands.push(Command::PlaySample(chan, sample as i8));
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

pub fn check_fifo_timer_overflow(timer: usize, gba: &mut Gba) {
    if gba.mem.ioregs.soundcnt_h.dma_enable(FifoChannel::A)
        && gba.mem.ioregs.soundcnt_h.dma_timer_select(FifoChannel::A) == timer
    {
        gba.audio.fifo_play(FifoChannel::A, &mut gba.mem.ioregs);
    }

    if gba.mem.ioregs.soundcnt_h.dma_enable(FifoChannel::B)
        && gba.mem.ioregs.soundcnt_h.dma_timer_select(FifoChannel::B) == timer
    {
        gba.audio.fifo_play(FifoChannel::B, &mut gba.mem.ioregs);
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Command {
    Wait(u32),
    PlaySample(FifoChannel, i8),
}
