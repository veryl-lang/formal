//! First record in an empty real default buffer for width 192, amount 64, and
//! window width 64. Splits direct/log-only and separate/exact alias modes. The
//! no-op family admits symbolic metadata and null pointers. Multiple records, live
//! reallocation, general merged windows, and commit are outside these proofs.

use veryl_formal_semantics::words;
use veryl_simulator::wide_ops;

#[kani::proof]
#[kani::unwind(65)]
fn window_log_only_matches_reference() {
    record::<false, false>();
}
#[kani::proof]
#[kani::unwind(65)]
fn window_log_only_alias_matches_reference() {
    record::<false, true>();
}
#[kani::proof]
#[kani::unwind(65)]
fn window_log_direct_matches_reference() {
    record::<true, false>();
}
#[kani::proof]
#[kani::unwind(65)]
fn window_log_direct_alias_matches_reference() {
    record::<true, true>();
}

fn record<const DIRECT: bool, const ALIAS: bool>() {
    let direct = DIRECT;
    let alias = ALIAS;
    let source: [u8; 9] = kani::any();
    let source_before = source;
    let mut destination: [u8; 26] = kani::any();
    let before = destination;
    let offset: u32 = kani::any();
    kani::assume(offset <= u32::MAX - 15);
    let mut log = Default::default();
    unsafe {
        let dst = destination.as_mut_ptr().add(1);
        let src = if alias {
            dst as *const u8
        } else {
            source.as_ptr().add(1)
        };
        wide_ops::wide_window_store(
            &mut log,
            dst,
            src,
            64,
            64 | (192u64 << 32),
            offset as u64 | ((direct as u64) << 32),
        );
    }
    assert!(
        log.wide_count() == 1 && log.narrow_count() == 0,
        "simulator window log semantics mismatch"
    );
    let entry = &log.wide_entries_slice()[0];
    assert!(
        entry.offset == offset + 8 && entry.native_bytes == 8,
        "simulator window log semantics mismatch"
    );
    let original = if alias {
        &before[1..25]
    } else {
        &source_before[1..]
    };
    let observed = usize::from(kani::any::<u8>() & 7);
    assert!(
        entry.payload[observed] == original[observed],
        "simulator window log semantics mismatch"
    );
    let dst_observed = usize::from(kani::any::<u8>() % 26);
    let bit = usize::from(kani::any::<u8>() & 7);
    let inside = dst_observed > 0 && dst_observed < 25;
    let expected = if direct && inside {
        words::window_stored_bit(
            &before[1..25],
            original,
            192,
            64,
            64,
            (dst_observed - 1) * 8 + bit,
        )
    } else {
        words::bit(&before, dst_observed * 8 + bit)
    };
    assert!(
        words::bit(&destination, dst_observed * 8 + bit) == expected,
        "simulator window log semantics mismatch"
    );
    let src_observed = usize::from(kani::any::<u8>() % 9);
    assert!(
        source[src_observed] == source_before[src_observed],
        "simulator window log semantics mismatch"
    );
    kani::cover!(offset == 0, "window log zero offset is reachable");
    kani::cover!(
        offset == u32::MAX - 15,
        "window log highest safe offset is reachable"
    );
    kani::cover!(offset == 1, "window log unaligned offset is reachable");
    kani::cover!(
        observed == 0 && original[0] == 255,
        "window log first payload byte is reachable"
    );
    kani::cover!(
        observed == 7 && original[7] == 0,
        "window log last payload byte is reachable"
    );
    kani::cover!(
        before[9] != original[0],
        "window log nonidentity merge is reachable"
    );
    kani::cover!(dst_observed == 0, "window log prefix guard is reachable");
    kani::cover!(dst_observed == 25, "window log suffix guard is reachable");
}

#[kani::proof]
#[kani::unwind(2)]
fn window_log_empty_noop_matches_reference() {
    let width = kani::any::<u32>();
    let dst_width = kani::any::<u32>();
    let amount: u64 = kani::any();
    kani::assume(width == 0 || amount >= dst_width as u64);
    let mut log = Default::default();
    unsafe {
        wide_ops::wide_window_store(
            &mut log,
            std::ptr::null_mut(),
            std::ptr::null(),
            amount,
            width as u64 | ((dst_width as u64) << 32),
            kani::any(),
        );
    }
    assert!(
        log.wide_count() == 0 && log.narrow_count() == 0,
        "simulator window log semantics mismatch"
    );
    kani::cover!(
        width == 0 && dst_width == 192,
        "window log zero width is reachable"
    );
    kani::cover!(
        dst_width == 0 && width > 0,
        "window log empty destination is reachable"
    );
    kani::cover!(
        dst_width == 192 && amount == 192 && width > 0,
        "window log outside destination is reachable"
    );
    kani::cover!(
        amount == u64::MAX && width > 0,
        "window log maximum amount is reachable"
    );
}
