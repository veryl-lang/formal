//! Public bitwise operations over all normalized four-state native sources.
//! Binary sources have independent widths/signs in 1..=64; the caller sign is
//! their conjunction, matching the executable trace at the common source width.
//! A second four-call sequence admits every following context from natural through
//! 64, with the first sequence executed on the same cache before observation.
//! Unary NOT checks both caller signs at natural
//! and widened width. Every prefix call executes on the same cache; all outputs,
//! full u128 getter positions, metadata and source preservation are checked.
//! IEEE 1800 11.4.8 (Tables 11-11..11-15) and 11.8.2 are modeled independently.
use veryl_analyzer::ir::Op;
use veryl_analyzer::value::{MaskCache, Value};
use veryl_formal_semantics::comparison::Operand;
use veryl_formal_semantics::gates::{self, BitOp};
use veryl_formal_semantics::logic4::{Logic4, read_logic};
use veryl_formal_semantics::resize::extended_logic_bit;

// BigUint conversion/cloning/masks, heap allocation, and mask-cache access are
// forbidden in these small native proofs. Each replacement panics, proving it unreachable
// for every admitted input. Bitwise operations, expansion, constructors, and getters run
// unchanged. Kani 0.68 otherwise expands unreachable BigUint paths before solving.
fn reject_gate_biguint(_: u128) -> num_bigint::BigUint {
    panic!("public gate semantics mismatch");
}
fn reject_gate_cache(_: &mut veryl_analyzer::value::MaskCache, _: usize) -> &num_bigint::BigUint {
    panic!("public gate semantics mismatch");
}
fn reject_gate_from_u64(_: u64) -> num_bigint::BigUint {
    panic!("public gate semantics mismatch");
}
fn reject_gate_clone(
    _: &veryl_analyzer::value::ValueBigUint,
) -> veryl_analyzer::value::ValueBigUint {
    panic!("public gate semantics mismatch");
}
fn reject_gate_mask(_: usize) -> num_bigint::BigUint {
    panic!("public gate semantics mismatch");
}
unsafe fn reject_gate_allocation(_: std::alloc::Layout) -> *mut u8 {
    panic!("public gate semantics mismatch");
}

fn reject_gate_plane(_: &veryl_analyzer::value::ValueBigUint) -> &num_bigint::BigUint {
    panic!("public gate semantics mismatch");
}
fn reject_gate_to_native(
    _: &veryl_analyzer::value::ValueBigUint,
) -> Option<veryl_analyzer::value::ValueU64> {
    panic!("public gate semantics mismatch");
}
kani::stub_set!(
    small_native_gate,
    stub(veryl_analyzer::value::u128_to_biguint, reject_gate_biguint),
    stub(veryl_analyzer::value::MaskCache::get, reject_gate_cache),
    stub(
        <num_bigint::BigUint as std::convert::From<u64>>::from,
        reject_gate_from_u64
    ),
    stub(
        <veryl_analyzer::value::ValueBigUint as std::clone::Clone>::clone,
        reject_gate_clone
    ),
    stub(
        veryl_analyzer::value::ValueBigUint::gen_mask,
        reject_gate_mask
    ),
    stub(
        veryl_analyzer::value::ValueBigUint::payload,
        reject_gate_plane
    ),
    stub(
        veryl_analyzer::value::ValueBigUint::mask_xz,
        reject_gate_plane
    ),
    stub(
        veryl_analyzer::value::ValueBigUint::to_value_u64,
        reject_gate_to_native
    ),
    stub(std::alloc::alloc, reject_gate_allocation),
    stub(std::alloc::alloc_zeroed, reject_gate_allocation),
);

fn assert_input(input: &Value, before: Operand) {
    assert!(
        matches!(input, Value::U64(_))
            && input.width() == before.width
            && input.signed() == before.signed
            && input.payload_u128() == before.planes.0
            && input.mask_xz_u128() == before.planes.1,
        "public gate semantics mismatch"
    );
}
fn input() -> (Value, Operand) {
    let width = usize::from(kani::any::<u8>() & 63) + 1;
    let mask = u64::MAX >> (64 - width);
    let before = Operand {
        planes: (
            u128::from(kani::any::<u64>() & mask),
            u128::from(kani::any::<u64>() & mask),
        ),
        width,
        signed: kani::any(),
    };
    let value = Value::from_u128(before.planes.0, before.planes.1, width, before.signed);
    assert_input(&value, before);
    let Value::U64(native) = value else {
        panic!("public gate semantics mismatch");
    };
    (Value::U64(native), before)
}
fn assert_binary(
    output: Value,
    a: Operand,
    b: Operand,
    width: usize,
    position: usize,
    kind: BitOp,
) {
    let Value::U64(native) = output else {
        panic!("public gate semantics mismatch");
    };
    let output = Value::U64(native);
    let signed = a.signed && b.signed;
    let expected = if position < width {
        gates::binary(
            extended_logic_bit(a.planes, a.width, width, a.signed, signed, position),
            extended_logic_bit(b.planes, b.width, width, b.signed, signed, position),
            kind,
        )
    } else {
        Logic4::Zero
    };
    assert!(
        output.width() == width
            && !output.signed()
            && read_logic(output.payload_u128(), output.mask_xz_u128(), position) == expected,
        "public gate semantics mismatch"
    );
}

#[kani::proof]
#[kani::unwind(1)]
#[kani::use_stub_set(small_native_gate)]
fn public_bitwise_native_sequence_matches_reference_semantics() {
    let (a, ar) = input();
    let (b, br) = input();
    let natural = ar.width.max(br.width);
    let width = natural;
    let signed = ar.signed && br.signed;
    let position = usize::from(kani::any::<u8>() & 127);
    let mut cache = MaskCache::default();
    let output = Op::BitAnd.eval_value_binary(&a, &b, natural, signed, &mut cache);
    assert_binary(output, ar, br, natural, position, BitOp::And);
    let output = Op::BitOr.eval_value_binary(&a, &b, natural, signed, &mut cache);
    assert_binary(output, ar, br, natural, position, BitOp::Or);
    let output = Op::BitXor.eval_value_binary(&a, &b, natural, signed, &mut cache);
    assert_binary(output, ar, br, natural, position, BitOp::Xor);
    let output = Op::BitXnor.eval_value_binary(&a, &b, natural, signed, &mut cache);
    assert_binary(output, ar, br, natural, position, BitOp::Xnor);
    assert_input(&a, ar);
    assert_input(&b, br);
    kani::cover!(ar.width == 1, "public bitwise LHS width 1 is reachable");
    kani::cover!(ar.width == 64, "public bitwise LHS width 64 is reachable");
    kani::cover!(br.width == 1, "public bitwise RHS width 1 is reachable");
    kani::cover!(br.width == 64, "public bitwise RHS width 64 is reachable");
    kani::cover!(ar.width > br.width, "public bitwise wider LHS is reachable");
    kani::cover!(ar.width < br.width, "public bitwise wider RHS is reachable");
    kani::cover!(
        width == natural,
        "public bitwise unchanged context is reachable"
    );
    kani::cover!(width == 64, "public bitwise width 64 is reachable");
    kani::cover!(signed, "public bitwise signed context is reachable");
    kani::cover!(
        ar.signed != br.signed,
        "public bitwise mixed signs are reachable"
    );
    kani::cover!(
        ar.planes.1 & !ar.planes.0 != 0,
        "public bitwise source X is reachable"
    );
    kani::cover!(
        br.planes.1 & br.planes.0 != 0,
        "public bitwise source Z is reachable"
    );
    kani::cover!(position == 127, "public bitwise u128 padding is reachable");
}

fn assert_not(output: Value, before: Operand, width: usize, signed: bool, position: usize) {
    let Value::U64(native) = output else {
        panic!("public gate semantics mismatch");
    };
    let output = Value::U64(native);
    let expected = if position < width {
        gates::not(extended_logic_bit(
            before.planes,
            before.width,
            width,
            before.signed,
            signed,
            position,
        ))
    } else {
        Logic4::Zero
    };
    assert!(
        output.width() == width
            && output.signed() == (before.signed && (width == before.width || signed))
            && read_logic(output.payload_u128(), output.mask_xz_u128(), position) == expected,
        "public gate semantics mismatch"
    );
}

#[kani::proof]
#[kani::unwind(1)]
#[kani::use_stub_set(small_native_gate)]
fn public_bit_not_native_sequence_matches_reference_semantics() {
    let (input, before) = input();
    let natural = before.width;
    let width = natural.max(usize::from(kani::any::<u8>() & 63) + 1);
    let position = usize::from(kani::any::<u8>() & 127);
    let mut cache = MaskCache::default();
    let output = Op::BitNot.eval_value_unary(&input, natural, false, &mut cache);
    assert_not(output, before, natural, false, position);
    let output = Op::BitNot.eval_value_unary(&input, natural, true, &mut cache);
    assert_not(output, before, natural, true, position);
    let output = Op::BitNot.eval_value_unary(&input, width, false, &mut cache);
    assert_not(output, before, width, false, position);
    let output = Op::BitNot.eval_value_unary(&input, width, true, &mut cache);
    assert_not(output, before, width, true, position);
    assert_input(&input, before);
    kani::cover!(natural == 1, "public NOT source width 1 is reachable");
    kani::cover!(natural == 64, "public NOT source width 64 is reachable");
    kani::cover!(width == 64, "public NOT result width 64 is reachable");
    kani::cover!(width > natural, "public NOT widened context is reachable");
    kani::cover!(
        width == natural,
        "public NOT unchanged context is reachable"
    );
    kani::cover!(before.signed, "public NOT signed source is reachable");
    kani::cover!(!before.signed, "public NOT unsigned source is reachable");
    kani::cover!(
        before.planes.1 & !before.planes.0 != 0,
        "public NOT source X is reachable"
    );
    kani::cover!(
        before.planes.1 & before.planes.0 != 0,
        "public NOT source Z is reachable"
    );
    kani::cover!(position == 127, "public NOT u128 padding is reachable");
}

fn assert_logical_and(output: Value, width: usize, expected: Logic4) {
    let planes = match expected {
        Logic4::Zero => (0, 0),
        Logic4::One => (1, 0),
        Logic4::X => (0, 1),
        Logic4::Z => unreachable!(),
    };
    assert!(
        matches!(output, Value::U64(_))
            && output.width() == width
            && !output.signed()
            && (output.payload_u128(), output.mask_xz_u128()) == planes,
        "public gate semantics mismatch"
    );
}

// IEEE 1800 11.4.9: scalar AND fold, with a known zero dominating X/Z.
// All normalized four-state source widths 1..=64, either source sign, and
// independent second NOT context natural..=64. Four NOT calls share one cache
// before reduction AND at result widths 1 then 8. BigUint results stay executable.
#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(small_native_gate)]
#[kani::stub(veryl_analyzer::value::biguint_to_u128, reject_logical_biguint_getter)]
fn public_reduction_and_native_continuation_matches_reference_semantics() {
    let (value, before) = input();
    let natural = before.width;
    let width = natural.max(usize::from(kani::any::<u8>() & 63) + 1);
    let mut cache = MaskCache::default();
    let _ = Op::BitNot.eval_value_unary(&value, natural, false, &mut cache);
    let _ = Op::BitNot.eval_value_unary(&value, natural, true, &mut cache);
    let _ = Op::BitNot.eval_value_unary(&value, width, false, &mut cache);
    let _ = Op::BitNot.eval_value_unary(&value, width, true, &mut cache);
    // reduction-and-reference proves this scalar tree equals the sequential fold.
    let expected = native_reduction_and_tree(before);
    let output = Op::BitAnd.eval_value_unary(&value, 1, before.signed, &mut cache);
    assert_logical_and(output, 1, expected);
    let output = Op::BitAnd.eval_value_unary(&value, 8, before.signed, &mut cache);
    assert_logical_and(output, 8, expected);
    assert_input(&value, before);
    kani::cover!(
        natural == 1,
        "public reduction AND source width 1 is reachable"
    );
    kani::cover!(
        natural == 64,
        "public reduction AND source width 64 is reachable"
    );
    kani::cover!(
        width == natural,
        "public reduction AND unchanged NOT context is reachable"
    );
    kani::cover!(
        width > natural,
        "public reduction AND widened NOT context is reachable"
    );
    kani::cover!(
        width == 64,
        "public reduction AND NOT context width 64 is reachable"
    );
    kani::cover!(
        before.signed,
        "public reduction AND signed source is reachable"
    );
    kani::cover!(
        !before.signed,
        "public reduction AND unsigned source is reachable"
    );
    kani::cover!(
        expected == Logic4::Zero,
        "public reduction AND false result is reachable"
    );
    kani::cover!(
        expected == Logic4::One,
        "public reduction AND true result is reachable"
    );
    kani::cover!(
        expected == Logic4::X,
        "public reduction AND X result is reachable"
    );
    kani::cover!(
        before.planes.1 != 0 && expected == Logic4::Zero,
        "public reduction AND zero dominates unknown is reachable"
    );
    kani::cover!(
        before.planes.0 & before.planes.1 != 0,
        "public reduction AND source Z is reachable"
    );
    kani::cover!(
        before.planes.1 & !before.planes.0 != 0,
        "public reduction AND source X is reachable"
    );
}

