//! IEEE 1800-2023 rule mapping:
//! SV-ARITH (11.4.3, 11.4.3.1, 11.6.1): Add/subtract and negate in two's-complement arithmetic; discard carry beyond the prepared width; any X/Z operand yields all X.
//! SV-UPLUS (11.4.3, Table 11-6): This model takes "same as m" as identity,
//! including X/Z, after the width/sign preparation of 11.6.1 and 11.8.2.
//! The following general arithmetic X/Z sentence does not state an exception.
//! Their precedence is unresolved here: X/Z unary-plus equivalence checks this
//! interpretation, not an unqualified claim of standard conformance.
//!
//! Operands have normalized storage and caller-resolved width/sign. Carry is discarded
//! beyond the result width.

use crate::logic4::{Logic4, logic_from_planes, read_logic};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Arithmetic {
    Add,
    Subtract,
    Negate,
    Plus,
}

fn fold_arithmetic(
    a: (u128, u128),
    b: (u128, u128),
    width: usize,
    kind: Arithmetic,
    mut bit: impl FnMut(usize, Logic4),
) -> bool {
    assert!((1..=128).contains(&width));
    let (mut lhs, mut rhs) = match kind {
        Arithmetic::Add | Arithmetic::Subtract => (a, b),
        Arithmetic::Negate => ((0, 0), a),
        Arithmetic::Plus => (a, (0, 0)),
    };
    let subtract = matches!(kind, Arithmetic::Subtract | Arithmetic::Negate);
    let mut carry = subtract;
    let mut poison = false;
    for position in 0..width {
        let x = lhs.0 & 1 != 0;
        let xu = lhs.1 & 1 != 0;
        let y = rhs.0 & 1 != 0;
        let yu = rhs.1 & 1 != 0;
        if kind == Arithmetic::Plus {
            bit(position, logic_from_planes(x, xu));
        } else {
            poison |= xu || yu;
            let y = y != subtract;
            let sum = x ^ y ^ carry;
            let next_carry = (x && y) || (x && carry) || (y && carry);
            bit(position, if sum { Logic4::One } else { Logic4::Zero });
            carry = next_carry;
        }
        lhs.0 >>= 1;
        lhs.1 >>= 1;
        rhs.0 >>= 1;
        rhs.1 >>= 1;
    }
    poison
}

pub fn arithmetic_planes(
    a: (u128, u128),
    b: (u128, u128),
    width: usize,
    kind: Arithmetic,
) -> (u128, u128) {
    let mut payload = 0;
    let mut mask = 0;
    let poison = fold_arithmetic(a, b, width, kind, |position, bit| {
        if matches!(bit, Logic4::One | Logic4::Z) {
            payload |= 1u128 << position;
        }
        if matches!(bit, Logic4::X | Logic4::Z) {
            mask |= 1u128 << position;
        }
    });
    if poison {
        (0, u128::MAX >> (128 - width))
    } else {
        (payload, mask)
    }
}

pub fn arithmetic_bit(
    a: (u128, u128),
    b: (u128, u128),
    width: usize,
    kind: Arithmetic,
    position: usize,
) -> Logic4 {
    assert!((1..=128).contains(&width));
    if position >= width {
        return Logic4::Zero;
    }
    if kind == Arithmetic::Plus {
        return read_logic(a.0, a.1, position);
    }
    let mut result = Logic4::Zero;
    let poison = fold_arithmetic(a, b, width, kind, |index, bit| {
        if index == position {
            result = bit;
        }
    });
    if poison { Logic4::X } else { result }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unary_plus_identity_interpretation_preserves_four_state_bits() {
        for input in [(1, 2), (3, 2), (0, 3), (3, 3), (3, 0)] {
            assert_eq!(arithmetic_planes(input, (0, 0), 2, Arithmetic::Plus), input);
            for bit in 0..=2 {
                assert_eq!(
                    arithmetic_bit(input, (0, 0), 2, Arithmetic::Plus, bit),
                    read_logic(input.0, input.1, bit)
                );
            }
        }
    }
}
