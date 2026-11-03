//! IEEE 1800-2023 11.4.4/11.8.2: prepared 65..128-bit relational operators.
//! The known-unsigned, known-signed, and unknown-input partitions collectively
//! cover all normalized four-state planes, independent operand flags, resolved signed
//! comparison mode, and result widths1..128. ValueBigUint::to_bigint is proved
//! separately against signed i128 decoding;
//! kernels use that summary under the numeric BigUint/BigInt API contract. The
//! scalar bit fold is separate from sign/magnitude numeric ordering. No allocator
//! correctness, arbitrary widths, expression effects, or float theorem follows.
use super::*;
use num_bigint::{BigInt, BigUint, Sign};
use veryl_formal_semantics::bits256::Signed256;

const SIGNED_CAP: usize = 8;
static mut SIGNED_VALUES: [Signed256; SIGNED_CAP] = [Signed256::ZERO; SIGNED_CAP];
static mut SIGNED_NEXT: usize = 1;

pub(super) fn reset() {
    unsafe {
        SIGNED_NEXT = 1;
    }
}
pub(super) fn unsigned_cmp(a: &BigUint, b: &BigUint) -> std::cmp::Ordering {
    get(a).unsigned_cmp(get(b))
}
pub(super) fn from_biguint(sign: Sign, value: BigUint) -> BigInt {
    let magnitude = if sign == Sign::NoSign {
        N::ZERO
    } else {
        get(&value)
    };
    let value = Signed256::new(magnitude, sign == Sign::Minus);
    unsafe {
        let i = SIGNED_NEXT;
        assert!(i < SIGNED_CAP, "signed numeric model domain exceeded");
        SIGNED_VALUES[i] = value;
        SIGNED_NEXT += 1;
        BigInt::new_const(((TAG & 0x7fff0000) | i as u32) as i32)
    }
}
fn signed_value(value: &BigInt) -> Signed256 {
    let mut digits = value.magnitude().iter_u64_digits();
    let raw = digits.next().unwrap_or(0);
    unsafe {
        let i = (raw & 0xffff) as usize;
        assert!(
            value.sign() == Sign::Plus
                && digits.next().is_none()
                && raw >> 16 == u64::from((TAG & 0x7fff0000) >> 16)
                && i > 0
                && i < SIGNED_NEXT
                && i < SIGNED_CAP,
            "signed numeric model domain exceeded"
        );
        SIGNED_VALUES[i]
    }
}
pub(super) fn signed_cmp(a: &BigInt, b: &BigInt) -> std::cmp::Ordering {
    signed_value(a).compare(signed_value(b))
}
pub(super) fn signed_eq(a: &BigInt, b: &BigInt) -> bool {
    signed_value(a) == signed_value(b)
}

// A scalar order definition: each more significant differing magnitude bit
// overrides lower bits; negative magnitudes have negative bit weights.
fn scalar_order(a: N, b: N, negative_a: bool, negative_b: bool) -> std::cmp::Ordering {
    use std::cmp::Ordering::*;
    let mut order = Equal;
    let negative_a = negative_a && (a.lo != 0 || a.hi != 0);
    let negative_b = negative_b && (b.lo != 0 || b.hi != 0);
    for i in 0..256 {
        let x = a.bit(i);
        let y = b.bit(i);
        if x != y {
            order = if x != negative_a { Greater } else { Less };
        }
    }
    let na = negative_a;
    let nb = negative_b;
    if na != nb {
        if na { Less } else { Greater }
    } else {
        order
    }
}

