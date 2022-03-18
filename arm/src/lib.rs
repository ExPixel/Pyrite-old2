mod alu;
mod arm_instructions;
mod memory;
mod registers;
mod thumb_instructions;

pub use memory::{AccessType, Memory, Waitstates};
pub use registers::CpuMode;
pub use registers::Registers;

use util::bits::Bits as _;

/// Function that executes and ARM instruction and returns the number of cycles
/// that were required to complete it.
type InstrFunction = fn(cpu: &mut Cpu, memory: &mut dyn Memory, opcode: u32) -> Cycles;

pub type ExceptionHandler =
    Box<dyn Send + Sync + FnMut(&mut Cpu, &mut dyn Memory, CpuException) -> ExceptionHandlerResult>;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ExceptionHandlerResult {
    Handled,
    Ignored,
}

const EXCEPTION_BASE: u32 = 0;

/// mov r0, r0 -- opcode for an ARM instruction that does nothing.
const ARM_NOOP_OPCODE: u32 = 0xe1a00000;

/// mov r0, r0 -- opcode for a THUMB instruction that does nothing.
const THUMB_NOOP_OPCODE: u16 = 0x46c0;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Isa {
    Arm,
    Thumb,
}

pub struct Cpu {
    pub registers: Registers,
    pub exception_handler: Option<ExceptionHandler>,

    // pipeline:
    fetched: u32,
    decoded: u32,
    decoded_fn: InstrFunction,
}

impl Cpu {
    /// **IMPORTANT**: [`Cpu::branch`] must always be called with the starting address of the CPU
    /// before [`Cpu::step`] if this method is used to construct a [`Cpu`]. If not the PC
    /// will be 4 bytes ahead of where it should be.
    pub fn uninitialized(isa: Isa, mode: CpuMode) -> Self {
        let mut registers = Registers::new(mode);

        let noop_opcode = if isa == Isa::Thumb {
            registers.setf_t();
            THUMB_NOOP_OPCODE as u32
        } else {
            registers.clearf_t();
            ARM_NOOP_OPCODE
        };

        Cpu {
            registers,
            exception_handler: None,
            fetched: noop_opcode,
            decoded: noop_opcode,
            decoded_fn: noop,
        }
    }

    pub fn new(isa: Isa, mode: CpuMode, memory: &mut dyn Memory) -> Self {
        let mut cpu = Cpu::uninitialized(isa, mode);
        cpu.branch(0, memory);
        cpu
    }

    /// Steps the CPU forward. This will run the next fetch/decode/execute step of the ARM CPU pipeline
    /// as well as handle any interrupts that may have occurred while doing so. This returns the number
    /// of cycles that were required to complete the step.
    ///
    /// At the start of the step function, the program counter will be one instruction ahead of the address
    /// of the instruction that wil be executed. Before execution occurs it will be set to be two instructions
    /// ahead.
    pub fn step(&mut self, memory: &mut dyn Memory) -> Cycles {
        if self.registers.getf_t() {
            self.step_thumb(memory)
        } else {
            self.step_arm(memory)
        }
    }

    /// Returns the number of cycles required to step the CPU in the ARM state.
    fn step_arm(&mut self, memory: &mut dyn Memory) -> Cycles {
        let exec_opcode = self.decoded;
        let exec_fn = self.decoded_fn;

        self.decoded = self.fetched;
        self.decoded_fn = Self::decode_arm_opcode(self.decoded);

        let fetch_pc = (self.registers.read(15) & !0x3).wrapping_add(4);
        self.registers.write(15, fetch_pc);
        let (fetched, fetch_wait) = memory.fetch32(fetch_pc, AccessType::Seq);
        self.fetched = fetched;

        let cycles = Cycles::ONE + fetch_wait;

        if check_condition(exec_opcode >> 28, &self.registers) {
            cycles + exec_fn(self, memory, exec_opcode)
        } else {
            cycles
        }
    }

    /// Returns the number of cycles required to step the CPU in the THUMB state.
    fn step_thumb(&mut self, memory: &mut dyn Memory) -> Cycles {
        let exec_opcode = self.decoded;
        let exec_fn = self.decoded_fn;

        self.decoded = self.fetched;
        self.decoded_fn = Self::decode_thumb_opcode(self.decoded);

        let fetch_pc = (self.registers.read(15) & !0x1).wrapping_add(2);
        self.registers.write(15, fetch_pc);
        let (fetched, fetch_wait) = memory.load16(fetch_pc, AccessType::Seq);
        self.fetched = fetched as u32;

        let cycles = Cycles::ONE + fetch_wait;
        cycles + exec_fn(self, memory, exec_opcode)
    }

