//! Public shift trace at all normalized four-state native source widths 1..=64.
//! Sources and amounts are unsigned; amounts span arbitrary four-state 64-bit
//! planes with unsigned magnitude decoding and saturation. The first four shifts
//! execute on one cache in test order. Later sign groups remain direct checks.
//! Result width stays native
//! source width. Context widening and BigUint amount/source paths remain separate.
//! IEEE 1800 11.4.10 is modeled independently by shifted_logic_bit.
use veryl_analyzer::ir::Op;
use veryl_analyzer::value::{MaskCache, Value};
use veryl_formal_semantics::logic4::{Logic4, read_logic};
use veryl_formal_semantics::shift::{Shift, shifted_logic_bit};

// BigUint conversion/cloning/masks, heap allocation, and mask-cache access are
// forbidden in these small native proofs. Each replacement panics, proving it unreachable
// for every admitted input. Shifts, decoding, constructors, and getters run
// unchanged. Kani 0.68 otherwise expands unreachable BigUint paths before solving.
fn reject_shift_biguint(_: u128) -> num_bigint::BigUint {
    panic!("public shift semantics mismatch");
}
fn reject_shift_cache(_: &mut veryl_analyzer::value::MaskCache, _: usize) -> &num_bigint::BigUint {
    panic!("public shift semantics mismatch");
}
fn reject_shift_from_u64(_: u64) -> num_bigint::BigUint {
    panic!("public shift semantics mismatch");
}
fn reject_shift_clone(
    _: &veryl_analyzer::value::ValueBigUint,
) -> veryl_analyzer::value::ValueBigUint {
    panic!("public shift semantics mismatch");
}
fn reject_shift_mask(_: usize) -> num_bigint::BigUint {
    panic!("public shift semantics mismatch");
}
unsafe fn reject_shift_allocation(_: std::alloc::Layout) -> *mut u8 {
    panic!("public shift semantics mismatch");
}

fn reject_shift_plane(_: &veryl_analyzer::value::ValueBigUint) -> &num_bigint::BigUint {
    panic!("public shift semantics mismatch");
}
fn reject_shift_to_native(
    _: &veryl_analyzer::value::ValueBigUint,
) -> Option<veryl_analyzer::value::ValueU64> {
    panic!("public shift semantics mismatch");
}
fn reject_shift_unknown(_: &veryl_analyzer::value::ValueBigUint) -> bool {
    panic!("public shift semantics mismatch");
}
kani::stub_set!(
    small_native_shift,
    stub(veryl_analyzer::value::u128_to_biguint, reject_shift_biguint),
    stub(veryl_analyzer::value::MaskCache::get, reject_shift_cache),
    stub(
        <num_bigint::BigUint as std::convert::From<u64>>::from,
        reject_shift_from_u64
    ),
    stub(
        <veryl_analyzer::value::ValueBigUint as std::clone::Clone>::clone,
        reject_shift_clone
    ),
    stub(
        veryl_analyzer::value::ValueBigUint::gen_mask,
        reject_shift_mask
    ),
    stub(
        veryl_analyzer::value::ValueBigUint::payload,
        reject_shift_plane
    ),
    stub(
        veryl_analyzer::value::ValueBigUint::mask_xz,
        reject_shift_plane
    ),
    stub(
        veryl_analyzer::value::ValueBigUint::to_value_u64,
        reject_shift_to_native
    ),
    stub(
        veryl_analyzer::value::ValueBigUint::is_xz,
        reject_shift_unknown
    ),
    stub(std::alloc::alloc, reject_shift_allocation),
    stub(std::alloc::alloc_zeroed, reject_shift_allocation),
);

