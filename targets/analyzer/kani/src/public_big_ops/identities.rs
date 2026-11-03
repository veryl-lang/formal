//! IEEE1800-2023 11.4.3/11.8.2: known-value unit arithmetic.
//! Prepared widths65..128; original widths1..128 with nonnarrowing65..128
//! contexts and all four storage pairs. Masks are zero; source signs remain
//! independent. Multiplication allows either caller sign and a 0/1 factor on
//! either side. Division/remainder require unsigned mode and prepared RHS1.
//! The class precondition is on PREPARED values: signed one-bit1 sign-extends
//! to -1, so it is not a unit-one factor in a wider signed context.
//! Actual Op bodies run under guarded mathematical BigUint unit APIs and the
//! separately proved value-preserving expansion summary. API adapter checks
//! compare with native-half arithmetic; they do not prove num-bigint internals.
//! Scalar shift/add and restoring division provide independent compiler oracles.
//! The restoring-division identity is proved separately before its summary is
//! composed with the actual division/remainder Op bodies.
//! General factors/divisors, signed division, X/Z, allocation, arbitrary cache
//! histories and expression effects are outside these identity contracts.
use super::*;
use num_bigint::{BigInt, BigUint};
use veryl_formal_semantics::{nonlinear, resize::expanded_planes};

// Unit arithmetic can return a clone or canonical zero. Reusing immutable
// numeric handles avoids creating equivalent store entries without changing
// mathematical values or the supported API domain.
fn mul_units<'a: 'a, 'b: 'b>(a: &'a BigUint, b: &'b BigUint) -> BigUint {
    let x = get(a);
    let y = get(b);
    let one = N::from_u128(1);
    assert!(
        x == N::ZERO || x == one || y == N::ZERO || y == one,
        "unit numeric multiplication domain exceeded"
    );
    if x == N::ZERO || y == N::ZERO {
        BigUint::ZERO
    } else if x == one {
        clone_number(b)
    } else {
        clone_number(a)
    }
}
fn div_unit<'a: 'a, 'b: 'b>(a: &'a BigUint, b: &'b BigUint) -> BigUint {
    assert!(
        get(b) == N::from_u128(1),
        "unit numeric divisor domain exceeded"
    );
    clone_number(a)
}
fn rem_unit<'a: 'a, 'b: 'b>(a: &'a BigUint, b: &'b BigUint) -> BigUint {
    let _ = get(a);
    assert!(
        get(b) == N::from_u128(1),
        "unit numeric divisor domain exceeded"
    );
    BigUint::ZERO
}
fn reject_native_division(
    _: &veryl_analyzer::value::ValueU64,
    _: &veryl_analyzer::value::ValueU64,
    _: usize,
    _: bool,
    _: bool,
) -> veryl_analyzer::value::ValueU64 {
    panic!("unit native division boundary reached")
}
fn reject_conversion(_: &ValueBigUint) -> Option<BigInt> {
    panic!("unit signed division boundary reached")
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Mul<&num_bigint::BigUint>>::mul, mul_units)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, div_unit)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, rem_unit)]
fn numeric_unit_apis_preserve_native_half_arithmetic() {
    init();
    let n = N {
        lo: kani::any(),
        hi: kani::any(),
    };
    let factor = u128::from(kani::any::<bool>());
    let (lo, carry) = n.lo.overflowing_mul(factor);
    let (hi, overflow) = n.hi.overflowing_mul(factor);
    let expected = N { lo, hi };
    let x = put(n);
    let u = from128(factor);
    let one = from64(1);
    let right = &x * &u;
    let left = &u * &x;
    let q = &x / &one;
    let r = &x % &one;
    assert!(
        !carry
            && !overflow
            && get(&right) == expected
            && get(&left) == expected
            && get(&q)
                == N {
                    lo: n.lo / 1,
                    hi: n.hi / 1
                }
            && get(&r)
                == N {
                    lo: n.lo % 1,
                    hi: n.hi % 1
                }
            && get(&x) == n
            && get(&u) == N::from_u128(factor)
            && get(&one) == N::from_u128(1),
        "unit numeric API mismatch"
    );
    kani::cover!(factor == 0, "unit API zero factor is reachable");
    kani::cover!(factor == 1, "unit API one factor is reachable");
    kani::cover!(
        n.hi >> 127 != 0 && factor == 1,
        "unit API bit255 is reachable"
    );
    kani::cover!(n == N::ZERO, "unit API zero number is reachable");
    kani::cover!(
        n.lo == u128::MAX && n.hi == u128::MAX && factor == 1,
        "unit API full magnitude is reachable"
    );
}

