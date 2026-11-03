//! IEEE1800-2023 11.4.3/Table11-4,11.6.1/11.8.2: all-ones-base positive powers.
//! Bases65..128 after nonnarrowing source widths1..128; independent source
//! flags/caller mode. Exponents keep original widths1..128/signs; known positive
//! values only. Unsigned upper-bit exponents and signed positive exponents included.
//! The actual Op, pow_mod_width and result constructors execute. Signed bases
//! use the separately proved value-preserving conversion summary at magnitude1,
//! checked against the actual all-ones prepared word before conversion.
//! The modular API is specialized to residues1 and modulus-1. Byte multiplication
//! and a zero/successor law under the checked all-ones multiplication identity
//! validate its bounded representation, not the num-bigint implementation.
//! Modulus2^128 retains its high bit. The actual numeric modpow inputs are checked
//! against the full self-determined exponent and prepared base interpretation.
//! Negative/zero exponent compiler branches and X/Z retain their separate proofs.
use super::*;
use num_bigint::BigUint;
use veryl_formal_semantics::{
    nonlinear::{all_ones_power_word, all_ones_product_word},
    resize::expanded_planes,
};

static mut WIDTH: usize = 0;
static mut EXPONENT: u128 = 0;
static mut SIGNED_BASE: bool = false;
static mut MOD_CALLS: usize = 0;
static mut SOURCE: *const Value = std::ptr::null();
static mut CALLER: bool = false;
static mut EXPANDS: usize = 0;
fn configure(width: usize, exponent: u128, base_signed: bool, x: &Value, caller: bool) {
    unsafe {
        WIDTH = width;
        EXPONENT = exponent;
        SIGNED_BASE = base_signed;
        MOD_CALLS = 0;
        SOURCE = x;
        CALLER = caller;
        EXPANDS = 0;
    }
    opposite::configure_magnitude(1, width);
    multiplication::reset_product_mask();
}
fn modular_api(base: &BigUint, exponent: &BigUint, modulus: &BigUint) -> BigUint {
    let width = unsafe { WIDTH };
    assert!(
        (1..=128).contains(&width),
        "power ones modular domain exceeded"
    );
    let b = get(base);
    let e = get(exponent);
    let m = get(modulus);
    assert!(
        m == N::from_u128(1).checked_shl(width)
            && e.hi == 0
            && (b == N::from_u128(1) || b == N::mask(width)),
        "power ones modular domain exceeded"
    );
    put(if b == N::from_u128(1) || e.lo & 1 == 0 {
        N::from_u128(1)
    } else {
        N::mask(width)
    })
}
fn observed_modular(base: &BigUint, exponent: &BigUint, modulus: &BigUint) -> BigUint {
    unsafe {
        assert!(
            MOD_CALLS == 0
                && get(exponent) == N::from_u128(EXPONENT)
                && get(modulus) == N::from_u128(1).checked_shl(WIDTH)
                && get(base)
                    == if SIGNED_BASE {
                        N::from_u128(1)
                    } else {
                        N::mask(WIDTH)
                    },
            "wide operator semantics mismatch"
        );
        MOD_CALLS += 1;
    }
    modular_api(base, exponent, modulus)
}
fn observed_expand(value: &Value, width: usize, signed: bool) -> std::borrow::Cow<'_, Value> {
    unsafe {
        assert!(
            EXPANDS == 0 && std::ptr::eq(value, SOURCE) && width == WIDTH && signed == CALLER,
            "wide operator semantics mismatch"
        );
        EXPANDS += 1;
    }
    let source = value.width();
    assert!(
        (1..=128).contains(&source)
            && (65..=128).contains(&width)
            && source <= width
            && value.payload_u128() == u128::MAX >> (128 - source)
            && value.mask_xz_u128() == 0
            && (source == width || (value.signed() && signed)),
        "power ones expansion domain exceeded"
    );
    // The domain harness compares this specialization against expanded_planes.
    // Equal-width borrowing preserves the source flag, including unsigned callers.
    if source == width {
        let _ = from128(0);
        let _ = from128(0);
        std::borrow::Cow::Borrowed(value)
    } else {
        std::borrow::Cow::Owned(Value::BigUint(ValueBigUint {
            payload: Box::new(from128(u128::MAX >> (128 - width))),
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
    panic!("power ones native arithmetic boundary reached")
}
fn inputs<const ORIGINAL: bool>() -> (Value, Operand, Value, Operand, usize, bool, u128, bool) {
    init();
    let width = usize::from(kani::any::<u8>() & 63) + 65;
    let aw = if ORIGINAL {
        usize::from(kani::any::<u8>() & 127) + 1
    } else {
        width
    };
    let bw = usize::from(kani::any::<u8>() & 127) + 1;
    kani::assume(aw <= width);
    let caller: bool = kani::any();
    let (x, a) = identities::known_input(aw, Some(u128::MAX));
    let (y, b) = identities::known_input(bw, None);
    let ap = expanded_planes(a.planes, aw, width, a.signed, caller).0;
    kani::assume(ap == u128::MAX >> (128 - width));
    kani::assume(b.planes.0 != 0 && !(b.signed && b.planes.0 >> (bw - 1) & 1 != 0));
    let result_sign = a.signed && (aw == width || caller);
    // The base is moved after this generator returns; set its observed pointer
    // in the caller, after its final stack location is established.
    (x, a, y, b, width, caller, ap, result_sign)
}
fn check<const ORIGINAL: bool>() {
    let (x, a, y, b, width, caller, ap, result_sign) = inputs::<ORIGINAL>();
    configure(width, b.planes.0, result_sign, &x, caller);
    let out = Op::Pow.eval_value_binary(&x, &y, width, caller, &mut MaskCache::default());
    assert!(
        unsafe { MOD_CALLS == 1 } && (!ORIGINAL || unsafe { EXPANDS == 1 }),
        "wide operator semantics mismatch"
    );
    let expected = all_ones_power_word(ap, b.planes.0, width, b.width, b.signed);
    preserve(
        &out,
        Operand {
            planes: (expected, 0),
            width,
            signed: result_sign,
        },
    );
    let pos = usize::from(kani::any::<u8>() & 127);
    result(&out, width, result_sign, pos, read_logic(expected, 0, pos));
    preserve(&x, a);
    preserve(&y, b);
    if ORIGINAL {
        kani::cover!(
            width == 65 && b.planes.0 == 3 && result_sign,
            "ones power class original context is reachable"
        );
    } else {
        kani::cover!(
            width == 65 && result_sign,
            "ones power class minimum signed base is reachable"
        );
        kani::cover!(
            width == 128 && !result_sign,
            "ones power class maximum unsigned base is reachable"
        );
        kani::cover!(
            b.planes.0 > 1 && b.planes.0 & 1 == 0,
            "ones power class even exponent is reachable"
        );
        kani::cover!(
            b.planes.0 > 1 && b.planes.0 & 1 != 0,
            "ones power class odd exponent is reachable"
        );
        kani::cover!(
            b.width == 128 && b.planes.0 == u128::MAX,
            "ones power class full unsigned exponent is reachable"
        );
        kani::cover!(
            b.width == 128 && b.signed,
            "ones power class wide signed exponent is reachable"
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
    super::opposite::witnessed_conversion
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
fn all_ones_power_api_matches_byte_products() {
    init();
    let width = usize::from(kani::any::<u8>() & 7) + 1;
    unsafe {
        WIDTH = width;
    }
    let mask = (1u16 << width) - 1;
    let unit: bool = kani::any();
    let base = if unit { 1 } else { mask };
    let exponent = kani::any::<u8>() & 3;
    let mut expected = 1u16;
    for i in 0..3 {
        if i < exponent {
            expected = (expected * base) & mask;
        }
    }
    let b = from64(u64::from(base));
    let e = from64(u64::from(exponent));
    let m = from64(1u64 << width);
    let out = b.modpow(&e, &m);
    assert!(
        get(&out) == N::from_u128(u128::from(expected))
            && get(&b) == N::from_u128(u128::from(base))
            && get(&e) == N::from_u128(u128::from(exponent))
            && get(&m) == N::from_u128(1u128 << width),
        "ones power byte API mismatch"
    );
    kani::cover!(exponent == 0, "ones power API zero exponent is reachable");
    kani::cover!(
        exponent == 2 && !unit,
        "ones power API even exponent is reachable"
    );
    kani::cover!(
        exponent == 3 && !unit,
        "ones power API odd exponent is reachable"
    );
    kani::cover!(
        width == 8 && !unit,
        "ones power API maximum byte base is reachable"
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
    super::opposite::witnessed_conversion
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
fn all_ones_power_api_satisfies_bounded_successor_law() {
    init();
    let width = usize::from(kani::any::<u8>() & 127) + 1;
    unsafe {
        WIDTH = width;
    }
    let unit: bool = kani::any();
    let exponent: u128 = kani::any();
    let mask = u128::MAX >> (128 - width);
    let base = if unit { 1 } else { mask };
    let b = from128(base);
    let e = from128(exponent);
    let m = put(N::from_u128(1).checked_shl(width));
    let zero = from64(0);
    let initial = b.modpow(&zero, &m);
    let out = b.modpow(&e, &m);
    assert!(
        get(&initial) == N::from_u128(1),
        "ones power successor mismatch"
    );
    let current = get(&out);
    assert!(
        current.hi == 0 && current.lo == if unit || exponent & 1 == 0 { 1 } else { mask },
        "ones power successor mismatch"
    );
    // Base0 and all bounded successor steps characterize powers up to u128::MAX.
    // The multiplication identity is checked independently in ones.rs.
    if exponent != u128::MAX {
        let next_exp = from128(exponent + 1);
        let next = b.modpow(&next_exp, &m);
        let product = if unit {
            current.lo
        } else {
            all_ones_product_word(current.lo, mask, width)
        };
        assert!(
            get(&next) == N::from_u128(product),
            "ones power successor mismatch"
        );
    }
    let delta = &m - &out;
    let reduced = delta % &m;
    let complement = if current.lo == 1 { mask } else { 1 };
    assert!(
        get(&reduced) == N::from_u128(complement)
            && get(&reduced).checked_add(current) == get(&m)
            && get(&b) == N::from_u128(base)
            && get(&e) == N::from_u128(exponent),
        "ones power successor mismatch"
    );
    kani::cover!(width == 1, "ones power recurrence unit width is reachable");
    kani::cover!(
        width == 128 && exponent == u128::MAX,
        "ones power recurrence full exponent is reachable"
    );
    kani::cover!(exponent == 0, "ones power recurrence zero is reachable");
    kani::cover!(
        exponent > 0 && exponent & 1 == 0,
        "ones power recurrence even is reachable"
    );
    kani::cover!(
        exponent > 1 && exponent & 1 != 0,
        "ones power recurrence odd is reachable"
    );
    kani::cover!(
        width == 65 && exponent >> 127 != 0,
        "ones power recurrence upper exponent is reachable"
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
    super::opposite::witnessed_conversion
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
fn all_ones_power_original_domain_preserves_prepared_base() {
    let (x, a, y, b, width, caller, ap, result_sign) = inputs::<true>();
    configure(width, b.planes.0, result_sign, &x, caller);
    let xp = observed_expand(&x, width, caller);
    preserve(
        xp.as_ref(),
        Operand {
            planes: (ap, 0),
            width,
            signed: result_sign,
        },
    );
    let pos = usize::from(kani::any::<u8>() & 127);
    result(xp.as_ref(), width, result_sign, pos, read_logic(ap, 0, pos));
    assert!(
        unsafe { EXPANDS == 1 } && ap == u128::MAX >> (128 - width),
        "ones power original domain mismatch"
    );
    preserve(&x, a);
    preserve(&y, b);
    kani::cover!(
        a.width == 64 && b.width == 64 && width == 128,
        "ones power domain native native is reachable"
    );
    kani::cover!(
        a.width == 64 && b.width == 65 && width == 128,
        "ones power domain native big is reachable"
    );
    kani::cover!(
        a.width == 65 && b.width == 64 && width == 128,
        "ones power domain big native is reachable"
    );
    kani::cover!(
        a.width == 65 && b.width == 127 && width == 128,
        "ones power domain big big is reachable"
    );
    kani::cover!(result_sign, "ones power domain signed base is reachable");
    kani::cover!(!result_sign, "ones power domain unsigned base is reachable");
    kani::cover!(
        width == 65 && b.width == 128 && b.planes.0 >> 127 != 0,
        "ones power domain exponent exceeds result width is reachable"
    );
    kani::cover!(
        b.width == 128 && b.signed,
        "ones power domain wide signed exponent is reachable"
    );
    kani::cover!(
        a.width == width && a.signed && !caller,
        "ones power domain equal width retains sign is reachable"
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
    super::opposite::witnessed_conversion
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
#[kani::stub(veryl_analyzer::value::Value::expand, super::prepared_wide)]
fn wide_all_ones_power_prepared_matches_reference() {
    check::<false>();
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
    super::opposite::witnessed_conversion
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
fn wide_all_ones_power_context_matches_reference() {
    check::<true>();
}
