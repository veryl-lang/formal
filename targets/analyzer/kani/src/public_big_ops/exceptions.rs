//! IEEE 1800-2023 11.4.3 (X/Z arithmetic and zero divisor), 11.8.2.
//! These are exceptional-path proofs, not arbitrary-input numeric arithmetic.
//! Prepared operands are normalized65..128-bit four-state values. Composition
//! admits independent source widths1..128 and nonnarrowing contexts65..128,
//! all four source storage pairs, source signs and caller-resolved sign modes.
//! At least one X/Z bit, or a known zero divisor for / and %, is required.
//! The result must be canonical all-X with the requested width and caller sign.
//! Actual Op bodies execute under the numeric/mask and independently proved
//! expansion contracts. Numerical multiplication/division/remainder and signed
//! conversion are forbidden boundaries: reaching one fails with a distinct
//! guard message and never counts as successful semantic mutation detection.
//! Allocation, arbitrary cache histories, normal numeric results, power and
//! expression effects are outside scope. Direct comparisons retain these domains.
use super::*;
use num_bigint::{BigInt, BigUint};

fn reject_numeric<'a: 'a, 'b: 'b>(_: &'a BigUint, _: &'b BigUint) -> BigUint {
    panic!("exception numeric arithmetic boundary reached")
}
fn reject_conversion(_: &ValueBigUint) -> Option<BigInt> {
    panic!("exception signed conversion boundary reached")
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

fn check<const K: u8, const ORIGINAL: bool>() -> (Operand, Operand, usize, bool) {
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
    let (x, a) = input(left);
    let (y, b) = input(right);
    let sign: bool = kani::any();
    // Normalized zero/nonzero and X/Z presence are preserved by nonnarrowing
    // extension. The predicate uses original independent planes, not Op helpers.
    kani::assume((a.planes.1 | b.planes.1) != 0 || (K != 0 && b.planes.0 == 0));
    unsafe {
        SOURCES = [&x, &y];
        WIDTH = width;
        SIGN = sign;
        CALLS = 0;
    }
    let op = match K {
        0 => Op::Mul,
        1 => Op::Div,
        _ => Op::Rem,
    };
    let ret = op.eval_value_binary(&x, &y, width, sign, &mut MaskCache::default());
    if ORIGINAL {
        assert!(unsafe { CALLS } == 2, "wide operator semantics mismatch");
    }
    let mask = u128::MAX >> (128 - width);
    assert!(
        (ret.payload_u128(), ret.mask_xz_u128()) == (0, mask),
        "wide operator semantics mismatch"
    );
    let position = usize::from(kani::any::<u8>() & 127);
    result(
        &ret,
        width,
        sign,
        position,
        if position < width {
            Logic4::X
        } else {
            Logic4::Zero
        },
    );
    preserve(&x, a);
    preserve(&y, b);
    (a, b, width, sign)
}

fn common_covers(a: Operand, b: Operand, width: usize, sign: bool) {
    kani::cover!(width == 65, "wide exception minimum width is reachable");
    kani::cover!(width == 128, "wide exception maximum width is reachable");
    kani::cover!(
        a.planes.1 != 0 && b.planes.1 == 0 && b.planes.0 != 0,
        "wide exception only lhs unknown is reachable"
    );
    kani::cover!(
        a.planes.1 == 0 && b.planes.1 != 0,
        "wide exception only rhs unknown is reachable"
    );
    kani::cover!(
        a.planes.1 != 0 && b.planes.1 != 0,
        "wide exception both unknown is reachable"
    );
    kani::cover!(
        a.width == 128 && a.planes.1 >> 64 != 0,
        "wide exception upper unknown is reachable"
    );
    kani::cover!(
        sign && a.signed && !b.signed,
        "wide exception signed mixed sources is reachable"
    );
    kani::cover!(
        !sign && a.signed && b.signed,
        "wide exception unsigned mode signed sources is reachable"
    );
}
fn composition_covers(a: Operand, b: Operand, width: usize) {
    kani::cover!(
        a.width == 64 && b.width == 1 && width == 128,
        "wide exception native native preparation is reachable"
    );
    kani::cover!(
        a.width == 64 && b.width == 65 && width == 128,
        "wide exception native big preparation is reachable"
    );
    kani::cover!(
        a.width == 65 && b.width == 64 && width == 128,
        "wide exception big native preparation is reachable"
    );
    kani::cover!(
        a.width == 65 && b.width == 127 && width == 128,
        "wide exception big big preparation is reachable"
    );
}
fn zero_covers(a: Operand, b: Operand) {
    kani::cover!(
        a.planes.1 == 0 && b.planes == (0, 0) && a.planes.0 != 0,
        "wide exception known zero divisor is reachable"
    );
    kani::cover!(
        a.planes == (0, 0) && b.planes == (0, 0),
        "wide exception zero over zero is reachable"
    );
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::prepared_wide)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Mul<&num_bigint::BigUint>>::mul, reject_numeric)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, reject_numeric)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, reject_numeric)]
#[kani::stub(veryl_analyzer::value::ValueBigUint::to_bigint, reject_conversion)]
fn wide_mul_exception_prepared_matches_all_x() {
    let (a, b, width, sign) = check::<0, false>();
    common_covers(a, b, width, sign);
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, observed_expand)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Mul<&num_bigint::BigUint>>::mul, reject_numeric)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, reject_numeric)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, reject_numeric)]
#[kani::stub(veryl_analyzer::value::ValueBigUint::to_bigint, reject_conversion)]
fn wide_mul_exception_preparation_matches_all_x() {
    let (a, b, width, sign) = check::<0, true>();
    common_covers(a, b, width, sign);
    composition_covers(a, b, width);
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::prepared_wide)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Mul<&num_bigint::BigUint>>::mul, reject_numeric)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, reject_numeric)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, reject_numeric)]
#[kani::stub(veryl_analyzer::value::ValueBigUint::to_bigint, reject_conversion)]
fn wide_div_exception_prepared_matches_all_x() {
    let (a, b, width, sign) = check::<1, false>();
    common_covers(a, b, width, sign);
    zero_covers(a, b);
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, observed_expand)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Mul<&num_bigint::BigUint>>::mul, reject_numeric)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, reject_numeric)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, reject_numeric)]
#[kani::stub(veryl_analyzer::value::ValueBigUint::to_bigint, reject_conversion)]
fn wide_div_exception_preparation_matches_all_x() {
    let (a, b, width, sign) = check::<1, true>();
    common_covers(a, b, width, sign);
    composition_covers(a, b, width);
    zero_covers(a, b);
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::prepared_wide)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Mul<&num_bigint::BigUint>>::mul, reject_numeric)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, reject_numeric)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, reject_numeric)]
#[kani::stub(veryl_analyzer::value::ValueBigUint::to_bigint, reject_conversion)]
fn wide_rem_exception_prepared_matches_all_x() {
    let (a, b, width, sign) = check::<2, false>();
    common_covers(a, b, width, sign);
    zero_covers(a, b);
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, observed_expand)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Mul<&num_bigint::BigUint>>::mul, reject_numeric)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, reject_numeric)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, reject_numeric)]
#[kani::stub(veryl_analyzer::value::ValueBigUint::to_bigint, reject_conversion)]
fn wide_rem_exception_preparation_matches_all_x() {
    let (a, b, width, sign) = check::<2, true>();
    common_covers(a, b, width, sign);
    composition_covers(a, b, width);
    zero_covers(a, b);
}
