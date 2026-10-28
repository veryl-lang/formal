//! Normalized, already-sized four-state operands at widths 1..=64. Checks modular
//! add/subtract, negation, unary-plus expansion, and metadata. Public single-call
//! proofs use a fresh cache, normalized source widths 1..=3,
//! independent source signs and caller sign, and natural/natural+2 result widths
//! (1..=5). They check complete u128 getters, storage variant, metadata, and input
//! preservation through arbitrary getter-bit observations at all 128 positions.
//! The one-bit public dispatcher also checks raw native result fields.
//! The initial natural-width unsigned Add/Sub pair also has a shared-cache proof.
//! Longer sequences, wider public operations, and traversal remain executable checks.
//! X/Z unary plus checks the Table 11-6 identity interpretation documented in
//! veryl_formal_semantics::arithmetic; it does not resolve the conflicting general X/Z sentence.

use veryl_analyzer::ir::Op;
use veryl_analyzer::value::{MaskCache, Value, ValueU64};
use veryl_formal_semantics::arithmetic::{Arithmetic, arithmetic_bit, arithmetic_planes};
use veryl_formal_semantics::comparison::Operand;
use veryl_formal_semantics::logic4::read_logic;

#[kani::proof]
#[kani::unwind(2)]
fn public_arithmetic_one_bit_matches_reference_semantics() {
    fn bit() -> ValueU64 {
        ValueU64 {
            payload: u64::from(kani::any::<bool>()),
            mask_xz: u64::from(kani::any::<bool>()),
            width: 1,
            signed: kani::any(),
        }
    }
    let a = bit();
    let b = bit();
    let signed: bool = kani::any();
    let mode = kani::any::<u8>() & 3;
    let kind = match mode {
        0 => Arithmetic::Add,
        1 => Arithmetic::Subtract,
        2 => Arithmetic::Negate,
        _ => Arithmetic::Plus,
    };
    let expected = arithmetic_planes(
        (a.payload as u128, a.mask_xz as u128),
        (b.payload as u128, b.mask_xz as u128),
        1,
        kind,
    );
    let expected_signed = if mode < 2 { signed } else { a.signed };
    let a = Value::U64(a);
    let b = Value::U64(b);
    let mut cache = MaskCache::default();
    let output = match mode {
        0 => Op::Add.eval_value_binary(&a, &b, 1, signed, &mut cache),
        1 => Op::Sub.eval_value_binary(&a, &b, 1, signed, &mut cache),
        2 => Op::Sub.eval_value_unary(&a, 1, signed, &mut cache),
        _ => Op::Add.eval_value_unary(&a, 1, signed, &mut cache),
    };
    let Value::U64(output) = output else {
        panic!("unexpected BigUint result");
    };
    assert!(
        (output.payload as u128, output.mask_xz as u128) == expected,
        "public arithmetic semantics mismatch"
    );
    assert!(
        output.width == 1 && output.signed == expected_signed,
        "public arithmetic semantics mismatch"
    );
    kani::cover!(mode == 0, "public addition is reachable");
    kani::cover!(mode == 1, "public subtraction is reachable");
    kani::cover!(mode == 2, "public negation is reachable");
    kani::cover!(mode == 3, "public unary plus is reachable");
    kani::cover!(
        mode == 3 && output.payload == 1 && output.mask_xz == 1,
        "public plus preserves Z"
    );
}

// BigUint conversion/cloning/masks, heap allocation, and mask-cache access are
// forbidden in these small native proofs. Each replacement panics, proving it unreachable
// for every admitted input. Arithmetic, expansion, constructors, and getters run
// unchanged. Kani 0.68 otherwise expands unreachable BigUint paths before solving.
fn reject_arithmetic_biguint(_: u128) -> num_bigint::BigUint {
    panic!("public arithmetic semantics mismatch");
}
fn reject_arithmetic_cache(_: &mut MaskCache, _: usize) -> &num_bigint::BigUint {
    panic!("public arithmetic semantics mismatch");
}
fn reject_arithmetic_from_u64(_: u64) -> num_bigint::BigUint {
    panic!("public arithmetic semantics mismatch");
}
fn reject_arithmetic_clone(
    _: &veryl_analyzer::value::ValueBigUint,
) -> veryl_analyzer::value::ValueBigUint {
    panic!("public arithmetic semantics mismatch");
}
fn reject_arithmetic_mask(_: usize) -> num_bigint::BigUint {
    panic!("public arithmetic semantics mismatch");
}
unsafe fn reject_arithmetic_allocation(_: std::alloc::Layout) -> *mut u8 {
    panic!("public arithmetic semantics mismatch");
}

