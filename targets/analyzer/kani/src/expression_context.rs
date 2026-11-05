//! Resolved Logic/U64 literal leaves with normalized arbitrary four-state planes,
//! source widths 1..4 and independent signs. Incoming contexts have widths 1..8
//! and arbitrary signed/const/global flags. Actual constructors and Factor
//! gather/apply/eval execute; checks observe stored metadata and unchanged raw
//! value/width/sign/storage. IEEE 1800-2023 11.6.1/11.8.2 motivates deferred sizing;
//! const/global/evaluated flags are internal analyzer interface contracts.
//! Types are resolved by the harness. No parser, variables, deferred widths,
//! aggregates, floats, functions, recursive ASTs or diagnostics theorem.
//! Kani models atomics sequentially. The normal catch_unwind summary executes
//! initialization and keeps every panic as a failed assertion; panic recovery,
//! concurrent initialization and allocation failure are outside scope.
//! Unsupported grammar/numeric branches panic; harness teardown is excluded.
//! The native literal_leaf_context_boundary test checks real initialization,
//! empty deferred-width data and diagnostics across 43520 finite cases.
use std::panic::catch_unwind as catching;
use veryl_analyzer::{
    Context,
    ir::{Comptime, Expression, Op},
    value::Value,
};

#[kani::proof]
#[kani::unwind(3)]
#[kani::stub(catching, normal_catch_unwind)]
#[kani::stub(veryl_analyzer::value::biguint_to_u128, reject_big_getter)]
#[kani::stub(veryl_analyzer::value::biguint_low_u64, reject_big_low)]
#[kani::stub(<num_bigint::BigUint as std::convert::From<u64>>::from, reject_big_from64)]
#[kani::stub(veryl_analyzer::value::ValueBigUint::payload, reject_big_plane)]
#[kani::stub(veryl_analyzer::value::ValueBigUint::mask_xz, reject_big_plane)]
#[kani::stub(veryl_analyzer::value::ValueBigUint::to_value_u64, reject_big_native)]
#[kani::stub(veryl_analyzer::value::ValueBigUint::is_xz, reject_big_unknown)]
#[kani::stub(veryl_analyzer::ir::TypeKindStruct::width, reject_struct_width)]
#[kani::stub(veryl_analyzer::ir::TypeKindUnion::width, reject_union_width)]
#[kani::stub(veryl_analyzer::ir::TypeKindEnum::width, reject_enum_width)]
#[kani::stub(veryl_analyzer::ir::VarIndex::eval_value, reject_index)]
#[kani::stub(veryl_analyzer::ir::VarSelect::eval_comptime, reject_selection_shape)]
#[kani::stub(veryl_analyzer::ir::VarSelect::eval_value, reject_selection_value)]
#[kani::stub(veryl_analyzer::ir::Op::invalid_operand, reject_invalid_operand)]
#[kani::stub(
    veryl_analyzer::ir::Op::invalid_logical_operand,
    reject_invalid_logical
)]
#[kani::stub(veryl_analyzer::ir::Op::eval_float_binary, reject_float_binary)]
#[kani::stub(veryl_analyzer::ir::Op::eval_float_unary, reject_float_unary)]
#[kani::stub(veryl_analyzer::value::u128_to_biguint, reject_biguint)]
#[kani::stub(veryl_analyzer::value::ValueBigUint::to_bigint, reject_bigint)]
#[kani::stub(<veryl_analyzer::value::ValueBigUint as std::clone::Clone>::clone, reject_big_clone)]
#[kani::stub(veryl_analyzer::value::ValueBigUint::gen_mask, reject_big_mask)]
#[kani::stub(veryl_analyzer::value::MaskCache::get, reject_big_cache)]
#[kani::stub(veryl_analyzer::ir::FunctionCall::eval_value, reject_function_value)]
#[kani::stub(veryl_analyzer::ir::FunctionCall::eval_type, reject_function_type)]
#[kani::stub(
    veryl_analyzer::ir::SystemFunctionCall::eval_value,
    reject_system_value
)]
#[kani::stub(veryl_analyzer::ir::Expression::gather_context, reject_expr_gather)]
#[kani::stub(veryl_analyzer::ir::Expression::apply_context, reject_expr_apply)]
#[kani::stub(veryl_analyzer::ir::Expression::eval_value, reject_expr_value)]
fn literal_leaf_context_matches_reference() {
    let source_width = usize::from(kani::any::<u8>() & 3) + 1;
    let source_signed: bool = kani::any();
    let payload = u64::from(kani::any::<u8>()) & ((1u64 << source_width) - 1);
    let mask = u64::from(kani::any::<u8>()) & ((1u64 << source_width) - 1);
    let mut literal = veryl_analyzer::ir::Factor::create_value(
        Value::from_u128(
            u128::from(payload),
            u128::from(mask),
            source_width,
            source_signed,
        ),
        Default::default(),
    );
    literal.comptime_mut().r#type.signed = source_signed;
    literal.comptime_mut().r#type.kind = veryl_analyzer::ir::TypeKind::Logic;
    assert!(
        literal.comptime().r#type.width_expr().is_empty()
            && literal.comptime().r#type.array_expr().is_empty(),
        "literal leaf context semantics mismatch"
    );
    let mut context = Context::default();
    let shape = literal.gather_context(&mut context);
    assert!(
        shape.width == source_width
            && shape.signed == source_signed
            && shape.is_const
            && shape.is_global,
        "literal leaf context semantics mismatch"
    );
    let outer = veryl_analyzer::ir::ExpressionContext {
        width: usize::from(kani::any::<u8>() & 7) + 1,
        signed: kani::any(),
        is_const: kani::any(),
        is_global: kani::any(),
    };
    literal.apply_context(outer);
    let applied = literal.comptime().expr_context;
    assert!(
        applied.width == outer.width
            && applied.signed == outer.signed
            && applied.is_const == outer.is_const
            && applied.is_global == outer.is_global
            && literal.comptime().evaluated
            && literal.comptime().r#type.total_width() == Some(source_width)
            && literal.comptime().is_const
            && literal.comptime().is_global
            && literal.comptime().r#type.signed == source_signed
            && literal.comptime().r#type.width_expr().is_empty()
            && literal.comptime().r#type.array_expr().is_empty(),
        "literal leaf context semantics mismatch"
    );
    let out = literal.eval_value(&mut context).unwrap();
    assert!(
        matches!(out, Value::U64(_)),
        "literal leaf context semantics mismatch"
    );
    assert!(
        out.width() == source_width
            && out.signed() == source_signed
            && out.payload_u128() == u128::from(payload)
            && out.mask_xz_u128() == u128::from(mask),
        "literal leaf context semantics mismatch"
    );
    kani::cover!(
        source_width == 1 && source_signed,
        "literal leaf signed width1 source is reachable"
    );
    kani::cover!(
        source_width == 4 && !source_signed,
        "literal leaf unsigned width4 source is reachable"
    );
    kani::cover!(
        outer.width < source_width,
        "literal leaf smaller incoming width is reachable"
    );
    kani::cover!(
        outer.width > source_width,
        "literal leaf larger incoming width is reachable"
    );
    kani::cover!(
        mask != 0 && payload & mask == 0,
        "literal leaf X is reachable"
    );
    kani::cover!(
        mask != 0 && payload & mask != 0,
        "literal leaf Z is reachable"
    );
    std::mem::forget(out);
    std::mem::forget(literal);
    std::mem::forget(context);
}

