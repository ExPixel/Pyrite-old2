use super::super::{AccessType, Cpu, Cycles, Memory};
use util::bits::Bits as _;

const LOAD: bool = true;
const STORE: bool = false;

const POST: bool = false;
const PRE: bool = true;

const DEC: bool = false;
const INC: bool = true;

const WRITEBACK: bool = true;
const NO_WRITEBACK: bool = false;

/// Generates a function for a halfword or signed data transfer function.
macro_rules! arm_gen_hwsdt {
    ($name:ident, $transfer:expr, $transfer_type:expr, $data_size:expr, $get_offset:expr, $direction:expr, $indexing:expr, $writeback:expr) => {
        pub fn $name(cpu: &mut Cpu, memory: &mut dyn Memory, instr: u32) -> Cycles {
            let rd = instr.bits(12, 15);
            let rn = instr.bits(16, 19);
            let offset = $get_offset(cpu, instr);
            let mut addr = cpu.registers.read(rn);

            // pre-indexing
            if $indexing == PRE {
                if $direction == INC {
                    addr = addr.wrapping_add(offset);
                } else {
                    addr = addr.wrapping_sub(offset);
                }
            }

            // writeback to base register
            // post-indexing as well
            if $writeback {
                let writeback_addr = if $indexing == POST {
                    if $direction == INC {
                        addr.wrapping_add(offset)
                    } else {
                        addr.wrapping_sub(offset)
                    }
                } else {
                    addr
                };
                cpu.registers.write(rn, writeback_addr);
            }

            let mut cycles = $transfer(cpu, memory, rd, addr);

            if $transfer_type == LOAD {
                if rd == 15 || ($writeback == WRITEBACK && rn == 15) {
                    let dest_pc = cpu.registers.read(15);
                    cycles += cpu.branch_arm(dest_pc, memory);
                }
            }

            return cycles;
        }
    };
}

#[must_use]
fn ldrh(cpu: &mut Cpu, memory: &mut dyn Memory, rd: u32, addr: u32) -> Cycles {
    // We don't align the address here. If bit 0 is high then behaviour is just
    // unpredictable (depends on memory hardware).
    let (value, wait) = memory.load16(addr, AccessType::NonSeq);
    cpu.registers.write(rd, value as u32);
    return Cycles::ONE + wait;
}

#[must_use]
fn ldrsh(cpu: &mut Cpu, memory: &mut dyn Memory, rd: u32, addr: u32) -> Cycles {
    // We don't align the address here. If bit 0 is high then behaviour is just
    // unpredictable (depends on memory hardware).
    let (value, wait) = memory.load16(addr, AccessType::NonSeq);
    cpu.registers.write(rd, value as i16 as i32 as u32);
    return Cycles::ONE + wait;
}

#[must_use]
fn ldrsb(cpu: &mut Cpu, memory: &mut dyn Memory, rd: u32, addr: u32) -> Cycles {
    let (value, wait) = memory.load8(addr, AccessType::NonSeq);
    cpu.registers.write(rd, value as i8 as i32 as u32);
    return Cycles::ONE + wait;
}

#[must_use]
fn strh(cpu: &mut Cpu, memory: &mut dyn Memory, rd: u32, addr: u32) -> Cycles {
    let mut value = cpu.registers.read(rd);

    // If the program counter is used as the source register in a halfword store, it will
    // be 12 bytes ahead instead of 8 when read.
    if rd == 15 {
        value = value.wrapping_add(4);
    }

    let wait = memory.store16(addr, value as u16, AccessType::NonSeq);
    return Cycles::ONE + wait;
}

fn off_imm(_cpu: &Cpu, instr: u32) -> u32 {
    let lo = instr.bits(0, 3);
    let hi = instr.bits(8, 11);
    return lo | (hi << 4);
}

fn off_reg(cpu: &Cpu, instr: u32) -> u32 {
    let rm = instr.bits(0, 3);
    return cpu.registers.read(rm);
}

// Load halfword, Negative immediate offset
arm_gen_hwsdt!(
    arm_ldrh_ofim,
    ldrh,
    LOAD,
    16,
    off_imm,
    DEC,
    PRE,
    NO_WRITEBACK
);

// Load halfword, Positive immediate offset
arm_gen_hwsdt!(
    arm_ldrh_ofip,
    ldrh,
    LOAD,
    16,
    off_imm,
    INC,
    PRE,
    NO_WRITEBACK
);

// Load halfword, Negative register offset
arm_gen_hwsdt!(
    arm_ldrh_ofrm,
    ldrh,
    LOAD,
    16,
    off_reg,
    DEC,
    PRE,
    NO_WRITEBACK
);

// Load halfword, Positive register offset
arm_gen_hwsdt!(
    arm_ldrh_ofrp,
    ldrh,
    LOAD,
    16,
    off_reg,
    INC,
    PRE,
    NO_WRITEBACK
);

