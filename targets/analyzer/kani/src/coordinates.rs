//! Resolved clamped and checked endpoints admit all native inputs and orientations.
//! Size composition uses one packed and one unpacked extent of a logic type.
//! Defensive metadata is not thereby legal source syntax; recursive kind widths,
//! caller interval checks, and signed/X/Z selector decoding remain separate.

use veryl_analyzer::ir::VarSelectOp;
use veryl_formal_semantics::coordinates::{self, Select};

#[kani::proof]
#[kani::unwind(1)]
fn selection_endpoints_match_reference_semantics() {
    let a: usize = kani::any();
    let b: usize = kani::any();
    let array: bool = kani::any();
    let choice = kani::any::<u8>() & 3;
    let (actual, reference) = match choice {
        0 => (VarSelectOp::Colon, Select::Colon),
        1 => (VarSelectOp::PlusColon, Select::Plus),
        2 => (VarSelectOp::MinusColon, Select::Minus),
        _ => (VarSelectOp::Step, Select::Step),
    };
    assert!(
        actual.eval_value(a, b, array) == coordinates::endpoints(reference, a, b, array),
        "selection endpoint semantics mismatch"
    );
    kani::cover!(choice == 0, "colon endpoint is reachable");
    kani::cover!(choice == 1, "plus endpoint is reachable");
    kani::cover!(choice == 2, "minus endpoint is reachable");
    kani::cover!(choice == 3, "step endpoint is reachable");
    kani::cover!(
        choice == 1 && a == usize::MAX && b == 1,
        "maximum plus unit endpoint is reachable"
    );
    kani::cover!(
        choice == 2 && a == usize::MAX && b == 1,
        "maximum minus unit endpoint is reachable"
    );
    kani::cover!(
        choice == 2 && a == usize::MAX && b == usize::MAX,
        "maximum minus width is reachable"
    );
    kani::cover!(
        choice == 2 && a == 2 && b == 3,
        "minus endpoint at zero is reachable"
    );
    kani::cover!(
        choice == 2 && a == 1 && b == 3,
        "minus endpoint underflow is reachable"
    );
    kani::cover!(
        choice == 3 && a == usize::MAX && b == 2,
        "step endpoint overflow is reachable"
    );
    kani::cover!(b == 0, "zero endpoint width is reachable");
    kani::cover!(
        array && choice == 1 && a == 1 && b == 2,
        "array ascending endpoint is reachable"
    );
    kani::cover!(
        !array && choice == 1 && a == 1 && b == 2,
        "packed descending endpoint is reachable"
    );
}

#[kani::proof]
#[kani::unwind(3)]
fn packed_unpacked_bits_match_reference_semantics() {
    use veryl_analyzer::ir::{Type, TypeKind};
    let packed: usize = kani::any();
    let array: usize = kani::any();
    let mut ty = Type::new(TypeKind::Logic);
    ty.width_mut().push(Some(packed));
    ty.array.push(Some(array));
    assert!(
        ty.total_width() == Some(packed)
            && ty.total_bits() == usize::try_from(packed as u128 * array as u128).ok(),
        "packed unpacked size semantics mismatch"
    );
    kani::cover!(packed == 0, "zero packed size is reachable");
    kani::cover!(array == 0, "zero unpacked size is reachable");
    kani::cover!(
        packed == usize::MAX && array == 1,
        "maximum packed size is reachable"
    );
    kani::cover!(
        packed == 1 && array == usize::MAX,
        "maximum unpacked size is reachable"
    );
    kani::cover!(
        packed == usize::MAX && array == 2,
        "combined size overflow is reachable"
    );
    kani::cover!(
        packed == 3 && array == 4,
        "nontrivial combined size is reachable"
    );
}

#[kani::proof]
#[kani::unwind(1)]
fn checked_selection_endpoints_match_reference_semantics() {
    let a: usize = kani::any();
    let b: usize = kani::any();
    let array: bool = kani::any();
    let choice = kani::any::<u8>() & 3;
    let (actual, reference) = match choice {
        0 => (VarSelectOp::Colon, Select::Colon),
        1 => (VarSelectOp::PlusColon, Select::Plus),
        2 => (VarSelectOp::MinusColon, Select::Minus),
        _ => (VarSelectOp::Step, Select::Step),
    };
    assert!(
        actual.checked_eval_value(a, b, array)
            == coordinates::checked_endpoints(reference, a, b, array),
        "checked selection endpoint semantics mismatch"
    );
    kani::cover!(choice == 0, "checked colon is reachable");
    kani::cover!(choice == 1, "checked plus is reachable");
    kani::cover!(choice == 2, "checked minus is reachable");
    kani::cover!(choice == 3, "checked step is reachable");
    kani::cover!(choice != 0 && b == 0, "checked zero width is reachable");
    kani::cover!(
        choice == 1 && a == usize::MAX && b == 1,
        "checked maximum plus unit is reachable"
    );
    kani::cover!(
        choice == 2 && a == usize::MAX && b == 1,
        "checked maximum minus unit is reachable"
    );
    kani::cover!(
        choice == 3 && a == usize::MAX && b == 1,
        "checked maximum step unit is reachable"
    );
    kani::cover!(
        choice == 1 && a == usize::MAX && b == 2,
        "checked plus overflow is reachable"
    );
    kani::cover!(
        choice == 2 && a == 0 && b == 2,
        "checked minus underflow is reachable"
    );
    kani::cover!(
        choice == 3 && a == usize::MAX && b == 2,
        "checked step overflow is reachable"
    );
    kani::cover!(
        choice == 3 && a == usize::MAX / 2 && b == 2,
        "checked fitting step boundary is reachable"
    );
    kani::cover!(
        choice == 0 && a == usize::MAX && b == 0,
        "checked full colon endpoints are reachable"
    );
    kani::cover!(
        choice == 1 && a == 1 && b == 2 && array,
        "checked array order is reachable"
    );
    kani::cover!(
        choice == 1 && a == 1 && b == 2 && !array,
        "checked packed order is reachable"
    );
}
