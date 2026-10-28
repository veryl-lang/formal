//! Normalized destination widths 1..=64, arbitrary prepared RHS planes and metadata,
//! and arbitrary native endpoints. Checks every stored bit and preserved metadata.
//! Public constructor/assignment/getter dispatch also covers every pairing of
//! widths 1..=64, with normalized four-state planes, independent source
//! and destination signs, arbitrary usize endpoints, and all 128 getter bits.
//! Does not prove source sizing, selector decoding, or legal part-select syntax.
//! Native/BigUint conversions remain executable checks.

use veryl_analyzer::value::{Value, ValueU64};
use veryl_formal_semantics::logic4::{Logic4, read_logic};
use veryl_formal_semantics::partial_assignment::assigned_logic_bit;

#[kani::proof]
#[kani::unwind(1)]
fn native_assignment_matches_reference_semantics() {
    let mut dst = ValueU64 {
        payload: kani::any(),
        mask_xz: kani::any(),
        width: kani::any(),
        signed: kani::any(),
    };
    kani::assume(dst.width >= 1 && dst.width <= 64);
    let normalized_mask = u64::MAX >> (64 - dst.width);
    kani::assume(dst.payload & !normalized_mask == 0);
    kani::assume(dst.mask_xz & !normalized_mask == 0);
    let rhs = ValueU64 {
        payload: kani::any(),
        mask_xz: kani::any(),
        width: kani::any(),
        signed: kani::any(),
    };
    let beg: usize = kani::any();
    let end: usize = kani::any();
    let position: usize = kani::any();
    kani::assume(position < 64);
    let before = dst.clone();

    dst.assign(rhs.clone(), beg, end);

    let expected = if position < before.width as usize {
        assigned_logic_bit(
            (before.payload as u128, before.mask_xz as u128),
            (rhs.payload as u128, rhs.mask_xz as u128),
            position,
            beg,
            end,
        )
    } else {
        Logic4::Zero
    };
    // Arbitrary position proves equality of every stored bit, without a
    // loop over all positions or a separate mask-based specification.
    assert!(
        read_logic(dst.payload as u128, dst.mask_xz as u128, position) == expected,
        "partial-assignment semantics mismatch"
    );
    assert_eq!(dst.width, before.width);
    assert_eq!(dst.signed, before.signed);

    kani::cover!(before.width == 64, "full native width is reachable");
    kani::cover!(end >= before.width as usize, "ignored writes are reachable");
    kani::cover!(
        end < before.width as usize && beg >= before.width as usize,
        "clipped writes are reachable"
    );
    kani::cover!(
        beg == usize::MAX && end == 0,
        "maximum high bound is reachable"
    );
    kani::cover!(expected == Logic4::X, "X result is reachable");
    kani::cover!(expected == Logic4::Z, "Z result is reachable");
}

// Kani 0.68 expands the constructor's BigUint allocation before eliminating
// paths excluded by symbolic native widths. This panic asserts that the
// conversion is unreachable for every admitted input. Reaching it fails the
// proof, so the original conversion's return value is never abstracted.
// Constructor, assignment, getters, and reference operations remain unmodified.
fn reject_native_biguint(_: u128) -> num_bigint::BigUint {
    panic!("public assignment semantics mismatch");
}

fn public_native_input() -> (Value, ValueU64) {
    let width = u32::from(kani::any::<u8>() & 63) + 1;
    let mask = u64::MAX >> (64 - width);
    let before = ValueU64 {
        payload: kani::any::<u64>() & mask,
        mask_xz: kani::any::<u64>() & mask,
        width,
        signed: kani::any(),
    };
    let value = Value::from_u128(
        before.payload as u128,
        before.mask_xz as u128,
        width as usize,
        before.signed,
    );
    assert!(
        matches!(value, Value::U64(_))
            && value.width() == width as usize
            && value.signed() == before.signed
            && value.payload_u128() == before.payload as u128
            && value.mask_xz_u128() == before.mask_xz as u128,
        "public assignment semantics mismatch"
    );
    (value, before)
}

#[kani::proof]
#[kani::unwind(1)]
#[kani::stub(veryl_analyzer::value::u128_to_biguint, reject_native_biguint)]
fn public_assignment_native_matches_reference_semantics() {
    let (mut dst, before) = public_native_input();
    let (rhs, source) = public_native_input();
    let beg: usize = kani::any();
    let end: usize = kani::any();
    let position = usize::from(kani::any::<u8>() & 127);
    dst.assign(rhs, beg, end);
    let expected = if position < before.width as usize {
        assigned_logic_bit(
            (before.payload as u128, before.mask_xz as u128),
            (source.payload as u128, source.mask_xz as u128),
            position,
            beg,
            end,
        )
    } else {
        Logic4::Zero
    };
    assert!(
        matches!(dst, Value::U64(_))
            && dst.width() == before.width as usize
            && dst.signed() == before.signed
            && read_logic(dst.payload_u128(), dst.mask_xz_u128(), position) == expected,
        "public assignment semantics mismatch"
    );
    kani::cover!(
        source.width == 1,
        "public assignment source width 1 is reachable"
    );
    kani::cover!(
        source.width == 4,
        "public assignment source width 4 is reachable"
    );
    kani::cover!(
        source.width == 63,
        "public assignment source width 63 is reachable"
    );
    kani::cover!(
        source.width == 64,
        "public assignment source width 64 is reachable"
    );
    kani::cover!(
        end >= before.width as usize,
        "public assignment ignored write is reachable"
    );
    kani::cover!(
        end < before.width as usize && beg >= before.width as usize,
        "public assignment clipped write is reachable"
    );
    kani::cover!(
        beg < end,
        "public assignment reversed endpoints are reachable"
    );
    kani::cover!(
        beg == usize::MAX && end == 0,
        "public assignment maximum high bound is reachable"
    );
    kani::cover!(
        end == usize::MAX,
        "public assignment maximum low bound is reachable"
    );
    kani::cover!(
        beg == before.width as usize - 1 && end == 0,
        "public assignment full destination is reachable"
    );
    kani::cover!(
        expected == Logic4::X,
        "public assignment X result is reachable"
    );
    kani::cover!(
        expected == Logic4::Z,
        "public assignment Z result is reachable"
    );
    kani::cover!(
        before.signed != source.signed,
        "public assignment different signs are reachable"
    );
    kani::cover!(
        position == 127,
        "public assignment u128 padding is reachable"
    );
    kani::cover!(
        before.width == 1,
        "public assignment destination width 1 is reachable"
    );
    kani::cover!(
        before.width == 4,
        "public assignment destination width 4 is reachable"
    );
    kani::cover!(
        before.width == 14,
        "public assignment destination width 14 is reachable"
    );
    kani::cover!(
        before.width == 32,
        "public assignment destination width 32 is reachable"
    );
    kani::cover!(
        before.width == 63,
        "public assignment destination width 63 is reachable"
    );
    kani::cover!(
        before.width == 64,
        "public assignment destination width 64 is reachable"
    );
    kani::cover!(
        source.width == 14,
        "public assignment source width 14 is reachable"
    );
    kani::cover!(
        source.width == 32,
        "public assignment source width 32 is reachable"
    );
}
