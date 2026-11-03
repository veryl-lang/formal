//! IEEE1800-2023 11.4.3/11.8.2: bounded known division classes.
//! Prepared widths65..128; independent original widths1..128 and flags, with
//! nonnarrowing contexts. Equal nonzero prepared operands allow either mode.
//! Smaller numerators and single-hot divisors require unsigned mode. Zero
//! divisors and X/Z are outside these contracts and covered separately.
//! Actual Op and signed result constructors execute under guarded mathematical
//! numeric APIs. Conversion and expansion compose their separately proved
//! summaries. These contracts do not prove num-bigint's arithmetic algorithms.
//! Byte arithmetic checks the new adapters; arbitrary256-bit coordinate checks
//! validate the single-hot specialization's complete bounded representation.
use super::*;
use num_bigint::{BigInt, BigUint, Sign};
use veryl_formal_semantics::{
    nonlinear::{DivisionClass, division_class_bit},
    resize::expanded_planes,
};

fn unsigned_class(x: N, y: N) {
    assert!(
        y != N::ZERO && (!x.unsigned_cmp(y).is_gt() || (y.hi == 0 && y.lo.is_power_of_two())),
        "division numeric class domain exceeded"
    );
}
fn divide<'a: 'a, 'b: 'b>(a: &'a BigUint, b: &'b BigUint) -> BigUint {
    let x = get(a);
    let y = get(b);
    unsigned_class(x, y);
    let n = if x == y {
        N::from_u128(1)
    } else if x.unsigned_cmp(y).is_lt() {
        N::ZERO
    } else {
        x.shr(y.lo.trailing_zeros() as usize)
    };
    put(n)
}
fn remainder<'a: 'a, 'b: 'b>(a: &'a BigUint, b: &'b BigUint) -> BigUint {
    let x = get(a);
    let y = get(b);
    unsigned_class(x, y);
    let n = if x == y {
        N::ZERO
    } else if x.unsigned_cmp(y).is_lt() {
        x
    } else {
        x.and(N::mask(y.lo.trailing_zeros() as usize))
    };
    put(n)
}
fn signed_class(a: &BigInt, b: &BigInt) {
    assert!(
        get(a.magnitude()) != N::ZERO
            && get(a.magnitude()) == get(b.magnitude())
            && a.sign() == b.sign(),
        "division signed class domain exceeded"
    );
}
fn signed_divide(a: BigInt, b: BigInt) -> BigInt {
    signed_class(&a, &b);
    power::signed_from_numeric(Sign::Plus, from64(1))
}
fn signed_remainder(a: BigInt, b: BigInt) -> BigInt {
    signed_class(&a, &b);
    power::signed_from_numeric(Sign::Plus, from64(0))
}
fn reject_native(
    _: &veryl_analyzer::value::ValueU64,
    _: &veryl_analyzer::value::ValueU64,
    _: usize,
    _: bool,
    _: bool,
) -> veryl_analyzer::value::ValueU64 {
    panic!("division native boundary reached")
}

fn scalar_bit(n: N, position: usize) -> bool {
    position < 256 && [n.lo, n.hi][position / 128] >> (position % 128) & 1 != 0
}

// K0/1 equal div/rem; K2/3 smaller unsigned div/rem; K4/5 single-hot div/rem.
fn division_inputs<const ORIGINAL: bool, const SIGNED: bool>(
    class: u8,
) -> (
    Value,
    Operand,
    Value,
    Operand,
    usize,
    bool,
    u128,
    u128,
    usize,
) {
    init();
    assert!(class < 3, "division input class domain exceeded");
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
    let signed = class == 0 && SIGNED;
    let (x, a) = identities::known_input(aw, None);
    let ap = expanded_planes(a.planes, aw, width, a.signed, signed).0;
    let shift = usize::from(kani::any::<u8>() & 127);
    // For equal prepared words the RHS source is uniquely ap's low bw bits.
    // The class condition below validates sign extension, retaining every
    // normalized equal-value source pair rather than equating widths or flags.
    let seed = if class == 0 {
        Some(ap)
    } else if class == 2 {
        Some(1u128 << shift)
    } else {
        None
    };
    let (y, b) = identities::known_input(bw, seed);
    let bp = expanded_planes(b.planes, bw, width, b.signed, signed).0;
    if class == 0 {
        kani::assume(ap == bp && ap != 0);
    } else if class == 1 {
        kani::assume(ap < bp);
    } else {
        kani::assume(bp == 1u128 << shift);
    }
    (x, a, y, b, width, signed, ap, bp, shift)
}

