//! Width 192 in three words: arbitrary signs for amounts 0..=8, or sign-clear inputs
//! for every u64 amount. Negative inputs shifted by nine or more are not generally
//! proved. Empty metadata is a separate defensive no-access contract. Caller width
//! preparation and source decoding are outside the kernel proofs.

use veryl_formal_semantics::words;
use veryl_simulator::wide_ops;

#[kani::proof]
#[kani::unwind(9)]
fn wide_ashr_small_matches_reference() {
    signed_shift::<false, true>();
}

#[kani::proof]
#[kani::unwind(9)]
fn wide_ashr_small_inplace_matches_reference() {
    signed_shift::<true, true>();
}

#[kani::proof]
#[kani::unwind(4)]
fn wide_ashr_nonnegative_matches_reference() {
    signed_shift::<false, false>();
}

#[kani::proof]
#[kani::unwind(4)]
fn wide_ashr_nonnegative_inplace_matches_reference() {
    signed_shift::<true, false>();
}

fn signed_shift<const INPLACE: bool, const SMALL_AMOUNT: bool>() {
    let source: [u8; 25] = kani::any();
    let mut destination: [u8; 26] = kani::any();
    let source_before = source;
    let before = destination;
    let width = 192;
    let amount: u64 = kani::any();
    let observed = usize::from(kani::any::<u8>() % 26);
    let bit = usize::from(kani::any::<u8>() & 7);
    let original = if INPLACE {
        &before[1..25]
    } else {
        &source_before[1..]
    };
    let negative = words::bit(original, width - 1);
    if SMALL_AMOUNT {
        kani::assume(amount <= 8);
    } else {
        kani::assume(!negative);
    }
    unsafe {
        let dst = destination.as_mut_ptr().add(1);
        let src = if INPLACE {
            dst as *const u8
        } else {
            source.as_ptr().add(1)
        };
        wide_ops::wide_ashr(dst, src, amount, wide_ops::pack_nb_width(24, width));
    }
    let inside = observed > 0 && observed < 25;
    let position = observed.saturating_sub(1) * 8 + bit;
    let expected = if inside {
        words::arithmetic_shifted_bit(original, width, amount, position)
    } else {
        words::bit(&before, observed * 8 + bit)
    };
    assert!(
        words::bit(&destination, observed * 8 + bit) == expected,
        "simulator arithmetic shift semantics mismatch"
    );
    let src_observed = usize::from(kani::any::<u8>() % 25);
    assert!(
        source[src_observed] == source_before[src_observed],
        "simulator arithmetic shift semantics mismatch"
    );
    kani::cover!(amount == 0, "signed shift identity is reachable");
    if SMALL_AMOUNT {
        kani::cover!(
            amount == 1 && negative,
            "signed shift negative one-bit fill is reachable"
        );
        kani::cover!(
            amount == 8 && negative,
            "signed shift negative byte fill is reachable"
        );
        kani::cover!(
            amount == 8 && !negative,
            "signed shift positive byte fill is reachable"
        );
    } else {
        kani::cover!(amount == 64, "signed shift word boundary is reachable");
        kani::cover!(
            amount == 65,
            "signed shift above word boundary is reachable"
        );
        kani::cover!(amount == 191, "signed shift last bit is reachable");
        kani::cover!(
            amount == u64::MAX,
            "signed shift oversized positive fill is reachable"
        );
    }
    kani::cover!(observed == 0, "signed shift prefix guard is reachable");
    kani::cover!(observed == 25, "signed shift suffix guard is reachable");
}

#[kani::proof]
#[kani::unwind(4)]
fn wide_ashr_empty_metadata_preserves_storage() {
    let mut destination: [u8; 26] = kani::any();
    let before = destination;
    let observed = usize::from(kani::any::<u8>() % 26);
    let no_storage: bool = kani::any();
    let width = if no_storage {
        usize::from(kani::any::<u16>())
    } else {
        0
    };
    let nb = if no_storage { 0 } else { 24 };
    let amount: u64 = kani::any();
    unsafe {
        let dst = if no_storage {
            std::ptr::null_mut()
        } else {
            destination.as_mut_ptr().add(1)
        };
        wide_ops::wide_ashr(
            dst,
            std::ptr::null(),
            amount,
            wide_ops::pack_nb_width(nb, width),
        );
    }
    assert!(
        destination[observed] == before[observed],
        "simulator empty shift semantics mismatch"
    );
    kani::cover!(!no_storage, "signed shift zero width is reachable");
    kani::cover!(
        no_storage && width == 65535,
        "signed shift zero storage is reachable"
    );
    kani::cover!(
        no_storage && width == 0,
        "signed shift empty metadata is reachable"
    );
}
