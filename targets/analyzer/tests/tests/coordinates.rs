use veryl_analyzer::ir::VarSelectOp;
use veryl_formal_semantics::coordinates::{self, Select};

#[test]
fn selection_endpoint_native_boundaries() {
    let values = [0, 1, 2, 3, 15, usize::MAX / 2, usize::MAX - 1, usize::MAX];
    for a in values {
        for b in values {
            for array in [false, true] {
                for (actual, reference) in [
                    (VarSelectOp::Colon, Select::Colon),
                    (VarSelectOp::PlusColon, Select::Plus),
                    (VarSelectOp::MinusColon, Select::Minus),
                    (VarSelectOp::Step, Select::Step),
                ] {
                    assert_eq!(
                        actual.checked_eval_value(a, b, array),
                        coordinates::checked_endpoints(reference, a, b, array),
                        "checked endpoint boundary semantics mismatch"
                    );
                    assert_eq!(
                        actual.eval_value(a, b, array),
                        coordinates::endpoints(reference, a, b, array),
                        "{actual:?}, {a}, {b}, array={array}"
                    );
                }
            }
        }
    }
}

use veryl_analyzer::conv::Context;
use veryl_analyzer::ir::{Expression, Shape, Type, TypeKind, VarIndex, VarSelect};
use veryl_analyzer::value::Value;

fn expression(x: i64) -> Expression {
    let mut expr = Expression::create_value(Value::new(x as u64, 64, true), Default::default());
    expr.comptime_mut().r#type.signed = true;
    expr
}

#[test]
fn packed_and_unpacked_prefix_intervals() {
    for scalar in [1, 3] {
        for array in [false, true] {
            for dimensions in [vec![2, 3], vec![2, 3, 4]] {
                let mut inner = Type::new(TypeKind::Logic);
                inner.width_mut().push(Some(scalar));
                let mut ty = Type::new(TypeKind::Enum(
                    veryl_analyzer::ir::TypeKindEnum {
                        id: veryl_analyzer::symbol::SymbolId(0),
                        r#type: Box::new(inner),
                    }
                    .into(),
                ));
                let shape = Shape::new(dimensions.iter().copied().map(Some).collect());
                if array {
                    ty.array = shape;
                } else {
                    ty.set_concrete_width(shape);
                }
                for dim in 1..=dimensions.len() {
                    let tail: usize = dimensions[dim..].iter().product::<usize>()
                        * if array { 1 } else { scalar };
                    let axis = dimensions[dim - 1];
                    for prefix in 0..dimensions[..dim - 1].iter().product::<usize>() {
                        let mut remaining = prefix;
                        let mut coordinates = vec![0; dim - 1];
                        for i in (0..dim - 1).rev() {
                            coordinates[i] = remaining % dimensions[i];
                            remaining /= dimensions[i];
                        }
                        for low in 0..axis {
                            for high in low..axis {
                                let count = high - low + 1;
                                let mut operations = vec![
                                    (
                                        VarSelectOp::Colon,
                                        if array { low } else { high },
                                        if array { high } else { low },
                                    ),
                                    (VarSelectOp::PlusColon, low, high - low + 1),
                                    (VarSelectOp::MinusColon, high, high - low + 1),
                                ];
                                if low % count == 0 {
                                    operations.push((VarSelectOp::Step, low / count, count));
                                }
                                for (op, base, bound) in operations {
                                    let mut expressions: Vec<_> =
                                        coordinates.iter().map(|x| expression(*x as i64)).collect();
                                    expressions.push(expression(base as i64));
                                    let select = VarSelect(
                                        expressions,
                                        Some((op, expression(bound as i64))),
                                    );
                                    let mut ctx = Context::default();
                                    let first = (prefix * axis + low) * tail;
                                    let last = (prefix * axis + high + 1) * tail - 1;
                                    let expected =
                                        if array { (first, last) } else { (last, first) };
                                    assert_eq!(
                                        select.eval_value(&mut ctx, &ty, array),
                                        Some(expected),
                                        "{dimensions:?}, dim={dim}, low={low}, high={high}, array={array}"
                                    );
                                    assert!(ctx.drain_errors().is_empty());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn unpacked_clipping_preserves_selected_positions() {
    let dimensions = Shape::new(vec![Some(2), Some(4), Some(3)]);
    for row in 0..=2 {
        for base in -2i64..=6 {
            for bound in 1i64..=5 {
                for op in [
                    VarSelectOp::Colon,
                    VarSelectOp::PlusColon,
                    VarSelectOp::MinusColon,
                    VarSelectOp::Step,
                ] {
                    let (first, count) = match op {
                        VarSelectOp::Colon if base >= 0 && bound >= base => {
                            (base, bound - base + 1)
                        }
                        VarSelectOp::Colon => continue,
                        VarSelectOp::PlusColon => (base, bound),
                        VarSelectOp::MinusColon => (base - bound + 1, bound),
                        VarSelectOp::Step => (base * bound, bound),
                    };
                    let mut index = VarIndex::new(vec![expression(row), expression(base)]);
                    index.range = Some(Box::new((op.clone(), expression(bound))));
                    let mut ctx = Context::default();
                    let actual = index.eval_selection(&mut ctx, dimensions.as_shape_ref());
                    let valid: Vec<_> = (0..count)
                        .filter_map(|result| {
                            let coordinate = first + result;
                            (row < 2 && (0..4).contains(&coordinate))
                                .then_some(((row * 4 + coordinate) * 3, result * 3))
                        })
                        .collect();
                    let expected = valid.first().map(|(source, result)| {
                        (*source as usize, *result as usize, valid.len() * 3)
                    });
                    assert_eq!(
                        actual.map(|r| (r.source_start, r.result_start, r.length)),
                        expected,
                        "row={row}, base={base}, bound={bound}, op={op:?}"
                    );
                    assert!(ctx.drain_errors().is_empty());
                }
            }
        }
    }
}

#[test]
fn packed_unpacked_bit_counts_reject_overflow() {
    for packed in [0, 1, 2, usize::MAX / 2, usize::MAX] {
        for array in [0, 1, 2, usize::MAX] {
            let mut ty = Type::new(TypeKind::Logic);
            ty.width_mut().push(Some(packed));
            ty.array.push(Some(array));
            assert_eq!(ty.total_width(), Some(packed));
            assert_eq!(
                ty.total_bits(),
                usize::try_from(packed as u128 * array as u128).ok()
            );
        }
    }
}

#[test]
fn scalar_kind_and_packed_width_reject_overflow() {
    use veryl_analyzer::ir::TypeKindEnum;
    use veryl_analyzer::symbol::SymbolId;
    for scalar in [0, 1, 2, usize::MAX / 2, usize::MAX] {
        for packed in [0, 1, 2, usize::MAX] {
            let mut scalar_type = Type::new(TypeKind::Logic);
            scalar_type.width_mut().push(Some(scalar));
            let mut ty = Type::new(TypeKind::Enum(
                TypeKindEnum {
                    id: SymbolId(0),
                    r#type: Box::new(scalar_type),
                }
                .into(),
            ));
            ty.width_mut().push(Some(packed));
            assert_eq!(
                ty.total_width(),
                usize::try_from(scalar as u128 * packed as u128).ok()
            );
        }
    }
}
