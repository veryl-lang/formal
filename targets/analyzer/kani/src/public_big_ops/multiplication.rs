//! IEEE1800-2023 11.4.3/11.8.2: known multiplication with a single-hot
//! prepared factor on either side. Results65..128; original widths1..128,
//! nonnarrowing contexts, independent source signs and either caller mode.
//! Membership is checked after sizing. A top signed bit denotes a negative
//! factor; the modular bit-pattern identity still holds. No X/Z or general Mul.
//! Actual Op and result constructors execute under guarded mathematical numeric
//! APIs, not a proof of num-bigint algorithms. Full products retain256 bits so
//! missing result truncation remains observable. Expansion composes its proven
//! summary; byte arithmetic and full product coordinates check the new adapter.
use super::*;
use num_bigint::BigUint;
use veryl_formal_semantics::{
    nonlinear::{one_hot_product_bit, one_hot_product_word},
    resize::expanded_planes,
};

// The harness supplies a factor-position witness. Every call checks it against
// the actual numeric operand; it is not assumed to match a production value.
// This avoids rediscovering the same single-hot position through trailing_zeros.
static mut FACTOR_SHIFT: usize = 0;
static mut FACTOR_LEFT: bool = false;
fn configure_factor(shift: usize, left: bool) {
    unsafe {
        MASK_CALLS = 0;
        FACTOR_SHIFT = shift;
        FACTOR_LEFT = left;
    }
}
fn multiply<'a: 'a, 'b: 'b>(a: &'a BigUint, b: &'b BigUint) -> BigUint {
    let x = get(a);
    let y = get(b);
    let (shift, left) = unsafe { (FACTOR_SHIFT, FACTOR_LEFT) };
    let (number, factor) = if left { (y, x) } else { (x, y) };
    assert!(
        shift < 128 && number.hi == 0 && factor == N::from_u128(1u128 << shift),
        "multiplication numeric class domain exceeded"
    );
    put(number.checked_shl(shift))
}
// Mul retrieves its result mask once. Guard that call bound explicitly and
// avoid a symbolic index into the generic129-entry abstract cache. Real cache
// storage and histories remain covered by the existing executable contracts.
static mut PRODUCT_MASK: BigUint = BigUint::ZERO;
static mut MASK_CALLS: usize = 0;
pub(super) fn reset_product_mask() {
    unsafe {
        MASK_CALLS = 0;
    }
}
pub(super) fn product_mask(_: &mut MaskCache, width: usize) -> &BigUint {
    unsafe {
        assert!(
            MASK_CALLS == 0 && (65..=128).contains(&width),
            "multiplication cache summary domain exceeded"
        );
        MASK_CALLS += 1;
        PRODUCT_MASK = mask(width);
        &*std::ptr::addr_of!(PRODUCT_MASK)
    }
}
pub(super) fn reject_native(
    _: &veryl_analyzer::value::ValueU64,
    _: &veryl_analyzer::value::ValueU64,
    _: usize,
    _: bool,
) -> veryl_analyzer::value::ValueU64 {
    panic!("multiplication native boundary reached")
}
fn product_coordinate(n: N, position: usize) -> bool {
    [n.lo, n.hi][position / 128] >> (position % 128) & 1 != 0
}
fn check<const LEFT: bool, const ORIGINAL: bool>() {
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
    let shift = usize::from(kani::any::<u8>() & 127);
    let signed: bool = kani::any();
    let (x, a) = identities::known_input(aw, if LEFT { Some(1u128 << shift) } else { None });
    let (y, b) = identities::known_input(bw, if LEFT { None } else { Some(1u128 << shift) });
    let ap = expanded_planes(a.planes, aw, width, a.signed, signed).0;
    let bp = expanded_planes(b.planes, bw, width, b.signed, signed).0;
    let (number, factor) = if LEFT { (bp, ap) } else { (ap, bp) };
    kani::assume(factor == 1u128 << shift);
    configure_factor(shift, LEFT);
    special::observe_binary_sources(&x, &y, width, signed);
    let output = Op::Mul.eval_value_binary(&x, &y, width, signed, &mut MaskCache::default());
    if ORIGINAL {
        assert!(
            special::observed_calls() == 2,
            "wide operator semantics mismatch"
        );
    }
    let position = usize::from(kani::any::<u8>() & 127);
    let expected = one_hot_product_word(number, factor, shift, width);
    preserve(
        &output,
        Operand {
            planes: (expected, 0),
            width,
            signed,
        },
    );
    assert!(
        output.mask_xz_u128() == 0,
        "wide operator semantics mismatch"
    );
    result(
        &output,
        width,
        signed,
        position,
        read_logic(expected, 0, position),
    );
    preserve(&x, a);
    preserve(&y, b);
    kani::cover!(
        width == 65 && signed,
        "multiplication class minimum signed width is reachable"
    );
    kani::cover!(
        width == 128 && !signed,
        "multiplication class maximum unsigned width is reachable"
    );
    kani::cover!(
        a.signed && !b.signed,
        "multiplication class mixed source signs is reachable"
    );
    kani::cover!(shift == 0, "multiplication class factor one is reachable");
    kani::cover!(
        shift == 64 && number >> 64 != 0,
        "multiplication class word crossing is reachable"
    );
    kani::cover!(
        shift == 127 && number == 2 && signed,
        "multiplication class truncated high product is reachable"
    );
    if ORIGINAL {
        kani::cover!(
            aw == 64 && bw == 64 && width == 128,
            "multiplication class native native sources is reachable"
        );
        kani::cover!(
            aw == 64 && bw == 65 && width == 128,
            "multiplication class native big sources is reachable"
        );
        kani::cover!(
            aw == 65 && bw == 64 && width == 128,
            "multiplication class big native sources is reachable"
        );
        kani::cover!(
            aw == 65 && bw == 127 && width == 128,
            "multiplication class big big sources is reachable"
        );
    }
}
#[kani::proof]
#[kani::unwind(129)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Mul<&num_bigint::BigUint>>::mul, multiply)]
#[kani::stub(crate::public_big_ops::cached_mask, product_mask)]
fn numeric_one_hot_multiplication_matches_byte_arithmetic() {
    init();
    let number = kani::any::<u8>();
    let shift = kani::any::<u8>() & 7;
    let factor = 1u8 << shift;
    let x = from64(u64::from(number));
    let y = from64(u64::from(factor));
    configure_factor(shift as usize, false);
    let left = &x * &y;
    configure_factor(shift as usize, true);
    let right = &y * &x;
    let expected = N::from_u128(u128::from(u16::from(number) * u16::from(factor)));
    assert!(
        get(&left) == expected
            && get(&right) == expected
            && get(&x) == N::from_u128(u128::from(number))
            && get(&y) == N::from_u128(u128::from(factor)),
        "multiplication numeric API mismatch"
    );
    kani::cover!(number == 0, "multiplication API zero number is reachable");
    kani::cover!(
        number == 255 && shift == 7,
        "multiplication API byte high product is reachable"
    );
}
#[kani::proof]
#[kani::unwind(129)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Mul<&num_bigint::BigUint>>::mul, multiply)]
#[kani::stub(crate::public_big_ops::cached_mask, product_mask)]
fn numeric_one_hot_multiplication_matches_full_product_coordinates() {
    init();
    let number: u128 = kani::any();
    let shift = usize::from(kani::any::<u8>() & 127);
    let position = usize::from(kani::any::<u8>());
    let factor = 1u128 << shift;
    let x = from128(number);
    let y = from128(factor);
    configure_factor(shift as usize, false);
    let left = &x * &y;
    configure_factor(shift as usize, true);
    let right = &y * &x;
    let expected =
        position >= shift && position - shift < 128 && number >> (position - shift) & 1 != 0;
    assert!(
        product_coordinate(get(&left), position) == expected
            && product_coordinate(get(&right), position) == expected
            && get(&x) == N::from_u128(number)
            && get(&y) == N::from_u128(factor),
        "multiplication numeric API mismatch"
    );
    // Prove the word summary against scalar coordinates outside the compiler
    // body, while retaining every number and shift (width128 always admits it).
    let width = usize::from(kani::any::<u8>() & 63) + 65;
    kani::assume(shift < width);
    let mask = u128::MAX >> (128 - width);
    let coordinate = usize::from(kani::any::<u8>() & 127);
    let word = one_hot_product_word(number & mask, factor, shift, width);
    let scalar = one_hot_product_bit(number & mask, factor, shift, width, coordinate);
    assert!(
        (word >> coordinate & 1 != 0) == scalar,
        "multiplication numeric API mismatch"
    );
    assert!(
        get(&left).lo & mask == word && get(&right).lo & mask == word,
        "multiplication numeric API mismatch"
    );
    let mut cache = MaskCache::default();
    let actual_mask = cache.get(width);
    assert!(
        get(actual_mask) == N::from_u128(mask) && unsafe { MASK_CALLS == 1 },
        "multiplication numeric API mismatch"
    );
    kani::cover!(
        shift == 0 && width == 65,
        "multiplication API factor one is reachable"
    );
    kani::cover!(
        shift == 64 && number >> 64 != 0,
        "multiplication API word crossing is reachable"
    );
    kani::cover!(
        shift == 127 && number >> 127 != 0 && position == 254,
        "multiplication API bit254 is reachable"
    );
}
#[kani::proof]
#[kani::unwind(129)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Mul<&num_bigint::BigUint>>::mul, multiply)]
#[kani::stub(crate::public_big_ops::cached_mask, product_mask)]
#[kani::stub(veryl_analyzer::value::ValueU64::multiply, reject_native)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::prepared_wide)]
fn wide_one_hot_multiply_right_matches_reference() {
    check::<false, false>();
}
#[kani::proof]
#[kani::unwind(129)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Mul<&num_bigint::BigUint>>::mul, multiply)]
#[kani::stub(crate::public_big_ops::cached_mask, product_mask)]
#[kani::stub(veryl_analyzer::value::ValueU64::multiply, reject_native)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::special::observed_expand)]
fn wide_one_hot_multiply_right_context_matches_reference() {
    check::<false, true>();
}
#[kani::proof]
#[kani::unwind(129)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Mul<&num_bigint::BigUint>>::mul, multiply)]
#[kani::stub(crate::public_big_ops::cached_mask, product_mask)]
#[kani::stub(veryl_analyzer::value::ValueU64::multiply, reject_native)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::prepared_wide)]
fn wide_one_hot_multiply_left_matches_reference() {
    check::<true, false>();
}
#[kani::proof]
#[kani::unwind(129)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Mul<&num_bigint::BigUint>>::mul, multiply)]
#[kani::stub(crate::public_big_ops::cached_mask, product_mask)]
#[kani::stub(veryl_analyzer::value::ValueU64::multiply, reject_native)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::special::observed_expand)]
fn wide_one_hot_multiply_left_context_matches_reference() {
    check::<true, true>();
}
