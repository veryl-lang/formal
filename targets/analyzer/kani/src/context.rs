//! Arbitrary native widths and scalar metadata, including defensive zero-width cases.
//! Checks real operator policy, context merging, and cast preparation. Cast targets
//! are already decoded; recursive gather/apply/evaluation and name resolution are not proved.

use veryl_analyzer::ir::{ExpressionContext, Op};
use veryl_formal_semantics::context::{self, BinaryRule, Context};

fn arbitrary() -> ExpressionContext {
    ExpressionContext {
        width: kani::any(),
        signed: kani::any(),
        is_const: kani::any(),
        is_global: kani::any(),
    }
}

fn reference(x: ExpressionContext) -> Context {
    Context {
        width: x.width,
        signed: x.signed,
        is_const: x.is_const,
        is_global: x.is_global,
    }
}

#[kani::proof]
#[kani::unwind(1)]
fn context_merge_matches_reference_semantics() {
    let a = arbitrary();
    let b = arbitrary();
    assert!(
        reference(a.merge(b)) == context::joined(reference(a), reference(b)),
        "context merge semantics mismatch"
    );
    kani::cover!(a.width == 0, "zero context width is reachable");
    kani::cover!(a.width == usize::MAX, "maximum context width is reachable");
    kani::cover!(a.width < b.width, "wider right context is reachable");
    kani::cover!(a.width > b.width, "wider left context is reachable");
    kani::cover!(a.signed && !b.signed, "unsigned sibling is reachable");
    kani::cover!(a.signed && b.signed, "signed context merge is reachable");
    kani::cover!(
        !a.is_const && b.is_const,
        "nonconstant left context is reachable"
    );
    kani::cover!(
        a.is_global && !b.is_global,
        "nonglobal right context is reachable"
    );
}

#[kani::proof]
#[kani::unwind(1)]
fn unary_context_matches_reference_semantics() {
    let a = arbitrary();
    let mode: u8 = kani::any();
    kani::assume(mode < 10);
    let op = match mode {
        0 => Op::Add,
        1 => Op::Sub,
        2 => Op::BitNot,
        3 => Op::BitAnd,
        4 => Op::BitNand,
        5 => Op::BitOr,
        6 => Op::BitNor,
        7 => Op::BitXor,
        8 => Op::BitXnor,
        _ => Op::LogicNot,
    };
    let predicate = mode >= 3;
    assert!(
        reference(op.eval_context_unary(a)) == context::unary(reference(a), predicate)
            && op.unary_x_self_determined() == predicate,
        "unary context semantics mismatch"
    );
    kani::cover!(mode == 0, "unary plus context is reachable");
    kani::cover!(mode == 1, "unary negation context is reachable");
    kani::cover!(mode == 2, "unary complement context is reachable");
    kani::cover!(mode == 3, "unary and context is reachable");
    kani::cover!(mode == 4, "unary nand context is reachable");
    kani::cover!(mode == 5, "unary or context is reachable");
    kani::cover!(mode == 6, "unary nor context is reachable");
    kani::cover!(mode == 7, "unary xor context is reachable");
    kani::cover!(mode == 8, "unary xnor context is reachable");
    kani::cover!(mode == 9, "unary not context is reachable");
}