fn check<const K: u8, const ORIGINAL: bool, const SIGNED: bool>() {
    let (x, a, y, b, width, signed, ap, bp, shift) = division_inputs::<ORIGINAL, SIGNED>(K / 2);
    let (aw, bw) = (a.width, b.width);
    special::observe_binary_sources(&x, &y, width, signed);
    let rem = K % 2 == 1;
    let output = if rem { Op::Rem } else { Op::Div }.eval_value_binary(
        &x,
        &y,
        width,
        signed,
        &mut MaskCache::default(),
    );
    if ORIGINAL {
        assert!(
            special::observed_calls() == 2,
            "wide operator semantics mismatch"
        );
    }
    let position = usize::from(kani::any::<u8>() & 127);
    let class = if K < 2 {
        DivisionClass::Equal
    } else if K < 4 {
        DivisionClass::UnsignedSmaller
    } else {
        DivisionClass::UnsignedPowerOfTwo
    };
    let bit = division_class_bit(ap, bp, width, signed, position, rem, class);
    assert!(
        output.mask_xz_u128() == 0,
        "wide operator semantics mismatch"
    );
    result(
        &output,
        width,
        signed,
        position,
        if bit { Logic4::One } else { Logic4::Zero },
    );
    preserve(&x, a);
    preserve(&y, b);
    // A compact set of combined witnesses avoids repeated solver queries for
    // separately named cases already implied by the same class witness.
    kani::cover!(width == 65, "division class minimum width is reachable");
    kani::cover!(width == 128, "division class maximum width is reachable");
    kani::cover!(
        a.signed && !b.signed,
        "division class mixed source signs is reachable"
    );
    if K < 2 {
        kani::cover!(
            signed && ap == 1u128 << 127,
            "division class minimum signed self is reachable"
        );
        kani::cover!(
            !signed && ap > 1,
            "division class unsigned nonunit self is reachable"
        );
    } else if K < 4 {
        kani::cover!(
            ap == 0,
            "division class zero smaller numerator is reachable"
        );
        kani::cover!(
            ap > 0 && bp >> 127 != 0,
            "division class upper smaller divisor is reachable"
        );
    } else {
        kani::cover!(shift == 0, "division class divisor one is reachable");
        kani::cover!(
            shift == 64 && ap >> 64 != 0,
            "division class word crossing is reachable"
        );
        kani::cover!(
            shift == 127 && ap >> 127 != 0,
            "division class top divisor bit is reachable"
        );
    }
    if ORIGINAL && !(K < 2 && SIGNED) {
        kani::cover!(
            aw == 64 && bw == 64 && width == 128,
            "division class native native sources is reachable"
        );
        kani::cover!(
            aw == 64 && bw == 65 && width == 128,
            "division class native big sources is reachable"
        );
        kani::cover!(
            aw == 65 && bw == 64 && width == 128,
            "division class big native sources is reachable"
        );
        kani::cover!(
            aw == 65 && bw == 127 && width == 128,
            "division class big big sources is reachable"
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
#[kani::stub(<num_bigint::BigInt as std::ops::Div<num_bigint::BigInt>>::div, signed_divide)]
#[kani::stub(<num_bigint::BigInt as std::ops::Rem<num_bigint::BigInt>>::rem, signed_remainder)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, divide)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, remainder)]
#[kani::stub(veryl_analyzer::value::ValueU64::divide_remainder, reject_native)]
fn numeric_division_classes_match_byte_arithmetic() {
    init();
    let a = kani::any::<u8>();
    let b = kani::any::<u8>();
    kani::assume(b != 0 && (a <= b || b.is_power_of_two()));
    let x = from64(u64::from(a));
    let y = from64(u64::from(b));
    let q = &x / &y;
    let r = &x % &y;
    assert!(
        get(&q) == N::from_u128(u128::from(a / b))
            && get(&r) == N::from_u128(u128::from(a % b))
            && get(&x) == N::from_u128(u128::from(a))
            && get(&y) == N::from_u128(u128::from(b)),
        "division numeric API mismatch"
    );
    let n = kani::any::<i8>();
    kani::assume(n != 0);
    let input = power::signed_from_numeric(
        if n < 0 { Sign::Minus } else { Sign::Plus },
        from64(u64::from(n.unsigned_abs())),
    );
    let sq = input.clone() / input.clone();
    let sr = input.clone() % input.clone();
    assert!(
        get(sq.magnitude()) == N::from_u128(n.wrapping_div(n) as u128)
            && sq.sign() == Sign::Plus
            && get(sr.magnitude()) == N::from_u128(n.wrapping_rem(n) as u128)
            && sr.sign() == Sign::NoSign
            && get(input.magnitude()) == N::from_u128(u128::from(n.unsigned_abs()))
            && (input.sign() == Sign::Minus) == (n < 0),
        "division numeric API mismatch"
    );
    kani::cover!(a == b, "division API equal words are reachable");
    kani::cover!(a > 0 && a < b, "division API smaller words are reachable");
    kani::cover!(
        a == 255 && b == 128,
        "division API byte top bit is reachable"
    );
    kani::cover!(n == i8::MIN, "division API negative self is reachable");
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
#[kani::stub(<num_bigint::BigInt as std::ops::Div<num_bigint::BigInt>>::div, signed_divide)]
#[kani::stub(<num_bigint::BigInt as std::ops::Rem<num_bigint::BigInt>>::rem, signed_remainder)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, divide)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, remainder)]
#[kani::stub(veryl_analyzer::value::ValueU64::divide_remainder, reject_native)]
fn numeric_power_two_division_matches_full_word_coordinates() {
    init();
    let number = N {
        lo: kani::any(),
        hi: kani::any(),
    };
    let shift = usize::from(kani::any::<u8>() & 127);
    let x = put(number);
    let y = put(N::from_u128(1).checked_shl(shift));
    let q = &x / &y;
    let r = &x % &y;
    let position = usize::from(kani::any::<u8>());
    assert!(
        scalar_bit(get(&q), position) == scalar_bit(number, position + shift)
            && scalar_bit(get(&r), position) == (position < shift && scalar_bit(number, position))
            && get(&x) == number
            && get(&y) == N::from_u128(1).checked_shl(shift),
        "division numeric API mismatch"
    );
    kani::cover!(shift == 0, "division API zero shift is reachable");
    kani::cover!(
        shift == 64 && number.hi != 0,
        "division API high word crossing is reachable"
    );
    kani::cover!(
        shift == 127 && number.hi >> 127 != 0,
        "division API bit255 is reachable"
    );
    kani::cover!(number == N::ZERO, "division API zero number is reachable");
}

#[kani::proof]
#[kani::unwind(129)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(
    crate::public_big_ops::ordering::from_biguint,
    super::power::signed_from_numeric
)]
#[kani::stub(
    veryl_analyzer::value::ValueBigUint::to_bigint,
    super::ordering::converted
)]
#[kani::stub(<num_bigint::BigInt as std::ops::Div<num_bigint::BigInt>>::div, signed_divide)]
#[kani::stub(<num_bigint::BigInt as std::ops::Rem<num_bigint::BigInt>>::rem, signed_remainder)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, divide)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, remainder)]
#[kani::stub(veryl_analyzer::value::ValueU64::divide_remainder, reject_native)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::prepared_wide)]
fn wide_division_self_div_unsigned_prepared_matches_reference() {
    check::<0, false, false>();
}