#[kani::proof]
#[kani::unwind(257)]
#[kani::use_stub_set(numeric256)]
fn numeric256_ordering_api_matches_scalar_order() {
    init();
    let a = N {
        lo: kani::any(),
        hi: kani::any(),
    };
    let b = N {
        lo: kani::any(),
        hi: kani::any(),
    };
    let sa = [Sign::NoSign, Sign::Plus, Sign::Minus][usize::from(kani::any::<u8>() % 3)];
    let sb = [Sign::NoSign, Sign::Plus, Sign::Minus][usize::from(kani::any::<u8>() % 3)];
    let x = put(a);
    let y = put(b);
    let unsigned = scalar_order(a, b, false, false);
    assert!(
        x.cmp(&y) == unsigned && (x < y) == unsigned.is_lt() && (x >= y) == unsigned.is_ge(),
        "wide ordering numeric API mismatch"
    );
    let x = BigInt::from_biguint(sa, x);
    let y = BigInt::from_biguint(sb, y);
    let aa = if sa == Sign::NoSign { N::ZERO } else { a };
    let bb = if sb == Sign::NoSign { N::ZERO } else { b };
    let expected = scalar_order(aa, bb, sa == Sign::Minus, sb == Sign::Minus);
    assert!(
        x.cmp(&y) == expected
            && (x < y) == expected.is_lt()
            && (x <= y) == expected.is_le()
            && (x > y) == expected.is_gt()
            && (x >= y) == expected.is_ge()
            && (x == y) == expected.is_eq(),
        "wide ordering numeric API mismatch"
    );
    let x = Some(x);
    let y = Some(y);
    assert!(
        (x < y) == expected.is_lt()
            && (x <= y) == expected.is_le()
            && (x > y) == expected.is_gt()
            && (x >= y) == expected.is_ge(),
        "wide ordering numeric API mismatch"
    );
    kani::cover!(
        a.lo == b.lo && a.hi != b.hi,
        "ordering API high half difference is reachable"
    );
    kani::cover!(
        sa == Sign::Minus && sb == Sign::Minus && aa != bb,
        "ordering API negative magnitudes are reachable"
    );
    kani::cover!(
        a == N::ZERO && sa == Sign::Minus,
        "ordering API negative zero normalization is reachable"
    );
    kani::cover!(
        sa == Sign::NoSign && a != N::ZERO,
        "ordering API no-sign normalization is reachable"
    );
}

