//! Normalized four-state widths 1..=64 with arbitrary decoded known/unknown amounts.
//! Checks logical/arithmetic bit origins and internal result metadata. Caller amount
//! decoding, source context resolution, and arbitrary-width BigUint are not proved.

use veryl_analyzer::value::ValueU64;
use veryl_formal_semantics::logic4::{Logic4, read_logic};
use veryl_formal_semantics::shift::{Shift, shifted_logic_bit};

fn check_shift(kind: Shift) -> (ValueU64, Option<usize>, usize, Logic4) {
    let width = u32::from(kani::any::<u8>() & 63) + 1;
    let mask = u64::MAX >> (64 - width);
    let input = ValueU64 {
        payload: kani::any::<u64>() & mask,
        mask_xz: kani::any::<u64>() & mask,
        width,
        signed: kani::any(),
    };
    let amount: Option<usize> = kani::any();
    let position = usize::from(kani::any::<u8>() & 63);
    let expected = shifted_logic_bit(
        (input.payload as u128, input.mask_xz as u128),
        width as usize,
        amount,
        kind,
        input.signed,
        position,
    );
    let arithmetic = matches!(kind, Shift::ArithmeticLeft | Shift::ArithmeticRight);
    let output = match kind {
        Shift::LogicalLeft | Shift::ArithmeticLeft => {
            input.shift_left(amount, width as usize, arithmetic)
        }
        Shift::LogicalRight | Shift::ArithmeticRight => {
            input.shift_right(amount, width as usize, input.signed, arithmetic)
        }
    };
    assert!(
        read_logic(output.payload as u128, output.mask_xz as u128, position) == expected,
        "shift semantics mismatch"
    );
    assert!(output.width == width, "shift semantics mismatch");
    assert!(
        output.signed == (amount.is_some() && input.signed && arithmetic),
        "shift semantics mismatch"
    );
    kani::cover!(width == 1, "one-bit shift is reachable");
    kani::cover!(width == 64, "width-64 shift is reachable");
    kani::cover!(amount == Some(0), "zero shift is reachable");
    kani::cover!(
        amount == Some(width as usize - 1),
        "last-bit shift is reachable"
    );
    kani::cover!(
        amount == Some(width as usize),
        "width-sized shift is reachable"
    );
    kani::cover!(amount == Some(64), "shift by 64 is reachable");
    kani::cover!(amount == Some(usize::MAX), "maximum shift is reachable");
    kani::cover!(amount.is_none(), "unknown shift amount is reachable");
    kani::cover!(
        amount.is_some() && !input.signed,
        "known unsigned shift is reachable"
    );
    kani::cover!(
        amount.is_some() && input.signed,
        "known signed shift is reachable"
    );
    kani::cover!(
        amount.is_some() && expected == Logic4::X,
        "shifted source X is reachable"
    );
    kani::cover!(
        amount.is_some() && expected == Logic4::Z,
        "shifted source Z is reachable"
    );
    (input, amount, position, expected)
}

#[kani::proof]
#[kani::unwind(1)]
fn native_logical_left_shift_matches_reference_semantics() {
    check_shift(Shift::LogicalLeft);
}

#[kani::proof]
#[kani::unwind(1)]
fn native_logical_right_shift_matches_reference_semantics() {
    check_shift(Shift::LogicalRight);
}

#[kani::proof]
#[kani::unwind(1)]
fn native_arithmetic_left_shift_matches_reference_semantics() {
    check_shift(Shift::ArithmeticLeft);
}

#[kani::proof]
#[kani::unwind(1)]
fn native_arithmetic_right_shift_matches_reference_semantics() {
    let (input, amount, position, expected) = check_shift(Shift::ArithmeticRight);
    let fill = input.signed
        && position < input.width as usize
        && amount.is_some_and(|amount| amount >= input.width as usize - position);
    kani::cover!(
        fill && expected == Logic4::Zero,
        "shift sign zero fill is reachable"
    );
    kani::cover!(
        fill && expected == Logic4::One,
        "shift sign one fill is reachable"
    );
    kani::cover!(
        fill && expected == Logic4::X,
        "shift sign X fill is reachable"
    );
    kani::cover!(
        fill && expected == Logic4::Z,
        "shift sign Z fill is reachable"
    );
}
