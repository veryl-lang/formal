//! IEEE 1800-2023 rule mapping:
//! SV-ARITH (11.4.3, 11.4.3.1, 11.6.1): Add/subtract and negate in two's-complement arithmetic; discard carry beyond the prepared width; any X/Z operand yields all X.
//! SV-UPLUS (11.4.3, Table 11-6): This model takes "same as m" as identity,
//! including X/Z, after the width/sign preparation of 11.6.1 and 11.8.2.
//! The following general arithmetic X/Z sentence does not state an exception.
//! Their precedence is unresolved here: X/Z unary-plus equivalence checks this
//! interpretation, not an unqualified claim of standard conformance.
//!
//! Prepared views take normalized, already-sized operands. The contextual view
//! takes source records and caller-resolved width/sign. Carry beyond the result
//! width is discarded.

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

/// Contextual SV-ARITH/SV-UPLUS view with preparation from 11.6.1 and 11.8.2.
/// Sources are normalized four-state vectors, widths 1..=128, with a caller
/// width at least as large as each relevant source. Caller signedness controls
/// extension; expression-context resolution and result metadata are separate.
/// Add/subtract/negate inspect every prepared bit: an unknown above the observed
/// position also poisons an in-range result. No plane vectors are materialized.
pub fn contextual_arithmetic_bit(
    a: crate::comparison::Operand,
    b: crate::comparison::Operand,
    width: usize,
    signed: bool,
    kind: Arithmetic,
    position: usize,
) -> Logic4 {
    use crate::resize::extended_logic_bit;
    assert!((1..=128).contains(&width));
    assert!((1..=width).contains(&a.width));
    let binary = matches!(kind, Arithmetic::Add | Arithmetic::Subtract);
    if binary {
        assert!((1..=width).contains(&b.width));
    }
    if position >= width {
        return Logic4::Zero;
    }
    let bit = |input: crate::comparison::Operand, index| {
        extended_logic_bit(
            input.planes,
            input.width,
            width,
            input.signed,
            signed,
            index,
        )
    };
    if kind == Arithmetic::Plus {
        return bit(a, position);
    }
    let subtract = matches!(kind, Arithmetic::Subtract | Arithmetic::Negate);
    let mut carry = subtract;
    let mut poison = false;
    let mut result = Logic4::Zero;
    for index in 0..width {
        let (lhs, rhs) = if binary {
            (bit(a, index), bit(b, index))
        } else {
            (Logic4::Zero, bit(a, index))
        };
        poison |= matches!(lhs, Logic4::X | Logic4::Z) || matches!(rhs, Logic4::X | Logic4::Z);
        let x = matches!(lhs, Logic4::One | Logic4::Z);
        let y = matches!(rhs, Logic4::One | Logic4::Z) != subtract;
        let sum = x ^ y ^ carry;
        carry = (x && y) || (x && carry) || (y && carry);
        if index == position {
            result = if sum { Logic4::One } else { Logic4::Zero };
        }
    }
    if poison { Logic4::X } else { result }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contextual_bits_match_prepared_small_planes() {
        use crate::comparison::Operand;
        use crate::resize::{extended_logic_bit, reference_planes};
        let mut inputs = Vec::new();
        for width in 1..=3 {
            let mask = (1u128 << width) - 1;
            for state in 0..(1u128 << (2 * width)) {
                for signed in [false, true] {
                    inputs.push(Operand {
                        planes: (state & mask, state >> width),
                        width,
                        signed,
                    });
                }
            }
        }
        let prepare = |input: Operand, width, signed| {
            reference_planes(width, |position| {
                extended_logic_bit(
                    input.planes,
                    input.width,
                    width,
                    input.signed,
                    signed,
                    position,
                )
            })
        };
        let check = |a: Operand, b: Operand, width, signed, kind| {
            let ap = prepare(a, width, signed);
            let bp = if matches!(kind, Arithmetic::Add | Arithmetic::Subtract) {
                prepare(b, width, signed)
            } else {
                (0, 0)
            };
            let planes = arithmetic_planes(ap, bp, width, kind);
            for position in 0..128 {
                assert_eq!(
                    contextual_arithmetic_bit(a, b, width, signed, kind, position),
                    read_logic(planes.0, planes.1, position)
                );
            }
        };
        let zero = Operand {
            planes: (0, 0),
            width: 1,
            signed: false,
        };
        for &a in &inputs {
            for signed in [false, true] {
                for growth in [0, 2] {
                    check(a, zero, a.width + growth, signed, Arithmetic::Negate);
                    check(a, zero, a.width + growth, signed, Arithmetic::Plus);
                    for &b in &inputs {
                        let width = a.width.max(b.width) + growth;
                        check(a, b, width, signed, Arithmetic::Add);
                        check(a, b, width, signed, Arithmetic::Subtract);
                    }
                }
            }
        }
    }

    #[test]
    fn unary_plus_identity_covers_all_small_four_state_vectors() {
        use Logic4::{One, X, Z, Zero};
        let encoding = [Zero, One, X, Z];
        for width in 1..=5 {
            let limit = 1u128 << width;
            for payload in 0..limit {
                for mask_xz in 0..limit {
                    let input = (payload, mask_xz);
                    assert_eq!(
                        arithmetic_planes(input, (0, 0), width, Arithmetic::Plus),
                        input
                    );
                    for position in 0..128 {
                        let expected = if position < width {
                            let code =
                                ((payload >> position) & 1) + 2 * ((mask_xz >> position) & 1);
                            encoding[code as usize]
                        } else {
                            Zero
                        };
                        assert_eq!(
                            arithmetic_bit(input, (0, 0), width, Arithmetic::Plus, position),
                            expected
                        );
                    }
                }
            }
        }
    }
}
