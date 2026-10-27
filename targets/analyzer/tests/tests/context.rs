use veryl_analyzer::conv::Context;
use veryl_analyzer::ir::{Comptime, Expression, Factor, Op, Shape, Type, TypeKind, ValueVariant};
use veryl_analyzer::value::Value;
use veryl_formal_semantics::arithmetic::{Arithmetic, arithmetic_planes};
use veryl_formal_semantics::comparison::{Comparison, Operand, compared_logic};
use veryl_formal_semantics::gates::{BitOp, binary};
use veryl_formal_semantics::resize::{extended_logic_bit, reference_planes};

fn leaf(a: Operand) -> Expression {
    let mut expr = Expression::create_value(
        Value::from_u128(a.planes.0, a.planes.1, a.width, a.signed),
        Default::default(),
    );
    expr.comptime_mut().r#type.signed = a.signed;
    expr.comptime_mut().r#type.kind = TypeKind::Logic;
    expr
}

fn node(a: Expression, op: Op, b: Expression) -> Expression {
    Expression::Binary(Box::new(a), op, Box::new(b), Box::default())
}

fn evaluate(expr: &mut Expression, width: usize) -> Value {
    let mut ctx = Context::default();
    let mut gathered = expr.gather_context(&mut ctx);
    gathered.width = gathered.width.max(width);
    expr.apply_context(&mut ctx, gathered);
    let result = expr.eval_value(&mut ctx).unwrap();
    assert!(ctx.drain_errors().is_empty(), "{expr:?}");
    result
}

fn widened(a: Operand, width: usize, signed: bool) -> (u128, u128) {
    reference_planes(width, |p| {
        extended_logic_bit(a.planes, a.width, width, a.signed, signed, p)
    })
}

fn check_binary(a: Operand, b: Operand, width: usize) {
    let width = width.max(a.width).max(b.width);
    let signed = a.signed && b.signed;
    let ap = widened(a, width, signed);
    let bp = widened(b, width, signed);
    for (op, kind) in [(Op::Add, Arithmetic::Add), (Op::Sub, Arithmetic::Subtract)] {
        let mut expr = node(leaf(a), op, leaf(b));
        let result = evaluate(&mut expr, width);
        assert_eq!(
            (result.payload_u128(), result.mask_xz_u128()),
            arithmetic_planes(ap, bp, width, kind),
            "{a:?}, {b:?}, {op:?}, width={width}"
        );
        let Expression::Binary(x, _, y, meta) = expr else {
            unreachable!()
        };
        for child in [x, y] {
            assert_eq!(
                (
                    child.comptime().expr_context.width,
                    child.comptime().expr_context.signed
                ),
                (width, signed)
            );
        }
        assert_eq!(
            (meta.expr_context.width, meta.expr_context.signed),
            (width, signed)
        );
        assert_eq!(result.width(), width);
    }
    for (op, kind) in [
        (Op::BitAnd, BitOp::And),
        (Op::BitOr, BitOp::Or),
        (Op::BitXor, BitOp::Xor),
        (Op::BitXnor, BitOp::Xnor),
    ] {
        let mut expr = node(leaf(a), op, leaf(b));
        let result = evaluate(&mut expr, width);
        let expected = reference_planes(width, |p| {
            binary(
                veryl_formal_semantics::logic4::read_logic(ap.0, ap.1, p),
                veryl_formal_semantics::logic4::read_logic(bp.0, bp.1, p),
                kind,
            )
        });
        assert_eq!((result.payload_u128(), result.mask_xz_u128()), expected);
    }
}