#[kani::proof]
#[kani::unwind(129)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(
    crate::public_big_ops::ordering::from_biguint,
    super::power::signed_from_numeric
)]
#[kani::stub(
    veryl_analyzer::value::ValueBigUint::to_bigint,
    super::ordering::converted
)]
#[kani::stub(<num_bigint::BigInt as std::ops::Div<num_bigint::BigInt>>::div, signed_divide)]
#[kani::stub(<num_bigint::BigInt as std::ops::Rem<num_bigint::BigInt>>::rem, signed_remainder)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, divide)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, remainder)]
#[kani::stub(veryl_analyzer::value::ValueU64::divide_remainder, reject_native)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::special::observed_expand)]
fn wide_division_self_div_unsigned_context_matches_reference() {
    check::<0, true, false>();
}

#[kani::proof]
#[kani::unwind(129)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(
    crate::public_big_ops::ordering::from_biguint,
    super::power::signed_from_numeric
)]
#[kani::stub(
    veryl_analyzer::value::ValueBigUint::to_bigint,
    super::ordering::converted
)]
#[kani::stub(<num_bigint::BigInt as std::ops::Div<num_bigint::BigInt>>::div, signed_divide)]
#[kani::stub(<num_bigint::BigInt as std::ops::Rem<num_bigint::BigInt>>::rem, signed_remainder)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, divide)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, remainder)]
#[kani::stub(veryl_analyzer::value::ValueU64::divide_remainder, reject_native)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::prepared_wide)]
fn wide_division_self_div_signed_prepared_matches_reference() {
    check::<0, false, true>();
}

