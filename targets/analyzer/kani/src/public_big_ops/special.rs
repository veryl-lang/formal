//! IEEE1800-2023 11.4.3/Table11-4 and 11.8.2: known arithmetic identities.
//! Prepared widths65..128; sources1..128, nonnarrowing contexts and independent
//! source flags. Division covers signed +/-1 and zero over any nonzero divisor
//! in either mode. Power covers exponent +1 or prepared base0/1 and any positive
//! self-determined exponent1..128 bits. No X/Z or general arithmetic theorem.
//! Actual Op, new_bigint and pow_mod_width bodies execute under mathematical
//! numeric APIs. These assumptions do not prove the num-bigint implementation.
//! Signed conversion and expansion compose their separately checked summaries.
//! The scalar two's-complement and integral power identities are independent
//! specification oracles; real-library boundaries and general cases stay direct.
use super::*;
use num_bigint::{BigInt, BigUint, Sign};
use veryl_formal_semantics::resize::expanded_planes;

fn signed_number(magnitude: BigUint, negative: bool) -> BigInt {
    power::signed_from_numeric(if negative { Sign::Minus } else { Sign::Plus }, magnitude)
}
fn signed_div(a: BigInt, b: BigInt) -> BigInt {
    let x = get(a.magnitude());
    let y = get(b.magnitude());
    assert!(
        y != N::ZERO && (x == N::ZERO || y == N::from_u128(1)),
        "special signed arithmetic domain exceeded"
    );
    signed_number(
        clone_number(a.magnitude()),
        (a.sign() == Sign::Minus) != (b.sign() == Sign::Minus),
    )
}
fn signed_rem(a: BigInt, b: BigInt) -> BigInt {
    let x = get(a.magnitude());
    let y = get(b.magnitude());
    assert!(
        y != N::ZERO && (x == N::ZERO || y == N::from_u128(1)),
        "special signed arithmetic domain exceeded"
    );
    BigInt::ZERO
}
fn unsigned_div<'a: 'a, 'b: 'b>(a: &'a BigUint, b: &'b BigUint) -> BigUint {
    let x = get(a);
    let y = get(b);
    assert!(
        y != N::ZERO && (x == N::ZERO || y == N::from_u128(1)),
        "special unsigned arithmetic domain exceeded"
    );
    clone_number(a)
}
fn unsigned_rem<'a: 'a, 'b: 'b>(a: &'a BigUint, b: &'b BigUint) -> BigUint {
    let x = get(a);
    let y = get(b);
    assert!(
        y != N::ZERO && (y == N::from_u128(1) || !x.unsigned_cmp(y).is_gt()),
        "special unsigned remainder domain exceeded"
    );
    if y == N::from_u128(1) || x == y {
        BigUint::ZERO
    } else {
        clone_number(a)
    }
}
pub(super) fn subtract<'a: 'a, 'b: 'b>(a: &'a BigUint, b: &'b BigUint) -> BigUint {
    let x = get(a);
    let y = get(b);
    let (lo, borrow) = x.lo.overflowing_sub(y.lo);
    let (hi, first) = x.hi.overflowing_sub(y.hi);
    let (hi, second) = hi.overflowing_sub(u128::from(borrow));
    assert!(!first && !second, "special subtraction domain exceeded");
    put(N { lo, hi })
}

// Native exponents construct a numeric value; BigUint exponents clone one.
// Allocate for both API paths to keep all later abstract handle indices stable.
pub(super) fn allocating_clone(value: &BigUint) -> BigUint {
    put(get(value))
}

pub(super) fn from32(n: u32) -> BigUint {
    from64(u64::from(n))
}
fn modpow(base: &BigUint, exponent: &BigUint, modulus: &BigUint) -> BigUint {
    let b = get(base);
    let e = get(exponent);
    let m = get(modulus);
    assert!(
        m != N::ZERO
            && b.unsigned_cmp(m).is_lt()
            && e != N::ZERO
            && (e == N::from_u128(1) || b == N::ZERO || b == N::from_u128(1)),
        "special power API domain exceeded"
    );
    clone_number(base)
}
fn reject_native(
    _: &veryl_analyzer::value::ValueU64,
    _: &veryl_analyzer::value::ValueU64,
    _: usize,
    _: bool,
    _: bool,
) -> veryl_analyzer::value::ValueU64 {
    panic!("special native division boundary reached")
}