// This summary is used only after the separate conversion theorem below. It
// decodes two's-complement bits through native signed arithmetic, independently
// of production's complement/add/mask conversion sequence.
pub(super) fn converted(value: &ValueBigUint) -> Option<BigInt> {
    let w = value.width as usize;
    assert!(
        (65..=128).contains(&w),
        "signed conversion summary domain exceeded"
    );
    let payload = get(value.payload());
    let unknown = get(value.mask_xz());
    assert!(
        payload.hi == 0
            && unknown.hi == 0
            && (payload.lo | unknown.lo) & !(u128::MAX >> (128 - w)) == 0,
        "signed conversion summary domain exceeded"
    );
    if unknown != N::ZERO {
        return None;
    }
    let shift = 128 - w;
    let decoded = ((payload.lo << shift) as i128) >> shift;
    let sign = if decoded < 0 { Sign::Minus } else { Sign::Plus };
    Some(from_biguint(
        sign,
        put(N::from_u128(decoded.unsigned_abs())),
    ))
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
fn wide_bigint_conversion_matches_signed_decoding() {
    init();
    let w = usize::from(kani::any::<u8>() & 63) + 65;
    let (value, original) = super::followup::wide_input(w);
    let Value::BigUint(ref value_big) = value else {
        unreachable!()
    };
    let actual = value_big.to_bigint();
    let expected = converted(value_big);
    match (actual, expected) {
        (None, None) => {}
        (Some(a), Some(b)) => assert!(
            signed_value(&a) == signed_value(&b),
            "wide operator semantics mismatch"
        ),
        _ => panic!("wide operator semantics mismatch"),
    }
    preserve(&value, original);
    kani::cover!(
        w == 128 && original.planes.1 == 0 && original.planes.0 == 1u128 << 127,
        "wide signed conversion minimum is reachable"
    );
    kani::cover!(
        w == 65 && original.planes.1 == 0 && original.planes.0 == 0,
        "wide signed conversion zero is reachable"
    );
    kani::cover!(
        w == 128 && original.planes.1 >> 64 != 0,
        "wide signed conversion upper unknown is reachable"
    );
    kani::cover!(
        w == 65 && original.planes.1 == 0 && original.planes.0 == 1,
        "wide signed conversion positive is reachable"
    );
}

// Construct the entire known partition directly, rather than asking the solver
// to discover that arbitrary symbolic mask planes equal zero.
fn known_input(w: usize) -> (Value, Operand) {
    let a = Operand {
        planes: (kani::any::<u128>() & (u128::MAX >> (128 - w)), 0),
        width: w,
        signed: kani::any(),
    };
    let v = Value::BigUint(ValueBigUint {
        payload: Box::new(from128(a.planes.0)),
        mask_xz: Box::new(BigUint::ZERO),
        width: w as u32,
        signed: a.signed,
    });
    (v, a)
}
static mut ARBITRARY_ORDER_CALLS: usize = 0;

fn ordering<const K: u8, const MODE: u8>() {
    use veryl_formal_semantics::comparison::{Comparison, compared_logic};
    init();
    let w = usize::from(kani::any::<u8>() & 63) + 65;
    let out = usize::from(kani::any::<u8>() & 127) + 1;
    let sign: bool = if MODE == 2 { kani::any() } else { MODE == 1 };
    let (x, a) = if MODE == 2 {
        super::followup::wide_input(w)
    } else {
        known_input(w)
    };
    let (y, b) = if MODE == 2 {
        super::followup::wide_input(w)
    } else {
        known_input(w)
    };
    unsafe {
        ARBITRARY_ORDER_CALLS = 0;
    }
    let has_unknown = (a.planes.1 | b.planes.1) != 0;
    kani::assume(has_unknown == (MODE == 2));
    let (op, kind) = match K {
        0 => (Op::Less, Comparison::Less),
        1 => (Op::LessEq, Comparison::LessEqual),
        2 => (Op::Greater, Comparison::Greater),
        _ => (Op::GreaterEq, Comparison::GreaterEqual),
    };
    let ret = op.eval_value_binary(&x, &y, out, sign, &mut MaskCache::default());
    if MODE == 2 && !sign {
        assert!(
            unsafe { ARBITRARY_ORDER_CALLS } == 1,
            "unknown ordering abstraction path mismatch"
        );
    }
    // The kernel receives a resolved mode; its stored operand flags are independent.
    let expected = if MODE == 2 {
        Logic4::X
    } else {
        compared_logic(
            Operand { signed: sign, ..a },
            Operand { signed: sign, ..b },
            kind,
        )
    };
    let planes = match expected {
        Logic4::Zero => (0, 0),
        Logic4::One => (1, 0),
        Logic4::X => (0, 1),
        Logic4::Z => unreachable!(),
    };
    assert!(
        (ret.payload_u128(), ret.mask_xz_u128()) == planes,
        "wide operator semantics mismatch"
    );
    result(&ret, out, false, 0, expected);
    preserve(&x, a);
    preserve(&y, b);
    if MODE == 2 {
        kani::cover!(
            w == 128 && (a.planes.1 | b.planes.1) >> 64 != 0,
            "wide ordering upper unknown is reachable"
        );
        kani::cover!(
            a.planes.1 != 0 && b.planes.1 == 0,
            "wide ordering only lhs unknown is reachable"
        );
        kani::cover!(
            a.planes.1 == 0 && b.planes.1 != 0,
            "wide ordering only rhs unknown is reachable"
        );
        kani::cover!(
            a.planes.1 != 0 && b.planes.1 != 0,
            "wide ordering both unknown is reachable"
        );
        kani::cover!(
            (a.planes.0 ^ b.planes.0) & !(a.planes.1 | b.planes.1) != 0,
            "wide ordering known mismatch under unknown is reachable"
        );
    } else {
        kani::cover!(
            expected == Logic4::One,
            "wide ordering known true is reachable"
        );
        kani::cover!(
            expected == Logic4::Zero,
            "wide ordering known false is reachable"
        );
        kani::cover!(
            a.planes.0 >> (w - 1) != b.planes.0 >> (w - 1),
            "wide ordering opposite high bits are reachable"
        );
        kani::cover!(
            a.planes.0 == b.planes.0,
            "wide ordering equal operands are reachable"
        );
        kani::cover!(
            a.planes.0 >> (w - 1) != 0 && b.planes.0 >> (w - 1) != 0 && a.planes.0 != b.planes.0,
            "wide ordering both high bits set are reachable"
        );
    }
    kani::cover!(
        w == 65 && out == 128,
        "wide ordering widens boolean result is reachable"
    );
    kani::cover!(
        w == 128 && out == 1,
        "wide ordering narrow result keeps full comparison is reachable"
    );
}

// For unknown operands, numeric order is irrelevant. Arbitrary comparator
// results overapproximate the exact API and prove X dominance independently.
fn arbitrary_unsigned_order(_: &BigUint, _: &BigUint) -> std::cmp::Ordering {
    unsafe {
        ARBITRARY_ORDER_CALLS += 1;
    }
    [
        std::cmp::Ordering::Less,
        std::cmp::Ordering::Equal,
        std::cmp::Ordering::Greater,
    ][usize::from(kani::any::<u8>() % 3)]
}
fn arbitrary_signed_order(_: &BigInt, _: &BigInt) -> std::cmp::Ordering {
    arbitrary_unsigned_order(&BigUint::ZERO, &BigUint::ZERO)
}

#[kani::proof]
#[kani::unwind(129)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::followup::expanded)]
#[kani::stub(veryl_analyzer::value::ValueBigUint::to_bigint, converted)]
fn wide_ordering_less_unsigned_matches_scalar() {
    ordering::<0, 0>();
}

#[kani::proof]
#[kani::unwind(129)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::followup::expanded)]
#[kani::stub(veryl_analyzer::value::ValueBigUint::to_bigint, converted)]
fn wide_ordering_less_signed_matches_scalar() {
    ordering::<0, 1>();
}

#[kani::proof]
#[kani::unwind(129)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::followup::expanded)]
#[kani::stub(veryl_analyzer::value::ValueBigUint::to_bigint, converted)]
#[kani::stub(
    crate::public_big_ops::ordering::unsigned_cmp,
    arbitrary_unsigned_order
)]
#[kani::stub(crate::public_big_ops::ordering::signed_cmp, arbitrary_signed_order)]
fn wide_ordering_less_unknown_matches_scalar() {
    ordering::<0, 2>();
}

