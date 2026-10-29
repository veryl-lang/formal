//! Public equality for all normalized four-state sources of widths 1..=64
//! and independent signs. Results of widths 1 then 8 share one cache.
//! Other comparison operations remain executable checks. Full u128 getters,
//! native storage, unsigned result metadata, constructors and source preservation
//! are checked. IEEE 1800 11.4.4/11.4.5/11.4.6 are modeled independently in comparison.
use veryl_analyzer::ir::Op;
use veryl_analyzer::value::{MaskCache, Value, ValueU64};
use veryl_formal_semantics::comparison::{Comparison, Operand, compared_logic};
use veryl_formal_semantics::logic4::Logic4;

// BigUint conversion/cloning/masks, heap allocation, and mask-cache access are
// forbidden in these small native proofs. Each replacement panics, proving it unreachable
// for every admitted input. Comparison, expansion, constructors, and getters run
// unchanged. Kani 0.68 otherwise expands unreachable BigUint paths before solving.
fn reject_comparison_biguint(_: u128) -> num_bigint::BigUint {
    panic!("public comparison semantics mismatch");
}
fn reject_comparison_cache(_: &mut MaskCache, _: usize) -> &num_bigint::BigUint {
    panic!("public comparison semantics mismatch");
}
fn reject_comparison_from_u64(_: u64) -> num_bigint::BigUint {
    panic!("public comparison semantics mismatch");
}
fn reject_comparison_clone(
    _: &veryl_analyzer::value::ValueBigUint,
) -> veryl_analyzer::value::ValueBigUint {
    panic!("public comparison semantics mismatch");
}
fn reject_comparison_mask(_: usize) -> num_bigint::BigUint {
    panic!("public comparison semantics mismatch");
}
unsafe fn reject_comparison_allocation(_: std::alloc::Layout) -> *mut u8 {
    panic!("public comparison semantics mismatch");
}

// These comparisons/conversions only belong to the forbidden BigUint arms.
fn reject_comparison_biguint_equal(_: &num_bigint::BigUint, _: &num_bigint::BigUint) -> bool {
    panic!("unreachable BigUint comparison");
}
kani::stub_set!(
    small_native_comparison,
    stub(
        veryl_analyzer::value::u128_to_biguint,
        reject_comparison_biguint
    ),
    stub(MaskCache::get, reject_comparison_cache),
    stub(
        <num_bigint::BigUint as std::convert::From<u64>>::from,
        reject_comparison_from_u64
    ),
    stub(
        <veryl_analyzer::value::ValueBigUint as std::clone::Clone>::clone,
        reject_comparison_clone
    ),
    stub(
        veryl_analyzer::value::ValueBigUint::gen_mask,
        reject_comparison_mask
    ),
    stub(num_bigint::BigUint::eq, reject_comparison_biguint_equal),
    stub(std::alloc::alloc, reject_comparison_allocation),
    stub(std::alloc::alloc_zeroed, reject_comparison_allocation),
);

fn assert_public_input(value: &Value, before: Operand) {
    assert!(
        matches!(value, Value::U64(_))
            && value.width() == before.width
            && value.signed() == before.signed
            && value.payload_u128() == before.planes.0
            && value.mask_xz_u128() == before.planes.1,
        "public comparison semantics mismatch"
    );
}

fn public_input() -> (ValueU64, Operand) {
    let native_width = u32::from(kani::any::<u8>() & 63) + 1;
    let mask = u64::MAX >> (64 - native_width);
    let before = Operand {
        planes: (
            u128::from(kani::any::<u64>() & mask),
            u128::from(kani::any::<u64>() & mask),
        ),
        width: native_width as usize,
        signed: kani::any(),
    };
    let width = before.width;
    let value = Value::from_u128(before.planes.0, before.planes.1, width, before.signed);
    assert_public_input(&value, before);
    let Value::U64(native) = value else {
        panic!("public comparison semantics mismatch");
    };
    (native, before)
}

fn assert_result(output: Value, width: usize, expected: Logic4) {
    let Value::U64(native) = output else {
        panic!("public comparison semantics mismatch");
    };
    let output = Value::U64(native);
    let planes = match expected {
        Logic4::Zero => (0, 0),
        Logic4::One => (1, 0),
        Logic4::X => (0, 1),
        Logic4::Z => (1, 1),
    };
    assert!(
        output.width() == width
            && !output.signed()
            && output.payload_u128() == planes.0
            && output.mask_xz_u128() == planes.1,
        "public comparison semantics mismatch"
    );
}

#[kani::proof]
#[kani::unwind(65)]
#[kani::use_stub_set(small_native_comparison)]
fn public_equality_native_pair_matches_reference_semantics() {
    let (a, ar) = public_input();
    let (b, br) = public_input();
    let a = Value::U64(a);
    let b = Value::U64(b);
    let signed = ar.signed && br.signed;
    let mut cache = MaskCache::default();
    let expected = compared_logic(ar, br, Comparison::Equal);
    let output = Op::Eq.eval_value_binary(&a, &b, 1, signed, &mut cache);
    assert_result(output, 1, expected);
    let output = Op::Eq.eval_value_binary(&a, &b, 8, signed, &mut cache);
    assert_result(output, 8, expected);
    assert_public_input(&a, ar);
    assert_public_input(&b, br);
    kani::cover!(ar.width == 1, "public comparison LHS width 1 is reachable");
    kani::cover!(ar.width == 2, "public comparison LHS width 2 is reachable");
    kani::cover!(ar.width == 3, "public comparison LHS width 3 is reachable");
    kani::cover!(br.width == 1, "public comparison RHS width 1 is reachable");
    kani::cover!(br.width == 2, "public comparison RHS width 2 is reachable");
    kani::cover!(br.width == 3, "public comparison RHS width 3 is reachable");
    kani::cover!(
        ar.width == 64,
        "public comparison LHS width 64 is reachable"
    );
    kani::cover!(
        br.width == 64,
        "public comparison RHS width 64 is reachable"
    );
    kani::cover!(
        ar.width > br.width,
        "public comparison wider LHS is reachable"
    );
    kani::cover!(
        ar.width < br.width,
        "public comparison wider RHS is reachable"
    );
    kani::cover!(
        ar.signed && br.signed,
        "public comparison signed mode is reachable"
    );
    kani::cover!(
        ar.signed != br.signed,
        "public comparison mixed signs are reachable"
    );
    kani::cover!(
        expected == Logic4::X,
        "public comparison X result is reachable"
    );
    kani::cover!(
        expected == Logic4::One,
        "public comparison true result is reachable"
    );
    kani::cover!(
        expected == Logic4::Zero,
        "public comparison false result is reachable"
    );
}