// Load halfword, Immediate offset, pre-decrement
arm_gen_hwsdt!(arm_ldrh_prim, ldrh, LOAD, 16, off_imm, DEC, PRE, WRITEBACK);

// Load halfword, Immediate offset, pre-increment
arm_gen_hwsdt!(arm_ldrh_prip, ldrh, LOAD, 16, off_imm, INC, PRE, WRITEBACK);

// Load halfword, Register offset, pre-decrement
arm_gen_hwsdt!(arm_ldrh_prrm, ldrh, LOAD, 16, off_reg, DEC, PRE, WRITEBACK);

// Load halfword, Register offset, pre-increment
arm_gen_hwsdt!(arm_ldrh_prrp, ldrh, LOAD, 16, off_reg, INC, PRE, WRITEBACK);

// Load halfword, Immediate offset, post-decrement
arm_gen_hwsdt!(arm_ldrh_ptim, ldrh, LOAD, 16, off_imm, DEC, POST, WRITEBACK);

// Load halfword, Immediate offset, post-increment
arm_gen_hwsdt!(arm_ldrh_ptip, ldrh, LOAD, 16, off_imm, INC, POST, WRITEBACK);

// Load halfword, Register offset, post-decrement
arm_gen_hwsdt!(arm_ldrh_ptrm, ldrh, LOAD, 16, off_reg, DEC, POST, WRITEBACK);

// Load halfword, Register offset, post-increment
arm_gen_hwsdt!(arm_ldrh_ptrp, ldrh, LOAD, 16, off_reg, INC, POST, WRITEBACK);

// Load signed byte, Negative immediate offset
arm_gen_hwsdt!(
    arm_ldrsb_ofim,
    ldrsb,
    LOAD,
    8,
    off_imm,
    DEC,
    PRE,
    NO_WRITEBACK
);

// Load signed byte, Positive immediate offset
arm_gen_hwsdt!(
    arm_ldrsb_ofip,
    ldrsb,
    LOAD,
    8,
    off_imm,
    INC,
    PRE,
    NO_WRITEBACK
);

// Load signed byte, Negative register offset
arm_gen_hwsdt!(
    arm_ldrsb_ofrm,
    ldrsb,
    LOAD,
    8,
    off_reg,
    DEC,
    PRE,
    NO_WRITEBACK
);

// Load signed byte, Positive register offset
arm_gen_hwsdt!(
    arm_ldrsb_ofrp,
    ldrsb,
    LOAD,
    8,
    off_reg,
    INC,
    PRE,
    NO_WRITEBACK
);

// Load signed byte, Immediate offset, pre-decrement
arm_gen_hwsdt!(arm_ldrsb_prim, ldrsb, LOAD, 8, off_imm, DEC, PRE, WRITEBACK);

// Load signed byte, Immediate offset, pre-increment
arm_gen_hwsdt!(arm_ldrsb_prip, ldrsb, LOAD, 8, off_imm, INC, PRE, WRITEBACK);

// Load signed byte, Register offset, pre-decrement
arm_gen_hwsdt!(arm_ldrsb_prrm, ldrsb, LOAD, 8, off_reg, DEC, PRE, WRITEBACK);

// Load signed byte, Register offset, pre-increment
arm_gen_hwsdt!(arm_ldrsb_prrp, ldrsb, LOAD, 8, off_reg, INC, PRE, WRITEBACK);

// Load signed byte, Immediate offset, post-decrement
arm_gen_hwsdt!(
    arm_ldrsb_ptim,
    ldrsb,
    LOAD,
    8,
    off_imm,
    DEC,
    POST,
    WRITEBACK
);

// Load signed byte, Immediate offset, post-increment
arm_gen_hwsdt!(
    arm_ldrsb_ptip,
    ldrsb,
    LOAD,
    8,
    off_imm,
    INC,
    POST,
    WRITEBACK
);

// Load signed byte, Register offset, post-decrement
arm_gen_hwsdt!(
    arm_ldrsb_ptrm,
    ldrsb,
    LOAD,
    8,
    off_reg,
    DEC,
    POST,
    WRITEBACK
);

// Load signed byte, Register offset, post-increment
arm_gen_hwsdt!(
    arm_ldrsb_ptrp,
    ldrsb,
    LOAD,
    8,
    off_reg,
    INC,
    POST,
    WRITEBACK
);

// Load signed halfword, Negative immediate offset
arm_gen_hwsdt!(
    arm_ldrsh_ofim,
    ldrsh,
    LOAD,
    16,
    off_imm,
    DEC,
    PRE,
    NO_WRITEBACK
);

// Load signed halfword, Positive immediate offset
arm_gen_hwsdt!(
    arm_ldrsh_ofip,
    ldrsh,
    LOAD,
    16,
    off_imm,
    INC,
    PRE,
    NO_WRITEBACK
);

