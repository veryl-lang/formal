//! Public equality for all normalized four-state sources of widths 1..=64
//! and independent signs. Results of widths 1 then 8 share one cache.
//! Inequality continues that prefix at all native widths. Wildcard equality and
//! inequality continue it for source widths 1..=3. Signed and unsigned expression
//! contexts partition every source sign pair; widths and state domains are unchanged.
//! Other comparisons stay direct.
//! Full u128 getters,
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
    public_input_at_width::<false>(u32::from(kani::any::<u8>() & 63) + 1)
}

fn public_input_at_width<const SMALL: bool>(native_width: u32) -> (ValueU64, Operand) {
    public_input_with_sign::<SMALL>(native_width, kani::any())
}

fn public_input_with_sign<const SMALL: bool>(
    native_width: u32,
    signed: bool,
) -> (ValueU64, Operand) {
    let mask = u64::MAX >> (64 - native_width);
    let before = Operand {
        planes: (
            u128::from(
                if SMALL {
                    u64::from(kani::any::<u8>())
                } else {
                    kani::any::<u64>()
                } & mask,
            ),
            u128::from(
                if SMALL {
                    u64::from(kani::any::<u8>())
                } else {
                    kani::any::<u64>()
                } & mask,
            ),
        ),
        width: native_width as usize,
        signed,
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

fn native_result(output: Value) -> ValueU64 {
    let Value::U64(native) = output else {
        panic!("public comparison semantics mismatch");
    };
    native
}

// Observe Ne, wildcard equality or wildcard inequality only after every earlier
// pair, producing widths 1 then 8 in direct-test order on one cache.
fn public_comparison_trace<const MODE: u8>(
    a: ValueU64,
    ar: Operand,
    b: ValueU64,
    br: Operand,
) -> (Operand, Operand) {
    let a = Value::U64(a);
    let b = Value::U64(b);
    let signed = ar.signed && br.signed;
    let mut cache = MaskCache::default();
    let _ = native_result(Op::Eq.eval_value_binary(&a, &b, 1, signed, &mut cache));
    let _ = native_result(Op::Eq.eval_value_binary(&a, &b, 8, signed, &mut cache));
    if MODE > 1 {
        let _ = native_result(Op::Ne.eval_value_binary(&a, &b, 1, signed, &mut cache));
        let _ = native_result(Op::Ne.eval_value_binary(&a, &b, 8, signed, &mut cache));
    }
    if MODE > 2 {
        let _ = native_result(Op::EqWildcard.eval_value_binary(&a, &b, 1, signed, &mut cache));
        let _ = native_result(Op::EqWildcard.eval_value_binary(&a, &b, 8, signed, &mut cache));
    }
    let kind = match MODE {
        1 => Comparison::NotEqual,
        2 => Comparison::WildcardEqual,
        3 => Comparison::WildcardNotEqual,
        _ => unreachable!(),
    };
    let expected = compared_logic(ar, br, kind);
    let output = match MODE {
        1 => Op::Ne.eval_value_binary(&a, &b, 1, signed, &mut cache),
        2 => Op::EqWildcard.eval_value_binary(&a, &b, 1, signed, &mut cache),
        3 => Op::NeWildcard.eval_value_binary(&a, &b, 1, signed, &mut cache),
        _ => unreachable!(),
    };
    assert_result(output, 1, expected);
    let output = match MODE {
        1 => Op::Ne.eval_value_binary(&a, &b, 8, signed, &mut cache),
        2 => Op::EqWildcard.eval_value_binary(&a, &b, 8, signed, &mut cache),
        3 => Op::NeWildcard.eval_value_binary(&a, &b, 8, signed, &mut cache),
        _ => unreachable!(),
    };
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
        ar.width > br.width,
        "public comparison wider LHS is reachable"
    );
    kani::cover!(
        ar.width < br.width,
        "public comparison wider RHS is reachable"
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
    (ar, br)
}

#[kani::proof]
#[kani::unwind(4)]
#[kani::use_stub_set(small_native_comparison)]
fn public_small_wildcard_equality_continuation_matches_reference() {
    let (a, ar) = public_input_at_width::<true>(u32::from(kani::any::<u8>() & 3).min(2) + 1);
    let (b, br) = public_input_at_width::<true>(u32::from(kani::any::<u8>() & 3).min(2) + 1);
    let (ar, br) = public_comparison_trace::<2>(a, ar, b, br);
    cover_dynamic_signs(ar, br);
    cover_wildcard_inputs(ar, br);
}

fn cover_wildcard_inputs(ar: Operand, br: Operand) {
    kani::cover!(
        ar.width == 1 && br.width == 1 && ar.planes == (1, 0) && br.planes == (0, 1),
        "public comparison RHS wildcard is reachable"
    );
    kani::cover!(
        ar.width == 1 && br.width == 1 && ar.planes == (0, 1) && br.planes == (0, 0),
        "public comparison LHS X is significant"
    );
}

fn cover_dynamic_signs(ar: Operand, br: Operand) {
    let signed = ar.signed && br.signed;
    kani::cover!(
        ar.signed && br.signed,
        "public comparison signed mode is reachable"
    );
    kani::cover!(
        ar.signed != br.signed,
        "public comparison mixed signs are reachable"
    );
    kani::cover!(
        ar.width == 1 && br.width == 2 && signed && ar.planes == (1, 0) && br.planes == (3, 0),
        "public comparison sign extension is reachable"
    );
}

// The signed partition is exactly (true, true); the unsigned partition constructs
// (false, false), (false, true), and (true, false), with no input assumptions.
fn public_comparison_partition<const MODE: u8, const SIGNED: bool>() -> (Operand, Operand) {
    let flags = (kani::any::<u8>() & 3).min(2);
    let left_signed = SIGNED || flags == 2;
    let right_signed = SIGNED || flags == 1;
    let (a, ar) = if MODE == 1 {
        public_input_with_sign::<false>(u32::from(kani::any::<u8>() & 63) + 1, left_signed)
    } else {
        public_input_with_sign::<true>(u32::from(kani::any::<u8>() & 3).min(2) + 1, left_signed)
    };
    let (b, br) = if MODE == 1 {
        public_input_with_sign::<false>(u32::from(kani::any::<u8>() & 63) + 1, right_signed)
    } else {
        public_input_with_sign::<true>(u32::from(kani::any::<u8>() & 3).min(2) + 1, right_signed)
    };
    public_comparison_trace::<MODE>(a, ar, b, br)
}

fn cover_native_endpoints(ar: Operand, br: Operand) {
    kani::cover!(
        ar.width == 64,
        "public comparison LHS width 64 is reachable"
    );
    kani::cover!(
        br.width == 64,
        "public comparison RHS width 64 is reachable"
    );
}
fn cover_unsigned_signs(ar: Operand, br: Operand) {
    kani::cover!(
        !ar.signed && !br.signed,
        "public comparison unsigned sources are reachable"
    );
    kani::cover!(
        !ar.signed && br.signed,
        "public comparison signed RHS only is reachable"
    );
    kani::cover!(
        ar.signed && !br.signed,
        "public comparison signed LHS only is reachable"
    );
}
fn cover_signed_signs(ar: Operand, br: Operand) {
    kani::cover!(
        ar.signed && br.signed,
        "public comparison signed sources are reachable"
    );
}
fn cover_partition_sizing(ar: Operand, br: Operand) {
    kani::cover!(
        ar.width == 1 && br.width == 2 && ar.planes == (1, 0) && br.planes == (3, 0),
        "public comparison partition sizing is reachable"
    );
}

#[kani::proof]
#[kani::unwind(65)]
#[kani::use_stub_set(small_native_comparison)]
fn public_inequality_native_unsigned_continuation_matches_reference() {
    let (ar, br) = public_comparison_partition::<1, false>();
    cover_unsigned_signs(ar, br);
    cover_partition_sizing(ar, br);
    cover_native_endpoints(ar, br);
}

#[kani::proof]
#[kani::unwind(65)]
#[kani::use_stub_set(small_native_comparison)]
fn public_inequality_native_signed_continuation_matches_reference() {
    let (ar, br) = public_comparison_partition::<1, true>();
    cover_signed_signs(ar, br);
    cover_partition_sizing(ar, br);
    cover_native_endpoints(ar, br);
}

#[kani::proof]
#[kani::unwind(4)]
#[kani::use_stub_set(small_native_comparison)]
fn public_small_wildcard_inequality_unsigned_continuation_matches_reference() {
    let (ar, br) = public_comparison_partition::<3, false>();
    cover_unsigned_signs(ar, br);
    cover_partition_sizing(ar, br);
    cover_wildcard_inputs(ar, br);
}

#[kani::proof]
#[kani::unwind(4)]
#[kani::use_stub_set(small_native_comparison)]
fn public_small_wildcard_inequality_signed_continuation_matches_reference() {
    let (ar, br) = public_comparison_partition::<3, true>();
    cover_signed_signs(ar, br);
    cover_partition_sizing(ar, br);
    cover_wildcard_inputs(ar, br);
}
