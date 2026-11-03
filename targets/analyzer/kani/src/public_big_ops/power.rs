//! IEEE1800-2023 11.4.3/Table11-4,11.6/11.8.2: bounded power special cases.
//! Prepared bases65..128; original bases1..128 with nonnarrowing contexts65..128.
//! Exponent widths1..128/signs are self-determined. Partitions are unknown,
//! known negative, and known zero exponents. Base planes/flags and caller sign
//! are independent. Actual Op and zero-exponent pow_mod_width bodies execute.
//! Signed conversion uses its separately proved scalar-decoding summary.
//! Numeric APIs and expansion use separate mathematical contracts. The added
//! modpow contract is ONLY the empty-product identity for exponent zero and a
//! positive modulus; it is an API assumption, not a num-bigint implementation
//! theorem or a general modular exponentiation proof. Real-library integration
//! and ordinary positive exponents remain direct comparisons.
//! Reference power_exception propagates base/exponent X/Z before the zero
//! identity, consistent with the existing native proofs. For X/Z bases at zero
//! exponent this checks that established interpretation; it does not resolve
//! the precedence of the general X/Z sentence and the explicit zero-power rule.
//! No source-expression side effects, arbitrary widths or allocation theorem.
use super::*;
use num_bigint::{BigInt, BigUint, Sign};
use veryl_formal_semantics::{nonlinear::power_exception, resize::expanded_planes};

fn from32(n: u32) -> BigUint {
    from64(u64::from(n))
}
fn zero_modpow(_: &BigUint, exponent: &BigUint, modulus: &BigUint) -> BigUint {
    let e = get(exponent);
    let m = get(modulus);
    assert!(
        e == N::ZERO && m != N::ZERO,
        "power modular API domain exceeded"
    );
    from64(u64::from(m != N::from_u128(1)))
}
// Exact specialization of the separately checked numeric bit API. Returning a
// literal false after the guard prunes the impossible odd-zero-exponent path.
fn zero_bit(value: &BigUint, _: u64) -> bool {
    assert!(get(value) == N::ZERO, "power zero bit API domain exceeded");
    false
}
fn reject_modpow(_: &BigUint, _: &BigUint, _: &BigUint) -> BigUint {
    panic!("power modular arithmetic boundary reached")
}
fn reject_conversion(_: &ValueBigUint) -> Option<BigInt> {
    panic!("power signed conversion boundary reached")
}
// Keep real Sign and a numeric256 handle in the real magnitude field. This
// representation is local to power proofs; ordering's signed handle pool is
// unchanged. assign_from_slice avoids recursively calling the stubbed constructor.
pub(super) fn signed_from_numeric(sign: Sign, magnitude: BigUint) -> BigInt {
    let n = get(&magnitude);
    if sign == Sign::NoSign || n == N::ZERO {
        return BigInt::ZERO;
    }
    let raw = magnitude.iter_u64_digits().next().unwrap();
    let mut value = BigInt::ZERO;
    value.assign_from_slice(sign, &[raw as u32]);
    value
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(crate::public_big_ops::ordering::from_biguint, signed_from_numeric)]
#[kani::stub(num_bigint::BigUint::modpow, zero_modpow)]
#[kani::stub(crate::public_big_ops::bit, zero_bit)]
fn power_zero_api_and_signed_bridge_preserve_values() {
    init();
    let n = N {
        lo: kani::any(),
        hi: kani::any(),
    };
    let sign = [Sign::NoSign, Sign::Plus, Sign::Minus][usize::from(kani::any::<u8>() % 3)];
    let value = BigInt::from_biguint(sign, put(n));
    let canonical = veryl_formal_semantics::bits256::Signed256::new(
        if sign == Sign::NoSign { N::ZERO } else { n },
        sign == Sign::Minus,
    );
    assert!(
        get(value.magnitude()) == canonical.magnitude
            && value.sign()
                == if canonical.magnitude == N::ZERO {
                    Sign::NoSign
                } else if canonical.negative {
                    Sign::Minus
                } else {
                    Sign::Plus
                },
        "power numeric API mismatch"
    );
    let modulus = N {
        lo: kani::any(),
        hi: kani::any(),
    };
    kani::assume(modulus != N::ZERO);
    let base = put(n);
    let zero = put(N::ZERO);
    let m = put(modulus);
    assert!(!zero.bit(kani::any()), "power numeric API mismatch");
    let output = base.modpow(&zero, &m);
    assert!(
        get(&output) == N::from_u128(u128::from(modulus != N::from_u128(1)))
            && get(&base) == n
            && get(&zero) == N::ZERO
            && get(&m) == modulus,
        "power numeric API mismatch"
    );
    kani::cover!(
        sign == Sign::Minus && n == N::ZERO,
        "power bridge negative zero is reachable"
    );
    kani::cover!(
        sign == Sign::NoSign && n != N::ZERO,
        "power bridge no sign is reachable"
    );
    kani::cover!(
        sign == Sign::Minus && n.hi != 0,
        "power bridge wide magnitude is reachable"
    );
    kani::cover!(
        modulus == N::from_u128(1),
        "power zero API modulus one is reachable"
    );
    kani::cover!(
        modulus.hi == 1 && modulus.lo == 0,
        "power zero API bit128 modulus is reachable"
    );
}

