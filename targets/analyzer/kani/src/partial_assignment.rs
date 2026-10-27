//! Normalized destination widths 1..=64, arbitrary prepared RHS planes and metadata,
//! and arbitrary native endpoints. Checks every stored bit and preserved metadata.
//! Does not prove source sizing, selector decoding, or legal part-select syntax.

use veryl_analyzer::value::ValueU64;
use veryl_formal_semantics::logic4::{Logic4, read_logic};
use veryl_formal_semantics::partial_assignment::assigned_logic_bit;

#[kani::proof]
#[kani::unwind(1)]
fn native_assignment_matches_reference_semantics() {
    let mut dst = ValueU64 {
        payload: kani::any(),
        mask_xz: kani::any(),
        width: kani::any(),
        signed: kani::any(),
    };
    kani::assume(dst.width >= 1 && dst.width <= 64);
    let normalized_mask = u64::MAX >> (64 - dst.width);
    kani::assume(dst.payload & !normalized_mask == 0);
    kani::assume(dst.mask_xz & !normalized_mask == 0);
    let rhs = ValueU64 {
        payload: kani::any(),
        mask_xz: kani::any(),
        width: kani::any(),
        signed: kani::any(),
    };
    let beg: usize = kani::any();
    let end: usize = kani::any();
    let position: usize = kani::any();
    kani::assume(position < 64);
    let before = dst.clone();

    dst.assign(rhs.clone(), beg, end);

    let expected = if position < before.width as usize {
        assigned_logic_bit(
            (before.payload as u128, before.mask_xz as u128),
            (rhs.payload as u128, rhs.mask_xz as u128),
            position,
            beg,
            end,
        )
    } else {
        Logic4::Zero
    };
    // Arbitrary position proves equality of every stored bit, without a
    // loop over all positions or a separate mask-based specification.
    assert!(
        read_logic(dst.payload as u128, dst.mask_xz as u128, position) == expected,
        "partial-assignment semantics mismatch"
    );
    assert_eq!(dst.width, before.width);
    assert_eq!(dst.signed, before.signed);

    kani::cover!(before.width == 64, "full native width is reachable");
    kani::cover!(end >= before.width as usize, "ignored writes are reachable");
    kani::cover!(
        end < before.width as usize && beg >= before.width as usize,
        "clipped writes are reachable"
    );
    kani::cover!(
        beg == usize::MAX && end == 0,
        "maximum high bound is reachable"
    );
    kani::cover!(expected == Logic4::X, "X result is reachable");
    kani::cover!(expected == Logic4::Z, "Z result is reachable");
}
