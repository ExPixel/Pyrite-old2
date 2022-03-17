pub trait Bits {
    const BITS: u32 = 0;
    type Signed;
    type Unsigned;

    fn bits(self, start: u32, end: u32) -> Self;
    fn bits_from(self, start: u32) -> Self;
    fn bit(self, offset: u32) -> Self;

    #[allow(clippy::wrong_self_convention)]
    fn is_bit_set(self, offset: u32) -> bool;
    fn replace_bits(self, start: u32, end: u32, value: Self) -> Self;
    fn replace_bit(self, offset: u32, value: bool) -> Self;
    fn sign_extend(self, bits: u32) -> Self;
}

impl Bits for u32 {
    const BITS: u32 = 32;
    type Signed = i32;
    type Unsigned = Self;

    #[inline(always)]
    fn bits(self, start: u32, end: u32) -> Self {
        (self >> start) & ((1 << (end - start + 1)) - 1)
    }

    #[inline(always)]
    fn bits_from(self, start: u32) -> Self {
        (self >> start) & 1
    }

    #[inline(always)]
    fn bit(self, offset: u32) -> Self {
        (self >> offset) & 1
    }

    #[inline(always)]
    fn is_bit_set(self, offset: u32) -> bool {
        self.bit(offset) != 0
    }

    #[inline(always)]
    fn replace_bits(self, start: u32, end: u32, value: Self) -> Self {
        (self & !(((1 << (end - start + 1)) - 1) << start))
            | ((value & ((1 << (end - start + 1)) - 1)) << start)
    }

    #[inline(always)]
    fn replace_bit(self, offset: u32, value: bool) -> Self {
        self.replace_bits(offset, offset, value as Self)
    }

    #[inline(always)]
    fn sign_extend(self, bits: u32) -> Self {
        (((self << (Self::BITS as Self - bits as Self)) as Self::Signed) >> (32 - bits)) as Self
    }
}

impl Bits for u16 {
    const BITS: u32 = 16;
    type Signed = i32;
    type Unsigned = Self;

    #[inline(always)]
    fn bits(self, start: u32, end: u32) -> Self {
        (self >> start) & ((1 << (end - start + 1)) - 1)
    }

    #[inline(always)]
    fn bits_from(self, start: u32) -> Self {
        (self >> start) & 1
    }

    #[inline(always)]
    fn bit(self, offset: u32) -> Self {
        (self >> offset) & 1
    }

    #[inline(always)]
    fn is_bit_set(self, offset: u32) -> bool {
        self.bit(offset) != 0
    }

    #[inline(always)]
    fn replace_bits(self, start: u32, end: u32, value: Self) -> Self {
        (self & !(((1 << (end - start + 1)) - 1) << start))
            | ((value & ((1 << (end - start + 1)) - 1)) << start)
    }

    #[inline(always)]
    fn replace_bit(self, offset: u32, value: bool) -> Self {
        self.replace_bits(offset, offset, value as Self)
    }

    #[inline(always)]
    fn sign_extend(self, bits: u32) -> Self {
        (((self << (Self::BITS as Self - bits as Self)) as Self::Signed) >> (32 - bits)) as Self
    }
}

impl Bits for u64 {
    const BITS: u32 = 64;
    type Signed = i64;
    type Unsigned = Self;

    #[inline(always)]
    fn bits(self, start: u32, end: u32) -> Self {
        (self >> start) & ((1 << (end - start + 1)) - 1)
    }

    #[inline(always)]
    fn bits_from(self, start: u32) -> Self {
        (self >> start) & 1
    }

    #[inline(always)]
    fn bit(self, offset: u32) -> Self {
        (self >> offset) & 1
    }

    #[inline(always)]
    fn is_bit_set(self, offset: u32) -> bool {
        self.bit(offset) != 0
    }

    #[inline(always)]
    fn replace_bits(self, start: u32, end: u32, value: Self) -> Self {
        (self & !(((1 << (end - start + 1)) - 1) << start))
            | ((value & ((1 << (end - start + 1)) - 1)) << start)
    }

    #[inline(always)]
    fn replace_bit(self, offset: u32, value: bool) -> Self {
        self.replace_bits(offset, offset, value as Self)
    }

    #[inline(always)]
    fn sign_extend(self, bits: u32) -> Self {
        (((self << (Self::BITS as Self - bits as Self)) as Self::Signed) >> (32 - bits)) as Self
    }
}

impl Bits for u8 {
    const BITS: u32 = 8;
    type Signed = i8;
    type Unsigned = Self;

    #[inline(always)]
    fn bits(self, start: u32, end: u32) -> Self {
        (self >> start) & ((1 << (end - start + 1)) - 1)
    }

    #[inline(always)]
    fn bits_from(self, start: u32) -> Self {
        (self >> start) & 1
    }

    #[inline(always)]
    fn bit(self, offset: u32) -> Self {
        (self >> offset) & 1
    }

    #[inline(always)]
    fn is_bit_set(self, offset: u32) -> bool {
        self.bit(offset) != 0
    }

    #[inline(always)]
    fn replace_bits(self, start: u32, end: u32, value: Self) -> Self {
        (self & !(((1 << (end - start + 1)) - 1) << start))
            | ((value & ((1 << (end - start + 1)) - 1)) << start)
    }

    #[inline(always)]
    fn replace_bit(self, offset: u32, value: bool) -> Self {
        self.replace_bits(offset, offset, value as Self)
    }

    #[inline(always)]
    fn sign_extend(self, bits: u32) -> Self {
        (((self << (Self::BITS as Self - bits as Self)) as Self::Signed) >> (32 - bits)) as Self
    }
}
