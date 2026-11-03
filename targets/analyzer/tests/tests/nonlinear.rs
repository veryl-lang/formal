//! Kani proves the normalized widths 1..=3 Mul/Div/Rem prefix at natural/natural+1
//! contexts, with independent input/caller signs. Keep every call and compare
//! ordinary positive powers and native/BigUint boundaries. Harnesses in exceptions.rs,
//! power.rs, identities.rs, special.rs, division.rs, multiplication.rs, opposite.rs, signed_small.rs, signed_power_two.rs, ones.rs, ones_power.rs and hot_power.rs, unit_quotient.rs, power_quotient.rs and signed_power_quotient.rs cover known-value
//! arithmetic paths under numeric/expansion contracts. Keep all calls/cache
//! history, general numerical paths, model anchors and real-library boundaries.
#[path = "../wide_cases.rs"]
mod wide_cases;
use veryl_analyzer::ir::Op;
use veryl_analyzer::value::{MaskCache, Value};
use veryl_formal_semantics::comparison::Operand;
use veryl_formal_semantics::nonlinear::{
    DivisionClass, Nonlinear, all_ones_power_word, all_ones_product_word, division_class_bit,
    negated_word_bit, one_hot_power_bit, one_hot_product_bit, one_hot_product_word,
    opposite_division_word, planes, power_quotient_bit, power_two_division_magnitude_bit,
    signed_magnitude_word, signed_power_quotient_word, signed_power_two_division_word,
    signed_smaller_division_word, unit_quotient_bit,
};
use veryl_formal_semantics::resize::{
    expanded_planes, extended_logic_bit, reference_planes, single_hot_expansion_member,
};

