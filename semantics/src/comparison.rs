//! IEEE 1800-2023 rule mapping:
//! SV-EQUALITY (11.4.5): A known unequal bit decides logical equality; an otherwise ambiguous X/Z comparison yields X. Inequality complements known results.
//! SV-WILDCARD (11.4.6): Only RHS X/Z bits are wildcards; remaining bits follow logical equality.
//! SV-RELATION (11.4.4): Relational comparison yields X for any X/Z operand; otherwise order signed or unsigned values.
//! SV-COMPARE-SIZE (11.4.4, 11.4.5, 11.4.6): Use common operand width; sign-extend only if both operands are signed, otherwise zero-extend.
//!
//! Operands are widened to their common width, sign-extending only when both are signed.
//! Case equality, floating point, and width-zero all-bit sentinels are outside this model.

use crate::logic4::{Logic4, logic_from_planes, read_logic};

#[derive(Clone, Copy, Debug)]
pub struct Operand {
    pub planes: (u128, u128),
    pub width: usize,
    pub signed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Comparison {
    Equal,
    NotEqual,
    WildcardEqual,
    WildcardNotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}

pub fn compared_logic(x: Operand, y: Operand, kind: Comparison) -> Logic4 {
    assert!((1..=128).contains(&x.width));
    assert!((1..=128).contains(&y.width));
    let width = x.width.max(y.width);
    let signed = x.signed && y.signed;
    let equality = matches!(
        kind,
        Comparison::Equal
            | Comparison::NotEqual
            | Comparison::WildcardEqual
            | Comparison::WildcardNotEqual
    );
    let wildcard = matches!(
        kind,
        Comparison::WildcardEqual | Comparison::WildcardNotEqual
    );
    let sign_x = read_logic(x.planes.0, x.planes.1, x.width - 1);
    let sign_y = read_logic(y.planes.0, y.planes.1, y.width - 1);
    let mut bits_x = x.planes;
    let mut bits_y = y.planes;
    let mut unknown = false;
    let mut mismatch = false;
    let mut order = std::cmp::Ordering::Equal;
    // Ascending traversal lets a more significant unequal bit override lower bits.
    for position in 0..width {
        let a = if position < x.width {
            logic_from_planes(bits_x.0 & 1 != 0, bits_x.1 & 1 != 0)
        } else if signed {
            sign_x
        } else {
            Logic4::Zero
        };
        let b = if position < y.width {
            logic_from_planes(bits_y.0 & 1 != 0, bits_y.1 & 1 != 0)
        } else if signed {
            sign_y
        } else {
            Logic4::Zero
        };
        bits_x.0 >>= 1;
        bits_x.1 >>= 1;
        bits_y.0 >>= 1;
        bits_y.1 >>= 1;
        if equality && wildcard && matches!(b, Logic4::X | Logic4::Z) {
            continue;
        }
        if matches!(a, Logic4::X | Logic4::Z) || matches!(b, Logic4::X | Logic4::Z) {
            unknown = true;
        } else if a != b {
            mismatch = true;
            let negative_weight = signed && position == width - 1;
            order = if (a == Logic4::One) != negative_weight {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Less
            };
        }
    }
    let result = if equality {
        if mismatch {
            false
        } else if unknown {
            return Logic4::X;
        } else {
            true
        }
    } else {
        if unknown {
            return Logic4::X;
        }
        match kind {
            Comparison::Less => order.is_lt(),
            Comparison::LessEqual => order.is_le(),
            Comparison::Greater => order.is_gt(),
            Comparison::GreaterEqual => order.is_ge(),
            _ => unreachable!(),
        }
    };
    let negate = matches!(kind, Comparison::NotEqual | Comparison::WildcardNotEqual);
    if result != negate {
        Logic4::One
    } else {
        Logic4::Zero
    }
}
