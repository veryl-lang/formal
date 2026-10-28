//! Normalized native four-state values and target widths bounded by native storage.
//! Checks bit origins and metadata for truncation/extension, including no-op requests.
//! Public constructor/resize dispatch also covers source widths 1..=4 and requests
//! 1..=6. Caller context resolution and arbitrary-width BigUint paths remain separate.

use veryl_analyzer::value::{Value, ValueU64};
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

/// Public constructor and resize dispatch over the complete executable small domain:
/// source widths 1..=4, requests 1..=6, normalized four-state planes, both signs
/// and extension modes. Every stored bit, zero padding through bit 127, and
/// original-input metadata are checked.
fn public_resize_layout<const SOURCE: usize, const REQUESTED: usize>() {
    let source = SOURCE;
    let requested = REQUESTED;
    let mask = (1u128 << source) - 1;
    let payload = u128::from(kani::any::<u8>()) & mask;
    let xz = u128::from(kani::any::<u8>()) & mask;
    let signed: bool = kani::any();
    let use_sign: bool = kani::any();
    let position = usize::from(kani::any::<u8>() & 127);
    let input = Value::from_u128(payload, xz, source, signed);
    let mut truncated = input.clone();
    truncated.trunc(requested);
    let tw = source.min(requested);
    let expected = truncated_logic_bit((payload, xz), tw, position);
    assert!(
        matches!(truncated, Value::U64(_))
            && truncated.width() == tw
            && truncated.signed() == signed
            && read_logic(truncated.payload_u128(), truncated.mask_xz_u128(), position) == expected,
        "public resize semantics mismatch"
    );
    let expanded = input.expand(requested, use_sign);
    let ew = source.max(requested);
    let expected = extended_logic_bit((payload, xz), source, ew, signed, use_sign, position);
    assert!(
        matches!(expanded.as_ref(), Value::U64(_))
            && expanded.width() == ew
            && expanded.signed() == (signed && (requested <= source || use_sign))
            && read_logic(expanded.payload_u128(), expanded.mask_xz_u128(), position) == expected,
        "public resize semantics mismatch"
    );
    assert!(
        matches!(input, Value::U64(_))
            && input.payload_u128() == payload
            && input.mask_xz_u128() == xz
            && input.width() == source
            && input.signed() == signed,
        "public resize semantics mismatch"
    );
}

macro_rules! public_resize_proof {
    ($name:ident, $source:literal) => {
        #[kani::proof]
        #[kani::unwind(1)]
        fn $name() {
            let requested = usize::from(kani::any::<u8>() % 6) + 1;
            match requested {
                1 => public_resize_layout::<$source, 1>(),
                2 => public_resize_layout::<$source, 2>(),
                3 => public_resize_layout::<$source, 3>(),
                4 => public_resize_layout::<$source, 4>(),
                5 => public_resize_layout::<$source, 5>(),
                6 => public_resize_layout::<$source, 6>(),
                _ => unreachable!(),
            }
            kani::cover!(requested == 1, "public resize request 1 is reachable");
            kani::cover!(requested == 2, "public resize request 2 is reachable");
            kani::cover!(requested == 3, "public resize request 3 is reachable");
            kani::cover!(requested == 4, "public resize request 4 is reachable");
            kani::cover!(requested == 5, "public resize request 5 is reachable");
            kani::cover!(requested == 6, "public resize request 6 is reachable");
        }
    };
}

public_resize_proof!(public_resize_one_matches_reference_semantics, 1);
public_resize_proof!(public_resize_two_matches_reference_semantics, 2);
public_resize_proof!(public_resize_three_matches_reference_semantics, 3);
public_resize_proof!(public_resize_four_matches_reference_semantics, 4);
