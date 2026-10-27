//! Finite numeric caller intervals, limit diagnostics, checked rebasing, and empty
//! metadata. The oracle enumerates positions independently. No general recursive
//! AST, dynamic signed/X/Z selector decoding, or backend-byte-address proof.

use veryl_analyzer::conv::Context;
use veryl_analyzer::ir::{
    Expression, PartSelectPath, Shape, Type, TypeKind, TypeKindEnum, TypeKindMember,
    TypeKindStruct, VarPath, VarSelect, VarSelectOp,
};
use veryl_analyzer::symbol::SymbolId;
use veryl_analyzer::value::Value;
use veryl_formal_semantics::coordinates::{self, Select};

fn value(x: usize) -> Expression {
    Expression::create_value(Value::new(x as u64, 64, false), Default::default())
}
fn ty(shape: &[Option<usize>], element: usize, array: bool) -> Type {
    let mut scalar = Type::new(TypeKind::Logic);
    scalar.width_mut().push(Some(element));
    let mut ty = Type::new(TypeKind::Enum(
        TypeKindEnum {
            id: SymbolId(0),
            r#type: Box::new(scalar),
        }
        .into(),
    ));
    if array {
        ty.array = Shape::new(shape.to_vec());
    } else {
        ty.set_concrete_width(Shape::new(shape.to_vec()));
    }
    ty
}
fn selection(coords: &[usize], range: Option<(Select, usize)>) -> VarSelect {
    let op = |op| match op {
        Select::Colon => VarSelectOp::Colon,
        Select::Plus => VarSelectOp::PlusColon,
        Select::Minus => VarSelectOp::MinusColon,
        Select::Step => VarSelectOp::Step,
    };
    VarSelect(
        coords.iter().copied().map(value).collect(),
        range.map(|(o, b)| (op(o), value(b))),
    )
}
fn check(
    shape: &[Option<usize>],
    element: usize,
    coords: &[usize],
    range: Option<(Select, usize)>,
    array: bool,
    limit: usize,
) {
    let mut ctx = Context::default();
    ctx.config.evaluate_size_limit = limit;
    let expected = coordinates::selection_interval(
        shape,
        if array { 1 } else { element },
        coords,
        range,
        array,
        limit,
    );
    let actual = selection(coords, range).eval_value(&mut ctx, &ty(shape, element, array), array);
    assert_eq!(
        actual, expected,
        "selection layout semantics mismatch: shape={shape:?}, element={element}, coords={coords:?}, range={range:?}, array={array}, limit={limit}"
    );
    let needs_limit_error = expected.is_none()
        && coordinates::selection_interval(
            shape,
            if array { 1 } else { element },
            coords,
            range,
            array,
            usize::MAX,
        )
        .is_some();
    assert_eq!(
        ctx.drain_errors().is_empty(),
        !needs_limit_error,
        "selection layout semantics mismatch: limit diagnostics"
    );
}
#[test]
fn selection_storage_boundaries() {
    for array in [false, true] {
        for shape in [
            vec![Some(0)],
            vec![Some(2), Some(0)],
            vec![None],
            vec![Some(usize::MAX), Some(2)],
            vec![Some(usize::MAX)],
            vec![Some(2), Some(3)],
            vec![],
        ] {
            for coords in [
                vec![],
                vec![0],
                vec![1],
                vec![usize::MAX - 1],
                vec![usize::MAX],
                vec![0, 0],
            ] {
                for range in [
                    None,
                    Some((Select::Plus, 1)),
                    Some((Select::Plus, 2)),
                    Some((Select::Minus, 2)),
                    Some((Select::Step, 2)),
                    Some((Select::Colon, 0)),
                ] {
                    check(&shape, 1, &coords, range, array, usize::MAX);
                }
            }
        }
    }
}
#[test]
fn selection_small_coordinate_geometry() {
    for array in [false, true] {
        for element in [1, 3] {
            for shape in [
                vec![],
                vec![Some(1)],
                vec![Some(2), Some(3)],
                vec![Some(2), Some(3), Some(2)],
            ] {
                for arity in 0..=shape.len() + 1 {
                    for code in 0..4usize.pow(arity as u32) {
                        let mut code = code;
                        let coords: Vec<_> = (0..arity)
                            .map(|_| {
                                let x = code % 4;
                                code /= 4;
                                x
                            })
                            .collect();
                        check(&shape, element, &coords, None, array, usize::MAX);
                        for op in [Select::Colon, Select::Plus, Select::Minus, Select::Step] {
                            for bound in 0..=3 {
                                check(
                                    &shape,
                                    element,
                                    &coords,
                                    Some((op, bound)),
                                    array,
                                    usize::MAX,
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
fn selection_limits_and_scalar_offsets() {
    for array in [false, true] {
        for shape in [vec![], vec![Some(4)], vec![Some(2), Some(4)]] {
            for limit in [0, 1, 3, 4, 7, 8] {
                for coords in [
                    vec![],
                    vec![0],
                    vec![3],
                    vec![4],
                    vec![usize::MAX],
                    vec![1, 3],
                ] {
                    check(&shape, 1, &coords, None, array, limit);
                }
            }
        }
    }
}
#[test]
fn rebased_selection_rejects_empty_storage_and_missing_range_base() {
    for shape in [
        vec![Some(0)],
        vec![Some(2), Some(0)],
        vec![None],
        vec![Some(usize::MAX), Some(2)],
    ] {
        let path = PartSelectPath {
            base: ty(&shape, 1, false),
            path: VarPath::default(),
            part_select: vec![],
        };
        for select in [VarSelect::default(), selection(&[0], None)] {
            assert!(
                path.to_base_select(&mut Context::default(), &select)
                    .is_none(),
                "rebased selection storage semantics mismatch"
            );
        }
    }
    let aggregate = Type::new(TypeKind::Struct(
        TypeKindStruct {
            id: SymbolId(0),
            members: vec![
                TypeKindMember {
                    name: Default::default(),
                    r#type: ty(&[Some(0)], 1, false),
                },
                TypeKindMember {
                    name: Default::default(),
                    r#type: ty(&[Some(1)], 1, false),
                },
            ],
        }
        .into(),
    ));
    let fields = aggregate.expand_struct_union(&VarPath::default(), &[], None);
    assert!(
        fields[0]
            .to_base_select(&mut Context::default(), &VarSelect::default())
            .is_none(),
        "rebased selection storage semantics mismatch"
    );
    let path = PartSelectPath {
        base: ty(&[Some(4)], 1, false),
        path: VarPath::default(),
        part_select: vec![],
    };
    assert!(
        path.to_base_select(
            &mut Context::default(),
            &selection(&[], Some((Select::Colon, 0)))
        )
        .is_none(),
        "rebased selection storage semantics mismatch"
    );
    assert!(
        path.to_base_select(
            &mut Context::default(),
            &selection(&[1], Some((Select::Plus, 0)))
        )
        .is_none(),
        "rebased selection storage semantics mismatch"
    );
}

#[test]
fn selection_metadata_widths_do_not_wrap() {
    for shape in [vec![], vec![None]] {
        let base = ty(&shape, 4, false);
        assert!(
            selection(&[usize::MAX], Some((Select::Colon, 0)))
                .eval_comptime(&mut Context::default(), &base, false)
                .is_none(),
            "selection metadata semantics mismatch"
        );
        for op in [Select::Plus, Select::Minus, Select::Step] {
            let mut ctx = Context::default();
            assert!(
                selection(&[0], Some((op, 0)))
                    .eval_comptime(&mut ctx, &base, false)
                    .is_none(),
                "selection metadata semantics mismatch"
            );
            assert!(
                !ctx.drain_errors().is_empty(),
                "selection metadata semantics mismatch: zero width diagnostics"
            );
        }
        assert!(
            selection(&[], Some((Select::Colon, 0)))
                .eval_comptime(&mut Context::default(), &base, false)
                .is_none(),
            "selection metadata semantics mismatch"
        );
    }
    let base = ty(&[None], 1, false);
    assert_eq!(
        selection(&[3], Some((Select::Plus, 2)))
            .eval_comptime(&mut Context::default(), &base, false)
            .unwrap(),
        Shape::new(vec![Some(2)]),
        "selection metadata semantics mismatch"
    );
}

#[test]
fn unpacked_selection_rejects_empty_or_unrepresentable_storage() {
    use veryl_analyzer::ir::VarIndex;
    for dims in [
        vec![Some(0)],
        vec![Some(2), Some(0)],
        vec![None],
        vec![Some(usize::MAX), Some(2)],
    ] {
        let shape = Shape::new(dims);
        for coords in [vec![], vec![0], vec![0, 0]] {
            for range in [None, Some((VarSelectOp::PlusColon, value(1)))] {
                let mut index = VarIndex::new(coords.iter().copied().map(value).collect());
                index.range = range.map(Box::new);
                assert!(
                    index
                        .eval_selection(&mut Context::default(), shape.as_shape_ref())
                        .is_none(),
                    "unpacked selection storage semantics mismatch"
                );
            }
        }
    }
}
