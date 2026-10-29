//! Public shift trace at all normalized four-state native source widths 1..=64.
//! The initial group has unsigned sources and amounts, with arbitrary four-state
//! 64-bit amount planes decoded as unsigned magnitudes with saturation. Four shifts
//! execute on one cache in test order. At fixed widths 1..=4 and 31/32/63/64,
//! all later source and amount sign groups have proofs on that same cache.
//! Result width stays native
//! source width. Context widening 4->8/64 has separate fresh-cache proofs;
//! Four shared 128-bit amount fixtures also have word64 caller proofs. Their
//! production constructors are checked separately by finite native comparisons.
//! Other BigUint amount/source paths remain executable comparisons.
//! IEEE 1800 11.4.10 is modeled independently by shifted_logic_bit.
use veryl_analyzer::ir::Op;
use veryl_analyzer::value::{MaskCache, Value, ValueU64};
use veryl_formal_semantics::logic4::{Logic4, read_logic};
use veryl_formal_semantics::shift::{Shift, shifted_logic_bit, shifted_resized_logic_bit};

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

fn native_input(width: usize, planes: (u128, u128), signed: bool) -> ValueU64 {
    let Value::U64(native) = input(width, planes, signed) else {
        panic!("public shift semantics mismatch");
    };
    native
}
fn shift_group<const SIGNED: bool, const AMOUNT_SIGNED: bool>(
    source_value: &Value,
    source: (u128, u128),
    width: usize,
    amount: (u128, u128),
    observe: bool,
    selected: usize,
    observed: &mut Option<ValueU64>,
    cache: &mut MaskCache,
) {
    let amount_value = Value::U64(native_input(64, amount, AMOUNT_SIGNED));
    let output =
        Op::LogicShiftL.eval_value_binary(source_value, &amount_value, width, SIGNED, cache);
    let Value::U64(output) = output else {
        panic!("public shift semantics mismatch");
    };
    if observe && selected == 0 {
        *observed = Some(output);
    }
    let output =
        Op::LogicShiftR.eval_value_binary(source_value, &amount_value, width, SIGNED, cache);
    let Value::U64(output) = output else {
        panic!("public shift semantics mismatch");
    };
    if observe && selected == 1 {
        *observed = Some(output);
    }
    let output =
        Op::ArithShiftL.eval_value_binary(source_value, &amount_value, width, SIGNED, cache);
    let Value::U64(output) = output else {
        panic!("public shift semantics mismatch");
    };
    if observe && selected == 2 {
        *observed = Some(output);
    }
    let output =
        Op::ArithShiftR.eval_value_binary(source_value, &amount_value, width, SIGNED, cache);
    let Value::U64(output) = output else {
        panic!("public shift semantics mismatch");
    };
    if observe && selected == 3 {
        *observed = Some(output);
    }
    assert_input(&amount_value, 64, amount, AMOUNT_SIGNED);
    assert_input(source_value, width, source, SIGNED);
}