// BigUint results are permitted, while source-only BigUint paths and mask-cache
// access remain forbidden. Numeric conversions, allocation and getters run unchanged.
kani::stub_set!(
    native_gate_wide_result,
    stub(veryl_analyzer::value::u128_to_biguint, reject_gate_biguint),
    stub(veryl_analyzer::value::MaskCache::get, reject_gate_cache),
    stub(
        <veryl_analyzer::value::ValueBigUint as std::clone::Clone>::clone,
        reject_gate_clone
    ),
    stub(
        veryl_analyzer::value::ValueBigUint::gen_mask,
        reject_gate_mask
    ),
    stub(
        veryl_analyzer::value::ValueBigUint::payload,
        reject_gate_plane
    ),
    stub(
        veryl_analyzer::value::ValueBigUint::mask_xz,
        reject_gate_plane
    ),
    stub(
        veryl_analyzer::value::ValueBigUint::to_value_u64,
        reject_gate_to_native
    ),
);

fn assert_wide_boolean(output: Value, width: usize, expected: Logic4) {
    let planes = match expected {
        Logic4::Zero => (0, 0),
        Logic4::One => (1, 0),
        Logic4::X => (0, 1),
        Logic4::Z => unreachable!(),
    };
    let Value::BigUint(big) = output else {
        panic!("public gate semantics mismatch");
    };
    assert!(
        big.payload.bits() <= 128 && big.mask_xz.bits() <= 128,
        "public gate semantics mismatch"
    );
    let output = Value::BigUint(big);
    assert!(
        output.width() == width
            && !output.signed()
            && (output.payload_u128(), output.mask_xz_u128()) == planes,
        "public gate semantics mismatch"
    );
}

// Balanced scalar AND folding avoids increasing the global unwind bound for
// real BigUint loops. Out-of-width leaves use AND's identity, One.
fn scalar_and_four(states: [Logic4; 4]) -> Logic4 {
    gates::binary(
        gates::binary(states[0], states[1], BitOp::And),
        gates::binary(states[2], states[3], BitOp::And),
        BitOp::And,
    )
}

fn source_and_four(source: Operand, offset: usize) -> Logic4 {
    let bit = |position| {
        if position < source.width {
            read_logic(source.planes.0, source.planes.1, position)
        } else {
            Logic4::One
        }
    };
    scalar_and_four([
        bit(offset),
        bit(offset + 1),
        bit(offset + 2),
        bit(offset + 3),
    ])
}

fn source_and_sixteen(source: Operand, offset: usize) -> Logic4 {
    scalar_and_four([
        source_and_four(source, offset),
        source_and_four(source, offset + 4),
        source_and_four(source, offset + 8),
        source_and_four(source, offset + 12),
    ])
}

fn native_reduction_and_tree(source: Operand) -> Logic4 {
    scalar_and_four([
        source_and_sixteen(source, 0),
        source_and_sixteen(source, 16),
        source_and_sixteen(source, 32),
        source_and_sixteen(source, 48),
    ])
}

#[kani::proof]
#[kani::unwind(65)]
fn native_reduction_and_tree_matches_scalar_fold() {
    let source = Operand {
        planes: (kani::any(), kani::any()),
        width: usize::from(kani::any::<u8>() & 63) + 1,
        signed: kani::any(),
    };
    let expected = gates::reduced_logic(source.planes, source.width, gates::Reduction::And);
    assert!(
        native_reduction_and_tree(source) == expected,
        "reduction AND reference mismatch"
    );
    kani::cover!(
        source.width == 1,
        "reduction AND reference width 1 is reachable"
    );
    kani::cover!(
        source.width == 64,
        "reduction AND reference width 64 is reachable"
    );
    kani::cover!(
        expected == Logic4::Zero,
        "reduction AND reference zero is reachable"
    );
    kani::cover!(
        expected == Logic4::One,
        "reduction AND reference one is reachable"
    );
    kani::cover!(
        expected == Logic4::X,
        "reduction AND reference X is reachable"
    );
}

// IEEE 1800 11.4.9/11.8.2: the scalar AND result is zero extended.
// All normalized four-state native source widths and signs are admitted.
// Execute the native NOT prefix, then reductions at 1 and 8. The width-128
// proof also executes width 65 before observing its result, on the same cache.
// The separately checked scalar tree needs no loops. Unwind 3 bounds real
// BigUint conversion loops; unwinding and safety assertions remain enabled.
fn public_reduction_and_wide_continuation<const RESULT: usize>() {
    let (value, before) = input();
    let natural = before.width;
    let width = natural.max(usize::from(kani::any::<u8>() & 63) + 1);
    let mut cache = MaskCache::default();
    let _ = Op::BitNot.eval_value_unary(&value, natural, false, &mut cache);
    let _ = Op::BitNot.eval_value_unary(&value, natural, true, &mut cache);
    let _ = Op::BitNot.eval_value_unary(&value, width, false, &mut cache);
    let _ = Op::BitNot.eval_value_unary(&value, width, true, &mut cache);
    let expected = native_reduction_and_tree(before);
    let output = Op::BitAnd.eval_value_unary(&value, 1, before.signed, &mut cache);
    assert_logical_and(output, 1, expected);
    let output = Op::BitAnd.eval_value_unary(&value, 8, before.signed, &mut cache);
    assert_logical_and(output, 8, expected);
    if RESULT == 128 {
        let _ = Op::BitAnd.eval_value_unary(&value, 65, before.signed, &mut cache);
    }
    let output = Op::BitAnd.eval_value_unary(&value, RESULT, before.signed, &mut cache);
    assert_wide_boolean(output, RESULT, expected);
    assert_input(&value, before);
    kani::cover!(
        natural == 1
            && !before.signed
            && before.planes == (0, 0)
            && width == 1
            && expected == Logic4::Zero,
        "public wide reduction AND width 1 zero unchanged is reachable"
    );
    kani::cover!(
        natural == 64
            && before.signed
            && before.planes == (u64::MAX as u128, 0)
            && width == 64
            && expected == Logic4::One,
        "public wide reduction AND width 64 signed one is reachable"
    );
    kani::cover!(
        natural == 32
            && !before.signed
            && before.planes == (3, 0xffff_fffe)
            && width == 64
            && expected == Logic4::X,
        "public wide reduction AND widened X and Z condition is reachable"
    );
    kani::cover!(
        natural == 32
            && before.signed
            && before.planes == (0xffff_fffa, 0xffff_fffc)
            && width == 64
            && expected == Logic4::Zero,
        "public wide reduction AND widened zero dominates X and Z is reachable"
    );
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(native_gate_wide_result)]
fn public_reduction_and_width65_matches_reference_semantics() {
    public_reduction_and_wide_continuation::<65>();
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(native_gate_wide_result)]
fn public_reduction_and_width128_matches_reference_semantics() {
    public_reduction_and_wide_continuation::<128>();
}

// IEEE 1800 11.4.9/11.8.2: NAND negates the scalar AND fold and zero extends.
// All normalized four-state source widths 1..=64 and either source sign are free.
// Preserve the four-call native NOT prefix and AND widths 1, 8, 65, 128 before
// observing NAND widths 1 then 8 on the same cache. BigUint prefix construction
// and allocation remain real; native observations must never call its getter.
// reduction-and-reference proves the balanced scalar fold for this full domain.
#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(native_gate_wide_result)]
#[kani::stub(veryl_analyzer::value::biguint_to_u128, reject_logical_biguint_getter)]
fn public_reduction_nand_native_continuation_matches_reference_semantics() {
    let (value, before) = input();
    let natural = before.width;
    let width = natural.max(usize::from(kani::any::<u8>() & 63) + 1);
    let mut cache = MaskCache::default();
    let _ = Op::BitNot.eval_value_unary(&value, natural, false, &mut cache);
    let _ = Op::BitNot.eval_value_unary(&value, natural, true, &mut cache);
    let _ = Op::BitNot.eval_value_unary(&value, width, false, &mut cache);
    let _ = Op::BitNot.eval_value_unary(&value, width, true, &mut cache);
    let _ = Op::BitAnd.eval_value_unary(&value, 1, before.signed, &mut cache);
    let _ = Op::BitAnd.eval_value_unary(&value, 8, before.signed, &mut cache);
    let _ = Op::BitAnd.eval_value_unary(&value, 65, before.signed, &mut cache);
    let _ = Op::BitAnd.eval_value_unary(&value, 128, before.signed, &mut cache);
    let expected = gates::not(native_reduction_and_tree(before));
    let output = Op::BitNand.eval_value_unary(&value, 1, before.signed, &mut cache);
    assert_logical_and(output, 1, expected);
    let output = Op::BitNand.eval_value_unary(&value, 8, before.signed, &mut cache);
    assert_logical_and(output, 8, expected);
    assert_input(&value, before);
    kani::cover!(
        natural == 1
            && !before.signed
            && before.planes == (0, 0)
            && width == 1
            && expected == Logic4::One,
        "public native reduction NAND width 1 zero unchanged is reachable"
    );
    kani::cover!(
        natural == 64
            && before.signed
            && before.planes == (u64::MAX as u128, 0)
            && width == 64
            && expected == Logic4::Zero,
        "public native reduction NAND width 64 signed one is reachable"
    );
    kani::cover!(
        natural == 32
            && !before.signed
            && before.planes == (3, 0xffff_fffe)
            && width == 64
            && expected == Logic4::X,
        "public native reduction NAND widened X and Z is reachable"
    );
    kani::cover!(
        natural == 32
            && before.signed
            && before.planes == (0xffff_fffa, 0xffff_fffc)
            && width == 64
            && expected == Logic4::One,
        "public native reduction NAND widened zero dominates X and Z is reachable"
    );
}

// IEEE 1800 11.4.9/11.8.2: zero extend the independent scalar NAND result.
// Sources span every normalized four-state native width 1..=64 and either sign.
// Preserve the NOT prefix, all four AND results, and NAND results 1/8. The
// width-128 proof also executes NAND width 65 before observing its result.
// Numeric BigUint construction, allocation and both public getters run unchanged.
// Unwind 2 suffices for these one-digit results; safety and unwind checks stay on.
fn public_reduction_nand_wide_continuation<const RESULT: usize>() {
    let (value, before) = input();
    let natural = before.width;
    let width = natural.max(usize::from(kani::any::<u8>() & 63) + 1);
    let mut cache = MaskCache::default();
    let _ = Op::BitNot.eval_value_unary(&value, natural, false, &mut cache);
    let _ = Op::BitNot.eval_value_unary(&value, natural, true, &mut cache);
    let _ = Op::BitNot.eval_value_unary(&value, width, false, &mut cache);
    let _ = Op::BitNot.eval_value_unary(&value, width, true, &mut cache);
    let _ = Op::BitAnd.eval_value_unary(&value, 1, before.signed, &mut cache);
    let _ = Op::BitAnd.eval_value_unary(&value, 8, before.signed, &mut cache);
    let _ = Op::BitAnd.eval_value_unary(&value, 65, before.signed, &mut cache);
    let _ = Op::BitAnd.eval_value_unary(&value, 128, before.signed, &mut cache);
    let expected = gates::not(native_reduction_and_tree(before));
    let output = Op::BitNand.eval_value_unary(&value, 1, before.signed, &mut cache);
    assert_logical_and(output, 1, expected);
    let output = Op::BitNand.eval_value_unary(&value, 8, before.signed, &mut cache);
    assert_logical_and(output, 8, expected);
    if RESULT == 128 {
        let _ = Op::BitNand.eval_value_unary(&value, 65, before.signed, &mut cache);
    }
    let output = Op::BitNand.eval_value_unary(&value, RESULT, before.signed, &mut cache);
    assert_wide_boolean(output, RESULT, expected);
    assert_input(&value, before);
    kani::cover!(
        natural == 1
            && !before.signed
            && before.planes == (0, 0)
            && width == 1
            && expected == Logic4::One,
        "public wide reduction NAND width 1 zero unchanged is reachable"
    );
    kani::cover!(
        natural == 64
            && before.signed
            && before.planes == (u64::MAX as u128, 0)
            && width == 64
            && expected == Logic4::Zero,
        "public wide reduction NAND width 64 signed one is reachable"
    );
    kani::cover!(
        natural == 32
            && !before.signed
            && before.planes == (3, 0xffff_fffe)
            && width == 64
            && expected == Logic4::X,
        "public wide reduction NAND widened X and Z is reachable"
    );
    kani::cover!(
        natural == 32
            && before.signed
            && before.planes == (0xffff_fffa, 0xffff_fffc)
            && width == 64
            && expected == Logic4::One,
        "public wide reduction NAND widened zero dominates X and Z is reachable"
    );
}

