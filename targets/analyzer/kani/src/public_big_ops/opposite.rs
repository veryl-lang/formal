//! IEEE1800-2023 11.4.3/11.6.1/11.8.2: known signed opposite operands.
//! Result widths65..128; nonnarrowing sources1..128 with independent flags.
//! Membership is checked after sizing: nonzero, distinct two's-complement
//! inverses. This excludes the unpaired minimum signed value and includes both
//! orientations. Zero divisors, X/Z and unequal magnitudes remain separate.
//! Actual Op and signed result constructors execute under mathematical numeric
//! APIs. Conversion and expansion compose their separately proved summaries;
//! num-bigint algorithms are not proved. Native/unsigned arithmetic branches
//! must be unreachable. Original storage witnesses use the exact input generator
//! shared with the value proofs, avoiding duplicate arithmetic solver queries.
use super::*;
use num_bigint::{BigInt, BigUint, Sign};
use veryl_formal_semantics::{nonlinear::opposite_division_word, resize::expanded_planes};

fn signed_class(a: &BigInt, b: &BigInt) {
    assert!(
        get(a.magnitude()) != N::ZERO
            && get(a.magnitude()) == get(b.magnitude())
            && a.sign() != b.sign()
            && a.sign() != Sign::NoSign
            && b.sign() != Sign::NoSign,
        "opposite signed arithmetic domain exceeded"
    );
}
fn divide(a: BigInt, b: BigInt) -> BigInt {
    signed_class(&a, &b);
    power::signed_from_numeric(Sign::Minus, from64(1))
}
fn remainder(a: BigInt, b: BigInt) -> BigInt {
    signed_class(&a, &b);
    power::signed_from_numeric(Sign::Plus, from64(0))
}
fn reject_unsigned<'a: 'a, 'b: 'b>(_: &'a BigUint, _: &'b BigUint) -> BigUint {
    panic!("opposite unsigned arithmetic boundary reached")
}
fn reject_native(
    _: &veryl_analyzer::value::ValueU64,
    _: &veryl_analyzer::value::ValueU64,
    _: usize,
    _: bool,
    _: bool,
) -> veryl_analyzer::value::ValueU64 {
    panic!("opposite native arithmetic boundary reached")
}
// Every nonzero signed opposite pair has one unique positive magnitude below
// the sign bit. Sources are uniquely its prepared words' low source-width bits;
// validating expansion below retains every original width/flag combination.
static mut MAGNITUDE: u128 = 0;
static mut PREPARED_WIDTH: usize = 0;
pub(super) fn configure_magnitude(magnitude: u128, width: usize) {
    unsafe {
        MAGNITUDE = magnitude;
        PREPARED_WIDTH = width;
    }
}
pub(super) fn witnessed_conversion(value: &ValueBigUint) -> Option<BigInt> {
    let (magnitude, width) = unsafe { (MAGNITUDE, PREPARED_WIDTH) };
    assert!(
        (65..=128).contains(&width) && magnitude != 0 && magnitude <= u128::MAX >> (129 - width),
        "opposite conversion witness domain exceeded"
    );
    let number = get(value.payload());
    let unknown = get(value.mask_xz());
    let negative = number.lo != magnitude;
    let expected = if negative {
        magnitude.wrapping_neg() & (u128::MAX >> (128 - width))
    } else {
        magnitude
    };
    assert!(
        value.width as usize == width && number == N::from_u128(expected) && unknown == N::ZERO,
        "opposite conversion witness domain exceeded"
    );
    Some(power::signed_from_numeric(
        if negative { Sign::Minus } else { Sign::Plus },
        from128(magnitude),
    ))
}
fn inputs<const ORIGINAL: bool, const ORIENTATION: u8>()
-> (Value, Operand, Value, Operand, usize, u128, u128) {
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
    let magnitude = kani::any::<u128>() & (u128::MAX >> (129 - width));
    kani::assume(magnitude != 0);
    let negative = if ORIENTATION == 0 {
        kani::any()
    } else {
        ORIENTATION == 2
    };
    let mask = u128::MAX >> (128 - width);
    let negated = magnitude.wrapping_neg() & mask;
    let ap = if negative { negated } else { magnitude };
    let bp = if negative { magnitude } else { negated };
    let (x, a) = identities::known_input(aw, Some(ap));
    let (y, b) = identities::known_input(bw, Some(bp));
    kani::assume(expanded_planes(a.planes, aw, width, a.signed, true).0 == ap);
    kani::assume(expanded_planes(b.planes, bw, width, b.signed, true).0 == bp);
    configure_magnitude(magnitude, width);
    (x, a, y, b, width, ap, bp)
}
fn check<const REM: bool, const ORIGINAL: bool, const ORIENTATION: u8>() {
    let (x, a, y, b, width, ap, bp) = inputs::<ORIGINAL, ORIENTATION>();
    special::observe_binary_sources(&x, &y, width, true);
    let output = if REM { Op::Rem } else { Op::Div }.eval_value_binary(
        &x,
        &y,
        width,
        true,
        &mut MaskCache::default(),
    );
    if ORIGINAL {
        assert!(
            special::observed_calls() == 2,
            "wide operator semantics mismatch"
        );
    }
    // The independently checked opposite-word/conversion theorem justifies
    // this class summary; all source values and actual conversion inputs remain
    // checked against the magnitude witness, not replaced by fixed operands.
    let expected = if REM { 0 } else { u128::MAX >> (128 - width) };
    preserve(
        &output,
        Operand {
            planes: (expected, 0),
            width,
            signed: true,
        },
    );
    let position = usize::from(kani::any::<u8>() & 127);
    result(
        &output,
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
                width == 65 && ap == 1,
                "opposite class positive context is reachable"
            );
        } else {
            kani::cover!(
                width == 65 && ap == u128::MAX >> (128 - width),
                "opposite class negative context is reachable"
            );
        }
    } else {
        kani::cover!(
            width == 65 && ap >> (width - 1) == 0,
            "opposite class minimum positive dividend is reachable"
        );
        kani::cover!(
            width == 128 && ap >> 127 != 0,
            "opposite class maximum negative dividend is reachable"
        );
        kani::cover!(
            ap == u128::MAX >> 1 && width == 128,
            "opposite class maximum magnitude is reachable"
        );
        kani::cover!(
            a.signed != b.signed,
            "opposite class mixed source signs is reachable"
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
#[kani::stub(
    veryl_analyzer::value::ValueBigUint::to_bigint,
    super::ordering::converted
)]
#[kani::stub(<num_bigint::BigInt as std::ops::Div<num_bigint::BigInt>>::div, divide)]
#[kani::stub(<num_bigint::BigInt as std::ops::Rem<num_bigint::BigInt>>::rem, remainder)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, reject_unsigned)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, reject_unsigned)]
#[kani::stub(veryl_analyzer::value::ValueU64::divide_remainder, reject_native)]
fn numeric_opposite_division_matches_byte_arithmetic() {
    init();
    let n = kani::any::<i8>();
    kani::assume(n != 0);
    let sign = if n < 0 { Sign::Minus } else { Sign::Plus };
    let other = if n < 0 { Sign::Plus } else { Sign::Minus };
    let x = power::signed_from_numeric(sign, from64(u64::from(n.unsigned_abs())));
    let y = power::signed_from_numeric(other, from64(u64::from(n.unsigned_abs())));
    let q = x.clone() / y.clone();
    let r = x.clone() % y.clone();
    let a = i16::from(n);
    let b = -a;
    assert!(
        q.sign() == Sign::Minus
            && get(q.magnitude()) == N::from_u128((a / b).unsigned_abs() as u128)
            && r.sign() == Sign::NoSign
            && get(r.magnitude()) == N::from_u128((a % b).unsigned_abs() as u128)
            && x.sign() == sign
            && y.sign() == other
            && get(x.magnitude()) == N::from_u128(u128::from(n.unsigned_abs()))
            && get(y.magnitude()) == get(x.magnitude()),
        "opposite numeric API mismatch"
    );
    kani::cover!(n > 0, "opposite API positive dividend is reachable");
    kani::cover!(n < 0, "opposite API negative dividend is reachable");
    kani::cover!(
        n == i8::MIN,
        "opposite API byte minimum magnitude is reachable"
    );
}
#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(
    crate::public_big_ops::ordering::from_biguint,
    super::power::signed_from_numeric
)]
#[kani::stub(
    veryl_analyzer::value::ValueBigUint::to_bigint,
    super::ordering::converted
)]
#[kani::stub(<num_bigint::BigInt as std::ops::Div<num_bigint::BigInt>>::div, divide)]
#[kani::stub(<num_bigint::BigInt as std::ops::Rem<num_bigint::BigInt>>::rem, remainder)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, reject_unsigned)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, reject_unsigned)]
#[kani::stub(veryl_analyzer::value::ValueU64::divide_remainder, reject_native)]
fn opposite_original_domain_preserves_prepared_values() {
    let (x, a, y, b, width, ap, bp) = inputs::<true, 0>();
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
    let position = usize::from(kani::any::<u8>() & 127);
    result(
        xp.as_ref(),
        width,
        a.signed,
        position,
        read_logic(ap, 0, position),
    );
    result(
        yp.as_ref(),
        width,
        b.signed,
        position,
        read_logic(bp, 0, position),
    );
    assert!(
        special::observed_calls() == 2 && opposite_division_word(ap, bp, width, true) == 0,
        "opposite context domain mismatch"
    );
    preserve(&x, a);
    preserve(&y, b);
    // Same generator as both context value proofs. Keep the additional domain
    // witnesses here to avoid repeated queries through signed arithmetic.
    kani::cover!(
        width == 65 && ap >> (width - 1) == 0,
        "opposite class minimum positive dividend is reachable"
    );
    kani::cover!(
        width == 128 && ap >> 127 != 0,
        "opposite class maximum negative dividend is reachable"
    );
    kani::cover!(
        ap == u128::MAX >> 1 && width == 128,
        "opposite class maximum magnitude is reachable"
    );
    kani::cover!(
        a.signed != b.signed,
        "opposite class mixed source signs is reachable"
    );
    kani::cover!(
        a.width == 64 && b.width == 64 && width == 128,
        "opposite class native native sources is reachable"
    );
    kani::cover!(
        a.width == 64 && b.width == 65 && width == 128,
        "opposite class native big sources is reachable"
    );
    kani::cover!(
        a.width == 65 && b.width == 64 && width == 128,
        "opposite class big native sources is reachable"
    );
    kani::cover!(
        a.width == 65 && b.width == 127 && width == 128,
        "opposite class big big sources is reachable"
    );
    kani::cover!(
        a.width == width && b.width == width,
        "opposite class prepared sources is reachable"
    );
}
#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(
    crate::public_big_ops::ordering::from_biguint,
    super::power::signed_from_numeric
)]
#[kani::stub(veryl_analyzer::value::ValueBigUint::to_bigint, witnessed_conversion)]
#[kani::stub(<num_bigint::BigInt as std::ops::Div<num_bigint::BigInt>>::div, divide)]
#[kani::stub(<num_bigint::BigInt as std::ops::Rem<num_bigint::BigInt>>::rem, remainder)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, reject_unsigned)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, reject_unsigned)]
#[kani::stub(veryl_analyzer::value::ValueU64::divide_remainder, reject_native)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::prepared_wide)]
fn wide_opposite_div_matches_reference() {
    check::<false, false, 0>();
}
#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(
    crate::public_big_ops::ordering::from_biguint,
    super::power::signed_from_numeric
)]
#[kani::stub(veryl_analyzer::value::ValueBigUint::to_bigint, witnessed_conversion)]
#[kani::stub(<num_bigint::BigInt as std::ops::Div<num_bigint::BigInt>>::div, divide)]
#[kani::stub(<num_bigint::BigInt as std::ops::Rem<num_bigint::BigInt>>::rem, remainder)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, reject_unsigned)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, reject_unsigned)]
#[kani::stub(veryl_analyzer::value::ValueU64::divide_remainder, reject_native)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::special::observed_expand)]
fn wide_opposite_div_positive_context_matches_reference() {
    check::<false, true, 1>();
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(
    crate::public_big_ops::ordering::from_biguint,
    super::power::signed_from_numeric
)]
#[kani::stub(veryl_analyzer::value::ValueBigUint::to_bigint, witnessed_conversion)]
#[kani::stub(<num_bigint::BigInt as std::ops::Div<num_bigint::BigInt>>::div, divide)]
#[kani::stub(<num_bigint::BigInt as std::ops::Rem<num_bigint::BigInt>>::rem, remainder)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, reject_unsigned)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, reject_unsigned)]
#[kani::stub(veryl_analyzer::value::ValueU64::divide_remainder, reject_native)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::special::observed_expand)]
fn wide_opposite_div_negative_context_matches_reference() {
    check::<false, true, 2>();
}
#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(
    crate::public_big_ops::ordering::from_biguint,
    super::power::signed_from_numeric
)]
#[kani::stub(veryl_analyzer::value::ValueBigUint::to_bigint, witnessed_conversion)]
#[kani::stub(<num_bigint::BigInt as std::ops::Div<num_bigint::BigInt>>::div, divide)]
#[kani::stub(<num_bigint::BigInt as std::ops::Rem<num_bigint::BigInt>>::rem, remainder)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, reject_unsigned)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, reject_unsigned)]
#[kani::stub(veryl_analyzer::value::ValueU64::divide_remainder, reject_native)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::prepared_wide)]
fn wide_opposite_rem_matches_reference() {
    check::<true, false, 0>();
}
#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(
    crate::public_big_ops::ordering::from_biguint,
    super::power::signed_from_numeric
)]
#[kani::stub(veryl_analyzer::value::ValueBigUint::to_bigint, witnessed_conversion)]
#[kani::stub(<num_bigint::BigInt as std::ops::Div<num_bigint::BigInt>>::div, divide)]
#[kani::stub(<num_bigint::BigInt as std::ops::Rem<num_bigint::BigInt>>::rem, remainder)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, reject_unsigned)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, reject_unsigned)]
#[kani::stub(veryl_analyzer::value::ValueU64::divide_remainder, reject_native)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::special::observed_expand)]
fn wide_opposite_rem_positive_context_matches_reference() {
    check::<true, true, 1>();
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(
    crate::public_big_ops::ordering::from_biguint,
    super::power::signed_from_numeric
)]
#[kani::stub(veryl_analyzer::value::ValueBigUint::to_bigint, witnessed_conversion)]
#[kani::stub(<num_bigint::BigInt as std::ops::Div<num_bigint::BigInt>>::div, divide)]
#[kani::stub(<num_bigint::BigInt as std::ops::Rem<num_bigint::BigInt>>::rem, remainder)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, reject_unsigned)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, reject_unsigned)]
#[kani::stub(veryl_analyzer::value::ValueU64::divide_remainder, reject_native)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::special::observed_expand)]
fn wide_opposite_rem_negative_context_matches_reference() {
    check::<true, true, 2>();
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(
    crate::public_big_ops::ordering::from_biguint,
    super::power::signed_from_numeric
)]
fn opposite_conversion_witness_matches_signed_decoding() {
    init();
    let width = usize::from(kani::any::<u8>() & 63) + 65;
    let magnitude = kani::any::<u128>() & (u128::MAX >> (129 - width));
    kani::assume(magnitude != 0);
    configure_magnitude(magnitude, width);
    let negative = magnitude.wrapping_neg() & (u128::MAX >> (128 - width));
    let (x, a) = identities::known_input(width, Some(magnitude));
    let (y, b) = identities::known_input(width, Some(negative));
    let Value::BigUint(xb) = &x else {
        panic!("opposite conversion representation mismatch")
    };
    let Value::BigUint(yb) = &y else {
        panic!("opposite conversion representation mismatch")
    };
    let decoded_x = ordering::converted(xb).unwrap();
    let decoded_y = ordering::converted(yb).unwrap();
    let witnessed_x = witnessed_conversion(xb).unwrap();
    let witnessed_y = witnessed_conversion(yb).unwrap();
    assert!(
        decoded_x.sign() == Sign::Plus
            && decoded_y.sign() == Sign::Minus
            && witnessed_x.sign() == decoded_x.sign()
            && witnessed_y.sign() == decoded_y.sign()
            && get(decoded_x.magnitude()) == N::from_u128(magnitude)
            && get(decoded_y.magnitude()) == N::from_u128(magnitude)
            && get(witnessed_x.magnitude()) == get(decoded_x.magnitude())
            && get(witnessed_y.magnitude()) == get(decoded_y.magnitude())
            && opposite_division_word(magnitude, negative, width, false)
                == u128::MAX >> (128 - width)
            && opposite_division_word(magnitude, negative, width, true) == 0
            && opposite_division_word(negative, magnitude, width, false)
                == u128::MAX >> (128 - width)
            && opposite_division_word(negative, magnitude, width, true) == 0,
        "opposite conversion witness mismatch"
    );
    preserve(&x, a);
    preserve(&y, b);
    kani::cover!(
        width == 65,
        "opposite conversion minimum width is reachable"
    );
    kani::cover!(
        width == 128,
        "opposite conversion maximum width is reachable"
    );
    kani::cover!(
        magnitude == 1,
        "opposite conversion unit magnitude is reachable"
    );
    kani::cover!(
        magnitude == u128::MAX >> 1,
        "opposite conversion maximum magnitude is reachable"
    );
}
