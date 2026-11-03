//! IEEE1800-2023 11.4.3/11.6.1/11.8.2: unsigned power-of-two quotients.
//! Known result65..128; independent nonnarrowing originals1..128 and source
//! flags, caller unsigned. The QUOTIENT is2^k; the divisor need not be single-hot.
//! Membership follows preparation: b>0,b<=a>>k,a-(b<<k)<b. This retains all
//! matching pairs without overflowing the scaled divisor. k0 includes quotient1.
//! The numeric adapter checks the actual operands and exact256-bit b<<k before
//! returning2^k or a-b*2^k. Byte division, full product coordinates and the
//! Euclidean identity with remainder<b validate the mathematical API, not the
//! num-bigint implementation. Scalar subtraction borrows check remainder bits.
//! Actual Op/result constructors execute. Expansion and single mask retrieval
//! reuse separately checked summaries. Original-domain witnesses share the value
//! generator; signed/native arithmetic must be unreachable. Cache history and
//! storage boundaries remain executable comparisons.
use super::*;
use num_bigint::{BigInt, BigUint};
use veryl_formal_semantics::{nonlinear::power_quotient_bit, resize::expanded_planes};
static mut SHIFT: usize = 0;
pub(super) fn configure(k: usize) {
    unsafe {
        SHIFT = k;
    }
    multiplication::reset_product_mask();
}
fn numeric_remainder(a: &BigUint, b: &BigUint) -> u128 {
    let x = get(a);
    let y = get(b);
    let k = unsafe { SHIFT };
    assert!(
        k < 128 && x.hi == 0 && y.hi == 0 && y.lo != 0,
        "power quotient numeric domain exceeded"
    );
    let product = y.checked_shl(k);
    assert!(
        product.hi == 0 && product.lo <= x.lo,
        "power quotient numeric domain exceeded"
    );
    let r = x.lo - product.lo;
    assert!(r < y.lo, "power quotient numeric domain exceeded");
    r
}
pub(super) fn divide<'a: 'a, 'b: 'b>(a: &'a BigUint, b: &'b BigUint) -> BigUint {
    let _ = numeric_remainder(a, b);
    from128(1u128 << unsafe { SHIFT })
}
pub(super) fn remainder<'a: 'a, 'b: 'b>(a: &'a BigUint, b: &'b BigUint) -> BigUint {
    from128(numeric_remainder(a, b))
}
fn reject_signed(_: BigInt, _: BigInt) -> BigInt {
    panic!("power quotient signed boundary reached")
}
fn reject_native(
    _: &veryl_analyzer::value::ValueU64,
    _: &veryl_analyzer::value::ValueU64,
    _: usize,
    _: bool,
    _: bool,
) -> veryl_analyzer::value::ValueU64 {
    panic!("power quotient native boundary reached")
}
fn inputs<const ORIGINAL: bool>() -> (Value, Operand, Value, Operand, usize, u128, u128, usize) {
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
    let k = usize::from(kani::any::<u8>() & 127);
    kani::assume(aw <= w && bw <= w && k < w);
    let (x, a) = identities::known_input(aw, None);
    let (y, b) = identities::known_input(bw, None);
    let ap = expanded_planes(a.planes, aw, w, a.signed, false).0;
    let bp = expanded_planes(b.planes, bw, w, b.signed, false).0;
    kani::assume(bp != 0 && bp <= ap >> k && ap - (bp << k) < bp);
    configure(k);
    (x, a, y, b, w, ap, bp, k)
}
fn check<const REM: bool, const ORIGINAL: bool>() {
    let (x, a, y, b, w, ap, bp, k) = inputs::<ORIGINAL>();
    special::observe_binary_sources(&x, &y, w, false);
    let out = if REM { Op::Rem } else { Op::Div }.eval_value_binary(
        &x,
        &y,
        w,
        false,
        &mut MaskCache::default(),
    );
    if ORIGINAL {
        assert!(
            special::observed_calls() == 2,
            "wide operator semantics mismatch"
        );
    }
    let expected = if REM { ap - (bp << k) } else { 1u128 << k };
    preserve(
        &out,
        Operand {
            planes: (expected, 0),
            width: w,
            signed: false,
        },
    );
    let p = usize::from(kani::any::<u8>() & 127);
    result(
        &out,
        w,
        false,
        p,
        if power_quotient_bit(ap, bp, k, w, p, REM) {
            Logic4::One
        } else {
            Logic4::Zero
        },
    );
    preserve(&x, a);
    preserve(&y, b);
    if ORIGINAL {
        kani::cover!(
            w == 65 && a.width == 64 && b.width == 65 && k == 2 && ap == 13 && bp == 3,
            "power quotient class original context is reachable"
        );
    } else {
        kani::cover!(
            w == 65 && k == 1 && bp == 3 && ap == 7,
            "power quotient class minimum nonunit quotient is reachable"
        );
        kani::cover!(
            w == 128 && k == 127,
            "power quotient class largest quotient is reachable"
        );
        kani::cover!(k == 0, "power quotient class unit quotient is reachable");
        kani::cover!(
            k == 2 && bp == 3 && ap == 13,
            "power quotient class nonpower divisor is reachable"
        );
        kani::cover!(
            ap - (bp << k) + 1 == bp,
            "power quotient class maximum remainder is reachable"
        );
    }
}
#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, divide)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, remainder)]
#[kani::stub(<num_bigint::BigInt as std::ops::Div<num_bigint::BigInt>>::div, reject_signed)]
#[kani::stub(<num_bigint::BigInt as std::ops::Rem<num_bigint::BigInt>>::rem, reject_signed)]
#[kani::stub(veryl_analyzer::value::ValueU64::divide_remainder, reject_native)]
#[kani::stub(
    crate::public_big_ops::cached_mask,
    super::multiplication::product_mask
)]

