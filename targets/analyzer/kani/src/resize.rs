//! Normalized native four-state values and target widths bounded by native storage.
//! Checks bit origins and metadata for truncation/extension, including no-op requests.
//! Public constructor/resize dispatch covers every source/request width in 1..=64.
//! Caller context resolution and arbitrary-width BigUint paths remain separate.

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

// BigUint conversion/cloning/masks, heap allocation, and mask-cache access are
// forbidden in these small native proofs. Each replacement panics, proving it unreachable
// for every admitted input. Truncation, expansion, constructors, and getters run
// unchanged. Kani 0.68 otherwise expands unreachable BigUint paths before solving.
fn reject_resize_biguint(_: u128) -> num_bigint::BigUint {
    panic!("public resize semantics mismatch");
}
fn reject_resize_cache(_: &mut veryl_analyzer::value::MaskCache, _: usize) -> &num_bigint::BigUint {
    panic!("public resize semantics mismatch");
}
fn reject_resize_from_u64(_: u64) -> num_bigint::BigUint {
    panic!("public resize semantics mismatch");
}
fn reject_resize_clone(
    _: &veryl_analyzer::value::ValueBigUint,
) -> veryl_analyzer::value::ValueBigUint {
    panic!("public resize semantics mismatch");
}
fn reject_resize_mask(_: usize) -> num_bigint::BigUint {
    panic!("public resize semantics mismatch");
}
unsafe fn reject_resize_allocation(_: std::alloc::Layout) -> *mut u8 {
    panic!("public resize semantics mismatch");
}

kani::stub_set!(
    small_native_resize,
    stub(
        veryl_analyzer::value::u128_to_biguint,
        reject_resize_biguint
    ),
    stub(veryl_analyzer::value::MaskCache::get, reject_resize_cache),
    stub(
        <num_bigint::BigUint as std::convert::From<u64>>::from,
        reject_resize_from_u64
    ),
    stub(
        <veryl_analyzer::value::ValueBigUint as std::clone::Clone>::clone,
        reject_resize_clone
    ),
    stub(
        veryl_analyzer::value::ValueBigUint::gen_mask,
        reject_resize_mask
    ),
    stub(std::alloc::alloc, reject_resize_allocation),
    stub(std::alloc::alloc_zeroed, reject_resize_allocation),
);

// Quantify the public constructor/clone/truncation/extension path at every native
// width. All four-state source planes are normalized; widths 1..=64, both source
// signs and extension flags, no-op requests and padding through bit 127 are included.
// The independent per-bit model follows IEEE 1800 11.6.1 and 11.8.2.
#[kani::proof]
#[kani::unwind(1)]
#[kani::use_stub_set(small_native_resize)]
fn public_resize_native_matches_reference_semantics() {
    let source = usize::from(kani::any::<u8>() & 63) + 1;
    let requested = usize::from(kani::any::<u8>() & 63) + 1;
    let mask = u64::MAX >> (64 - source);
    let payload = u128::from(kani::any::<u64>() & mask);
    let xz = u128::from(kani::any::<u64>() & mask);
    let signed: bool = kani::any();
    let use_sign: bool = kani::any();
    let position = usize::from(kani::any::<u8>() & 127);
    let input = Value::from_u128(payload, xz, source, signed);
    assert!(
        matches!(input, Value::U64(_))
            && input.width() == source
            && input.signed() == signed
            && input.payload_u128() == payload
            && input.mask_xz_u128() == xz,
        "public resize semantics mismatch"
    );
    let Value::U64(native) = input else {
        panic!("public resize semantics mismatch");
    };
    let input = Value::U64(native);
    let mut truncated = input.clone();
    truncated.trunc(requested);
    let Value::U64(native) = truncated else {
        panic!("public resize semantics mismatch");
    };
    let truncated = Value::U64(native);
    let tw = source.min(requested);
    let expected_t = truncated_logic_bit((payload, xz), tw, position);
    assert!(
        truncated.width() == tw
            && truncated.signed() == signed
            && read_logic(truncated.payload_u128(), truncated.mask_xz_u128(), position)
                == expected_t,
        "public resize semantics mismatch"
    );
    let expanded = input.expand(requested, use_sign).into_owned();
    let Value::U64(native) = expanded else {
        panic!("public resize semantics mismatch");
    };
    let expanded = Value::U64(native);
    let ew = source.max(requested);
    let expected_e = extended_logic_bit((payload, xz), source, ew, signed, use_sign, position);
    assert!(
        expanded.width() == ew
            && expanded.signed() == (signed && (requested <= source || use_sign))
            && read_logic(expanded.payload_u128(), expanded.mask_xz_u128(), position) == expected_e,
        "public resize semantics mismatch"
    );
    assert!(
        input.width() == source
            && input.signed() == signed
            && input.payload_u128() == payload
            && input.mask_xz_u128() == xz,
        "public resize semantics mismatch"
    );
    kani::cover!(
        source == 1,
        "public native resize source width 1 is reachable"
    );
    kani::cover!(
        source == 64,
        "public native resize source width 64 is reachable"
    );
    kani::cover!(
        requested == 1,
        "public native resize request 1 is reachable"
    );
    kani::cover!(
        requested == 64,
        "public native resize request 64 is reachable"
    );
    kani::cover!(
        requested < source,
        "public native resize smaller request is reachable"
    );
    kani::cover!(
        requested == source,
        "public native resize equal request is reachable"
    );
    kani::cover!(
        requested > source,
        "public native resize larger request is reachable"
    );
    kani::cover!(signed, "public native resize signed source is reachable");
    kani::cover!(!signed, "public native resize unsigned source is reachable");
    kani::cover!(
        position == 127,
        "public native resize u128 padding is reachable"
    );
    kani::cover!(
        expected_t == Logic4::X,
        "public native resize truncated X is reachable"
    );
    kani::cover!(
        expected_t == Logic4::Z,
        "public native resize truncated Z is reachable"
    );
    let added = position >= source && position < ew;
    kani::cover!(
        added && signed && !use_sign,
        "public native resize disables sign extension"
    );
    kani::cover!(
        added && !signed && use_sign,
        "public native resize zero-extends unsigned source"
    );
    kani::cover!(
        added && signed && use_sign && expected_e == Logic4::Zero,
        "public native resize extends zero"
    );
    kani::cover!(
        added && expected_e == Logic4::One,
        "public native resize extends one"
    );
    kani::cover!(
        added && expected_e == Logic4::X,
        "public native resize extends X"
    );
    kani::cover!(
        added && expected_e == Logic4::Z,
        "public native resize extends Z"
    );
}
