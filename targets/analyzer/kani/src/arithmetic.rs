//! Normalized, already-sized four-state operands at widths 1..=64. Checks modular
//! add/subtract, negation, unary-plus expansion, and metadata. Public single-call
//! add/subtract/negate proofs use a fresh cache, normalized source widths 1..=3,
//! independent source signs and caller sign, and natural/natural+2 result widths
//! (1..=5). They check
//! complete u128 getters, storage variant, metadata, and input preservation.
//! The one-bit public dispatch check also retains unary-plus integration.
//! Shared-cache sequences, wider public operations, and traversal remain executable checks.
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

fn prepared_public(input: Operand, width: usize, signed: bool) -> (u128, u128) {
    veryl_formal_semantics::resize::reference_planes(width, |position| {
        veryl_formal_semantics::resize::extended_logic_bit(
            input.planes,
            input.width,
            width,
            input.signed,
            signed,
            position,
        )
    })
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
    let ap = prepared_public(ar, width, signed);
    let bp = if binary {
        prepared_public(br, width, signed)
    } else {
        (0, 0)
    };
    let expected = arithmetic_planes(ap, bp, width, kind);
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
            && planes == expected,
        "public arithmetic semantics mismatch"
    );
    assert_public_input(&a, ar);
    if binary {
        assert_public_input(&b, br);
    }
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
