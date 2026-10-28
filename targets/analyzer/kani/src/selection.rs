//! Normalized four-state widths 1..=64 and arbitrary native endpoints, with result
//! width limited to 64. Checks missing-bit X, unsigned selection metadata, and guards.
//! Public dispatch adds source widths 1..=4 and endpoints 0..=width+2, including
//! construction, clone/input preservation, and every returned u128 bit.
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

struct SmallSelectionWitness {
    source_width: usize,
    reversed: bool,
    single: bool,
    full: bool,
    missing: bool,
    overlap: bool,
    highest: bool,
    highest_end: bool,
    source_x: bool,
    source_z: bool,
    signed: bool,
    padding: bool,
}

// Public native dispatch, including construction and cloning. All normalized
// four-state inputs and signs are symbolic; each layout checks endpoints 0..W+2.
// Observe every u128 bit so missing-bit X and high zero padding are both checked.
fn public_small_selection<const W: usize>() -> SmallSelectionWitness {
    use veryl_analyzer::value::Value;
    let mask = (1u128 << W) - 1;
    let payload = u128::from(kani::any::<u8>()) & mask;
    let xz = u128::from(kani::any::<u8>()) & mask;
    let signed: bool = kani::any();
    let beg = usize::from(kani::any::<u8>() & 7).min(W + 2);
    let end = usize::from(kani::any::<u8>() & 7).min(W + 2);
    let position = usize::from(kani::any::<u8>() & 127);
    let input = Value::from_u128(payload, xz, W, signed);
    assert!(
        matches!(input, Value::U64(_))
            && input.width() == W
            && input.signed() == signed
            && input.payload_u128() == payload
            && input.mask_xz_u128() == xz,
        "public selection semantics mismatch"
    );
    let original = input.clone();
    assert!(input == original, "public selection semantics mismatch");
    let result = input.select(beg, end);
    let width = if beg < end { 0 } else { beg - end + 1 };
    let expected = selected_logic_bit((payload, xz), W, beg, end, position);
    assert!(
        matches!(result, Value::U64(_))
            && result.width() == width
            && !result.signed()
            && read_logic(result.payload_u128(), result.mask_xz_u128(), position) == expected,
        "public selection semantics mismatch"
    );
    assert!(
        input == original
            && input.width() == W
            && input.signed() == signed
            && input.payload_u128() == payload
            && input.mask_xz_u128() == xz,
        "public selection semantics mismatch"
    );
    SmallSelectionWitness {
        source_width: W,
        reversed: beg < end,
        single: beg == end,
        full: beg == W - 1 && end == 0,
        missing: beg >= end && end >= W,
        overlap: beg >= W && end < W,
        highest: beg == W + 2 && end == 0,
        highest_end: beg == end && end == W + 2,
        source_x: expected == Logic4::X && end < W && position < W - end,
        source_z: expected == Logic4::Z,
        signed: signed && beg >= end && end < W,
        padding: position == 127 && expected == Logic4::Zero,
    }
}

#[kani::proof]
#[kani::unwind(1)]
fn public_selection_small_matches_reference_semantics() {
    let width = usize::from(kani::any::<u8>() & 3) + 1;
    let cases = match width {
        1 => public_small_selection::<1>(),
        2 => public_small_selection::<2>(),
        3 => public_small_selection::<3>(),
        4 => public_small_selection::<4>(),
        _ => unreachable!(),
    };
    kani::cover!(
        cases.source_width == 1,
        "public selection width 1 is reachable"
    );
    kani::cover!(
        cases.source_width == 2,
        "public selection width 2 is reachable"
    );
    kani::cover!(
        cases.source_width == 3,
        "public selection width 3 is reachable"
    );
    kani::cover!(
        cases.source_width == 4,
        "public selection width 4 is reachable"
    );
    kani::cover!(
        cases.reversed,
        "public selection reversed endpoints are reachable"
    );
    kani::cover!(cases.single, "public selection one bit is reachable");
    kani::cover!(cases.full, "public selection full source is reachable");
    kani::cover!(
        cases.missing,
        "public selection missing source is reachable"
    );
    kani::cover!(cases.overlap, "public selection overlap is reachable");
    kani::cover!(
        cases.highest,
        "public selection highest endpoint is reachable"
    );
    kani::cover!(
        cases.highest_end,
        "public selection maximum end is reachable"
    );
    kani::cover!(cases.source_x, "public selection source X is reachable");
    kani::cover!(cases.source_z, "public selection source Z is reachable");
    kani::cover!(cases.signed, "public selection signed source is reachable");
    kani::cover!(cases.padding, "public selection u128 padding is reachable");
}
