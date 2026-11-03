//! IEEE1800-2023 11.4.3/11.6.1/11.8.2: signed |a|<|b| division/remainder.
//! Known result widths65..128; independent nonnarrowing source widths1..128 and
//! source flags. Both signs, zero dividends and minimum signed divisors included.
//! Magnitudes generate every signed pair in this class; source words are their
//! uniquely determined low bits, with both expansions validated after sizing.
//! Context proofs partition nonnegative/negative dividends; their union is the
//! complete class. Original source geometry witnesses share the same generator.
//! Actual Op and signed result constructors execute under guarded mathematical
//! numeric APIs; native and unsigned arithmetic branches must be unreachable.
//! The general conversion witness theorem includes zero/minimum signed values;
//! interior values reuse the separately proved opposite-class conversion body.
//! Every conversion checks its complete numeric operand before returning its
//! arbitrary original magnitude/sign. No num-bigint algorithm theorem is claimed.
use super::*;
use num_bigint::{BigInt, BigUint, Sign};
use veryl_formal_semantics::{
    nonlinear::{signed_magnitude_word, signed_smaller_division_word},
    resize::expanded_planes,
};

fn checked_conversion(
    value: &ValueBigUint,
    width: usize,
    magnitude: u128,
    negative: bool,
) -> Option<BigInt> {
    assert!(
        (65..=128).contains(&width),
        "small signed conversion domain exceeded"
    );
    let word = signed_magnitude_word(magnitude, negative, width);
    assert!(
        value.width as usize == width
            && get(value.payload()) == N::from_u128(word)
            && get(value.mask_xz()) == N::ZERO,
        "small signed conversion domain exceeded"
    );
    let top = 1u128 << (width - 1);
    if magnitude != 0 && magnitude < top {
        opposite::configure_magnitude(magnitude, width);
        opposite::witnessed_conversion(value)
    } else {
        Some(power::signed_from_numeric(
            if negative { Sign::Minus } else { Sign::Plus },
            from128(magnitude),
        ))
    }
}
static mut MAGNITUDES: [u128; 2] = [0; 2];
static mut NEGATIVE: [bool; 2] = [false; 2];
static mut WIDTH: usize = 0;
static mut CONVERSIONS: usize = 0;
pub(super) fn configure(magnitudes: [u128; 2], negative: [bool; 2], width: usize) {
    unsafe {
        MAGNITUDES = magnitudes;
        NEGATIVE = negative;
        WIDTH = width;
        CONVERSIONS = 0;
    }
}
pub(super) fn converted_input(value: &ValueBigUint) -> Option<BigInt> {
    unsafe {
        let i = CONVERSIONS;
        assert!(i < 2, "small signed conversion call bound exceeded");
        CONVERSIONS += 1;
        checked_conversion(value, WIDTH, MAGNITUDES[i], NEGATIVE[i])
    }
}
// Shared conversion interface; its theorem covers every signed magnitude,
// independently of the smaller-dividend arithmetic specialization.
pub(super) fn conversion_calls() -> usize {
    unsafe { CONVERSIONS }
}
fn arithmetic_class(a: &BigInt, b: &BigInt) {
    let x = get(a.magnitude());
    let y = get(b.magnitude());
    assert!(
        x.hi == 0 && y.hi == 0 && x.lo < y.lo,
        "small signed arithmetic domain exceeded"
    );
}
fn divide(a: BigInt, b: BigInt) -> BigInt {
    arithmetic_class(&a, &b);
    BigInt::ZERO
}
fn remainder(a: BigInt, b: BigInt) -> BigInt {
    arithmetic_class(&a, &b);
    a
}
fn reject_unsigned<'a: 'a, 'b: 'b>(_: &'a BigUint, _: &'b BigUint) -> BigUint {
    panic!("small signed unsigned arithmetic boundary reached")
}
fn reject_native(
    _: &veryl_analyzer::value::ValueU64,
    _: &veryl_analyzer::value::ValueU64,
    _: usize,
    _: bool,
    _: bool,
) -> veryl_analyzer::value::ValueU64 {
    panic!("small signed native arithmetic boundary reached")
}

