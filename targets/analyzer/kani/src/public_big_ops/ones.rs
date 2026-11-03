//! IEEE1800-2023 11.4.3/11.6.1/11.8.2: known all-ones-factor multiplication.
//! Prepared widths65..128; independent nonnarrowing sources1..128 and flags.
//! Both caller modes and both factor orientations are included. Membership is
//! checked after sizing; an all-ones factor denotes -1 only in signed mode,
//! while both modes produce the same modular negation of the other raw word.
//! Actual Op and unsigned result constructors execute under guarded mathematical
//! BigUint APIs, not a proof of num-bigint algorithms. Full256-bit intermediates
//! retain the exact product n*(2^w-1)=(n<<w)-n before result-width truncation.
//! Byte arithmetic and a nonoverflowing full-product inverse law validate the
//! adapter; independent scalar subtraction carry checks the modular result.
//! The one-retrieval mask summary and unreachable native-path guard are shared
//! with single-hot multiplication. Real cache histories and boundaries stay direct.
use super::*;
use num_bigint::BigUint;
use veryl_formal_semantics::{
    nonlinear::{all_ones_product_word, negated_word_bit},
    resize::expanded_planes,
};

static mut WIDTH: usize = 0;
static mut LEFT: bool = false;
fn configure(width: usize, left: bool) {
    unsafe {
        WIDTH = width;
        LEFT = left;
    }
    multiplication::reset_product_mask();
}
fn full_product(number: N, width: usize) -> N {
    assert!(
        number.hi == 0 && (1..=128).contains(&width),
        "ones numeric product domain exceeded"
    );
    let shifted = number.checked_shl(width);
    let (lo, borrow) = shifted.lo.overflowing_sub(number.lo);
    let (hi, underflow) = shifted.hi.overflowing_sub(u128::from(borrow));
    assert!(!underflow, "ones numeric subtraction domain exceeded");
    N { lo, hi }
}
fn multiply<'a: 'a, 'b: 'b>(a: &'a BigUint, b: &'b BigUint) -> BigUint {
    let (width, left) = unsafe { (WIDTH, LEFT) };
    assert!(
        (1..=128).contains(&width),
        "ones numeric factor domain exceeded"
    );
    let x = get(a);
    let y = get(b);
    let (number, factor) = if left { (y, x) } else { (x, y) };
    assert!(
        factor == N::mask(width),
        "ones numeric factor domain exceeded"
    );
    put(full_product(number, width))
}
fn inputs<const LEFT_FACTOR: bool, const ORIGINAL: bool>()
-> (Value, Operand, Value, Operand, usize, bool, u128, u128) {
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
    let signed: bool = kani::any();
    let (x, a) = identities::known_input(aw, if LEFT_FACTOR { Some(u128::MAX) } else { None });
    let (y, b) = identities::known_input(bw, if LEFT_FACTOR { None } else { Some(u128::MAX) });
    let ap = expanded_planes(a.planes, aw, width, a.signed, signed).0;
    let bp = expanded_planes(b.planes, bw, width, b.signed, signed).0;
    let factor = if LEFT_FACTOR { ap } else { bp };
    kani::assume(factor == u128::MAX >> (128 - width));
    configure(width, LEFT_FACTOR);
    (x, a, y, b, width, signed, ap, bp)
}
fn check<const LEFT_FACTOR: bool, const ORIGINAL: bool>() {
    let (x, a, y, b, width, signed, ap, bp) = inputs::<LEFT_FACTOR, ORIGINAL>();
    special::observe_binary_sources(&x, &y, width, signed);
    let out = Op::Mul.eval_value_binary(&x, &y, width, signed, &mut MaskCache::default());
    if ORIGINAL {
        assert!(
            special::observed_calls() == 2,
            "wide operator semantics mismatch"
        );
    }
    let (number, factor) = if LEFT_FACTOR { (bp, ap) } else { (ap, bp) };
    let expected = all_ones_product_word(number, factor, width);
    preserve(
        &out,
        Operand {
            planes: (expected, 0),
            width,
            signed,
        },
    );
    let pos = usize::from(kani::any::<u8>() & 127);
    result(&out, width, signed, pos, read_logic(expected, 0, pos));
    preserve(&x, a);
    preserve(&y, b);
    if ORIGINAL {
        kani::cover!(
            width == 65 && number == 1 && signed,
            "ones class original context is reachable"
        );
    } else {
        kani::cover!(
            width == 65 && signed,
            "ones class minimum signed width is reachable"
        );
        kani::cover!(
            width == 128 && !signed,
            "ones class maximum unsigned width is reachable"
        );
        kani::cover!(number == 0, "ones class zero number is reachable");
        kani::cover!(
            width == 128 && number == 2,
            "ones class truncated high product is reachable"
        );
        kani::cover!(
            width == 128 && number == 1u128 << 127 && signed,
            "ones class signed minimum number is reachable"
        );
    }
}
#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Mul<&num_bigint::BigUint>>::mul, multiply)]
#[kani::stub(
    crate::public_big_ops::cached_mask,
    super::multiplication::product_mask
)]
#[kani::stub(
    veryl_analyzer::value::ValueU64::multiply,
    super::multiplication::reject_native
)]
fn all_ones_numeric_api_matches_byte_products() {
    init();
    let width = usize::from(kani::any::<u8>() & 7) + 1;
    let number = kani::any::<u8>();
    let factor = (1u16 << width) - 1;
    let x = from64(u64::from(number));
    let y = from64(u64::from(factor));
    configure(width, false);
    let a = &x * &y;
    configure(width, true);
    let b = &y * &x;
    let expected = N::from_u128(u128::from(u16::from(number) * factor));
    assert!(
        get(&a) == expected
            && get(&b) == expected
            && get(&x) == N::from_u128(u128::from(number))
            && get(&y) == N::from_u128(u128::from(factor)),
        "ones numeric API mismatch"
    );
    kani::cover!(
        width == 1 && number == 255,
        "ones API unit factor is reachable"
    );
    kani::cover!(
        width == 8 && number == 255,
        "ones API maximum byte product is reachable"
    );
    kani::cover!(number == 0, "ones API zero product is reachable");
}
#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Mul<&num_bigint::BigUint>>::mul, multiply)]
#[kani::stub(
    crate::public_big_ops::cached_mask,
    super::multiplication::product_mask
)]
#[kani::stub(
    veryl_analyzer::value::ValueU64::multiply,
    super::multiplication::reject_native
)]
fn all_ones_full_product_satisfies_nonoverflowing_inverse() {
    init();
    let width = usize::from(kani::any::<u8>() & 127) + 1;
    let number: u128 = kani::any();
    let x = from128(number);
    let y = put(N::mask(width));
    configure(width, false);
    let left = &x * &y;
    configure(width, true);
    let right = &y * &x;
    let n = N::from_u128(number);
    let shifted = n.checked_shl(width);
    assert!(
        get(&left).checked_add(n) == shifted
            && get(&right).checked_add(n) == shifted
            && get(&left) == get(&right)
            && get(&x) == n
            && get(&y) == N::mask(width),
        "ones full product inverse mismatch"
    );
    kani::cover!(width == 1, "ones inverse unit width is reachable");
    kani::cover!(
        width == 128 && number == u128::MAX && get(&left).hi >> 127 != 0,
        "ones inverse bit255 is reachable"
    );
    kani::cover!(number == 0, "ones inverse zero is reachable");
    kani::cover!(
        width == 128 && number == 2,
        "ones inverse high borrow is reachable"
    );
    kani::cover!(
        width == 65 && number == 1u128 << 127,
        "ones inverse cross word is reachable"
    );
}
#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Mul<&num_bigint::BigUint>>::mul, multiply)]
#[kani::stub(
    crate::public_big_ops::cached_mask,
    super::multiplication::product_mask
)]
#[kani::stub(
    veryl_analyzer::value::ValueU64::multiply,
    super::multiplication::reject_native
)]
fn all_ones_modular_product_matches_scalar_subtraction_carry() {
    init();
    let width = usize::from(kani::any::<u8>() & 63) + 65;
    let factor = u128::MAX >> (128 - width);
    let number = kani::any::<u128>() & factor;
    let x = from128(number);
    let y = from128(factor);
    configure(width, false);
    let left = &x * &y;
    configure(width, true);
    let right = &y * &x;
    let mut cache = MaskCache::default();
    let mask = cache.get(width);
    let a = &left & mask;
    let b = &right & mask;
    let expected = all_ones_product_word(number, factor, width);
    let pos = usize::from(kani::any::<u8>() & 127);
    assert!(
        get(&a) == N::from_u128(expected)
            && get(&b) == N::from_u128(expected)
            && (expected >> pos & 1 != 0) == negated_word_bit(number, width, pos)
            && get(mask) == N::from_u128(factor),
        "ones modular product mismatch"
    );
    kani::cover!(width == 65, "ones word minimum width is reachable");
    kani::cover!(width == 128, "ones word maximum width is reachable");
    kani::cover!(number == 0, "ones word zero is reachable");
    kani::cover!(number == 1, "ones word one is reachable");
    kani::cover!(
        width == 128 && number == 2 && get(&left).hi != 0,
        "ones word high truncation is reachable"
    );
    kani::cover!(
        width == 128 && number == factor,
        "ones word all ones square is reachable"
    );
}
fn domain<const LEFT_FACTOR: bool>() {
    let (x, a, y, b, width, signed, ap, bp) = inputs::<LEFT_FACTOR, true>();
    special::observe_binary_sources(&x, &y, width, signed);
    let xp = special::observed_expand(&x, width, signed);
    let yp = special::observed_expand(&y, width, signed);
    let asigned = a.signed && (a.width == width || signed);
    let bsigned = b.signed && (b.width == width || signed);
    preserve(
        xp.as_ref(),
        Operand {
            planes: (ap, 0),
            width,
            signed: asigned,
        },
    );
    preserve(
        yp.as_ref(),
        Operand {
            planes: (bp, 0),
            width,
            signed: bsigned,
        },
    );
    let pos = usize::from(kani::any::<u8>() & 127);
    result(xp.as_ref(), width, asigned, pos, read_logic(ap, 0, pos));
    result(yp.as_ref(), width, bsigned, pos, read_logic(bp, 0, pos));
    assert!(
        special::observed_calls() == 2
            && (if LEFT_FACTOR { ap } else { bp }) == u128::MAX >> (128 - width),
        "ones original domain mismatch"
    );
    preserve(&x, a);
    preserve(&y, b);
    kani::cover!(
        a.width == 64 && b.width == 64 && width == 128,
        "ones domain native native is reachable"
    );
    kani::cover!(
        a.width == 64 && b.width == 65 && width == 128,
        "ones domain native big is reachable"
    );
    kani::cover!(
        a.width == 65 && b.width == 64 && width == 128,
        "ones domain big native is reachable"
    );
    kani::cover!(
        a.width == 65 && b.width == 127 && width == 128,
        "ones domain big big is reachable"
    );
    kani::cover!(
        width == 65 && signed,
        "ones domain minimum signed width is reachable"
    );
    kani::cover!(
        width == 128 && !signed,
        "ones domain maximum unsigned width is reachable"
    );
    kani::cover!(
        a.signed != b.signed,
        "ones domain mixed source signs is reachable"
    );
}
#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Mul<&num_bigint::BigUint>>::mul, multiply)]
#[kani::stub(
    crate::public_big_ops::cached_mask,
    super::multiplication::product_mask
)]
#[kani::stub(
    veryl_analyzer::value::ValueU64::multiply,
    super::multiplication::reject_native
)]
fn all_ones_right_original_domain_preserves_prepared_values() {
    domain::<false>();
}
#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Mul<&num_bigint::BigUint>>::mul, multiply)]
#[kani::stub(
    crate::public_big_ops::cached_mask,
    super::multiplication::product_mask
)]
#[kani::stub(
    veryl_analyzer::value::ValueU64::multiply,
    super::multiplication::reject_native
)]
fn all_ones_left_original_domain_preserves_prepared_values() {
    domain::<true>();
}
#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Mul<&num_bigint::BigUint>>::mul, multiply)]
#[kani::stub(
    crate::public_big_ops::cached_mask,
    super::multiplication::product_mask
)]
#[kani::stub(
    veryl_analyzer::value::ValueU64::multiply,
    super::multiplication::reject_native
)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::prepared_wide)]
fn wide_all_ones_right_matches_reference() {
    check::<false, false>();
}
#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Mul<&num_bigint::BigUint>>::mul, multiply)]
#[kani::stub(
    crate::public_big_ops::cached_mask,
    super::multiplication::product_mask
)]
#[kani::stub(
    veryl_analyzer::value::ValueU64::multiply,
    super::multiplication::reject_native
)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::special::observed_expand)]
fn wide_all_ones_right_context_matches_reference() {
    check::<false, true>();
}
#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Mul<&num_bigint::BigUint>>::mul, multiply)]
#[kani::stub(
    crate::public_big_ops::cached_mask,
    super::multiplication::product_mask
)]
#[kani::stub(
    veryl_analyzer::value::ValueU64::multiply,
    super::multiplication::reject_native
)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::prepared_wide)]
fn wide_all_ones_left_matches_reference() {
    check::<true, false>();
}
#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(<&num_bigint::BigUint as std::ops::Mul<&num_bigint::BigUint>>::mul, multiply)]
#[kani::stub(
    crate::public_big_ops::cached_mask,
    super::multiplication::product_mask
)]
#[kani::stub(
    veryl_analyzer::value::ValueU64::multiply,
    super::multiplication::reject_native
)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::special::observed_expand)]
fn wide_all_ones_left_context_matches_reference() {
    check::<true, true>();
}
