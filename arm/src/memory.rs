pub trait Memory {
    /// Used to fetch a 32bit opcode in ARM mode.
    fn fetch32(&mut self, address: u32, access: AccessType) -> (u32, Waitstates) {
        self.load32(address, access)
    }

    /// Used to to fetch a 16bit opcode in THUMB mode.
    fn fetch16(&mut self, address: u32, access: AccessType) -> (u16, Waitstates) {
        self.load16(address, access)
    }

    fn load32(&mut self, address: u32, access: AccessType) -> (u32, Waitstates);
    fn load16(&mut self, address: u32, access: AccessType) -> (u16, Waitstates);
    fn load8(&mut self, address: u32, access: AccessType) -> (u8, Waitstates);

    fn store32(&mut self, address: u32, value: u32, access: AccessType) -> Waitstates;
    fn store16(&mut self, address: u32, value: u16, access: AccessType) -> Waitstates;
    fn store8(&mut self, address: u32, value: u8, access: AccessType) -> Waitstates;

    /// Stalling for some number of internal cycles.
    fn stall(&mut self, _cycles: super::Cycles) {
        /* NOP */
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AccessType {
    NonSeq = 0,
    Seq = 1,
}

impl AccessType {
    #[inline]
    pub fn is_seq(self) -> bool {
        self == AccessType::Seq
    }

    #[inline]
    pub fn is_nonseq(self) -> bool {
        self == AccessType::NonSeq
    }
}

/// The number of waitstates (single clock cycles) that were required to perform a memory operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Waitstates(u8);

impl Waitstates {
    pub const ZERO: Waitstates = Waitstates(0);
    pub const ONE: Waitstates = Waitstates(1);
}

impl std::fmt::Display for Waitstates {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl std::ops::Add<Self> for Waitstates {
    type Output = Self;

    #[inline]
    fn add(self, other: Waitstates) -> Self {
        Waitstates(self.0 + other.0)
    }
}

impl std::ops::AddAssign<Self> for Waitstates {
    #[inline]
    fn add_assign(&mut self, other: Waitstates) {
        self.0 += other.0
    }
}

impl std::ops::Sub<Self> for Waitstates {
    type Output = Self;

    #[inline]
    fn sub(self, other: Waitstates) -> Self {
        Waitstates(self.0 - other.0)
    }
}

impl From<u8> for Waitstates {
    #[inline]
    fn from(value: u8) -> Self {
        Waitstates(value)
    }
}

impl From<Waitstates> for u32 {
    #[inline]
    fn from(wait: Waitstates) -> Self {
        wait.0 as u32
    }
}