#[kani::proof]
#[kani::unwind(129)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(
    crate::public_big_ops::ordering::from_biguint,
    super::power::signed_from_numeric
)]
#[kani::stub(
    veryl_analyzer::value::ValueBigUint::to_bigint,
    super::ordering::converted
)]
#[kani::stub(<num_bigint::BigInt as std::ops::Div<num_bigint::BigInt>>::div, signed_divide)]
#[kani::stub(<num_bigint::BigInt as std::ops::Rem<num_bigint::BigInt>>::rem, signed_remainder)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, divide)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, remainder)]
#[kani::stub(veryl_analyzer::value::ValueU64::divide_remainder, reject_native)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::special::observed_expand)]
fn wide_division_self_div_signed_context_matches_reference() {
    check::<0, true, true>();
}

#[kani::proof]
#[kani::unwind(129)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(
    crate::public_big_ops::ordering::from_biguint,
    super::power::signed_from_numeric
)]
#[kani::stub(
    veryl_analyzer::value::ValueBigUint::to_bigint,
    super::ordering::converted
)]
#[kani::stub(<num_bigint::BigInt as std::ops::Div<num_bigint::BigInt>>::div, signed_divide)]
#[kani::stub(<num_bigint::BigInt as std::ops::Rem<num_bigint::BigInt>>::rem, signed_remainder)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, divide)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, remainder)]
#[kani::stub(veryl_analyzer::value::ValueU64::divide_remainder, reject_native)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::prepared_wide)]
fn wide_division_self_rem_unsigned_prepared_matches_reference() {
    check::<1, false, false>();
}

#[kani::proof]
#[kani::unwind(129)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(
    crate::public_big_ops::ordering::from_biguint,
    super::power::signed_from_numeric
)]
#[kani::stub(
    veryl_analyzer::value::ValueBigUint::to_bigint,
    super::ordering::converted
)]
#[kani::stub(<num_bigint::BigInt as std::ops::Div<num_bigint::BigInt>>::div, signed_divide)]
#[kani::stub(<num_bigint::BigInt as std::ops::Rem<num_bigint::BigInt>>::rem, signed_remainder)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, divide)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, remainder)]
#[kani::stub(veryl_analyzer::value::ValueU64::divide_remainder, reject_native)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::special::observed_expand)]
fn wide_division_self_rem_unsigned_context_matches_reference() {
    check::<1, true, false>();
}

#[kani::proof]
#[kani::unwind(129)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(
    crate::public_big_ops::ordering::from_biguint,
    super::power::signed_from_numeric
)]
#[kani::stub(
    veryl_analyzer::value::ValueBigUint::to_bigint,
    super::ordering::converted
)]
#[kani::stub(<num_bigint::BigInt as std::ops::Div<num_bigint::BigInt>>::div, signed_divide)]
#[kani::stub(<num_bigint::BigInt as std::ops::Rem<num_bigint::BigInt>>::rem, signed_remainder)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, divide)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, remainder)]
#[kani::stub(veryl_analyzer::value::ValueU64::divide_remainder, reject_native)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::prepared_wide)]
fn wide_division_self_rem_signed_prepared_matches_reference() {
    check::<1, false, true>();
}