#[kani::proof]
#[kani::unwind(1)]
fn binary_context_matches_reference_semantics() {
    let a = arbitrary();
    let b = arbitrary();
    let mode: u8 = kani::any();
    kani::assume(mode < 25);
    let op = match mode {
        0 => Op::Add,
        1 => Op::Sub,
        2 => Op::Mul,
        3 => Op::Div,
        4 => Op::Rem,
        5 => Op::BitAnd,
        6 => Op::BitOr,
        7 => Op::BitXor,
        8 => Op::BitXnor,
        9 => Op::Eq,
        10 => Op::Ne,
        11 => Op::EqWildcard,
        12 => Op::NeWildcard,
        13 => Op::Less,
        14 => Op::LessEq,
        15 => Op::Greater,
        16 => Op::GreaterEq,
        17 => Op::LogicAnd,
        18 => Op::LogicOr,
        19 => Op::LogicShiftL,
        20 => Op::LogicShiftR,
        21 => Op::ArithShiftL,
        22 => Op::ArithShiftR,
        23 => Op::Pow,
        _ => Op::As,
    };
    let rule = match mode {
        0..=8 => BinaryRule::Common,
        9..=18 => BinaryRule::Predicate,
        19..=23 => BinaryRule::Left,
        _ => BinaryRule::Cast,
    };
    assert!(
        reference(op.eval_context_binary(a, b))
            == context::binary(reference(a), reference(b), rule),
        "binary context semantics mismatch"
    );
    assert!(
        op.binary_x_self_determined() == (17..=18).contains(&mode)
            && op.binary_y_self_determined() == (17..=23).contains(&mode)
            && op.binary_op_self_determined() == (9..=16).contains(&mode),
        "binary context semantics mismatch"
    );
    kani::cover!(mode == 0, "binary Add context is reachable");
    kani::cover!(mode == 1, "binary Sub context is reachable");
    kani::cover!(mode == 2, "binary Mul context is reachable");
    kani::cover!(mode == 3, "binary Div context is reachable");
    kani::cover!(mode == 4, "binary Rem context is reachable");
    kani::cover!(mode == 5, "binary BitAnd context is reachable");
    kani::cover!(mode == 6, "binary BitOr context is reachable");
    kani::cover!(mode == 7, "binary BitXor context is reachable");
    kani::cover!(mode == 8, "binary BitXnor context is reachable");
    kani::cover!(mode == 9, "binary Eq context is reachable");
    kani::cover!(mode == 10, "binary Ne context is reachable");
    kani::cover!(mode == 11, "binary EqWildcard context is reachable");
    kani::cover!(mode == 12, "binary NeWildcard context is reachable");
    kani::cover!(mode == 13, "binary Less context is reachable");
    kani::cover!(mode == 14, "binary LessEq context is reachable");
    kani::cover!(mode == 15, "binary Greater context is reachable");
    kani::cover!(mode == 16, "binary GreaterEq context is reachable");
    kani::cover!(mode == 17, "binary LogicAnd context is reachable");
    kani::cover!(mode == 18, "binary LogicOr context is reachable");
    kani::cover!(mode == 19, "binary LogicShiftL context is reachable");
    kani::cover!(mode == 20, "binary LogicShiftR context is reachable");
    kani::cover!(mode == 21, "binary ArithShiftL context is reachable");
    kani::cover!(mode == 22, "binary ArithShiftR context is reachable");
    kani::cover!(mode == 23, "binary Pow context is reachable");
    kani::cover!(mode == 24, "binary As context is reachable");
    kani::cover!(
        mode == 24 && b.width == 0,
        "unsized cast target fallback is reachable"
    );
    kani::cover!(
        mode == 24 && b.width < a.width && b.width != 0,
        "cast context narrowing is reachable"
    );
    kani::cover!(
        mode == 24 && b.width > a.width,
        "cast context growth is reachable"
    );
}

#[kani::proof]
#[kani::unwind(1)]
fn cast_context_matches_reference_semantics() {
    let a = arbitrary();
    let target = arbitrary();
    let target_is_type: bool = kani::any();
    assert!(
        reference(a.cast_operand(target.width))
            == context::cast_operand(reference(a), target.width)
            && reference(Op::eval_context_cast(a, target, target_is_type))
                == context::cast_result(reference(a), reference(target), target_is_type),
        "cast context semantics mismatch"
    );
    kani::cover!(target_is_type, "type cast context is reachable");
    kani::cover!(!target_is_type, "numeric cast context is reachable");
    kani::cover!(
        !target_is_type && a.signed && !target.signed,
        "numeric cast preserves signed source"
    );
    kani::cover!(
        !target_is_type && !a.signed && target.signed,
        "numeric cast preserves unsigned source"
    );
    kani::cover!(
        target_is_type && a.signed && !target.signed,
        "type cast changes signedness"
    );
    kani::cover!(
        target.width > a.width,
        "cast operand context growth is reachable"
    );
    kani::cover!(
        target.width < a.width,
        "cast operand context retains wider source"
    );
    kani::cover!(
        target.width == a.width,
        "cast operand context equal width is reachable"
    );
    kani::cover!(
        target.width == 0,
        "unresolved cast width fallback is reachable"
    );
    kani::cover!(
        target.width == usize::MAX,
        "maximum cast context width is reachable"
    );
}
