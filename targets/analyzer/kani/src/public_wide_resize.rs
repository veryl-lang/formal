//! Public native-to-BigUint zero extension: normalized four-state sources of
//! every width 1..=64, independent target widths 65..=128, either source sign,
//! and use_sign=false. The only assumption normalizes both input bit planes.
//! Execute clone/truncation first, then extension; numeric BigUint conversion,
//! heap allocation, bit lengths and getters execute unchanged.
//! IEEE 1800 11.6.1/11.8.2 zero extension is identity on normalized bit planes.
//! Unwind 2 suffices for reachable conversion/getter loops; unwinding assertions
//! remain enabled. Mandatory cover cubes span endpoint widths, all four states,
//! both signs/target endpoints, and an interior source/target pair.
use veryl_analyzer::value::Value;

// With nonzero native sources and use_sign=false, sentinel/sign-fill masks
// cannot execute. A call fails the proof rather than supplying a mask oracle.
fn reject_sign_mask(_: usize) -> num_bigint::BigUint {
    panic!("public wide resize semantics mismatch");
}
fn reject_native_source(_: u128) -> num_bigint::BigUint {
    panic!("public wide resize semantics mismatch");
}
// Moving the owned result needs no Veryl BigUint wrapper clone/conversion.
// These guards reject non-native source paths; numeric BigUint operations stay real.
fn reject_source_clone(
    _: &veryl_analyzer::value::ValueBigUint,
) -> veryl_analyzer::value::ValueBigUint {
    panic!("public wide resize semantics mismatch");
}
fn reject_source_to_native(
    _: &veryl_analyzer::value::ValueBigUint,
) -> Option<veryl_analyzer::value::ValueU64> {
    panic!("public wide resize semantics mismatch");
}
fn assert_input(input: &Value, source: usize, planes: (u128, u128), signed: bool) {
    assert!(
        matches!(input, Value::U64(_))
            && input.width() == source
            && input.signed() == signed
            && input.payload_u128() == planes.0
            && input.mask_xz_u128() == planes.1,
        "public wide resize semantics mismatch"
    );
}
// Zero extension is numeric identity for the two normalized bit planes.
// Sign filling and BigUint sources remain direct comparisons.
fn public_wide_zero_extension(target: usize) {
    let source = usize::from(kani::any::<u8>() & 63) + 1;
    let mask = u64::MAX >> (64 - source);
    let planes = (
        u128::from(kani::any::<u64>()),
        u128::from(kani::any::<u64>()),
    );
    kani::assume((planes.0 | planes.1) & !u128::from(mask) == 0);
    let signed: bool = kani::any();
    let input = Value::from_u128(planes.0, planes.1, source, signed);
    assert_input(&input, source, planes, signed);
    let Value::U64(native) = input else {
        panic!("public wide resize semantics mismatch");
    };
    let input = Value::U64(native);
    let mut truncated = input.clone();
    truncated.trunc(target);
    let Value::U64(truncated_native) = truncated else {
        panic!("public wide resize semantics mismatch");
    };
    let truncated = Value::U64(truncated_native);
    assert_input(&truncated, source, planes, signed);
    let output = input.expand(target, false).into_owned();
    let Value::BigUint(big) = output else {
        panic!("public wide resize semantics mismatch");
    };
    assert!(
        big.payload.bits() <= target as u64 && big.mask_xz.bits() <= target as u64,
        "public wide resize semantics mismatch"
    );
    let output = Value::BigUint(big);
    assert!(
        output.width() == target
            && !output.signed()
            && output.payload_u128() == planes.0
            && output.mask_xz_u128() == planes.1,
        "public wide resize semantics mismatch"
    );
    assert_input(&input, source, planes, signed);
    kani::cover!(
        source == 1 && planes == (0, 0) && signed == false && target == 65,
        "public native wide zero extension width 1 zero is reachable"
    );
    kani::cover!(
        source == 1 && planes == (1u128 << 0, 0) && signed == true && target == 128,
        "public native wide zero extension width 1 one is reachable"
    );
    kani::cover!(
        source == 1 && planes == (0, 1u128 << 0) && signed == true && target == 65,
        "public native wide zero extension width 1 X is reachable"
    );
    kani::cover!(
        source == 1 && planes == (1u128 << 0, 1u128 << 0) && signed == false && target == 128,
        "public native wide zero extension width 1 Z is reachable"
    );
    kani::cover!(
        source == 64 && planes == (0, 0) && signed == true && target == 128,
        "public native wide zero extension width 64 zero is reachable"
    );
    kani::cover!(
        source == 64 && planes == (1u128 << 63, 0) && signed == false && target == 65,
        "public native wide zero extension width 64 one is reachable"
    );
    kani::cover!(
        source == 64 && planes == (0, 1u128 << 63) && signed == false && target == 128,
        "public native wide zero extension width 64 X is reachable"
    );
    kani::cover!(
        source == 64 && planes == (1u128 << 63, 1u128 << 63) && signed == true && target == 65,
        "public native wide zero extension width 64 Z is reachable"
    );

    kani::cover!(
        source == 32 && target == 127 && planes == (1u128 << 31, 1u128 << 31) && !signed,
        "public native wide zero extension interior widths are reachable"
    );
}
#[kani::proof]
#[kani::stub(veryl_analyzer::value::ValueBigUint::gen_mask, reject_sign_mask)]
#[kani::unwind(2)]
#[kani::stub(veryl_analyzer::value::u128_to_biguint, reject_native_source)]
#[kani::stub(<veryl_analyzer::value::ValueBigUint as std::clone::Clone>::clone, reject_source_clone)]
#[kani::stub(
    veryl_analyzer::value::ValueBigUint::to_value_u64,
    reject_source_to_native
)]
fn public_native_zero_extension_wide_matches_reference_semantics() {
    public_wide_zero_extension(usize::from(kani::any::<u8>() & 63) + 65);
}