#[kani::proof]
#[kani::unwind(2)]
#[kani::use_stub_set(native_gate_wide_result)]
fn public_reduction_nand_width65_matches_reference_semantics() {
    public_reduction_nand_wide_continuation::<65>();
}

#[kani::proof]
#[kani::unwind(2)]
#[kani::use_stub_set(native_gate_wide_result)]
fn public_reduction_nand_width128_matches_reference_semantics() {
    public_reduction_nand_wide_continuation::<128>();
}

// Balanced scalar OR folding avoids increasing the global unwind bound for
// real BigUint loops. Out-of-width leaves use OR's identity, Zero.
fn scalar_or_four(states: [Logic4; 4]) -> Logic4 {
    gates::binary(
        gates::binary(states[0], states[1], BitOp::Or),
        gates::binary(states[2], states[3], BitOp::Or),
        BitOp::Or,
    )
}

fn source_or_four(source: Operand, offset: usize) -> Logic4 {
    let bit = |position| {
        if position < source.width {
            read_logic(source.planes.0, source.planes.1, position)
        } else {
            Logic4::Zero
        }
    };
    scalar_or_four([
        bit(offset),
        bit(offset + 1),
        bit(offset + 2),
        bit(offset + 3),
    ])
}

fn source_or_sixteen(source: Operand, offset: usize) -> Logic4 {
    scalar_or_four([
        source_or_four(source, offset),
        source_or_four(source, offset + 4),
        source_or_four(source, offset + 8),
        source_or_four(source, offset + 12),
    ])
}

fn native_reduction_or_tree(source: Operand) -> Logic4 {
    scalar_or_four([
        source_or_sixteen(source, 0),
        source_or_sixteen(source, 16),
        source_or_sixteen(source, 32),
        source_or_sixteen(source, 48),
    ])
}

#[kani::proof]
#[kani::unwind(65)]
fn native_reduction_or_tree_matches_scalar_fold() {
    let source = Operand {
        planes: (kani::any(), kani::any()),
        width: usize::from(kani::any::<u8>() & 63) + 1,
        signed: kani::any(),
    };
    let expected = gates::reduced_logic(source.planes, source.width, gates::Reduction::Or);
    assert!(
        native_reduction_or_tree(source) == expected,
        "reduction OR reference mismatch"
    );
    kani::cover!(
        source.width == 1,
        "reduction OR reference width 1 is reachable"
    );
    kani::cover!(
        source.width == 64,
        "reduction OR reference width 64 is reachable"
    );
    kani::cover!(
        expected == Logic4::Zero,
        "reduction OR reference zero is reachable"
    );
    kani::cover!(
        expected == Logic4::One,
        "reduction OR reference one is reachable"
    );
    kani::cover!(
        expected == Logic4::X,
        "reduction OR reference X is reachable"
    );
}

// IEEE 1800 11.4.9/11.8.2: scalar OR folding with known one dominating X/Z.
// Sources span all normalized four-state native widths 1..=64 and either sign.
// Execute four native NOT calls, then AND and NAND at widths 1, 8, 65, 128 on
// one cache before observing OR widths 1 then 8. BigUint prefix construction and
// allocation remain real. Native observations must never enter its getter.
// reduction-or-reference separately proves the balanced scalar model.
#[kani::proof]
#[kani::unwind(2)]
#[kani::use_stub_set(native_gate_wide_result)]
#[kani::stub(veryl_analyzer::value::biguint_to_u128, reject_logical_biguint_getter)]
fn public_reduction_or_native_continuation_matches_reference_semantics() {
    let (value, before) = input();
    let natural = before.width;
    let width = natural.max(usize::from(kani::any::<u8>() & 63) + 1);
    let mut cache = MaskCache::default();
    let _ = Op::BitNot.eval_value_unary(&value, natural, false, &mut cache);
    let _ = Op::BitNot.eval_value_unary(&value, natural, true, &mut cache);
    let _ = Op::BitNot.eval_value_unary(&value, width, false, &mut cache);
    let _ = Op::BitNot.eval_value_unary(&value, width, true, &mut cache);
    let _ = Op::BitAnd.eval_value_unary(&value, 1, before.signed, &mut cache);
    let _ = Op::BitAnd.eval_value_unary(&value, 8, before.signed, &mut cache);
    let _ = Op::BitAnd.eval_value_unary(&value, 65, before.signed, &mut cache);
    let _ = Op::BitAnd.eval_value_unary(&value, 128, before.signed, &mut cache);
    let _ = Op::BitNand.eval_value_unary(&value, 1, before.signed, &mut cache);
    let _ = Op::BitNand.eval_value_unary(&value, 8, before.signed, &mut cache);
    let _ = Op::BitNand.eval_value_unary(&value, 65, before.signed, &mut cache);
    let _ = Op::BitNand.eval_value_unary(&value, 128, before.signed, &mut cache);
    let expected = native_reduction_or_tree(before);
    let output = Op::BitOr.eval_value_unary(&value, 1, before.signed, &mut cache);
    assert_logical_and(output, 1, expected);
    let output = Op::BitOr.eval_value_unary(&value, 8, before.signed, &mut cache);
    assert_logical_and(output, 8, expected);
    assert_input(&value, before);
    kani::cover!(
        natural == 1
            && !before.signed
            && before.planes == (0, 0)
            && width == 1
            && expected == Logic4::Zero,
        "public native reduction OR width 1 zero unchanged is reachable"
    );
    kani::cover!(
        natural == 64
            && before.signed
            && before.planes == (1u128 << 63, 0)
            && width == 64
            && expected == Logic4::One,
        "public native reduction OR width 64 signed high one is reachable"
    );
    kani::cover!(
        natural == 32
            && !before.signed
            && before.planes == (2, 0xffff_ffff)
            && width == 64
            && expected == Logic4::X,
        "public native reduction OR widened X and Z is reachable"
    );
    kani::cover!(
        natural == 32
            && before.signed
            && before.planes == (3, 0xffff_fffe)
            && width == 64
            && expected == Logic4::One,
        "public native reduction OR widened one dominates X and Z is reachable"
    );
}

// IEEE 1800 11.4.9/11.8.2: zero extend the independent scalar OR result.
// Sources span all normalized four-state native widths 1..=64 and either sign.
// Preserve four native NOT calls, AND/NAND widths 1, 8, 65, 128, then OR 1/8.
// The width-128 proof also executes OR width 65 before observing its result.
// BigUint numeric construction, allocation and both public getters run unchanged.
// Unwind 2 covers one-digit results, with safety and unwind assertions enabled.
// reduction-or-reference separately proves the balanced scalar OR model.
fn public_reduction_or_wide_continuation<const RESULT: usize>() {
    let (value, before) = input();
    let natural = before.width;
    let width = natural.max(usize::from(kani::any::<u8>() & 63) + 1);
    let mut cache = MaskCache::default();
    let _ = Op::BitNot.eval_value_unary(&value, natural, false, &mut cache);
    let _ = Op::BitNot.eval_value_unary(&value, natural, true, &mut cache);
    let _ = Op::BitNot.eval_value_unary(&value, width, false, &mut cache);
    let _ = Op::BitNot.eval_value_unary(&value, width, true, &mut cache);
    let _ = Op::BitAnd.eval_value_unary(&value, 1, before.signed, &mut cache);
    let _ = Op::BitAnd.eval_value_unary(&value, 8, before.signed, &mut cache);
    let _ = Op::BitAnd.eval_value_unary(&value, 65, before.signed, &mut cache);
    let _ = Op::BitAnd.eval_value_unary(&value, 128, before.signed, &mut cache);
    let _ = Op::BitNand.eval_value_unary(&value, 1, before.signed, &mut cache);
    let _ = Op::BitNand.eval_value_unary(&value, 8, before.signed, &mut cache);
    let _ = Op::BitNand.eval_value_unary(&value, 65, before.signed, &mut cache);
    let _ = Op::BitNand.eval_value_unary(&value, 128, before.signed, &mut cache);
    let expected = native_reduction_or_tree(before);
    let output = Op::BitOr.eval_value_unary(&value, 1, before.signed, &mut cache);
    assert_logical_and(output, 1, expected);
    let output = Op::BitOr.eval_value_unary(&value, 8, before.signed, &mut cache);
    assert_logical_and(output, 8, expected);
    if RESULT == 128 {
        let _ = Op::BitOr.eval_value_unary(&value, 65, before.signed, &mut cache);
    }
    let output = Op::BitOr.eval_value_unary(&value, RESULT, before.signed, &mut cache);
    assert_wide_boolean(output, RESULT, expected);
    assert_input(&value, before);
    kani::cover!(
        natural == 1
            && !before.signed
            && before.planes == (0, 0)
            && width == 1
            && expected == Logic4::Zero,
        "public wide reduction OR width 1 zero unchanged is reachable"
    );
    kani::cover!(
        natural == 64
            && before.signed
            && before.planes == (1u128 << 63, 0)
            && width == 64
            && expected == Logic4::One,
        "public wide reduction OR width 64 signed high one is reachable"
    );
    kani::cover!(
        natural == 32
            && !before.signed
            && before.planes == (2, 0xffff_ffff)
            && width == 64
            && expected == Logic4::X,
        "public wide reduction OR widened X and Z is reachable"
    );
    kani::cover!(
        natural == 32
            && before.signed
            && before.planes == (3, 0xffff_fffe)
            && width == 64
            && expected == Logic4::One,
        "public wide reduction OR widened one dominates X and Z is reachable"
    );
}

#[kani::proof]
#[kani::unwind(2)]
#[kani::use_stub_set(native_gate_wide_result)]
fn public_reduction_or_width65_matches_reference_semantics() {
    public_reduction_or_wide_continuation::<65>();
}

#[kani::proof]
#[kani::unwind(2)]
#[kani::use_stub_set(native_gate_wide_result)]
fn public_reduction_or_width128_matches_reference_semantics() {
    public_reduction_or_wide_continuation::<128>();
}

// IEEE 1800 11.4.9/11.8.2: scalar NOR negates OR folding and zero extends.
// Sources span every normalized four-state native width 1..=64 and either sign.
// Execute four NOT calls at natural and arbitrary native context natural..=64,
// with both caller signs, then AND/NAND/OR widths 1, 8, 65, 128 on one cache.
// Observe NOR widths 1 then 8. BigUint prefix construction and allocation remain
// real; native observations must never enter its getter. Unwind/safety checks
// remain enabled. reduction-or-reference separately proves the scalar OR tree.
#[kani::proof]
#[kani::unwind(2)]
#[kani::use_stub_set(native_gate_wide_result)]
#[kani::stub(veryl_analyzer::value::biguint_to_u128, reject_logical_biguint_getter)]
fn public_reduction_nor_native_continuation_matches_reference_semantics() {
    let (value, before) = input();
    let natural = before.width;
    let width = natural.max(usize::from(kani::any::<u8>() & 63) + 1);
    let mut cache = MaskCache::default();
    let _ = Op::BitNot.eval_value_unary(&value, natural, false, &mut cache);
    let _ = Op::BitNot.eval_value_unary(&value, natural, true, &mut cache);
    let _ = Op::BitNot.eval_value_unary(&value, width, false, &mut cache);
    let _ = Op::BitNot.eval_value_unary(&value, width, true, &mut cache);
    let _ = Op::BitAnd.eval_value_unary(&value, 1, before.signed, &mut cache);
    let _ = Op::BitAnd.eval_value_unary(&value, 8, before.signed, &mut cache);
    let _ = Op::BitAnd.eval_value_unary(&value, 65, before.signed, &mut cache);
    let _ = Op::BitAnd.eval_value_unary(&value, 128, before.signed, &mut cache);
    let _ = Op::BitNand.eval_value_unary(&value, 1, before.signed, &mut cache);
    let _ = Op::BitNand.eval_value_unary(&value, 8, before.signed, &mut cache);
    let _ = Op::BitNand.eval_value_unary(&value, 65, before.signed, &mut cache);
    let _ = Op::BitNand.eval_value_unary(&value, 128, before.signed, &mut cache);
    let _ = Op::BitOr.eval_value_unary(&value, 1, before.signed, &mut cache);
    let _ = Op::BitOr.eval_value_unary(&value, 8, before.signed, &mut cache);
    let _ = Op::BitOr.eval_value_unary(&value, 65, before.signed, &mut cache);
    let _ = Op::BitOr.eval_value_unary(&value, 128, before.signed, &mut cache);
    let expected = gates::not(native_reduction_or_tree(before));
    let output = Op::BitNor.eval_value_unary(&value, 1, before.signed, &mut cache);
    assert_logical_and(output, 1, expected);
    let output = Op::BitNor.eval_value_unary(&value, 8, before.signed, &mut cache);
    assert_logical_and(output, 8, expected);
    assert_input(&value, before);
    kani::cover!(
        natural == 1
            && !before.signed
            && before.planes == (0, 0)
            && width == 1
            && expected == Logic4::One,
        "public native reduction NOR width 1 zero unchanged is reachable"
    );
    kani::cover!(
        natural == 64
            && before.signed
            && before.planes == (1u128 << 63, 0)
            && width == 64
            && expected == Logic4::Zero,
        "public native reduction NOR width 64 signed high one is reachable"
    );
    kani::cover!(
        natural == 32
            && !before.signed
            && before.planes == (2, 0xffff_ffff)
            && width == 64
            && expected == Logic4::X,
        "public native reduction NOR widened X and Z is reachable"
    );
    kani::cover!(
        natural == 32
            && before.signed
            && before.planes == (3, 0xffff_fffe)
            && width == 64
            && expected == Logic4::Zero,
        "public native reduction NOR widened one dominates X and Z is reachable"
    );
}

