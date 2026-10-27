use veryl_analyzer::ir::Op;
use veryl_analyzer::value::{MaskCache, Value};
use veryl_formal_semantics::comparison::Operand;
use veryl_formal_semantics::gates::{self, BitOp, Reduction};
use veryl_formal_semantics::logic4::Logic4;
use veryl_formal_semantics::resize::{extended_logic_bit, reference_planes};

const BINARY: [(Op, BitOp); 4] = [
    (Op::BitAnd, BitOp::And),
    (Op::BitOr, BitOp::Or),
    (Op::BitXor, BitOp::Xor),
    (Op::BitXnor, BitOp::Xnor),
];
const REDUCTIONS: [(Op, Reduction); 6] = [
    (Op::BitAnd, Reduction::And),
    (Op::BitNand, Reduction::Nand),
    (Op::BitOr, Reduction::Or),
    (Op::BitNor, Reduction::Nor),
    (Op::BitXor, Reduction::Xor),
    (Op::BitXnor, Reduction::Xnor),
];

fn planes(value: &Value) -> (u128, u128) {
    (value.payload_u128(), value.mask_xz_u128())
}

fn check_binary(x: Operand, y: Operand, widths: &[usize]) {
    let lhs = Value::from_u128(x.planes.0, x.planes.1, x.width, x.signed);
    let rhs = Value::from_u128(y.planes.0, y.planes.1, y.width, y.signed);
    let signed = x.signed && y.signed;
    let mut cache = MaskCache::default();
    for &width in widths {
        for (op, kind) in BINARY {
            let result = op.eval_value_binary(&lhs, &rhs, width, signed, &mut cache);
            let expected = reference_planes(width, |position| {
                gates::binary(
                    extended_logic_bit(x.planes, x.width, width, x.signed, signed, position),
                    extended_logic_bit(y.planes, y.width, width, y.signed, signed, position),
                    kind,
                )
            });
            assert_eq!(
                planes(&result),
                expected,
                "{kind:?}, x={x:?}, y={y:?}, width={width}"
            );
            assert_eq!(result.width(), width);
            assert!(!result.signed());
            assert_eq!(matches!(result, Value::U64(_)), width <= 64);
        }
    }
    for (op, kind) in [(Op::LogicAnd, BitOp::And), (Op::LogicOr, BitOp::Or)] {
        let expected = gates::binary(
            gates::logical_truth(x.planes, x.width),
            gates::logical_truth(y.planes, y.width),
            kind,
        );
        for width in [1, 8, 65] {
            let result = op.eval_value_binary(&lhs, &rhs, width, signed, &mut cache);
            assert_eq!(
                planes(&result),
                reference_planes(width, |position| if position == 0 {
                    expected
                } else {
                    Logic4::Zero
                }),
                "{op:?}, x={x:?}, y={y:?}"
            );
            assert_eq!(result.width(), width);
            assert!(!result.signed());
            assert_eq!(matches!(result, Value::U64(_)), width <= 64);
        }
    }
    assert_eq!(
        (planes(&lhs), lhs.width(), lhs.signed()),
        (x.planes, x.width, x.signed)
    );
    assert_eq!(
        (planes(&rhs), rhs.width(), rhs.signed()),
        (y.planes, y.width, y.signed)
    );
}

fn check_unary(x: Operand, widths: &[usize]) {
    let input = Value::from_u128(x.planes.0, x.planes.1, x.width, x.signed);
    let mut cache = MaskCache::default();
    for &width in widths {
        for use_sign in [false, true] {
            let result = Op::BitNot.eval_value_unary(&input, width, use_sign, &mut cache);
            assert_eq!(
                planes(&result),
                reference_planes(width, |position| {
                    gates::not(extended_logic_bit(
                        x.planes, x.width, width, x.signed, use_sign, position,
                    ))
                }),
                "bit not, x={x:?}, width={width}, use_sign={use_sign}"
            );
            assert_eq!(result.width(), width);
            assert_eq!(result.signed(), x.signed && (width == x.width || use_sign));
            assert_eq!(matches!(result, Value::U64(_)), width <= 64);
        }
    }
    for (op, expected) in REDUCTIONS
        .into_iter()
        .map(|(op, kind)| (op, gates::reduced_logic(x.planes, x.width, kind)))
        .chain([(
            Op::LogicNot,
            gates::not(gates::logical_truth(x.planes, x.width)),
        )])
    {
        for width in [1, 8, 65, 128] {
            let result = op.eval_value_unary(&input, width, x.signed, &mut cache);
            assert_eq!(
                planes(&result),
                reference_planes(width, |position| if position == 0 {
                    expected
                } else {
                    Logic4::Zero
                }),
                "{op:?}, x={x:?}, width={width}"
            );
            assert_eq!(result.width(), width);
            assert!(!result.signed());
            assert_eq!(matches!(result, Value::U64(_)), width <= 64);
        }
    }
    assert_eq!(
        (planes(&input), input.width(), input.signed()),
        (x.planes, x.width, x.signed)
    );
}