// Three later sign groups follow the initial unsigned group. Every operator
// executes in order; phase/operator selection observes each of the twelve later
// results. Inputs and discarded results are reified as native values, failing
// on unsupported variants rather than expanding unreachable BigUint drops.
// Widths are separate constants for solver cost: 1..=4 and 31/32/63/64. Amounts
// admit all 64-bit four-state planes and are decoded as unsigned magnitudes.
// Positions 0..63 plus explicit high getter checks cover all 128 result bits.
fn native_shift_continuation<const WIDTH: usize>() {
    let width = WIDTH;
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
    let selected = usize::from(kani::any::<u8>() & 3);
    let phase = usize::from(kani::any::<u8>() & 3).max(1);
    let mut observed = None;
    let mut cache = MaskCache::default();
    let unsigned_source = Value::U64(native_input(width, source, false));
    shift_group::<false, false>(
        &unsigned_source,
        source,
        width,
        amount,
        false,
        selected,
        &mut observed,
        &mut cache,
    );
    shift_group::<false, true>(
        &unsigned_source,
        source,
        width,
        amount,
        phase == 1,
        selected,
        &mut observed,
        &mut cache,
    );
    let signed_source = Value::U64(native_input(width, source, true));
    shift_group::<true, false>(
        &signed_source,
        source,
        width,
        amount,
        phase == 2,
        selected,
        &mut observed,
        &mut cache,
    );
    shift_group::<true, true>(
        &signed_source,
        source,
        width,
        amount,
        phase == 3,
        selected,
        &mut observed,
        &mut cache,
    );
    let kind = match selected {
        0 => Shift::LogicalLeft,
        1 => Shift::LogicalRight,
        2 => Shift::ArithmeticLeft,
        _ => Shift::ArithmeticRight,
    };
    assert_result(
        Value::U64(observed.expect("public shift semantics mismatch")),
        source,
        width,
        phase >= 2,
        decoded,
        kind,
        position,
    );
    kani::cover!(
        phase == 1 && selected == 0 && source == (1, 0) && amount == (0, 0) && position == 0,
        "public shift continuation signed amount is reachable"
    );
    kani::cover!(
        phase == 2
            && selected == 3
            && source == (1u128 << (WIDTH - 1), 0)
            && amount == (WIDTH as u128, 0)
            && position == WIDTH - 1,
        "public shift continuation signed one fill is reachable"
    );
    kani::cover!(
        phase == 3
            && selected == 3
            && source == (0, 1u128 << (WIDTH - 1))
            && amount == (usize::MAX as u128, 0)
            && position == 0,
        "public shift continuation signed X fill is reachable"
    );
    kani::cover!(
        phase == 3
            && selected == 3
            && source == (1u128 << (WIDTH - 1), 1u128 << (WIDTH - 1))
            && amount == (1, 0)
            && position == WIDTH - 1,
        "public shift continuation signed Z fill is reachable"
    );
    kani::cover!(
        phase == 1
            && selected == 1
            && source == (u128::from(mask), 0)
            && amount == (0, 1)
            && position == 0,
        "public shift continuation unknown amount is reachable"
    );
    kani::cover!(
        phase == 2 && selected == 2 && source == (1, 0) && amount == (64, 0) && position == 63,
        "public shift continuation arithmetic left overshift is reachable"
    );
    kani::cover!(
        phase == 3
            && selected == 1
            && source == (u128::from(mask), 0)
            && amount == (1u128 << 63, 1u128 << 63)
            && position == 0,
        "public shift continuation upper Z amount is reachable"
    );
}

#[kani::proof]
#[kani::unwind(1)]
#[kani::use_stub_set(small_native_shift)]
fn public_small_shift_continuation_width_1_matches_reference_semantics() {
    native_shift_continuation::<1>();
}

#[kani::proof]
#[kani::unwind(1)]
#[kani::use_stub_set(small_native_shift)]
fn public_small_shift_continuation_width_2_matches_reference_semantics() {
    native_shift_continuation::<2>();
}

#[kani::proof]
#[kani::unwind(1)]
#[kani::use_stub_set(small_native_shift)]
fn public_small_shift_continuation_width_3_matches_reference_semantics() {
    native_shift_continuation::<3>();
}

#[kani::proof]
#[kani::unwind(1)]
#[kani::use_stub_set(small_native_shift)]
fn public_small_shift_continuation_width_4_matches_reference_semantics() {
    native_shift_continuation::<4>();
}

#[kani::proof]
#[kani::unwind(1)]
#[kani::use_stub_set(small_native_shift)]
fn public_shift_boundary_width_31_matches_reference_semantics() {
    native_shift_continuation::<31>();
}

#[kani::proof]
#[kani::unwind(1)]
#[kani::use_stub_set(small_native_shift)]
fn public_shift_boundary_width_32_matches_reference_semantics() {
    native_shift_continuation::<32>();
}

#[kani::proof]
#[kani::unwind(1)]
#[kani::use_stub_set(small_native_shift)]
fn public_shift_boundary_width_63_matches_reference_semantics() {
    native_shift_continuation::<63>();
}