    /// The address of the instruction that will be executed next.
    pub fn next_exec_pc(&self) -> u32 {
        if self.registers.getf_t() {
            self.registers.read(15).wrapping_sub(2)
        } else {
            self.registers.read(15).wrapping_sub(4)
        }
    }

    pub fn branch(&mut self, address: u32, memory: &mut dyn Memory) -> Cycles {
        if self.registers.getf_t() {
            self.branch_thumb(address, memory)
        } else {
            self.branch_arm(address, memory)
        }
    }

    fn branch_arm(&mut self, address: u32, memory: &mut dyn Memory) -> Cycles {
        let address = address & !0x3;

        let (decoded, wd) = memory.fetch32(address, AccessType::NonSeq);
        let (fetched, wf) = memory.fetch32(address.wrapping_add(4), AccessType::Seq);

        self.decoded = decoded;
        self.decoded_fn = Self::decode_arm_opcode(decoded);
        self.fetched = fetched;

        self.registers.write(15, address.wrapping_add(4));

        Cycles::from(2u32) + wd + wf
    }

    fn branch_thumb(&mut self, address: u32, memory: &mut dyn Memory) -> Cycles {
        let address = address & !0x1;

        let (decoded, wd) = memory.fetch16(address, AccessType::NonSeq);
        let (fetched, wf) = memory.fetch16(address.wrapping_add(2), AccessType::Seq);

        self.decoded = decoded as u32;
        self.decoded_fn = Self::decode_thumb_opcode(decoded as u32);
        self.fetched = fetched as u32;

        self.registers.write(15, address.wrapping_add(2));

        Cycles::from(2u32) + wd + wf
    }

    fn decode_arm_opcode(opcode: u32) -> InstrFunction {
        let opcode_row = opcode.bits(20, 27);
        let opcode_col = opcode.bits(4, 7);
        let opcode_idx = (opcode_row * 16) + opcode_col;
        arm_instructions::ARM_OPCODE_TABLE[opcode_idx as usize]
    }

    fn decode_thumb_opcode(opcode: u32) -> InstrFunction {
        let opcode_row = opcode.bits(12, 15);
        let opcode_col = opcode.bits(8, 11);
        let opcode_idx = (opcode_row * 16) + opcode_col;
        thumb_instructions::THUMB_OPCODE_TABLE[opcode_idx as usize]
    }

    /// Sets the exception handler that will be called whenever the CPU encounters an
    /// exception such as an IRQ, SWI, ect.
    ///
    /// Exception handlers can use [`Cpu::next_exec_pc`] in order to retrieve an
    /// exception's return address.
    pub fn set_exception_handler<F>(&mut self, handler: F) -> Option<ExceptionHandler>
    where
        F: 'static
            + Send
            + Sync
            + FnMut(&mut Cpu, &mut dyn Memory, CpuException) -> ExceptionHandlerResult,
    {
        self.exception_handler.replace(Box::new(handler))
    }

    pub fn exception(&mut self, exception: CpuException, memory: &mut dyn Memory) -> Cycles {
        self.exception_with_ret(exception, self.next_exec_pc(), memory)
    }

    /// This version is meant to be called when an exception is thrown inside of an
    /// instruction.
    fn exception_internal(&mut self, exception: CpuException, memory: &mut dyn Memory) -> Cycles {
        let return_addr = self
            .registers
            .read(15)
            .wrapping_sub(if self.registers.getf_t() { 2 } else { 4 });
        self.exception_with_ret(exception, return_addr, memory)
    }

