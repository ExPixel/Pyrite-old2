pub trait Bits {
    fn bits(self, start: u32, end: u32) -> Self;
    fn bits_from(self, start: u32) -> Self;
    fn bit(self, offset: u32) -> Self;
    #[allow(clippy::wrong_self_convention)]
    fn is_bit_set(self, offset: u32) -> bool;
    fn replace_bits(self, start: u32, end: u32, value: Self) -> Self;
    fn replace_bit(self, offset: u32, value: Self) -> Self;
    fn sign_extend(self, bits: u32) -> Self;
}

impl Bits for u32 {
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
    fn replace_bit(self, offset: u32, value: Self) -> Self {
        self.replace_bits(offset, offset, value & 1)
    }

    #[inline(always)]
    fn sign_extend(self, bits: u32) -> Self {
        (((self << (32 - bits)) as i32) >> (32 - bits)) as u32
    }
}

impl Bits for u16 {
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
    fn replace_bit(self, offset: u32, value: Self) -> Self {
        self.replace_bits(offset, offset, value & 1)
    }

    #[inline(always)]
    fn sign_extend(self, bits: u32) -> Self {
        (((self << (16 - bits as u16)) as i16) >> (16 - bits as u16)) as u16
    }
}