fn normal_catch_unwind<F: FnOnce() -> R + std::panic::UnwindSafe, R>(
    f: F,
) -> std::thread::Result<R> {
    // Execute initialization; a panic remains a failed Kani assertion.
    Ok(f())
}

fn reject_big_getter(_: &num_bigint::BigUint) -> u128 {
    panic!("BigUint getter reached");
}

fn reject_big_low(_: &num_bigint::BigUint) -> u64 {
    panic!("BigUint low getter reached");
}

fn reject_big_from64(_: u64) -> num_bigint::BigUint {
    panic!("BigUint conversion reached");
}

fn reject_big_plane(_: &veryl_analyzer::value::ValueBigUint) -> &num_bigint::BigUint {
    panic!("BigUint plane reached");
}

fn reject_big_native(
    _: &veryl_analyzer::value::ValueBigUint,
) -> Option<veryl_analyzer::value::ValueU64> {
    panic!("BigUint native reached");
}

fn reject_big_unknown(_: &veryl_analyzer::value::ValueBigUint) -> bool {
    panic!("BigUint unknown reached");
}

fn reject_struct_width(_: &veryl_analyzer::ir::TypeKindStruct) -> Option<usize> {
    panic!("aggregate width reached");
}

fn reject_union_width(_: &veryl_analyzer::ir::TypeKindUnion) -> Option<usize> {
    panic!("aggregate width reached");
}