// Pure scalar reference theorem: no production operation or numeric stub is
// involved. This discharges the restoring-reference summary used by Op proofs.
#[kani::proof]
#[kani::unwind(129)]
fn restoring_division_by_one_preserves_number() {
    let width = usize::from(kani::any::<u8>() & 127) + 1;
    let number = kani::any::<u128>() & (u128::MAX >> (128 - width));
    let (q, r) = nonlinear::quotient_remainder(number, 1, width, false);
    assert!(q == number && r == 0, "unit division reference mismatch");
    kani::cover!(
        width == 1,
        "unit division reference minimum width is reachable"
    );
    kani::cover!(
        width == 128,
        "unit division reference maximum width is reachable"
    );
    kani::cover!(
        width == 128 && number >> 127 != 0,
        "unit division reference upper bit is reachable"
    );
    kani::cover!(number == 0, "unit division reference zero is reachable");
    kani::cover!(
        number == u128::MAX,
        "unit division reference maximum number is reachable"
    );
}

pub(super) fn known_input(width: usize, literal: Option<u128>) -> (Value, Operand) {
    let p = literal.unwrap_or_else(kani::any::<u128>) & (u128::MAX >> (128 - width));
    let a = Operand {
        planes: (p, 0),
        width,
        signed: kani::any(),
    };
    // Reserve one payload slot on both storage paths, keeping later indices stable.
    let payload = from128(p);
    let value = if width <= 64 {
        Value::from_u128(p, 0, width, a.signed)
    } else {
        Value::BigUint(ValueBigUint {
            payload: Box::new(payload),
            mask_xz: Box::new(BigUint::ZERO),
            width: width as u32,
            signed: a.signed,
        })
    };
    preserve(&value, a);
    (value, a)
}
static mut SOURCES: [*const Value; 2] = [std::ptr::null(); 2];
static mut WIDTH: usize = 0;
static mut SIGN: bool = false;
static mut CALLS: usize = 0;
fn observed_expand(value: &Value, width: usize, signed: bool) -> std::borrow::Cow<'_, Value> {
    unsafe {
        let call = CALLS;
        assert!(
            call < 2 && std::ptr::eq(value, SOURCES[call]) && width == WIDTH && signed == SIGN,
            "wide operator semantics mismatch"
        );
        CALLS += 1;
    }
    prepared_wide(value, width, signed)
}

