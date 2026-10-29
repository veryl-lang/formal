//! Kani proves the normalized widths 1..=3 Mul/Div prefix at natural/natural+1
//! contexts, with independent input/caller signs. Keep every call and compare
//! Rem/Pow and native/BigUint boundaries. Boundary Div has a different preceding
//! wide Mul cache history and is deliberately retained.
use veryl_analyzer::ir::Op;
use veryl_analyzer::value::{MaskCache, Value};
use veryl_formal_semantics::comparison::Operand;
use veryl_formal_semantics::nonlinear::{Nonlinear, planes};
use veryl_formal_semantics::resize::{extended_logic_bit, reference_planes};

fn widened(a: Operand, width: usize, signed: bool) -> (u128, u128) {
    reference_planes(width, |p| {
        extended_logic_bit(a.planes, a.width, width, a.signed, signed, p)
    })
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
                // Execute the proved prefix to preserve the cache used by Rem/Pow.
                let small = (1..=3).contains(&a.width)
                    && (1..=3).contains(&b.width)
                    && (a.planes.0 | a.planes.1) >> a.width == 0
                    && (b.planes.0 | b.planes.1) >> b.width == 0;
                if small
                    && ((!boundaries && matches!(kind, Nonlinear::Multiply | Nonlinear::Divide))
                        || (boundaries && width == min && matches!(kind, Nonlinear::Multiply)))
                {
                    continue;
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
