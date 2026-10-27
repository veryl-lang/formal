//! Normalized native operands, matching bitwise widths, and symbolic supported operator
//! choices. Checks bitwise, logical, and reduction tables plus internal metadata.
//! Does not prove operand evaluation effects, short-circuiting, or caller context.

use veryl_analyzer::value::ValueU64;
use veryl_formal_semantics::gates::{self, BitOp, Reduction};
use veryl_formal_semantics::logic4::{Logic4, read_logic};

fn operand(width: u32) -> ValueU64 {
    let mask = u64::MAX >> (64 - width);
    ValueU64 {
        payload: kani::any::<u64>() & mask,
        mask_xz: kani::any::<u64>() & mask,
        width,
        signed: kani::any(),
    }
}

fn width() -> u32 {
    u32::from(kani::any::<u8>() & 63) + 1
}
fn planes(value: &ValueU64) -> (u128, u128) {
    (value.payload as u128, value.mask_xz as u128)
}

#[kani::proof]
#[kani::unwind(1)]
fn native_bitwise_matches_reference_semantics() {
    let width = width();
    let x = operand(width);
    let y = operand(width);
    let mode = kani::any::<u8>() & 3;
    let (output, kind) = match mode {
        0 => (x.bit_and(&y, width as usize), BitOp::And),
        1 => (x.bit_or(&y, width as usize), BitOp::Or),
        2 => (x.bit_xor(&y, width as usize), BitOp::Xor),
        _ => (x.bit_xnor(&y, width as usize), BitOp::Xnor),
    };
    let position = usize::from(kani::any::<u8>() & 63);
    let a = read_logic(planes(&x).0, planes(&x).1, position);
    let b = read_logic(planes(&y).0, planes(&y).1, position);
    let expected = if position < width as usize {
        gates::binary(a, b, kind)
    } else {
        Logic4::Zero
    };
    assert!(
        read_logic(planes(&output).0, planes(&output).1, position) == expected,
        "bitwise semantics mismatch"
    );
    assert!(
        output.width == width && !output.signed,
        "bitwise semantics mismatch"
    );
    kani::cover!(mode == 0, "bitwise AND is reachable");
    kani::cover!(mode == 1, "bitwise OR is reachable");
    kani::cover!(mode == 2, "bitwise XOR is reachable");
    kani::cover!(mode == 3, "bitwise XNOR is reachable");
    kani::cover!(width == 1, "one-bit bitwise operation is reachable");
    kani::cover!(width == 64, "width-64 bitwise operation is reachable");
    kani::cover!(position >= width as usize, "upper stored bit is reachable");
    kani::cover!(
        mode == 0 && a == Logic4::Zero && matches!(b, Logic4::X | Logic4::Z),
        "AND zero dominates RHS unknown"
    );
    kani::cover!(
        mode == 0 && b == Logic4::Zero && matches!(a, Logic4::X | Logic4::Z),
        "AND zero dominates LHS unknown"
    );
    kani::cover!(
        mode == 1 && a == Logic4::One && matches!(b, Logic4::X | Logic4::Z),
        "OR one dominates RHS unknown"
    );
    kani::cover!(
        mode == 1 && b == Logic4::One && matches!(a, Logic4::X | Logic4::Z),
        "OR one dominates LHS unknown"
    );
    kani::cover!(expected == Logic4::Zero, "bitwise zero is reachable");
    kani::cover!(expected == Logic4::One, "bitwise one is reachable");
    kani::cover!(expected == Logic4::X, "bitwise X is reachable");
    kani::cover!(
        a == Logic4::Z && position < width as usize,
        "bitwise source Z is reachable"
    );
}

#[kani::proof]
#[kani::unwind(1)]
fn native_bit_not_matches_reference_semantics() {
    let width = width();
    let x = operand(width);
    let position = usize::from(kani::any::<u8>() & 63);
    let a = read_logic(planes(&x).0, planes(&x).1, position);
    let expected = if position < width as usize {
        gates::not(a)
    } else {
        Logic4::Zero
    };
    let output = x.bit_not(width as usize);
    assert!(
        read_logic(planes(&output).0, planes(&output).1, position) == expected,
        "bit-not semantics mismatch"
    );
    assert!(
        output.width == width && output.signed == x.signed,
        "bit-not semantics mismatch"
    );
    kani::cover!(width == 1, "one-bit complement is reachable");
    kani::cover!(width == 64, "width-64 complement is reachable");
    kani::cover!(x.signed, "signed complement is reachable");
    kani::cover!(
        position >= width as usize,
        "complement upper stored bit is reachable"
    );
    kani::cover!(
        position < width as usize && a == Logic4::Zero,
        "complement source zero is reachable"
    );
    kani::cover!(a == Logic4::One, "complement source one is reachable");
    kani::cover!(a == Logic4::X, "complement source X is reachable");
    kani::cover!(a == Logic4::Z, "complement source Z is reachable");
}