fn widened(a: Operand, width: usize, signed: bool) -> (u128, u128) {
    reference_planes(width, |p| {
        extended_logic_bit(a.planes, a.width, width, a.signed, signed, p)
    })
}
fn signed_magnitude(word: u128, width: usize) -> u128 {
    let shift = 128 - width;
    (((word << shift) as i128) >> shift).unsigned_abs()
}
fn check(a: Operand, b: Operand, boundaries: bool) {
    let x = Value::from_u128(a.planes.0, a.planes.1, a.width, a.signed);
    let y = Value::from_u128(b.planes.0, b.planes.1, b.width, b.signed);
    let mut cache = MaskCache::default();
    for (op, kind) in [
        (Op::Mul, Nonlinear::Multiply),
        (Op::Div, Nonlinear::Divide),
        (Op::Rem, Nonlinear::Remainder),
        (Op::Pow, Nonlinear::Power),
    ] {
        let power = matches!(kind, Nonlinear::Power);
        let min = if power { a.width } else { a.width.max(b.width) };
        for width in [min, if boundaries { 128 } else { min + 1 }] {
            for signed in [false, true] {
                let result = op.eval_value_binary(&x, &y, width, signed, &mut cache);
                // Execute the proved prefix to preserve the cache used by Pow.
                let small = (1..=3).contains(&a.width)
                    && (1..=3).contains(&b.width)
                    && (a.planes.0 | a.planes.1) >> a.width == 0
                    && (b.planes.0 | b.planes.1) >> b.width == 0;
                if small
                    && ((!boundaries
                        && matches!(
                            kind,
                            Nonlinear::Multiply | Nonlinear::Divide | Nonlinear::Remainder
                        ))
                        || (boundaries && width == min && matches!(kind, Nonlinear::Multiply)))
                {
                    continue;
                }
                let unknown = (a.planes.1 | b.planes.1) != 0;
                let exceptional = match kind {
                    Nonlinear::Multiply => unknown,
                    Nonlinear::Divide | Nonlinear::Remainder => unknown || b.planes.0 == 0,
                    Nonlinear::Power => {
                        b.planes.1 != 0
                            || b.planes.0 == 0
                            || ((1..=128).contains(&b.width)
                                && b.signed
                                && b.planes.0 >> (b.width - 1) & 1 != 0)
                    }
                };
                let wide_domain = (65..=128).contains(&width)
                    && a.width <= width
                    && (power || b.width <= width)
                    && wide_cases::normalized(a)
                    && wide_cases::normalized(b);
                let numerical_identity = if wide_domain && !unknown {
                    let ap = expanded_planes(a.planes, a.width, width, a.signed, signed).0;
                    let bp = if power {
                        b.planes.0
                    } else {
                        expanded_planes(b.planes, b.width, width, b.signed, signed).0
                    };
                    let mask = u128::MAX >> (128 - width);
                    match kind {
                        Nonlinear::Multiply => {
                            ap <= 1
                                || bp <= 1
                                || ap.is_power_of_two()
                                || bp.is_power_of_two()
                                || ap == mask
                                || bp == mask
                        }
                        Nonlinear::Divide | Nonlinear::Remainder => {
                            bp != 0
                                && (ap == 0
                                    || (!signed && bp == 1)
                                    || (signed && (bp == 1 || bp == mask))
                                    || ap == bp
                                    || (signed && ap != bp && bp == (ap.wrapping_neg() & mask))
                                    || (signed
                                        && signed_magnitude(ap, width)
                                            < signed_magnitude(bp, width))
                                    || (signed && signed_magnitude(bp, width).is_power_of_two())
                                    || (signed
                                        && (signed_magnitude(ap, width)
                                            / signed_magnitude(bp, width))
                                        .is_power_of_two())
                                    || (!signed
                                        && (ap < bp
                                            || bp.is_power_of_two()
                                            || (ap >= bp && ap - bp < bp)
                                            || (ap / bp).is_power_of_two())))
                        }
                        Nonlinear::Power => {
                            let positive = bp != 0 && !(b.signed && bp >> (b.width - 1) & 1 != 0);
                            positive && (bp == 1 || ap <= 1 || ap == mask || ap.is_power_of_two())
                        }
                    }
                } else {
                    false
                };
                if wide_domain && (exceptional || numerical_identity) {
                    let result_signed = if power {
                        b.planes.1 == 0 && a.signed && (width == a.width || signed)
                    } else {
                        signed
                    };
                    wide_cases::shape(&result, width, result_signed);
                    // Small-exponent cases also compare two independent models.
                    if !(wide_cases::boundary(a.width) && wide_cases::boundary(b.width))
                        && !(power && b.width <= 3)
                    {
                        continue;
                    }
                }
                let ap = widened(a, width, signed);
                let bp = if power {
                    b.planes
                } else {
                    widened(b, width, signed)
                };
                let base_signed = a.signed && (width == a.width || signed);
                let expected = planes(
                    ap,
                    bp,
                    width,
                    if power { base_signed } else { signed },
                    b.width,
                    b.signed,
                    kind,
                );
                if power && b.width <= 3 {
                    assert_eq!(
                        veryl_formal_semantics::nonlinear::small_power_planes(
                            ap,
                            bp,
                            width,
                            base_signed,
                            b.width,
                            b.signed
                        ),
                        expected
                    );
                }
                assert_eq!(
                    (result.payload_u128(), result.mask_xz_u128()),
                    expected,
                    "{kind:?}, a={a:?}, b={b:?}, width={width}, signed={signed}"
                );
                assert_eq!(result.width(), width);
                assert_eq!(
                    result.signed(),
                    if power {
                        b.planes.1 == 0 && base_signed
                    } else {
                        signed
                    }
                );
                assert_eq!(matches!(result, Value::U64(_)), width <= 64);
            }
        }
    }
    for (value, before) in [(&x, a), (&y, b)] {
        assert_eq!((value.payload_u128(), value.mask_xz_u128()), before.planes);
        assert_eq!(
            (value.width(), value.signed()),
            (before.width, before.signed)
        );
    }
}
#[test]
fn nonlinear_exhaustive_four_state_small_widths() {
    for aw in 1..=2 {
        for bw in 1..=2 {
            for av in 0..(1u128 << (aw * 2)) {
                for bv in 0..(1u128 << (bw * 2)) {
                    for sa in [false, true] {
                        for sb in [false, true] {
                            check(
                                Operand {
                                    planes: (av & ((1 << aw) - 1), av >> aw),
                                    width: aw,
                                    signed: sa,
                                },
                                Operand {
                                    planes: (bv & ((1 << bw) - 1), bv >> bw),
                                    width: bw,
                                    signed: sb,
                                },
                                false,
                            );
                        }
                    }
                }
            }
        }
    }
}
#[test]
fn nonlinear_native_biguint_boundaries() {
    fn patterns(w: usize) -> [(u128, u128); 8] {
        let mask = u128::MAX >> (128 - w);
        let sign = 1u128 << (w - 1);
        [
            (0, 0),
            (1, 0),
            (mask, 0),
            (sign, 0),
            (sign - 1, 0),
            (0, sign),
            (sign, sign),
            (mask, mask),
        ]
    }
    for aw in [1, 4, 63, 64, 65, 127, 128] {
        for bw in [1, 4, 63, 64, 65, 127, 128] {
            for ap in patterns(aw) {
                for bp in patterns(bw) {
                    for sa in [false, true] {
                        for sb in [false, true] {
                            check(
                                Operand {
                                    planes: ap,
                                    width: aw,
                                    signed: sa,
                                },
                                Operand {
                                    planes: bp,
                                    width: bw,
                                    signed: sb,
                                },
                                true,
                            );
                        }
                    }
                }
            }
        }
    }
}
#[test]
fn nonlinear_reference_integer_oracles() {
    for width in 1..=6 {
        let m = 1u128 << width;
        for a in 0..m {
            for b in 0..m {
                assert_eq!(
                    planes(
                        (a, 0),
                        (b, 0),
                        width,
                        false,
                        width,
                        false,
                        Nonlinear::Multiply
                    ),
                    ((a * b) % m, 0)
                );
                {
                    let padding = 128 - width;
                    let x = ((a << padding) as i128) >> padding;
                    let y = ((b << padding) as i128) >> padding;
                    if y != 0 {
                        let quotient = x.unsigned_abs() / y.unsigned_abs();
                        if quotient.is_power_of_two() {
                            let k = quotient.trailing_zeros() as usize;
                            for remainder in [false, true] {
                                let expected =
                                    (if remainder { x % y } else { x / y }) as u128 & (m - 1);
                                assert_eq!(
                                    signed_power_quotient_word(a, b, k, width, remainder),
                                    expected
                                );
                            }
                        }
                    }
                }
                if b != 0 {
                    let quotient = a / b;
                    if quotient.is_power_of_two() {
                        let shift = quotient.trailing_zeros() as usize;
                        for remainder in [false, true] {
                            let expected = if remainder { a % b } else { quotient };
                            for p in 0..=width {
                                assert_eq!(
                                    power_quotient_bit(a, b, shift, width, p, remainder),
                                    expected >> p & 1 != 0
                                );
                            }
                        }
                    }
                }
                if b != 0 && a >= b && a - b < b {
                    for remainder in [false, true] {
                        let expected = if remainder { a % b } else { a / b };
                        for p in 0..=width {
                            assert_eq!(
                                unit_quotient_bit(a, b, width, p, remainder),
                                expected >> p & 1 != 0
                            );
                        }
                    }
                }
                if b == 0 {
                    for target in [width, width + 1, 65, 128] {
                        for source_signed in [false, true] {
                            for caller in [false, true] {
                                let expanded = widened(
                                    Operand {
                                        planes: (a, 0),
                                        width,
                                        signed: source_signed,
                                    },
                                    target,
                                    caller,
                                );
                                for k in 0..target {
                                    assert_eq!(
                                        single_hot_expansion_member(
                                            a,
                                            width,
                                            target,
                                            source_signed,
                                            caller,
                                            k
                                        ),
                                        expanded == (1u128 << k, 0)
                                    );
                                }
                            }
                        }
                    }
                }
                if a.is_power_of_two() {
                    let mut expected = 1;
                    for _ in 0..b {
                        expected = (expected * a) % m;
                    }
                    for p in 0..=width {
                        assert_eq!(
                            one_hot_power_bit(a.trailing_zeros() as usize, b, width, p),
                            expected >> p & 1 != 0
                        );
                    }
                }
                if a == m - 1 && b != 0 {
                    let mut expected = 1;
                    for _ in 0..b {
                        expected = (expected * a) % m;
                    }
                    for exponent_signed in [false, true] {
                        if !exponent_signed || b >> (width - 1) & 1 == 0 {
                            assert_eq!(
                                all_ones_power_word(a, b, width, width, exponent_signed),
                                expected
                            );
                        }
                    }
                }
                for (number, factor) in [(a, b), (b, a)] {
                    if factor == m - 1 {
                        let word = all_ones_product_word(number, factor, width);
                        assert_eq!(word, (a * b) % m);
                        for p in 0..=width {
                            assert_eq!(negated_word_bit(number, width, p), word >> p & 1 != 0);
                        }
                    }
                }
                for (number, factor) in [(a, b), (b, a)] {
                    if factor.is_power_of_two() {
                        assert_eq!(
                            one_hot_product_word(
                                number,
                                factor,
                                factor.trailing_zeros() as usize,
                                width
                            ),
                            (a * b) % m
                        );
                        for p in 0..=width {
                            assert_eq!(
                                one_hot_product_bit(
                                    number,
                                    factor,
                                    factor.trailing_zeros() as usize,
                                    width,
                                    p
                                ),
                                ((a * b) % m) >> p & 1 != 0
                            );
                        }
                    }
                }
                if b != 0 {
                    assert_eq!(
                        planes(
                            (a, 0),
                            (b, 0),
                            width,
                            false,
                            width,
                            false,
                            Nonlinear::Divide
                        ),
                        (a / b, 0)
                    );
                    assert_eq!(
                        planes(
                            (a, 0),
                            (b, 0),
                            width,
                            false,
                            width,
                            false,
                            Nonlinear::Remainder
                        ),
                        (a % b, 0)
                    );
                    if a == 0 {
                        assert_eq!(
                            veryl_formal_semantics::nonlinear::division_identity(
                                a, b, width, false
                            ),
                            (a / b, a % b)
                        );
                    }
                    for (class, eligible) in [
                        (DivisionClass::Equal, a == b),
                        (DivisionClass::UnsignedSmaller, a < b),
                        (DivisionClass::UnsignedPowerOfTwo, b.is_power_of_two()),
                    ] {
                        if eligible {
                            for p in 0..=width {
                                assert_eq!(
                                    division_class_bit(a, b, width, false, p, false, class),
                                    a / b >> p & 1 != 0
                                );
                                assert_eq!(
                                    division_class_bit(a, b, width, false, p, true, class),
                                    a % b >> p & 1 != 0
                                );
                            }
                        }
                    }
                    let sa = if a >= m / 2 {
                        a as i128 - m as i128
                    } else {
                        a as i128
                    };
                    let sb = if b >= m / 2 {
                        b as i128 - m as i128
                    } else {
                        b as i128
                    };
                    assert_eq!(signed_magnitude_word(sa.unsigned_abs(), sa < 0, width), a);
                    assert_eq!(signed_magnitude_word(sb.unsigned_abs(), sb < 0, width), b);
                    if sb.unsigned_abs().is_power_of_two() {
                        assert_eq!(
                            signed_power_two_division_word(
                                a,
                                b,
                                sb.unsigned_abs().trailing_zeros() as usize,
                                width,
                                false
                            ),
                            (sa / sb).rem_euclid(m as i128) as u128
                        );
                        assert_eq!(
                            signed_power_two_division_word(
                                a,
                                b,
                                sb.unsigned_abs().trailing_zeros() as usize,
                                width,
                                true
                            ),
                            (sa % sb).rem_euclid(m as i128) as u128
                        );
                        let shift = sb.unsigned_abs().trailing_zeros() as usize;
                        for p in 0..=width {
                            assert_eq!(
                                power_two_division_magnitude_bit(
                                    sa.unsigned_abs(),
                                    shift,
                                    p,
                                    false
                                ),
                                sa.unsigned_abs() / sb.unsigned_abs() >> p & 1 != 0
                            );
                            assert_eq!(
                                power_two_division_magnitude_bit(sa.unsigned_abs(), shift, p, true),
                                sa.unsigned_abs() % sb.unsigned_abs() >> p & 1 != 0
                            );
                        }
                    }
                    if sa.unsigned_abs() < sb.unsigned_abs() {
                        assert_eq!(
                            signed_smaller_division_word(a, b, width, false),
                            (sa / sb).rem_euclid(m as i128) as u128
                        );
                        assert_eq!(
                            signed_smaller_division_word(a, b, width, true),
                            (sa % sb).rem_euclid(m as i128) as u128
                        );
                    }
                    if sa != 0 && sa == -sb {
                        assert_eq!(
                            opposite_division_word(a, b, width, false),
                            (sa / sb).rem_euclid(m as i128) as u128
                        );
                        assert_eq!(
                            opposite_division_word(a, b, width, true),
                            (sa % sb).rem_euclid(m as i128) as u128
                        );
                    }
                    if a == b {
                        for p in 0..=width {
                            assert_eq!(
                                division_class_bit(
                                    a,
                                    b,
                                    width,
                                    true,
                                    p,
                                    false,
                                    DivisionClass::Equal
                                ),
                                (sa / sb).rem_euclid(m as i128) as u128 >> p & 1 != 0
                            );
                            assert_eq!(
                                division_class_bit(
                                    a,
                                    b,
                                    width,
                                    true,
                                    p,
                                    true,
                                    DivisionClass::Equal
                                ),
                                (sa % sb).rem_euclid(m as i128) as u128 >> p & 1 != 0
                            );
                        }
                    }
                    if a == 0 || b == 1 || b == m - 1 {
                        assert_eq!(
                            veryl_formal_semantics::nonlinear::division_identity(a, b, width, true),
                            (
                                ((sa / sb).rem_euclid(m as i128)) as u128,
                                ((sa % sb).rem_euclid(m as i128)) as u128
                            )
                        );
                    }
                    assert_eq!(
                        planes((a, 0), (b, 0), width, true, width, false, Nonlinear::Divide),
                        (((sa / sb).rem_euclid(m as i128)) as u128, 0)
                    );
                    assert_eq!(
                        planes(
                            (a, 0),
                            (b, 0),
                            width,
                            true,
                            width,
                            false,
                            Nonlinear::Remainder
                        ),
                        (((sa % sb).rem_euclid(m as i128)) as u128, 0)
                    );
                }
            }
        }
        for a in 0..m {
            for exponent in 0..=7 {
                if exponent > 0 && (exponent == 1 || a <= 1) {
                    assert_eq!(
                        veryl_formal_semantics::nonlinear::positive_power_identity(
                            a, exponent, width, 4, false
                        ),
                        a.pow(exponent as u32) % m
                    );
                }
                assert_eq!(
                    planes(
                        (a, 0),
                        (exponent, 0),
                        width,
                        false,
                        4,
                        false,
                        Nonlinear::Power
                    ),
                    (a.pow(exponent as u32) % m, 0)
                );
            }
        }
    }
}

#[test]
fn nonlinear_power_unknown_negative_and_wide_exponents() {
    let mut cache = MaskCache::default();
    for width in [4, 64, 65, 128] {
        let base = Value::new(1, width, false);
        let exponent = Value::from_u128(1, 1, 1, true);
        let output = Op::Pow.eval_value_binary(&base, &exponent, width, false, &mut cache);
        assert_eq!(
            (output.payload_u128(), output.mask_xz_u128()),
            (0, u128::MAX >> (128 - width))
        );
    }
    let exponent = Value::from_u128(1u128 << 64, 0, 128, false);
    for width in [4, 64, 65, 128] {
        let base = Value::new(3, width, false);
        let output = Op::Pow.eval_value_binary(&base, &exponent, width, false, &mut cache);
        let expected = planes(
            (3, 0),
            (1u128 << 64, 0),
            width,
            false,
            128,
            false,
            Nonlinear::Power,
        );
        assert_eq!((output.payload_u128(), output.mask_xz_u128()), expected);
    }
}