// Complete numeric plane equality without collecting digits in this producer
// proof: bit length <= 1 and bit 0 uniquely determine each entire plane.
// public-big-getters-128 separately proves the public u128 getters for every
// canonical BigUint plane bounded by 128 bits, at all metadata widths 65..=128
// and either sign. These results satisfy that domain, so the two proofs compose.
fn assert_wide_boolean_planes(output: Value, width: usize, expected: Logic4) {
    let planes = match expected {
        Logic4::Zero => (false, false),
        Logic4::One => (true, false),
        Logic4::X => (false, true),
        Logic4::Z => unreachable!(),
    };
    let Value::BigUint(big) = output else {
        panic!("public gate semantics mismatch");
    };
    assert!(
        big.payload.bits() <= 1
            && big.mask_xz.bits() <= 1
            && big.payload.bit(0) == planes.0
            && big.mask_xz.bit(0) == planes.1,
        "public gate semantics mismatch"
    );
    let output = Value::BigUint(big);
    assert!(
        output.width() == width && !output.signed(),
        "public gate semantics mismatch"
    );
}

// IEEE 1800 11.4.9/11.8.2: zero extend the independent scalar NOR result.
// Sources span all normalized four-state native widths 1..=64 and either sign.
// The second NOT context independently spans natural..=64; both caller signs run
// at both contexts. Preserve AND/NAND/OR widths 1, 8, 65, 128, then NOR 1/8.
// The width-128 proof also executes NOR width 65 before observing its result.
// BigUint numeric construction and allocation run unchanged. Exact numeric
// planes compose with public-big-getters-128; native prefix/input observations
// must never enter the BigUint getter conversion.
// Unwind 2 covers one-digit results, with safety and unwind assertions enabled.
// reduction-or-reference separately proves the balanced scalar OR model.
fn public_reduction_nor_wide_continuation<const RESULT: usize>() {
    let (value, before) = input();
    let natural = before.width;
    let width = natural.max(usize::from(kani::any::<u8>() & 63) + 1);
    let mut cache = MaskCache::default();
    let _ = Op::BitNot.eval_value_unary(&value, natural, false, &mut cache);
    let _ = Op::BitNot.eval_value_unary(&value, natural, true, &mut cache);
    let _ = Op::BitNot.eval_value_unary(&value, width, false, &mut cache);
    let _ = Op::BitNot.eval_value_unary(&value, width, true, &mut cache);
    let _ = Op::BitAnd.eval_value_unary(&value, 1, before.signed, &mut cache);
    let _ = Op::BitAnd.eval_value_unary(&value, 8, before.signed, &mut cache);
    let _ = Op::BitAnd.eval_value_unary(&value, 65, before.signed, &mut cache);
    let _ = Op::BitAnd.eval_value_unary(&value, 128, before.signed, &mut cache);
    let _ = Op::BitNand.eval_value_unary(&value, 1, before.signed, &mut cache);
    let _ = Op::BitNand.eval_value_unary(&value, 8, before.signed, &mut cache);
    let _ = Op::BitNand.eval_value_unary(&value, 65, before.signed, &mut cache);
    let _ = Op::BitNand.eval_value_unary(&value, 128, before.signed, &mut cache);
    let _ = Op::BitOr.eval_value_unary(&value, 1, before.signed, &mut cache);
    let _ = Op::BitOr.eval_value_unary(&value, 8, before.signed, &mut cache);
    let _ = Op::BitOr.eval_value_unary(&value, 65, before.signed, &mut cache);
    let _ = Op::BitOr.eval_value_unary(&value, 128, before.signed, &mut cache);
    let expected = gates::not(native_reduction_or_tree(before));
    let output = Op::BitNor.eval_value_unary(&value, 1, before.signed, &mut cache);
    assert_logical_and(output, 1, expected);
    let output = Op::BitNor.eval_value_unary(&value, 8, before.signed, &mut cache);
    assert_logical_and(output, 8, expected);
    if RESULT == 128 {
        let _ = Op::BitNor.eval_value_unary(&value, 65, before.signed, &mut cache);
    }
    let output = Op::BitNor.eval_value_unary(&value, RESULT, before.signed, &mut cache);
    assert_wide_boolean_planes(output, RESULT, expected);
    assert_input(&value, before);
    kani::cover!(
        natural == 1
            && !before.signed
            && before.planes == (0, 0)
            && width == 1
            && expected == Logic4::One,
        "public wide reduction NOR width 1 zero unchanged is reachable"
    );
    kani::cover!(
        natural == 64
            && before.signed
            && before.planes == (1u128 << 63, 0)
            && width == 64
            && expected == Logic4::Zero,
        "public wide reduction NOR width 64 signed high one is reachable"
    );
    kani::cover!(
        natural == 32
            && !before.signed
            && before.planes == (2, 0xffff_ffff)
            && width == 64
            && expected == Logic4::X,
        "public wide reduction NOR widened X and Z is reachable"
    );
    kani::cover!(
        natural == 32
            && before.signed
            && before.planes == (3, 0xffff_fffe)
            && width == 64
            && expected == Logic4::Zero,
        "public wide reduction NOR widened one dominates X and Z is reachable"
    );
}

#[kani::proof]
#[kani::unwind(2)]
#[kani::use_stub_set(native_gate_wide_result)]
#[kani::stub(veryl_analyzer::value::biguint_to_u128, reject_logical_biguint_getter)]
fn public_reduction_nor_width65_matches_reference_semantics() {
    public_reduction_nor_wide_continuation::<65>();
}

#[kani::proof]
#[kani::unwind(2)]
#[kani::use_stub_set(native_gate_wide_result)]
#[kani::stub(veryl_analyzer::value::biguint_to_u128, reject_logical_biguint_getter)]
fn public_reduction_nor_width128_matches_reference_semantics() {
    public_reduction_nor_wide_continuation::<128>();
}

// Balanced scalar XOR folding avoids increasing the global unwind bound for
// real BigUint loops. Out-of-width leaves use XOR's identity, Zero.
fn scalar_xor_four(states: [Logic4; 4]) -> Logic4 {
    gates::binary(
        gates::binary(states[0], states[1], BitOp::Xor),
        gates::binary(states[2], states[3], BitOp::Xor),
        BitOp::Xor,
    )
}

fn source_xor_four(source: Operand, offset: usize) -> Logic4 {
    let bit = |position| {
        if position < source.width {
            read_logic(source.planes.0, source.planes.1, position)
        } else {
            Logic4::Zero
        }
    };
    scalar_xor_four([
        bit(offset),
        bit(offset + 1),
        bit(offset + 2),
        bit(offset + 3),
    ])
}

fn source_xor_sixteen(source: Operand, offset: usize) -> Logic4 {
    scalar_xor_four([
        source_xor_four(source, offset),
        source_xor_four(source, offset + 4),
        source_xor_four(source, offset + 8),
        source_xor_four(source, offset + 12),
    ])
}

fn native_reduction_xor_tree(source: Operand) -> Logic4 {
    scalar_xor_four([
        source_xor_sixteen(source, 0),
        source_xor_sixteen(source, 16),
        source_xor_sixteen(source, 32),
        source_xor_sixteen(source, 48),
    ])
}

#[kani::proof]
#[kani::unwind(65)]
fn native_reduction_xor_tree_matches_scalar_fold() {
    let source = Operand {
        planes: (kani::any(), kani::any()),
        width: usize::from(kani::any::<u8>() & 63) + 1,
        signed: kani::any(),
    };
    let expected = gates::reduced_logic(source.planes, source.width, gates::Reduction::Xor);
    assert!(
        native_reduction_xor_tree(source) == expected,
        "reduction XOR reference mismatch"
    );
    kani::cover!(
        source.width == 1,
        "reduction XOR reference width 1 is reachable"
    );
    kani::cover!(
        source.width == 64,
        "reduction XOR reference width 64 is reachable"
    );
    kani::cover!(
        expected == Logic4::Zero,
        "reduction XOR reference zero is reachable"
    );
    kani::cover!(
        expected == Logic4::One,
        "reduction XOR reference one is reachable"
    );
    kani::cover!(
        expected == Logic4::X,
        "reduction XOR reference X is reachable"
    );
}

// IEEE 1800 11.4.9/11.8.2: scalar XOR folding, zero extension and unsigned result.
// Sources span all normalized four-state native widths 1..=64 and either sign.
// Execute four NOT calls at natural and independent native context natural..=64
// with both caller signs, then AND/NAND/OR/NOR widths 1, 8, 65, 128 on one cache.
// Observe XOR widths 1 then 8. BigUint prefix construction and allocation stay
// real; native observations must never enter its getter. Safety and unwind
// assertions remain enabled. reduction-xor-reference proves the scalar model.
#[kani::proof]
#[kani::unwind(2)]
#[kani::use_stub_set(native_gate_wide_result)]
#[kani::stub(veryl_analyzer::value::biguint_to_u128, reject_logical_biguint_getter)]
fn public_reduction_xor_native_continuation_matches_reference_semantics() {
    let (value, before) = input();
    let natural = before.width;
    let width = natural.max(usize::from(kani::any::<u8>() & 63) + 1);
    let mut cache = MaskCache::default();
    let _ = Op::BitNot.eval_value_unary(&value, natural, false, &mut cache);
    let _ = Op::BitNot.eval_value_unary(&value, natural, true, &mut cache);
    let _ = Op::BitNot.eval_value_unary(&value, width, false, &mut cache);
    let _ = Op::BitNot.eval_value_unary(&value, width, true, &mut cache);
    let _ = Op::BitAnd.eval_value_unary(&value, 1, before.signed, &mut cache);
    let _ = Op::BitAnd.eval_value_unary(&value, 8, before.signed, &mut cache);
    let _ = Op::BitAnd.eval_value_unary(&value, 65, before.signed, &mut cache);
    let _ = Op::BitAnd.eval_value_unary(&value, 128, before.signed, &mut cache);
    let _ = Op::BitNand.eval_value_unary(&value, 1, before.signed, &mut cache);
    let _ = Op::BitNand.eval_value_unary(&value, 8, before.signed, &mut cache);
    let _ = Op::BitNand.eval_value_unary(&value, 65, before.signed, &mut cache);
    let _ = Op::BitNand.eval_value_unary(&value, 128, before.signed, &mut cache);
    let _ = Op::BitOr.eval_value_unary(&value, 1, before.signed, &mut cache);
    let _ = Op::BitOr.eval_value_unary(&value, 8, before.signed, &mut cache);
    let _ = Op::BitOr.eval_value_unary(&value, 65, before.signed, &mut cache);
    let _ = Op::BitOr.eval_value_unary(&value, 128, before.signed, &mut cache);
    let _ = Op::BitNor.eval_value_unary(&value, 1, before.signed, &mut cache);
    let _ = Op::BitNor.eval_value_unary(&value, 8, before.signed, &mut cache);
    let _ = Op::BitNor.eval_value_unary(&value, 65, before.signed, &mut cache);
    let _ = Op::BitNor.eval_value_unary(&value, 128, before.signed, &mut cache);
    let expected = native_reduction_xor_tree(before);
    let output = Op::BitXor.eval_value_unary(&value, 1, before.signed, &mut cache);
    assert_logical_and(output, 1, expected);
    let output = Op::BitXor.eval_value_unary(&value, 8, before.signed, &mut cache);
    assert_logical_and(output, 8, expected);
    assert_input(&value, before);
    kani::cover!(
        natural == 1
            && !before.signed
            && before.planes == (0, 0)
            && width == 1
            && expected == Logic4::Zero,
        "public native reduction XOR width 1 zero unchanged is reachable"
    );
    kani::cover!(
        natural == 64
            && before.signed
            && before.planes == (1u128 << 63, 0)
            && width == 64
            && expected == Logic4::One,
        "public native reduction XOR width 64 signed odd parity is reachable"
    );
    kani::cover!(
        natural == 64
            && !before.signed
            && before.planes == ((1u128 << 63) | 1, 0)
            && width == 64
            && expected == Logic4::Zero,
        "public native reduction XOR width 64 even parity is reachable"
    );
    kani::cover!(
        natural == 32
            && !before.signed
            && before.planes == (2, 0xffff_ffff)
            && width == 64
            && expected == Logic4::X,
        "public native reduction XOR widened X and Z is reachable"
    );
    kani::cover!(
        natural == 32
            && before.signed
            && before.planes == (3, 0xffff_fffe)
            && width == 64
            && expected == Logic4::X,
        "public native reduction XOR widened unknown dominates one is reachable"
    );
}