kani::stub_set!(
    small_native_arithmetic,
    stub(
        veryl_analyzer::value::u128_to_biguint,
        reject_arithmetic_biguint
    ),
    stub(MaskCache::get, reject_arithmetic_cache),
    stub(
        <num_bigint::BigUint as std::convert::From<u64>>::from,
        reject_arithmetic_from_u64
    ),
    stub(
        <veryl_analyzer::value::ValueBigUint as std::clone::Clone>::clone,
        reject_arithmetic_clone
    ),
    stub(
        veryl_analyzer::value::ValueBigUint::gen_mask,
        reject_arithmetic_mask
    ),
    stub(std::alloc::alloc, reject_arithmetic_allocation),
    stub(std::alloc::alloc_zeroed, reject_arithmetic_allocation),
);

fn assert_public_input(value: &Value, before: Operand) {
    assert!(
        matches!(value, Value::U64(_))
            && value.width() == before.width
            && value.signed() == before.signed
            && value.payload_u128() == before.planes.0
            && value.mask_xz_u128() == before.planes.1,
        "public arithmetic semantics mismatch"
    );
}

fn public_input() -> (ValueU64, Operand) {
    let native_width = u32::from(kani::any::<u8>() & 3).min(2) + 1;
    let mask = u64::MAX >> (64 - native_width);
    let before = Operand {
        planes: (
            u128::from(u64::from(kani::any::<u8>()) & mask),
            u128::from(u64::from(kani::any::<u8>()) & mask),
        ),
        width: native_width as usize,
        signed: kani::any(),
    };
    let width = before.width;
    let value = Value::from_u128(before.planes.0, before.planes.1, width, before.signed);
    assert_public_input(&value, before);
    let Value::U64(native) = value else {
        panic!("public arithmetic semantics mismatch");
    };
    (native, before)
}

// MODE fixes the public operation before symbolic execution. A symbolic Op value
// makes Kani expand unrelated dispatcher arms. Input widths and flags still vary.
fn public_small_arithmetic<const MODE: u8>() {
    let (a, ar) = public_input();
    let (b, br) = if MODE < 2 {
        public_input()
    } else {
        (
            ValueU64 {
                payload: 0,
                mask_xz: 0,
                width: 1,
                signed: false,
            },
            Operand {
                planes: (0, 0),
                width: 1,
                signed: false,
            },
        )
    };
    let a = Value::U64(a);
    let b = Value::U64(b);
    let binary = MODE < 2;
    let natural = if binary {
        ar.width.max(br.width)
    } else {
        ar.width
    };
    let grown: bool = kani::any();
    let width = natural + if grown { 2 } else { 0 };
    let signed: bool = kani::any();
    let position = usize::from(kani::any::<u8>() & 127);
    let mut cache = MaskCache::default();
    let actual = match MODE {
        0 => Op::Add.eval_value_binary(&a, &b, width, signed, &mut cache),
        1 => Op::Sub.eval_value_binary(&a, &b, width, signed, &mut cache),
        2 => Op::Sub.eval_value_unary(&a, width, signed, &mut cache),
        _ => panic!("invalid arithmetic proof mode"),
    };
    let kind = match MODE {
        0 => Arithmetic::Add,
        1 => Arithmetic::Subtract,
        2 => Arithmetic::Negate,
        _ => panic!("invalid arithmetic proof mode"),
    };
    let expected = veryl_formal_semantics::arithmetic::contextual_arithmetic_bit(
        ar, br, width, signed, kind, position,
    );
    let expected_signed = if binary {
        signed
    } else {
        ar.signed && (!grown || signed)
    };
    let planes = (actual.payload_u128(), actual.mask_xz_u128());
    assert!(
        matches!(actual, Value::U64(_))
            && actual.width() == width
            && actual.signed() == expected_signed
            && read_logic(planes.0, planes.1, position) == expected,
        "public arithmetic semantics mismatch"
    );
    assert_public_input(&a, ar);
    if binary {
        assert_public_input(&b, br);
    }
    kani::cover!(
        position == 127,
        "public arithmetic u128 padding is reachable"
    );
    kani::cover!(true, "public arithmetic operation is reachable");
    kani::cover!(
        ar.width == 1,
        "public arithmetic source width 1 is reachable"
    );
    kani::cover!(
        ar.width == 2,
        "public arithmetic source width 2 is reachable"
    );
    kani::cover!(
        ar.width == 3,
        "public arithmetic source width 3 is reachable"
    );
    kani::cover!(width == 1, "public arithmetic result width 1 is reachable");
    kani::cover!(width == 5, "public arithmetic result width 5 is reachable");
    kani::cover!(signed, "public arithmetic signed context is reachable");
    kani::cover!(!signed, "public arithmetic unsigned context is reachable");
    kani::cover!(grown, "public arithmetic growth is reachable");
    kani::cover!(!grown, "public arithmetic natural width is reachable");
    kani::cover!(
        ar.planes.0 == 0 && ar.planes.1 != 0,
        "public arithmetic source X is reachable"
    );
    kani::cover!(
        ar.planes.0 & ar.planes.1 != 0,
        "public arithmetic source Z is reachable"
    );
    kani::cover!(
        ar.signed
            && signed
            && grown
            && ar.planes.0 & (1 << (ar.width - 1)) != 0
            && ar.planes.1 == 0,
        "public arithmetic signed extension is reachable"
    );
    kani::cover!(
        ar.signed && !signed && grown,
        "public arithmetic sign suppression is reachable"
    );
}