#[kani::proof]
#[kani::unwind(1)]
#[kani::use_stub_set(small_native_shift)]
fn public_shift_boundary_width_64_matches_reference_semantics() {
    native_shift_continuation::<64>();
}

// Select one operator with the fresh cache used by the direct context test.
// Widths 4->8/64 are constants; both signs and arbitrary four-state native
// amount planes are admitted. Known amounts use the direct test's constructor.
// The independent model composes sizing before selecting the shifted bit.
fn public_native_shift_widening<const SOURCE: usize, const WIDTH: usize>() {
    let source_mask = u64::MAX >> (64 - SOURCE);
    let source = (
        u128::from(kani::any::<u64>() & source_mask),
        u128::from(kani::any::<u64>() & source_mask),
    );
    let signed: bool = kani::any();
    let amount_signed: bool = kani::any();
    let amount = (
        u128::from(kani::any::<u64>()),
        u128::from(kani::any::<u64>()),
    );
    let source_value = Value::U64(native_input(SOURCE, source, signed));
    let amount_value = if amount.1 == 0 {
        Value::new(amount.0 as u64, 64, amount_signed)
    } else {
        Value::from_u128(amount.0, amount.1, 64, amount_signed)
    };
    assert_input(&amount_value, 64, amount, amount_signed);
    let Value::U64(amount_native) = amount_value else {
        panic!("public shift semantics mismatch");
    };
    let amount_value = Value::U64(amount_native);
    let decoded = if amount.1 == 0 {
        Some(usize::try_from(amount.0).unwrap_or(usize::MAX))
    } else {
        None
    };
    let selected = usize::from(kani::any::<u8>() & 3);
    let position = usize::from(kani::any::<u8>() & 63);
    let mut cache = MaskCache::default();
    let (output, kind) = match selected {
        0 => (
            Op::LogicShiftL.eval_value_binary(
                &source_value,
                &amount_value,
                WIDTH,
                signed,
                &mut cache,
            ),
            Shift::LogicalLeft,
        ),
        1 => (
            Op::LogicShiftR.eval_value_binary(
                &source_value,
                &amount_value,
                WIDTH,
                signed,
                &mut cache,
            ),
            Shift::LogicalRight,
        ),
        2 => (
            Op::ArithShiftL.eval_value_binary(
                &source_value,
                &amount_value,
                WIDTH,
                signed,
                &mut cache,
            ),
            Shift::ArithmeticLeft,
        ),
        _ => (
            Op::ArithShiftR.eval_value_binary(
                &source_value,
                &amount_value,
                WIDTH,
                signed,
                &mut cache,
            ),
            Shift::ArithmeticRight,
        ),
    };
    let Value::U64(native) = output else {
        panic!("public shift semantics mismatch");
    };
    let output = Value::U64(native);
    let expected =
        shifted_resized_logic_bit(source, SOURCE, WIDTH, decoded, kind, signed, position);
    let arithmetic = matches!(kind, Shift::ArithmeticLeft | Shift::ArithmeticRight);
    let payload = output.payload_u128();
    let xz = output.mask_xz_u128();
    assert!(
        output.width() == WIDTH
            && output.signed() == (decoded.is_some() && arithmetic && signed)
            && payload >> 64 == 0
            && xz >> 64 == 0
            && read_logic(payload, xz, position) == expected,
        "public shift semantics mismatch"
    );
    assert_input(&source_value, SOURCE, source, signed);
    assert_input(&amount_value, 64, amount, amount_signed);
    kani::cover!(
        selected == 0
            && signed
            && source == (1u128 << (SOURCE - 1), 0)
            && amount == (1, 0)
            && position == WIDTH - 1,
        "public shift widening signed one before left shift is reachable"
    );
    kani::cover!(
        selected == 1
            && signed
            && source == (0, 1u128 << (SOURCE - 1))
            && amount == (1, 0)
            && position == WIDTH - 2,
        "public shift widening signed X before logical right is reachable"
    );
    kani::cover!(
        selected == 3
            && signed
            && source == (1u128 << (SOURCE - 1), 1u128 << (SOURCE - 1))
            && amount == (WIDTH as u128, 0)
            && position == 0,
        "public shift widening signed Z fill is reachable"
    );
    kani::cover!(
        selected == 2
            && !signed
            && source == (1, 0)
            && amount == (0, 0)
            && amount_signed
            && position == 0,
        "public shift widening unsigned arithmetic left is reachable"
    );
    kani::cover!(
        selected == 3
            && signed
            && source == (1, 0)
            && amount == (1u128 << 63, 1u128 << 63)
            && amount_signed
            && position == 0,
        "public shift widening upper Z amount is reachable"
    );
    kani::cover!(
        selected == 0
            && !signed
            && source == (1, 0)
            && amount == (usize::MAX as u128, 0)
            && position == 63,
        "public shift widening overshift padding is reachable"
    );
}
#[kani::proof]
#[kani::unwind(1)]
#[kani::use_stub_set(small_native_shift)]
fn public_shift_widening_4_to_64_matches_reference_semantics() {
    public_native_shift_widening::<4, 64>();
}