// IEEE 1800 11.4.9/11.8.2: zero extend the independent scalar XOR result.
// Sources span all normalized four-state native widths 1..=64 and either sign.
// The second NOT context independently spans natural..=64; both caller signs run
// at both contexts. Preserve AND/NAND/OR/NOR widths 1, 8, 65, 128, then XOR 1/8.
// The width-128 proof also executes XOR width 65 before observing its result.
// Real numeric construction/allocation and exact raw planes compose with
// public-big-getters-128, including its u64 constructor domain. Native prefix
// observations must never enter BigUint getter conversion. Unwind 2 keeps all
// safety/unwind assertions enabled. reduction-xor-reference proves the model.
fn public_reduction_xor_wide_continuation<const RESULT: usize>() {
    let (value, before) = input();
    let natural = before.width;
    let width = natural.max(usize::from(kani::any::<u8>() & 63) + 1);
    let mut cache = MaskCache::default();
    let _ = Op::BitNot.eval_value_unary(&value, natural, false, &mut cache);
    let _ = Op::BitNot.eval_value_unary(&value, natural, true, &mut cache);
    let _ = Op::BitNot.eval_value_unary(&value, width, false, &mut cache);
    let _ = Op::BitNot.eval_value_unary(&value, width, true, &mut cache);
    let _ = Op::BitAnd.eval_value_unary(&value, 1, before.signed, &mut cache);
    let _ = Op::BitAnd.eval_value_unary(&value, 8, before.signed, &mut cache);
    let _ = Op::BitAnd.eval_value_unary(&value, 65, before.signed, &mut cache);
    let _ = Op::BitAnd.eval_value_unary(&value, 128, before.signed, &mut cache);
    let _ = Op::BitNand.eval_value_unary(&value, 1, before.signed, &mut cache);
    let _ = Op::BitNand.eval_value_unary(&value, 8, before.signed, &mut cache);
    let _ = Op::BitNand.eval_value_unary(&value, 65, before.signed, &mut cache);
    let _ = Op::BitNand.eval_value_unary(&value, 128, before.signed, &mut cache);
    let _ = Op::BitOr.eval_value_unary(&value, 1, before.signed, &mut cache);
    let _ = Op::BitOr.eval_value_unary(&value, 8, before.signed, &mut cache);
    let _ = Op::BitOr.eval_value_unary(&value, 65, before.signed, &mut cache);
    let _ = Op::BitOr.eval_value_unary(&value, 128, before.signed, &mut cache);
    let _ = Op::BitNor.eval_value_unary(&value, 1, before.signed, &mut cache);
    let _ = Op::BitNor.eval_value_unary(&value, 8, before.signed, &mut cache);
    let _ = Op::BitNor.eval_value_unary(&value, 65, before.signed, &mut cache);
    let _ = Op::BitNor.eval_value_unary(&value, 128, before.signed, &mut cache);
    let expected = native_reduction_xor_tree(before);
    let output = Op::BitXor.eval_value_unary(&value, 1, before.signed, &mut cache);
    assert_logical_and(output, 1, expected);
    let output = Op::BitXor.eval_value_unary(&value, 8, before.signed, &mut cache);
    assert_logical_and(output, 8, expected);
    if RESULT == 128 {
        let _ = Op::BitXor.eval_value_unary(&value, 65, before.signed, &mut cache);
    }
    let output = Op::BitXor.eval_value_unary(&value, RESULT, before.signed, &mut cache);
    assert_wide_boolean_planes(output, RESULT, expected);
    assert_input(&value, before);
    kani::cover!(
        natural == 1
            && !before.signed
            && before.planes == (0, 0)
            && width == 1
            && expected == Logic4::Zero,
        "public wide reduction XOR width 1 zero unchanged is reachable"
    );
    kani::cover!(
        natural == 64
            && before.signed
            && before.planes == (1u128 << 63, 0)
            && width == 64
            && expected == Logic4::One,
        "public wide reduction XOR width 64 signed odd parity is reachable"
    );
    kani::cover!(
        natural == 64
            && !before.signed
            && before.planes == ((1u128 << 63) | 1, 0)
            && width == 64
            && expected == Logic4::Zero,
        "public wide reduction XOR width 64 even parity is reachable"
    );
    kani::cover!(
        natural == 32
            && !before.signed
            && before.planes == (2, 0xffff_ffff)
            && width == 64
            && expected == Logic4::X,
        "public wide reduction XOR widened X and Z is reachable"
    );
    kani::cover!(
        natural == 32
            && before.signed
            && before.planes == (3, 0xffff_fffe)
            && width == 64
            && expected == Logic4::X,
        "public wide reduction XOR widened unknown dominates one is reachable"
    );
}

#[kani::proof]
#[kani::unwind(2)]
#[kani::use_stub_set(native_gate_wide_result)]
#[kani::stub(veryl_analyzer::value::biguint_to_u128, reject_logical_biguint_getter)]
fn public_reduction_xor_width65_matches_reference_semantics() {
    public_reduction_xor_wide_continuation::<65>();
}

#[kani::proof]
#[kani::unwind(2)]
#[kani::use_stub_set(native_gate_wide_result)]
#[kani::stub(veryl_analyzer::value::biguint_to_u128, reject_logical_biguint_getter)]
fn public_reduction_xor_width128_matches_reference_semantics() {
    public_reduction_xor_wide_continuation::<128>();
}

// IEEE 1800 11.4.9/11.8.2: XNOR negates the scalar XOR fold and zero extends.
// Sources span all normalized four-state native widths 1..=64 and either sign.
// Execute four NOT calls at natural and independent native context natural..=64
// with both caller signs, then AND/NAND/OR/NOR/XOR widths 1, 8, 65, 128 on one
// cache. Observe XNOR widths 1 then 8. BigUint prefix construction/allocation stay
// real; native observations must never enter its getter. Safety and unwind
// assertions remain enabled. reduction-xor-reference proves the scalar model.
#[kani::proof]
#[kani::unwind(2)]
#[kani::use_stub_set(native_gate_wide_result)]
#[kani::stub(veryl_analyzer::value::biguint_to_u128, reject_logical_biguint_getter)]
fn public_reduction_xnor_native_continuation_matches_reference_semantics() {
    let (value, before) = input();
    let natural = before.width;
    let width = natural.max(usize::from(kani::any::<u8>() & 63) + 1);
    let mut cache = MaskCache::default();
    let _ = Op::BitNot.eval_value_unary(&value, natural, false, &mut cache);
    let _ = Op::BitNot.eval_value_unary(&value, natural, true, &mut cache);
    let _ = Op::BitNot.eval_value_unary(&value, width, false, &mut cache);
    let _ = Op::BitNot.eval_value_unary(&value, width, true, &mut cache);
    let _ = Op::BitAnd.eval_value_unary(&value, 1, before.signed, &mut cache);
    let _ = Op::BitAnd.eval_value_unary(&value, 8, before.signed, &mut cache);
    let _ = Op::BitAnd.eval_value_unary(&value, 65, before.signed, &mut cache);
    let _ = Op::BitAnd.eval_value_unary(&value, 128, before.signed, &mut cache);
    let _ = Op::BitNand.eval_value_unary(&value, 1, before.signed, &mut cache);
    let _ = Op::BitNand.eval_value_unary(&value, 8, before.signed, &mut cache);
    let _ = Op::BitNand.eval_value_unary(&value, 65, before.signed, &mut cache);
    let _ = Op::BitNand.eval_value_unary(&value, 128, before.signed, &mut cache);
    let _ = Op::BitOr.eval_value_unary(&value, 1, before.signed, &mut cache);
    let _ = Op::BitOr.eval_value_unary(&value, 8, before.signed, &mut cache);
    let _ = Op::BitOr.eval_value_unary(&value, 65, before.signed, &mut cache);
    let _ = Op::BitOr.eval_value_unary(&value, 128, before.signed, &mut cache);
    let _ = Op::BitNor.eval_value_unary(&value, 1, before.signed, &mut cache);
    let _ = Op::BitNor.eval_value_unary(&value, 8, before.signed, &mut cache);
    let _ = Op::BitNor.eval_value_unary(&value, 65, before.signed, &mut cache);
    let _ = Op::BitNor.eval_value_unary(&value, 128, before.signed, &mut cache);
    let _ = Op::BitXor.eval_value_unary(&value, 1, before.signed, &mut cache);
    let _ = Op::BitXor.eval_value_unary(&value, 8, before.signed, &mut cache);
    let _ = Op::BitXor.eval_value_unary(&value, 65, before.signed, &mut cache);
    let _ = Op::BitXor.eval_value_unary(&value, 128, before.signed, &mut cache);
    let expected = gates::not(native_reduction_xor_tree(before));
    let output = Op::BitXnor.eval_value_unary(&value, 1, before.signed, &mut cache);
    assert_logical_and(output, 1, expected);
    let output = Op::BitXnor.eval_value_unary(&value, 8, before.signed, &mut cache);
    assert_logical_and(output, 8, expected);
    assert_input(&value, before);
    kani::cover!(
        natural == 1
            && !before.signed
            && before.planes == (0, 0)
            && width == 1
            && expected == Logic4::One,
        "public native reduction XNOR width 1 zero unchanged is reachable"
    );
    kani::cover!(
        natural == 64
            && before.signed
            && before.planes == (1u128 << 63, 0)
            && width == 64
            && expected == Logic4::Zero,
        "public native reduction XNOR width 64 signed odd parity is reachable"
    );
    kani::cover!(
        natural == 64
            && !before.signed
            && before.planes == ((1u128 << 63) | 1, 0)
            && width == 64
            && expected == Logic4::One,
        "public native reduction XNOR width 64 even parity is reachable"
    );
    kani::cover!(
        natural == 32
            && !before.signed
            && before.planes == (2, 0xffff_ffff)
            && width == 64
            && expected == Logic4::X,
        "public native reduction XNOR widened X and Z is reachable"
    );
    kani::cover!(
        natural == 32
            && before.signed
            && before.planes == (3, 0xffff_fffe)
            && width == 64
            && expected == Logic4::X,
        "public native reduction XNOR widened unknown dominates one is reachable"
    );
}

