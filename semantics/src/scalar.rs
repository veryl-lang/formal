//! Small two-state integer programs evaluated independently of compiler IR.
//! IEEE 1800-2023 11.6.1 (Table 11-21), 11.8.1..11.8.3: propagate
//! context width and operand signedness before evaluating integral operators.
//! Shift amounts, conditions, concatenation inputs, and selects are self-determined;
//! comparison operands do not inherit the assignment width.

#[derive(Clone, Copy, Debug)]
pub enum Operation {
    Add,
    Subtract,
    Multiply,
    And,
    Or,
    Xor,
    ShiftLeft,
    ShiftRight,
    ArithmeticRight,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Equal,
    Unequal,
    ChooseLowBit,
    Concatenate,
    Extend,
    MostSignificantBit,
}

/// Inputs are equal-width scalars of 1..16 bits; results occupy 1..32 bits.
pub fn evaluate(
    op: Operation,
    a: u64,
    b: u64,
    width: usize,
    result_width: usize,
    signed: bool,
) -> u64 {
    assert!((1..=16).contains(&width) && (1..=32).contains(&result_width));
    assert!(a < 1 << width && b < 1 << width);
    let decode = |n: u64| {
        if signed && n & (1 << (width - 1)) != 0 {
            n as i128 - (1 << width)
        } else {
            n as i128
        }
    };
    let sa = decode(a);
    let sb = decode(b);
    let expression_width = width.max(result_width);
    let expanded_a = sa as u64 & ((1 << expression_width) - 1);
    let result = match op {
        Operation::Add => (sa + sb) as u64,
        Operation::Subtract => (sa - sb) as u64,
        Operation::Multiply => (sa * sb) as u64,
        Operation::And => (sa & sb) as u64,
        Operation::Or => (sa | sb) as u64,
        Operation::Xor => (sa ^ sb) as u64,
        Operation::ShiftLeft => expanded_a.checked_shl(b as u32).unwrap_or(0),
        Operation::ShiftRight => expanded_a.checked_shr(b as u32).unwrap_or(0),
        Operation::ArithmeticRight => {
            sa.checked_shr(b as u32)
                .unwrap_or(if sa < 0 { -1 } else { 0 }) as u64
        }
        Operation::Less => u64::from(sa < sb),
        Operation::LessEqual => u64::from(sa <= sb),
        Operation::Greater => u64::from(sa > sb),
        Operation::GreaterEqual => u64::from(sa >= sb),
        Operation::Equal => u64::from(a == b),
        Operation::Unequal => u64::from(a != b),
        Operation::ChooseLowBit => {
            if b & 1 != 0 {
                sa as u64
            } else {
                sb as u64
            }
        }
        Operation::Concatenate => (a << width) | b,
        Operation::Extend => sa as u64,
        Operation::MostSignificantBit => a >> (width - 1),
    };
    result & ((1 << result_width) - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signed_context_extends_before_evaluation() {
        for (op, a, b, expected) in [
            (Operation::And, 15, 15, 0xff),
            (Operation::Or, 8, 1, 0xf9),
            (Operation::Xor, 15, 1, 0xfe),
            (Operation::ShiftLeft, 15, 1, 0xfe),
            (Operation::ShiftRight, 15, 1, 0x7f),
            (Operation::ArithmeticRight, 15, 1, 0xff),
            (Operation::ChooseLowBit, 15, 1, 0xff),
            (Operation::ChooseLowBit, 1, 14, 0xfe),
            (Operation::Concatenate, 15, 15, 0xff),
            (Operation::MostSignificantBit, 15, 0, 1),
        ] {
            assert_eq!(evaluate(op, a, b, 4, 8, true), expected, "{op:?}");
        }
        for op in [Operation::ShiftLeft, Operation::ShiftRight] {
            assert_eq!(evaluate(op, 15, 15, 4, 8, true), 0, "{op:?}");
        }
        assert_eq!(evaluate(Operation::ShiftRight, 15, 1, 4, 1, true), 1);
        assert_eq!(evaluate(Operation::Less, 15, 0, 4, 8, true), 1);
    }

    #[test]
    fn unsigned_context_zero_extends() {
        for op in [Operation::And, Operation::Or, Operation::ChooseLowBit] {
            assert_eq!(evaluate(op, 15, 15, 4, 8, false), 15);
        }
        assert_eq!(evaluate(Operation::ShiftLeft, 15, 1, 4, 8, false), 30);
        assert_eq!(evaluate(Operation::ShiftRight, 15, 1, 4, 8, false), 7);
    }
}
