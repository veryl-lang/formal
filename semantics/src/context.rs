//! IEEE 1800-2023 rule mapping:
//! SV-CONTEXT (10.8, 11.6.1, Table 11-21, 11.8.1, 11.8.2, 11.8.3): Determine width/sign and propagate context only to context-determined operands; comparisons prepare their operands independently of the enclosing expression.
//! SV-CAST-CONTEXT (6.24.1): A size cast retains operand signedness; a type cast uses target signedness. The cast width bounds its own operand context, independently of an enclosing expression.
//!
//! Width/sign policy is separate from recursive expression traversal and type decoding.
//! The const/global attributes and zero-width fallback are internal analyzer contracts.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Context {
    pub width: usize,
    pub signed: bool,
    pub is_const: bool,
    pub is_global: bool,
}

#[derive(Clone, Copy)]
pub enum BinaryRule {
    Common,
    Predicate,
    Left,
    Cast,
}

pub fn joined(a: Context, b: Context) -> Context {
    Context {
        width: if a.width >= b.width { a.width } else { b.width },
        signed: a.signed && b.signed,
        is_const: a.is_const && b.is_const,
        is_global: a.is_global && b.is_global,
    }
}

pub fn unary(a: Context, predicate: bool) -> Context {
    if predicate {
        Context {
            width: 1,
            signed: false,
            ..a
        }
    } else {
        a
    }
}

pub fn binary(a: Context, b: Context, rule: BinaryRule) -> Context {
    let flags = joined(a, b);
    match rule {
        BinaryRule::Common => flags,
        BinaryRule::Predicate => Context {
            width: 1,
            signed: false,
            ..flags
        },
        BinaryRule::Left => Context {
            width: a.width,
            signed: a.signed,
            ..flags
        },
        BinaryRule::Cast => Context {
            width: if b.width == 0 { a.width } else { b.width },
            signed: b.signed,
            ..flags
        },
    }
}

pub fn cast_operand(a: Context, target: usize) -> Context {
    Context {
        width: if a.width > target { a.width } else { target },
        ..a
    }
}

pub fn cast_result(a: Context, target: Context, target_is_type: bool) -> Context {
    Context {
        width: if target.width == 0 {
            a.width
        } else {
            target.width
        },
        signed: if target_is_type {
            target.signed
        } else {
            a.signed
        },
        is_const: a.is_const && target.is_const,
        is_global: a.is_global && target.is_global,
    }
}