// IEEE 1800 11.4.9/11.8.2: zero extend the independent scalar XNOR result.
// Sources span all normalized four-state native widths 1..=64 and either sign.
// The second NOT context independently spans natural..=64; both caller signs run
// at both contexts. Preserve AND/NAND/OR/NOR/XOR widths 1, 8, 65, 128, then XNOR 1/8.
// The width-128 proof also executes XNOR width 65 before observing its result.
// Real numeric construction/allocation and exact raw planes compose with
// public-big-getters-128, including its u64 constructor domain. Native prefix
// observations must never enter BigUint getter conversion. Unwind 2 keeps all
// safety/unwind assertions enabled. reduction-xor-reference proves the model.
fn public_reduction_xnor_wide_continuation<const RESULT: usize>() {
    let (value, before) = input();
    let natural = before.width;
    let width = natural.max(usize::from(kani::any::<u8>() & 63) + 1);
    let mut cache = MaskCache::default();
    let _ = Op::BitNot.eval_value_unary(&value, natural, false, &mut cache);
    let _ = Op::BitNot.eval_value_unary(&value, natural, true, &mut cache);
    let _ = Op::BitNot.eval_value_unary(&value, width, false, &mut cache);
    let _ = Op::BitNot.eval_value_unary(&value, width, true, &mut cache);
    let _ = Op::BitAnd.eval_value_unary(&value, 1, before.signed, &mut cache);
    let _ = Op::BitAnd.eval_value_unary(&value, 8, before.signed, &mut cache);
    let _ = Op::BitAnd.eval_value_unary(&value, 65, before.signed, &mut cache);
    let _ = Op::BitAnd.eval_value_unary(&value, 128, before.signed, &mut cache);
    let _ = Op::BitNand.eval_value_unary(&value, 1, before.signed, &mut cache);
    let _ = Op::BitNand.eval_value_unary(&value, 8, before.signed, &mut cache);
    let _ = Op::BitNand.eval_value_unary(&value, 65, before.signed, &mut cache);
    let _ = Op::BitNand.eval_value_unary(&value, 128, before.signed, &mut cache);
    let _ = Op::BitOr.eval_value_unary(&value, 1, before.signed, &mut cache);
    let _ = Op::BitOr.eval_value_unary(&value, 8, before.signed, &mut cache);
    let _ = Op::BitOr.eval_value_unary(&value, 65, before.signed, &mut cache);
    let _ = Op::BitOr.eval_value_unary(&value, 128, before.signed, &mut cache);
    let _ = Op::BitNor.eval_value_unary(&value, 1, before.signed, &mut cache);
    let _ = Op::BitNor.eval_value_unary(&value, 8, before.signed, &mut cache);
    let _ = Op::BitNor.eval_value_unary(&value, 65, before.signed, &mut cache);
    let _ = Op::BitNor.eval_value_unary(&value, 128, before.signed, &mut cache);
    let _ = Op::BitXor.eval_value_unary(&value, 1, before.signed, &mut cache);
    let _ = Op::BitXor.eval_value_unary(&value, 8, before.signed, &mut cache);
    let _ = Op::BitXor.eval_value_unary(&value, 65, before.signed, &mut cache);
    let _ = Op::BitXor.eval_value_unary(&value, 128, before.signed, &mut cache);
    let expected = gates::not(native_reduction_xor_tree(before));
    let output = Op::BitXnor.eval_value_unary(&value, 1, before.signed, &mut cache);
    assert_logical_and(output, 1, expected);
    let output = Op::BitXnor.eval_value_unary(&value, 8, before.signed, &mut cache);
    assert_logical_and(output, 8, expected);
    if RESULT == 128 {
        let _ = Op::BitXnor.eval_value_unary(&value, 65, before.signed, &mut cache);
    }
    let output = Op::BitXnor.eval_value_unary(&value, RESULT, before.signed, &mut cache);
    assert_wide_boolean_planes(output, RESULT, expected);
    assert_input(&value, before);
    kani::cover!(
        natural == 1
            && !before.signed
            && before.planes == (0, 0)
            && width == 1
            && expected == Logic4::One,
        "public wide reduction XNOR width 1 zero unchanged is reachable"
    );
    kani::cover!(
        natural == 64
            && before.signed
            && before.planes == (1u128 << 63, 0)
            && width == 64
            && expected == Logic4::Zero,
        "public wide reduction XNOR width 64 signed odd parity is reachable"
    );
    kani::cover!(
        natural == 64
            && !before.signed
            && before.planes == ((1u128 << 63) | 1, 0)
            && width == 64
            && expected == Logic4::One,
        "public wide reduction XNOR width 64 even parity is reachable"
    );
    kani::cover!(
        natural == 32
            && !before.signed
            && before.planes == (2, 0xffff_ffff)
            && width == 64
            && expected == Logic4::X,
        "public wide reduction XNOR widened X and Z is reachable"
    );
    kani::cover!(
        natural == 32
            && before.signed
            && before.planes == (3, 0xffff_fffe)
            && width == 64
            && expected == Logic4::X,
        "public wide reduction XNOR widened unknown dominates one is reachable"
    );
}

#[kani::proof]
#[kani::unwind(2)]
#[kani::use_stub_set(native_gate_wide_result)]
#[kani::stub(veryl_analyzer::value::biguint_to_u128, reject_logical_biguint_getter)]
fn public_reduction_xnor_width65_matches_reference_semantics() {
    public_reduction_xnor_wide_continuation::<65>();
}

#[kani::proof]
#[kani::unwind(2)]
#[kani::use_stub_set(native_gate_wide_result)]
#[kani::stub(veryl_analyzer::value::biguint_to_u128, reject_logical_biguint_getter)]
fn public_reduction_xnor_width128_matches_reference_semantics() {
    public_reduction_xnor_wide_continuation::<128>();
}

// Per-bit classification for normalized native planes. Unlike aggregate plane
// arithmetic in the compiler, each bit uses the independent scalar state table.
// Unrolling avoids adding width-dependent oracle branches to the heap trace.
fn unrolled_native_logical_truth(source: Operand) -> Logic4 {
    assert!(
        (1..=64).contains(&source.width)
            && (source.planes.0 | source.planes.1) >> source.width == 0,
        "logical unrolled truth reference mismatch"
    );
    let mut saw_one = false;
    let mut saw_unknown = false;
    macro_rules! bit {
        ($position:expr) => {{
            let state = read_logic(source.planes.0, source.planes.1, $position);
            saw_one |= state == Logic4::One;
            saw_unknown |= matches!(state, Logic4::X | Logic4::Z);
        }};
    }
    bit!(0);
    bit!(1);
    bit!(2);
    bit!(3);
    bit!(4);
    bit!(5);
    bit!(6);
    bit!(7);
    bit!(8);
    bit!(9);
    bit!(10);
    bit!(11);
    bit!(12);
    bit!(13);
    bit!(14);
    bit!(15);
    bit!(16);
    bit!(17);
    bit!(18);
    bit!(19);
    bit!(20);
    bit!(21);
    bit!(22);
    bit!(23);
    bit!(24);
    bit!(25);
    bit!(26);
    bit!(27);
    bit!(28);
    bit!(29);
    bit!(30);
    bit!(31);
    bit!(32);
    bit!(33);
    bit!(34);
    bit!(35);
    bit!(36);
    bit!(37);
    bit!(38);
    bit!(39);
    bit!(40);
    bit!(41);
    bit!(42);
    bit!(43);
    bit!(44);
    bit!(45);
    bit!(46);
    bit!(47);
    bit!(48);
    bit!(49);
    bit!(50);
    bit!(51);
    bit!(52);
    bit!(53);
    bit!(54);
    bit!(55);
    bit!(56);
    bit!(57);
    bit!(58);
    bit!(59);
    bit!(60);
    bit!(61);
    bit!(62);
    bit!(63);
    if saw_one {
        Logic4::One
    } else if saw_unknown {
        Logic4::X
    } else {
        Logic4::Zero
    }
}

#[kani::proof]
#[kani::unwind(65)]
fn unrolled_native_logical_truth_matches_reference_semantics() {
    let width = usize::from(kani::any::<u8>() & 63) + 1;
    let mask = u64::MAX >> (64 - width);
    let source = Operand {
        planes: (
            u128::from(kani::any::<u64>() & mask),
            u128::from(kani::any::<u64>() & mask),
        ),
        width,
        signed: kani::any(),
    };
    let expected = gates::logical_truth(source.planes, width);
    assert!(
        unrolled_native_logical_truth(source) == expected,
        "logical unrolled truth reference mismatch"
    );
    kani::cover!(width == 1, "logical unrolled truth width 1 is reachable");
    kani::cover!(width == 64, "logical unrolled truth width 64 is reachable");
    kani::cover!(
        expected == Logic4::Zero,
        "logical unrolled truth zero is reachable"
    );
    kani::cover!(
        expected == Logic4::One,
        "logical unrolled truth one is reachable"
    );
    kani::cover!(
        expected == Logic4::X,
        "logical unrolled truth X is reachable"
    );
}

// IEEE 1800 11.4.7/11.8.2. Native operands cover every normalized four-state
// value at widths 1..=64 and either sign. The NOT caller runs both signs at the
// natural width and an independent native context natural..=64. Every subsequent
// reduction runs at 1, 8, 65, 128 on the same cache before logical NOT.
// gates::logical_truth is scalar OR folding; reduction-or-reference proves the
// tree classifier. Numeric BigUint construction/allocation are real. Wide raw
// planes compose with public-big-getters-128, including its u64 constructor.
fn public_logical_not_trace<const RESULT: usize>() {
    let (value, before) = input();
    let natural = before.width;
    let context = natural.max(usize::from(kani::any::<u8>() & 63) + 1);
    let mut cache = MaskCache::default();
    let _ = Op::BitNot.eval_value_unary(&value, natural, false, &mut cache);
    let _ = Op::BitNot.eval_value_unary(&value, natural, true, &mut cache);
    let _ = Op::BitNot.eval_value_unary(&value, context, false, &mut cache);
    let _ = Op::BitNot.eval_value_unary(&value, context, true, &mut cache);
    // Expand the fixed trace without raising BigUint loop bounds.
    macro_rules! reduction {
        ($op:expr) => {{
            let _ = ($op).eval_value_unary(&value, 1, before.signed, &mut cache);
            let _ = ($op).eval_value_unary(&value, 8, before.signed, &mut cache);
            let _ = ($op).eval_value_unary(&value, 65, before.signed, &mut cache);
            let _ = ($op).eval_value_unary(&value, 128, before.signed, &mut cache);
        }};
    }
    reduction!(Op::BitAnd);
    reduction!(Op::BitNand);
    reduction!(Op::BitOr);
    reduction!(Op::BitNor);
    reduction!(Op::BitXor);
    reduction!(Op::BitXnor);
    let expected = gates::not(native_reduction_or_tree(before));
    let output = Op::LogicNot.eval_value_unary(&value, 1, before.signed, &mut cache);
    assert_logical_and(output, 1, expected);
    let output = Op::LogicNot.eval_value_unary(&value, 8, before.signed, &mut cache);
    assert_logical_and(output, 8, expected);
    if RESULT != 8 {
        if RESULT == 128 {
            let _ = Op::LogicNot.eval_value_unary(&value, 65, before.signed, &mut cache);
        }
        let output = Op::LogicNot.eval_value_unary(&value, RESULT, before.signed, &mut cache);
        assert_wide_boolean_planes(output, RESULT, expected);
    }
    assert_input(&value, before);
    kani::cover!(
        natural == 1
            && !before.signed
            && before.planes == (0, 0)
            && context == 1
            && expected == Logic4::One,
        "public logical NOT width 1 zero unchanged is reachable"
    );
    kani::cover!(
        natural == 64
            && before.signed
            && before.planes == (1u128 << 63, 0)
            && context == 64
            && expected == Logic4::Zero,
        "public logical NOT width 64 signed high one is reachable"
    );
    kani::cover!(
        natural == 32
            && !before.signed
            && before.planes == (2, 0xffff_ffff)
            && context == 64
            && expected == Logic4::X,
        "public logical NOT widened X and Z is reachable"
    );
    kani::cover!(
        natural == 32
            && before.signed
            && before.planes == (3, 0xffff_fffe)
            && context == 64
            && expected == Logic4::Zero,
        "public logical NOT widened one dominates X and Z is reachable"
    );
}

// Fixed-width symbolic witnesses preserve the full trace and admit arbitrary
// native bit planes and signs. They detect mutations without paying the variable
// width expansion cost; the separate universal harnesses still cover 1..=64.
fn logical_binary_inputs<const WITNESS: bool>() -> (Value, Operand, Value, Operand, usize) {
    if !WITNESS {
        let (a, ar) = input();
        let (b, br) = input();
        let context = ar
            .width
            .max(br.width)
            .max(usize::from(kani::any::<u8>() & 63) + 1);
        return (a, ar, b, br, context);
    }
    let ar = Operand {
        planes: (
            u128::from(kani::any::<u64>()),
            u128::from(kani::any::<u64>()),
        ),
        width: 64,
        signed: kani::any(),
    };
    let br = Operand {
        planes: (
            u128::from(kani::any::<u64>()),
            u128::from(kani::any::<u64>()),
        ),
        width: 64,
        signed: kani::any(),
    };
    let a = Value::from_u128(ar.planes.0, ar.planes.1, 64, ar.signed);
    let b = Value::from_u128(br.planes.0, br.planes.1, 64, br.signed);
    assert_input(&a, ar);
    assert_input(&b, br);
    let (Value::U64(a), Value::U64(b)) = (a, b) else {
        panic!("public gate semantics mismatch");
    };
    (Value::U64(a), ar, Value::U64(b), br, 64)
}

// IEEE 1800 11.4.7/11.8.2. Independent normalized four-state source widths
// 1..=64 and signs; caller sign is their conjunction, matching the direct trace.
// Execute AND/OR/XOR/XNOR at natural then independent context natural..=64 on
// one cache. Then logical AND 1/8/65, followed by logical OR 1/8/65 as selected.
// STAGE 0 observes AND65, 1 observes OR1/8, 2 observes OR65. All earlier calls run.
// Evaluated-value dispatch only; this is not a short-circuit/effect theorem.
// Per-bit classifiers compose with logical-truth-unrolled-reference. Full unsigned
// numeric planes and metadata compose with public-big-getters-128 for BigUint.
fn public_logical_binary_trace<const STAGE: u8, const WITNESS: bool>() {
    let (a, ar, b, br, context) = logical_binary_inputs::<WITNESS>();
    let natural = ar.width.max(br.width);
    let signed = ar.signed && br.signed;
    let mut cache = MaskCache::default();
    macro_rules! bitwise {
        ($width:expr) => {{
            let _ = Op::BitAnd.eval_value_binary(&a, &b, $width, signed, &mut cache);
            let _ = Op::BitOr.eval_value_binary(&a, &b, $width, signed, &mut cache);
            let _ = Op::BitXor.eval_value_binary(&a, &b, $width, signed, &mut cache);
            let _ = Op::BitXnor.eval_value_binary(&a, &b, $width, signed, &mut cache);
        }};
    }
    bitwise!(natural);
    bitwise!(context);
    let lhs = unrolled_native_logical_truth(ar);
    let rhs = unrolled_native_logical_truth(br);
    let conjunction = gates::binary(lhs, rhs, BitOp::And);
    let output = Op::LogicAnd.eval_value_binary(&a, &b, 1, signed, &mut cache);
    assert_logical_and(output, 1, conjunction);
    let output = Op::LogicAnd.eval_value_binary(&a, &b, 8, signed, &mut cache);
    assert_logical_and(output, 8, conjunction);
    let output = Op::LogicAnd.eval_value_binary(&a, &b, 65, signed, &mut cache);
    if STAGE == 0 {
        assert_wide_boolean_planes(output, 65, conjunction);
    } else {
        drop(output);
        let disjunction = gates::binary(lhs, rhs, BitOp::Or);
        let output = Op::LogicOr.eval_value_binary(&a, &b, 1, signed, &mut cache);
        assert_logical_and(output, 1, disjunction);
        let output = Op::LogicOr.eval_value_binary(&a, &b, 8, signed, &mut cache);
        assert_logical_and(output, 8, disjunction);
        if STAGE == 2 {
            let output = Op::LogicOr.eval_value_binary(&a, &b, 65, signed, &mut cache);
            assert_wide_boolean_planes(output, 65, disjunction);
        }
    }
    assert_input(&a, ar);
    assert_input(&b, br);
    if WITNESS {
        let outcome = gates::binary(lhs, rhs, if STAGE == 0 { BitOp::And } else { BitOp::Or });
        kani::cover!(
            outcome == Logic4::Zero,
            "public logical binary witness zero result is reachable"
        );
        kani::cover!(
            outcome == Logic4::One,
            "public logical binary witness one result is reachable"
        );
        kani::cover!(
            outcome == Logic4::X,
            "public logical binary witness X result is reachable"
        );
    } else {
        kani::cover!(true, "public logical binary full continuation is reachable");
    }
}