#[kani::proof]
#[kani::unwind(6)]
#[kani::use_stub_set(small_native_arithmetic)]
fn public_arithmetic_small_add_matches_reference_semantics() {
    public_small_arithmetic::<0>();
}

#[kani::proof]
#[kani::unwind(6)]
#[kani::use_stub_set(small_native_arithmetic)]
fn public_arithmetic_small_subtract_matches_reference_semantics() {
    public_small_arithmetic::<1>();
}

#[kani::proof]
#[kani::unwind(6)]
#[kani::use_stub_set(small_native_arithmetic)]
fn public_arithmetic_small_negate_matches_reference_semantics() {
    public_small_arithmetic::<2>();
}

// SV-UPLUS identity after preparation is one extended source bit. Arbitrary
// observation of all 128 positions checks both getter planes, including padding,
// without packing intermediate vectors. The reference identity is checked over
// every four-state vector at these result widths in the arithmetic reference tests.
#[kani::proof]
#[kani::unwind(1)]
#[kani::use_stub_set(small_native_arithmetic)]
fn public_arithmetic_small_plus_matches_reference_semantics() {
    use veryl_formal_semantics::logic4::Logic4;
    use veryl_formal_semantics::resize::extended_logic_bit;
    let (native, before) = public_input();
    let input = Value::U64(native);
    let grown: bool = kani::any();
    let width = before.width + if grown { 2 } else { 0 };
    let signed: bool = kani::any();
    let position = usize::from(kani::any::<u8>() & 127);
    let mut cache = MaskCache::default();
    let output = Op::Add.eval_value_unary(&input, width, signed, &mut cache);
    let expected = extended_logic_bit(
        before.planes,
        before.width,
        width,
        before.signed,
        signed,
        position,
    );
    assert!(
        matches!(output, Value::U64(_))
            && output.width() == width
            && output.signed() == (before.signed && (!grown || signed))
            && read_logic(output.payload_u128(), output.mask_xz_u128(), position) == expected,
        "public arithmetic semantics mismatch"
    );
    assert_public_input(&input, before);
    kani::cover!(before.width == 1, "public plus source width 1 is reachable");
    kani::cover!(before.width == 2, "public plus source width 2 is reachable");
    kani::cover!(before.width == 3, "public plus source width 3 is reachable");
    kani::cover!(grown, "public plus growth is reachable");
    kani::cover!(!grown, "public plus natural width is reachable");
    kani::cover!(
        signed && before.signed && grown && expected == Logic4::One && position >= before.width,
        "public plus sign extension is reachable"
    );
    kani::cover!(
        !signed && before.signed && grown,
        "public plus sign suppression is reachable"
    );
    kani::cover!(expected == Logic4::X, "public plus preserves X");
    kani::cover!(expected == Logic4::Z, "public plus preserves Z");
    kani::cover!(position == 127, "public plus u128 padding is reachable");
    kani::cover!(
        signed && before.signed && grown && position >= before.width && expected == Logic4::X,
        "public plus extended X is reachable"
    );
    kani::cover!(
        signed && before.signed && grown && position >= before.width && expected == Logic4::Z,
        "public plus extended Z is reachable"
    );
}

