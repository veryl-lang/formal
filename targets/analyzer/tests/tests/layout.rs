//! ShapeRef index/range domains are proved in targets/analyzer/kani/src/layout.rs.
//! Keep owned geometry, wider boundaries, expression coordinates, and model anchors.

use veryl_analyzer::conv::Context;
use veryl_analyzer::ir::{Expression, Shape, ShapeRef};
use veryl_analyzer::value::Value;
use veryl_formal_semantics::layout;

#[test]
fn shape_exhaustive_small_geometry() {
    for rank in 0u32..=4 {
        for code in 0..6usize.pow(rank) {
            let mut code = code;
            let dimensions: Vec<_> = (0..rank)
                .map(|_| {
                    let digit = code % 6;
                    code /= 6;
                    if digit == 5 { None } else { Some(digit) }
                })
                .collect();
            let shape = Shape::new(dimensions.clone());
            assert_eq!(shape.total(), layout::total(&dimensions));
            for arity in 0u32..=4 {
                for code in 0..5usize.pow(arity) {
                    let mut code = code;
                    let indices: Vec<_> = (0..arity)
                        .map(|_| {
                            let i = code % 5;
                            code /= 5;
                            i
                        })
                        .collect();
                    assert_eq!(
                        shape.calc_index(&indices),
                        layout::index(&dimensions, &indices),
                        "{dimensions:?}, {indices:?}"
                    );
                    if rank <= 3 {
                        assert_eq!(
                            layout::range_three(&dimensions, &indices),
                            layout::range(&dimensions, &indices)
                        );
                    }
                    assert_eq!(
                        shape.calc_range(&indices),
                        layout::range(&dimensions, &indices),
                        "{dimensions:?}, {indices:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn shape_native_boundaries_and_overflow() {
    for dimensions in [
        vec![],
        vec![Some(1)],
        vec![Some(0)],
        vec![None],
        vec![Some(0), None],
        vec![None, Some(0)],
        vec![Some(usize::MAX), Some(1)],
        vec![Some(1), Some(usize::MAX)],
        vec![Some(usize::MAX), Some(2)],
        vec![Some(usize::MAX), Some(2), Some(0)],
        vec![Some(usize::MAX), None, Some(0)],
    ] {
        let shape = Shape::new(dimensions.clone());
        assert_eq!(shape.total(), layout::total(&dimensions));
        for indices in [
            vec![],
            vec![0],
            vec![usize::MAX],
            vec![usize::MAX - 1],
            vec![0, 0],
            vec![usize::MAX - 1, 0],
            vec![0, usize::MAX - 1],
            vec![usize::MAX, 0],
            vec![0, 0, 0],
        ] {
            assert_eq!(
                shape.calc_index(&indices),
                layout::index(&dimensions, &indices),
                "{dimensions:?}, {indices:?}"
            );
            assert_eq!(
                shape.calc_range(&indices),
                layout::range(&dimensions, &indices),
                "{dimensions:?}, {indices:?}"
            );
        }
    }
    assert_eq!(
        ShapeRef::new(&[Some(usize::MAX), Some(2), Some(0)]).total(),
        Some(0)
    );
    assert_eq!(ShapeRef::new(&[Some(2), Some(3)]).calc_index(&[0, 3]), None);
    assert_eq!(ShapeRef::new(&[Some(0)]).calc_range(&[]), None);
}

#[test]
fn shape_reference_row_major_storage_enumeration() {
    let dimensions = [Some(2), Some(3), Some(4)];
    let mut flat = 0;
    for a in 0..2 {
        for b in 0..3 {
            assert_eq!(layout::range(&dimensions, &[a, b]), Some((flat, flat + 3)));
            assert_eq!(
                layout::range_three(&dimensions, &[a, b]),
                Some((flat, flat + 3))
            );
            for c in 0..4 {
                assert_eq!(layout::index(&dimensions, &[a, b, c]), Some(flat));
                flat += 1;
            }
        }
    }
    assert_eq!(layout::total(&dimensions), Some(flat));
    assert_eq!(layout::range(&dimensions, &[]), Some((0, flat - 1)));
    assert_eq!(layout::range_three(&dimensions, &[]), Some((0, flat - 1)));
}

#[test]
fn shape_expression_indices_match_numeric_coordinates() {
    for dimensions in [
        vec![],
        vec![Some(1)],
        vec![Some(2), Some(3)],
        vec![Some(2), Some(3), Some(4)],
        vec![Some(2); 4],
    ] {
        let shape = Shape::new(dimensions.clone());
        for last in [false, true] {
            let indices: Vec<_> = dimensions
                .iter()
                .map(|n| if last { n.unwrap() - 1 } else { 0 })
                .collect();
            let expressions: Vec<_> = indices
                .iter()
                .map(|i| {
                    Expression::create_value(Value::new(*i as u64, 64, false), Default::default())
                })
                .collect();
            let mut expression = shape.calc_index_expr(&expressions).unwrap();
            let mut ctx = Context::default();
            let value = expression
                .eval_comptime(&mut ctx, Some(64))
                .get_value()
                .unwrap()
                .to_usize();
            assert_eq!(value, shape.calc_index(&indices));
            assert!(ctx.drain_errors().is_empty());
        }
    }
}

#[test]
fn shape_expression_rejects_unresolved_empty_and_overflowing_storage() {
    for dimensions in [vec![None], vec![Some(0)], vec![Some(usize::MAX), Some(2)]] {
        let shape = Shape::new(dimensions.clone());
        let indices: Vec<_> = dimensions
            .iter()
            .map(|_| Expression::create_value(Value::new(0, 32, false), Default::default()))
            .collect();
        assert!(shape.calc_index_expr(&indices).is_none());
    }
}