fn input(width: usize, planes: (u128, u128), signed: bool) -> Value {
    let input = Value::from_u128(planes.0, planes.1, width, signed);
    assert_input(&input, width, planes, signed);
    let Value::U64(native) = input else {
        panic!("public shift semantics mismatch");
    };
    Value::U64(native)
}
fn assert_input(input: &Value, width: usize, planes: (u128, u128), signed: bool) {
    assert!(
        matches!(input, Value::U64(_))
            && input.width() == width
            && input.signed() == signed
            && input.payload_u128() == planes.0
            && input.mask_xz_u128() == planes.1,
        "public shift semantics mismatch"
    );
}
fn assert_result(
    output: Value,
    source: (u128, u128),
    width: usize,
    signed: bool,
    amount: Option<usize>,
    kind: Shift,
    position: usize,
) {
    let Value::U64(native) = output else {
        panic!("public shift semantics mismatch");
    };
    let output = Value::U64(native);
    let payload = output.payload_u128();
    let xz = output.mask_xz_u128();
    // Positions 0..=63 cover every active native bit. Check all higher getter
    // bits directly, keeping the complete 128-bit result guarantee.
    assert!(
        payload >> 64 == 0 && xz >> 64 == 0,
        "public shift semantics mismatch"
    );
    let expected = shifted_logic_bit(source, width, amount, kind, signed, position);
    let arithmetic = matches!(kind, Shift::ArithmeticLeft | Shift::ArithmeticRight);
    assert!(
        output.width() == width
            && output.signed() == (amount.is_some() && arithmetic && signed)
            && read_logic(payload, xz, position) == expected,
        "public shift semantics mismatch"
    );
}

// The first four calls of check_shift: unsigned source and unsigned amount,
// source width 1..=64 and arbitrary four-state 64-bit amount. Later sign groups
// and BigUint inputs remain executable comparisons.
#[kani::proof]
#[kani::unwind(1)]
#[kani::use_stub_set(small_native_shift)]
fn public_shift_unsigned_initial_calls_match_reference_semantics() {
    let width = usize::from(kani::any::<u8>() & 63) + 1;
    let mask = u64::MAX >> (64 - width);
    let source = (
        u128::from(kani::any::<u64>() & mask),
        u128::from(kani::any::<u64>() & mask),
    );
    let amount = (
        u128::from(kani::any::<u64>()),
        u128::from(kani::any::<u64>()),
    );
    let decoded = if amount.1 == 0 {
        Some(usize::try_from(amount.0).unwrap_or(usize::MAX))
    } else {
        None
    };
    let position = usize::from(kani::any::<u8>() & 63);
    let source_value = input(width, source, false);
    let amount_value = input(64, amount, false);
    let mut cache = MaskCache::default();
    let output =
        Op::LogicShiftL.eval_value_binary(&source_value, &amount_value, width, false, &mut cache);
    assert_result(
        output,
        source,
        width,
        false,
        decoded,
        Shift::LogicalLeft,
        position,
    );
    let output =
        Op::LogicShiftR.eval_value_binary(&source_value, &amount_value, width, false, &mut cache);
    assert_result(
        output,
        source,
        width,
        false,
        decoded,
        Shift::LogicalRight,
        position,
    );
    let output =
        Op::ArithShiftL.eval_value_binary(&source_value, &amount_value, width, false, &mut cache);
    assert_result(
        output,
        source,
        width,
        false,
        decoded,
        Shift::ArithmeticLeft,
        position,
    );
    let output =
        Op::ArithShiftR.eval_value_binary(&source_value, &amount_value, width, false, &mut cache);
    assert_result(
        output,
        source,
        width,
        false,
        decoded,
        Shift::ArithmeticRight,
        position,
    );
    assert_input(&source_value, width, source, false);
    assert_input(&amount_value, 64, amount, false);
    kani::cover!(width == 1, "public shift source width 1 is reachable");
    kani::cover!(width == 64, "public shift source width 64 is reachable");
    kani::cover!(decoded == Some(0), "public shift zero amount is reachable");
    kani::cover!(
        decoded == Some(width - 1),
        "public shift last-bit amount is reachable"
    );
    kani::cover!(
        decoded == Some(width),
        "public shift source-width amount is reachable"
    );
    kani::cover!(decoded == Some(64), "public shift amount 64 is reachable");
    kani::cover!(
        decoded == Some(usize::MAX),
        "public shift maximum amount is reachable"
    );
    kani::cover!(
        decoded.is_none(),
        "public shift unknown amount is reachable"
    );
    kani::cover!(amount.1 & !amount.0 != 0, "public shift amount contains X");
    kani::cover!(amount.1 & amount.0 != 0, "public shift amount contains Z");
    kani::cover!(
        amount.1 & (1u128 << 63) != 0,
        "public shift amount has an upper unknown"
    );
    kani::cover!(source.1 & !source.0 != 0, "public shift source contains X");
    kani::cover!(source.1 & source.0 != 0, "public shift source contains Z");
    kani::cover!(
        source.1 == 0 && source.0 & (1u128 << (width - 1)) != 0,
        "public shift source high bit is reachable"
    );
    kani::cover!(
        position == 63,
        "public shift highest native observation is reachable"
    );
}