// Initial two calls of the executable binary sequence, on the same cache.
// The caller starts at natural width in an unsigned context. Both sources retain
// independent signs and every normalized four-state value at widths 1..=3.
#[kani::proof]
#[kani::unwind(6)]
#[kani::use_stub_set(small_native_arithmetic)]
fn public_arithmetic_initial_pair_matches_reference_semantics() {
    use veryl_formal_semantics::logic4::Logic4;
    let (a, ar) = public_input();
    let (b, br) = public_input();
    let a = Value::U64(a);
    let b = Value::U64(b);
    let width = ar.width.max(br.width);
    let observe_subtract: bool = kani::any();
    let position = usize::from(kani::any::<u8>() & 127);
    let mut cache = MaskCache::default();
    let sum = Op::Add.eval_value_binary(&a, &b, width, false, &mut cache);
    let difference = Op::Sub.eval_value_binary(&a, &b, width, false, &mut cache);
    let (output, kind) = if observe_subtract {
        (difference, Arithmetic::Subtract)
    } else {
        (sum, Arithmetic::Add)
    };
    let expected = veryl_formal_semantics::arithmetic::contextual_arithmetic_bit(
        ar, br, width, false, kind, position,
    );
    assert!(
        matches!(output, Value::U64(_))
            && output.width() == width
            && !output.signed()
            && read_logic(output.payload_u128(), output.mask_xz_u128(), position) == expected,
        "public arithmetic semantics mismatch"
    );
    assert_public_input(&a, ar);
    assert_public_input(&b, br);
    kani::cover!(
        !observe_subtract,
        "arithmetic initial pair observes addition"
    );
    kani::cover!(
        observe_subtract,
        "arithmetic initial pair observes subtraction"
    );
    kani::cover!(
        ar.width == 1,
        "arithmetic initial pair LHS width 1 is reachable"
    );
    kani::cover!(
        ar.width == 2,
        "arithmetic initial pair LHS width 2 is reachable"
    );
    kani::cover!(
        ar.width == 3,
        "arithmetic initial pair LHS width 3 is reachable"
    );
    kani::cover!(
        br.width == 1,
        "arithmetic initial pair RHS width 1 is reachable"
    );
    kani::cover!(
        br.width == 2,
        "arithmetic initial pair RHS width 2 is reachable"
    );
    kani::cover!(
        br.width == 3,
        "arithmetic initial pair RHS width 3 is reachable"
    );
    kani::cover!(
        ar.width > br.width,
        "arithmetic initial pair wider LHS is reachable"
    );
    kani::cover!(
        ar.width < br.width,
        "arithmetic initial pair wider RHS is reachable"
    );
    kani::cover!(
        ar.signed != br.signed,
        "arithmetic initial pair mixed source signs are reachable"
    );
    kani::cover!(
        expected == Logic4::X,
        "arithmetic initial pair X result is reachable"
    );
    kani::cover!(
        position == 127,
        "arithmetic initial pair u128 padding is reachable"
    );
}

fn operand(width: u32) -> ValueU64 {
    let mask = u64::MAX >> (64 - width);
    ValueU64 {
        payload: kani::any::<u64>() & mask,
        mask_xz: kani::any::<u64>() & mask,
        width,
        signed: kani::any(),
    }
}

#[kani::proof]
#[kani::unwind(65)]
fn native_add_sub_matches_reference_semantics() {
    let width = u32::from(kani::any::<u8>() & 63) + 1;
    let a = operand(width);
    let b = operand(width);
    let signed: bool = kani::any();
    let subtract: bool = kani::any();
    let output = a.add_sub(&b, width as usize, signed, subtract);
    let position = usize::from(kani::any::<u8>() & 63);
    let expected = arithmetic_bit(
        (a.payload as u128, a.mask_xz as u128),
        (b.payload as u128, b.mask_xz as u128),
        width as usize,
        if subtract {
            Arithmetic::Subtract
        } else {
            Arithmetic::Add
        },
        position,
    );
    assert!(
        read_logic(output.payload as u128, output.mask_xz as u128, position) == expected,
        "add-sub semantics mismatch"
    );
    assert!(
        output.width == width && output.signed == signed,
        "add-sub semantics mismatch"
    );
    let known = a.mask_xz == 0 && b.mask_xz == 0;
    let mask = u64::MAX >> (64 - width);
    kani::cover!(!subtract, "addition is reachable");
    kani::cover!(subtract, "subtraction is reachable");
    kani::cover!(width == 1, "one-bit arithmetic is reachable");
    kani::cover!(width == 64, "width-64 arithmetic is reachable");
    kani::cover!(signed, "signed arithmetic result is reachable");
    kani::cover!(!signed, "unsigned arithmetic result is reachable");
    kani::cover!(
        known && !subtract && a.payload == mask && b.payload == 1,
        "addition carry wrap is reachable"
    );
    kani::cover!(
        known && width == 64 && !subtract && a.payload == u64::MAX && b.payload == 1,
        "native addition overflow is reachable"
    );
    kani::cover!(
        known && subtract && a.payload == 0 && b.payload == 1,
        "subtraction borrow wrap is reachable"
    );
    kani::cover!(
        known && width == 64 && subtract && a.payload == 0 && b.payload == 1,
        "native subtraction underflow is reachable"
    );
    kani::cover!(
        known && a.payload == 0 && b.payload == 0,
        "zero arithmetic is reachable"
    );
    kani::cover!(
        a.mask_xz != 0 && b.mask_xz == 0,
        "arithmetic unknown LHS is reachable"
    );
    kani::cover!(
        a.mask_xz == 0 && b.mask_xz != 0,
        "arithmetic unknown RHS is reachable"
    );
    kani::cover!(
        (a.payload & a.mask_xz) != 0 || (b.payload & b.mask_xz) != 0,
        "arithmetic source Z is reachable"
    );
}

