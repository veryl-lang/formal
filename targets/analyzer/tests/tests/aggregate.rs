//! Finite flat and bounded nested struct/union layout comparisons from resolved
//! widths and independent coordinates. Union alternatives are equal-width.
//! No aggregate Kani theorem or arbitrary unresolved recursive-layout claim.

use std::sync::Arc;
use veryl_analyzer::conv::Context;
use veryl_analyzer::ir::{
    Expression, PartSelectPath, Shape, Type, TypeKind, TypeKindMember, TypeKindStruct,
    TypeKindUnion, VarPath, VarSelect, VarSelectOp,
};
use veryl_analyzer::symbol::SymbolId;
use veryl_analyzer::value::Value;
use veryl_formal_semantics::aggregate;

fn bits(width: Option<usize>) -> Type {
    let mut ty = Type::new(TypeKind::Logic);
    ty.width_mut().push(width);
    ty
}
fn member(name: usize, ty: Type) -> TypeKindMember {
    let mut ret = TypeKindMember {
        name: Default::default(),
        r#type: ty,
    };
    ret.name.0 = name;
    ret
}
fn structure(members: Vec<TypeKindMember>, count: usize) -> Type {
    let mut ty = Type::new(TypeKind::Struct(Arc::new(TypeKindStruct {
        id: SymbolId(0),
        members,
    })));
    ty.width_mut().push(Some(count));
    ty
}
fn union(members: Vec<TypeKindMember>, count: usize) -> Type {
    let mut ty = Type::new(TypeKind::Union(Arc::new(TypeKindUnion {
        id: SymbolId(0),
        members,
    })));
    ty.width_mut().push(Some(count));
    ty
}
fn numeric(x: usize) -> Expression {
    Expression::create_value(Value::new(x as u64, 64, false), Default::default())
}
fn interval(path: &PartSelectPath, coords: &[usize]) -> Option<(usize, usize)> {
    interval_range(path, coords, None)
}
fn interval_range(
    path: &PartSelectPath,
    coords: &[usize],
    range: Option<(VarSelectOp, usize)>,
) -> Option<(usize, usize)> {
    let mut ctx = Context::default();
    let select = VarSelect(
        coords.iter().map(|x| numeric(*x)).collect(),
        range.map(|(op, bound)| (op, numeric(bound))),
    );
    let flat = path.to_base_select(&mut ctx, &select)?;
    let ret = flat.eval_value(&mut ctx, &Type::default(), false);
    assert!(ctx.drain_errors().is_empty());
    ret
}

#[test]
fn struct_width_native_boundaries() {
    for a in [0, 1, 2, usize::MAX / 2, usize::MAX] {
        for b in [0, 1, 2, usize::MAX] {
            let ty = structure(vec![member(1, bits(Some(a))), member(2, bits(Some(b)))], 1);
            assert_eq!(
                ty.total_width(),
                aggregate::struct_width(&[Some(a), Some(b)]),
                "aggregate width semantics mismatch: {a}, {b}"
            );
        }
    }
    let ty = structure(vec![member(1, bits(None)), member(2, bits(Some(0)))], 1);
    assert_eq!(ty.total_width(), None);
    assert_eq!(structure(vec![], 1).total_width(), Some(0));
}

#[test]
fn flat_struct_and_union_spans() {
    for a in 0..=4 {
        for b in 0..=4 {
            for c in 0..=4 {
                let widths = [Some(a), Some(b), Some(c)];
                let ty = structure(
                    vec![
                        member(1, bits(Some(a))),
                        member(2, bits(Some(b))),
                        member(3, bits(Some(c))),
                    ],
                    1,
                );
                let paths = ty.expand_struct_union(&VarPath::default(), &[], None);
                assert_eq!(paths.len(), 3);
                assert_eq!(ty.kind.width(), aggregate::struct_width(&widths));
                for (i, path) in paths.iter().enumerate() {
                    let expected = aggregate::member_span(&widths, i, false).unwrap();
                    assert_eq!(
                        (
                            path.part_select[0].pos,
                            path.part_select[0].r#type.total_width().unwrap()
                        ),
                        expected,
                        "aggregate member span semantics mismatch"
                    );
                    assert_eq!(
                        path.path.0[0].0,
                        i + 1,
                        "aggregate member span semantics mismatch"
                    );
                }
            }
        }
        let widths = [Some(a); 3];
        let ty = union((1..=3).map(|name| member(name, bits(Some(a)))).collect(), 1);
        let paths = ty.expand_struct_union(&VarPath::default(), &[], None);
        assert_eq!(paths.len(), 3);
        assert_eq!(ty.kind.width(), Some(a));
        for (i, path) in paths.iter().enumerate() {
            assert_eq!(
                (
                    path.part_select[0].pos,
                    path.part_select[0].r#type.total_width().unwrap()
                ),
                aggregate::member_span(&widths, i, true).unwrap(),
                "aggregate member span semantics mismatch"
            );
        }
    }
}

