//! Equality and inequality prefixes at native source widths, followed by wildcard
//! equality/inequality and less-than pairs for widths 1..=3, are proved in public_comparison.rs.
//! Keep every production call on the shared cache before later observations;
//! retain all other comparison checks and independent reference anchors.
use veryl_analyzer::ir::Op;
use veryl_analyzer::value::{MaskCache, Value};
use veryl_formal_semantics::comparison::{Comparison, Operand, compared_logic};
use veryl_formal_semantics::logic4::Logic4;
use veryl_formal_semantics::resize::reference_planes;

const COMPARISONS: [(Op, Comparison); 8] = [
    (Op::Eq, Comparison::Equal),
    (Op::Ne, Comparison::NotEqual),
    (Op::EqWildcard, Comparison::WildcardEqual),
    (Op::NeWildcard, Comparison::WildcardNotEqual),
    (Op::Less, Comparison::Less),
    (Op::LessEq, Comparison::LessEqual),
    (Op::Greater, Comparison::Greater),
    (Op::GreaterEq, Comparison::GreaterEqual),
];

const NATIVE_COMPARISON_PREFIX: [(Op, usize); 10] = [
    (Op::Eq, 1),
    (Op::Eq, 8),
    (Op::Ne, 1),
    (Op::Ne, 8),
    (Op::EqWildcard, 1),
    (Op::EqWildcard, 8),
    (Op::NeWildcard, 1),
    (Op::NeWildcard, 8),
    (Op::Less, 1),
    (Op::Less, 8),
];

fn check_comparisons(x: Operand, y: Operand, result_widths: &[usize]) {
    let lhs = Value::from_u128(x.planes.0, x.planes.1, x.width, x.signed);
    let rhs = Value::from_u128(y.planes.0, y.planes.1, y.width, y.signed);
    let mut cache = MaskCache::default();
    let normalized_native =
        |x: Operand| (1..=64).contains(&x.width) && ((x.planes.0 | x.planes.1) >> x.width) == 0;
    let native_sources = normalized_native(x) && normalized_native(y);
    let prefix_len = if x.width <= 3 && y.width <= 3 { 10 } else { 4 };
    let proved_prefix = &NATIVE_COMPARISON_PREFIX[..prefix_len];
    let mut calls = 0;
    let mut prefix_matches = true;
    for (op, kind) in COMPARISONS {
        let mut expected = None;
        for &width in result_widths {
            let result = op.eval_value_binary(&lhs, &rhs, width, x.signed && y.signed, &mut cache);
            let covered =
                prefix_matches && native_sources && proved_prefix.get(calls) == Some(&(op, width));
            prefix_matches = covered;
            calls += 1;
            if covered {
                continue;
            }
            let expected = *expected.get_or_insert_with(|| compared_logic(x, y, kind));
            let expected = reference_planes(width, |position| {
                if position == 0 {
                    expected
                } else {
                    Logic4::Zero
                }
            });
            assert_eq!(
                (result.payload_u128(), result.mask_xz_u128()),
                expected,
                "{kind:?}, x={x:?}, y={y:?}, result_width={width}"
            );
            assert_eq!(result.width(), width);
            assert!(!result.signed());
            assert_eq!(matches!(result, Value::U64(_)), width <= 64);
        }
    }
    assert_eq!((lhs.payload_u128(), lhs.mask_xz_u128()), x.planes);
    assert_eq!((rhs.payload_u128(), rhs.mask_xz_u128()), y.planes);
    assert_eq!((lhs.width(), lhs.signed()), (x.width, x.signed));
    assert_eq!((rhs.width(), rhs.signed()), (y.width, y.signed));
}

