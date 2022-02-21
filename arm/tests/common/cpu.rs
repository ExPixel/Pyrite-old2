#![allow(dead_code)]

use super::devkit;
use super::memory::TestMemory;
use arm::{Cpu, CpuMode, Isa};

/// An opcode that is actually an undefined instruction that is
/// used for signaling the end of execution in ARM mode.
const ARM_END_OPCODE: u32 = 0xF777F777;

/// An opcode that is used to signal the end of execution in THUMB mode.
/// By itself this is an undefined instruction. (2 of them make a branch with link but w/e)
const THUMB_END_OPCODE: u16 = 0xF777;

pub fn execute_arm(name: &str, source: &str) -> (Cpu, TestMemory) {
    let mut exec = Executor::new(name, arm::Isa::Arm);
    exec.push(source);
    (exec.cpu, exec.mem)
}

pub struct Executor {
    pub cpu: Cpu,
    pub mem: TestMemory,
    pub name: String,

    data: String,
    source: String,
    base_isa: Isa,
    count: u32,
}

impl Executor {
    pub fn new(name: impl Into<String>, base_isa: Isa) -> Self {
        let mut mem = TestMemory::with_padding(Vec::new(), 8);
        let cpu = Cpu::new(base_isa, CpuMode::System, &mut mem);

        Executor {
            cpu,
            mem,
            name: name.into(),
            source: String::new(),
            data: String::new(),
            base_isa,
            count: 0,
        }
    }

    pub fn clear_source(&mut self) {
        self.source.clear();
    }

    pub fn data(&mut self, data_source: &str) {
        self.data.push_str(data_source);
        self.data.push('\n');
    }

    pub fn push_no_exec(&mut self, source: &str) {
        self.source.push_str(source);
        self.source.push('\n');
        self.count += 1;
    }

    pub fn push(&mut self, source: &str) {
        self.push_no_exec(source);
        self.execute();
    }

    fn execute(&mut self) {
        let name = format!("{}-{}", self.name, self.count);

        let mut source = String::new();
        if !self.data.is_empty() {
            source.push_str(".data\n");
            source.push_str(&self.data);
        }
        source.push_str(".text\n");
        source.push_str(&self.source);
        source.push_str(".text\n");
        source.push_str("_exit:\n");
        source.push_str(".word 0xF777F777\n");
        let bin = devkit::assemble(self.base_isa, &name, &source).unwrap();

        let min_len = bin.len() + 8;
        self.mem.set_memory_with_padding(bin, min_len);

        self.cpu.registers.putf_t(self.base_isa == Isa::Thumb);
        self.cpu.branch(0, &mut self.mem);

        loop {
            let next_pc = self.cpu.next_exec_pc();

            // break in ARM mode
            if !self.cpu.registers.getf_t() && self.mem.view32(next_pc) == ARM_END_OPCODE {
                break;
            }

            // break in THUMB mode
            if self.cpu.registers.getf_t() && self.mem.view16(next_pc) == THUMB_END_OPCODE {
                break;
            }

            self.cpu.step(&mut self.mem);
        }
    }
}