fn cover_logical_binary_domain(ar: Operand, br: Operand, context: usize, lhs: Logic4, rhs: Logic4) {
    kani::cover!(
        ar.width == 1
            && br.width == 64
            && !ar.signed
            && !br.signed
            && ar.planes == (0, 0)
            && br.planes == (1u128 << 63, 0)
            && context == 64,
        "public logical binary unsigned narrow LHS and high RHS are reachable"
    );
    kani::cover!(
        ar.width == 64
            && br.width == 1
            && ar.signed
            && br.signed
            && ar.planes == (1u128 << 63, 0)
            && br.planes == (0, 0)
            && context == 64,
        "public logical binary signed high LHS and narrow RHS are reachable"
    );
    kani::cover!(
        ar.width == 32
            && br.width == 2
            && ar.signed
            && !br.signed
            && ar.planes == (1, 2)
            && br.planes == (3, 2)
            && context == 64
            && lhs == Logic4::One
            && rhs == Logic4::One,
        "public logical binary signed LHS known ones dominate X and Z"
    );
    kani::cover!(
        ar.width == 2
            && br.width == 32
            && !ar.signed
            && br.signed
            && ar.planes == (0, 2)
            && br.planes == (0, 0)
            && context == 64
            && lhs == Logic4::X
            && rhs == Logic4::Zero,
        "public logical binary signed RHS zero and unknown LHS are reachable"
    );
    kani::cover!(
        ar.width == 32
            && br.width == 2
            && !ar.signed
            && !br.signed
            && ar.planes == (0, 0)
            && br.planes == (2, 2)
            && context == 64
            && lhs == Logic4::Zero
            && rhs == Logic4::X,
        "public logical binary zero LHS and Z RHS are reachable"
    );
    kani::cover!(
        ar.width == 2
            && br.width == 32
            && ar.signed
            && br.signed
            && ar.planes == (0, 2)
            && br.planes == (0, 2)
            && context == 64
            && lhs == Logic4::X
            && rhs == Logic4::X,
        "public logical binary both unknown widened sources are reachable"
    );
    kani::cover!(
        ar.width == 1
            && br.width == 1
            && !ar.signed
            && !br.signed
            && ar.planes == (0, 0)
            && br.planes == (0, 0)
            && context == 1
            && lhs == Logic4::Zero
            && rhs == Logic4::Zero,
        "public logical binary both zero unchanged sources are reachable"
    );
}

// Check the admitted constructor/sign/width/truth cubes separately from the
// heap continuation. The semantic harness has the identical unconstrained input
// generator and checks all inputs, plus reachability of each actual result state.
#[kani::proof]
#[kani::unwind(1)]
#[kani::use_stub_set(small_native_gate)]
#[kani::stub(veryl_analyzer::value::biguint_to_u128, reject_logical_biguint_getter)]
fn public_logical_binary_domain_is_reachable() {
    let (_, ar) = input();
    let (_, br) = input();
    let natural = ar.width.max(br.width);
    let context = natural.max(usize::from(kani::any::<u8>() & 63) + 1);
    let lhs = unrolled_native_logical_truth(ar);
    let rhs = unrolled_native_logical_truth(br);
    cover_logical_binary_domain(ar, br, context, lhs, rhs);
}

#[kani::proof]
#[kani::unwind(2)]
#[kani::use_stub_set(native_gate_wide_result)]
#[kani::stub(veryl_analyzer::value::biguint_to_u128, reject_logical_biguint_getter)]
fn public_logical_not_native_continuation_matches_reference_semantics() {
    public_logical_not_trace::<8>();
}

#[kani::proof]
#[kani::unwind(2)]
#[kani::use_stub_set(native_gate_wide_result)]
#[kani::stub(veryl_analyzer::value::biguint_to_u128, reject_logical_biguint_getter)]
fn public_logical_not_width65_matches_reference_semantics() {
    public_logical_not_trace::<65>();
}

#[kani::proof]
#[kani::unwind(2)]
#[kani::use_stub_set(native_gate_wide_result)]
#[kani::stub(veryl_analyzer::value::biguint_to_u128, reject_logical_biguint_getter)]
fn public_logical_not_width128_matches_reference_semantics() {
    public_logical_not_trace::<128>();
}

#[kani::proof]
#[kani::unwind(1)]
#[kani::use_stub_set(native_gate_wide_result)]
#[kani::stub(veryl_analyzer::value::biguint_to_u128, reject_logical_biguint_getter)]
fn public_logical_and_width65_matches_reference_semantics() {
    public_logical_binary_trace::<0, false>();
}

#[kani::proof]
#[kani::unwind(1)]
#[kani::use_stub_set(native_gate_wide_result)]
#[kani::stub(veryl_analyzer::value::biguint_to_u128, reject_logical_biguint_getter)]
fn public_logical_or_native_continuation_matches_reference_semantics() {
    public_logical_binary_trace::<1, false>();
}

#[kani::proof]
#[kani::unwind(1)]
#[kani::use_stub_set(native_gate_wide_result)]
#[kani::stub(veryl_analyzer::value::biguint_to_u128, reject_logical_biguint_getter)]
fn public_logical_or_width65_matches_reference_semantics() {
    public_logical_binary_trace::<2, false>();
}

#[kani::proof]
#[kani::unwind(1)]
#[kani::use_stub_set(native_gate_wide_result)]
#[kani::stub(veryl_analyzer::value::biguint_to_u128, reject_logical_biguint_getter)]
fn public_logical_and_width65_witness_matches_reference_semantics() {
    public_logical_binary_trace::<0, true>();
}

#[kani::proof]
#[kani::unwind(1)]
#[kani::use_stub_set(native_gate_wide_result)]
#[kani::stub(veryl_analyzer::value::biguint_to_u128, reject_logical_biguint_getter)]
fn public_logical_or_native_witness_matches_reference_semantics() {
    public_logical_binary_trace::<1, true>();
}

#[kani::proof]
#[kani::unwind(1)]
#[kani::use_stub_set(native_gate_wide_result)]
#[kani::stub(veryl_analyzer::value::biguint_to_u128, reject_logical_biguint_getter)]
fn public_logical_or_width65_witness_matches_reference_semantics() {
    public_logical_binary_trace::<2, true>();
}

// IEEE 1800 11.4.7: classify each source bit independently. A scalar known
// one determines true; absent any known one, X/Z determines unknown, else false.
// Nested loops visit every bit while bounding each loop by eight. This scalar
// scan is independent of the production operation's aggregate plane arithmetic.
pub(crate) fn chunked_logical_truth(source: Operand) -> Logic4 {
    let mut saw_one = false;
    let mut saw_unknown = false;
    for chunk in 0..8 {
        for bit in 0..8 {
            let position = chunk * 8 + bit;
            if position < source.width {
                match read_logic(source.planes.0, source.planes.1, position) {
                    Logic4::One => saw_one = true,
                    Logic4::X | Logic4::Z => saw_unknown = true,
                    Logic4::Zero => {}
                }
            }
        }
    }
    if saw_one {
        Logic4::One
    } else if saw_unknown {
        Logic4::X
    } else {
        Logic4::Zero
    }
}

// Native-only observations must never enter the BigUint getter conversion.
// Fail closed here instead of expanding unreachable digit-collection loops.
fn reject_logical_biguint_getter(_: &num_bigint::BigUint) -> u128 {
    panic!("public gate semantics mismatch");
}

// Full independent normalized four-state source widths 1..=64 and signs.
// Execute all four bitwise calls at natural then arbitrary context natural..=64
// on one fresh cache, followed by logical AND widths 1 then 8. This strictly
// includes the former nine small-width, natural/natural+2 continuations.
// Value dispatch only: no short-circuit or operand-evaluation effect theorem.
#[kani::proof]
#[kani::unwind(9)]
#[kani::use_stub_set(small_native_gate)]
#[kani::stub(veryl_analyzer::value::biguint_to_u128, reject_logical_biguint_getter)]
fn public_logical_and_native_continuation_matches_reference_semantics() {
    let (a, ar) = input();
    let (b, br) = input();
    let natural = ar.width.max(br.width);
    let width = natural.max(usize::from(kani::any::<u8>() & 63) + 1);
    let signed = ar.signed && br.signed;
    let mut cache = MaskCache::default();
    let _ = Op::BitAnd.eval_value_binary(&a, &b, natural, signed, &mut cache);
    let _ = Op::BitOr.eval_value_binary(&a, &b, natural, signed, &mut cache);
    let _ = Op::BitXor.eval_value_binary(&a, &b, natural, signed, &mut cache);
    let _ = Op::BitXnor.eval_value_binary(&a, &b, natural, signed, &mut cache);
    let _ = Op::BitAnd.eval_value_binary(&a, &b, width, signed, &mut cache);
    let _ = Op::BitOr.eval_value_binary(&a, &b, width, signed, &mut cache);
    let _ = Op::BitXor.eval_value_binary(&a, &b, width, signed, &mut cache);
    let _ = Op::BitXnor.eval_value_binary(&a, &b, width, signed, &mut cache);
    let lhs = chunked_logical_truth(ar);
    let rhs = chunked_logical_truth(br);
    let expected = gates::binary(lhs, rhs, BitOp::And);
    let output = Op::LogicAnd.eval_value_binary(&a, &b, 1, signed, &mut cache);
    assert_logical_and(output, 1, expected);
    let output = Op::LogicAnd.eval_value_binary(&a, &b, 8, signed, &mut cache);
    assert_logical_and(output, 8, expected);
    assert_input(&a, ar);
    assert_input(&b, br);
    kani::cover!(
        ar.width == 1,
        "public native logical AND LHS width 1 is reachable"
    );
    kani::cover!(
        ar.width == 64,
        "public native logical AND LHS width 64 is reachable"
    );
    kani::cover!(
        br.width == 1,
        "public native logical AND RHS width 1 is reachable"
    );
    kani::cover!(
        br.width == 64,
        "public native logical AND RHS width 64 is reachable"
    );
    kani::cover!(
        !ar.signed && !br.signed,
        "public native logical AND unsigned sources is reachable"
    );
    kani::cover!(
        ar.signed && br.signed,
        "public native logical AND signed sources is reachable"
    );
    kani::cover!(
        ar.signed && !br.signed,
        "public native logical AND signed LHS only is reachable"
    );
    kani::cover!(
        !ar.signed && br.signed,
        "public native logical AND signed RHS only is reachable"
    );
    kani::cover!(
        expected == Logic4::Zero,
        "public native logical AND false result is reachable"
    );
    kani::cover!(
        expected == Logic4::One,
        "public native logical AND true result is reachable"
    );
    kani::cover!(
        expected == Logic4::X,
        "public native logical AND X result is reachable"
    );
    kani::cover!(
        lhs == Logic4::Zero && rhs == Logic4::X,
        "public native logical AND zero dominates RHS unknown is reachable"
    );
    kani::cover!(
        lhs == Logic4::X && rhs == Logic4::Zero,
        "public native logical AND zero dominates LHS unknown is reachable"
    );
    kani::cover!(
        ar.planes.0 & ar.planes.1 != 0,
        "public native logical AND source Z is reachable"
    );
    kani::cover!(
        (ar.planes.1 | br.planes.1) != 0 && lhs == Logic4::One && rhs == Logic4::One,
        "public native logical AND known ones dominate unknown is reachable"
    );
    kani::cover!(
        width == natural,
        "public native logical AND unchanged context is reachable"
    );
    kani::cover!(
        width > natural,
        "public native logical AND widened context is reachable"
    );
    kani::cover!(
        width == 64,
        "public native logical AND context width 64 is reachable"
    );
}

