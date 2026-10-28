//! Normalized four-state widths 1..=64 and arbitrary native endpoints, with result
//! width limited to 64. Checks missing-bit X, unsigned selection metadata, and guards.
//! Public dispatch covers every native source width and endpoint in this domain, including
//! construction, clone/input preservation, and every returned u128 bit.
//! Declared ranges, signed/X/Z selector decoding, and expression lowering are not proved.

use veryl_analyzer::value::ValueU64;
use veryl_formal_semantics::logic4::{Logic4, read_logic};
use veryl_formal_semantics::selection::selected_logic_bit;

#[kani::proof]
#[kani::unwind(1)]
fn native_selection_matches_reference_semantics() {
    let input = ValueU64 {
        payload: kani::any(),
        mask_xz: kani::any(),
        width: kani::any(),
        signed: kani::any(),
    };
    kani::assume(input.width >= 1 && input.width <= 64);
    let normalized_mask = u64::MAX >> (64 - input.width);
    kani::assume(input.payload & !normalized_mask == 0);
    kani::assume(input.mask_xz & !normalized_mask == 0);
    let beg: usize = kani::any();
    let end: usize = kani::any();
    kani::assume(beg < end || beg - end < 64);
    let position: usize = kani::any();
    kani::assume(position < 64);
    let actual = input.select(beg, end);
    let expected = selected_logic_bit(
        (input.payload as u128, input.mask_xz as u128),
        input.width as usize,
        beg,
        end,
        position,
    );
    assert!(
        read_logic(actual.payload as u128, actual.mask_xz as u128, position) == expected,
        "selection semantics mismatch"
    );
    let output_width = if beg < end { 0 } else { beg - end + 1 };
    assert!(
        actual.width as usize == output_width,
        "selection semantics mismatch"
    );
    assert!(!actual.signed, "selection semantics mismatch");
    kani::cover!(beg < end, "reversed selection is reachable");
    kani::cover!(beg == end, "single-bit selection is reachable");
    kani::cover!(
        input.width == 64 && beg == 63 && end == 0,
        "full-width selection is reachable"
    );
    kani::cover!(
        beg >= end && end >= input.width as usize && end < 64,
        "invalid selection below bit 64 is reachable"
    );
    kani::cover!(
        beg >= end && end >= 64,
        "invalid selection above bit 63 is reachable"
    );
    kani::cover!(
        beg >= input.width as usize && end < input.width as usize,
        "overlapping selection is reachable"
    );
    kani::cover!(
        beg == usize::MAX && end == usize::MAX,
        "maximum selection bounds are reachable"
    );
    kani::cover!(
        end < input.width as usize
            && end <= beg
            && beg < input.width as usize
            && expected == Logic4::X,
        "selected source X is reachable"
    );
    kani::cover!(expected == Logic4::Z, "selected source Z is reachable");
    kani::cover!(
        input.signed && beg >= end,
        "signed input selection is reachable"
    );
}

// BigUint conversion/cloning/masks, heap allocation, and mask-cache access are
// forbidden in these small native proofs. Each replacement panics, proving it unreachable
// for every admitted input. Selection, cloning, constructors, and getters run
// unchanged. Kani 0.68 otherwise expands unreachable BigUint paths before solving.
fn reject_selection_biguint(_: u128) -> num_bigint::BigUint {
    panic!("public selection semantics mismatch");
}
fn reject_selection_cache(
    _: &mut veryl_analyzer::value::MaskCache,
    _: usize,
) -> &num_bigint::BigUint {
    panic!("public selection semantics mismatch");
}
fn reject_selection_from_u64(_: u64) -> num_bigint::BigUint {
    panic!("public selection semantics mismatch");
}
fn reject_selection_clone(
    _: &veryl_analyzer::value::ValueBigUint,
) -> veryl_analyzer::value::ValueBigUint {
    panic!("public selection semantics mismatch");
}
fn reject_selection_mask(_: usize) -> num_bigint::BigUint {
    panic!("public selection semantics mismatch");
}
unsafe fn reject_selection_allocation(_: std::alloc::Layout) -> *mut u8 {
    panic!("public selection semantics mismatch");
}