fn words(width: usize) -> (u128, u128, bool, bool, u128, u128) {
    let top = 1u128 << (width - 1);
    let ma = kani::any::<u128>() & (top - 1);
    let mb = kani::any::<u128>() & (u128::MAX >> (128 - width));
    kani::assume(ma < mb && mb <= top);
    let na = ma != 0 && kani::any::<bool>();
    let nb: bool = kani::any();
    kani::assume(mb < top || nb);
    (
        ma,
        mb,
        na,
        nb,
        signed_magnitude_word(ma, na, width),
        signed_magnitude_word(mb, nb, width),
    )
}
fn inputs<const ORIGINAL: bool, const ORIENTATION: u8>() -> (
    Value,
    Operand,
    Value,
    Operand,
    usize,
    u128,
    u128,
    u128,
    u128,
    bool,
    bool,
) {
    init();
    let width = usize::from(kani::any::<u8>() & 63) + 65;
    let aw = if ORIGINAL {
        usize::from(kani::any::<u8>() & 127) + 1
    } else {
        width
    };
    let bw = if ORIGINAL {
        usize::from(kani::any::<u8>() & 127) + 1
    } else {
        width
    };
    kani::assume(aw <= width && bw <= width);
    let (ma, mb, na, nb, ap, bp) = words(width);
    if ORIENTATION != 0 {
        kani::assume(na == (ORIENTATION == 2));
    }
    let (x, a) = identities::known_input(aw, Some(ap));
    let (y, b) = identities::known_input(bw, Some(bp));
    kani::assume(expanded_planes(a.planes, aw, width, a.signed, true).0 == ap);
    kani::assume(expanded_planes(b.planes, bw, width, b.signed, true).0 == bp);
    configure([ma, mb], [na, nb], width);
    (x, a, y, b, width, ap, bp, ma, mb, na, nb)
}
fn check<const REM: bool, const ORIGINAL: bool, const ORIENTATION: u8>() {
    let (x, a, y, b, width, ap, _bp, ma, mb, na, nb) = inputs::<ORIGINAL, ORIENTATION>();
    special::observe_binary_sources(&x, &y, width, true);
    let out = if REM { Op::Rem } else { Op::Div }.eval_value_binary(
        &x,
        &y,
        width,
        true,
        &mut MaskCache::default(),
    );
    assert!(
        unsafe { CONVERSIONS == 2 },
        "wide operator semantics mismatch"
    );
    if ORIGINAL {
        assert!(
            special::observed_calls() == 2,
            "wide operator semantics mismatch"
        );
    }
    // The independent word-class theorem validates this mathematical summary.
    // Actual arithmetic receives the full magnitudes checked at both conversions.
    let expected = if REM { ap } else { 0 };
    preserve(
        &out,
        Operand {
            planes: (expected, 0),
            width,
            signed: true,
        },
    );
    let position = usize::from(kani::any::<u8>() & 127);
    result(
        &out,
        width,
        true,
        position,
        read_logic(expected, 0, position),
    );
    preserve(&x, a);
    preserve(&y, b);
    if ORIGINAL {
        if ORIENTATION == 1 {
            kani::cover!(
                width == 65 && ma == 1,
                "small signed class nonnegative context is reachable"
            );
        } else {
            kani::cover!(
                width == 65 && ma == 1,
                "small signed class negative context is reachable"
            );
        }
    } else {
        kani::cover!(
            width == 65 && ma > 0 && !na && !nb,
            "small signed class minimum positive pair is reachable"
        );
        kani::cover!(
            width == 128 && mb == 1u128 << 127,
            "small signed class minimum divisor is reachable"
        );
        kani::cover!(ma == 0, "small signed class zero dividend is reachable");
        kani::cover!(na, "small signed class negative dividend is reachable");
    }
}
#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(
    crate::public_big_ops::ordering::from_biguint,
    super::power::signed_from_numeric
)]
#[kani::stub(<num_bigint::BigInt as std::ops::Div<num_bigint::BigInt>>::div, divide)]
#[kani::stub(<num_bigint::BigInt as std::ops::Rem<num_bigint::BigInt>>::rem, remainder)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, reject_unsigned)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, reject_unsigned)]
#[kani::stub(veryl_analyzer::value::ValueU64::divide_remainder, reject_native)]
fn small_signed_conversion_includes_zero_and_minimum() {
    init();
    let width = usize::from(kani::any::<u8>() & 63) + 65;
    let top = 1u128 << (width - 1);
    let magnitude = kani::any::<u128>() & (u128::MAX >> (128 - width));
    kani::assume(magnitude <= top);
    let negative: bool = kani::any();
    kani::assume(magnitude < top || negative);
    let word = signed_magnitude_word(magnitude, negative, width);
    let (value, a) = identities::known_input(width, Some(word));
    let Value::BigUint(v) = &value else {
        panic!("small signed conversion representation mismatch")
    };
    let decoded = ordering::converted(v).unwrap();
    let witnessed = checked_conversion(v, width, magnitude, negative).unwrap();
    let sign = if magnitude == 0 {
        Sign::NoSign
    } else if negative {
        Sign::Minus
    } else {
        Sign::Plus
    };
    assert!(
        decoded.sign() == sign
            && witnessed.sign() == sign
            && get(decoded.magnitude()) == N::from_u128(magnitude)
            && get(witnessed.magnitude()) == get(decoded.magnitude()),
        "small signed conversion witness mismatch"
    );
    preserve(&value, a);
    kani::cover!(
        width == 65,
        "small signed conversion minimum width is reachable"
    );
    kani::cover!(
        width == 128,
        "small signed conversion maximum width is reachable"
    );
    kani::cover!(magnitude == 0, "small signed conversion zero is reachable");
    kani::cover!(
        width == 128 && magnitude == top,
        "small signed conversion minimum integer is reachable"
    );
    kani::cover!(
        !negative && magnitude == u128::MAX >> 1,
        "small signed conversion maximum positive is reachable"
    );
}
#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(
    crate::public_big_ops::ordering::from_biguint,
    super::power::signed_from_numeric
)]
#[kani::stub(<num_bigint::BigInt as std::ops::Div<num_bigint::BigInt>>::div, divide)]
#[kani::stub(<num_bigint::BigInt as std::ops::Rem<num_bigint::BigInt>>::rem, remainder)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, reject_unsigned)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, reject_unsigned)]
#[kani::stub(veryl_analyzer::value::ValueU64::divide_remainder, reject_native)]
fn small_signed_arithmetic_matches_byte_division() {
    init();
    let a = kani::any::<i8>();
    let b = kani::any::<i8>();
    kani::assume(a.unsigned_abs() < b.unsigned_abs());
    let x = power::signed_from_numeric(
        if a < 0 { Sign::Minus } else { Sign::Plus },
        from64(u64::from(a.unsigned_abs())),
    );
    let y = power::signed_from_numeric(
        if b < 0 { Sign::Minus } else { Sign::Plus },
        from64(u64::from(b.unsigned_abs())),
    );
    let q = x.clone() / y.clone();
    let r = x.clone() % y.clone();
    let nq = i16::from(a) / i16::from(b);
    let nr = i16::from(a) % i16::from(b);
    assert!(
        q.sign() == Sign::NoSign
            && get(q.magnitude()) == N::from_u128(nq.unsigned_abs() as u128)
            && r.sign() == x.sign()
            && get(r.magnitude()) == N::from_u128(nr.unsigned_abs() as u128)
            && get(x.magnitude()) == N::from_u128(u128::from(a.unsigned_abs()))
            && get(y.magnitude()) == N::from_u128(u128::from(b.unsigned_abs()))
            && (x.sign() == Sign::Minus) == (a < 0)
            && (y.sign() == Sign::Minus) == (b < 0),
        "small signed numeric API mismatch"
    );
    kani::cover!(a == 0, "small signed API zero dividend is reachable");
    kani::cover!(a > 0, "small signed API positive dividend is reachable");
    kani::cover!(a < 0, "small signed API negative dividend is reachable");
    kani::cover!(
        b == i8::MIN,
        "small signed API minimum divisor is reachable"
    );
    kani::cover!(
        a > 0 && b < 0,
        "small signed API opposite signs is reachable"
    );
}
#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(
    crate::public_big_ops::ordering::from_biguint,
    super::power::signed_from_numeric
)]
#[kani::stub(<num_bigint::BigInt as std::ops::Div<num_bigint::BigInt>>::div, divide)]
#[kani::stub(<num_bigint::BigInt as std::ops::Rem<num_bigint::BigInt>>::rem, remainder)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, reject_unsigned)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, reject_unsigned)]
#[kani::stub(veryl_analyzer::value::ValueU64::divide_remainder, reject_native)]
fn small_signed_word_class_matches_decoded_magnitudes() {
    init();
    let width = usize::from(kani::any::<u8>() & 63) + 65;
    let (ma, mb, na, nb, ap, bp) = words(width);
    assert!(
        signed_smaller_division_word(ap, bp, width, false) == 0
            && signed_smaller_division_word(ap, bp, width, true) == ap,
        "small signed word class mismatch"
    );
    kani::cover!(width == 65, "small signed word minimum width is reachable");
    kani::cover!(width == 128, "small signed word maximum width is reachable");
    kani::cover!(ma == 0, "small signed word zero dividend is reachable");
    kani::cover!(
        mb == 1u128 << 127,
        "small signed word minimum divisor is reachable"
    );
    kani::cover!(na && nb, "small signed word both negative is reachable");
}
#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(
    crate::public_big_ops::ordering::from_biguint,
    super::power::signed_from_numeric
)]
#[kani::stub(<num_bigint::BigInt as std::ops::Div<num_bigint::BigInt>>::div, divide)]
#[kani::stub(<num_bigint::BigInt as std::ops::Rem<num_bigint::BigInt>>::rem, remainder)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, reject_unsigned)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, reject_unsigned)]
#[kani::stub(veryl_analyzer::value::ValueU64::divide_remainder, reject_native)]
fn small_signed_original_domain_preserves_prepared_values() {
    let (x, a, y, b, width, ap, bp, ma, mb, na, nb) = inputs::<true, 0>();
    special::observe_binary_sources(&x, &y, width, true);
    let xp = special::observed_expand(&x, width, true);
    let yp = special::observed_expand(&y, width, true);
    preserve(
        xp.as_ref(),
        Operand {
            planes: (ap, 0),
            width,
            signed: a.signed,
        },
    );
    preserve(
        yp.as_ref(),
        Operand {
            planes: (bp, 0),
            width,
            signed: b.signed,
        },
    );
    let pos = usize::from(kani::any::<u8>() & 127);
    result(xp.as_ref(), width, a.signed, pos, read_logic(ap, 0, pos));
    result(yp.as_ref(), width, b.signed, pos, read_logic(bp, 0, pos));
    let Value::BigUint(xb) = xp.as_ref() else {
        panic!("small signed prepared representation mismatch")
    };
    let Value::BigUint(yb) = yp.as_ref() else {
        panic!("small signed prepared representation mismatch")
    };
    let cx = converted_input(xb).unwrap();
    let cy = converted_input(yb).unwrap();
    assert!(
        get(cx.magnitude()) == N::from_u128(ma)
            && get(cy.magnitude()) == N::from_u128(mb)
            && (cx.sign() == Sign::Minus) == na
            && (cy.sign() == Sign::Minus) == nb
            && unsafe { CONVERSIONS == 2 }
            && special::observed_calls() == 2,
        "small signed context domain mismatch"
    );
    preserve(&x, a);
    preserve(&y, b);
    kani::cover!(
        a.width == 64 && b.width == 64 && width == 128,
        "small signed domain native native is reachable"
    );
    kani::cover!(
        a.width == 64 && b.width == 65 && width == 128,
        "small signed domain native big is reachable"
    );
    kani::cover!(
        a.width == 65 && b.width == 64 && width == 128,
        "small signed domain big native is reachable"
    );
    kani::cover!(
        a.width == 65 && b.width == 127 && width == 128,
        "small signed domain big big is reachable"
    );
    kani::cover!(
        !na && !nb && ma > 0,
        "small signed domain positive positive is reachable"
    );
    kani::cover!(
        !na && nb && ma > 0,
        "small signed domain positive negative is reachable"
    );
    kani::cover!(
        na && !nb,
        "small signed domain negative positive is reachable"
    );
    kani::cover!(
        na && nb,
        "small signed domain negative negative is reachable"
    );
    kani::cover!(
        width == 65 && ma == 0,
        "small signed domain minimum zero is reachable"
    );
    kani::cover!(
        width == 128 && mb == 1u128 << 127,
        "small signed domain maximum minimum divisor is reachable"
    );
    kani::cover!(
        a.signed != b.signed,
        "small signed domain mixed source signs is reachable"
    );
}
#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(
    crate::public_big_ops::ordering::from_biguint,
    super::power::signed_from_numeric
)]
#[kani::stub(<num_bigint::BigInt as std::ops::Div<num_bigint::BigInt>>::div, divide)]
#[kani::stub(<num_bigint::BigInt as std::ops::Rem<num_bigint::BigInt>>::rem, remainder)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, reject_unsigned)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, reject_unsigned)]
#[kani::stub(veryl_analyzer::value::ValueU64::divide_remainder, reject_native)]
#[kani::stub(veryl_analyzer::value::ValueBigUint::to_bigint, converted_input)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::prepared_wide)]
fn wide_small_signed_div_matches_reference() {
    check::<false, false, 0>();
}
#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(
    crate::public_big_ops::ordering::from_biguint,
    super::power::signed_from_numeric
)]
#[kani::stub(<num_bigint::BigInt as std::ops::Div<num_bigint::BigInt>>::div, divide)]
#[kani::stub(<num_bigint::BigInt as std::ops::Rem<num_bigint::BigInt>>::rem, remainder)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, reject_unsigned)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, reject_unsigned)]
#[kani::stub(veryl_analyzer::value::ValueU64::divide_remainder, reject_native)]
#[kani::stub(veryl_analyzer::value::ValueBigUint::to_bigint, converted_input)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::special::observed_expand)]
fn wide_small_signed_div_nonnegative_context_matches_reference() {
    check::<false, true, 1>();
}
#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(
    crate::public_big_ops::ordering::from_biguint,
    super::power::signed_from_numeric
)]
#[kani::stub(<num_bigint::BigInt as std::ops::Div<num_bigint::BigInt>>::div, divide)]
#[kani::stub(<num_bigint::BigInt as std::ops::Rem<num_bigint::BigInt>>::rem, remainder)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, reject_unsigned)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, reject_unsigned)]
#[kani::stub(veryl_analyzer::value::ValueU64::divide_remainder, reject_native)]
#[kani::stub(veryl_analyzer::value::ValueBigUint::to_bigint, converted_input)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::special::observed_expand)]
fn wide_small_signed_div_negative_context_matches_reference() {
    check::<false, true, 2>();
}
#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(
    crate::public_big_ops::ordering::from_biguint,
    super::power::signed_from_numeric
)]
#[kani::stub(<num_bigint::BigInt as std::ops::Div<num_bigint::BigInt>>::div, divide)]
#[kani::stub(<num_bigint::BigInt as std::ops::Rem<num_bigint::BigInt>>::rem, remainder)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, reject_unsigned)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, reject_unsigned)]
#[kani::stub(veryl_analyzer::value::ValueU64::divide_remainder, reject_native)]
#[kani::stub(veryl_analyzer::value::ValueBigUint::to_bigint, converted_input)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::prepared_wide)]
fn wide_small_signed_rem_matches_reference() {
    check::<true, false, 0>();
}
#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(
    crate::public_big_ops::ordering::from_biguint,
    super::power::signed_from_numeric
)]
#[kani::stub(<num_bigint::BigInt as std::ops::Div<num_bigint::BigInt>>::div, divide)]
#[kani::stub(<num_bigint::BigInt as std::ops::Rem<num_bigint::BigInt>>::rem, remainder)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, reject_unsigned)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, reject_unsigned)]
#[kani::stub(veryl_analyzer::value::ValueU64::divide_remainder, reject_native)]
#[kani::stub(veryl_analyzer::value::ValueBigUint::to_bigint, converted_input)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::special::observed_expand)]
fn wide_small_signed_rem_nonnegative_context_matches_reference() {
    check::<true, true, 1>();
}
#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(
    crate::public_big_ops::ordering::from_biguint,
    super::power::signed_from_numeric
)]
#[kani::stub(<num_bigint::BigInt as std::ops::Div<num_bigint::BigInt>>::div, divide)]
#[kani::stub(<num_bigint::BigInt as std::ops::Rem<num_bigint::BigInt>>::rem, remainder)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, reject_unsigned)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, reject_unsigned)]
#[kani::stub(veryl_analyzer::value::ValueU64::divide_remainder, reject_native)]
#[kani::stub(veryl_analyzer::value::ValueBigUint::to_bigint, converted_input)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::special::observed_expand)]
fn wide_small_signed_rem_negative_context_matches_reference() {
    check::<true, true, 2>();
}