static mut SOURCES: [*const Value; 2] = [std::ptr::null(); 2];
static mut WIDTH: usize = 0;
static mut SIGN: bool = false;
static mut CALLS: usize = 0;
pub(super) fn observed_expand(
    value: &Value,
    width: usize,
    signed: bool,
) -> std::borrow::Cow<'_, Value> {
    unsafe {
        let i = CALLS;
        assert!(
            i < 2 && std::ptr::eq(value, SOURCES[i]) && width == WIDTH && signed == SIGN,
            "wide operator semantics mismatch"
        );
        CALLS += 1;
    }
    let equal = value.width() == width;
    let expanded = prepared_wide(value, width, signed);
    // Equal-width expansion borrows; widening allocates two numeric handles.
    // Pad only the abstract store so later handle indices are input-independent.
    // The Cow, source identity and complete value remain unchanged.
    if equal {
        let _ = from128(0);
        let _ = from128(0);
    }
    expanded
}

pub(super) fn observe_binary_sources(x: &Value, y: &Value, width: usize, signed: bool) {
    unsafe {
        SOURCES = [x, y];
        WIDTH = width;
        SIGN = signed;
        CALLS = 0;
    }
}
pub(super) fn observed_calls() -> usize {
    unsafe { CALLS }
}

// K0/1: signed div/rem by +/-1; K2/3: zero div/rem; K4: x**1;
// K5: unit-base positive power. Membership is checked AFTER independent sizing.
fn check<const K: u8, const ORIGINAL: bool>() {
    init();
    let w = usize::from(kani::any::<u8>() & 63) + 65;
    let aw = if ORIGINAL {
        usize::from(kani::any::<u8>() & 127) + 1
    } else {
        w
    };
    let bw = if ORIGINAL || K >= 4 {
        usize::from(kani::any::<u8>() & 127) + 1
    } else {
        w
    };
    kani::assume(aw <= w && (K >= 4 || bw <= w));
    let minus: bool = kani::any();
    let unit = u128::from(kani::any::<bool>());
    let (x, a) = identities::known_input(
        aw,
        if K == 2 || K == 3 {
            Some(0)
        } else if K == 5 {
            Some(unit)
        } else {
            None
        },
    );
    let (y, b) = identities::known_input(
        bw,
        if K < 2 {
            Some(if minus { u128::MAX } else { 1 })
        } else if K == 4 {
            Some(1)
        } else {
            None
        },
    );
    let sign = if K < 2 { true } else { kani::any() };
    let ap = expanded_planes(a.planes, aw, w, a.signed, sign);
    let bp = if K >= 4 {
        b.planes
    } else {
        expanded_planes(b.planes, bw, w, b.signed, sign)
    };
    let mask = u128::MAX >> (128 - w);
    if K < 2 {
        kani::assume(bp.0 == if minus { mask } else { 1 });
    }
    if K == 2 || K == 3 {
        kani::assume(bp.0 != 0);
    }
    if K >= 4 {
        kani::assume(bp.0 != 0 && !(b.signed && bp.0 >> (bw - 1) & 1 != 0));
    }
    if K == 5 {
        kani::assume(ap.0 == unit);
    }
    unsafe {
        SOURCES = [&x, &y];
        WIDTH = w;
        SIGN = sign;
        CALLS = 0;
    }
    let op = if K == 0 || K == 2 {
        Op::Div
    } else if K == 1 || K == 3 {
        Op::Rem
    } else {
        Op::Pow
    };
    let output = op.eval_value_binary(&x, &y, w, sign, &mut MaskCache::default());
    if ORIGINAL {
        assert!(
            unsafe { CALLS } == if K >= 4 { 1 } else { 2 },
            "wide operator semantics mismatch"
        );
    }
    let result_sign = if K >= 4 {
        a.signed && (w == aw || sign)
    } else {
        sign
    };
    let expected = if K < 4 {
        let (q, r) = veryl_formal_semantics::nonlinear::division_identity(ap.0, bp.0, w, sign);
        if K == 0 || K == 2 { q } else { r }
    } else {
        veryl_formal_semantics::nonlinear::positive_power_identity(ap.0, bp.0, w, bw, b.signed)
    };
    assert!(
        output.mask_xz_u128() == 0,
        "wide operator semantics mismatch"
    );
    let position = usize::from(kani::any::<u8>() & 127);
    result(
        &output,
        w,
        result_sign,
        position,
        read_logic(expected, 0, position),
    );
    preserve(&x, a);
    preserve(&y, b);
    kani::cover!(
        w == 65 && (K >= 2 || (!minus && ap.0 == 1)),
        "special minimum width is reachable"
    );
    kani::cover!(w == 128, "special maximum width is reachable");
    kani::cover!(
        a.signed && !b.signed,
        "special mixed source signs is reachable"
    );
    if K < 2 {
        kani::cover!(ap.0 == 0, "special zero dividend is reachable");
        kani::cover!(
            ap.0 == 1u128 << (w - 1) && minus,
            "special minimum signed dividend is reachable"
        );
    } else if K < 4 {
        kani::cover!(sign, "special signed zero division is reachable");
        kani::cover!(!sign, "special unsigned zero division is reachable");
        kani::cover!(bp.0 == mask, "special all ones divisor is reachable");
        kani::cover!(
            bp.0 == 1u128 << (w - 1),
            "special upper divisor bit is reachable"
        );
    } else {
        kani::cover!(bw > w, "special exponent wider than result is reachable");
        kani::cover!(
            bw == 1 && !b.signed,
            "special one bit positive exponent is reachable"
        );
        kani::cover!(
            bw == 128 && b.signed,
            "special wide signed positive exponent is reachable"
        );
        if K == 4 {
            kani::cover!(
                result_sign && ap.0 == 1u128 << (w - 1) && minus,
                "special power minimum signed base is reachable"
            );
            kani::cover!(
                !result_sign && ap.0 >> 64 != 0,
                "special power upper unsigned base is reachable"
            );
        } else {
            kani::cover!(unit == 0, "special power zero base is reachable");
            kani::cover!(unit == 1, "special power one base is reachable");
            kani::cover!(
                bp.0 >> 127 != 0,
                "special power upper exponent bit is reachable"
            );
            kani::cover!(
                bp.0 > 1 && bp.0 & 1 == 0,
                "special power positive even exponent is reachable"
            );
            kani::cover!(
                bp.0 > 1 && bp.0 & 1 != 0,
                "special power positive odd exponent is reachable"
            );
        }
    }
    if ORIGINAL {
        kani::cover!(
            aw == 64 && bw == 1 && w == 128,
            "special native native sources is reachable"
        );
        kani::cover!(
            aw == 64 && bw == 65 && w == 128,
            "special native big sources is reachable"
        );
        kani::cover!(
            aw == 65 && bw == 64 && w == 128,
            "special big native sources is reachable"
        );
        kani::cover!(
            aw == 65 && bw == 127 && w == 128,
            "special big big sources is reachable"
        );
        if K < 2 {
            kani::cover!(
                b.width == 1 && b.signed && minus,
                "special signed one bit minus one is reachable"
            );
        }
        if K == 4 {
            kani::cover!(
                a.width == 1 && a.signed && sign && ap.0 == mask,
                "special signed one bit base is reachable"
            );
        }
        if K == 5 {
            kani::cover!(
                a.width == 1 && a.signed && !sign && unit == 1,
                "special unit base zero extension is reachable"
            );
        }
    }
}