#[kani::proof]
#[kani::unwind(65)]
fn native_logical_matches_reference_semantics() {
    let x = operand(width());
    let y = operand(width());
    let conjunction: bool = kani::any();
    let a = gates::logical_truth(planes(&x), x.width as usize);
    let b = gates::logical_truth(planes(&y), y.width as usize);
    let expected = gates::binary(a, b, if conjunction { BitOp::And } else { BitOp::Or });
    let output = x.logical_binary(&y, conjunction);
    assert!(
        read_logic(planes(&output).0, planes(&output).1, 0) == expected,
        "logical semantics mismatch"
    );
    assert!(
        output.payload <= 1 && output.mask_xz <= 1 && output.width == 1 && !output.signed,
        "logical semantics mismatch"
    );
    kani::cover!(conjunction, "logical AND is reachable");
    kani::cover!(!conjunction, "logical OR is reachable");
    kani::cover!(
        x.width == 64 && y.width == 64,
        "width-64 logical operation is reachable"
    );
    kani::cover!(
        x.width != y.width,
        "unequal-width logical operation is reachable"
    );
    kani::cover!(
        conjunction && a == Logic4::Zero && b == Logic4::X,
        "logical AND zero dominates RHS unknown"
    );
    kani::cover!(
        conjunction && b == Logic4::Zero && a == Logic4::X,
        "logical AND zero dominates LHS unknown"
    );
    kani::cover!(
        !conjunction && a == Logic4::One && b == Logic4::X,
        "logical OR one dominates RHS unknown"
    );
    kani::cover!(
        !conjunction && b == Logic4::One && a == Logic4::X,
        "logical OR one dominates LHS unknown"
    );
    kani::cover!(
        (x.mask_xz != 0 || y.mask_xz != 0) && a == Logic4::One && b == Logic4::One,
        "known truth despite XZ is reachable"
    );
    kani::cover!(expected == Logic4::Zero, "logical zero is reachable");
    kani::cover!(expected == Logic4::One, "logical one is reachable");
    kani::cover!(expected == Logic4::X, "logical X is reachable");
}

#[kani::proof]
#[kani::unwind(65)]
fn native_reductions_matches_reference_semantics() {
    let x = operand(width());
    let mode = kani::any::<u8>() & 7;
    kani::assume(mode < 7);
    let (output, kind) = match mode {
        0 => (x.reduce_and(false), Reduction::And),
        1 => (x.reduce_and(true), Reduction::Nand),
        2 => (x.reduce_or(false), Reduction::Or),
        3 => (x.reduce_or(true), Reduction::Nor),
        4 => (x.reduce_xor(false), Reduction::Xor),
        5 => (x.reduce_xor(true), Reduction::Xnor),
        // Logical NOT of a nonempty vector is the same scalar fold as NOR.
        _ => (x.reduce_or(true), Reduction::Nor),
    };
    let expected = gates::reduced_logic(planes(&x), x.width as usize, kind);
    assert!(
        read_logic(planes(&output).0, planes(&output).1, 0) == expected,
        "reduction semantics mismatch"
    );
    assert!(
        output.payload <= 1 && output.mask_xz <= 1 && output.width == 1 && !output.signed,
        "reduction semantics mismatch"
    );
    kani::cover!(mode == 0, "reduction AND is reachable");
    kani::cover!(mode == 1, "reduction NAND is reachable");
    kani::cover!(mode == 2, "reduction OR is reachable");
    kani::cover!(mode == 3, "reduction NOR is reachable");
    kani::cover!(mode == 4, "reduction XOR is reachable");
    kani::cover!(mode == 5, "reduction XNOR is reachable");
    kani::cover!(mode == 6, "logical NOT is reachable");
    kani::cover!(x.width == 1, "one-bit reduction is reachable");
    kani::cover!(x.width == 64, "width-64 reduction is reachable");
    kani::cover!(
        mode == 0 && x.mask_xz != 0 && expected == Logic4::Zero,
        "reduction zero dominates unknown"
    );
    kani::cover!(
        mode == 2 && x.mask_xz != 0 && expected == Logic4::One,
        "reduction one dominates unknown"
    );
    kani::cover!(
        mode == 4 && x.mask_xz == 0 && expected == Logic4::Zero,
        "even parity is reachable"
    );
    kani::cover!(
        mode == 4 && expected == Logic4::One,
        "odd parity is reachable"
    );
    kani::cover!(expected == Logic4::X, "reduction X is reachable");
    kani::cover!(
        x.mask_xz != 0 && x.payload & x.mask_xz != 0,
        "reduction source Z is reachable"
    );
}
