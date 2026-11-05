//! Internal metadata contracts, not source-language typing or AST evaluation.
//! Type: Bit/Logic scalars, one extent in 1..=usize::MAX, arbitrary signed/positive
//! flags, and empty or matching concrete deferred width. Actual clone, independent
//! shape edits, set_concrete_width and clear_width execute. Arrays stay empty.
//! Comptime: Logic/U64 widths 1..4, normalized arbitrary four-state planes,
//! independent type/value signs and metadata flags, contexts 1..8, all optional
//! classification/member-sign states. Clone invalidates elaboration_const;
//! edits to the copy preserve the source's observed value/type/metadata.
//! Part selects/member domains are absent, clocks/tokens are default. No allocator,
//! refcount/pointer identity, diagnostics, concurrency, panic recovery or teardown
//! guarantee. Actual LazyLock/Arc initialization executes; the normal catch summary
//! runs its closure and every panic still fails. BigUint/part-select clones fail
//! closed. The native metadata tests exercise normal initialization and teardown.
use std::panic::catch_unwind as catching;
use veryl_analyzer::{
    ir::{
        Comptime, ExpressionContext, PartSelectPath, Shape, Type, TypeKind, ValueVariant, WidthExpr,
    },
    symbol::ClockDomain,
    value::{Value, ValueBigUint, ValueU64},
};
fn normal_catch_unwind<F: FnOnce() -> R + std::panic::UnwindSafe, R>(
    f: F,
) -> std::thread::Result<R> {
    Ok(f())
}
fn arbitrary_positive_width() -> usize {
    let w: usize = kani::any();
    kani::assume(w > 0);
    w
}
fn scalar_type(width: usize, logic: bool, signed: bool, positive: bool, concrete: bool) -> Type {
    let mut t = Type::new(if logic {
        TypeKind::Logic
    } else {
        TypeKind::Bit
    });
    t.signed = signed;
    t.is_positive = positive;
    if concrete {
        t.set_concrete_width(Shape::new(vec![Some(width)]));
    } else {
        *t.width_mut() = Shape::new(vec![Some(width)]);
    }
    t
}
fn type_matches(
    t: &Type,
    width: usize,
    logic: bool,
    signed: bool,
    positive: bool,
    concrete: bool,
) -> bool {
    matches!(
        (&t.kind, logic),
        (TypeKind::Logic, true) | (TypeKind::Bit, false)
    ) && t.signed == signed
        && t.is_positive == positive
        && t.width().as_slice() == [Some(width)]
        && t.array.is_empty()
        && t.array_expr().is_empty()
        && if concrete {
            matches!(t.width_expr(), [WidthExpr::Concrete(w)] if *w == width)
        } else {
            t.width_expr().is_empty()
        }
}
fn assert_type(t: &Type, width: usize, logic: bool, signed: bool, positive: bool, concrete: bool) {
    assert!(
        type_matches(t, width, logic, signed, positive, concrete),
        "resolved type semantics mismatch"
    );
}
#[kani::proof]
#[kani::unwind(3)]
#[kani::stub(catching, normal_catch_unwind)]
fn resolved_type_clone_update_matches_reference() {
    let w = arbitrary_positive_width();
    let raw_target = arbitrary_positive_width();
    let target = arbitrary_positive_width();
    let logic = kani::any();
    let signed = kani::any();
    let positive = kani::any();
    let concrete = kani::any();
    let t = scalar_type(w, logic, signed, positive, concrete);
    let mut copy = t.clone();
    assert_type(&copy, w, logic, signed, positive, concrete);
    copy.width_mut().as_mut_slice()[0] = Some(raw_target);
    assert!(
        copy.width().as_slice() == [Some(raw_target)],
        "resolved type semantics mismatch"
    );
    assert_type(&t, w, logic, signed, positive, concrete);
    copy.set_concrete_width(Shape::new(vec![Some(target)]));
    copy.signed = !signed;
    copy.is_positive = !positive;
    assert_type(&copy, target, logic, !signed, !positive, true);
    assert_type(&t, w, logic, signed, positive, concrete);
    kani::cover!(concrete && w < target, "cloned concrete width can grow");
    kani::cover!(concrete && w > target, "cloned concrete width can shrink");
    kani::cover!(
        !concrete && w != target,
        "empty deferred clone can change width"
    );
    kani::cover!(
        logic && signed && positive,
        "signed positive logic metadata is reachable"
    );
    kani::cover!(
        !logic && !signed && !positive,
        "unsigned bit metadata is reachable"
    );
    copy.clear_width();
    assert!(
        copy.width().is_empty()
            && copy.width_expr().is_empty()
            && copy.array.is_empty()
            && copy.array_expr().is_empty()
            && matches!(
                (&copy.kind, logic),
                (TypeKind::Logic, true) | (TypeKind::Bit, false)
            )
            && copy.signed == !signed
            && copy.is_positive == !positive,
        "resolved type semantics mismatch"
    );
    assert_type(&t, w, logic, signed, positive, concrete);
    kani::cover!(
        w == usize::MAX && target == 1,
        "maximum source width can narrow"
    );
    kani::cover!(
        w == 1 && target == usize::MAX,
        "minimum source width can widen"
    );
    kani::cover!(
        w == 64 && raw_target == 63 && target == 65,
        "native boundary updates are reachable"
    );
    std::mem::forget(copy);
    std::mem::forget(t);
}

