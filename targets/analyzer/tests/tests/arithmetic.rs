//! Fresh-cache small arithmetic is proved in `targets/analyzer/kani/src/arithmetic.rs`.
//! Execute every call to preserve cache history. Kani covers the first binary
//! Add/Sub pair and first six unary calls (natural width, then width + 2)
//! for normalized sources at widths 1..=3.
//! Compare later results after executing the complete prefix on the same cache.
//! Wider/BigUint operations, sentinels, and reference anchors remain here.
//! X/Z unary plus follows the Table 11-6 identity interpretation in veryl_formal_semantics::arithmetic.
use veryl_analyzer::ir::Op;
use veryl_analyzer::value::{MaskCache, Value};
use veryl_formal_semantics::arithmetic::{Arithmetic, arithmetic_bit, arithmetic_planes};
use veryl_formal_semantics::comparison::Operand;
use veryl_formal_semantics::logic4::read_logic;
use veryl_formal_semantics::resize::{extended_logic_bit, reference_planes};

fn prepared(x: Operand, width: usize, signed: bool) -> (u128, u128) {
    reference_planes(width, |position| {
        extended_logic_bit(x.planes, x.width, width, x.signed, signed, position)
    })
}

fn small_normalized(input: Operand) -> bool {
    (1..=3).contains(&input.width) && ((input.planes.0 | input.planes.1) >> input.width) == 0
}

fn check_binary(a: Operand, b: Operand, widths: &[usize]) {
    let x = Value::from_u128(a.planes.0, a.planes.1, a.width, a.signed);
    let y = Value::from_u128(b.planes.0, b.planes.1, b.width, b.signed);
    let mut cache = MaskCache::default();
    let mut calls = 0;
    let mut initial_add_covered = false;
    for &width in widths {
        for signed in [false, true] {
            let ap = prepared(a, width, signed);
            let bp = prepared(b, width, signed);
            for (op, kind) in [(Op::Add, Arithmetic::Add), (Op::Sub, Arithmetic::Subtract)] {
                let result = op.eval_value_binary(&x, &y, width, signed, &mut cache);
                let initial_layout = small_normalized(a)
                    && small_normalized(b)
                    && width == a.width.max(b.width)
                    && !signed;
                let covered = initial_layout
                    && ((calls == 0 && op == Op::Add)
                        || (calls == 1 && initial_add_covered && op == Op::Sub));
                if calls == 0 {
                    initial_add_covered = covered;
                }
                calls += 1;
                if covered {
                    continue;
                }
                assert_eq!(
                    (result.payload_u128(), result.mask_xz_u128()),
                    arithmetic_planes(ap, bp, width, kind),
                    "{kind:?}, a={a:?}, b={b:?}, width={width}, signed={signed}"
                );
                assert_eq!(result.width(), width);
                assert_eq!(result.signed(), signed);
                assert_eq!(matches!(result, Value::U64(_)), width <= 64);
            }
        }
    }
    for (value, expected) in [(&x, a), (&y, b)] {
        assert_eq!(
            (value.payload_u128(), value.mask_xz_u128()),
            expected.planes
        );
        assert_eq!(
            (value.width(), value.signed()),
            (expected.width, expected.signed)
        );
    }
}

fn check_unary(a: Operand, widths: &[usize]) {
    let x = Value::from_u128(a.planes.0, a.planes.1, a.width, a.signed);
    let mut cache = MaskCache::default();
    let mut calls = 0;
    let mut prefix_matches = true;
    let prefix = [
        (Op::Add, false, 0),
        (Op::Sub, false, 0),
        (Op::Add, true, 0),
        (Op::Sub, true, 0),
        (Op::Add, false, 2),
        (Op::Sub, false, 2),
    ];
    for &width in widths {
        for signed in [false, true] {
            let ap = prepared(a, width, signed);
            for (op, kind) in [(Op::Add, Arithmetic::Plus), (Op::Sub, Arithmetic::Negate)] {
                let result = op.eval_value_unary(&x, width, signed, &mut cache);
                let covered = prefix_matches
                    && small_normalized(a)
                    && prefix
                        .get(calls)
                        .is_some_and(|&(expected_op, expected_sign, extra)| {
                            op == expected_op && signed == expected_sign && width == a.width + extra
                        });
                prefix_matches = covered;
                calls += 1;
                if covered {
                    continue;
                }
                assert_eq!(
                    (result.payload_u128(), result.mask_xz_u128()),
                    arithmetic_planes(ap, (0, 0), width, kind),
                    "{kind:?}, a={a:?}, width={width}, use_sign={signed}"
                );
                assert_eq!(result.width(), width);
                assert_eq!(result.signed(), a.signed && (width == a.width || signed));
                assert_eq!(matches!(result, Value::U64(_)), width <= 64);
            }
        }
    }
    assert_eq!((x.payload_u128(), x.mask_xz_u128()), a.planes);
    assert_eq!((x.width(), x.signed()), (a.width, a.signed));
}