#[kani::proof]
#[kani::unwind(65)]
fn native_negation_matches_reference_semantics() {
    let width = u32::from(kani::any::<u8>() & 63) + 1;
    let a = operand(width);
    let output = a.negate(width as usize);
    let position = usize::from(kani::any::<u8>() & 63);
    let expected = arithmetic_bit(
        (a.payload as u128, a.mask_xz as u128),
        (0, 0),
        width as usize,
        Arithmetic::Negate,
        position,
    );
    assert!(
        read_logic(output.payload as u128, output.mask_xz as u128, position) == expected,
        "negation semantics mismatch"
    );
    assert!(
        output.width == width && output.signed == a.signed,
        "negation semantics mismatch"
    );
    let known = a.mask_xz == 0;
    let sign = 1u64 << (width - 1);
    kani::cover!(width == 1, "one-bit negation is reachable");
    kani::cover!(width == 64, "width-64 negation is reachable");
    kani::cover!(known && a.payload == 0, "negation of zero is reachable");
    kani::cover!(
        known && width == 64 && a.payload == 0,
        "native zero negation carry is reachable"
    );
    kani::cover!(
        known && a.signed && a.payload == sign,
        "minimum signed negation is reachable"
    );
    kani::cover!(
        known && width == 64 && a.signed && a.payload == sign,
        "native minimum signed negation is reachable"
    );
    kani::cover!(known && a.payload == 1, "negation of one is reachable");
    kani::cover!(
        a.mask_xz != 0 && a.payload == 0,
        "negation source X is reachable"
    );
    kani::cover!(
        (a.mask_xz & a.payload) != 0,
        "negation source Z is reachable"
    );
}

#[kani::proof]
#[kani::unwind(1)]
fn native_unary_plus_matches_reference_semantics() {
    use veryl_formal_semantics::resize::extended_logic_bit;
    let before = operand(u32::from(kani::any::<u8>() & 63) + 1);
    let requested = usize::from(kani::any::<u8>() & 63) + 1;
    let use_sign: bool = kani::any();
    let width = requested.max(before.width as usize);
    let position = usize::from(kani::any::<u8>() & 63);
    let expected = if position < before.width as usize {
        arithmetic_bit(
            (before.payload as u128, before.mask_xz as u128),
            (0, 0),
            before.width as usize,
            Arithmetic::Plus,
            position,
        )
    } else {
        extended_logic_bit(
            (before.payload as u128, before.mask_xz as u128),
            before.width as usize,
            width,
            before.signed,
            use_sign,
            position,
        )
    };
    let output = before.expand(requested, use_sign);
    assert!(
        read_logic(output.payload as u128, output.mask_xz as u128, position) == expected,
        "unary-plus semantics mismatch"
    );
    assert!(
        output.width as usize == width
            && output.signed == (before.signed && (requested <= before.width as usize || use_sign)),
        "unary-plus semantics mismatch"
    );
    kani::cover!(before.width == 1, "one-bit unary plus is reachable");
    kani::cover!(width == 64, "width-64 unary plus is reachable");
    kani::cover!(
        requested > before.width as usize,
        "unary plus growth is reachable"
    );
    kani::cover!(
        requested == before.width as usize,
        "unary plus equal width is reachable"
    );
    kani::cover!(
        requested < before.width as usize,
        "unary plus smaller request is reachable"
    );
    kani::cover!(
        before.mask_xz != 0
            && before.mask_xz != (u64::MAX >> (64 - before.width))
            && before.payload == 0,
        "unary plus preserves partial X"
    );
    kani::cover!(
        (before.mask_xz & before.payload) != 0,
        "unary plus preserves Z"
    );
    kani::cover!(
        before.signed && !use_sign && requested > before.width as usize,
        "unary plus disabled sign is reachable"
    );
    kani::cover!(
        before.signed
            && use_sign
            && requested > before.width as usize
            && (before.mask_xz & before.payload) & (1u64 << (before.width - 1)) != 0,
        "unary plus extended Z is reachable"
    );
}

