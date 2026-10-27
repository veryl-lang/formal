//! Normalized four-state widths 1..=64 and arbitrary native endpoints, with result
//! width limited to 64. Checks missing-bit X, unsigned selection metadata, and guards.
//! Declared ranges, signed/X/Z selector decoding, and expression lowering are not proved.

use veryl_analyzer::value::ValueU64;
use veryl_formal_semantics::logic4::{Logic4, read_logic};
use veryl_formal_semantics::selection::selected_logic_bit;

#[kani::proof]
#[kani::unwind(1)]
fn native_selection_matches_reference_semantics() {
    let input = ValueU64 {
        payload: kani::any(),
        mask_xz: kani::any(),
        width: kani::any(),
        signed: kani::any(),
    };
    kani::assume(input.width >= 1 && input.width <= 64);
    let normalized_mask = u64::MAX >> (64 - input.width);
    kani::assume(input.payload & !normalized_mask == 0);
    kani::assume(input.mask_xz & !normalized_mask == 0);
    let beg: usize = kani::any();
    let end: usize = kani::any();
    kani::assume(beg < end || beg - end < 64);
    let position: usize = kani::any();
    kani::assume(position < 64);
    let actual = input.select(beg, end);
    let expected = selected_logic_bit(
        (input.payload as u128, input.mask_xz as u128),
        input.width as usize,
        beg,
        end,
        position,
    );
    assert!(
        read_logic(actual.payload as u128, actual.mask_xz as u128, position) == expected,
        "selection semantics mismatch"
    );
    let output_width = if beg < end { 0 } else { beg - end + 1 };
    assert!(
        actual.width as usize == output_width,
        "selection semantics mismatch"
    );
    assert!(!actual.signed, "selection semantics mismatch");
    kani::cover!(beg < end, "reversed selection is reachable");
    kani::cover!(beg == end, "single-bit selection is reachable");
    kani::cover!(
        input.width == 64 && beg == 63 && end == 0,
        "full-width selection is reachable"
    );
    kani::cover!(
        beg >= end && end >= input.width as usize && end < 64,
        "invalid selection below bit 64 is reachable"
    );
    kani::cover!(
        beg >= end && end >= 64,
        "invalid selection above bit 63 is reachable"
    );
    kani::cover!(
        beg >= input.width as usize && end < input.width as usize,
        "overlapping selection is reachable"
    );
    kani::cover!(
        beg == usize::MAX && end == usize::MAX,
        "maximum selection bounds are reachable"
    );
    kani::cover!(
        end < input.width as usize
            && end <= beg
            && beg < input.width as usize
            && expected == Logic4::X,
        "selected source X is reachable"
    );
    kani::cover!(expected == Logic4::Z, "selected source Z is reachable");
    kani::cover!(
        input.signed && beg >= end,
        "signed input selection is reachable"
    );
}