#[test]
fn nested_struct_union_packed_and_unpacked_coordinates() {
    for a in 1..=3 {
        for b in 1..=3 {
            for p in 1..=3 {
                let inner = structure(vec![member(3, bits(Some(a))), member(4, bits(Some(b)))], p);
                let view = union(
                    vec![member(2, inner), member(5, bits(Some((a + b) * p)))],
                    2,
                );
                let mut base = structure(
                    vec![
                        member(6, bits(Some(1))),
                        member(1, view),
                        member(7, bits(Some(4))),
                    ],
                    3,
                );
                base.array = Shape::new(vec![Some(2), Some(3)]);
                let paths = base.expand_struct_union(&VarPath::default(), &[], None);
                let path = paths
                    .iter()
                    .find(|path| path.path.0.iter().map(|x| x.0).collect::<Vec<_>>() == [1, 2, 3])
                    .unwrap();
                let view = paths
                    .iter()
                    .find(|path| path.path.0.iter().map(|x| x.0).collect::<Vec<_>>() == [1, 2])
                    .unwrap();
                let raw = paths
                    .iter()
                    .find(|path| path.path.0.iter().map(|x| x.0).collect::<Vec<_>>() == [1, 5])
                    .unwrap();
                assert_eq!(path.base.array.total(), Some(6));
                let width = 1 + 2 * p * (a + b) + 4;
                for outer in 0..3 {
                    for alternative in 0..2 {
                        let low = outer * width + 4 + alternative * p * (a + b);
                        assert_eq!(
                            interval(raw, &[outer, alternative]),
                            Some((low + p * (a + b) - 1, low)),
                            "aggregate nested coordinate semantics mismatch: a={a}, b={b}, p={p}, outer={outer}, alternative={alternative}, parts={:?}",
                            raw.part_select
                                .iter()
                                .map(|x| (
                                    x.pos,
                                    x.r#type.total_width(),
                                    x.r#type.kind.width(),
                                    x.r#type.width().as_shape_ref().to_owned()
                                ))
                                .collect::<Vec<_>>()
                        );
                        for first in 0..p {
                            for last in first..p {
                                let count = last - first + 1;
                                let mut ops = vec![
                                    (VarSelectOp::Colon, last, first),
                                    (VarSelectOp::PlusColon, first, count),
                                    (VarSelectOp::MinusColon, last, count),
                                ];
                                if first % count == 0 {
                                    ops.push((VarSelectOp::Step, first / count, count));
                                }
                                for (op, base, bound) in ops {
                                    assert_eq!(
                                        interval_range(
                                            view,
                                            &[outer, alternative, base],
                                            Some((op.clone(), bound))
                                        ),
                                        Some((
                                            low + (last + 1) * (a + b) - 1,
                                            low + first * (a + b)
                                        )),
                                        "aggregate nested coordinate semantics mismatch: a={a}, b={b}, p={p}, outer={outer}, alternative={alternative}, first={first}, last={last}, op={op:?}"
                                    );
                                }
                            }
                        }
                        for inner in 0..p {
                            let low =
                                outer * width + 4 + alternative * p * (a + b) + inner * (a + b) + b;
                            assert_eq!(
                                interval(path, &[outer, alternative, inner]),
                                Some((low + a - 1, low)),
                                "aggregate nested coordinate semantics mismatch"
                            );
                            for bit in 0..a {
                                assert_eq!(
                                    interval(path, &[outer, alternative, inner, bit]),
                                    Some((low + bit, low + bit)),
                                    "aggregate nested coordinate semantics mismatch"
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn aggregate_invalid_storage_does_not_wrap() {
    for widths in [
        [Some(usize::MAX), Some(1)],
        [Some(1), Some(usize::MAX)],
        [None, Some(usize::MAX)],
        [Some(usize::MAX), None],
    ] {
        let base = structure(
            widths
                .iter()
                .enumerate()
                .map(|(i, w)| member(i + 1, bits(*w)))
                .collect(),
            1,
        );
        assert!(base.total_width().is_none());
        assert!(
            base.expand_struct_union(&VarPath::default(), &[], None)
                .is_empty(),
            "aggregate invalid storage semantics mismatch"
        );
    }
    let unresolved = structure(vec![member(1, bits(None)), member(2, bits(Some(2)))], 1);
    let paths = unresolved.expand_struct_union(&VarPath::default(), &[], None);
    assert_eq!(
        paths.len(),
        2,
        "aggregate invalid storage semantics mismatch"
    );
    for path in paths {
        assert!(
            path.to_base_select(&mut Context::default(), &VarSelect::default())
                .is_none(),
            "aggregate invalid storage semantics mismatch"
        );
    }
    let inner = structure(
        vec![member(3, bits(Some(usize::MAX))), member(4, bits(Some(1)))],
        1,
    );
    let nested = structure(vec![member(1, inner), member(2, bits(Some(1)))], 1);
    for path in nested.expand_struct_union(&VarPath::default(), &[], None) {
        assert!(
            path.to_base_select(&mut Context::default(), &VarSelect::default())
                .is_none(),
            "aggregate invalid storage semantics mismatch"
        );
    }
}
