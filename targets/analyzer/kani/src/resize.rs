//! Normalized native four-state values and target widths bounded by native storage.
//! Checks bit origins and metadata for truncation/extension, including no-op requests.
//! Caller choice of extension mode and arbitrary-width BigUint paths are outside scope.

use veryl_analyzer::value::ValueU64;
use veryl_formal_semantics::logic4::{Logic4, read_logic};
use veryl_formal_semantics::resize::{extended_logic_bit, truncated_logic_bit};

fn normalized_value() -> ValueU64 {
    let value = ValueU64 {
        payload: kani::any(),
        mask_xz: kani::any(),
        width: u32::from(kani::any::<u8>() & 63) + 1,
        signed: kani::any(),
    };
    kani::assume(value.width >= 1 && value.width <= 64);
    let mask = u64::MAX >> (64 - value.width);
    kani::assume(value.payload & !mask == 0);
    kani::assume(value.mask_xz & !mask == 0);
    value
}

#[kani::proof]
#[kani::unwind(1)]
fn native_truncation_matches_reference_semantics() {
    let before = normalized_value();
    let width: usize = kani::any();
    kani::assume(width >= 1 && width <= before.width as usize);
    let position: usize = kani::any();
    kani::assume(position < 64);
    let mut after = before.clone();
    after.trunc(width);
    let expected = truncated_logic_bit(
        (before.payload as u128, before.mask_xz as u128),
        width,
        position,
    );
    assert!(
        read_logic(after.payload as u128, after.mask_xz as u128, position) == expected,
        "truncation semantics mismatch"
    );
    assert_eq!(after.width as usize, width);
    assert_eq!(after.signed, before.signed);
    kani::cover!(before.width == 64, "truncation from width 64 is reachable");
    kani::cover!(width == 1, "truncation to one bit is reachable");
    kani::cover!(
        width < before.width as usize,
        "strict truncation is reachable"
    );
    kani::cover!(
        width == before.width as usize,
        "unchanged truncation is reachable"
    );
    kani::cover!(expected == Logic4::X, "truncated X is reachable");
    kani::cover!(expected == Logic4::Z, "truncated Z is reachable");
}

#[kani::proof]
#[kani::unwind(1)]
fn native_extension_matches_reference_semantics() {
    let before = normalized_value();
    let width = usize::from(kani::any::<u8>() & 63) + 1;
    let use_sign: bool = kani::any();
    let position: usize = kani::any();
    kani::assume(position < 64);
    let after = before.expand(width, use_sign);
    let output_width = width.max(before.width as usize);
    let expected = extended_logic_bit(
        (before.payload as u128, before.mask_xz as u128),
        before.width as usize,
        output_width,
        before.signed,
        use_sign,
        position,
    );
    assert!(
        read_logic(after.payload as u128, after.mask_xz as u128, position) == expected,
        "extension semantics mismatch"
    );
    assert_eq!(after.width as usize, output_width);
    let expected_signed = before.signed && (width <= before.width as usize || use_sign);
    assert!(
        after.signed == expected_signed,
        "extension semantics mismatch"
    );
    let added_bit = position >= before.width as usize && position < output_width;
    kani::cover!(
        width == 64 && before.width < 64,
        "extension to width 64 is reachable"
    );
    kani::cover!(
        width == before.width as usize,
        "equal-width extension is reachable"
    );
    kani::cover!(
        width < before.width as usize,
        "smaller extension request is reachable"
    );
    kani::cover!(
        added_bit && !before.signed && use_sign,
        "unsigned zero extension is reachable"
    );
    kani::cover!(
        added_bit && before.signed && !use_sign,
        "disabled sign extension is reachable"
    );
    kani::cover!(
        added_bit && before.signed && use_sign && expected == Logic4::Zero,
        "extended sign zero is reachable"
    );
    kani::cover!(
        added_bit && expected == Logic4::One,
        "extended sign one is reachable"
    );
    kani::cover!(
        added_bit && expected == Logic4::X,
        "extended sign X is reachable"
    );
    kani::cover!(
        added_bit && expected == Logic4::Z,
        "extended sign Z is reachable"
    );
}
