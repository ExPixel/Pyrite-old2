//! Stubs for ARM _instructions that have yet to be implemented.

use super::super::{Cpu, Cycles, Memory};

// Perform coprocessor data operation
pub fn arm_cdp(cpu: &mut Cpu, memory: &mut dyn Memory, _instr: u32) -> Cycles {
    log::warn!(
        "unsupported instruction `cdp` at 0x{:08X}",
        cpu.registers.read(15).wrapping_sub(8)
    );
    Cycles::ZERO
}

// Load coprocessor data from memory, Negative offset
pub fn arm_ldc_ofm(cpu: &mut Cpu, memory: &mut dyn Memory, _instr: u32) -> Cycles {
    log::warn!(
        "unsupported instruction `ldc` at 0x{:08X}",
        cpu.registers.read(15).wrapping_sub(8)
    );
    Cycles::ZERO
}

// Load coprocessor data from memory, Positive offset
pub fn arm_ldc_ofp(cpu: &mut Cpu, memory: &mut dyn Memory, _instr: u32) -> Cycles {
    log::warn!(
        "unsupported instruction `ldc` at 0x{:08X}",
        cpu.registers.read(15).wrapping_sub(8)
    );
    Cycles::ZERO
}

// Load coprocessor data from memory, Pre-decrement
pub fn arm_ldc_prm(cpu: &mut Cpu, memory: &mut dyn Memory, _instr: u32) -> Cycles {
    log::warn!(
        "unsupported instruction `ldc` at 0x{:08X}",
        cpu.registers.read(15).wrapping_sub(8)
    );
    Cycles::ZERO
}

// Load coprocessor data from memory, Pre-increment
pub fn arm_ldc_prp(cpu: &mut Cpu, memory: &mut dyn Memory, _instr: u32) -> Cycles {
    log::warn!(
        "unsupported instruction `ldc` at 0x{:08X}",
        cpu.registers.read(15).wrapping_sub(8)
    );
    Cycles::ZERO
}

// Load coprocessor data from memory, Post-decrement
pub fn arm_ldc_ptm(cpu: &mut Cpu, memory: &mut dyn Memory, _instr: u32) -> Cycles {
    log::warn!(
        "unsupported instruction `ldc` at 0x{:08X}",
        cpu.registers.read(15).wrapping_sub(8)
    );
    Cycles::ZERO
}

// Load coprocessor data from memory, Post-increment
pub fn arm_ldc_ptp(cpu: &mut Cpu, memory: &mut dyn Memory, _instr: u32) -> Cycles {
    log::warn!(
        "unsupported instruction `ldc` at 0x{:08X}",
        cpu.registers.read(15).wrapping_sub(8)
    );
    Cycles::ZERO
}

// Load coprocessor data from memory, Unindexed, bits 7-0 available for copro use
pub fn arm_ldc_unm(cpu: &mut Cpu, memory: &mut dyn Memory, _instr: u32) -> Cycles {
    log::warn!(
        "unsupported instruction `ldc` at 0x{:08X}",
        cpu.registers.read(15).wrapping_sub(8)
    );
    Cycles::ZERO
}

// Load coprocessor data from memory, Unindexed, bits 7-0 available for copro use
pub fn arm_ldc_unp(cpu: &mut Cpu, memory: &mut dyn Memory, _instr: u32) -> Cycles {
    log::warn!(
        "unsupported instruction `ldc` at 0x{:08X}",
        cpu.registers.read(15).wrapping_sub(8)
    );
    Cycles::ZERO
}

// Write coprocessor register from ARM register
pub fn arm_mcr(cpu: &mut Cpu, memory: &mut dyn Memory, _instr: u32) -> Cycles {
    log::warn!(
        "unsupported instruction `mcr` at 0x{:08X}",
        cpu.registers.read(15).wrapping_sub(8)
    );
    Cycles::ZERO
}

// Read coprocessor register to ARM register
pub fn arm_mrc(cpu: &mut Cpu, memory: &mut dyn Memory, _instr: u32) -> Cycles {
    log::warn!(
        "unsupported instruction `mrc` at 0x{:08X}",
        cpu.registers.read(15).wrapping_sub(8)
    );
    Cycles::ZERO
}

// Store coprocessor data to memory, Negative offset
pub fn arm_stc_ofm(cpu: &mut Cpu, memory: &mut dyn Memory, _instr: u32) -> Cycles {
    log::warn!(
        "unsupported instruction `stc` at 0x{:08X}",
        cpu.registers.read(15).wrapping_sub(8)
    );
    Cycles::ZERO
}

// Store coprocessor data to memory, Positive offset
pub fn arm_stc_ofp(cpu: &mut Cpu, memory: &mut dyn Memory, _instr: u32) -> Cycles {
    log::warn!(
        "unsupported instruction `stc` at 0x{:08X}",
        cpu.registers.read(15).wrapping_sub(8)
    );
    Cycles::ZERO
}

// Store coprocessor data to memory, Pre-decrement
pub fn arm_stc_prm(cpu: &mut Cpu, memory: &mut dyn Memory, _instr: u32) -> Cycles {
    log::warn!(
        "unsupported instruction `stc` at 0x{:08X}",
        cpu.registers.read(15).wrapping_sub(8)
    );
    Cycles::ZERO
}

// Store coprocessor data to memory, Pre-increment
pub fn arm_stc_prp(cpu: &mut Cpu, memory: &mut dyn Memory, _instr: u32) -> Cycles {
    log::warn!(
        "unsupported instruction `stc` at 0x{:08X}",
        cpu.registers.read(15).wrapping_sub(8)
    );
    Cycles::ZERO
}

// Store coprocessor data to memory, Post-decrement
pub fn arm_stc_ptm(cpu: &mut Cpu, memory: &mut dyn Memory, _instr: u32) -> Cycles {
    log::warn!(
        "unsupported instruction `stc` at 0x{:08X}",
        cpu.registers.read(15).wrapping_sub(8)
    );
    Cycles::ZERO
}

// Store coprocessor data to memory, Post-increment
pub fn arm_stc_ptp(cpu: &mut Cpu, memory: &mut dyn Memory, _instr: u32) -> Cycles {
    log::warn!(
        "unsupported instruction `stc` at 0x{:08X}",
        cpu.registers.read(15).wrapping_sub(8)
    );
    Cycles::ZERO
}

// Store coprocessor data to memory, Unindexed, bits 7-0 available for copro use
pub fn arm_stc_unm(cpu: &mut Cpu, memory: &mut dyn Memory, _instr: u32) -> Cycles {
    log::warn!(
        "unsupported instruction `stc` at 0x{:08X}",
        cpu.registers.read(15).wrapping_sub(8)
    );
    Cycles::ZERO
}

// Store coprocessor data to memory, Unindexed, bits 7-0 available for copro use
pub fn arm_stc_unp(cpu: &mut Cpu, memory: &mut dyn Memory, _instr: u32) -> Cycles {
    log::warn!(
        "unsupported instruction `stc` at 0x{:08X}",
        cpu.registers.read(15).wrapping_sub(8)
    );
    Cycles::ZERO
}