// The first two unary calls share the real cache. Sources are normalized at
// widths 1..=3, with arbitrary four-state planes and signs. A natural-width
// unsigned caller leaves the source sign on both results (IEEE 1800 11.8.2).
// The preceding plus output is covered by public_arithmetic_small_plus.
// Only the second result is observed here; the first call still executes.
#[kani::proof]
#[kani::unwind(6)]
#[kani::use_stub_set(small_native_arithmetic)]
fn public_arithmetic_negate_after_plus_matches_reference_semantics() {
    use veryl_formal_semantics::logic4::Logic4;
    let (native, before) = public_input();
    let input = Value::U64(native);
    let width = before.width;
    let position = usize::from(kani::any::<u8>() & 127);
    let mut cache = MaskCache::default();
    let _ = Op::Add.eval_value_unary(&input, width, false, &mut cache);
    let output = Op::Sub.eval_value_unary(&input, width, false, &mut cache);
    let expected = veryl_formal_semantics::arithmetic::contextual_arithmetic_bit(
        before,
        before,
        width,
        false,
        Arithmetic::Negate,
        position,
    );
    assert!(
        matches!(output, Value::U64(_))
            && output.width() == width
            && output.signed() == before.signed
            && read_logic(output.payload_u128(), output.mask_xz_u128(), position) == expected,
        "public arithmetic semantics mismatch"
    );
    assert_public_input(&input, before);
    kani::cover!(width == 1, "unary initial pair width 1 is reachable");
    kani::cover!(width == 2, "unary initial pair width 2 is reachable");
    kani::cover!(width == 3, "unary initial pair width 3 is reachable");
    kani::cover!(
        before.signed,
        "unary initial pair signed source is reachable"
    );
    kani::cover!(
        !before.signed,
        "unary initial pair unsigned source is reachable"
    );
    kani::cover!(
        expected == Logic4::X,
        "unary initial pair X result is reachable"
    );
    kani::cover!(
        position == 127,
        "unary initial pair u128 padding is reachable"
    );
}

// Continue the executable unary trace at natural width: unsigned Plus/Negate,
// then signed Plus/Negate. Each proof observes its own result after executing
// every preceding call on the same cache. Earlier outputs have separate proofs.
// Sources span all normalized four-state values and both signs at widths 1..=3.
// Result metadata follows 11.8.2; unary plus uses the model's Table 11-6 reading.
fn public_unary_signed_after_initial<const NEGATE: bool>() {
    use veryl_formal_semantics::logic4::Logic4;
    let (native, before) = public_input();
    let input = Value::U64(native);
    let width = before.width;
    let position = usize::from(kani::any::<u8>() & 127);
    let mut cache = MaskCache::default();
    let _ = Op::Add.eval_value_unary(&input, width, false, &mut cache);
    let _ = Op::Sub.eval_value_unary(&input, width, false, &mut cache);
    let plus = Op::Add.eval_value_unary(&input, width, true, &mut cache);
    let output = if NEGATE {
        Op::Sub.eval_value_unary(&input, width, true, &mut cache)
    } else {
        plus
    };
    assert!(
        matches!(output, Value::U64(_))
            && output.width() == width
            && output.signed() == before.signed,
        "public arithmetic semantics mismatch"
    );
    if NEGATE {
        let expected = veryl_formal_semantics::arithmetic::contextual_arithmetic_bit(
            before,
            before,
            width,
            true,
            Arithmetic::Negate,
            position,
        );
        assert!(
            read_logic(output.payload_u128(), output.mask_xz_u128(), position) == expected,
            "public arithmetic semantics mismatch"
        );
        kani::cover!(
            expected == Logic4::X,
            "unary signed sequence X result is reachable"
        );
        kani::cover!(
            expected == Logic4::One,
            "unary signed sequence known one is reachable"
        );
        kani::cover!(
            position == 127,
            "unary signed sequence u128 padding is reachable"
        );
    } else {
        // At unchanged width, the model's unary-plus identity is the source
        // planes themselves. Equality checks padding through bit 127 as well.
        assert!(
            output.payload_u128() == before.planes.0 && output.mask_xz_u128() == before.planes.1,
            "public arithmetic semantics mismatch"
        );
        kani::cover!(
            before.planes.1 & !before.planes.0 != 0,
            "unary signed sequence X result is reachable"
        );
        kani::cover!(
            before.planes.1 == 0 && before.planes.0 != 0,
            "unary signed sequence known one is reachable"
        );
        kani::cover!(
            before.planes.1 & before.planes.0 != 0,
            "unary signed sequence Z result is reachable"
        );
    }
    assert_public_input(&input, before);
    kani::cover!(width == 1, "unary signed sequence width 1 is reachable");
    kani::cover!(width == 2, "unary signed sequence width 2 is reachable");
    kani::cover!(width == 3, "unary signed sequence width 3 is reachable");
    kani::cover!(
        before.signed,
        "unary signed sequence signed source is reachable"
    );
    kani::cover!(
        !before.signed,
        "unary signed sequence unsigned source is reachable"
    );
}