fn reject_big_clone(_: &ValueBigUint) -> ValueBigUint {
    panic!("BigUint metadata is outside scope");
}
fn reject_part_clone(_: &PartSelectPath) -> PartSelectPath {
    panic!("part-select metadata is outside scope");
}
#[derive(Clone, Copy)]
struct MetadataSnapshot {
    width: usize,
    payload: u64,
    mask: u64,
    value_signed: bool,
    type_signed: bool,
    positive: bool,
    constant: bool,
    global: bool,
    runtime: bool,
    evaluated: bool,
    cached: Option<bool>,
    member_signed: Option<bool>,
    context: ExpressionContext,
}
fn arbitrary_cached_flag() -> Option<bool> {
    let kind = kani::any::<u8>() % 3;
    match kind {
        0 => None,
        1 => Some(false),
        _ => Some(true),
    }
}
fn assert_stamp(m: &Comptime, s: MetadataSnapshot, cached: Option<bool>) {
    assert!(
        m.is_const == s.constant
            && m.is_global == s.global
            && m.runtime_dependent == s.runtime
            && m.evaluated == s.evaluated
            && m.elaboration_const == cached
            && m.member_signed == s.member_signed
            && m.part_select.is_none()
            && m.member_select_domain.is_none()
            && matches!(m.clock_domain, ClockDomain::None)
            && m.expr_context.width == s.context.width
            && m.expr_context.signed == s.context.signed
            && m.expr_context.is_const == s.context.is_const
            && m.expr_context.is_global == s.context.is_global,
        "metadata clone semantics mismatch"
    );
    assert!(
        type_matches(&m.r#type, s.width, true, s.type_signed, s.positive, false),
        "metadata clone semantics mismatch"
    );
    assert!(
        matches!(&m.value,ValueVariant::Numeric(Value::U64(v)) if v.width as usize==s.width&&v.payload==s.payload&&v.mask_xz==s.mask&&v.signed==s.value_signed),
        "metadata clone semantics mismatch"
    );
}
#[kani::proof]
#[kani::unwind(3)]
#[kani::stub(catching, normal_catch_unwind)]
#[kani::stub(<veryl_analyzer::value::ValueBigUint as std::clone::Clone>::clone,reject_big_clone)]
#[kani::stub(<veryl_analyzer::ir::PartSelectPath as std::clone::Clone>::clone,reject_part_clone)]
fn literal_metadata_clone_matches_reference() {
    let w = usize::from(kani::any::<u8>() & 3) + 1;
    let s = MetadataSnapshot {
        width: w,
        payload: u64::from(kani::any::<u8>()) & ((1 << w) - 1),
        mask: u64::from(kani::any::<u8>()) & ((1 << w) - 1),
        value_signed: kani::any(),
        type_signed: kani::any(),
        positive: kani::any(),
        constant: kani::any(),
        global: kani::any(),
        runtime: kani::any(),
        evaluated: kani::any(),
        cached: arbitrary_cached_flag(),
        member_signed: arbitrary_cached_flag(),
        context: ExpressionContext {
            width: usize::from(kani::any::<u8>() & 7) + 1,
            signed: kani::any(),
            is_const: kani::any(),
            is_global: kani::any(),
        },
    };
    let mut original = Comptime::create_value(
        Value::U64(ValueU64 {
            payload: s.payload,
            mask_xz: s.mask,
            width: w as u32,
            signed: s.value_signed,
        }),
        Default::default(),
    );
    original.r#type.kind = TypeKind::Logic;
    original.r#type.signed = s.type_signed;
    original.r#type.is_positive = s.positive;
    original.is_const = s.constant;
    original.is_global = s.global;
    original.runtime_dependent = s.runtime;
    original.evaluated = s.evaluated;
    original.elaboration_const = s.cached;
    original.member_signed = s.member_signed;
    original.expr_context = s.context;
    let mut copy = original.clone();
    assert_stamp(&copy, s, None);
    assert!(
        copy.token == original.token,
        "metadata clone semantics mismatch"
    );
    if let ValueVariant::Numeric(Value::U64(v)) = &mut copy.value {
        v.payload ^= 1;
        v.mask_xz ^= 1;
        v.signed = !v.signed;
    }
    copy.r#type
        .set_concrete_width(Shape::new(vec![Some(s.context.width)]));
    copy.r#type.signed = !s.type_signed;
    copy.r#type.is_positive = !s.positive;
    copy.is_const = !s.constant;
    copy.is_global = !s.global;
    copy.runtime_dependent = !s.runtime;
    copy.evaluated = !s.evaluated;
    copy.elaboration_const = Some(true);
    copy.member_signed = Some(true);
    copy.expr_context = ExpressionContext::default();
    assert_stamp(&original, s, s.cached);
    kani::cover!(
        s.cached == Some(true),
        "true classification cache is invalidated"
    );
    kani::cover!(
        s.cached == Some(false),
        "false classification cache is invalidated"
    );
    kani::cover!(
        s.cached.is_none(),
        "absent classification cache is reachable"
    );
    kani::cover!(
        s.type_signed != s.value_signed,
        "independent type and value signs are reachable"
    );
    kani::cover!(
        s.mask != 0 && s.payload & s.mask == 0,
        "unknown metadata payload is reachable"
    );
    kani::cover!(
        s.mask != 0 && s.payload & s.mask != 0,
        "high impedance metadata payload is reachable"
    );
    kani::cover!(
        w == 4 && s.context.width == 1,
        "narrower stored context is reachable"
    );
    kani::cover!(
        w == 1 && s.context.width == 8,
        "wider stored context is reachable"
    );
    std::mem::forget(copy);
    std::mem::forget(original);
}
