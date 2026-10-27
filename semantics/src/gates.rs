//! IEEE 1800-2023 rule mapping:
//! SV-BITWISE (11.4.8, Tables 11-11..11-15): Apply scalar four-state tables per bit; NOT maps X/Z to X.
//! SV-BOOLEAN (11.4.7): Known nonzero is true, known zero is false, otherwise unknown; combine truth values and negate them.
//! SV-REDUCTION (11.4.9, Tables 11-16..11-19): Fold the scalar tables; NAND/NOR/XNOR invert the corresponding fold.
//!
//! Operands are evaluated values. Short-circuit effects and source operand restrictions
//! are not modeled by scalar truth tables.

use crate::logic4::{Logic4, logic_from_planes};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BitOp {
    And,
    Or,
    Xor,
    Xnor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reduction {
    And,
    Nand,
    Or,
    Nor,
    Xor,
    Xnor,
}

pub fn not(a: Logic4) -> Logic4 {
    match a {
        Logic4::Zero => Logic4::One,
        Logic4::One => Logic4::Zero,
        Logic4::X | Logic4::Z => Logic4::X,
    }
}

pub fn binary(a: Logic4, b: Logic4, kind: BitOp) -> Logic4 {
    use Logic4::*;
    match kind {
        BitOp::And => match (a, b) {
            (Zero, _) | (_, Zero) => Zero,
            (One, One) => One,
            _ => X,
        },
        BitOp::Or => match (a, b) {
            (One, _) | (_, One) => One,
            (Zero, Zero) => Zero,
            _ => X,
        },
        BitOp::Xor | BitOp::Xnor => {
            let xor = match (a, b) {
                (Zero, Zero) | (One, One) => Zero,
                (Zero, One) | (One, Zero) => One,
                _ => X,
            };
            if kind == BitOp::Xnor { not(xor) } else { xor }
        }
    }
}

pub fn reduced_logic(mut source: (u128, u128), width: usize, kind: Reduction) -> Logic4 {
    assert!((1..=128).contains(&width));
    let op = match kind {
        Reduction::And | Reduction::Nand => BitOp::And,
        Reduction::Or | Reduction::Nor => BitOp::Or,
        Reduction::Xor | Reduction::Xnor => BitOp::Xor,
    };
    let mut result = if op == BitOp::And {
        Logic4::One
    } else {
        Logic4::Zero
    };
    for _ in 0..width {
        result = binary(
            result,
            logic_from_planes(source.0 & 1 != 0, source.1 & 1 != 0),
            op,
        );
        source.0 >>= 1;
        source.1 >>= 1;
    }
    if matches!(kind, Reduction::Nand | Reduction::Nor | Reduction::Xnor) {
        not(result)
    } else {
        result
    }
}

pub fn logical_truth(source: (u128, u128), width: usize) -> Logic4 {
    reduced_logic(source, width, Reduction::Or)
}