#[kani::proof]
#[kani::unwind(1)]
#[kani::use_stub_set(small_native_arithmetic)]
fn public_arithmetic_plus_after_unsigned_pair_matches_reference_semantics() {
    public_unary_signed_after_initial::<false>();
}

#[kani::proof]
#[kani::unwind(6)]
#[kani::use_stub_set(small_native_arithmetic)]
fn public_arithmetic_negate_after_signed_plus_matches_reference_semantics() {
    public_unary_signed_after_initial::<true>();
}

// Observe calls 5 and 6 of the executable trace after the four natural-width
// calls. Both use width + 2 and an unsigned caller, so preparation zero-extends
// even signed sources (IEEE 1800 11.8.2). Sources span normalized four-state
// planes and both signs at widths 1..=3; results have widths 3..=5.
// Unary plus uses the model's Table 11-6 identity interpretation.
fn public_unary_widened_unsigned<const NEGATE: bool>() {
    use veryl_formal_semantics::logic4::Logic4;
    let (native, before) = public_input();
    let input = Value::U64(native);
    let natural = before.width;
    let width = natural + 2;
    let mut cache = MaskCache::default();
    let _ = Op::Add.eval_value_unary(&input, natural, false, &mut cache);
    let _ = Op::Sub.eval_value_unary(&input, natural, false, &mut cache);
    let _ = Op::Add.eval_value_unary(&input, natural, true, &mut cache);
    let _ = Op::Sub.eval_value_unary(&input, natural, true, &mut cache);
    let plus = Op::Add.eval_value_unary(&input, width, false, &mut cache);
    let output = if NEGATE {
        Op::Sub.eval_value_unary(&input, width, false, &mut cache)
    } else {
        plus
    };
    // Prove the native dispatch before observing its getters, avoiding expansion
    // of getter branches for an unreachable BigUint representation.
    let Value::U64(native) = output else {
        panic!("public arithmetic semantics mismatch");
    };
    let output = Value::U64(native);
    assert!(
        output.width() == width && !output.signed(),
        "public arithmetic semantics mismatch"
    );
    if NEGATE {
        let position = usize::from(kani::any::<u8>() & 127);
        let expected = veryl_formal_semantics::arithmetic::contextual_arithmetic_bit(
            before,
            before,
            width,
            false,
            Arithmetic::Negate,
            position,
        );
        assert!(
            read_logic(output.payload_u128(), output.mask_xz_u128(), position) == expected,
            "public arithmetic semantics mismatch"
        );
        kani::cover!(
            expected == Logic4::X,
            "unary widened sequence X result is reachable"
        );
        kani::cover!(
            expected == Logic4::One,
            "unary widened sequence known one is reachable"
        );
        kani::cover!(
            position == 127,
            "unary widened sequence u128 padding is reachable"
        );
    } else {
        // Zero extension leaves both normalized source planes numerically intact.
        // Full getter equality includes the two added bits and padding to bit 127.
        assert!(
            output.payload_u128() == before.planes.0 && output.mask_xz_u128() == before.planes.1,
            "public arithmetic semantics mismatch"
        );
        kani::cover!(
            before.planes.1 & !before.planes.0 != 0,
            "unary widened sequence X result is reachable"
        );
        kani::cover!(
            before.planes.1 == 0 && before.planes.0 != 0,
            "unary widened sequence known one is reachable"
        );
        kani::cover!(
            before.planes.1 & before.planes.0 != 0,
            "unary widened sequence Z result is reachable"
        );
    }
    assert_public_input(&input, before);
    kani::cover!(
        before.width == 1,
        "unary widened sequence source width 1 is reachable"
    );
    kani::cover!(
        before.width == 2,
        "unary widened sequence source width 2 is reachable"
    );
    kani::cover!(
        before.width == 3,
        "unary widened sequence source width 3 is reachable"
    );
    kani::cover!(
        width == 3,
        "unary widened sequence result width 3 is reachable"
    );
    kani::cover!(
        width == 4,
        "unary widened sequence result width 4 is reachable"
    );
    kani::cover!(
        width == 5,
        "unary widened sequence result width 5 is reachable"
    );
    kani::cover!(
        before.signed,
        "unary widened sequence signed source is reachable"
    );
    kani::cover!(
        !before.signed,
        "unary widened sequence unsigned source is reachable"
    );
    kani::cover!(
        before.signed && before.planes.1 == 0 && before.planes.0 & (1 << (natural - 1)) != 0,
        "unary widened sequence suppresses sign extension"
    );
}

#[kani::proof]
#[kani::unwind(1)]
#[kani::use_stub_set(small_native_arithmetic)]
fn public_arithmetic_unsigned_growth_plus_matches_reference_semantics() {
    public_unary_widened_unsigned::<false>();
}

