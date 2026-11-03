//! IEEE1800-2023 11.4.3/11.6.1/11.8.2: signed quotient-magnitude2^k.
//! Known result65..128, originals1..128 nonnarrowing with independent flags,
//! caller signed. The divisor magnitude is arbitrary and nonzero. Class
//! membership is checked on magnitudes: mb<=ma>>k, ma-(mb<<k)<mb.
//! Both input signs, canonical zero remainders and minimum/-1 wrapping included.
//! Unsigned magnitude arithmetic reuses the unchanged guarded power_quotient
//! API: actual magnitudes/full256-bit products are checked before results.
//! Byte signed division checks sign composition; a separate word theorem checks
//! signed decoding/encoding against scalar magnitude and negation coordinates.
//! Signed conversion reuses the checked signed_small interface including minima.
//! Prepared proofs partition dividend signs; original contexts partition both
//! input signs. The unions retain the complete class.
//! Actual Op and signed result constructors execute; unsigned/native branches
//! must be unreachable. No num-bigint algorithm proof is claimed.
use super::*;
use num_bigint::{BigInt, BigUint, Sign};
use veryl_formal_semantics::{
    nonlinear::{
        negated_word_bit, power_quotient_bit, signed_magnitude_word, signed_power_quotient_word,
    },
    resize::expanded_planes,
};
fn nonzero_signs(a: &BigInt, b: &BigInt) {
    assert!(
        get(a.magnitude()) != N::ZERO
            && get(b.magnitude()) != N::ZERO
            && a.sign() != Sign::NoSign
            && b.sign() != Sign::NoSign,
        "signed quotient sign domain exceeded"
    );
}
fn divide(a: BigInt, b: BigInt) -> BigInt {
    nonzero_signs(&a, &b);
    let magnitude = power_quotient::divide(a.magnitude(), b.magnitude());
    power::signed_from_numeric(
        if (a.sign() == Sign::Minus) != (b.sign() == Sign::Minus) {
            Sign::Minus
        } else {
            Sign::Plus
        },
        magnitude,
    )
}
fn remainder(a: BigInt, b: BigInt) -> BigInt {
    nonzero_signs(&a, &b);
    let magnitude = power_quotient::remainder(a.magnitude(), b.magnitude());
    power::signed_from_numeric(
        if a.sign() == Sign::Minus {
            Sign::Minus
        } else {
            Sign::Plus
        },
        magnitude,
    )
}
fn reject_unsigned<'a: 'a, 'b: 'b>(_: &'a BigUint, _: &'b BigUint) -> BigUint {
    panic!("signed quotient unsigned boundary reached")
}
fn reject_native(
    _: &veryl_analyzer::value::ValueU64,
    _: &veryl_analyzer::value::ValueU64,
    _: usize,
    _: bool,
    _: bool,
) -> veryl_analyzer::value::ValueU64 {
    panic!("signed quotient native boundary reached")
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
fn expected_word(ma: u128, mb: u128, k: usize, na: bool, nb: bool, w: usize, rem: bool) -> u128 {
    let mag = if rem { ma - (mb << k) } else { 1u128 << k };
    let neg = if rem { na } else { na != nb };
    (if neg { mag.wrapping_neg() } else { mag }) & (u128::MAX >> (128 - w))
}
fn words(w: usize) -> (u128, u128, usize, bool, bool, u128, u128) {
    let top = 1u128 << (w - 1);
    let mask = u128::MAX >> (128 - w);
    let ma = kani::any::<u128>() & mask;
    let mb = kani::any::<u128>() & mask;
    let k = usize::from(kani::any::<u8>() & 127);
    kani::assume(
        ma <= top && mb <= top && mb != 0 && k < w && mb <= ma >> k && ma - (mb << k) < mb,
    );
    let na: bool = kani::any();
    let nb: bool = kani::any();
    kani::assume(ma < top || na);
    kani::assume(mb < top || nb);
    (
        ma,
        mb,
        k,
        na,
        nb,
        signed_magnitude_word(ma, na, w),
        signed_magnitude_word(mb, nb, w),
    )
}
fn inputs<
    const ORIGINAL: bool,
    const NEGATIVE: bool,
    const PARTITION: bool,
    const DIVISOR_SIGN: u8,
>() -> (
    Value,
    Operand,
    Value,
    Operand,
    usize,
    u128,
    u128,
    usize,
    bool,
    bool,
    u128,
    u128,
) {
    init();
    let w = usize::from(kani::any::<u8>() & 63) + 65;
    let aw = if ORIGINAL {
        usize::from(kani::any::<u8>() & 127) + 1
    } else {
        w
    };
    let bw = if ORIGINAL {
        usize::from(kani::any::<u8>() & 127) + 1
    } else {
        w
    };
    kani::assume(aw <= w && bw <= w);
    let (ma, mb, k, na, nb, ap, bp) = words(w);
    if PARTITION {
        kani::assume(na == NEGATIVE);
    }
    if DIVISOR_SIGN < 2 {
        kani::assume(nb == (DIVISOR_SIGN == 1));
    }

    let (x, a) = identities::known_input(aw, Some(ap));
    let (y, b) = identities::known_input(bw, Some(bp));
    kani::assume(expanded_planes(a.planes, aw, w, a.signed, true).0 == ap);
    kani::assume(expanded_planes(b.planes, bw, w, b.signed, true).0 == bp);
    signed_small::configure([ma, mb], [na, nb], w);
    power_quotient::configure(k);
    (x, a, y, b, w, ma, mb, k, na, nb, ap, bp)
}
fn check<const REM: bool, const ORIGINAL: bool, const NEGATIVE: bool, const DIVISOR_SIGN: u8>() {
    let (x, a, y, b, w, ma, mb, k, na, nb, _, _) =
        inputs::<ORIGINAL, NEGATIVE, true, DIVISOR_SIGN>();
    special::observe_binary_sources(&x, &y, w, true);
    let out = if REM { Op::Rem } else { Op::Div }.eval_value_binary(
        &x,
        &y,
        w,
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
    let expected = expected_word(ma, mb, k, na, nb, w, REM);
    preserve(
        &out,
        Operand {
            planes: (expected, 0),
            width: w,
            signed: true,
        },
    );
    let p = usize::from(kani::any::<u8>() & 127);
    result(&out, w, true, p, read_logic(expected, 0, p));
    preserve(&x, a);
    preserve(&y, b);
    if ORIGINAL {
        if NEGATIVE {
            if DIVISOR_SIGN == 1 {
                kani::cover!(
                    w == 65 && a.width == 64 && b.width == 65 && ma == 13 && mb == 3 && k == 2,
                    "signed quotient class negative negative context is reachable"
                );
            } else {
                kani::cover!(
                    w == 65 && a.width == 64 && b.width == 65 && ma == 13 && mb == 3 && k == 2,
                    "signed quotient class negative positive context is reachable"
                );
            }
        } else if DIVISOR_SIGN == 1 {
            kani::cover!(
                w == 65 && a.width == 64 && b.width == 65 && ma == 13 && mb == 3 && k == 2,
                "signed quotient class positive negative context is reachable"
            );
        } else {
            kani::cover!(
                w == 65 && a.width == 64 && b.width == 65 && ma == 13 && mb == 3 && k == 2,
                "signed quotient class positive positive context is reachable"
            );
        }
    } else {
        kani::cover!(
            w == 65 && ma == 7 && mb == 3 && k == 1,
            "signed quotient class minimum nonpower divisor is reachable"
        );
        kani::cover!(
            w == 128 && k == 64 && mb == 3,
            "signed quotient class word crossing is reachable"
        );
        kani::cover!(
            ma == mb << k,
            "signed quotient class zero remainder is reachable"
        );
        kani::cover!(
            mb > 1 && ma - (mb << k) + 1 == mb,
            "signed quotient class maximum nonzero remainder is reachable"
        );
        if NEGATIVE {
            kani::cover!(
                w == 128 && ma == 1u128 << 127 && mb == 1 && nb && k == 127,
                "signed quotient class minimum over minus one is reachable"
            );
        } else {
            kani::cover!(
                w == 128 && ma == (1u128 << 127) - 1,
                "signed quotient class maximum positive dividend is reachable"
            );
        }
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

fn signed_quotient_numeric_api_matches_byte_division() {
    init();
    let a = kani::any::<i8>();
    let b = kani::any::<i8>();
    kani::assume(b != 0);
    let quotient = a.unsigned_abs() / b.unsigned_abs();
    kani::assume(quotient.is_power_of_two());
    power_quotient::configure(quotient.trailing_zeros() as usize);
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
        "signed quotient byte API mismatch"
    );
    kani::cover!(
        a > 0 && b > 0,
        "signed quotient API positive positive is reachable"
    );
    kani::cover!(
        a > 0 && b < 0,
        "signed quotient API positive negative is reachable"
    );
    kani::cover!(
        a < 0 && b > 0,
        "signed quotient API negative positive is reachable"
    );
    kani::cover!(
        a < 0 && b < 0,
        "signed quotient API negative negative is reachable"
    );
    kani::cover!(nr == 0, "signed quotient API zero remainder is reachable");
    kani::cover!(
        a == i8::MIN && b == -1,
        "signed quotient API minimum over minus one is reachable"
    );
}
fn check_word<const REM: bool>() {
    let w = usize::from(kani::any::<u8>() & 127) + 1;
    let k = usize::from(kani::any::<u8>() & 127);
    kani::assume(k < w);
    // Quantify raw signed words directly, avoiding an encoding/decode round trip.
    // Input conversion has its separate value-preserving witness theorem.
    let mask = u128::MAX >> (128 - w);
    let ap = kani::any::<u128>() & mask;
    let bp = kani::any::<u128>() & mask;
    let padding = 128 - w;
    let a = ((ap << padding) as i128) >> padding;
    let b = ((bp << padding) as i128) >> padding;
    let ma = a.unsigned_abs();
    let mb = b.unsigned_abs();
    let na = a < 0;
    let nb = b < 0;
    kani::assume(mb != 0 && mb <= ma >> k && ma - (mb << k) < mb);
    let rem = REM;
    let p = usize::from(kani::any::<u8>() & 127);
    let mag = if rem { ma - (mb << k) } else { 1u128 << k };
    let neg = if rem { na } else { na != nb };
    let bit = if neg {
        negated_word_bit(mag, w, p)
    } else {
        power_quotient_bit(ma, mb, k, w, p, rem)
    };
    let expected = expected_word(ma, mb, k, na, nb, w, rem);
    assert!(
        expected == signed_power_quotient_word(ap, bp, k, w, rem)
            && (expected >> p & 1 != 0) == bit,
        "signed quotient word mismatch"
    );

    kani::cover!(w == 1, "signed quotient word unit width is reachable");
    kani::cover!(
        mb == 3 && k == 2,
        "signed quotient word nonpower divisor is reachable"
    );
    if REM {
        kani::cover!(
            na && mag != 0,
            "signed quotient word negative remainder is reachable"
        );
        kani::cover!(mag == 0, "signed quotient word zero remainder is reachable");
        kani::cover!(
            mb > 1 && ma - (mb << k) + 1 == mb,
            "signed quotient word maximum remainder is reachable"
        );
    } else {
        kani::cover!(
            w == 128 && ma == 1u128 << 127 && mb == 1 && nb,
            "signed quotient word minimum overflow is reachable"
        );
        kani::cover!(
            na != nb,
            "signed quotient word negative quotient is reachable"
        );
        kani::cover!(
            w == 128 && k == 64,
            "signed quotient word word crossing is reachable"
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

fn signed_quotient_quotient_word_matches_reference() {
    check_word::<false>();
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

fn signed_quotient_remainder_word_matches_reference() {
    check_word::<true>();
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

fn signed_quotient_original_domain_preserves_prepared_values() {
    // Same generator, without the sign restriction used by the two partitions.
    let (x, a, y, b, w, ma, mb, k, na, nb, ap, bp) = inputs::<true, false, false, 2>();
    special::observe_binary_sources(&x, &y, w, true);
    let xp = special::observed_expand(&x, w, true);
    let yp = special::observed_expand(&y, w, true);
    preserve(
        xp.as_ref(),
        Operand {
            planes: (ap, 0),
            width: w,
            signed: a.signed,
        },
    );
    preserve(
        yp.as_ref(),
        Operand {
            planes: (bp, 0),
            width: w,
            signed: b.signed,
        },
    );
    let p = usize::from(kani::any::<u8>() & 127);
    result(xp.as_ref(), w, a.signed, p, read_logic(ap, 0, p));
    result(yp.as_ref(), w, b.signed, p, read_logic(bp, 0, p));
    assert!(
        special::observed_calls() == 2
            && mb != 0
            && mb <= ma >> k
            && ma - (mb << k) < mb
            && ap == signed_magnitude_word(ma, na, w)
            && bp == signed_magnitude_word(mb, nb, w),
        "signed quotient original domain mismatch"
    );
    preserve(&x, a);
    preserve(&y, b);
    kani::cover!(
        a.width == 64 && b.width == 64 && w == 128,
        "signed quotient domain native native is reachable"
    );
    kani::cover!(
        a.width == 64 && b.width == 65 && w == 128,
        "signed quotient domain native big is reachable"
    );
    kani::cover!(
        a.width == 65 && b.width == 64 && w == 128,
        "signed quotient domain big native is reachable"
    );
    kani::cover!(
        a.width == 65 && b.width == 127 && w == 128,
        "signed quotient domain big big is reachable"
    );
    kani::cover!(
        a.signed && !b.signed,
        "signed quotient domain left signed only is reachable"
    );
    kani::cover!(
        !a.signed && b.signed,
        "signed quotient domain right signed only is reachable"
    );
    kani::cover!(
        w == 128 && ma == 1u128 << 127,
        "signed quotient domain minimum dividend is reachable"
    );
    kani::cover!(
        k == 127,
        "signed quotient domain largest quotient is reachable"
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

fn wide_signed_quotient_div_positive_matches_reference() {
    check::<false, false, false, 2>();
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

fn wide_signed_quotient_div_negative_matches_reference() {
    check::<false, false, true, 2>();
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

fn wide_signed_quotient_rem_positive_matches_reference() {
    check::<true, false, false, 2>();
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

fn wide_signed_quotient_rem_negative_matches_reference() {
    check::<true, false, true, 2>();
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

fn wide_signed_quotient_div_positive_positive_context_matches_reference() {
    check::<false, true, false, 0>();
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

fn wide_signed_quotient_div_positive_negative_context_matches_reference() {
    check::<false, true, false, 1>();
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

fn wide_signed_quotient_div_negative_positive_context_matches_reference() {
    check::<false, true, true, 0>();
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

fn wide_signed_quotient_div_negative_negative_context_matches_reference() {
    check::<false, true, true, 1>();
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

fn wide_signed_quotient_rem_positive_positive_context_matches_reference() {
    check::<true, true, false, 0>();
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

fn wide_signed_quotient_rem_positive_negative_context_matches_reference() {
    check::<true, true, false, 1>();
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

fn wide_signed_quotient_rem_negative_positive_context_matches_reference() {
    check::<true, true, true, 0>();
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

fn wide_signed_quotient_rem_negative_negative_context_matches_reference() {
    check::<true, true, true, 1>();
}