kani::stub_set!(
    small_native_selection,
    stub(
        veryl_analyzer::value::u128_to_biguint,
        reject_selection_biguint
    ),
    stub(
        veryl_analyzer::value::MaskCache::get,
        reject_selection_cache
    ),
    stub(
        <num_bigint::BigUint as std::convert::From<u64>>::from,
        reject_selection_from_u64
    ),
    stub(
        <veryl_analyzer::value::ValueBigUint as std::clone::Clone>::clone,
        reject_selection_clone
    ),
    stub(
        veryl_analyzer::value::ValueBigUint::gen_mask,
        reject_selection_mask
    ),
    stub(std::alloc::alloc, reject_selection_allocation),
    stub(std::alloc::alloc_zeroed, reject_selection_allocation),
);

// All normalized four-state source widths 1..=64, either sign, and arbitrary
// usize endpoints whose result fits native storage. Reversed endpoints yield
// width zero; missing source bits are X (IEEE 1800 11.5.1). This includes distant
// and usize::MAX endpoints without assuming that selectors are in range.
#[kani::proof]
#[kani::unwind(1)]
#[kani::use_stub_set(small_native_selection)]
fn public_selection_native_matches_reference_semantics() {
    use veryl_analyzer::value::Value;
    let source = usize::from(kani::any::<u8>() & 63) + 1;
    let mask = u64::MAX >> (64 - source);
    let payload = u128::from(kani::any::<u64>() & mask);
    let xz = u128::from(kani::any::<u64>() & mask);
    let signed: bool = kani::any();
    let beg: usize = kani::any();
    let end: usize = kani::any();
    kani::assume(beg < end || beg - end < 64);
    let position = usize::from(kani::any::<u8>() & 127);
    let input = Value::from_u128(payload, xz, source, signed);
    assert!(
        matches!(input, Value::U64(_))
            && input.width() == source
            && input.signed() == signed
            && input.payload_u128() == payload
            && input.mask_xz_u128() == xz,
        "public selection semantics mismatch"
    );
    let Value::U64(native) = input else {
        panic!("public selection semantics mismatch");
    };
    let input = Value::U64(native);
    let original = input.clone();
    assert!(
        matches!(original, Value::U64(_))
            && original.width() == source
            && original.signed() == signed
            && original.payload_u128() == payload
            && original.mask_xz_u128() == xz,
        "public selection semantics mismatch"
    );
    let result = input.select(beg, end);
    let Value::U64(native) = result else {
        panic!("public selection semantics mismatch");
    };
    let result = Value::U64(native);
    let width = if beg < end { 0 } else { beg - end + 1 };
    let expected = selected_logic_bit((payload, xz), source, beg, end, position);
    assert!(
        result.width() == width
            && !result.signed()
            && read_logic(result.payload_u128(), result.mask_xz_u128(), position) == expected,
        "public selection semantics mismatch"
    );
    assert!(
        input.width() == source
            && input.signed() == signed
            && input.payload_u128() == payload
            && input.mask_xz_u128() == xz,
        "public selection semantics mismatch"
    );
    kani::cover!(
        source == 1,
        "public native selection source width 1 is reachable"
    );
    kani::cover!(
        source == 64,
        "public native selection source width 64 is reachable"
    );
    kani::cover!(
        beg < end,
        "public native selection reversed endpoints are reachable"
    );
    kani::cover!(
        beg == end,
        "public native selection single bit is reachable"
    );
    kani::cover!(
        beg == 63 && end == 0,
        "public native selection 64-bit result is reachable"
    );
    kani::cover!(
        beg == source - 1 && end == 0,
        "public native selection full source is reachable"
    );
    kani::cover!(
        beg >= end && end >= source,
        "public native selection missing source is reachable"
    );
    kani::cover!(
        beg >= source && end < source,
        "public native selection partial overlap is reachable"
    );
    kani::cover!(
        beg == usize::MAX && end == usize::MAX,
        "public native selection maximum single endpoint is reachable"
    );
    kani::cover!(
        beg == usize::MAX && end == usize::MAX - 3,
        "public native selection maximum range is reachable"
    );
    kani::cover!(
        end == usize::MAX && beg == 0,
        "public native selection maximum reversed endpoint is reachable"
    );
    kani::cover!(signed, "public native selection signed source is reachable");
    kani::cover!(
        !signed,
        "public native selection unsigned source is reachable"
    );
    kani::cover!(
        expected == Logic4::X && end < source && position < source - end,
        "public native selection source X is reachable"
    );
    kani::cover!(
        expected == Logic4::Z,
        "public native selection source Z is reachable"
    );
    kani::cover!(
        position == 127,
        "public native selection u128 padding is reachable"
    );
}