#[kani::proof]
#[kani::unwind(129)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::followup::expanded)]
#[kani::stub(veryl_analyzer::value::ValueBigUint::to_bigint, converted)]
fn wide_ordering_less_eq_unsigned_matches_scalar() {
    ordering::<1, 0>();
}

#[kani::proof]
#[kani::unwind(129)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::followup::expanded)]
#[kani::stub(veryl_analyzer::value::ValueBigUint::to_bigint, converted)]
fn wide_ordering_less_eq_signed_matches_scalar() {
    ordering::<1, 1>();
}

#[kani::proof]
#[kani::unwind(129)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::followup::expanded)]
#[kani::stub(veryl_analyzer::value::ValueBigUint::to_bigint, converted)]
#[kani::stub(
    crate::public_big_ops::ordering::unsigned_cmp,
    arbitrary_unsigned_order
)]
#[kani::stub(crate::public_big_ops::ordering::signed_cmp, arbitrary_signed_order)]
fn wide_ordering_less_eq_unknown_matches_scalar() {
    ordering::<1, 2>();
}

#[kani::proof]
#[kani::unwind(129)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::followup::expanded)]
#[kani::stub(veryl_analyzer::value::ValueBigUint::to_bigint, converted)]
fn wide_ordering_greater_unsigned_matches_scalar() {
    ordering::<2, 0>();
}

#[kani::proof]
#[kani::unwind(129)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::followup::expanded)]
#[kani::stub(veryl_analyzer::value::ValueBigUint::to_bigint, converted)]
fn wide_ordering_greater_signed_matches_scalar() {
    ordering::<2, 1>();
}

#[kani::proof]
#[kani::unwind(129)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::followup::expanded)]
#[kani::stub(veryl_analyzer::value::ValueBigUint::to_bigint, converted)]
#[kani::stub(
    crate::public_big_ops::ordering::unsigned_cmp,
    arbitrary_unsigned_order
)]
#[kani::stub(crate::public_big_ops::ordering::signed_cmp, arbitrary_signed_order)]
fn wide_ordering_greater_unknown_matches_scalar() {
    ordering::<2, 2>();
}

#[kani::proof]
#[kani::unwind(129)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::followup::expanded)]
#[kani::stub(veryl_analyzer::value::ValueBigUint::to_bigint, converted)]
fn wide_ordering_greater_eq_unsigned_matches_scalar() {
    ordering::<3, 0>();
}

#[kani::proof]
#[kani::unwind(129)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::followup::expanded)]
#[kani::stub(veryl_analyzer::value::ValueBigUint::to_bigint, converted)]
fn wide_ordering_greater_eq_signed_matches_scalar() {
    ordering::<3, 1>();
}

#[kani::proof]
#[kani::unwind(129)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::followup::expanded)]
#[kani::stub(veryl_analyzer::value::ValueBigUint::to_bigint, converted)]
#[kani::stub(
    crate::public_big_ops::ordering::unsigned_cmp,
    arbitrary_unsigned_order
)]
#[kani::stub(crate::public_big_ops::ordering::signed_cmp, arbitrary_signed_order)]
fn wide_ordering_greater_eq_unknown_matches_scalar() {
    ordering::<3, 2>();
}
