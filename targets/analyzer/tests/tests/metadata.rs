//! Native initialization and teardown for the bounded metadata contracts.
use veryl_analyzer::ir::{
    Comptime, ExpressionContext, Shape, Type, TypeKind, ValueVariant, WidthExpr,
};
use veryl_analyzer::value::{Value, ValueU64};

#[test]
fn scalar_type_copy_has_independent_dimensions() {
    for width in [1, 64, 65, usize::MAX] {
        for target in [1, 65, usize::MAX] {
            for concrete in [false, true] {
                for signed in [false, true] {
                    let mut source = Type::new(TypeKind::Logic);
                    source.signed = signed;
                    source.is_positive = !signed;
                    if concrete {
                        source.set_concrete_width(Shape::new(vec![Some(width)]));
                    } else {
                        *source.width_mut() = Shape::new(vec![Some(width)]);
                    }
                    let mut copy = source.clone();
                    copy.width_mut().as_mut_slice()[0] = Some(target);
                    copy.set_concrete_width(Shape::new(vec![Some(target)]));
                    copy.signed = !signed;
                    copy.is_positive = signed;
                    assert_eq!(source.width().as_slice(), &[Some(width)]);
                    assert_eq!(copy.width().as_slice(), &[Some(target)]);
                    assert!(matches!(copy.width_expr(), [WidthExpr::Concrete(w)] if *w == target));
                    copy.clear_width();
                    assert!(copy.width().is_empty() && copy.width_expr().is_empty());
                    assert_eq!((source.signed, source.is_positive), (signed, !signed));
                    assert_eq!((copy.signed, copy.is_positive), (!signed, signed));
                    assert_eq!(source.width().as_slice(), &[Some(width)]);
                    if concrete {
                        assert!(
                            matches!(source.width_expr(), [WidthExpr::Concrete(w)] if *w == width)
                        );
                    } else {
                        assert!(source.width_expr().is_empty());
                    }
                }
            }
        }
    }
}

#[test]
fn literal_metadata_copy_invalidates_only_the_classification_cache() {
    for width in 1..=4 {
        let mask = (1u64 << width) - 1;
        for payload in [0, mask] {
            for mask_xz in [0, mask] {
                for value_signed in [false, true] {
                    for type_signed in [false, true] {
                        for cached in [None, Some(false), Some(true)] {
                            for flags in 0..32 {
                                for context_width in [1, 8] {
                                    let mut source = Comptime::create_value(
                                        Value::U64(ValueU64 {
                                            payload,
                                            mask_xz,
                                            width,
                                            signed: value_signed,
                                        }),
                                        Default::default(),
                                    );
                                    source.r#type.kind = TypeKind::Logic;
                                    source.r#type.signed = type_signed;
                                    source.r#type.is_positive = flags & 16 != 0;
                                    source.is_const = flags & 1 != 0;
                                    source.is_global = flags & 2 != 0;
                                    source.runtime_dependent = flags & 4 != 0;
                                    source.evaluated = flags & 8 != 0;
                                    source.elaboration_const = cached;
                                    source.member_signed = cached;
                                    source.expr_context = ExpressionContext {
                                        width: context_width,
                                        signed: value_signed,
                                        is_const: flags & 2 != 0,
                                        is_global: flags & 1 != 0,
                                    };
                                    let mut copy = source.clone();
                                    assert_eq!(copy.elaboration_const, None);
                                    assert_eq!(copy.member_signed, cached);
                                    assert_eq!(
                                        (
                                            copy.is_const,
                                            copy.is_global,
                                            copy.runtime_dependent,
                                            copy.evaluated
                                        ),
                                        (
                                            flags & 1 != 0,
                                            flags & 2 != 0,
                                            flags & 4 != 0,
                                            flags & 8 != 0
                                        )
                                    );
                                    assert_eq!(
                                        (copy.r#type.signed, copy.r#type.is_positive),
                                        (type_signed, flags & 16 != 0)
                                    );
                                    assert_eq!(
                                        (
                                            copy.expr_context.width,
                                            copy.expr_context.signed,
                                            copy.expr_context.is_const,
                                            copy.expr_context.is_global
                                        ),
                                        (
                                            context_width,
                                            value_signed,
                                            flags & 2 != 0,
                                            flags & 1 != 0
                                        )
                                    );
                                    assert!(
                                        matches!(&copy.value, ValueVariant::Numeric(Value::U64(v))
                                        if v.width == width && v.payload == payload && v.mask_xz == mask_xz && v.signed == value_signed)
                                    );
                                    if let ValueVariant::Numeric(Value::U64(v)) = &mut copy.value {
                                        v.payload ^= 1;
                                        v.mask_xz ^= 1;
                                        v.signed = !value_signed;
                                    }
                                    copy.r#type.set_concrete_width(Shape::new(vec![Some(65)]));
                                    copy.is_const = !source.is_const;
                                    copy.elaboration_const = Some(true);
                                    assert!(
                                        matches!(&source.value, ValueVariant::Numeric(Value::U64(v))
                                        if v.width == width && v.payload == payload && v.mask_xz == mask_xz && v.signed == value_signed)
                                    );
                                    assert_eq!(
                                        source.r#type.width().as_slice(),
                                        &[Some(width as usize)]
                                    );
                                    assert!(source.r#type.width_expr().is_empty());
                                    assert_eq!(source.elaboration_const, cached);
                                    assert_eq!(source.is_const, flags & 1 != 0);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