#[test]
fn arithmetic_exhaustive_four_state_small_widths() {
    for a_width in 1..=3 {
        for a_state in 0..(1u128 << (a_width * 2)) {
            for a_signed in [false, true] {
                let a = Operand {
                    planes: (a_state & ((1 << a_width) - 1), a_state >> a_width),
                    width: a_width,
                    signed: a_signed,
                };
                check_unary(a, &[a_width, a_width + 2]);
                for b_width in 1..=3 {
                    for b_state in 0..(1u128 << (b_width * 2)) {
                        for b_signed in [false, true] {
                            let b = Operand {
                                planes: (b_state & ((1 << b_width) - 1), b_state >> b_width),
                                width: b_width,
                                signed: b_signed,
                            };
                            check_binary(a, b, &[a_width.max(b_width), a_width.max(b_width) + 2]);
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn arithmetic_native_biguint_boundaries() {
    fn patterns(width: usize) -> [(u128, u128); 9] {
        let mask = u128::MAX >> (128 - width);
        let sign = 1u128 << (width - 1);
        [
            (0, 0),
            (1, 0),
            (mask, 0),
            (sign, 0),
            (sign - 1, 0),
            (0, sign),
            (sign, sign),
            (1, sign),
            (mask, mask),
        ]
    }
    for a_width in [1, 2, 31, 32, 63, 64, 65, 127, 128] {
        for a in patterns(a_width) {
            for a_signed in [false, true] {
                let a = Operand {
                    planes: a,
                    width: a_width,
                    signed: a_signed,
                };
                check_unary(a, &[a_width, 128]);
                for b_width in [1, 2, 31, 32, 63, 64, 65, 127, 128] {
                    for b in patterns(b_width) {
                        for b_signed in [false, true] {
                            let b = Operand {
                                planes: b,
                                width: b_width,
                                signed: b_signed,
                            };
                            check_binary(a, b, &[a_width.max(b_width), 128]);
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn arithmetic_reference_matches_integer_residues() {
    for width in 1..=6 {
        let modulus = 1u128 << width;
        for a in 0..modulus {
            assert_eq!(
                arithmetic_planes((a, 0), (0, 0), width, Arithmetic::Plus),
                (a, 0)
            );
            assert_eq!(
                arithmetic_planes((a, 0), (0, 0), width, Arithmetic::Negate),
                ((modulus - a) % modulus, 0)
            );
            for b in 0..modulus {
                assert_eq!(
                    arithmetic_planes((a, 0), (b, 0), width, Arithmetic::Add),
                    ((a + b) % modulus, 0)
                );
                assert_eq!(
                    arithmetic_planes((a, 0), (b, 0), width, Arithmetic::Subtract),
                    ((modulus + a - b) % modulus, 0)
                );
            }
        }
    }
    let source = (0b1001, 0b0011);
    assert_eq!(
        arithmetic_planes(source, (0, 0), 4, Arithmetic::Plus),
        source
    );
    for kind in [Arithmetic::Add, Arithmetic::Subtract, Arithmetic::Negate] {
        assert_eq!(arithmetic_planes(source, (0, 0), 4, kind), (0, 15));
    }
}

#[test]
fn arithmetic_reference_bit_observations_match_planes() {
    for width in 1..=3 {
        let mask = (1u128 << width) - 1;
        for a in 0..(1u128 << (width * 2)) {
            for b in 0..(1u128 << (width * 2)) {
                let a = (a & mask, a >> width);
                let b = (b & mask, b >> width);
                for kind in [
                    Arithmetic::Add,
                    Arithmetic::Subtract,
                    Arithmetic::Negate,
                    Arithmetic::Plus,
                ] {
                    let planes = arithmetic_planes(a, b, width, kind);
                    for position in 0..=width {
                        assert_eq!(
                            arithmetic_bit(a, b, width, kind, position),
                            read_logic(planes.0, planes.1, position)
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn arithmetic_materializes_all_bit_sentinels() {
    for width in [1, 64, 65, 128] {
        let mask = u128::MAX >> (128 - width);
        let one = Value::new(1, width, false);
        for literal in ["'0", "'1", "'x", "'z"] {
            let input: Value = literal.parse().unwrap();
            let unknown = literal == "'x" || literal == "'z";
            let identity = match literal {
                "'0" => (0, 0),
                "'1" => (mask, 0),
                "'x" => (0, mask),
                _ => (mask, mask),
            };
            let mut cache = MaskCache::default();
            let outputs = [
                (
                    Op::Add.eval_value_unary(&input, width, false, &mut cache),
                    identity,
                ),
                (
                    Op::Sub.eval_value_unary(&input, width, false, &mut cache),
                    if unknown {
                        (0, mask)
                    } else if literal == "'0" {
                        (0, 0)
                    } else {
                        (1, 0)
                    },
                ),
                (
                    Op::Add.eval_value_binary(&input, &one, width, false, &mut cache),
                    if unknown {
                        (0, mask)
                    } else if literal == "'0" {
                        (1, 0)
                    } else {
                        (0, 0)
                    },
                ),
                (
                    Op::Sub.eval_value_binary(&input, &one, width, false, &mut cache),
                    if unknown {
                        (0, mask)
                    } else if literal == "'0" {
                        (mask, 0)
                    } else {
                        (mask ^ 1, 0)
                    },
                ),
            ];
            for (result, expected) in outputs {
                assert_eq!(
                    (result.payload_u128(), result.mask_xz_u128()),
                    expected,
                    "{literal}, width={width}"
                );
                assert_eq!(result.width(), width);
                assert!(!result.signed());
                assert_eq!(matches!(result, Value::U64(_)), width <= 64);
            }
        }
    }
}