pub(super) fn rem_owned<'a: 'a>(a: BigUint, b: &'a BigUint) -> BigUint {
    unsigned_rem(&a, b)
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
#[kani::stub(<num_bigint::BigInt as std::ops::Div<num_bigint::BigInt>>::div, signed_div)]
#[kani::stub(<num_bigint::BigInt as std::ops::Rem<num_bigint::BigInt>>::rem, signed_rem)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, unsigned_div)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, unsigned_rem)]
#[kani::stub(<num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, rem_owned)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Sub<&num_bigint::BigUint>>::sub, subtract)]
#[kani::stub(<num_bigint::BigUint as std::convert::From<u32>>::from, from32)]
#[kani::stub(num_bigint::BigUint::modpow, modpow)]
#[kani::stub(veryl_analyzer::value::ValueU64::divide_remainder, reject_native)]
fn special_signed_numeric_apis_match_native_arithmetic() {
    init();
    let zero: bool = kani::any();
    let x = if zero { 0 } else { kani::any::<i128>() };
    let negative: bool = kani::any();
    let divisor = if zero { kani::any::<u128>() } else { 1 };
    kani::assume(divisor != 0);
    let a = signed_number(from128(x.unsigned_abs()), x < 0);
    let b = signed_number(from128(divisor), negative);
    let q = a.clone() / b.clone();
    let r = a % b;
    let expected = if zero {
        0
    } else if negative {
        x.wrapping_div(-1)
    } else {
        x / 1
    };
    assert!(
        get(q.magnitude()) == N::from_u128(x.unsigned_abs())
            && (q.sign() == Sign::Minus) == (x != 0 && ((x < 0) != negative))
            && (if q.sign() == Sign::Minus {
                get(q.magnitude()).lo.wrapping_neg()
            } else {
                get(q.magnitude()).lo
            }) == expected as u128
            && get(r.magnitude()) == N::ZERO
            && r.sign() == Sign::NoSign,
        "special numeric API mismatch"
    );
    kani::cover!(
        zero && divisor >> 127 != 0,
        "special API zero over upper divisor is reachable"
    );
    kani::cover!(
        !zero && x == i128::MIN && negative,
        "special API minimum over minus one is reachable"
    );
    kani::cover!(
        !zero && x < 0 && !negative,
        "special API negative quotient is reachable"
    );
    kani::cover!(
        !zero && x > 0 && negative,
        "special API negative divisor is reachable"
    );
}

#[kani::proof]
#[kani::stub(crate::public_big_ops::clone_number, allocating_clone)]
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
#[kani::stub(<num_bigint::BigInt as std::ops::Div<num_bigint::BigInt>>::div, signed_div)]
#[kani::stub(<num_bigint::BigInt as std::ops::Rem<num_bigint::BigInt>>::rem, signed_rem)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, unsigned_div)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, unsigned_rem)]
#[kani::stub(<num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, rem_owned)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Sub<&num_bigint::BigUint>>::sub, subtract)]
#[kani::stub(<num_bigint::BigUint as std::convert::From<u32>>::from, from32)]
#[kani::stub(num_bigint::BigUint::modpow, modpow)]
#[kani::stub(veryl_analyzer::value::ValueU64::divide_remainder, reject_native)]
fn special_unsigned_and_power_apis_match_scalar_identities() {
    init();
    let n = kani::any::<u128>();
    let d = kani::any::<u128>();
    kani::assume(d != 0);
    let zero = from64(0);
    let divisor = from128(d);
    let q = &zero / &divisor;
    let r = &zero % &divisor;
    assert!(
        get(&q) == N::ZERO && get(&r) == N::ZERO,
        "special numeric API mismatch"
    );
    let w = usize::from(kani::any::<u8>() & 63) + 65;
    let bound = N::from_u128(1).checked_shl(w);
    let magnitude = n & (u128::MAX >> (128 - w));
    let unit: bool = kani::any();
    let base = if unit {
        u128::from(kani::any::<bool>())
    } else {
        magnitude
    };
    let exp = if unit { kani::any::<u128>() } else { 1 };
    kani::assume(exp != 0);
    let b = from128(base);
    let e = from128(exp);
    let m = put(bound);
    let cloned = allocating_clone(&b);
    assert!(
        get(&cloned) == N::from_u128(base),
        "special numeric API mismatch"
    );
    let out = b.modpow(&e, &m);
    assert!(
        get(&out)
            == N::from_u128(if exp == 1 {
                base
            } else if base == 0 {
                0
            } else {
                1
            })
            && get(&b) == N::from_u128(base)
            && get(&e) == N::from_u128(exp)
            && get(&m) == bound,
        "special numeric API mismatch"
    );
    let delta = &m - &b;
    let reduced = delta % &m;
    let expected = if base == 0 {
        0
    } else {
        base.wrapping_neg() & (u128::MAX >> (128 - w))
    };
    assert!(
        get(&reduced) == N::from_u128(expected),
        "special numeric API mismatch"
    );
    kani::cover!(w == 128, "special API bit128 modulus is reachable");
    kani::cover!(
        unit && base == 0 && exp >> 127 != 0,
        "special API zero power upper exponent is reachable"
    );
    kani::cover!(
        unit && base == 1 && exp > 1,
        "special API one power is reachable"
    );
    kani::cover!(
        !unit && base >> 127 != 0,
        "special API upper base is reachable"
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
#[kani::stub(<num_bigint::BigInt as std::ops::Div<num_bigint::BigInt>>::div, signed_div)]
#[kani::stub(<num_bigint::BigInt as std::ops::Rem<num_bigint::BigInt>>::rem, signed_rem)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, unsigned_div)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, unsigned_rem)]
#[kani::stub(<num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, rem_owned)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Sub<&num_bigint::BigUint>>::sub, subtract)]
#[kani::stub(<num_bigint::BigUint as std::convert::From<u32>>::from, from32)]
#[kani::stub(num_bigint::BigUint::modpow, modpow)]
#[kani::stub(veryl_analyzer::value::ValueU64::divide_remainder, reject_native)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::prepared_wide)]
fn wide_special_signed_div_unit_prepared_matches_reference() {
    check::<0, false>();
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
#[kani::stub(<num_bigint::BigInt as std::ops::Div<num_bigint::BigInt>>::div, signed_div)]
#[kani::stub(<num_bigint::BigInt as std::ops::Rem<num_bigint::BigInt>>::rem, signed_rem)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, unsigned_div)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, unsigned_rem)]
#[kani::stub(<num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, rem_owned)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Sub<&num_bigint::BigUint>>::sub, subtract)]
#[kani::stub(<num_bigint::BigUint as std::convert::From<u32>>::from, from32)]
#[kani::stub(num_bigint::BigUint::modpow, modpow)]
#[kani::stub(veryl_analyzer::value::ValueU64::divide_remainder, reject_native)]
#[kani::stub(veryl_analyzer::value::Value::expand, observed_expand)]
fn wide_special_signed_div_unit_context_matches_reference() {
    check::<0, true>();
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
#[kani::stub(<num_bigint::BigInt as std::ops::Div<num_bigint::BigInt>>::div, signed_div)]
#[kani::stub(<num_bigint::BigInt as std::ops::Rem<num_bigint::BigInt>>::rem, signed_rem)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, unsigned_div)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, unsigned_rem)]
#[kani::stub(<num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, rem_owned)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Sub<&num_bigint::BigUint>>::sub, subtract)]
#[kani::stub(<num_bigint::BigUint as std::convert::From<u32>>::from, from32)]
#[kani::stub(num_bigint::BigUint::modpow, modpow)]
#[kani::stub(veryl_analyzer::value::ValueU64::divide_remainder, reject_native)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::prepared_wide)]
fn wide_special_signed_rem_unit_prepared_matches_reference() {
    check::<1, false>();
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
#[kani::stub(<num_bigint::BigInt as std::ops::Div<num_bigint::BigInt>>::div, signed_div)]
#[kani::stub(<num_bigint::BigInt as std::ops::Rem<num_bigint::BigInt>>::rem, signed_rem)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, unsigned_div)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, unsigned_rem)]
#[kani::stub(<num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, rem_owned)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Sub<&num_bigint::BigUint>>::sub, subtract)]
#[kani::stub(<num_bigint::BigUint as std::convert::From<u32>>::from, from32)]
#[kani::stub(num_bigint::BigUint::modpow, modpow)]
#[kani::stub(veryl_analyzer::value::ValueU64::divide_remainder, reject_native)]
#[kani::stub(veryl_analyzer::value::Value::expand, observed_expand)]
fn wide_special_signed_rem_unit_context_matches_reference() {
    check::<1, true>();
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
#[kani::stub(<num_bigint::BigInt as std::ops::Div<num_bigint::BigInt>>::div, signed_div)]
#[kani::stub(<num_bigint::BigInt as std::ops::Rem<num_bigint::BigInt>>::rem, signed_rem)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, unsigned_div)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, unsigned_rem)]
#[kani::stub(<num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, rem_owned)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Sub<&num_bigint::BigUint>>::sub, subtract)]
#[kani::stub(<num_bigint::BigUint as std::convert::From<u32>>::from, from32)]
#[kani::stub(num_bigint::BigUint::modpow, modpow)]
#[kani::stub(veryl_analyzer::value::ValueU64::divide_remainder, reject_native)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::prepared_wide)]
fn wide_special_zero_div_prepared_matches_reference() {
    check::<2, false>();
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
#[kani::stub(<num_bigint::BigInt as std::ops::Div<num_bigint::BigInt>>::div, signed_div)]
#[kani::stub(<num_bigint::BigInt as std::ops::Rem<num_bigint::BigInt>>::rem, signed_rem)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, unsigned_div)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, unsigned_rem)]
#[kani::stub(<num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, rem_owned)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Sub<&num_bigint::BigUint>>::sub, subtract)]
#[kani::stub(<num_bigint::BigUint as std::convert::From<u32>>::from, from32)]
#[kani::stub(num_bigint::BigUint::modpow, modpow)]
#[kani::stub(veryl_analyzer::value::ValueU64::divide_remainder, reject_native)]
#[kani::stub(veryl_analyzer::value::Value::expand, observed_expand)]
fn wide_special_zero_div_context_matches_reference() {
    check::<2, true>();
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
#[kani::stub(<num_bigint::BigInt as std::ops::Div<num_bigint::BigInt>>::div, signed_div)]
#[kani::stub(<num_bigint::BigInt as std::ops::Rem<num_bigint::BigInt>>::rem, signed_rem)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, unsigned_div)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, unsigned_rem)]
#[kani::stub(<num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, rem_owned)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Sub<&num_bigint::BigUint>>::sub, subtract)]
#[kani::stub(<num_bigint::BigUint as std::convert::From<u32>>::from, from32)]
#[kani::stub(num_bigint::BigUint::modpow, modpow)]
#[kani::stub(veryl_analyzer::value::ValueU64::divide_remainder, reject_native)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::prepared_wide)]
fn wide_special_zero_rem_prepared_matches_reference() {
    check::<3, false>();
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
#[kani::stub(<num_bigint::BigInt as std::ops::Div<num_bigint::BigInt>>::div, signed_div)]
#[kani::stub(<num_bigint::BigInt as std::ops::Rem<num_bigint::BigInt>>::rem, signed_rem)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, unsigned_div)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, unsigned_rem)]
#[kani::stub(<num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, rem_owned)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Sub<&num_bigint::BigUint>>::sub, subtract)]
#[kani::stub(<num_bigint::BigUint as std::convert::From<u32>>::from, from32)]
#[kani::stub(num_bigint::BigUint::modpow, modpow)]
#[kani::stub(veryl_analyzer::value::ValueU64::divide_remainder, reject_native)]
#[kani::stub(veryl_analyzer::value::Value::expand, observed_expand)]
fn wide_special_zero_rem_context_matches_reference() {
    check::<3, true>();
}

#[kani::proof]
#[kani::stub(crate::public_big_ops::clone_number, allocating_clone)]
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
#[kani::stub(<num_bigint::BigInt as std::ops::Div<num_bigint::BigInt>>::div, signed_div)]
#[kani::stub(<num_bigint::BigInt as std::ops::Rem<num_bigint::BigInt>>::rem, signed_rem)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, unsigned_div)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, unsigned_rem)]
#[kani::stub(<num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, rem_owned)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Sub<&num_bigint::BigUint>>::sub, subtract)]
#[kani::stub(<num_bigint::BigUint as std::convert::From<u32>>::from, from32)]
#[kani::stub(num_bigint::BigUint::modpow, modpow)]
#[kani::stub(veryl_analyzer::value::ValueU64::divide_remainder, reject_native)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::prepared_wide)]
fn wide_special_power_one_prepared_matches_reference() {
    check::<4, false>();
}

#[kani::proof]
#[kani::stub(crate::public_big_ops::clone_number, allocating_clone)]
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
#[kani::stub(<num_bigint::BigInt as std::ops::Div<num_bigint::BigInt>>::div, signed_div)]
#[kani::stub(<num_bigint::BigInt as std::ops::Rem<num_bigint::BigInt>>::rem, signed_rem)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, unsigned_div)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, unsigned_rem)]
#[kani::stub(<num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, rem_owned)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Sub<&num_bigint::BigUint>>::sub, subtract)]
#[kani::stub(<num_bigint::BigUint as std::convert::From<u32>>::from, from32)]
#[kani::stub(num_bigint::BigUint::modpow, modpow)]
#[kani::stub(veryl_analyzer::value::ValueU64::divide_remainder, reject_native)]
#[kani::stub(veryl_analyzer::value::Value::expand, observed_expand)]
fn wide_special_power_one_context_matches_reference() {
    check::<4, true>();
}

#[kani::proof]
#[kani::stub(crate::public_big_ops::clone_number, allocating_clone)]
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
#[kani::stub(<num_bigint::BigInt as std::ops::Div<num_bigint::BigInt>>::div, signed_div)]
#[kani::stub(<num_bigint::BigInt as std::ops::Rem<num_bigint::BigInt>>::rem, signed_rem)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, unsigned_div)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, unsigned_rem)]
#[kani::stub(<num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, rem_owned)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Sub<&num_bigint::BigUint>>::sub, subtract)]
#[kani::stub(<num_bigint::BigUint as std::convert::From<u32>>::from, from32)]
#[kani::stub(num_bigint::BigUint::modpow, modpow)]
#[kani::stub(veryl_analyzer::value::ValueU64::divide_remainder, reject_native)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::prepared_wide)]
fn wide_special_power_unit_base_prepared_matches_reference() {
    check::<5, false>();
}

#[kani::proof]
#[kani::stub(crate::public_big_ops::clone_number, allocating_clone)]
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
#[kani::stub(<num_bigint::BigInt as std::ops::Div<num_bigint::BigInt>>::div, signed_div)]
#[kani::stub(<num_bigint::BigInt as std::ops::Rem<num_bigint::BigInt>>::rem, signed_rem)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, unsigned_div)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, unsigned_rem)]
#[kani::stub(<num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, rem_owned)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Sub<&num_bigint::BigUint>>::sub, subtract)]
#[kani::stub(<num_bigint::BigUint as std::convert::From<u32>>::from, from32)]
#[kani::stub(num_bigint::BigUint::modpow, modpow)]
#[kani::stub(veryl_analyzer::value::ValueU64::divide_remainder, reject_native)]
#[kani::stub(veryl_analyzer::value::Value::expand, observed_expand)]
fn wide_special_power_unit_base_context_matches_reference() {
    check::<5, true>();
}