#[kani::proof]
#[kani::unwind(1)]
#[kani::use_stub_set(small_native_shift)]
fn public_shift_widening_4_to_8_matches_reference_semantics() {
    public_native_shift_widening::<4, 8>();
}

#[path = "../../shift_amount_cases.rs"]
mod shift_amount_cases;
use shift_amount_cases::WIDE_SHIFT_AMOUNTS;

// Caller proof inputs are canonical numerical BigUint values. The actual Veryl
// constructor is covered by the native finite bitwise check of all shared cases
// and both signs. No constructor stub is installed. BigUint count decoding,
// input getters and allocation remain real; guards forbid BigUint source paths.
kani::stub_set!(
    native_source_big_amount,
    stub(veryl_analyzer::value::MaskCache::get, reject_shift_cache),
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
);
fn assert_big_amount(input: &Value, amount: (u128, u128), signed: bool) {
    assert!(
        matches!(input, Value::BigUint(_))
            && input.width() == 128
            && input.signed() == signed
            && input.payload_u128() == amount.0
            && input.mask_xz_u128() == amount.1,
        "public shift semantics mismatch"
    );
}

fn big_amount_shift_group<const SIGNED: bool, const AMOUNT_SIGNED: bool>(
    source_value: &Value,
    source: (u128, u128),
    width: usize,
    amount: (u128, u128),
    observe: bool,
    selected: usize,
    observed: &mut Option<ValueU64>,
    cache: &mut MaskCache,
) {
    let amount_value = Value::BigUint(veryl_analyzer::value::ValueBigUint {
        payload: Box::new(num_bigint::BigUint::from(amount.0)),
        mask_xz: Box::new(num_bigint::BigUint::from(amount.1)),
        width: 128,
        signed: AMOUNT_SIGNED,
    });
    let Value::BigUint(big) = amount_value else {
        panic!("public shift semantics mismatch");
    };
    assert!(
        big.payload.bits() <= 128 && big.mask_xz.bits() <= 128,
        "public shift semantics mismatch"
    );
    let amount_value = Value::BigUint(big);
    let output =
        Op::LogicShiftL.eval_value_binary(source_value, &amount_value, width, SIGNED, cache);
    let Value::U64(output) = output else {
        panic!("public shift semantics mismatch");
    };
    if observe && selected == 0 {
        *observed = Some(output);
    }
    let output =
        Op::LogicShiftR.eval_value_binary(source_value, &amount_value, width, SIGNED, cache);
    let Value::U64(output) = output else {
        panic!("public shift semantics mismatch");
    };
    if observe && selected == 1 {
        *observed = Some(output);
    }
    let output =
        Op::ArithShiftL.eval_value_binary(source_value, &amount_value, width, SIGNED, cache);
    let Value::U64(output) = output else {
        panic!("public shift semantics mismatch");
    };
    if observe && selected == 2 {
        *observed = Some(output);
    }
    let output =
        Op::ArithShiftR.eval_value_binary(source_value, &amount_value, width, SIGNED, cache);
    let Value::U64(output) = output else {
        panic!("public shift semantics mismatch");
    };
    if observe && selected == 3 {
        *observed = Some(output);
    }
    assert_big_amount(&amount_value, amount, AMOUNT_SIGNED);
    assert_input(source_value, width, source, SIGNED);
}