fn power_quotient_numeric_api_matches_byte_division() {
    init();
    let x = kani::any::<u8>();
    let y = kani::any::<u8>();
    let k = usize::from(kani::any::<u8>() & 7);
    kani::assume(y != 0 && y <= x >> k && u16::from(x) - (u16::from(y) << k) < u16::from(y));
    configure(k);
    let a = from64(u64::from(x));
    let b = from64(u64::from(y));
    let q = &a / &b;
    let r = &a % &b;
    assert!(
        get(&q) == N::from_u128(u128::from(x / y))
            && get(&r) == N::from_u128(u128::from(x % y))
            && get(&a) == N::from_u128(u128::from(x))
            && get(&b) == N::from_u128(u128::from(y)),
        "power quotient byte API mismatch"
    );
    kani::cover!(k == 0, "power quotient API unit quotient is reachable");
    kani::cover!(
        y == 3 && x == 13 && k == 2,
        "power quotient API nonpower divisor is reachable"
    );
    kani::cover!(x == 255, "power quotient API maximum dividend is reachable");
    kani::cover!(
        k == 7,
        "power quotient API largest byte quotient is reachable"
    );
}
#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, divide)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, remainder)]
#[kani::stub(<num_bigint::BigInt as std::ops::Div<num_bigint::BigInt>>::div, reject_signed)]
#[kani::stub(<num_bigint::BigInt as std::ops::Rem<num_bigint::BigInt>>::rem, reject_signed)]
#[kani::stub(veryl_analyzer::value::ValueU64::divide_remainder, reject_native)]
#[kani::stub(
    crate::public_big_ops::cached_mask,
    super::multiplication::product_mask
)]

