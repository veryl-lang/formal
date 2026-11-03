//! IEEE1800-2023 11.4.3/Table11-4,11.6.1/11.8.2: single-hot-base powers.
//! Known positive exponents retain original widths1..128/signs. Result65..128;
//! original base1..128 is nonnarrowing, with independent source flags/caller.
//! Membership uses a separately checked necessary-and-sufficient expansion
//! condition; neither direction narrows the original domain. Signed minimum
//! bases are included.
//! Actual Op, pow_mod_width sign correction and result constructors execute.
//! Numeric APIs are mathematical summaries, not num-bigint algorithm proofs.
//! Byte products, independent scalar coordinates and a bounded zero/successor
//! law under the checked single-hot multiplication identity validate modpow.
//! The original-domain harness validates specialized expansion against the
//! reference with the same generator; all source storage pairs remain reachable.
//! The complete exponent, base magnitude and 129-bit modulus are observed.
//! Signed conversion reuses the general checked zero/minimum conversion summary.
use super::*;
use num_bigint::BigUint;
use veryl_formal_semantics::{
    nonlinear::{one_hot_power_bit, one_hot_product_word},
    resize::{expanded_planes, single_hot_expansion_member},
};
static mut WIDTH: usize = 0;
static mut SHIFT: usize = 0;
static mut EXPONENT: u128 = 0;
static mut MOD_CALLS: usize = 0;
static mut SOURCE: *const Value = std::ptr::null();
static mut CALLER: bool = false;
static mut EXPANDS: usize = 0;
fn configure(w: usize, k: usize, e: u128, x: &Value, caller: bool) {
    unsafe {
        WIDTH = w;
        SHIFT = k;
        EXPONENT = e;
        MOD_CALLS = 0;
        SOURCE = x;
        CALLER = caller;
        EXPANDS = 0;
    }
    signed_small::configure([1u128 << k, 0], [k == w - 1, false], w);
    multiplication::reset_product_mask();
}
fn modular_api(base: &BigUint, exponent: &BigUint, modulus: &BigUint) -> BigUint {
    let (w, k) = unsafe { (WIDTH, SHIFT) };
    assert!(
        (1..=128).contains(&w) && k < w,
        "single hot power modular domain exceeded"
    );
    let e = get(exponent);
    assert!(
        get(base) == N::from_u128(1u128 << k)
            && e.hi == 0
            && get(modulus) == N::from_u128(1).checked_shl(w),
        "single hot power modular domain exceeded"
    );
    let word = if k == 0 {
        1
    } else if e.lo > 127 {
        0
    } else {
        let position = (k as u16) * (e.lo as u16);
        if position < (w as u16) {
            1u128 << (position as usize)
        } else {
            0
        }
    };
    from128(word)
}
fn observed_modular(base: &BigUint, exponent: &BigUint, modulus: &BigUint) -> BigUint {
    unsafe {
        assert!(
            MOD_CALLS == 0
                && get(base) == N::from_u128(1u128 << SHIFT)
                && get(exponent) == N::from_u128(EXPONENT)
                && get(modulus) == N::from_u128(1).checked_shl(WIDTH),
            "wide operator semantics mismatch"
        );
        MOD_CALLS += 1;
    }
    modular_api(base, exponent, modulus)
}
fn observed_expand(value: &Value, width: usize, signed: bool) -> std::borrow::Cow<'_, Value> {
    let k = unsafe {
        assert!(
            EXPANDS == 0 && std::ptr::eq(value, SOURCE) && width == WIDTH && signed == CALLER,
            "wide operator semantics mismatch"
        );
        EXPANDS += 1;
        SHIFT
    };
    let source = value.width();
    assert!(
        (1..=128).contains(&source)
            && (65..=128).contains(&width)
            && source <= width
            && k < source
            && value.payload_u128() == 1u128 << k
            && value.mask_xz_u128() == 0
            && (source == width || !signed || !value.signed() || k + 1 < source),
        "single hot power expansion domain exceeded"
    );
    if source == width {
        let _ = from128(0);
        let _ = from128(0);
        std::borrow::Cow::Borrowed(value)
    } else {
        std::borrow::Cow::Owned(Value::BigUint(ValueBigUint {
            payload: Box::new(from128(1u128 << k)),
            mask_xz: Box::new(from128(0)),
            width: width as u32,
            signed: value.signed() && signed,
        }))
    }
}
fn reject_native(
    _: &veryl_analyzer::value::ValueU64,
    _: &veryl_analyzer::value::ValueU64,
    _: usize,
) -> veryl_analyzer::value::ValueU64 {
    panic!("single hot power native arithmetic boundary reached")
}
fn inputs<const ORIGINAL: bool>() -> (Value, Operand, Value, Operand, usize, bool, usize, bool) {
    init();
    let width = usize::from(kani::any::<u8>() & 63) + 65;
    let aw = if ORIGINAL {
        usize::from(kani::any::<u8>() & 127) + 1
    } else {
        width
    };
    let bw = usize::from(kani::any::<u8>() & 127) + 1;
    let k = usize::from(kani::any::<u8>() & 127);
    kani::assume(aw <= width && k < width);
    let caller: bool = kani::any();
    let (x, a) = identities::known_input(aw, Some(1u128 << k));
    let (y, b) = identities::known_input(bw, None);
    kani::assume(single_hot_expansion_member(
        a.planes.0, aw, width, a.signed, caller, k,
    ));
    kani::assume(b.planes.0 != 0 && !(b.signed && b.planes.0 >> (bw - 1) & 1 != 0));
    let result_sign = a.signed && (aw == width || caller);
    (x, a, y, b, width, caller, k, result_sign)
}
fn check<const ORIGINAL: bool, const SIGNED: bool>() {
    let (x, a, y, b, w, caller, k, result_sign) = inputs::<ORIGINAL>();
    // The signed and unsigned harnesses partition the complete input domain.
    kani::assume(result_sign == SIGNED);
    configure(w, k, b.planes.0, &x, caller);
    let out = Op::Pow.eval_value_binary(&x, &y, w, caller, &mut MaskCache::default());
    assert!(
        unsafe { MOD_CALLS == 1 && EXPANDS == 1 },
        "wide operator semantics mismatch"
    );
    let p = usize::from(kani::any::<u8>() & 127);
    result(
        &out,
        w,
        SIGNED,
        p,
        if one_hot_power_bit(k, b.planes.0, w, p) {
            Logic4::One
        } else {
            Logic4::Zero
        },
    );
    preserve(&x, a);
    preserve(&y, b);
    if ORIGINAL {
        if SIGNED {
            kani::cover!(
                w == 65 && a.width == 64 && b.width == 128 && b.planes.0 == 129 && k == 1,
                "single hot power class signed original context is reachable"
            );
        } else {
            kani::cover!(
                w == 65 && a.width == 64 && b.width == 128 && b.planes.0 == 129 && k == 1,
                "single hot power class unsigned original context is reachable"
            );
        }
    } else if SIGNED {
        kani::cover!(
            w == 65,
            "single hot power class minimum signed width is reachable"
        );
        kani::cover!(k == 0, "single hot power class unit base is reachable");
        kani::cover!(
            k == 127 && b.planes.0 == 1,
            "single hot power class signed minimum is reachable"
        );
        kani::cover!(
            b.width == 128 && b.signed,
            "single hot power class wide signed exponent is reachable"
        );
    } else {
        kani::cover!(
            w == 128,
            "single hot power class maximum unsigned width is reachable"
        );
        kani::cover!(
            b.width == 128 && b.planes.0 == u128::MAX && k > 0,
            "single hot power class full unsigned exponent is reachable"
        );
        kani::cover!(
            w == 128 && k == 1 && b.planes.0 == 127,
            "single hot power class high result bit is reachable"
        );
        kani::cover!(
            w == 128 && k == 1 && b.planes.0 == 128,
            "single hot power class first zero result is reachable"
        );
    }
}
#[kani::proof]
#[kani::unwind(5)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(crate::public_big_ops::clone_number, super::special::allocating_clone)]
#[kani::stub(
    crate::public_big_ops::ordering::from_biguint,
    super::power::signed_from_numeric
)]
#[kani::stub(
    veryl_analyzer::value::ValueBigUint::to_bigint,
    super::signed_small::converted_input
)]
#[kani::stub(<num_bigint::BigUint as std::convert::From<u32>>::from, super::special::from32)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Sub<&num_bigint::BigUint>>::sub, super::special::subtract)]
#[kani::stub(<num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, super::special::rem_owned)]
#[kani::stub(
    crate::public_big_ops::cached_mask,
    super::multiplication::product_mask
)]
#[kani::stub(veryl_analyzer::value::ValueU64::power, reject_native)]
#[kani::stub(num_bigint::BigUint::modpow, modular_api)]
fn single_hot_power_api_matches_byte_products() {
    init();
    let w = usize::from(kani::any::<u8>() & 7) + 1;
    let k = usize::from(kani::any::<u8>() & 7);
    kani::assume(k < w);
    unsafe {
        WIDTH = w;
        SHIFT = k;
    }
    let base = 1u16 << k;
    let mask = (1u16 << w) - 1;
    let exponent = kani::any::<u8>() & 3;
    let mut expected = 1u16;
    for i in 0..3 {
        if i < exponent {
            expected = (expected * base) & mask;
        }
    }
    let b = from64(u64::from(base));
    let e = from64(u64::from(exponent));
    let m = from64(1u64 << w);
    let out = b.modpow(&e, &m);
    assert!(
        get(&out) == N::from_u128(u128::from(expected))
            && get(&b) == N::from_u128(u128::from(base))
            && get(&e) == N::from_u128(u128::from(exponent))
            && get(&m) == N::from_u128(1u128 << w),
        "single hot power byte API mismatch"
    );
    kani::cover!(
        exponent == 0,
        "single hot power API zero exponent is reachable"
    );
    kani::cover!(k == 0, "single hot power API unit base is reachable");
    kani::cover!(
        w == 8 && k == 7 && exponent == 3,
        "single hot power API clipped product is reachable"
    );
    kani::cover!(
        w == 8 && k == 7 && exponent == 1,
        "single hot power API maximum byte base is reachable"
    );
}
#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(crate::public_big_ops::clone_number, super::special::allocating_clone)]
#[kani::stub(
    crate::public_big_ops::ordering::from_biguint,
    super::power::signed_from_numeric
)]
#[kani::stub(
    veryl_analyzer::value::ValueBigUint::to_bigint,
    super::signed_small::converted_input
)]
#[kani::stub(<num_bigint::BigUint as std::convert::From<u32>>::from, super::special::from32)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Sub<&num_bigint::BigUint>>::sub, super::special::subtract)]
#[kani::stub(<num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, super::special::rem_owned)]
#[kani::stub(
    crate::public_big_ops::cached_mask,
    super::multiplication::product_mask
)]
#[kani::stub(veryl_analyzer::value::ValueU64::power, reject_native)]
#[kani::stub(num_bigint::BigUint::modpow, modular_api)]
fn single_hot_power_api_matches_scalar_coordinates() {
    init();
    let w = usize::from(kani::any::<u8>() & 127) + 1;
    let k = usize::from(kani::any::<u8>() & 127);
    kani::assume(k < w);
    let exponent: u128 = kani::any();
    let p = usize::from(kani::any::<u8>());
    unsafe {
        WIDTH = w;
        SHIFT = k;
    }
    let b = from128(1u128 << k);
    let e = from128(exponent);
    let m = put(N::from_u128(1).checked_shl(w));
    let out = b.modpow(&e, &m);
    let n = get(&out);
    let bit = if p < 128 {
        n.lo >> p & 1 != 0
    } else {
        n.hi >> (p - 128) & 1 != 0
    };
    assert!(
        bit == one_hot_power_bit(k, exponent, w, p)
            && n.hi == 0
            && n.lo & !(u128::MAX >> (128 - w)) == 0
            && get(&b) == N::from_u128(1u128 << k)
            && get(&e) == N::from_u128(exponent)
            && get(&m) == N::from_u128(1).checked_shl(w),
        "single hot power coordinate mismatch"
    );
    kani::cover!(
        w == 1,
        "single hot power coordinates unit width is reachable"
    );
    kani::cover!(
        exponent == u128::MAX && k > 0,
        "single hot power coordinates full exponent is reachable"
    );
    kani::cover!(
        w == 128 && k == 1 && exponent == 127 && p == 127,
        "single hot power coordinates high result bit is reachable"
    );
    kani::cover!(
        exponent >> 127 != 0 && k > 0,
        "single hot power coordinates upper exponent is reachable"
    );
    kani::cover!(
        w == 128 && k == 127 && exponent == 1,
        "single hot power coordinates top base bit is reachable"
    );
    kani::cover!(
        p >= w,
        "single hot power coordinates outside width is reachable"
    );
}
#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(crate::public_big_ops::clone_number, super::special::allocating_clone)]
#[kani::stub(
    crate::public_big_ops::ordering::from_biguint,
    super::power::signed_from_numeric
)]
#[kani::stub(
    veryl_analyzer::value::ValueBigUint::to_bigint,
    super::signed_small::converted_input
)]
#[kani::stub(<num_bigint::BigUint as std::convert::From<u32>>::from, super::special::from32)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Sub<&num_bigint::BigUint>>::sub, super::special::subtract)]
#[kani::stub(<num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, super::special::rem_owned)]
#[kani::stub(
    crate::public_big_ops::cached_mask,
    super::multiplication::product_mask
)]
#[kani::stub(veryl_analyzer::value::ValueU64::power, reject_native)]
#[kani::stub(num_bigint::BigUint::modpow, modular_api)]
fn single_hot_power_api_satisfies_bounded_successor_law() {
    init();
    let w = usize::from(kani::any::<u8>() & 127) + 1;
    let k = usize::from(kani::any::<u8>() & 127);
    kani::assume(k < w);
    let exponent: u128 = kani::any();
    unsafe {
        WIDTH = w;
        SHIFT = k;
    }
    let b = from128(1u128 << k);
    let e = from128(exponent);
    let m = put(N::from_u128(1).checked_shl(w));
    let zero = from64(0);
    let first = b.modpow(&zero, &m);
    let out = b.modpow(&e, &m);
    let current = get(&out);
    assert!(
        get(&first) == N::from_u128(1)
            && current.hi == 0
            && current.lo & !(u128::MAX >> (128 - w)) == 0,
        "single hot power successor mismatch"
    );
    // The base case and every nonoverflowing successor step characterize the
    // bounded mathematical API, using the independently checked Mul identity.
    if exponent != u128::MAX {
        let next_exp = from128(exponent + 1);
        let next = b.modpow(&next_exp, &m);
        assert!(
            get(&next) == N::from_u128(one_hot_product_word(current.lo, 1u128 << k, k, w)),
            "single hot power successor mismatch"
        );
    }
    assert!(
        get(&b) == N::from_u128(1u128 << k) && get(&e) == N::from_u128(exponent),
        "single hot power successor mismatch"
    );
    kani::cover!(k == 0, "single hot power recurrence unit base is reachable");
    kani::cover!(
        w == 1,
        "single hot power recurrence unit width is reachable"
    );
    kani::cover!(
        exponent == u128::MAX,
        "single hot power recurrence full exponent is reachable"
    );
    kani::cover!(
        w == 128 && k == 1 && exponent == 127,
        "single hot power recurrence high result bit is reachable"
    );
    kani::cover!(
        w == 128 && k == 1 && exponent == 128,
        "single hot power recurrence first zero result is reachable"
    );
    kani::cover!(
        w == 128 && k == 127 && exponent == 1,
        "single hot power recurrence top base bit is reachable"
    );
}
#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(crate::public_big_ops::clone_number, super::special::allocating_clone)]
#[kani::stub(
    crate::public_big_ops::ordering::from_biguint,
    super::power::signed_from_numeric
)]
#[kani::stub(
    veryl_analyzer::value::ValueBigUint::to_bigint,
    super::signed_small::converted_input
)]
#[kani::stub(<num_bigint::BigUint as std::convert::From<u32>>::from, super::special::from32)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Sub<&num_bigint::BigUint>>::sub, super::special::subtract)]
#[kani::stub(<num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, super::special::rem_owned)]
#[kani::stub(
    crate::public_big_ops::cached_mask,
    super::multiplication::product_mask
)]
#[kani::stub(veryl_analyzer::value::ValueU64::power, reject_native)]
#[kani::stub(num_bigint::BigUint::modpow, observed_modular)]
fn single_hot_power_original_domain_preserves_prepared_base() {
    let (x, a, y, b, w, caller, k, result_sign) = inputs::<true>();
    configure(w, k, b.planes.0, &x, caller);
    let xp = observed_expand(&x, w, caller);
    let ap = expanded_planes(a.planes, a.width, w, a.signed, caller).0;
    preserve(
        xp.as_ref(),
        Operand {
            planes: (ap, 0),
            width: w,
            signed: result_sign,
        },
    );
    let p = usize::from(kani::any::<u8>() & 127);
    result(xp.as_ref(), w, result_sign, p, read_logic(ap, 0, p));
    assert!(
        unsafe { EXPANDS == 1 } && ap == 1u128 << k,
        "single hot power original domain mismatch"
    );
    preserve(&x, a);
    preserve(&y, b);
    kani::cover!(
        a.width == 64 && b.width == 64 && w == 128,
        "single hot power domain native native is reachable"
    );
    kani::cover!(
        a.width == 64 && b.width == 65 && w == 128,
        "single hot power domain native big is reachable"
    );
    kani::cover!(
        a.width == 65 && b.width == 64 && w == 128,
        "single hot power domain big native is reachable"
    );
    kani::cover!(
        a.width == 65 && b.width == 127 && w == 128,
        "single hot power domain big big is reachable"
    );
    kani::cover!(
        k == w - 1 && result_sign,
        "single hot power domain signed minimum is reachable"
    );
    kani::cover!(
        w == 65 && b.width == 128 && b.planes.0 >> 127 != 0,
        "single hot power domain exponent exceeds result width is reachable"
    );
    kani::cover!(
        a.width == w && a.signed && !caller,
        "single hot power domain equal width retains sign is reachable"
    );
    kani::cover!(k == 0, "single hot power domain unit base is reachable");
    kani::cover!(
        b.width == 128 && b.signed,
        "single hot power domain wide signed exponent is reachable"
    );
    kani::cover!(
        a.width == 64 && k == 63 && a.signed && !caller && w == 128,
        "single hot power domain source sign bit zero extends is reachable"
    );
    kani::cover!(
        a.width == 64 && k == 62 && a.signed && caller && w == 128,
        "single hot power domain narrow signed positive is reachable"
    );
}
#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(crate::public_big_ops::clone_number, super::special::allocating_clone)]
#[kani::stub(
    crate::public_big_ops::ordering::from_biguint,
    super::power::signed_from_numeric
)]
#[kani::stub(
    veryl_analyzer::value::ValueBigUint::to_bigint,
    super::signed_small::converted_input
)]
#[kani::stub(<num_bigint::BigUint as std::convert::From<u32>>::from, super::special::from32)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Sub<&num_bigint::BigUint>>::sub, super::special::subtract)]
#[kani::stub(<num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, super::special::rem_owned)]
#[kani::stub(
    crate::public_big_ops::cached_mask,
    super::multiplication::product_mask
)]
#[kani::stub(veryl_analyzer::value::ValueU64::power, reject_native)]
#[kani::stub(num_bigint::BigUint::modpow, observed_modular)]
#[kani::stub(veryl_analyzer::value::Value::expand, observed_expand)]
fn wide_single_hot_power_prepared_signed_matches_reference() {
    check::<false, true>();
}
#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(crate::public_big_ops::clone_number, super::special::allocating_clone)]
#[kani::stub(
    crate::public_big_ops::ordering::from_biguint,
    super::power::signed_from_numeric
)]
#[kani::stub(
    veryl_analyzer::value::ValueBigUint::to_bigint,
    super::signed_small::converted_input
)]
#[kani::stub(<num_bigint::BigUint as std::convert::From<u32>>::from, super::special::from32)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Sub<&num_bigint::BigUint>>::sub, super::special::subtract)]
#[kani::stub(<num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, super::special::rem_owned)]
#[kani::stub(
    crate::public_big_ops::cached_mask,
    super::multiplication::product_mask
)]
#[kani::stub(veryl_analyzer::value::ValueU64::power, reject_native)]
#[kani::stub(num_bigint::BigUint::modpow, observed_modular)]
#[kani::stub(veryl_analyzer::value::Value::expand, observed_expand)]
fn wide_single_hot_power_prepared_unsigned_matches_reference() {
    check::<false, false>();
}
#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(crate::public_big_ops::clone_number, super::special::allocating_clone)]
#[kani::stub(
    crate::public_big_ops::ordering::from_biguint,
    super::power::signed_from_numeric
)]
#[kani::stub(
    veryl_analyzer::value::ValueBigUint::to_bigint,
    super::signed_small::converted_input
)]
#[kani::stub(<num_bigint::BigUint as std::convert::From<u32>>::from, super::special::from32)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Sub<&num_bigint::BigUint>>::sub, super::special::subtract)]
#[kani::stub(<num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, super::special::rem_owned)]
#[kani::stub(
    crate::public_big_ops::cached_mask,
    super::multiplication::product_mask
)]
#[kani::stub(veryl_analyzer::value::ValueU64::power, reject_native)]
#[kani::stub(num_bigint::BigUint::modpow, observed_modular)]
#[kani::stub(veryl_analyzer::value::Value::expand, observed_expand)]
fn wide_single_hot_power_context_signed_matches_reference() {
    check::<true, true>();
}
#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(crate::public_big_ops::clone_number, super::special::allocating_clone)]
#[kani::stub(
    crate::public_big_ops::ordering::from_biguint,
    super::power::signed_from_numeric
)]
#[kani::stub(
    veryl_analyzer::value::ValueBigUint::to_bigint,
    super::signed_small::converted_input
)]
#[kani::stub(<num_bigint::BigUint as std::convert::From<u32>>::from, super::special::from32)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Sub<&num_bigint::BigUint>>::sub, super::special::subtract)]
#[kani::stub(<num_bigint::BigUint as std::ops::Rem<&num_bigint::BigUint>>::rem, super::special::rem_owned)]
#[kani::stub(
    crate::public_big_ops::cached_mask,
    super::multiplication::product_mask
)]
#[kani::stub(veryl_analyzer::value::ValueU64::power, reject_native)]
#[kani::stub(num_bigint::BigUint::modpow, observed_modular)]
#[kani::stub(veryl_analyzer::value::Value::expand, observed_expand)]
fn wide_single_hot_power_context_unsigned_matches_reference() {
    check::<true, false>();
}

