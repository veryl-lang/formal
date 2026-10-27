//! Normalized widths 1..=64, arbitrary four-state payloads and operand signedness.
//! Real expansion prepares common width before comparison. Checks equality, wildcard,
//! and relations; caller selection of relational mode and public widening are separate.

use veryl_analyzer::value::ValueU64;
use veryl_formal_semantics::comparison::{Comparison, Operand, compared_logic};
use veryl_formal_semantics::logic4::{Logic4, read_logic};

fn operand() -> ValueU64 {
    let width = u32::from(kani::any::<u8>() & 63) + 1;
    let mask = u64::MAX >> (64 - width);
    ValueU64 {
        payload: kani::any::<u64>() & mask,
        mask_xz: kani::any::<u64>() & mask,
        width,
        signed: kani::any(),
    }
}

fn reference_operand(value: &ValueU64) -> Operand {
    Operand {
        planes: (value.payload as u128, value.mask_xz as u128),
        width: value.width as usize,
        signed: value.signed,
    }
}

fn check_comparison(kind: Comparison) -> (ValueU64, ValueU64, Logic4) {
    let x = operand();
    let y = operand();
    let expected = compared_logic(reference_operand(&x), reference_operand(&y), kind);
    let width = x.width.max(y.width) as usize;
    let signed = x.signed && y.signed;
    let a = x.expand(width, signed);
    let b = y.expand(width, signed);
    let output = match kind {
        Comparison::Equal => a.equal(&b, false, false),
        Comparison::NotEqual => a.equal(&b, false, true),
        Comparison::WildcardEqual => a.equal(&b, true, false),
        Comparison::WildcardNotEqual => a.equal(&b, true, true),
        Comparison::Less => a.compare(&b, signed, false, false),
        Comparison::LessEqual => a.compare(&b, signed, false, true),
        Comparison::Greater => a.compare(&b, signed, true, false),
        Comparison::GreaterEqual => a.compare(&b, signed, true, true),
    };
    assert!(
        read_logic(output.payload as u128, output.mask_xz as u128, 0) == expected,
        "comparison semantics mismatch"
    );
    assert!(
        output.payload <= 1 && output.mask_xz <= 1,
        "comparison semantics mismatch"
    );
    assert!(
        output.width == 1 && !output.signed,
        "comparison semantics mismatch"
    );
    kani::cover!(width == 64, "width-64 comparison is reachable");
    kani::cover!(
        x.width == 1 && y.width == 1,
        "one-bit comparison is reachable"
    );
    kani::cover!(x.width != y.width, "unequal-width comparison is reachable");
    kani::cover!(x.signed != y.signed, "mixed-sign comparison is reachable");
    kani::cover!(signed, "signed comparison is reachable");
    kani::cover!(!x.signed && !y.signed, "unsigned comparison is reachable");
    kani::cover!(expected == Logic4::Zero, "comparison zero is reachable");
    kani::cover!(expected == Logic4::One, "comparison one is reachable");
    kani::cover!(expected == Logic4::X, "comparison X is reachable");
    kani::cover!(
        signed
            && x.width < y.width
            && (x.mask_xz >> (x.width - 1)) & 1 != 0
            && expected == Logic4::X,
        "unknown sign extension comparison is reachable"
    );
    (x, y, expected)
}

fn check_equality(kind: Comparison) {
    let (x, y, expected) = check_comparison(kind);
    kani::cover!(
        (x.mask_xz | y.mask_xz) != 0 && expected != Logic4::X,
        "known mismatch despite unknown bits is reachable"
    );
    kani::cover!(
        x.width == 1
            && y.width == 1
            && x.payload == 1
            && x.mask_xz == 0
            && y.mask_xz == 1
            && expected == Logic4::X,
        "one versus unknown equality is reachable"
    );
}

fn check_wildcard(kind: Comparison) {
    let (x, y, expected) = check_comparison(kind);
    kani::cover!(
        x.width == 1 && y.width == 1 && y.mask_xz == 1 && expected != Logic4::X,
        "RHS wildcard match is reachable"
    );
    kani::cover!(
        x.width == 1 && y.width == 1 && x.mask_xz == 1 && y.mask_xz == 0 && expected == Logic4::X,
        "unmatched LHS unknown is reachable"
    );
    kani::cover!(
        (x.mask_xz | y.mask_xz) != 0 && expected != Logic4::X && x.width > 1 && y.mask_xz == 0,
        "wildcard known mismatch despite unknown bits is reachable"
    );
}

fn check_relation(kind: Comparison) {
    let (x, y, _) = check_comparison(kind);
    let known = x.mask_xz == 0 && y.mask_xz == 0;
    let x_negative = (x.payload >> (x.width - 1)) & 1 != 0;
    let y_negative = (y.payload >> (y.width - 1)) & 1 != 0;
    kani::cover!(
        known && x.signed && y.signed && x_negative && !y_negative,
        "negative versus positive comparison is reachable"
    );
    kani::cover!(
        known && x.signed && y.signed && !x_negative && y_negative,
        "positive versus negative comparison is reachable"
    );
    kani::cover!(
        known && x.width == y.width && x.payload == y.payload,
        "equal known comparison is reachable"
    );
}

#[kani::proof]
#[kani::unwind(65)]
fn native_equality_matches_reference_semantics() {
    check_equality(Comparison::Equal);
}

#[kani::proof]
#[kani::unwind(65)]
fn native_inequality_matches_reference_semantics() {
    check_equality(Comparison::NotEqual);
}

#[kani::proof]
#[kani::unwind(65)]
fn native_wildcard_equality_matches_reference_semantics() {
    check_wildcard(Comparison::WildcardEqual);
}

#[kani::proof]
#[kani::unwind(65)]
fn native_wildcard_inequality_matches_reference_semantics() {
    check_wildcard(Comparison::WildcardNotEqual);
}

#[kani::proof]
#[kani::unwind(65)]
fn native_less_than_matches_reference_semantics() {
    check_relation(Comparison::Less);
}

#[kani::proof]
#[kani::unwind(65)]
fn native_less_equal_matches_reference_semantics() {
    check_relation(Comparison::LessEqual);
}

#[kani::proof]
#[kani::unwind(65)]
fn native_greater_than_matches_reference_semantics() {
    check_relation(Comparison::Greater);
}

#[kani::proof]
#[kani::unwind(65)]
fn native_greater_equal_matches_reference_semantics() {
    check_relation(Comparison::GreaterEqual);
}