#[kani::proof]
#[kani::unwind(129)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(
    crate::public_big_ops::ordering::from_biguint,
    super::power::signed_from_numeric
)]
#[kani::stub(
    veryl_analyzer::value::ValueBigUint::to_bigint,
    super::ordering::converted
)]
#[kani::stub(<num_bigint::BigInt as std::ops::Div<num_bigint::BigInt>>::div, signed_divide)]
#[kani::stub(<num_bigint::BigInt as std::ops::Rem<num_bigint::BigInt>>::rem, signed_remainder)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, divide)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, remainder)]
#[kani::stub(veryl_analyzer::value::ValueU64::divide_remainder, reject_native)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::special::observed_expand)]
fn wide_division_self_rem_signed_context_matches_reference() {
    check::<1, true, true>();
}

#[kani::proof]
#[kani::unwind(129)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(
    crate::public_big_ops::ordering::from_biguint,
    super::power::signed_from_numeric
)]
#[kani::stub(
    veryl_analyzer::value::ValueBigUint::to_bigint,
    super::ordering::converted
)]
#[kani::stub(<num_bigint::BigInt as std::ops::Div<num_bigint::BigInt>>::div, signed_divide)]
#[kani::stub(<num_bigint::BigInt as std::ops::Rem<num_bigint::BigInt>>::rem, signed_remainder)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, divide)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, remainder)]
#[kani::stub(veryl_analyzer::value::ValueU64::divide_remainder, reject_native)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::prepared_wide)]
fn wide_division_smaller_div_prepared_matches_reference() {
    check::<2, false, false>();
}

#[kani::proof]
#[kani::unwind(129)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(
    crate::public_big_ops::ordering::from_biguint,
    super::power::signed_from_numeric
)]
#[kani::stub(
    veryl_analyzer::value::ValueBigUint::to_bigint,
    super::ordering::converted
)]
#[kani::stub(<num_bigint::BigInt as std::ops::Div<num_bigint::BigInt>>::div, signed_divide)]
#[kani::stub(<num_bigint::BigInt as std::ops::Rem<num_bigint::BigInt>>::rem, signed_remainder)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, divide)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, remainder)]
#[kani::stub(veryl_analyzer::value::ValueU64::divide_remainder, reject_native)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::special::observed_expand)]
fn wide_division_smaller_div_context_matches_reference() {
    check::<2, true, false>();
}

#[kani::proof]
#[kani::unwind(129)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(
    crate::public_big_ops::ordering::from_biguint,
    super::power::signed_from_numeric
)]
#[kani::stub(
    veryl_analyzer::value::ValueBigUint::to_bigint,
    super::ordering::converted
)]
#[kani::stub(<num_bigint::BigInt as std::ops::Div<num_bigint::BigInt>>::div, signed_divide)]
#[kani::stub(<num_bigint::BigInt as std::ops::Rem<num_bigint::BigInt>>::rem, signed_remainder)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, divide)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, remainder)]
#[kani::stub(veryl_analyzer::value::ValueU64::divide_remainder, reject_native)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::prepared_wide)]
fn wide_division_smaller_rem_prepared_matches_reference() {
    check::<3, false, false>();
}

#[kani::proof]
#[kani::unwind(129)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(
    crate::public_big_ops::ordering::from_biguint,
    super::power::signed_from_numeric
)]
#[kani::stub(
    veryl_analyzer::value::ValueBigUint::to_bigint,
    super::ordering::converted
)]
#[kani::stub(<num_bigint::BigInt as std::ops::Div<num_bigint::BigInt>>::div, signed_divide)]
#[kani::stub(<num_bigint::BigInt as std::ops::Rem<num_bigint::BigInt>>::rem, signed_remainder)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, divide)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, remainder)]
#[kani::stub(veryl_analyzer::value::ValueU64::divide_remainder, reject_native)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::special::observed_expand)]
fn wide_division_smaller_rem_context_matches_reference() {
    check::<3, true, false>();
}