#[test]
fn context_four_state_arithmetic_and_gates() {
    for aw in 1..=2 {
        for bw in 1..=2 {
            for av in 0..(1u128 << (2 * aw)) {
                for bv in 0..(1u128 << (2 * bw)) {
                    for sa in [false, true] {
                        for sb in [false, true] {
                            let a = Operand {
                                planes: (av & ((1 << aw) - 1), av >> aw),
                                width: aw,
                                signed: sa,
                            };
                            let b = Operand {
                                planes: (bv & ((1 << bw) - 1), bv >> bw),
                                width: bw,
                                signed: sb,
                            };
                            check_binary(a, b, 0);
                            check_binary(a, b, 4);
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn context_native_biguint_boundaries() {
    for aw in [1, 4, 63, 64, 65, 128] {
        for bw in [1, 4, 63, 64, 65, 128] {
            let am = u128::MAX >> (128 - aw);
            let bm = u128::MAX >> (128 - bw);
            for ap in [(am, 0), (0, 1u128 << (aw - 1)), (am, am)] {
                for bp in [(1, 0), (bm, bm)] {
                    for sa in [false, true] {
                        for sb in [false, true] {
                            let a = Operand {
                                planes: ap,
                                width: aw,
                                signed: sa,
                            };
                            let b = Operand {
                                planes: bp,
                                width: bw,
                                signed: sb,
                            };
                            check_binary(a, b, 0);
                            check_binary(a, b, 128);
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn context_comparisons_ignore_unsigned_siblings() {
    for aw in [1, 4, 64, 65, 128] {
        for bw in [1, 4, 64, 65, 128] {
            for sa in [false, true] {
                for sb in [false, true] {
                    let a = Operand {
                        planes: (u128::MAX >> (128 - aw), 0),
                        width: aw,
                        signed: sa,
                    };
                    let b = Operand {
                        planes: (0, 0),
                        width: bw,
                        signed: sb,
                    };
                    for (op, kind) in [
                        (Op::Eq, Comparison::Equal),
                        (Op::Ne, Comparison::NotEqual),
                        (Op::EqWildcard, Comparison::WildcardEqual),
                        (Op::NeWildcard, Comparison::WildcardNotEqual),
                        (Op::Less, Comparison::Less),
                        (Op::LessEq, Comparison::LessEqual),
                        (Op::Greater, Comparison::Greater),
                        (Op::GreaterEq, Comparison::GreaterEqual),
                    ] {
                        let cmp = node(leaf(a), op, leaf(b));
                        let mut expr = node(
                            cmp,
                            Op::Add,
                            leaf(Operand {
                                planes: (0, 0),
                                width: 128,
                                signed: false,
                            }),
                        );
                        let result = evaluate(&mut expr, 128);
                        let expected = reference_planes(128, |p| {
                            if p == 0 {
                                compared_logic(a, b, kind)
                            } else {
                                veryl_formal_semantics::logic4::Logic4::Zero
                            }
                        });
                        assert_eq!((result.payload_u128(), result.mask_xz_u128()), expected);
                        let Expression::Binary(cmp, _, _, _) = expr else {
                            unreachable!()
                        };
                        let Expression::Binary(a, _, b, _) = *cmp else {
                            unreachable!()
                        };
                        for child in [a, b] {
                            assert_eq!(
                                (
                                    child.comptime().expr_context.width,
                                    child.comptime().expr_context.signed
                                ),
                                (aw.max(bw), sa && sb)
                            );
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn context_cast_boundaries_and_signedness() {
    for source in [1, 4, 63, 64, 65, 128] {
        for target in [1, 4, 63, 64, 65, 128] {
            for signed in [false, true] {
                for target_signed in [false, true] {
                    for typed in [false, true] {
                        let mask = u128::MAX >> (128 - source);
                        for planes in [(mask, 0), (mask, mask), (0, 1u128 << (source - 1))] {
                            let a = Operand {
                                planes,
                                width: source,
                                signed,
                            };
                            let target_expr = if typed {
                                let mut target_type = Type::new(TypeKind::Logic);
                                target_type.set_concrete_width(Shape::new(vec![Some(target)]));
                                target_type.signed = target_signed;
                                let mut meta_type = Type::new(TypeKind::Type);
                                meta_type.signed = target_signed;
                                Expression::Term(Box::new(Factor::Value(Comptime {
                                    value: ValueVariant::Type(target_type),
                                    r#type: meta_type,
                                    is_const: true,
                                    is_global: true,
                                    ..Default::default()
                                })))
                            } else {
                                leaf(Operand {
                                    planes: (target as u128, 0),
                                    width: 32,
                                    signed: target_signed,
                                })
                            };
                            let cast = node(leaf(a), Op::As, target_expr);
                            let mut expr = node(
                                cast,
                                Op::Add,
                                leaf(Operand {
                                    planes: (0, 0),
                                    width: 128,
                                    signed: true,
                                }),
                            );
                            let result = evaluate(&mut expr, 128);
                            let Expression::Binary(cast, _, _, _) = expr else {
                                unreachable!()
                            };
                            let Expression::Binary(input, _, _, meta) = cast.as_ref() else {
                                unreachable!()
                            };
                            assert_eq!(
                                (
                                    input.comptime().expr_context.width,
                                    input.comptime().expr_context.signed
                                ),
                                (source.max(target), signed)
                            );
                            assert_eq!(meta.r#type.total_width(), Some(target));
                            assert_eq!(
                                meta.r#type.signed,
                                if typed { target_signed } else { signed }
                            );
                            let mut ctx = Context::default();
                            let cast_value = cast.eval_value(&mut ctx).unwrap();
                            assert_eq!(
                                (cast_value.payload_u128(), cast_value.mask_xz_u128()),
                                widened(a, target, signed)
                            );
                            assert_eq!(cast_value.width(), target);
                            let cast_signed = if typed { target_signed } else { signed };
                            let expected_cast = Operand {
                                planes: widened(a, target, signed),
                                width: target,
                                signed: cast_signed,
                            };
                            assert_eq!(
                                (result.payload_u128(), result.mask_xz_u128()),
                                arithmetic_planes(
                                    widened(expected_cast, 128, cast_signed),
                                    (0, 0),
                                    128,
                                    Arithmetic::Add
                                )
                            );
                            assert_eq!(result.width(), 128);
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn context_self_determined_operands_and_conditional_branches() {
    for width in [1, 4, 63, 64, 65, 127] {
        let mask = u128::MAX >> (128 - width);
        let a = Operand {
            planes: (mask, 0),
            width,
            signed: false,
        };
        let one = Operand {
            planes: (1, 0),
            width,
            signed: false,
        };
        for (op, expected) in [(Op::BitOr, 0), (Op::BitNor, 1)] {
            let sum = node(leaf(a), Op::Add, leaf(one));
            let reduced = Expression::Unary(op, Box::new(sum), Box::default());
            let mut expr = node(
                reduced,
                Op::Add,
                leaf(Operand {
                    planes: (0, 0),
                    width: 128,
                    signed: false,
                }),
            );
            let result = evaluate(&mut expr, 128);
            assert_eq!(
                (result.payload_u128(), result.mask_xz_u128()),
                (expected, 0)
            );
        }
        for op in [Op::LogicShiftL, Op::LogicShiftR] {
            let amount = node(leaf(a), Op::Add, leaf(one));
            let mut expr = node(leaf(one), op, amount);
            let result = evaluate(&mut expr, 128);
            assert_eq!((result.payload_u128(), result.mask_xz_u128()), (1, 0));
            let Expression::Binary(_, _, amount, _) = expr else {
                unreachable!()
            };
            assert_eq!(amount.comptime().expr_context.width, width);
        }
        let mut expr = Expression::Ternary(
            Box::new(leaf(Operand {
                planes: (1, 0),
                width: 1,
                signed: false,
            })),
            Box::new(node(leaf(a), Op::Add, leaf(one))),
            Box::new(leaf(Operand {
                planes: (0, 0),
                width: 128,
                signed: false,
            })),
            Box::default(),
        );
        let result = evaluate(&mut expr, 128);
        assert_eq!(
            (result.payload_u128(), result.mask_xz_u128()),
            (1u128 << width, 0)
        );
    }
}
