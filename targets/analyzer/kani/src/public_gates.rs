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

// Fixed width witness for mutation checks; the four full native-width proofs
// remain required. Keeping this counterexample small avoids spending the mutation
// budget solving unrelated symbolic width and getter branches.
fn mutation_input() -> (Value, Operand) {
    let before = Operand {
        planes: (
            u128::from(kani::any::<u8>() & 3),
            u128::from(kani::any::<u8>() & 3),
        ),
        width: 2,
        signed: kani::any(),
    };
    let value = Value::from_u128(
        before.planes.0,
        before.planes.1,
        before.width,
        before.signed,
    );
    assert_input(&value, before);
    (value, before)
}

// Each proof observes one continuation while retaining its complete call history.
fn public_bitwise_widened_continuation<const MODE: u8, const FIXED: bool>() {
    let (a, ar) = if FIXED { mutation_input() } else { input() };
    let (b, br) = if FIXED { mutation_input() } else { input() };
    let natural = ar.width.max(br.width);
    let width = if FIXED {
        4
    } else {
        natural.max(usize::from(kani::any::<u8>() & 63) + 1)
    };
    let signed = ar.signed && br.signed;
    let position = usize::from(kani::any::<u8>() & 127);
    let mut cache = MaskCache::default();
    let _ = Op::BitAnd.eval_value_binary(&a, &b, natural, signed, &mut cache);
    let _ = Op::BitOr.eval_value_binary(&a, &b, natural, signed, &mut cache);
    let _ = Op::BitXor.eval_value_binary(&a, &b, natural, signed, &mut cache);
    let _ = Op::BitXnor.eval_value_binary(&a, &b, natural, signed, &mut cache);
    if MODE > 1 {
        let _ = Op::BitAnd.eval_value_binary(&a, &b, width, signed, &mut cache);
    }
    if MODE > 2 {
        let _ = Op::BitOr.eval_value_binary(&a, &b, width, signed, &mut cache);
    }
    if MODE > 3 {
        let _ = Op::BitXor.eval_value_binary(&a, &b, width, signed, &mut cache);
    }
    let (output, kind) = match MODE {
        1 => (
            Op::BitAnd.eval_value_binary(&a, &b, width, signed, &mut cache),
            BitOp::And,
        ),
        2 => (
            Op::BitOr.eval_value_binary(&a, &b, width, signed, &mut cache),
            BitOp::Or,
        ),
        3 => (
            Op::BitXor.eval_value_binary(&a, &b, width, signed, &mut cache),
            BitOp::Xor,
        ),
        4 => (
            Op::BitXnor.eval_value_binary(&a, &b, width, signed, &mut cache),
            BitOp::Xnor,
        ),
        _ => unreachable!(),
    };
    assert_binary(output, ar, br, width, position, kind);
    assert_input(&a, ar);
    assert_input(&b, br);
    if FIXED {
        kani::cover!(
            signed,
            "public widened bitwise witness signed context is reachable"
        );
        kani::cover!(
            !signed,
            "public widened bitwise witness unsigned context is reachable"
        );
        kani::cover!(
            ar.planes.1 != 0,
            "public widened bitwise witness unknown input is reachable"
        );
        kani::cover!(
            position == 127,
            "public widened bitwise witness padding is reachable"
        );
        return;
    }
    kani::cover!(
        ar.width == 1,
        "public widened bitwise LHS width 1 is reachable"
    );
    kani::cover!(
        ar.width == 64,
        "public widened bitwise LHS width 64 is reachable"
    );
    kani::cover!(
        br.width == 1,
        "public widened bitwise RHS width 1 is reachable"
    );
    kani::cover!(
        br.width == 64,
        "public widened bitwise RHS width 64 is reachable"
    );
    kani::cover!(
        ar.width > br.width,
        "public widened bitwise wider LHS is reachable"
    );
    kani::cover!(
        ar.width < br.width,
        "public widened bitwise wider RHS is reachable"
    );
    kani::cover!(
        width == natural,
        "public widened bitwise unchanged context is reachable"
    );
    kani::cover!(
        width > natural,
        "public widened bitwise growth is reachable"
    );
    kani::cover!(width == 64, "public widened bitwise width 64 is reachable");
    kani::cover!(
        signed && width > natural && (ar.planes.0 & ar.planes.1) >> (ar.width - 1) == 1,
        "public widened bitwise signed Z extension is reachable"
    );
    kani::cover!(signed, "public widened bitwise signed context is reachable");
    kani::cover!(
        ar.signed != br.signed,
        "public widened bitwise mixed signs are reachable"
    );
    kani::cover!(
        ar.planes.1 & !ar.planes.0 != 0,
        "public widened bitwise source X is reachable"
    );
    kani::cover!(
        br.planes.1 & br.planes.0 != 0,
        "public widened bitwise source Z is reachable"
    );
    kani::cover!(
        position == 127,
        "public widened bitwise u128 padding is reachable"
    );
}