#[test]
fn gates_exhaustive_four_state_small_widths() {
    for x_width in 1..=3 {
        let mask = (1u128 << x_width) - 1;
        for state in 0..(1u128 << (2 * x_width)) {
            for x_signed in [false, true] {
                let x = Operand {
                    planes: (state & mask, state >> x_width),
                    width: x_width,
                    signed: x_signed,
                };
                check_unary(x, &[x_width, x_width + 2]);
                for y_width in 1..=3 {
                    let mask = (1u128 << y_width) - 1;
                    for state in 0..(1u128 << (2 * y_width)) {
                        for y_signed in [false, true] {
                            let y = Operand {
                                planes: (state & mask, state >> y_width),
                                width: y_width,
                                signed: y_signed,
                            };
                            check_binary(x, y, &[x_width.max(y_width), x_width.max(y_width) + 2]);
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn gates_native_biguint_and_context_boundaries() {
    fn patterns(width: usize) -> [(u128, u128); 8] {
        let mask = u128::MAX >> (128 - width);
        let sign = 1u128 << (width - 1);
        [
            (0, 0),
            (mask, 0),
            (0, mask),
            (mask, mask),
            (1, mask ^ 1),
            (sign, 0),
            (sign, sign),
            (0xaaaa_aaaa_aaaa_aaaa_aaaa_aaaa_aaaa_aaaa & mask, mask >> 1),
        ]
    }
    for x_width in [1, 2, 31, 32, 63, 64, 65, 127, 128] {
        for y_width in [1, 2, 31, 32, 63, 64, 65, 127, 128] {
            for x in patterns(x_width) {
                for y in patterns(y_width) {
                    for x_signed in [false, true] {
                        for y_signed in [false, true] {
                            check_binary(
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
                                &[x_width.max(y_width), 128],
                            );
                        }
                    }
                }
            }
        }
        for x in patterns(x_width) {
            for signed in [false, true] {
                check_unary(
                    Operand {
                        planes: x,
                        width: x_width,
                        signed,
                    },
                    &[x_width, 128],
                );
            }
        }
    }
}

#[test]
fn gates_bitand_narrows_cast_operands() {
    for source_width in [4, 64, 65, 128] {
        let source_mask = u128::MAX >> (128 - source_width);
        for width in [1, 4, 64] {
            if width > source_width {
                continue;
            }
            for (a, b) in [
                ((source_mask, 0), (source_mask, 0)),
                ((source_mask, source_mask), (0, 0)),
                ((source_mask, 0), (0, source_mask)),
            ] {
                for signed in [false, true] {
                    let x = Value::from_u128(a.0, a.1, source_width, signed);
                    let y = Value::from_u128(b.0, b.1, source_width, signed);
                    let output = Op::BitAnd.eval_value_binary(
                        &x,
                        &y,
                        width,
                        signed,
                        &mut MaskCache::default(),
                    );
                    let expected = reference_planes(width, |position| {
                        gates::binary(
                            veryl_formal_semantics::logic4::read_logic(a.0, a.1, position),
                            veryl_formal_semantics::logic4::read_logic(b.0, b.1, position),
                            BitOp::And,
                        )
                    });
                    assert_eq!(planes(&output), expected);
                    assert_eq!(output.width(), width);
                    assert!(!output.signed());
                    assert!(matches!(output, Value::U64(_)));
                    assert_eq!(planes(&x), a);
                    assert_eq!(planes(&y), b);
                }
            }
        }
    }
}

#[test]
fn gates_reference_truth_tables() {
    use Logic4::{One as I, X, Z, Zero as O};
    let bits = [O, I, X, Z];
    for (kind, table) in [
        (
            BitOp::And,
            [[O, O, O, O], [O, I, X, X], [O, X, X, X], [O, X, X, X]],
        ),
        (
            BitOp::Or,
            [[O, I, X, X], [I, I, I, I], [X, I, X, X], [X, I, X, X]],
        ),
        (
            BitOp::Xor,
            [[O, I, X, X], [I, O, X, X], [X, X, X, X], [X, X, X, X]],
        ),
        (
            BitOp::Xnor,
            [[I, O, X, X], [O, I, X, X], [X, X, X, X], [X, X, X, X]],
        ),
    ] {
        for (i, a) in bits.into_iter().enumerate() {
            for (j, b) in bits.into_iter().enumerate() {
                assert_eq!(gates::binary(a, b, kind), table[i][j]);
            }
        }
    }
    for (a, expected) in bits.into_iter().zip([I, O, X, X]) {
        assert_eq!(gates::not(a), expected);
    }
    for (planes, and, or, xor) in [
        ((0, 0), O, O, O),
        ((15, 0), I, I, O),
        ((6, 0), O, I, O),
        ((8, 0), O, I, I),
        ((3, 11), O, X, X),
        ((7, 11), X, I, X),
        ((15, 15), X, X, X),
    ] {
        for (kind, expected) in [
            (Reduction::And, and),
            (Reduction::Or, or),
            (Reduction::Xor, xor),
            (Reduction::Nand, gates::not(and)),
            (Reduction::Nor, gates::not(or)),
            (Reduction::Xnor, gates::not(xor)),
        ] {
            assert_eq!(gates::reduced_logic(planes, 4, kind), expected);
        }
        assert_eq!(gates::logical_truth(planes, 4), or);
    }
}
