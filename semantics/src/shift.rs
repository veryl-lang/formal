//! IEEE 1800-2023 rule mapping:
//! SV-SHIFT (11.4.10): Left and logical-right shifts zero-fill; signed arithmetic-right shifts sign-fill. X/Z amount yields unknown; amount is unsigned.
//!
//! Inputs are already sized; unknown decoded amounts are distinct from oversized known
//! amounts. The caller chooses result signedness and decodes source expressions.

use crate::logic4::{Logic4, read_logic};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shift {
    LogicalLeft,
    LogicalRight,
    ArithmeticLeft,
    ArithmeticRight,
}

pub fn shifted_logic_bit(
    source: (u128, u128),
    width: usize,
    amount: Option<usize>,
    kind: Shift,
    signed: bool,
    position: usize,
) -> Logic4 {
    assert!((1..=128).contains(&width));
    if position >= width {
        return Logic4::Zero;
    }
    let Some(amount) = amount else {
        return Logic4::X;
    };
    match kind {
        Shift::LogicalLeft | Shift::ArithmeticLeft => {
            if amount <= position {
                read_logic(source.0, source.1, position - amount)
            } else {
                Logic4::Zero
            }
        }
        Shift::LogicalRight | Shift::ArithmeticRight => {
            if amount < width - position {
                read_logic(source.0, source.1, position + amount)
            } else if kind == Shift::ArithmeticRight && signed {
                read_logic(source.0, source.1, width - 1)
            } else {
                Logic4::Zero
            }
        }
    }
}