// Load signed halfword, Negative register offset
arm_gen_hwsdt!(
    arm_ldrsh_ofrm,
    ldrsh,
    LOAD,
    16,
    off_reg,
    DEC,
    PRE,
    NO_WRITEBACK
);

// Load signed halfword, Positive register offset
arm_gen_hwsdt!(
    arm_ldrsh_ofrp,
    ldrsh,
    LOAD,
    16,
    off_reg,
    INC,
    PRE,
    NO_WRITEBACK
);

// Load signed halfword, Immediate offset, pre-decrement
arm_gen_hwsdt!(
    arm_ldrsh_prim,
    ldrsh,
    LOAD,
    16,
    off_imm,
    DEC,
    PRE,
    WRITEBACK
);

// Load signed halfword, Immediate offset, pre-increment
arm_gen_hwsdt!(
    arm_ldrsh_prip,
    ldrsh,
    LOAD,
    16,
    off_imm,
    INC,
    PRE,
    WRITEBACK
);

// Load signed halfword, Register offset, pre-decrement
arm_gen_hwsdt!(
    arm_ldrsh_prrm,
    ldrsh,
    LOAD,
    16,
    off_reg,
    DEC,
    PRE,
    WRITEBACK
);

// Load signed halfword, Register offset, pre-increment
arm_gen_hwsdt!(
    arm_ldrsh_prrp,
    ldrsh,
    LOAD,
    16,
    off_reg,
    INC,
    PRE,
    WRITEBACK
);

// Load signed halfword, Immediate offset, post-decrement
arm_gen_hwsdt!(
    arm_ldrsh_ptim,
    ldrsh,
    LOAD,
    16,
    off_imm,
    DEC,
    POST,
    WRITEBACK
);

// Load signed halfword, Immediate offset, post-increment
arm_gen_hwsdt!(
    arm_ldrsh_ptip,
    ldrsh,
    LOAD,
    16,
    off_imm,
    INC,
    POST,
    WRITEBACK
);

// Load signed halfword, Register offset, post-decrement
arm_gen_hwsdt!(
    arm_ldrsh_ptrm,
    ldrsh,
    LOAD,
    16,
    off_reg,
    DEC,
    POST,
    WRITEBACK
);

// Load signed halfword, Register offset, post-increment
arm_gen_hwsdt!(
    arm_ldrsh_ptrp,
    ldrsh,
    LOAD,
    16,
    off_reg,
    INC,
    POST,
    WRITEBACK
);

// Store halfword, Negative immediate offset
arm_gen_hwsdt!(
    arm_strh_ofim,
    strh,
    STORE,
    16,
    off_imm,
    DEC,
    PRE,
    NO_WRITEBACK
);

// Store halfword, Positive immediate offset
arm_gen_hwsdt!(
    arm_strh_ofip,
    strh,
    STORE,
    16,
    off_imm,
    INC,
    PRE,
    NO_WRITEBACK
);

// Store halfword, Negative register offset
arm_gen_hwsdt!(
    arm_strh_ofrm,
    strh,
    STORE,
    16,
    off_reg,
    DEC,
    PRE,
    NO_WRITEBACK
);

// Store halfword, Positive register offset
arm_gen_hwsdt!(
    arm_strh_ofrp,
    strh,
    STORE,
    16,
    off_reg,
    INC,
    PRE,
    NO_WRITEBACK
);

// Store halfword, Immediate offset, pre-decrement
arm_gen_hwsdt!(arm_strh_prim, strh, STORE, 16, off_imm, DEC, PRE, WRITEBACK);

// Store halfword, Immediate offset, pre-increment
arm_gen_hwsdt!(arm_strh_prip, strh, STORE, 16, off_imm, INC, PRE, WRITEBACK);

// Store halfword, Register offset, pre-decrement
arm_gen_hwsdt!(arm_strh_prrm, strh, STORE, 16, off_reg, DEC, PRE, WRITEBACK);

// Store halfword, Register offset, pre-increment
arm_gen_hwsdt!(arm_strh_prrp, strh, STORE, 16, off_reg, INC, PRE, WRITEBACK);

// Store halfword, Immediate offset, post-decrement
arm_gen_hwsdt!(
    arm_strh_ptim,
    strh,
    STORE,
    16,
    off_imm,
    DEC,
    POST,
    WRITEBACK
);

// Store halfword, Immediate offset, post-increment
arm_gen_hwsdt!(
    arm_strh_ptip,
    strh,
    STORE,
    16,
    off_imm,
    INC,
    POST,
    WRITEBACK
);

// Store halfword, Register offset, post-decrement
arm_gen_hwsdt!(
    arm_strh_ptrm,
    strh,
    STORE,
    16,
    off_reg,
    DEC,
    POST,
    WRITEBACK
);

// Store halfword, Register offset, post-increment
arm_gen_hwsdt!(
    arm_strh_ptrp,
    strh,
    STORE,
    16,
    off_reg,
    INC,
    POST,
    WRITEBACK
);