fn big_amount_native_trace<const PAYLOAD: u128, const XZ: u128>() {
    let width = 64;
    let source = (
        u128::from(kani::any::<u64>()),
        u128::from(kani::any::<u64>()),
    );
    let amount = (PAYLOAD, XZ);
    let decoded = if XZ != 0 {
        None
    } else {
        Some(usize::try_from(PAYLOAD).unwrap_or(usize::MAX))
    };
    let phase = usize::from(kani::any::<u8>() & 3);
    let selected = usize::from(kani::any::<u8>() & 3);
    let position = usize::from(kani::any::<u8>() & 63);
    let mut observed = None;
    let mut cache = MaskCache::default();
    let unsigned = Value::U64(native_input(width, source, false));
    big_amount_shift_group::<false, false>(
        &unsigned,
        source,
        width,
        amount,
        phase == 0,
        selected,
        &mut observed,
        &mut cache,
    );
    big_amount_shift_group::<false, true>(
        &unsigned,
        source,
        width,
        amount,
        phase == 1,
        selected,
        &mut observed,
        &mut cache,
    );
    let signed = Value::U64(native_input(width, source, true));
    big_amount_shift_group::<true, false>(
        &signed,
        source,
        width,
        amount,
        phase == 2,
        selected,
        &mut observed,
        &mut cache,
    );
    big_amount_shift_group::<true, true>(
        &signed,
        source,
        width,
        amount,
        phase == 3,
        selected,
        &mut observed,
        &mut cache,
    );
    let kind = match selected {
        0 => Shift::LogicalLeft,
        1 => Shift::LogicalRight,
        2 => Shift::ArithmeticLeft,
        _ => Shift::ArithmeticRight,
    };
    assert_result(
        Value::U64(observed.expect("public shift semantics mismatch")),
        source,
        width,
        phase >= 2,
        decoded,
        kind,
        position,
    );
    kani::cover!(
        phase == 1 && selected == 0 && source == (1, 0) && position == 0,
        "public wide amount trace signed operand is reachable"
    );
    kani::cover!(
        phase == 2 && selected == 3 && source == (1u128 << 63, 0) && position == 63,
        "public wide amount trace negative source is reachable"
    );
    kani::cover!(
        phase == 3 && selected == 3 && source == (0, 1u128 << 63) && position == 0,
        "public wide amount trace signed X source is reachable"
    );
    kani::cover!(
        phase == 0 && selected == 1 && source == (1u128 << 63, 1u128 << 63) && position == 63,
        "public wide amount trace unsigned Z source is reachable"
    );
    kani::cover!(
        phase == 2 && selected == 2 && source == (1, 0) && position == 63,
        "public wide amount trace arithmetic left is reachable"
    );
}

#[kani::proof]
#[kani::unwind(2)]
#[kani::use_stub_set(native_source_big_amount)]
fn public_word64_shift_small_amount_matches_reference() {
    big_amount_native_trace::<{ WIDE_SHIFT_AMOUNTS[0].0 }, { WIDE_SHIFT_AMOUNTS[0].1 }>();
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(native_source_big_amount)]
fn public_word64_shift_high_amount_matches_reference() {
    big_amount_native_trace::<{ WIDE_SHIFT_AMOUNTS[1].0 }, { WIDE_SHIFT_AMOUNTS[1].1 }>();
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(native_source_big_amount)]
fn public_word64_shift_upper_x_amount_matches_reference() {
    big_amount_native_trace::<{ WIDE_SHIFT_AMOUNTS[2].0 }, { WIDE_SHIFT_AMOUNTS[2].1 }>();
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(native_source_big_amount)]
fn public_word64_shift_upper_z_amount_matches_reference() {
    big_amount_native_trace::<{ WIDE_SHIFT_AMOUNTS[3].0 }, { WIDE_SHIFT_AMOUNTS[3].1 }>();
}