// K0: x*unit; K1: unit*x; K2: unsigned x/1; K3: unsigned x%1.
fn check<const K: u8, const ORIGINAL: bool>() -> (Operand, Operand, usize, bool, u128, u128) {
    init();
    let width = usize::from(kani::any::<u8>() & 63) + 65;
    let left = if ORIGINAL {
        usize::from(kani::any::<u8>() & 127) + 1
    } else {
        width
    };
    let right = if ORIGINAL {
        usize::from(kani::any::<u8>() & 127) + 1
    } else {
        width
    };
    kani::assume(left <= width && right <= width);
    let unit = if K < 2 {
        u128::from(kani::any::<bool>())
    } else {
        1
    };
    let (x, a) = known_input(left, if K == 1 { Some(unit) } else { None });
    let (y, b) = known_input(right, if K == 1 { None } else { Some(unit) });
    let sign: bool = if K < 2 { kani::any() } else { false };
    let ap = expanded_planes(a.planes, left, width, a.signed, sign);
    let bp = expanded_planes(b.planes, right, width, b.signed, sign);
    let (number, factor) = if K == 1 { (bp.0, ap.0) } else { (ap.0, bp.0) };
    assert!(
        ap.1 == 0 && bp.1 == 0,
        "unit preparation reference mismatch"
    );
    kani::assume(factor == unit);
    unsafe {
        SOURCES = [&x, &y];
        WIDTH = width;
        SIGN = sign;
        CALLS = 0;
    }
    let op = if K < 2 {
        Op::Mul
    } else if K == 2 {
        Op::Div
    } else {
        Op::Rem
    };
    let output = op.eval_value_binary(&x, &y, width, sign, &mut MaskCache::default());
    if ORIGINAL {
        assert!(unsafe { CALLS } == 2, "wide operator semantics mismatch");
    }
    // Integer multiplication is commutative. Put the bounded factor second in
    // the independent shift/add reference even for the unit*x production arm.
    let expected = if K < 2 {
        nonlinear::product(number, unit, width)
    } else {
        // Discharged for every normalized width1..128 by the scalar theorem.
        if K == 2 { number } else { 0 }
    };
    if K < 2 {
        assert!(
            (output.payload_u128(), output.mask_xz_u128()) == (expected, 0),
            "wide operator semantics mismatch"
        );
    } else {
        assert!(
            output.mask_xz_u128() == 0,
            "wide operator semantics mismatch"
        );
    }
    // The position is independent of all inputs and spans every payload bit.
    // Together with result's upper-plane bounds, this proves full value equality
    // for division/remainder without duplicating the whole-vector assertion.
    let p = usize::from(kani::any::<u8>() & 127);
    result(&output, width, sign, p, read_logic(expected, 0, p));
    preserve(&x, a);
    preserve(&y, b);
    (a, b, width, sign, number, factor)
}
fn cover_common(a: Operand, b: Operand, width: usize, sign: bool, number: u128) {
    let mask = u128::MAX >> (128 - width);
    kani::cover!(width == 65, "unit identity minimum width is reachable");
    kani::cover!(width == 128, "unit identity maximum width is reachable");
    kani::cover!(number == 0, "unit identity zero number is reachable");
    kani::cover!(number == mask, "unit identity all ones number is reachable");
    kani::cover!(
        width == 128 && number >> 127 != 0,
        "unit identity upper bit is reachable"
    );
    kani::cover!(
        a.signed && !b.signed,
        "unit identity mixed source signs is reachable"
    );
    kani::cover!(
        !sign && a.signed && b.signed,
        "unit identity unsigned mode signed sources is reachable"
    );
    kani::cover!(
        width == 128 && number == 0xaaaa_aaaa_aaaa_aaaa_aaaa_aaaa_aaaa_aaaa,
        "unit identity alternating number is reachable"
    );
}
fn cover_multiply(factor: u128, sign: bool) {
    kani::cover!(factor == 0, "unit identity zero factor is reachable");
    kani::cover!(factor == 1, "unit identity one factor is reachable");
    kani::cover!(sign, "unit identity signed multiplication is reachable");
}
fn cover_context(a: Operand, b: Operand, width: usize, sign: bool, unit: Operand) {
    kani::cover!(
        a.width == 64 && b.width == 1 && width == 128,
        "unit identity native native sources is reachable"
    );
    kani::cover!(
        a.width == 64 && b.width == 65 && width == 128,
        "unit identity native big sources is reachable"
    );
    kani::cover!(
        a.width == 65 && b.width == 64 && width == 128,
        "unit identity big native sources is reachable"
    );
    kani::cover!(
        a.width == 65 && b.width == 127 && width == 128,
        "unit identity big big sources is reachable"
    );
    kani::cover!(
        unit.width == 1 && unit.signed && !sign && unit.planes.0 == 1,
        "unit identity signed one bit one zero extension is reachable"
    );
}
fn cover_context_multiply(unit: Operand, sign: bool) {
    kani::cover!(
        unit.width == 1 && unit.signed && sign && unit.planes.0 == 0,
        "unit identity signed one bit zero sign extension is reachable"
    );
}

#[kani::proof]
#[kani::unwind(129)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::prepared_wide)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Mul<&num_bigint::BigUint>>::mul, mul_units)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, div_unit)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, rem_unit)]
#[kani::stub(veryl_analyzer::value::ValueBigUint::to_bigint, reject_conversion)]
fn wide_unit_mul_right_prepared_matches_reference() {
    let (a, b, w, s, n, u) = check::<0, false>();
    cover_common(a, b, w, s, n);
    cover_multiply(u, s);
}

