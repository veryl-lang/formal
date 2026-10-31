//! Exact nonnegative 256-bit arithmetic for bounded numeric API abstractions.
//! Two native halves retain assignment/shift intermediates above bit 127. This
//! is a numeric representation, not the oracle for compiler operations; Kani
//! checks bit projections and shifts independently against scalar coordinates.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Bits256 {
    pub lo: u128,
    pub hi: u128,
}
impl Bits256 {
    pub const ZERO: Self = Self { lo: 0, hi: 0 };
    pub fn from_u128(lo: u128) -> Self {
        Self { lo, hi: 0 }
    }
    pub fn bit(self, position: usize) -> bool {
        if position < 128 {
            self.lo >> position & 1 != 0
        } else if position < 256 {
            self.hi >> (position - 128) & 1 != 0
        } else {
            false
        }
    }
    pub fn bits(self) -> u64 {
        if self.hi != 0 {
            256 - u64::from(self.hi.leading_zeros())
        } else {
            128 - u64::from(self.lo.leading_zeros())
        }
    }
    pub fn and(self, b: Self) -> Self {
        Self {
            lo: self.lo & b.lo,
            hi: self.hi & b.hi,
        }
    }
    pub fn or(self, b: Self) -> Self {
        Self {
            lo: self.lo | b.lo,
            hi: self.hi | b.hi,
        }
    }
    pub fn xor(self, b: Self) -> Self {
        Self {
            lo: self.lo ^ b.lo,
            hi: self.hi ^ b.hi,
        }
    }
    pub fn mask(width: usize) -> Self {
        assert!(width <= 256);
        if width == 0 {
            Self::ZERO
        } else if width <= 128 {
            Self::from_u128(u128::MAX >> (128 - width))
        } else {
            Self {
                lo: u128::MAX,
                hi: u128::MAX >> (256 - width),
            }
        }
    }
    pub fn shr(self, shift: usize) -> Self {
        if shift == 0 {
            self
        } else if shift < 128 {
            Self {
                lo: (self.lo >> shift) | (self.hi << (128 - shift)),
                hi: self.hi >> shift,
            }
        } else if shift < 256 {
            Self::from_u128(self.hi >> (shift - 128))
        } else {
            Self::ZERO
        }
    }
    pub fn shl(self, shift: usize) -> Self {
        if shift == 0 {
            self
        } else if shift < 128 {
            Self {
                lo: self.lo << shift,
                hi: (self.hi << shift) | (self.lo >> (128 - shift)),
            }
        } else if shift < 256 {
            Self {
                lo: 0,
                hi: self.lo << (shift - 128),
            }
        } else {
            Self::ZERO
        }
    }
    /// BigUint is unbounded: fail instead of silently modelling overflow modulo 256.
    pub fn checked_shl(self, shift: usize) -> Self {
        let result = self.shl(shift);
        assert!(result.shr(shift) == self, "256-bit numeric model overflow");
        result
    }
}