// Validate the scalar classification reference against the established four-state
// OR-table fold for all widths 1..=64, including arbitrary upper plane bits.
// This pure reference theorem calls no compiler operation or constructor.
#[kani::proof]
#[kani::unwind(65)]
fn logical_truth_scan_matches_scalar_fold() {
    let source = Operand {
        planes: (kani::any(), kani::any()),
        width: usize::from(kani::any::<u8>() & 63) + 1,
        signed: kani::any(),
    };
    let expected = gates::logical_truth(source.planes, source.width);
    assert!(
        chunked_logical_truth(source) == expected,
        "logical truth reference mismatch"
    );
    kani::cover!(
        source.width == 1,
        "logical truth reference width 1 is reachable"
    );
    kani::cover!(
        source.width == 64,
        "logical truth reference width 64 is reachable"
    );
    kani::cover!(
        expected == Logic4::Zero,
        "logical truth reference false is reachable"
    );
    kani::cover!(
        expected == Logic4::One,
        "logical truth reference true is reachable"
    );
    kani::cover!(
        expected == Logic4::X,
        "logical truth reference X is reachable"
    );
}

// A fixed counterexample domain keeps mutation detection inexpensive. It never
// replaces the required full native-width positive proof above. Source signs and
// all four scalar states remain independent; second bitwise context is width 3.
fn logical_witness_input() -> (Value, Operand) {
    let before = Operand {
        planes: (
            u128::from(kani::any::<u8>() & 1),
            u128::from(kani::any::<u8>() & 1),
        ),
        width: 1,
        signed: kani::any(),
    };
    let value = Value::from_u128(before.planes.0, before.planes.1, 1, before.signed);
    assert_input(&value, before);
    let Value::U64(native) = value else {
        panic!("public gate semantics mismatch");
    };
    (Value::U64(native), before)
}

#[kani::proof]
#[kani::unwind(9)]
#[kani::use_stub_set(small_native_gate)]
#[kani::stub(veryl_analyzer::value::biguint_to_u128, reject_logical_biguint_getter)]
fn public_logical_and_mutation_witness_matches_reference_semantics() {
    let (a, ar) = logical_witness_input();
    let (b, br) = logical_witness_input();
    let signed = ar.signed && br.signed;
    let mut cache = MaskCache::default();
    let _ = Op::BitAnd.eval_value_binary(&a, &b, 1, signed, &mut cache);
    let _ = Op::BitOr.eval_value_binary(&a, &b, 1, signed, &mut cache);
    let _ = Op::BitXor.eval_value_binary(&a, &b, 1, signed, &mut cache);
    let _ = Op::BitXnor.eval_value_binary(&a, &b, 1, signed, &mut cache);
    let _ = Op::BitAnd.eval_value_binary(&a, &b, 3, signed, &mut cache);
    let _ = Op::BitOr.eval_value_binary(&a, &b, 3, signed, &mut cache);
    let _ = Op::BitXor.eval_value_binary(&a, &b, 3, signed, &mut cache);
    let _ = Op::BitXnor.eval_value_binary(&a, &b, 3, signed, &mut cache);
    let lhs = chunked_logical_truth(ar);
    let rhs = chunked_logical_truth(br);
    let expected = gates::binary(lhs, rhs, BitOp::And);
    let output = Op::LogicAnd.eval_value_binary(&a, &b, 1, signed, &mut cache);
    assert_logical_and(output, 1, expected);
    let output = Op::LogicAnd.eval_value_binary(&a, &b, 8, signed, &mut cache);
    assert_logical_and(output, 8, expected);
    assert_input(&a, ar);
    assert_input(&b, br);
    kani::cover!(
        !ar.signed && !br.signed,
        "logical AND witness unsigned sources is reachable"
    );
    kani::cover!(
        ar.signed && br.signed,
        "logical AND witness signed sources is reachable"
    );
    kani::cover!(
        ar.signed && !br.signed,
        "logical AND witness signed LHS only is reachable"
    );
    kani::cover!(
        !ar.signed && br.signed,
        "logical AND witness signed RHS only is reachable"
    );
    kani::cover!(
        expected == Logic4::Zero,
        "logical AND witness false result is reachable"
    );
    kani::cover!(
        expected == Logic4::One,
        "logical AND witness true result is reachable"
    );
    kani::cover!(
        expected == Logic4::X,
        "logical AND witness X result is reachable"
    );
    kani::cover!(
        lhs == Logic4::Zero && rhs == Logic4::X,
        "logical AND witness zero dominates RHS unknown is reachable"
    );
    kani::cover!(
        lhs == Logic4::X && rhs == Logic4::Zero,
        "logical AND witness zero dominates LHS unknown is reachable"
    );
    kani::cover!(
        ar.planes.0 & ar.planes.1 != 0,
        "logical AND witness source Z is reachable"
    );
}

// All normalized native source widths 1..=64, independent source signs, and
// caller sign their conjunction. Execute the natural-width prefix, then observe
// all four operations at arbitrary context natural..=64 on the same cache,
// including preservation after each widened call. Natural outputs have their own proof. Scalar reference tables and arbitrary getter positions 0..127
// check every stored bit and the upper u128 padding (IEEE 1800 11.4.8/11.8.2).
#[kani::proof]
#[kani::unwind(1)]
#[kani::use_stub_set(small_native_gate)]
#[kani::stub(veryl_analyzer::value::biguint_to_u128, reject_logical_biguint_getter)]
fn public_bitwise_widened_native_trace_matches_reference_semantics() {
    let (a, ar) = input();
    let (b, br) = input();
    let natural = ar.width.max(br.width);
    let width = natural.max(usize::from(kani::any::<u8>() & 63) + 1);
    let signed = ar.signed && br.signed;
    let position = usize::from(kani::any::<u8>() & 127);
    let mut cache = MaskCache::default();
    let _ = Op::BitAnd.eval_value_binary(&a, &b, natural, signed, &mut cache);
    let _ = Op::BitOr.eval_value_binary(&a, &b, natural, signed, &mut cache);
    let _ = Op::BitXor.eval_value_binary(&a, &b, natural, signed, &mut cache);
    let _ = Op::BitXnor.eval_value_binary(&a, &b, natural, signed, &mut cache);
    assert_input(&a, ar);
    assert_input(&b, br);
    let output = Op::BitAnd.eval_value_binary(&a, &b, width, signed, &mut cache);
    assert_binary(output, ar, br, width, position, BitOp::And);
    assert_input(&a, ar);
    assert_input(&b, br);
    let output = Op::BitOr.eval_value_binary(&a, &b, width, signed, &mut cache);
    assert_binary(output, ar, br, width, position, BitOp::Or);
    assert_input(&a, ar);
    assert_input(&b, br);
    let output = Op::BitXor.eval_value_binary(&a, &b, width, signed, &mut cache);
    assert_binary(output, ar, br, width, position, BitOp::Xor);
    assert_input(&a, ar);
    assert_input(&b, br);
    let output = Op::BitXnor.eval_value_binary(&a, &b, width, signed, &mut cache);
    assert_binary(output, ar, br, width, position, BitOp::Xnor);
    assert_input(&a, ar);
    assert_input(&b, br);
    kani::cover!(
        ar.width == 1,
        "public bitwise trace LHS width 1 is reachable"
    );
    kani::cover!(
        ar.width == 64,
        "public bitwise trace LHS width 64 is reachable"
    );
    kani::cover!(
        br.width == 1,
        "public bitwise trace RHS width 1 is reachable"
    );
    kani::cover!(
        br.width == 64,
        "public bitwise trace RHS width 64 is reachable"
    );
    kani::cover!(
        ar.width > br.width,
        "public bitwise trace wider LHS is reachable"
    );
    kani::cover!(
        ar.width < br.width,
        "public bitwise trace wider RHS is reachable"
    );
    kani::cover!(
        width == natural,
        "public bitwise trace unchanged context is reachable"
    );
    kani::cover!(
        width > natural,
        "public bitwise trace widened context is reachable"
    );
    kani::cover!(
        width == 64,
        "public bitwise trace context width 64 is reachable"
    );
    kani::cover!(
        !ar.signed && !br.signed,
        "public bitwise trace unsigned sources is reachable"
    );
    kani::cover!(
        ar.signed && br.signed,
        "public bitwise trace signed sources is reachable"
    );
    kani::cover!(
        ar.signed && !br.signed,
        "public bitwise trace signed LHS only is reachable"
    );
    kani::cover!(
        !ar.signed && br.signed,
        "public bitwise trace signed RHS only is reachable"
    );
    kani::cover!(
        ar.planes.1 & !ar.planes.0 != 0,
        "public bitwise trace source X is reachable"
    );
    kani::cover!(
        br.planes.0 & br.planes.1 != 0,
        "public bitwise trace source Z is reachable"
    );
    kani::cover!(
        signed && width > natural && (ar.planes.0 & ar.planes.1) >> (ar.width - 1) == 1,
        "public bitwise trace signed Z extension is reachable"
    );
    kani::cover!(
        position == 127,
        "public bitwise trace u128 padding is reachable"
    );
}

// Mutation-only inputs: distinct widths exercise natural sign extension as well
// as the following widened pass. The full native trace remains mandatory.
fn bitwise_witness_input(width: usize) -> (Value, Operand) {
    let mask = (1u8 << width) - 1;
    let before = Operand {
        planes: (
            u128::from(kani::any::<u8>() & mask),
            u128::from(kani::any::<u8>() & mask),
        ),
        width,
        signed: kani::any(),
    };
    let value = Value::from_u128(before.planes.0, before.planes.1, width, before.signed);
    assert_input(&value, before);
    let Value::U64(native) = value else {
        panic!("public gate semantics mismatch");
    };
    (Value::U64(native), before)
}

// Width-4-only mutations leave the entire natural-width prefix intact. A
// semantic rejection must therefore come from observing the widened operations.
#[kani::proof]
#[kani::unwind(1)]
#[kani::use_stub_set(small_native_gate)]
#[kani::stub(veryl_analyzer::value::biguint_to_u128, reject_logical_biguint_getter)]
fn public_bitwise_widened_mutation_witness_matches_reference_semantics() {
    let (a, ar) = bitwise_witness_input(1);
    let (b, br) = bitwise_witness_input(2);
    let signed = ar.signed && br.signed;
    let position = usize::from(kani::any::<u8>() & 127);
    let mut cache = MaskCache::default();
    let output = Op::BitAnd.eval_value_binary(&a, &b, 2, signed, &mut cache);
    assert_binary(output, ar, br, 2, position, BitOp::And);
    let output = Op::BitOr.eval_value_binary(&a, &b, 2, signed, &mut cache);
    assert_binary(output, ar, br, 2, position, BitOp::Or);
    let output = Op::BitXor.eval_value_binary(&a, &b, 2, signed, &mut cache);
    assert_binary(output, ar, br, 2, position, BitOp::Xor);
    let output = Op::BitXnor.eval_value_binary(&a, &b, 2, signed, &mut cache);
    assert_binary(output, ar, br, 2, position, BitOp::Xnor);
    assert_input(&a, ar);
    assert_input(&b, br);
    let output = Op::BitAnd.eval_value_binary(&a, &b, 4, signed, &mut cache);
    assert_binary(output, ar, br, 4, position, BitOp::And);
    let output = Op::BitOr.eval_value_binary(&a, &b, 4, signed, &mut cache);
    assert_binary(output, ar, br, 4, position, BitOp::Or);
    let output = Op::BitXor.eval_value_binary(&a, &b, 4, signed, &mut cache);
    assert_binary(output, ar, br, 4, position, BitOp::Xor);
    let output = Op::BitXnor.eval_value_binary(&a, &b, 4, signed, &mut cache);
    assert_binary(output, ar, br, 4, position, BitOp::Xnor);
    assert_input(&a, ar);
    assert_input(&b, br);
    kani::cover!(
        !ar.signed && !br.signed,
        "public bitwise witness unsigned sources is reachable"
    );
    kani::cover!(
        ar.signed && br.signed,
        "public bitwise witness signed sources is reachable"
    );
    kani::cover!(
        ar.signed && !br.signed,
        "public bitwise witness signed LHS only is reachable"
    );
    kani::cover!(
        !ar.signed && br.signed,
        "public bitwise witness signed RHS only is reachable"
    );
    kani::cover!(
        ar.planes.1 & !ar.planes.0 != 0,
        "public bitwise witness source X is reachable"
    );
    kani::cover!(
        br.planes.0 & br.planes.1 != 0,
        "public bitwise witness source Z is reachable"
    );
    kani::cover!(
        position == 127,
        "public bitwise witness u128 padding is reachable"
    );
}