fn reject_enum_width(_: &veryl_analyzer::ir::TypeKindEnum) -> Option<usize> {
    panic!("enum width reached");
}

fn reject_index(_: &veryl_analyzer::ir::VarIndex, _: &mut Context) -> Option<Vec<usize>> {
    panic!("variable index reached");
}

fn reject_selection_shape(
    _: &veryl_analyzer::ir::VarSelect,
    _: &mut Context,
    _: &veryl_analyzer::ir::Type,
    _: bool,
) -> Option<veryl_analyzer::ir::Shape> {
    panic!("variable selection reached");
}

fn reject_selection_value(
    _: &veryl_analyzer::ir::VarSelect,
    _: &mut Context,
    _: &veryl_analyzer::ir::Type,
    _: bool,
) -> Option<(usize, usize)> {
    panic!("variable selection reached");
}

fn reject_invalid_operand(_: &Op, _: &mut Context, _: &Comptime) -> veryl_analyzer::ir::Type {
    panic!("invalid operand reached");
}

fn reject_invalid_logical(_: &Op, _: &mut Context, _: &Comptime) {
    panic!("invalid logical operand reached");
}

fn reject_float_binary(
    _: &Op,
    _: &Value,
    _: &Value,
    _: &veryl_analyzer::ir::TypeKind,
) -> Option<Value> {
    panic!("float reached");
}

fn reject_float_unary(_: &Op, _: &Value, _: &veryl_analyzer::ir::TypeKind) -> Option<Value> {
    panic!("float reached");
}

fn reject_biguint(_: u128) -> num_bigint::BigUint {
    panic!("BigUint reached");
}

fn reject_bigint(_: &veryl_analyzer::value::ValueBigUint) -> Option<num_bigint::BigInt> {
    panic!("BigInt reached");
}

fn reject_big_clone(
    _: &veryl_analyzer::value::ValueBigUint,
) -> veryl_analyzer::value::ValueBigUint {
    panic!("BigUint clone reached");
}

fn reject_big_mask(_: usize) -> num_bigint::BigUint {
    panic!("BigUint mask reached");
}

fn reject_big_cache(_: &mut veryl_analyzer::value::MaskCache, _: usize) -> &num_bigint::BigUint {
    panic!("BigUint cache reached");
}

fn reject_function_value(_: &veryl_analyzer::ir::FunctionCall, _: &mut Context) -> Option<Value> {
    panic!("unreachable function call");
}

fn reject_function_type(_: &mut veryl_analyzer::ir::FunctionCall, _: &mut Context) {
    panic!("unreachable function type");
}

fn reject_system_value(
    _: &veryl_analyzer::ir::SystemFunctionCall,
    _: &mut Context,
) -> Option<Value> {
    panic!("unreachable system function");
}

fn reject_expr_gather(
    _: &mut Expression,
    _: &mut Context,
) -> veryl_analyzer::ir::ExpressionContext {
    panic!("unexpected recursive gather");
}

fn reject_expr_apply(
    _: &mut Expression,
    _: &mut Context,
    _: veryl_analyzer::ir::ExpressionContext,
) {
    panic!("unexpected recursive apply");
}

fn reject_expr_value(_: &Expression, _: &mut Context) -> Option<Value> {
    panic!("unexpected recursive eval");
}
