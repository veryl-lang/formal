//! IEEE1800-2023 11.4.3/11.6.1/11.8.2: known signed power-of-two divisors.
//! Result widths65..128; independent nonnarrowing source widths1..128 and flags.
//! Every valid signed dividend is included, including zero/minimum. Divisor
//! magnitudes2^k cover0<=k<width and both representable signs; the top magnitude
//! requires a negative divisor. Minimum/-1 encodes its result modulo2^width.
//! Source words are uniquely the prepared words' low bits, with both expansions
//! validated. Context proofs partition nonnegative/negative dividends completely.
//! Actual Op and signed result constructors execute under guarded mathematical
//! arithmetic APIs, not num-bigint algorithm proofs. The reused signed_small
//! conversion theorem preserves arbitrary values, including zero/minimum.
//! Scalar magnitude coordinates validate numeric shifts/masks independently;
//! native signed decoding validates the modular word summary separately.
use super::*;
use num_bigint::{BigInt, BigUint, Sign};
use veryl_formal_semantics::{
    nonlinear::{
        power_two_division_magnitude_bit, signed_magnitude_word, signed_power_two_division_word,
    },
    resize::expanded_planes,
};

// The shift witness is checked against the ACTUAL divisor magnitude on every
// numeric call; no production value is assumed to equal a fixed operand.
static mut SHIFT: usize = 0;
fn configure_shift(shift: usize) {
    unsafe {
        SHIFT = shift;
    }
}
fn arithmetic_class(a: &BigInt, b: &BigInt) -> (N, usize) {
    let x = get(a.magnitude());
    let y = get(b.magnitude());
    let shift = unsafe { SHIFT };
    assert!(
        shift < 128
            && x.hi == 0
            && y == N::from_u128(1u128 << shift)
            && b.sign() != Sign::NoSign
            && (x == N::ZERO) == (a.sign() == Sign::NoSign),
        "signed power arithmetic domain exceeded"
    );
    (x, shift)
}
fn divide(a: BigInt, b: BigInt) -> BigInt {
    let (x, shift) = arithmetic_class(&a, &b);
    power::signed_from_numeric(
        if (a.sign() == Sign::Minus) != (b.sign() == Sign::Minus) {
            Sign::Minus
        } else {
            Sign::Plus
        },
        put(x.shr(shift)),
    )
}
fn remainder(a: BigInt, b: BigInt) -> BigInt {
    let (x, shift) = arithmetic_class(&a, &b);
    power::signed_from_numeric(
        if a.sign() == Sign::Minus {
            Sign::Minus
        } else {
            Sign::Plus
        },
        put(x.and(N::mask(shift))),
    )
}
fn reject_unsigned<'a: 'a, 'b: 'b>(_: &'a BigUint, _: &'b BigUint) -> BigUint {
    panic!("signed power unsigned arithmetic boundary reached")
}
fn reject_native(
    _: &veryl_analyzer::value::ValueU64,
    _: &veryl_analyzer::value::ValueU64,
    _: usize,
    _: bool,
    _: bool,
) -> veryl_analyzer::value::ValueU64 {
    panic!("signed power native arithmetic boundary reached")
}
fn result_sign(magnitude: u128, negative: bool) -> Sign {
    if magnitude == 0 {
        Sign::NoSign
    } else if negative {
        Sign::Minus
    } else {
        Sign::Plus
    }
}
// Modular result representation; the positive minimum/-1 magnitude is allowed
// to reach the sign bit. The independent signed-word theorem below checks this
// summary rather than imposing a representability bound on intermediate results.
fn expected_word(ma: u128, shift: usize, na: bool, nb: bool, width: usize, rem: bool) -> u128 {
    let magnitude = if rem {
        ma & ((1u128 << shift) - 1)
    } else {
        ma >> shift
    };
    let negative = if rem { na } else { na != nb };
    (if negative {
        magnitude.wrapping_neg()
    } else {
        magnitude
    }) & (u128::MAX >> (128 - width))
}
fn words(width: usize) -> (u128, usize, bool, bool, u128, u128) {
    let top = 1u128 << (width - 1);
    let ma = kani::any::<u128>() & (u128::MAX >> (128 - width));
    kani::assume(ma <= top);
    let shift = usize::from(kani::any::<u8>() & 127);
    kani::assume(shift < width);
    let na = ma != 0 && kani::any::<bool>();
    let nb: bool = kani::any();
    kani::assume(ma < top || na);
    kani::assume(shift < width - 1 || nb);
    (
        ma,
        shift,
        na,
        nb,
        signed_magnitude_word(ma, na, width),
        signed_magnitude_word(1u128 << shift, nb, width),
    )
}
fn inputs<const ORIGINAL: bool, const ORIENTATION: u8>() -> (
    Value,
    Operand,
    Value,
    Operand,
    usize,
    u128,
    usize,
    bool,
    bool,
    u128,
    u128,
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
    let (ma, shift, na, nb, ap, bp) = words(width);
    if ORIENTATION != 0 {
        kani::assume(na == (ORIENTATION == 2));
    }
    let (x, a) = identities::known_input(aw, Some(ap));
    let (y, b) = identities::known_input(bw, Some(bp));
    kani::assume(expanded_planes(a.planes, aw, width, a.signed, true).0 == ap);
    kani::assume(expanded_planes(b.planes, bw, width, b.signed, true).0 == bp);
    signed_small::configure([ma, 1u128 << shift], [na, nb], width);
    configure_shift(shift);
    (x, a, y, b, width, ma, shift, na, nb, ap, bp)
}
fn check<const REM: bool, const ORIGINAL: bool, const ORIENTATION: u8>() {
    let (x, a, y, b, width, ma, shift, na, nb, _ap, _bp) = inputs::<ORIGINAL, ORIENTATION>();
    special::observe_binary_sources(&x, &y, width, true);
    let out = if REM { Op::Rem } else { Op::Div }.eval_value_binary(
        &x,
        &y,
        width,
        true,
        &mut MaskCache::default(),
    );
    assert!(
        signed_small::conversion_calls() == 2,
        "wide operator semantics mismatch"
    );
    if ORIGINAL {
        assert!(
            special::observed_calls() == 2,
            "wide operator semantics mismatch"
        );
    }
    let expected = expected_word(ma, shift, na, nb, width, REM);
    preserve(
        &out,
        Operand {
            planes: (expected, 0),
            width,
            signed: true,
        },
    );
    let pos = usize::from(kani::any::<u8>() & 127);
    result(&out, width, true, pos, read_logic(expected, 0, pos));
    preserve(&x, a);
    preserve(&y, b);
    if ORIGINAL {
        if ORIENTATION == 1 {
            kani::cover!(
                width == 65 && ma == 1 && shift == 1,
                "power signed class nonnegative context is reachable"
            );
        } else {
            kani::cover!(
                width == 65 && ma == 1 && shift == 1,
                "power signed class negative context is reachable"
            );
        }
    } else {
        kani::cover!(
            width == 65 && shift == 1 && ma == 3,
            "power signed class minimum nonunit divisor is reachable"
        );
        kani::cover!(
            width == 128 && ma == 1u128 << 127 && shift == 0 && nb,
            "power signed class minimum over minus one is reachable"
        );
        kani::cover!(ma == 0, "power signed class zero dividend is reachable");
        kani::cover!(
            shift == 64 && ma >> 64 != 0,
            "power signed class word crossing is reachable"
        );
        kani::cover!(
            shift == 127,
            "power signed class minimum divisor is reachable"
        );
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
fn signed_power_arithmetic_matches_byte_division() {
    init();
    let a = kani::any::<i8>();
    let b = kani::any::<i8>();
    kani::assume(b.unsigned_abs().is_power_of_two());
    configure_shift(b.unsigned_abs().trailing_zeros() as usize);
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
        q.sign() == result_sign(nq.unsigned_abs() as u128, nq < 0)
            && r.sign() == result_sign(nr.unsigned_abs() as u128, nr < 0)
            && get(q.magnitude()) == N::from_u128(nq.unsigned_abs() as u128)
            && get(r.magnitude()) == N::from_u128(nr.unsigned_abs() as u128)
            && get(x.magnitude()) == N::from_u128(u128::from(a.unsigned_abs()))
            && get(y.magnitude()) == N::from_u128(u128::from(b.unsigned_abs()))
            && (x.sign() == Sign::Minus) == (a < 0)
            && (y.sign() == Sign::Minus) == (b < 0),
        "signed power numeric API mismatch"
    );
    kani::cover!(a == 0, "power signed API zero dividend is reachable");
    kani::cover!(
        a < 0 && b > 0,
        "power signed API negative quotient is reachable"
    );
    kani::cover!(
        a < 0 && b < 0,
        "power signed API positive quotient is reachable"
    );
    kani::cover!(
        b == i8::MIN,
        "power signed API minimum divisor is reachable"
    );
    kani::cover!(
        a == i8::MIN && b == -1,
        "power signed API minimum over minus one is reachable"
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
fn signed_power_magnitude_matches_scalar_coordinates() {
    init();
    let number: u128 = kani::any();
    let shift = usize::from(kani::any::<u8>() & 127);
    let na: bool = kani::any();
    let nb: bool = kani::any();
    configure_shift(shift);
    let x = power::signed_from_numeric(if na { Sign::Minus } else { Sign::Plus }, from128(number));
    let y = power::signed_from_numeric(
        if nb { Sign::Minus } else { Sign::Plus },
        from128(1u128 << shift),
    );
    let q = x.clone() / y.clone();
    let r = x.clone() % y.clone();
    let position = usize::from(kani::any::<u8>() & 127);
    let qm = get(q.magnitude());
    let rm = get(r.magnitude());
    assert!(
        qm.hi == 0
            && rm.hi == 0
            && (qm.lo >> position & 1 != 0)
                == power_two_division_magnitude_bit(number, shift, position, false)
            && (rm.lo >> position & 1 != 0)
                == power_two_division_magnitude_bit(number, shift, position, true)
            && q.sign() == result_sign(number >> shift, na != nb)
            && r.sign() == result_sign(number & ((1u128 << shift) - 1), na)
            && get(x.magnitude()) == N::from_u128(number)
            && get(y.magnitude()) == N::from_u128(1u128 << shift)
            && x.sign() == result_sign(number, na)
            && y.sign() == result_sign(1u128 << shift, nb),
        "signed power coordinate API mismatch"
    );
    kani::cover!(
        shift == 0,
        "power signed coordinates unit divisor is reachable"
    );
    kani::cover!(
        number == 0,
        "power signed coordinates zero dividend is reachable"
    );
    kani::cover!(
        shift == 64 && number >> 64 != 0,
        "power signed coordinates word crossing is reachable"
    );
    kani::cover!(
        shift == 127 && number >> 127 != 0,
        "power signed coordinates top divisor is reachable"
    );
    kani::cover!(
        na && nb && number == u128::MAX,
        "power signed coordinates full magnitude is reachable"
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
fn signed_power_word_matches_signed_decoding() {
    init();
    let width = usize::from(kani::any::<u8>() & 63) + 65;
    let (ma, shift, na, nb, ap, bp) = words(width);
    assert!(
        signed_power_two_division_word(ap, bp, shift, width, false)
            == expected_word(ma, shift, na, nb, width, false)
            && signed_power_two_division_word(ap, bp, shift, width, true)
                == expected_word(ma, shift, na, nb, width, true),
        "signed power word class mismatch"
    );
    kani::cover!(width == 65, "power signed word minimum width is reachable");
    kani::cover!(width == 128, "power signed word maximum width is reachable");
    kani::cover!(ma == 0, "power signed word zero dividend is reachable");
    kani::cover!(
        width == 128 && ma == 1u128 << 127 && shift == 0 && nb,
        "power signed word minimum over minus one is reachable"
    );
    kani::cover!(
        shift == 127,
        "power signed word minimum divisor is reachable"
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
fn signed_power_original_domain_preserves_prepared_values() {
    let (x, a, y, b, width, ma, shift, na, nb, ap, bp) = inputs::<true, 0>();
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
        panic!("signed power prepared representation mismatch")
    };
    let Value::BigUint(yb) = yp.as_ref() else {
        panic!("signed power prepared representation mismatch")
    };
    let cx = signed_small::converted_input(xb).unwrap();
    let cy = signed_small::converted_input(yb).unwrap();
    assert!(
        get(cx.magnitude()) == N::from_u128(ma)
            && get(cy.magnitude()) == N::from_u128(1u128 << shift)
            && cx.sign() == result_sign(ma, na)
            && cy.sign() == result_sign(1u128 << shift, nb)
            && signed_small::conversion_calls() == 2
            && special::observed_calls() == 2,
        "signed power context domain mismatch"
    );
    preserve(&x, a);
    preserve(&y, b);
    kani::cover!(
        a.width == 64 && b.width == 64 && width == 128,
        "power signed domain native native is reachable"
    );
    kani::cover!(
        a.width == 64 && b.width == 65 && width == 128,
        "power signed domain native big is reachable"
    );
    kani::cover!(
        a.width == 65 && b.width == 64 && width == 128,
        "power signed domain big native is reachable"
    );
    kani::cover!(
        a.width == 65 && b.width == 127 && width == 128,
        "power signed domain big big is reachable"
    );
    kani::cover!(
        !na && !nb && ma > 0,
        "power signed domain positive positive is reachable"
    );
    kani::cover!(
        !na && nb && ma > 0,
        "power signed domain positive negative is reachable"
    );
    kani::cover!(
        na && !nb,
        "power signed domain negative positive is reachable"
    );
    kani::cover!(
        na && nb,
        "power signed domain negative negative is reachable"
    );
    kani::cover!(
        width == 65 && ma == 0,
        "power signed domain minimum zero is reachable"
    );
    kani::cover!(
        width == 128 && ma == 1u128 << 127 && shift == 0 && nb,
        "power signed domain minimum over minus one is reachable"
    );
    kani::cover!(
        shift == 127,
        "power signed domain minimum divisor is reachable"
    );
    kani::cover!(
        a.signed != b.signed,
        "power signed domain mixed source signs is reachable"
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
#[kani::stub(
    veryl_analyzer::value::ValueBigUint::to_bigint,
    super::signed_small::converted_input
)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::prepared_wide)]
fn wide_signed_power_div_matches_reference() {
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
#[kani::stub(
    veryl_analyzer::value::ValueBigUint::to_bigint,
    super::signed_small::converted_input
)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::special::observed_expand)]
fn wide_signed_power_div_nonnegative_context_matches_reference() {
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
#[kani::stub(
    veryl_analyzer::value::ValueBigUint::to_bigint,
    super::signed_small::converted_input
)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::special::observed_expand)]
fn wide_signed_power_div_negative_context_matches_reference() {
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
#[kani::stub(
    veryl_analyzer::value::ValueBigUint::to_bigint,
    super::signed_small::converted_input
)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::prepared_wide)]
fn wide_signed_power_rem_matches_reference() {
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
#[kani::stub(
    veryl_analyzer::value::ValueBigUint::to_bigint,
    super::signed_small::converted_input
)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::special::observed_expand)]
fn wide_signed_power_rem_nonnegative_context_matches_reference() {
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
#[kani::stub(
    veryl_analyzer::value::ValueBigUint::to_bigint,
    super::signed_small::converted_input
)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::special::observed_expand)]
fn wide_signed_power_rem_negative_context_matches_reference() {
    check::<true, true, 2>();
}