fn power_quotient_full_product_matches_scalar_coordinates() {
    init();
    let y: u128 = kani::any();
    let k = usize::from(kani::any::<u8>() & 127);
    let p = usize::from(kani::any::<u8>());
    let n = N::from_u128(y).checked_shl(k);
    let bit = if p < 128 {
        n.lo >> p & 1 != 0
    } else {
        n.hi >> (p - 128) & 1 != 0
    };
    let expected = p >= k && p - k < 128 && y >> (p - k) & 1 != 0;
    assert!(bit == expected, "power quotient product mismatch");
    kani::cover!(k == 0, "power quotient product unit quotient is reachable");
    kani::cover!(
        k == 127 && y >> 127 != 0 && p == 254,
        "power quotient product bit254 is reachable"
    );
    kani::cover!(
        k == 1 && y == 1u128 << 63 && p == 64,
        "power quotient product word crossing is reachable"
    );
    kani::cover!(n.hi != 0, "power quotient product high half is reachable");
    kani::cover!(y == 0, "power quotient product zero divisor is reachable");
}
#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, divide)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, remainder)]
#[kani::stub(<num_bigint::BigInt as std::ops::Div<num_bigint::BigInt>>::div, reject_signed)]
#[kani::stub(<num_bigint::BigInt as std::ops::Rem<num_bigint::BigInt>>::rem, reject_signed)]
#[kani::stub(veryl_analyzer::value::ValueU64::divide_remainder, reject_native)]
#[kani::stub(
    crate::public_big_ops::cached_mask,
    super::multiplication::product_mask
)]

fn power_quotient_numeric_api_satisfies_euclidean_identity() {
    init();
    let w = usize::from(kani::any::<u8>() & 127) + 1;
    let k = usize::from(kani::any::<u8>() & 127);
    kani::assume(k < w);
    let mask = u128::MAX >> (128 - w);
    let x = kani::any::<u128>() & mask;
    let y = kani::any::<u128>() & mask;
    kani::assume(y != 0 && y <= x >> k && x - (y << k) < y);
    configure(k);
    let a = from128(x);
    let b = from128(y);
    let q = &a / &b;
    let rem = &a % &b;
    let r = get(&rem);
    let product = get(&b).checked_shl(k);
    let p = usize::from(kani::any::<u8>() & 127);
    assert!(
        get(&q) == N::from_u128(1u128 << k)
            && product.hi == 0
            && r.hi == 0
            && r.lo < y
            && product.checked_add(r) == get(&a)
            && (r.lo >> p & 1 != 0) == power_quotient_bit(x, y, k, w, p, true)
            && (get(&q).lo >> p & 1 != 0) == power_quotient_bit(x, y, k, w, p, false)
            && get(&a) == N::from_u128(x)
            && get(&b) == N::from_u128(y),
        "power quotient Euclidean mismatch"
    );
    kani::cover!(w == 1, "power quotient Euclidean unit width is reachable");
    kani::cover!(
        w == 128 && x == u128::MAX,
        "power quotient Euclidean maximum dividend is reachable"
    );
    kani::cover!(
        k == 127,
        "power quotient Euclidean largest quotient is reachable"
    );
    kani::cover!(
        y == 3 && k == 2,
        "power quotient Euclidean nonpower divisor is reachable"
    );
    kani::cover!(
        r.lo + 1 == y,
        "power quotient Euclidean maximum remainder is reachable"
    );
    kani::cover!(
        x == 1u128 << 64 && y == (1u128 << 63) - 1 && k == 1,
        "power quotient Euclidean word borrow is reachable"
    );
}
#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, divide)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, remainder)]
#[kani::stub(<num_bigint::BigInt as std::ops::Div<num_bigint::BigInt>>::div, reject_signed)]
#[kani::stub(<num_bigint::BigInt as std::ops::Rem<num_bigint::BigInt>>::rem, reject_signed)]
#[kani::stub(veryl_analyzer::value::ValueU64::divide_remainder, reject_native)]
#[kani::stub(
    crate::public_big_ops::cached_mask,
    super::multiplication::product_mask
)]