#[kani::proof]
#[kani::unwind(6)]
#[kani::use_stub_set(small_native_arithmetic)]
fn public_arithmetic_unsigned_growth_negate_matches_reference_semantics() {
    public_unary_widened_unsigned::<true>();
}

// Observe calls 7 and 8 after the natural-width and unsigned widened pairs on
// the same cache. Sources cover normalized widths 1..=3, both signs and every
// four-state plane; results have width + 2. The signed caller sign-extends signed
// sources, and zero-extends unsigned sources (IEEE 1800 11.8.2).
// Unary plus follows the model's documented Table 11-6 identity interpretation.
fn public_unary_widened_signed<const NEGATE: bool>() {
    use veryl_formal_semantics::logic4::Logic4;
    let (native, before) = public_input();
    let input = Value::U64(native);
    let natural = before.width;
    let width = natural + 2;
    let position = usize::from(kani::any::<u8>() & 127);
    let mut cache = MaskCache::default();
    let _ = Op::Add.eval_value_unary(&input, natural, false, &mut cache);
    let _ = Op::Sub.eval_value_unary(&input, natural, false, &mut cache);
    let _ = Op::Add.eval_value_unary(&input, natural, true, &mut cache);
    let _ = Op::Sub.eval_value_unary(&input, natural, true, &mut cache);
    let _ = Op::Add.eval_value_unary(&input, width, false, &mut cache);
    let _ = Op::Sub.eval_value_unary(&input, width, false, &mut cache);
    let plus = Op::Add.eval_value_unary(&input, width, true, &mut cache);
    let output = if NEGATE {
        Op::Sub.eval_value_unary(&input, width, true, &mut cache)
    } else {
        plus
    };
    let Value::U64(native) = output else {
        panic!("public arithmetic semantics mismatch");
    };
    let output = Value::U64(native);
    let expected = if NEGATE {
        veryl_formal_semantics::arithmetic::contextual_arithmetic_bit(
            before,
            before,
            width,
            true,
            Arithmetic::Negate,
            position,
        )
    } else {
        veryl_formal_semantics::resize::extended_logic_bit(
            before.planes,
            natural,
            width,
            before.signed,
            true,
            position,
        )
    };
    assert!(
        output.width() == width
            && output.signed() == before.signed
            && read_logic(output.payload_u128(), output.mask_xz_u128(), position) == expected,
        "public arithmetic semantics mismatch"
    );
    assert_public_input(&input, before);
    kani::cover!(
        natural == 1,
        "unary signed growth source width 1 is reachable"
    );
    kani::cover!(
        natural == 2,
        "unary signed growth source width 2 is reachable"
    );
    kani::cover!(
        natural == 3,
        "unary signed growth source width 3 is reachable"
    );
    kani::cover!(
        width == 3,
        "unary signed growth result width 3 is reachable"
    );
    kani::cover!(
        width == 4,
        "unary signed growth result width 4 is reachable"
    );
    kani::cover!(
        width == 5,
        "unary signed growth result width 5 is reachable"
    );
    kani::cover!(
        before.signed,
        "unary signed growth signed source is reachable"
    );
    kani::cover!(
        !before.signed,
        "unary signed growth unsigned source is reachable"
    );
    kani::cover!(
        before.signed && before.planes.1 == 0 && before.planes.0 & (1 << (natural - 1)) != 0,
        "unary signed growth negative signed source is reachable"
    );
    kani::cover!(
        expected == Logic4::X,
        "unary signed growth X result is reachable"
    );
    kani::cover!(
        expected == Logic4::One,
        "unary signed growth known one is reachable"
    );
    kani::cover!(
        position == 127,
        "unary signed growth u128 padding is reachable"
    );
    if !NEGATE {
        kani::cover!(
            before.signed && position >= natural && position < width && expected == Logic4::One,
            "unary signed growth extends one"
        );
        kani::cover!(
            before.signed && position >= natural && position < width && expected == Logic4::X,
            "unary signed growth extends X"
        );
        kani::cover!(
            before.signed && position >= natural && position < width && expected == Logic4::Z,
            "unary signed growth extends Z"
        );
        kani::cover!(
            !before.signed && position >= natural && position < width && expected == Logic4::Zero,
            "unary signed growth zero-extends unsigned source"
        );
    }
}

#[kani::proof]
#[kani::unwind(1)]
#[kani::use_stub_set(small_native_arithmetic)]
fn public_arithmetic_signed_growth_plus_matches_reference_semantics() {
    public_unary_widened_signed::<false>();
}

#[kani::proof]
#[kani::unwind(6)]
#[kani::use_stub_set(small_native_arithmetic)]
fn public_arithmetic_signed_growth_negate_matches_reference_semantics() {
    public_unary_widened_signed::<true>();
}