static mut BASE: *const Value = std::ptr::null();
static mut WIDTH: usize = 0;
static mut SIGN: bool = false;
static mut CALLS: usize = 0;
fn observed_expand(value: &Value, width: usize, signed: bool) -> std::borrow::Cow<'_, Value> {
    unsafe {
        assert!(
            CALLS == 0 && std::ptr::eq(value, BASE) && width == WIDTH && signed == SIGN,
            "wide operator semantics mismatch"
        );
        CALLS += 1;
    }
    prepared_wide(value, width, signed)
}

// Construct the full zero-exponent partition directly. Width and sign remain
// arbitrary, while known-zero planes need no symbolic equality constraints.
fn zero_exponent(width: usize) -> (Value, Operand) {
    let a = Operand {
        planes: (0, 0),
        width,
        signed: kani::any(),
    };
    let v = if width <= 64 {
        Value::from_u128(0, 0, width, a.signed)
    } else {
        Value::BigUint(ValueBigUint {
            payload: Box::new(BigUint::ZERO),
            mask_xz: Box::new(BigUint::ZERO),
            width: width as u32,
            signed: a.signed,
        })
    };
    preserve(&v, a);
    (v, a)
}

fn check<const CASE: u8, const ORIGINAL: bool>() -> (Operand, Operand, usize, bool) {
    init();
    let width = usize::from(kani::any::<u8>() & 63) + 65;
    let source = if ORIGINAL {
        usize::from(kani::any::<u8>() & 127) + 1
    } else {
        width
    };
    let ew = usize::from(kani::any::<u8>() & 127) + 1;
    kani::assume(source <= width);
    let (x, a) = input(source);
    let (y, b) = if CASE == 2 {
        zero_exponent(ew)
    } else {
        input(ew)
    };
    let caller: bool = kani::any();
    let negative = b.signed && b.planes.0 >> (ew - 1) & 1 != 0;
    kani::assume(match CASE {
        0 => b.planes.1 != 0,
        1 => b.planes.1 == 0 && negative,
        _ => b.planes == (0, 0),
    });
    unsafe {
        BASE = &x;
        WIDTH = width;
        SIGN = caller;
        CALLS = 0;
    }
    let output = Op::Pow.eval_value_binary(&x, &y, width, caller, &mut MaskCache::default());
    if ORIGINAL {
        assert!(unsafe { CALLS } == 1, "wide operator semantics mismatch");
    }
    let prepared = expanded_planes(a.planes, source, width, a.signed, caller);
    let base_signed = a.signed && (source == width || caller);
    let expected = power_exception(prepared, b.planes, width, base_signed, ew, b.signed);
    assert!(
        (output.payload_u128(), output.mask_xz_u128()) == expected,
        "wide operator semantics mismatch"
    );
    let p = usize::from(kani::any::<u8>() & 127);
    result(
        &output,
        width,
        b.planes.1 == 0 && base_signed,
        p,
        read_logic(expected.0, expected.1, p),
    );
    preserve(&x, a);
    preserve(&y, b);
    (a, b, width, caller)
}