#[kani::proof]
#[kani::unwind(1)]
#[kani::use_stub_set(small_native_gate)]
fn public_bitwise_widened_and_matches_reference_semantics() {
    public_bitwise_widened_continuation::<1, false>();
}

#[kani::proof]
#[kani::unwind(1)]
#[kani::use_stub_set(small_native_gate)]
fn public_bitwise_widened_or_matches_reference_semantics() {
    public_bitwise_widened_continuation::<2, false>();
}

#[kani::proof]
#[kani::unwind(1)]
#[kani::use_stub_set(small_native_gate)]
fn public_bitwise_widened_xor_matches_reference_semantics() {
    public_bitwise_widened_continuation::<3, false>();
}

#[kani::proof]
#[kani::unwind(1)]
#[kani::use_stub_set(small_native_gate)]
fn public_bitwise_widened_xnor_matches_reference_semantics() {
    public_bitwise_widened_continuation::<4, false>();
}

#[kani::proof]
#[kani::unwind(1)]
#[kani::use_stub_set(small_native_gate)]
fn public_bitwise_widened_xnor_witness_matches_reference_semantics() {
    public_bitwise_widened_continuation::<4, true>();
}

// IEEE 1800 11.4.7: evaluated operands use independently folded scalar truth
// tables. This is a value-dispatch theorem, not a short-circuit/effect theorem.
// Independent normalized four-state widths 1..=3 and source signs; the caller
// sign is their conjunction. Preserve the natural/natural+2 bitwise prefix on
// one cache, then observe logical AND at widths 1 and 8. Width 65 stays executable.
fn small_logical_input(width: usize) -> (Value, Operand) {
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

// The nine constant pairs jointly cover independent source widths 1..=3.
// Constant sizing avoids changing the admitted state/sign domain to fit the solver.
fn public_small_logical_and_pair<const LEFT: usize, const RIGHT: usize>() {
    let (a, ar) = small_logical_input(LEFT);
    let (b, br) = small_logical_input(RIGHT);
    let natural = ar.width.max(br.width);
    let signed = ar.signed && br.signed;
    let mut cache = MaskCache::default();
    let _ = Op::BitAnd.eval_value_binary(&a, &b, natural, signed, &mut cache);
    let _ = Op::BitOr.eval_value_binary(&a, &b, natural, signed, &mut cache);
    let _ = Op::BitXor.eval_value_binary(&a, &b, natural, signed, &mut cache);
    let _ = Op::BitXnor.eval_value_binary(&a, &b, natural, signed, &mut cache);
    let _ = Op::BitAnd.eval_value_binary(&a, &b, natural + 2, signed, &mut cache);
    let _ = Op::BitOr.eval_value_binary(&a, &b, natural + 2, signed, &mut cache);
    let _ = Op::BitXor.eval_value_binary(&a, &b, natural + 2, signed, &mut cache);
    let _ = Op::BitXnor.eval_value_binary(&a, &b, natural + 2, signed, &mut cache);
    let lhs = gates::logical_truth(ar.planes, ar.width);
    let rhs = gates::logical_truth(br.planes, br.width);
    let expected = gates::binary(lhs, rhs, BitOp::And);
    let output = Op::LogicAnd.eval_value_binary(&a, &b, 1, signed, &mut cache);
    assert_logical_and(output, 1, expected);
    let output = Op::LogicAnd.eval_value_binary(&a, &b, 8, signed, &mut cache);
    assert_logical_and(output, 8, expected);
    assert_input(&a, ar);
    assert_input(&b, br);
    kani::cover!(
        !ar.signed && !br.signed,
        "public small logical AND unsigned sources is reachable"
    );
    kani::cover!(
        ar.signed && br.signed,
        "public small logical AND signed sources is reachable"
    );
    kani::cover!(
        ar.signed && !br.signed,
        "public small logical AND signed LHS only is reachable"
    );
    kani::cover!(
        !ar.signed && br.signed,
        "public small logical AND signed RHS only is reachable"
    );
    kani::cover!(
        expected == Logic4::Zero,
        "public small logical AND false result is reachable"
    );
    kani::cover!(
        expected == Logic4::One,
        "public small logical AND true result is reachable"
    );
    kani::cover!(
        expected == Logic4::X,
        "public small logical AND X result is reachable"
    );
    kani::cover!(
        lhs == Logic4::Zero && rhs == Logic4::X,
        "public small logical AND zero dominates RHS unknown is reachable"
    );
    kani::cover!(
        lhs == Logic4::X && rhs == Logic4::Zero,
        "public small logical AND zero dominates LHS unknown is reachable"
    );
    kani::cover!(
        ar.planes.0 & ar.planes.1 != 0,
        "public small logical AND source Z is reachable"
    );
    if LEFT > 1 || RIGHT > 1 {
        kani::cover!(
            (ar.planes.1 | br.planes.1) != 0 && lhs == Logic4::One && rhs == Logic4::One,
            "public small logical AND known ones dominate unknown is reachable"
        );
    }
}

#[kani::proof]
#[kani::unwind(4)]
#[kani::use_stub_set(small_native_gate)]
fn public_small_logical_and_width_1_by_1_matches_reference_semantics() {
    public_small_logical_and_pair::<1, 1>();
}

#[kani::proof]
#[kani::unwind(4)]
#[kani::use_stub_set(small_native_gate)]
fn public_small_logical_and_width_1_by_2_matches_reference_semantics() {
    public_small_logical_and_pair::<1, 2>();
}

#[kani::proof]
#[kani::unwind(4)]
#[kani::use_stub_set(small_native_gate)]
fn public_small_logical_and_width_1_by_3_matches_reference_semantics() {
    public_small_logical_and_pair::<1, 3>();
}

#[kani::proof]
#[kani::unwind(4)]
#[kani::use_stub_set(small_native_gate)]
fn public_small_logical_and_width_2_by_1_matches_reference_semantics() {
    public_small_logical_and_pair::<2, 1>();
}

#[kani::proof]
#[kani::unwind(4)]
#[kani::use_stub_set(small_native_gate)]
fn public_small_logical_and_width_2_by_2_matches_reference_semantics() {
    public_small_logical_and_pair::<2, 2>();
}

#[kani::proof]
#[kani::unwind(4)]
#[kani::use_stub_set(small_native_gate)]
fn public_small_logical_and_width_2_by_3_matches_reference_semantics() {
    public_small_logical_and_pair::<2, 3>();
}

#[kani::proof]
#[kani::unwind(4)]
#[kani::use_stub_set(small_native_gate)]
fn public_small_logical_and_width_3_by_1_matches_reference_semantics() {
    public_small_logical_and_pair::<3, 1>();
}

#[kani::proof]
#[kani::unwind(4)]
#[kani::use_stub_set(small_native_gate)]
fn public_small_logical_and_width_3_by_2_matches_reference_semantics() {
    public_small_logical_and_pair::<3, 2>();
}

#[kani::proof]
#[kani::unwind(4)]
#[kani::use_stub_set(small_native_gate)]
fn public_small_logical_and_width_3_by_3_matches_reference_semantics() {
    public_small_logical_and_pair::<3, 3>();
}
