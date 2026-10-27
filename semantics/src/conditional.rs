//! IEEE 1800-2023 rule mapping:
//! SV-CONDITIONAL (11.4.11): Select definite branches; ambiguous integral conditions combine branch values.
//! SV-CONDITIONAL-SIZE (11.6.1, 11.8.2, 11.8.3): Prepare branch widths/signs in expression context before combining values.
//!
//! Table 11-20 preserves only common 0/1 bits; common Z yields X. Branch context
//! is resolved by the caller. Statement-if ambiguity and optimizer reachability
//! are separate rules.

use crate::logic4::Logic4;
use crate::resize::extended_logic_bit;

pub fn merged_logic(condition: Logic4, on_true: Logic4, on_false: Logic4) -> Logic4 {
    match condition {
        Logic4::One => on_true,
        Logic4::Zero => on_false,
        Logic4::X | Logic4::Z => {
            if on_true == on_false && matches!(on_true, Logic4::Zero | Logic4::One) {
                on_true
            } else {
                Logic4::X
            }
        }
    }
}

pub fn conditional_logic_bit(
    condition: Logic4,
    on_true: ((u128, u128), usize, bool),
    on_false: ((u128, u128), usize, bool),
    output_width: usize,
    signed: bool,
    position: usize,
) -> Logic4 {
    if position >= output_width {
        return Logic4::Zero;
    }
    let a = extended_logic_bit(
        on_true.0,
        on_true.1,
        output_width,
        on_true.2,
        signed,
        position,
    );
    let b = extended_logic_bit(
        on_false.0,
        on_false.1,
        output_width,
        on_false.2,
        signed,
        position,
    );
    merged_logic(condition, a, b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ambiguous_conditions_follow_table_11_20() {
        use Logic4::{One as I, X, Z, Zero as O};
        let bits = [O, I, X, Z];
        let table = [[O, X, X, X], [X, I, X, X], [X, X, X, X], [X, X, X, X]];
        for condition in [X, Z] {
            for (i, a) in bits.into_iter().enumerate() {
                for (j, b) in bits.into_iter().enumerate() {
                    assert_eq!(merged_logic(condition, a, b), table[i][j]);
                }
            }
        }
        assert_eq!(merged_logic(I, Z, O), Z);
        assert_eq!(merged_logic(O, O, Z), Z);
    }
}