fn cover_common(a: Operand, b: Operand, width: usize, caller: bool) {
    kani::cover!(width == 65, "wide power minimum result is reachable");
    kani::cover!(width == 128, "wide power maximum result is reachable");
    kani::cover!(b.width == 1, "wide power minimum exponent is reachable");
    kani::cover!(b.width == 128, "wide power maximum exponent is reachable");
    kani::cover!(b.width <= 64, "wide power native exponent is reachable");
    kani::cover!(b.width > 64, "wide power big exponent is reachable");
    kani::cover!(
        b.width > width,
        "wide power exponent wider than result is reachable"
    );
    kani::cover!(a.signed && caller, "wide power signed context is reachable");
    kani::cover!(
        !a.signed && !caller,
        "wide power unsigned context is reachable"
    );
}
fn cover_layouts(a: Operand, b: Operand, width: usize) {
    kani::cover!(
        a.width == 64 && b.width == 1 && width == 128,
        "wide power native native source is reachable"
    );
    kani::cover!(
        a.width == 64 && b.width == 65 && width == 128,
        "wide power native big source is reachable"
    );
    kani::cover!(
        a.width == 65 && b.width == 64 && width == 128,
        "wide power big native source is reachable"
    );
    kani::cover!(
        a.width == 65 && b.width == 128 && width == 128,
        "wide power big big source is reachable"
    );
}
fn cover_unknown(a: Operand, b: Operand, _: usize, caller: bool) {
    kani::cover!(
        a.planes.1 == 0,
        "wide power only exponent unknown is reachable"
    );
    kani::cover!(
        a.planes.1 != 0,
        "wide power both operands unknown is reachable"
    );
    kani::cover!(
        b.signed && b.planes.0 >> (b.width - 1) & 1 != 0,
        "wide power unknown negative exponent is reachable"
    );
    kani::cover!(
        b.width == 128 && b.planes.1 >> 64 != 0,
        "wide power upper exponent unknown is reachable"
    );
    kani::cover!(
        a.signed && !caller && a.planes.1 == 0,
        "wide power unknown exponent clears result sign is reachable"
    );
}
fn cover_negative(a: Operand, b: Operand, width: usize, caller: bool) {
    let p = expanded_planes(a.planes, a.width, width, a.signed, caller);
    let signed = a.signed && (a.width == width || caller);
    let mask = u128::MAX >> (128 - width);
    kani::cover!(
        p == (0, 0),
        "wide power negative exponent zero base is reachable"
    );
    kani::cover!(
        p == (1, 0),
        "wide power negative exponent one base is reachable"
    );
    kani::cover!(
        p.1 != 0,
        "wide power negative exponent unknown base is reachable"
    );
    kani::cover!(
        signed && p == (mask, 0) && b.planes.0 & 1 != 0,
        "wide power minus one odd negative is reachable"
    );
    kani::cover!(
        signed && p == (mask, 0) && b.planes.0 & 1 == 0,
        "wide power minus one even negative is reachable"
    );
    kani::cover!(
        p == (2, 0),
        "wide power negative exponent other base is reachable"
    );
    kani::cover!(
        b.width == 128 && b.planes.0 == 1u128 << 127,
        "wide power minimum signed exponent is reachable"
    );
    kani::cover!(
        !signed && p == (mask, 0),
        "wide power unsigned all ones base is reachable"
    );
}
fn cover_zero(a: Operand, b: Operand, width: usize, caller: bool) {
    let p = expanded_planes(a.planes, a.width, width, a.signed, caller);
    kani::cover!(p == (0, 0), "wide power zero to zero is reachable");
    kani::cover!(p == (2, 0), "wide power other base to zero is reachable");
    kani::cover!(
        a.signed && (a.width == width || caller) && p == (1u128 << (width - 1), 0),
        "wide power signed minimum base to zero is reachable"
    );
    kani::cover!(p.1 != 0, "wide power unknown base to zero is reachable");
    kani::cover!(
        a.width == width && a.signed && !caller,
        "wide power zero exponent retains equal width sign is reachable"
    );
    kani::cover!(
        !a.signed && caller,
        "wide power zero exponent unsigned base is reachable"
    );
    kani::cover!(b.signed, "wide power signed zero exponent is reachable");
    kani::cover!(
        p == (u128::MAX >> (128 - width), 0),
        "wide power all ones to zero is reachable"
    );
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::prepared_wide)]
#[kani::stub(num_bigint::BigUint::modpow, reject_modpow)]
#[kani::stub(veryl_analyzer::value::ValueBigUint::to_bigint, reject_conversion)]
fn wide_power_unknown_prepared_matches_reference() {
    let (a, b, w, s) = check::<0, false>();
    cover_common(a, b, w, s);
    cover_unknown(a, b, w, s);
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, observed_expand)]
#[kani::stub(num_bigint::BigUint::modpow, reject_modpow)]
#[kani::stub(veryl_analyzer::value::ValueBigUint::to_bigint, reject_conversion)]
fn wide_power_unknown_context_matches_reference() {
    let (a, b, w, s) = check::<0, true>();
    cover_common(a, b, w, s);
    cover_unknown(a, b, w, s);
    cover_layouts(a, b, w);
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::prepared_wide)]
#[kani::stub(num_bigint::BigUint::modpow, reject_modpow)]
#[kani::stub(veryl_analyzer::value::ValueBigUint::to_bigint, reject_conversion)]
fn wide_power_negative_prepared_matches_reference() {
    let (a, b, w, s) = check::<1, false>();
    cover_common(a, b, w, s);
    cover_negative(a, b, w, s);
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, observed_expand)]
#[kani::stub(num_bigint::BigUint::modpow, reject_modpow)]
#[kani::stub(veryl_analyzer::value::ValueBigUint::to_bigint, reject_conversion)]
fn wide_power_negative_context_matches_reference() {
    let (a, b, w, s) = check::<1, true>();
    cover_common(a, b, w, s);
    cover_negative(a, b, w, s);
    cover_layouts(a, b, w);
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::prepared_wide)]
#[kani::stub(<num_bigint::BigUint as std::convert::From<u32>>::from, from32)]
#[kani::stub(crate::public_big_ops::ordering::from_biguint, signed_from_numeric)]
#[kani::stub(num_bigint::BigUint::modpow, zero_modpow)]
#[kani::stub(crate::public_big_ops::bit, zero_bit)]
#[kani::stub(
    veryl_analyzer::value::ValueBigUint::to_bigint,
    super::ordering::converted
)]
fn wide_power_zero_prepared_matches_reference() {
    let (a, b, w, s) = check::<2, false>();
    cover_common(a, b, w, s);
    cover_zero(a, b, w, s);
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, observed_expand)]
#[kani::stub(<num_bigint::BigUint as std::convert::From<u32>>::from, from32)]
#[kani::stub(crate::public_big_ops::ordering::from_biguint, signed_from_numeric)]
#[kani::stub(num_bigint::BigUint::modpow, zero_modpow)]
#[kani::stub(crate::public_big_ops::bit, zero_bit)]
#[kani::stub(
    veryl_analyzer::value::ValueBigUint::to_bigint,
    super::ordering::converted
)]
fn wide_power_zero_context_matches_reference() {
    let (a, b, w, s) = check::<2, true>();
    cover_common(a, b, w, s);
    cover_zero(a, b, w, s);
    cover_layouts(a, b, w);
}