#[kani::proof]
#[kani::unwind(129)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(
    crate::public_big_ops::ordering::from_biguint,
    super::power::signed_from_numeric
)]
#[kani::stub(
    veryl_analyzer::value::ValueBigUint::to_bigint,
    super::ordering::converted
)]
#[kani::stub(<num_bigint::BigInt as std::ops::Div<num_bigint::BigInt>>::div, signed_divide)]
#[kani::stub(<num_bigint::BigInt as std::ops::Rem<num_bigint::BigInt>>::rem, signed_remainder)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, divide)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, remainder)]
#[kani::stub(veryl_analyzer::value::ValueU64::divide_remainder, reject_native)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::prepared_wide)]
fn wide_division_pow2_div_prepared_matches_reference() {
    check::<4, false, false>();
}

#[kani::proof]
#[kani::unwind(129)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(
    crate::public_big_ops::ordering::from_biguint,
    super::power::signed_from_numeric
)]
#[kani::stub(
    veryl_analyzer::value::ValueBigUint::to_bigint,
    super::ordering::converted
)]
#[kani::stub(<num_bigint::BigInt as std::ops::Div<num_bigint::BigInt>>::div, signed_divide)]
#[kani::stub(<num_bigint::BigInt as std::ops::Rem<num_bigint::BigInt>>::rem, signed_remainder)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, divide)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, remainder)]
#[kani::stub(veryl_analyzer::value::ValueU64::divide_remainder, reject_native)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::special::observed_expand)]
fn wide_division_pow2_div_context_matches_reference() {
    check::<4, true, false>();
}

#[kani::proof]
#[kani::unwind(129)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(
    crate::public_big_ops::ordering::from_biguint,
    super::power::signed_from_numeric
)]
#[kani::stub(
    veryl_analyzer::value::ValueBigUint::to_bigint,
    super::ordering::converted
)]
#[kani::stub(<num_bigint::BigInt as std::ops::Div<num_bigint::BigInt>>::div, signed_divide)]
#[kani::stub(<num_bigint::BigInt as std::ops::Rem<num_bigint::BigInt>>::rem, signed_remainder)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, divide)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, remainder)]
#[kani::stub(veryl_analyzer::value::ValueU64::divide_remainder, reject_native)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::prepared_wide)]
fn wide_division_pow2_rem_prepared_matches_reference() {
    check::<5, false, false>();
}

#[kani::proof]
#[kani::unwind(129)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(
    crate::public_big_ops::ordering::from_biguint,
    super::power::signed_from_numeric
)]
#[kani::stub(
    veryl_analyzer::value::ValueBigUint::to_bigint,
    super::ordering::converted
)]
#[kani::stub(<num_bigint::BigInt as std::ops::Div<num_bigint::BigInt>>::div, signed_divide)]
#[kani::stub(<num_bigint::BigInt as std::ops::Rem<num_bigint::BigInt>>::rem, signed_remainder)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, divide)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, remainder)]
#[kani::stub(veryl_analyzer::value::ValueU64::divide_remainder, reject_native)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::special::observed_expand)]
fn wide_division_pow2_rem_context_matches_reference() {
    check::<5, true, false>();
}

// The signed div/rem value proofs use exactly this class0 input generator.
// Check its complete prepared representation and storage reachability here,
// avoiding four repeated solver queries through each signed arithmetic body.
#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
fn signed_self_context_domain_preserves_prepared_inputs() {
    let (x, a, y, b, width, signed, ap, bp, _) = division_inputs::<true, true>(0);
    special::observe_binary_sources(&x, &y, width, signed);
    let xp = special::observed_expand(&x, width, signed);
    let yp = special::observed_expand(&y, width, signed);
    let position = usize::from(kani::any::<u8>() & 127);
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
        xp.payload_u128() == yp.payload_u128()
            && xp.payload_u128() != 0
            && special::observed_calls() == 2,
        "division context domain mismatch"
    );
    preserve(&x, a);
    preserve(&y, b);
    kani::cover!(
        a.width == 64 && b.width == 64 && width == 128,
        "division class native native sources is reachable"
    );
    kani::cover!(
        a.width == 64 && b.width == 65 && width == 128,
        "division class native big sources is reachable"
    );
    kani::cover!(
        a.width == 65 && b.width == 64 && width == 128,
        "division class big native sources is reachable"
    );
    kani::cover!(
        a.width == 65 && b.width == 127 && width == 128,
        "division class big big sources is reachable"
    );
}