// No membership assumption is made here. Quantifying every normalized word
// checks both inclusion and exclusion, including words with multiple set bits.
#[kani::proof]
#[kani::unwind(3)]
fn single_hot_expansion_membership_is_exact() {
    let source = usize::from(kani::any::<u8>() & 127) + 1;
    let target = usize::from(kani::any::<u8>() & 127) + 1;
    let k = usize::from(kani::any::<u8>() & 127);
    kani::assume(source <= target && k < target);
    let word = kani::any::<u128>() & (u128::MAX >> (128 - source));
    let source_signed: bool = kani::any();
    let caller: bool = kani::any();
    let expanded = expanded_planes((word, 0), source, target, source_signed, caller);
    let member = single_hot_expansion_member(word, source, target, source_signed, caller, k);
    assert!(
        member == (expanded.0 == 1u128 << k) && expanded.1 == 0,
        "single hot expansion membership mismatch"
    );
    kani::cover!(
        target == 1 && member,
        "single hot membership unit width is reachable"
    );
    kani::cover!(
        source == target && source_signed && !caller && member,
        "single hot membership equal width is reachable"
    );
    kani::cover!(
        source == 64
            && target == 128
            && k == 63
            && word == 1u128 << k
            && source_signed
            && caller
            && !member,
        "single hot membership sign extension rejects is reachable"
    );
    kani::cover!(
        source == 64 && target == 128 && k == 63 && source_signed && !caller && member,
        "single hot membership zero extension accepts is reachable"
    );
    kani::cover!(
        source == 128 && target == 128 && k == 127 && member,
        "single hot membership upper bit is reachable"
    );
    kani::cover!(
        source == 64 && target == 128 && k == 127 && !member,
        "single hot membership missing source bit is reachable"
    );
    kani::cover!(
        word == 3 && !member,
        "single hot membership multiple bits rejects is reachable"
    );
}