    /// Actions performed by CPU when entering an exception
    ///   - R14_<new mode>=PC+nn   ;save old PC, ie. return address
    ///   - SPSR_<new mode>=CPSR   ;save old flags
    ///   - CPSR new T,M bits      ;set to T=0 (ARM state), and M4-0=new mode
    ///   - CPSR new I bit         ;IRQs disabled (I=1), done by ALL exceptions
    ///   - CPSR new F bit         ;FIQs disabled (F=1), done by Reset and FIQ only
    ///   - PC=exception_vector
    fn exception_with_ret(
        &mut self,
        exception: CpuException,
        return_addr: u32,
        memory: &mut dyn Memory,
    ) -> Cycles {
        let exception_info = exception.info();
        let exception_vector = EXCEPTION_BASE + exception_info.offset;

        // we temporarily remove the handler while processing and exception
        // we don't want reentrant exception handling and Rust's borrow checker
        // doesn't like it anyway.
        if let Some(mut handler) = self.exception_handler.take() {
            let result = handler(self, memory, exception);
            if result == ExceptionHandlerResult::Handled {
                // #TODO Probably should be smarter about how we return cycles here.
                //       For now a handled exception is just treated as an internal cycle.
                return Cycles::ONE;
            }
        }

        let cpsr = self.registers.read_cpsr();
        self.registers.write_mode(exception_info.mode_on_entry); // Set the entry mode.
        self.registers.write_spsr(cpsr); // Set the CPSR of the old mode to the SPSR of the new mode.
        self.registers
            .write(14, return_addr.wrapping_add(exception_info.pc_adjust)); // Save the return address.
        self.registers.clearf_t(); // Go into ARM mode.

        self.registers.putf_i(true); // IRQ disable (done by all modes)

        if let Some(f) = exception_info.f_flag {
            self.registers.putf_f(f); // FIQ disable (done by RESET and FIQ only)
        }

        self.branch_arm(exception_vector, memory) // PC = exception_vector
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CpuException {
    Reset,
    Undefined,
    SWI,
    PrefetchAbort,
    DataAbort,
    IRQ,
    FIQ,
    AddressExceeds26Bit,
}

impl CpuException {
    pub fn name(self) -> &'static str {
        match self {
            CpuException::Reset => "Reset",
            CpuException::Undefined => "Undefined",
            CpuException::SWI => "SWI",
            CpuException::PrefetchAbort => "Prefetch Abort",
            CpuException::DataAbort => "Data Abort",
            CpuException::IRQ => "IRQ",
            CpuException::FIQ => "FIQ",
            CpuException::AddressExceeds26Bit => "Address Exceeds 26 bit",
        }
    }

    pub fn info(self) -> CpuExceptionInfo {
        match self {
            CpuException::Reset => EXCEPTION_INFO_RESET,
            CpuException::Undefined => EXCEPTION_INFO_UNDEFINED,
            CpuException::SWI => EXCEPTION_INFO_SWI,
            CpuException::PrefetchAbort => EXCEPTION_INFO_PREFETCH_ABORT,
            CpuException::DataAbort => EXCEPTION_INFO_DATA_ABORT,
            CpuException::IRQ => EXCEPTION_INFO_IRQ,
            CpuException::FIQ => EXCEPTION_INFO_FIQ,
            CpuException::AddressExceeds26Bit => EXCEPTION_INFO_ADDRESS_EXCEEDS_26BIT,
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct CpuExceptionInfo {
    mode_on_entry: CpuMode,
    f_flag: Option<bool>,
    pc_adjust: u32,
    offset: u32,

    /// Lower number means higher priority.
    priority: u8,
}

impl CpuExceptionInfo {
    pub const fn new(
        priority: u8,
        mode_on_entry: CpuMode,
        f_flag: Option<bool>,
        pc_adjust: u32,
        offset: u32,
    ) -> CpuExceptionInfo {
        CpuExceptionInfo {
            priority,
            mode_on_entry,
            f_flag,
            pc_adjust,
            offset,
        }
    }
}

// The following are the exception vectors in memory. That is, when an exception arises, CPU is switched into ARM state, and the program counter (PC) is loaded by the respective address.
//   Address  Prio  Exception                  Mode on Entry      Interrupt Flags
//   BASE+00h 1     Reset                      Supervisor (_svc)  I=1, F=1
//   BASE+04h 7     Undefined Instruction      Undefined  (_und)  I=1, F=unchanged
//   BASE+08h 6     Software Interrupt (SWI)   Supervisor (_svc)  I=1, F=unchanged
//   BASE+0Ch 5     Prefetch Abort             Abort      (_abt)  I=1, F=unchanged
//   BASE+10h 2     Data Abort                 Abort      (_abt)  I=1, F=unchanged
//   BASE+14h ??    Address Exceeds 26bit      Supervisor (_svc)  I=1, F=unchanged
//   BASE+18h 4     Normal Interrupt (IRQ)     IRQ        (_irq)  I=1, F=unchanged
//   BASE+1Ch 3     Fast Interrupt (FIQ)       FIQ        (_fiq)  I=1, F=1
pub const EXCEPTION_INFO_RESET: CpuExceptionInfo =
    CpuExceptionInfo::new(1, CpuMode::Supervisor, Some(true), 0, 0x00);
pub const EXCEPTION_INFO_UNDEFINED: CpuExceptionInfo =
    CpuExceptionInfo::new(7, CpuMode::Undefined, None, 0, 0x04);
pub const EXCEPTION_INFO_SWI: CpuExceptionInfo =
    CpuExceptionInfo::new(6, CpuMode::Supervisor, None, 0, 0x08);
pub const EXCEPTION_INFO_PREFETCH_ABORT: CpuExceptionInfo =
    CpuExceptionInfo::new(5, CpuMode::Abort, None, 4, 0x0C);
pub const EXCEPTION_INFO_DATA_ABORT: CpuExceptionInfo =
    CpuExceptionInfo::new(2, CpuMode::Abort, None, 4, 0x10);
pub const EXCEPTION_INFO_IRQ: CpuExceptionInfo =
    CpuExceptionInfo::new(4, CpuMode::IRQ, None, 4, 0x18);
pub const EXCEPTION_INFO_FIQ: CpuExceptionInfo =
    CpuExceptionInfo::new(3, CpuMode::FIQ, Some(true), 4, 0x1C);

// #TODO I don't actually know the priority for the 26bit address overflow exception.
pub const EXCEPTION_INFO_ADDRESS_EXCEEDS_26BIT: CpuExceptionInfo =
    CpuExceptionInfo::new(8, CpuMode::Supervisor, None, 4, 0x14);

/// Returns true if an instruction should run based
/// the given condition code and cpsr.
fn check_condition(cond: u32, regs: &Registers) -> bool {
    match cond {
        0x0 => regs.getf_z(),  // 0:   EQ     Z=1           equal (zero) (same)
        0x1 => !regs.getf_z(), // 1:   NE     Z=0           not equal (nonzero) (not same)
        0x2 => regs.getf_c(),  // 2:   CS/HS  C=1           unsigned higher or same (carry set)
        0x3 => !regs.getf_c(), // 3:   CC/LO  C=0           unsigned lower (carry cleared)
        0x4 => regs.getf_n(),  // 4:   MI     N=1           negative (minus)
        0x5 => !regs.getf_n(), // 5:   PL     N=0           positive or zero (plus)
        0x6 => regs.getf_v(),  // 6:   VS     V=1           overflow (V set)
        0x7 => !regs.getf_v(), // 7:   VC     V=0           no overflow (V cleared)
        0x8 => regs.getf_c() & !regs.getf_z(), // 8:   HI     C=1 and Z=0   unsigned higher
        0x9 => !regs.getf_c() | regs.getf_z(), // 9:   LS     C=0 or Z=1    unsigned lower or same
        0xA => regs.getf_n() == regs.getf_v(), // A:   GE     N=V           greater or equal
        0xB => regs.getf_n() != regs.getf_v(), // B:   LT     N<>V          less than
        0xC => {
            // C:   GT     Z=0 and N=V   greater than
            !regs.getf_z() & (regs.getf_n() == regs.getf_v())
        }
        0xD => {
            // D:   LE     Z=1 or N<>V   less or equal
            regs.getf_z() | (regs.getf_n() != regs.getf_v())
        }
        0xE => true, // E:   AL     -             always (the "AL" suffix can be omitted)
        0xF => false, // F:   NV     -             never (ARMv1,v2 only) (Reserved ARMv3 and up)

        // :(
        _ => unreachable!("bad condition code: 0x{:08X} ({:04b})", cond, cond),
    }
}

fn noop(_cpu: &mut Cpu, _memory: &mut dyn Memory, _opcode: u32) -> Cycles {
    Cycles::ZERO
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Cycles(u32);

impl Cycles {
    pub const ZERO: Cycles = Cycles(0);
    pub const ONE: Cycles = Cycles(1);

    pub const fn new(cycles: u32) -> Self {
        Cycles(cycles)
    }
}

impl std::fmt::Display for Cycles {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl From<u32> for Cycles {
    #[inline]
    fn from(value: u32) -> Self {
        Cycles(value)
    }
}

impl From<Waitstates> for Cycles {
    #[inline]
    fn from(waitstates: Waitstates) -> Self {
        Cycles(u32::from(waitstates))
    }
}

impl From<Cycles> for u32 {
    #[inline]
    fn from(cycles: Cycles) -> Self {
        cycles.0
    }
}

impl std::ops::Add<Cycles> for Cycles {
    type Output = Self;

    #[inline]
    fn add(self, other: Self) -> Self {
        Cycles(self.0 + other.0)
    }
}

impl std::ops::AddAssign<Cycles> for Cycles {
    #[inline]
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
    }
}

impl std::ops::Sub<Cycles> for Cycles {
    type Output = Self;

    #[inline]
    fn sub(self, other: Self) -> Self {
        Cycles(self.0.saturating_sub(other.0))
    }
}

impl std::ops::SubAssign<Cycles> for Cycles {
    #[inline]
    fn sub_assign(&mut self, other: Self) {
        self.0 = self.0.saturating_sub(other.0);
    }
}

impl std::ops::Add<Waitstates> for Cycles {
    type Output = Self;

    #[inline]
    fn add(self, other: Waitstates) -> Self {
        Cycles(self.0 + u32::from(other))
    }
}

impl std::ops::AddAssign<Waitstates> for Cycles {
    #[inline]
    fn add_assign(&mut self, other: Waitstates) {
        self.0 += u32::from(other);
    }
}
