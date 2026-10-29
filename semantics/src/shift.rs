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
    shifted_logic_bit_from(width, amount, kind, signed, position, |p| {
        read_logic(source.0, source.1, p)
    })
}

/// Compose context sizing before shifting without materializing every resized bit.
/// Source and effective expression signedness are the same, as in the public
/// shift contexts checked here. Narrowing contexts are outside this model.
/// IEEE 1800 11.6.1/11.8.2 and 11.4.10 apply in that order.
pub fn shifted_resized_logic_bit(
    source: (u128, u128),
    source_width: usize,
    width: usize,
    amount: Option<usize>,
    kind: Shift,
    signed: bool,
    position: usize,
) -> Logic4 {
    assert!((1..=128).contains(&source_width) && source_width <= width);
    shifted_logic_bit_from(width, amount, kind, signed, position, |p| {
        crate::resize::extended_logic_bit(source, source_width, width, signed, signed, p)
    })
}

fn shifted_logic_bit_from(
    width: usize,
    amount: Option<usize>,
    kind: Shift,
    signed: bool,
    position: usize,
    mut source: impl FnMut(usize) -> Logic4,
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
                source(position - amount)
            } else {
                Logic4::Zero
            }
        }
        Shift::LogicalRight | Shift::ArithmeticRight => {
            if amount < width - position {
                source(position + amount)
            } else if kind == Shift::ArithmeticRight && signed {
                source(width - 1)
            } else {
                Logic4::Zero
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Shift, shifted_logic_bit, shifted_resized_logic_bit};
    use crate::resize::{extended_logic_bit, reference_planes};

    #[test]
    fn composed_sizing_matches_materialized_planes() {
        for source_width in 1..=4 {
            let mask = (1u128 << source_width) - 1;
            for state in 0..(1u128 << (2 * source_width)) {
                let source = (state & mask, state >> source_width);
                for width in [source_width, 8, 64, 128] {
                    for signed in [false, true] {
                        let prepared = reference_planes(width, |p| {
                            extended_logic_bit(source, source_width, width, signed, signed, p)
                        });
                        for amount in [
                            None,
                            Some(0),
                            Some(1),
                            Some(source_width),
                            Some(width),
                            Some(width + 1),
                            Some(usize::MAX),
                        ] {
                            for kind in [
                                Shift::LogicalLeft,
                                Shift::LogicalRight,
                                Shift::ArithmeticLeft,
                                Shift::ArithmeticRight,
                            ] {
                                for position in 0..=128 {
                                    assert_eq!(
                                        shifted_resized_logic_bit(
                                            source,
                                            source_width,
                                            width,
                                            amount,
                                            kind,
                                            signed,
                                            position
                                        ),
                                        shifted_logic_bit(
                                            prepared, width, amount, kind, signed, position
                                        ),
                                        "source={source:?}, source_width={source_width}, width={width}, amount={amount:?}, kind={kind:?}, signed={signed}, position={position}"
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