#[test]
fn comparison_exhaustive_four_state_small_widths() {
    for x_width in 1..=3 {
        for y_width in 1..=3 {
            let x_mask = (1u128 << x_width) - 1;
            let y_mask = (1u128 << y_width) - 1;
            for x_state in 0..(1u128 << (2 * x_width)) {
                for y_state in 0..(1u128 << (2 * y_width)) {
                    for x_signed in [false, true] {
                        for y_signed in [false, true] {
                            check_comparisons(
                                Operand {
                                    planes: (x_state & x_mask, x_state >> x_width),
                                    width: x_width,
                                    signed: x_signed,
                                },
                                Operand {
                                    planes: (y_state & y_mask, y_state >> y_width),
                                    width: y_width,
                                    signed: y_signed,
                                },
                                &[1, 8],
                            );
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn comparison_unknown_bits_do_not_create_known_mismatches() {
    for width in [1, 64, 65, 128] {
        for unknown in [(0, 1), (1, 1)] {
            for (x, y) in [((1, 0), unknown), (unknown, (1, 0))] {
                check_comparisons(
                    Operand {
                        planes: x,
                        width,
                        signed: false,
                    },
                    Operand {
                        planes: y,
                        width,
                        signed: false,
                    },
                    &[1, 64, 65, 128],
                );
            }
        }
    }
}

#[test]
fn comparison_native_biguint_and_signedness_boundaries() {
    fn patterns(width: usize) -> [(u128, u128); 9] {
        let mask = u128::MAX >> (128 - width);
        let sign = 1u128 << (width - 1);
        [
            (0, 0),
            (mask, 0),
            (1, 0),
            (sign, 0),
            (0, sign),
            (sign, sign),
            (1, sign),
            (0xaaaa_aaaa_aaaa_aaaa_aaaa_aaaa_aaaa_aaaa & mask, 0),
            (0x5555_5555_5555_5555_5555_5555_5555_5555 & mask, mask >> 1),
        ]
    }
    for x_width in [1, 2, 31, 32, 63, 64, 65, 127, 128] {
        for y_width in [1, 2, 31, 32, 63, 64, 65, 127, 128] {
            for x in patterns(x_width) {
                for y in patterns(y_width) {
                    for x_signed in [false, true] {
                        for y_signed in [false, true] {
                            check_comparisons(
                                Operand {
                                    planes: x,
                                    width: x_width,
                                    signed: x_signed,
                                },
                                Operand {
                                    planes: y,
                                    width: y_width,
                                    signed: y_signed,
                                },
                                &[1, 64, 65, 128],
                            );
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn comparison_reference_single_bit_truth_tables() {
    let bits = [(0, 0), (1, 0), (0, 1), (1, 1)];
    let z = Logic4::Zero;
    let o = Logic4::One;
    let u = Logic4::X;
    let eq = [[o, z, u, u], [z, o, u, u], [u, u, u, u], [u, u, u, u]];
    let wild = [[o, z, o, o], [z, o, o, o], [u, u, o, o], [u, u, o, o]];
    for (i, &a) in bits.iter().enumerate() {
        for (j, &b) in bits.iter().enumerate() {
            let x = Operand {
                planes: a,
                width: 1,
                signed: false,
            };
            let y = Operand {
                planes: b,
                width: 1,
                signed: false,
            };
            assert_eq!(compared_logic(x, y, Comparison::Equal), eq[i][j]);
            assert_eq!(compared_logic(x, y, Comparison::WildcardEqual), wild[i][j]);
            for kind in [Comparison::NotEqual, Comparison::WildcardNotEqual] {
                let expected = if kind == Comparison::NotEqual {
                    eq[i][j]
                } else {
                    wild[i][j]
                };
                let expected = match expected {
                    Logic4::Zero => o,
                    Logic4::One => z,
                    _ => u,
                };
                assert_eq!(compared_logic(x, y, kind), expected);
            }
        }
    }
}

#[test]
fn comparison_reference_known_relational_order() {
    for x_width in 1..=5 {
        for y_width in 1..=5 {
            for x in 0..(1u128 << x_width) {
                for y in 0..(1u128 << y_width) {
                    for x_signed in [false, true] {
                        for y_signed in [false, true] {
                            let signed = x_signed && y_signed;
                            let integer = |value: u128, width| {
                                let negative = signed && value >= 1u128 << (width - 1);
                                value as i128 - if negative { 1i128 << width } else { 0 }
                            };
                            let a = integer(x, x_width);
                            let b = integer(y, y_width);
                            for (kind, expected) in [
                                (Comparison::Less, a < b),
                                (Comparison::LessEqual, a <= b),
                                (Comparison::Greater, a > b),
                                (Comparison::GreaterEqual, a >= b),
                            ] {
                                let expected = if expected { Logic4::One } else { Logic4::Zero };
                                assert_eq!(
                                    compared_logic(
                                        Operand {
                                            planes: (x, 0),
                                            width: x_width,
                                            signed: x_signed
                                        },
                                        Operand {
                                            planes: (y, 0),
                                            width: y_width,
                                            signed: y_signed
                                        },
                                        kind,
                                    ),
                                    expected
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}
