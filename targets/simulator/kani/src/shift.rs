//! Three words at byte offset one, arbitrary binary payload and u64 amount, split
//! by direction and disjoint/exact alias. Results cover full storage capacity with
//! zero fill. Variable lengths, partial overlap, logical-width clipping, source
//! amount decoding, and generated helper calls are outside these proofs.

use veryl_formal_semantics::words;
use veryl_simulator::wide_ops;

#[kani::proof]
#[kani::unwind(4)]
fn wide_shl_matches_reference() {
    shift::<true, false>();
}

#[kani::proof]
#[kani::unwind(4)]
fn wide_lshr_matches_reference() {
    shift::<false, false>();
}

#[kani::proof]
#[kani::unwind(4)]
fn wide_shl_inplace_matches_reference() {
    shift::<true, true>();
}

#[kani::proof]
#[kani::unwind(4)]
fn wide_lshr_inplace_matches_reference() {
    shift::<false, true>();
}

fn shift<const LEFT: bool, const INPLACE: bool>() {
    let source: [u8; 25] = kani::any();
    let source_before = source;
    let mut destination: [u8; 26] = kani::any();
    let before = destination;
    let amount: u64 = kani::any();
    let observed = usize::from(kani::any::<u8>() % 26);
    let bit = usize::from(kani::any::<u8>() & 7);
    unsafe {
        let dst = destination.as_mut_ptr().add(1);
        let src = if INPLACE {
            dst as *const u8
        } else {
            source.as_ptr().add(1)
        };
        if LEFT {
            wide_ops::wide_shl(dst, src, amount, 24);
        } else {
            wide_ops::wide_lshr(dst, src, amount, 24);
        }
    }
    let original = if INPLACE {
        &before[1..25]
    } else {
        &source_before[1..]
    };
    let inside = observed > 0 && observed < 25;
    let position = observed.saturating_sub(1) * 8 + bit;
    let expected = if inside {
        words::logical_shifted_bit(original, amount, LEFT, position)
    } else {
        words::bit(&before, observed * 8 + bit)
    };
    assert!(
        words::bit(&destination, observed * 8 + bit) == expected,
        "simulator logical shift semantics mismatch"
    );
    let src_observed = usize::from(kani::any::<u8>() % 25);
    assert!(
        source[src_observed] == source_before[src_observed],
        "simulator logical shift semantics mismatch"
    );
    kani::cover!(amount == 0, "shift identity is reachable");
    kani::cover!(amount == 1, "shift one bit is reachable");
    kani::cover!(amount == 63, "shift below word boundary is reachable");
    kani::cover!(amount == 64, "shift word boundary is reachable");
    kani::cover!(amount == 65, "shift above word boundary is reachable");
    kani::cover!(amount == 127, "shift below two words is reachable");
    kani::cover!(amount == 128, "shift two words is reachable");
    kani::cover!(amount == 191, "shift last storage bit is reachable");
    kani::cover!(amount == 192, "shift full capacity is reachable");
    kani::cover!(
        amount == 1u64 << 32,
        "shift above 32-bit amount is reachable"
    );
    kani::cover!(amount == 1u64 << 63, "shift high amount bit is reachable");
    kani::cover!(amount == u64::MAX, "shift maximum amount is reachable");
    kani::cover!(
        inside && position == if LEFT { 64 } else { 63 } && amount == 1 && expected,
        "shift nonzero across word boundary is reachable"
    );
    kani::cover!(observed == 0, "shift prefix guard is reachable");
    kani::cover!(observed == 25, "shift suffix guard is reachable");
}
