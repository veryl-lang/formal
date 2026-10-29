//! Public scalar-to-BigUint zero extension: source width 1, every four-state
//! value/sign, result width 65 or 128, and use_sign=false.
//! Execute clone/truncation first, then zero extension as in the direct
//! resize test. BigUint conversion, heap allocation and getters execute unchanged.
//! Panicking guards forbid only unreachable non-native source paths.
//! IEEE 1800 11.6.1/11.8.2 zero extension leaves normalized bit planes unchanged.
use veryl_analyzer::value::Value;

fn reject_native_source(_: u128) -> num_bigint::BigUint {
    panic!("public wide resize semantics mismatch");
}
// Outputs are moved from owned Cow values; cloning/converting a Veryl BigUint
// wrapper would require an inadmissible BigUint source. Numeric BigUint clones,
// allocation, masks and arithmetic remain real and reachable.
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
// BigUint allocation/conversion execute unchanged; sign filling remains a direct check.
fn public_wide_zero_extension<const TARGET: usize>() {
    let source = 1usize;
    let mask = u64::MAX >> (64 - source);
    let planes = (
        u128::from(kani::any::<u64>() & mask),
        u128::from(kani::any::<u64>() & mask),
    );
    let signed: bool = kani::any();
    let input = Value::from_u128(planes.0, planes.1, source, signed);
    assert_input(&input, source, planes, signed);
    let Value::U64(native) = input else {
        panic!("public wide resize semantics mismatch");
    };
    let input = Value::U64(native);
    let mut truncated = input.clone();
    truncated.trunc(TARGET);
    assert_input(&truncated, source, planes, signed);
    let output = input.expand(TARGET, false).into_owned();
    let Value::BigUint(big) = output else {
        panic!("public wide resize semantics mismatch");
    };
    assert!(
        big.payload.bits() <= TARGET as u64 && big.mask_xz.bits() <= TARGET as u64,
        "public wide resize semantics mismatch"
    );
    let output = Value::BigUint(big);
    assert!(
        output.width() == TARGET
            && !output.signed()
            && output.payload_u128() == planes.0
            && output.mask_xz_u128() == planes.1,
        "public wide resize semantics mismatch"
    );
    assert_input(&input, source, planes, signed);
    kani::cover!(
        planes.0 == 0 && planes.1 == 0,
        "public wide zero extension source zero is reachable"
    );
    kani::cover!(
        planes.0 == 1 && planes.1 == 0,
        "public wide zero extension source one is reachable"
    );
    kani::cover!(
        source == 1,
        "public wide zero extension source width 1 is reachable"
    );
    kani::cover!(
        signed,
        "public wide zero extension signed source is reachable"
    );
    kani::cover!(
        !signed,
        "public wide zero extension unsigned source is reachable"
    );
    kani::cover!(
        planes.1 & !planes.0 != 0,
        "public wide zero extension source X is reachable"
    );
    kani::cover!(
        planes.1 & planes.0 != 0,
        "public wide zero extension source Z is reachable"
    );
}
#[kani::proof]
#[kani::unwind(6)]
#[kani::stub(veryl_analyzer::value::u128_to_biguint, reject_native_source)]
#[kani::stub(<veryl_analyzer::value::ValueBigUint as std::clone::Clone>::clone, reject_source_clone)]
#[kani::stub(
    veryl_analyzer::value::ValueBigUint::to_value_u64,
    reject_source_to_native
)]
fn public_scalar_zero_extension_65_matches_reference_semantics() {
    public_wide_zero_extension::<65>();
}
#[kani::proof]
#[kani::unwind(6)]
#[kani::stub(veryl_analyzer::value::u128_to_biguint, reject_native_source)]
#[kani::stub(<veryl_analyzer::value::ValueBigUint as std::clone::Clone>::clone, reject_source_clone)]
#[kani::stub(
    veryl_analyzer::value::ValueBigUint::to_value_u64,
    reject_source_to_native
)]
fn public_scalar_zero_extension_128_matches_reference_semantics() {
    public_wide_zero_extension::<128>();
}