fn power_quotient_original_domain_preserves_prepared_values() {
    let (x, a, y, b, w, ap, bp, k) = inputs::<true>();
    special::observe_binary_sources(&x, &y, w, false);
    let xp = special::observed_expand(&x, w, false);
    let yp = special::observed_expand(&y, w, false);
    let sx = a.signed && a.width == w;
    let sy = b.signed && b.width == w;
    preserve(
        xp.as_ref(),
        Operand {
            planes: (ap, 0),
            width: w,
            signed: sx,
        },
    );
    preserve(
        yp.as_ref(),
        Operand {
            planes: (bp, 0),
            width: w,
            signed: sy,
        },
    );
    let p = usize::from(kani::any::<u8>() & 127);
    result(xp.as_ref(), w, sx, p, read_logic(ap, 0, p));
    result(yp.as_ref(), w, sy, p, read_logic(bp, 0, p));
    assert!(
        special::observed_calls() == 2 && bp != 0 && bp <= ap >> k && ap - (bp << k) < bp,
        "power quotient original domain mismatch"
    );
    preserve(&x, a);
    preserve(&y, b);
    kani::cover!(
        a.width == 64 && b.width == 64 && w == 128,
        "power quotient domain native native is reachable"
    );
    kani::cover!(
        a.width == 64 && b.width == 65 && w == 128,
        "power quotient domain native big is reachable"
    );
    kani::cover!(
        a.width == 65 && b.width == 64 && w == 128,
        "power quotient domain big native is reachable"
    );
    kani::cover!(
        a.width == 65 && b.width == 127 && w == 128,
        "power quotient domain big big is reachable"
    );
    kani::cover!(
        a.signed && !b.signed,
        "power quotient domain left signed only is reachable"
    );
    kani::cover!(
        !a.signed && b.signed,
        "power quotient domain right signed only is reachable"
    );
    kani::cover!(
        k == 127,
        "power quotient domain largest quotient is reachable"
    );
}
#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, divide)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, remainder)]
#[kani::stub(<num_bigint::BigInt as std::ops::Div<num_bigint::BigInt>>::div, reject_signed)]
#[kani::stub(<num_bigint::BigInt as std::ops::Rem<num_bigint::BigInt>>::rem, reject_signed)]
#[kani::stub(veryl_analyzer::value::ValueU64::divide_remainder, reject_native)]
#[kani::stub(
    crate::public_big_ops::cached_mask,
    super::multiplication::product_mask
)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::prepared_wide)]

fn wide_power_quotient_div_matches_reference() {
    check::<false, false>();
}
#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, divide)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, remainder)]
#[kani::stub(<num_bigint::BigInt as std::ops::Div<num_bigint::BigInt>>::div, reject_signed)]
#[kani::stub(<num_bigint::BigInt as std::ops::Rem<num_bigint::BigInt>>::rem, reject_signed)]
#[kani::stub(veryl_analyzer::value::ValueU64::divide_remainder, reject_native)]
#[kani::stub(
    crate::public_big_ops::cached_mask,
    super::multiplication::product_mask
)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::special::observed_expand)]

fn wide_power_quotient_div_context_matches_reference() {
    check::<false, true>();
}
#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, divide)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, remainder)]
#[kani::stub(<num_bigint::BigInt as std::ops::Div<num_bigint::BigInt>>::div, reject_signed)]
#[kani::stub(<num_bigint::BigInt as std::ops::Rem<num_bigint::BigInt>>::rem, reject_signed)]
#[kani::stub(veryl_analyzer::value::ValueU64::divide_remainder, reject_native)]
#[kani::stub(
    crate::public_big_ops::cached_mask,
    super::multiplication::product_mask
)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::prepared_wide)]

fn wide_power_quotient_rem_matches_reference() {
    check::<true, false>();
}
#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Div<&num_bigint::BigUint>>::div, divide)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, remainder)]
#[kani::stub(<num_bigint::BigInt as std::ops::Div<num_bigint::BigInt>>::div, reject_signed)]
#[kani::stub(<num_bigint::BigInt as std::ops::Rem<num_bigint::BigInt>>::rem, reject_signed)]
#[kani::stub(veryl_analyzer::value::ValueU64::divide_remainder, reject_native)]
#[kani::stub(
    crate::public_big_ops::cached_mask,
    super::multiplication::product_mask
)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::special::observed_expand)]

fn wide_power_quotient_rem_context_matches_reference() {
    check::<true, true>();
}
