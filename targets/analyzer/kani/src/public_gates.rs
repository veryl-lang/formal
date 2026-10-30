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
#[kani::unwind(9)]
#[kani::use_stub_set(small_native_gate)]
fn public_reduction_and_native_continuation_matches_reference_semantics() {
    let (value, before) = input();
    let natural = before.width;
    let width = natural.max(usize::from(kani::any::<u8>() & 63) + 1);
    let mut cache = MaskCache::default();
    let _ = Op::BitNot.eval_value_unary(&value, natural, false, &mut cache);
    let _ = Op::BitNot.eval_value_unary(&value, natural, true, &mut cache);
    let _ = Op::BitNot.eval_value_unary(&value, width, false, &mut cache);
    let _ = Op::BitNot.eval_value_unary(&value, width, true, &mut cache);
    // AND is associative under the scalar four-state table. Folding at most
    // eight bits per chunk preserves the full 64-bit domain while bounding
    // every reference loop by eight rather than raising the global unwind.
    let mut expected = Logic4::One;
    for chunk in 0..8 {
        let start = chunk * 8;
        if start < natural {
            let part = gates::reduced_logic(
                (before.planes.0 >> start, before.planes.1 >> start),
                (natural - start).min(8),
                gates::Reduction::And,
            );
            expected = gates::binary(expected, part, BitOp::And);
        }
    }
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

// IEEE 1800 11.4.7: classify each source bit independently. A scalar known
// one determines true; absent any known one, X/Z determines unknown, else false.
// Nested loops visit every bit while bounding each loop by eight. This scalar
// scan is independent of the production operation's aggregate plane arithmetic.
fn chunked_logical_truth(source: Operand) -> Logic4 {
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