#[kani::proof]
#[kani::unwind(129)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, observed_expand)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Mul<&num_bigint::BigUint>>::mul, mul_units)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, div_unit)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, rem_unit)]
#[kani::stub(veryl_analyzer::value::ValueBigUint::to_bigint, reject_conversion)]
fn wide_unit_mul_right_context_matches_reference() {
    let (a, b, w, s, n, u) = check::<0, true>();
    cover_common(a, b, w, s, n);
    cover_multiply(u, s);
    cover_context(a, b, w, s, b);
    cover_context_multiply(b, s);
}

#[kani::proof]
#[kani::unwind(129)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::prepared_wide)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Mul<&num_bigint::BigUint>>::mul, mul_units)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, div_unit)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, rem_unit)]
#[kani::stub(veryl_analyzer::value::ValueBigUint::to_bigint, reject_conversion)]
fn wide_unit_mul_left_prepared_matches_reference() {
    let (a, b, w, s, n, u) = check::<1, false>();
    cover_common(a, b, w, s, n);
    cover_multiply(u, s);
}

#[kani::proof]
#[kani::unwind(129)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, observed_expand)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Mul<&num_bigint::BigUint>>::mul, mul_units)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, div_unit)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, rem_unit)]
#[kani::stub(veryl_analyzer::value::ValueBigUint::to_bigint, reject_conversion)]
fn wide_unit_mul_left_context_matches_reference() {
    let (a, b, w, s, n, u) = check::<1, true>();
    cover_common(a, b, w, s, n);
    cover_multiply(u, s);
    cover_context(a, b, w, s, a);
    cover_context_multiply(a, s);
}

#[kani::proof]
#[kani::unwind(129)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::prepared_wide)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Mul<&num_bigint::BigUint>>::mul, mul_units)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, div_unit)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, rem_unit)]
#[kani::stub(veryl_analyzer::value::ValueBigUint::to_bigint, reject_conversion)]
#[kani::stub(
    veryl_analyzer::value::ValueU64::divide_remainder,
    reject_native_division
)]
fn wide_unit_div_one_prepared_matches_reference() {
    let (a, b, w, s, n, u) = check::<2, false>();
    cover_common(a, b, w, s, n);
}

#[kani::proof]
#[kani::unwind(129)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, observed_expand)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Mul<&num_bigint::BigUint>>::mul, mul_units)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, div_unit)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, rem_unit)]
#[kani::stub(veryl_analyzer::value::ValueBigUint::to_bigint, reject_conversion)]
#[kani::stub(
    veryl_analyzer::value::ValueU64::divide_remainder,
    reject_native_division
)]
fn wide_unit_div_one_context_matches_reference() {
    let (a, b, w, s, n, u) = check::<2, true>();
    cover_common(a, b, w, s, n);
    cover_context(a, b, w, s, b);
}

#[kani::proof]
#[kani::unwind(129)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::prepared_wide)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Mul<&num_bigint::BigUint>>::mul, mul_units)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, div_unit)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, rem_unit)]
#[kani::stub(veryl_analyzer::value::ValueBigUint::to_bigint, reject_conversion)]
#[kani::stub(
    veryl_analyzer::value::ValueU64::divide_remainder,
    reject_native_division
)]
fn wide_unit_rem_one_prepared_matches_reference() {
    let (a, b, w, s, n, u) = check::<3, false>();
    cover_common(a, b, w, s, n);
}

#[kani::proof]
#[kani::unwind(129)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, observed_expand)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Mul<&num_bigint::BigUint>>::mul, mul_units)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, div_unit)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, rem_unit)]
#[kani::stub(veryl_analyzer::value::ValueBigUint::to_bigint, reject_conversion)]
#[kani::stub(
    veryl_analyzer::value::ValueU64::divide_remainder,
    reject_native_division
)]
fn wide_unit_rem_one_context_matches_reference() {
    let (a, b, w, s, n, u) = check::<3, true>();
    cover_common(a, b, w, s, n);
    cover_context(a, b, w, s, b);
}
