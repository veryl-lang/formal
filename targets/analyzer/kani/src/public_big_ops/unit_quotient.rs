//! IEEE1800-2023 11.4.3/11.6.1/11.8.2: unsigned quotient-one division.
//! Known normalized result65..128 and nonnarrowing original widths1..128,
//! with independent source flags. Caller mode is unsigned. Membership follows
//! preparation: b>0, a>=b and a-b<b, including divisors above2^127 without
//! overflowing2*b. Zero/XZ and signed arithmetic retain separate contracts.
//! Actual Op and unsigned result constructors execute under mathematical numeric
//! APIs. Byte division/remainder and the full Euclidean identity with r<b
//! validate this specialization; independent scalar borrow bits check remainder.
//! No num-bigint implementation proof. Expansion and the one-call result-mask
//! summary reuse their separately checked interfaces. Original-domain witnesses
//! share the value-proof generator; cache histories and boundaries stay direct.
use super::*;
use num_bigint::{BigInt, BigUint};
use veryl_formal_semantics::{nonlinear::unit_quotient_bit, resize::expanded_planes};

fn numeric_remainder(a: &BigUint, b: &BigUint) -> u128 {
    let x = get(a);
    let y = get(b);
    assert!(
        x.hi == 0 && y.hi == 0 && y.lo != 0 && x.lo >= y.lo,
        "unit quotient numeric domain exceeded"
    );
    let r = x.lo - y.lo;
    assert!(r < y.lo, "unit quotient numeric domain exceeded");
    r
}
fn divide<'a: 'a, 'b: 'b>(a: &'a BigUint, b: &'b BigUint) -> BigUint {
    let _ = numeric_remainder(a, b);
    from64(1)
}
fn remainder<'a: 'a, 'b: 'b>(a: &'a BigUint, b: &'b BigUint) -> BigUint {
    from128(numeric_remainder(a, b))
}
fn reject_signed(_: BigInt, _: BigInt) -> BigInt {
    panic!("unit quotient signed boundary reached")
}
fn reject_native(
    _: &veryl_analyzer::value::ValueU64,
    _: &veryl_analyzer::value::ValueU64,
    _: usize,
    _: bool,
    _: bool,
) -> veryl_analyzer::value::ValueU64 {
    panic!("unit quotient native boundary reached")
}
fn inputs<const ORIGINAL: bool>() -> (Value, Operand, Value, Operand, usize, u128, u128) {
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
    let (x, a) = identities::known_input(aw, None);
    let (y, b) = identities::known_input(bw, None);
    let ap = expanded_planes(a.planes, aw, w, a.signed, false).0;
    let bp = expanded_planes(b.planes, bw, w, b.signed, false).0;
    kani::assume(bp != 0 && ap >= bp && ap - bp < bp);
    multiplication::reset_product_mask();
    (x, a, y, b, w, ap, bp)
}
fn check<const REM: bool, const ORIGINAL: bool>() {
    let (x, a, y, b, w, ap, bp) = inputs::<ORIGINAL>();
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
    let expected = if REM { ap - bp } else { 1 };
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
        if unit_quotient_bit(ap, bp, w, p, REM) {
            Logic4::One
        } else {
            Logic4::Zero
        },
    );
    preserve(&x, a);
    preserve(&y, b);
    if ORIGINAL {
        kani::cover!(
            w == 65 && a.width == 64 && b.width == 65 && ap == 7 && bp == 5,
            "unit quotient class original context is reachable"
        );
    } else {
        kani::cover!(
            w == 65 && ap > bp,
            "unit quotient class minimum width is reachable"
        );
        kani::cover!(
            w == 128 && bp >> 127 != 0,
            "unit quotient class maximum upper divisor is reachable"
        );
        kani::cover!(ap == bp, "unit quotient class exact division is reachable");
        kani::cover!(
            ap - bp == 1,
            "unit quotient class adjacent operands is reachable"
        );
        kani::cover!(
            ap - bp + 1 == bp,
            "unit quotient class maximum remainder is reachable"
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

fn unit_quotient_numeric_api_matches_byte_division() {
    init();
    let x = kani::any::<u8>();
    let y = kani::any::<u8>();
    kani::assume(y != 0 && x >= y && x - y < y);
    let a = from64(u64::from(x));
    let b = from64(u64::from(y));
    let q = &a / &b;
    let r = &a % &b;
    assert!(
        get(&q) == N::from_u128(u128::from(x / y))
            && get(&r) == N::from_u128(u128::from(x % y))
            && get(&a) == N::from_u128(u128::from(x))
            && get(&b) == N::from_u128(u128::from(y)),
        "unit quotient byte API mismatch"
    );
    kani::cover!(y == 1, "unit quotient API unit divisor is reachable");
    kani::cover!(x == 255, "unit quotient API maximum dividend is reachable");
    kani::cover!(x == y, "unit quotient API exact division is reachable");
    kani::cover!(y > 128, "unit quotient API upper divisor is reachable");
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

fn unit_quotient_numeric_api_satisfies_euclidean_identity() {
    init();
    let w = usize::from(kani::any::<u8>() & 127) + 1;
    let mask = u128::MAX >> (128 - w);
    let x = kani::any::<u128>() & mask;
    let y = kani::any::<u128>() & mask;
    kani::assume(y != 0 && x >= y && x - y < y);
    let a = from128(x);
    let b = from128(y);
    let q = &a / &b;
    let rem = &a % &b;
    let r = get(&rem);
    let p = usize::from(kani::any::<u8>() & 127);
    // q=1, a=b+r and0<=r<b uniquely characterize unsigned division.
    // checked_add retains a possible bit128 rather than wrapping the identity.
    assert!(
        get(&q) == N::from_u128(1)
            && r.hi == 0
            && r.lo < y
            && get(&b).checked_add(r) == get(&a)
            && (r.lo >> p & 1 != 0) == unit_quotient_bit(x, y, w, p, true)
            && (get(&q).lo >> p & 1 != 0) == unit_quotient_bit(x, y, w, p, false)
            && get(&a) == N::from_u128(x)
            && get(&b) == N::from_u128(y),
        "unit quotient Euclidean mismatch"
    );
    kani::cover!(w == 1, "unit quotient Euclidean unit width is reachable");
    kani::cover!(
        w == 128 && x == u128::MAX,
        "unit quotient Euclidean maximum dividend is reachable"
    );
    kani::cover!(
        y >> 127 != 0,
        "unit quotient Euclidean upper divisor is reachable"
    );
    kani::cover!(
        r.lo + 1 == y,
        "unit quotient Euclidean maximum remainder is reachable"
    );
    kani::cover!(
        r.lo == 0,
        "unit quotient Euclidean exact division is reachable"
    );
    kani::cover!(
        x == 1u128 << 64 && y == (1u128 << 64) - 1,
        "unit quotient Euclidean word borrow is reachable"
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

fn unit_quotient_original_domain_preserves_prepared_values() {
    let (x, a, y, b, w, ap, bp) = inputs::<true>();
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
        special::observed_calls() == 2 && bp != 0 && ap >= bp && ap - bp < bp,
        "unit quotient original domain mismatch"
    );
    preserve(&x, a);
    preserve(&y, b);
    kani::cover!(
        a.width == 64 && b.width == 64 && w == 128,
        "unit quotient domain native native is reachable"
    );
    kani::cover!(
        a.width == 64 && b.width == 65 && w == 128,
        "unit quotient domain native big is reachable"
    );
    kani::cover!(
        a.width == 65 && b.width == 64 && w == 128,
        "unit quotient domain big native is reachable"
    );
    kani::cover!(
        a.width == 65 && b.width == 127 && w == 128,
        "unit quotient domain big big is reachable"
    );
    kani::cover!(
        a.signed && !b.signed,
        "unit quotient domain left signed only is reachable"
    );
    kani::cover!(
        !a.signed && b.signed,
        "unit quotient domain right signed only is reachable"
    );
    kani::cover!(
        bp >> 127 != 0,
        "unit quotient domain upper divisor is reachable"
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

fn wide_unit_quotient_div_matches_reference() {
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

fn wide_unit_quotient_div_context_matches_reference() {
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

fn wide_unit_quotient_rem_matches_reference() {
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

fn wide_unit_quotient_rem_context_matches_reference() {
    check::<true, true>();
}
