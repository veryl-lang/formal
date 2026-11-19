//! Two bounded binary write histories followed by both real commit wrappers.
//! A 128-bit window at bit 0 is followed by a 64-bit window at bit 64 in the
//! same 128-bit allocation. Both direct flags and both disjoint/exact-base source
//! aliases are independent. The second alias reads the current staging word,
//! including the first direct update, but not an uncommitted log-only value.
//! Wide capacity 1 forces growth to 64 slots on the second append. A second harness instead
//! enqueues a narrow 8-byte record after the wide window. Commit still consumes
//! narrow before wide, so pool order takes precedence over cross-pool enqueue order.
//! Every payload bit is arbitrary. Pointers have byte offset 1; disjoint sources
//! expose 16/8 bytes; commit addresses use base 1 in an 18-byte guarded buffer.
//! Scalar word assignments model staging and record replacement independently.
//! Commit starts from the post-store staging state. Final bytes, transient ordered
//! change events, guards, source frames and live record fields are checked.
//! No production operation or constructor is stubbed. Allocations succeed and
//! execution is single-threaded. Other layouts, partial-bit windows, longer or
//! prefilled histories, partial pointer overlaps and four-state execution are
//! excluded. This is the internal pool protocol, not a source-level NBA theorem.
use veryl_simulator::{
    ir::{WriteLogBuffer, ff_commit_from_log, ff_commit_from_log_marking},
    wide_ops,
};

#[cfg(not(all(target_endian = "little", target_pointer_width = "64")))]
compile_error!("window history proofs require a little-endian 64-bit target");

#[kani::proof]
#[kani::unwind(65)]
fn window_history_overlap_matches_reference() {
    let r = history::<false>();
    kani::cover!(
        !r.direct_a && !r.direct_b,
        "overlap both log-only writes are reachable"
    );
    kani::cover!(
        r.direct_a && r.direct_b,
        "overlap both direct writes are reachable"
    );
    kani::cover!(
        r.direct_a && !r.direct_b,
        "overlap direct then log-only is reachable"
    );
    kani::cover!(
        !r.direct_a && r.direct_b,
        "overlap log-only then direct is reachable"
    );
    kani::cover!(
        !r.alias_a && !r.alias_b,
        "overlap disjoint sources are reachable"
    );
    kani::cover!(
        r.alias_a && r.alias_b,
        "overlap both source aliases are reachable"
    );
    kani::cover!(
        !r.alias_a && r.alias_b,
        "overlap second source alias is reachable"
    );
    kani::cover!(
        r.alias_a && !r.alias_b,
        "overlap first source alias is reachable"
    );
    kani::cover!(
        r.direct_a && !r.alias_a && r.alias_b && r.a[0] != r.initial[0] && r.b == r.a[0],
        "second alias reads first direct update"
    );
    kani::cover!(
        !r.direct_a && !r.alias_a && r.alias_b && r.a[0] != r.initial[0] && r.b == r.initial[0],
        "second alias ignores uncommitted first value"
    );
    kani::cover!(
        !r.first && !r.second && r.events == 0,
        "overlap no changes emit no events"
    );
    kani::cover!(
        r.first && !r.second && r.events == 1,
        "overlap only first record changes"
    );
    kani::cover!(
        !r.first && r.second && r.events == 1,
        "overlap only second record changes"
    );
    kani::cover!(
        r.first && r.second && r.events == 2,
        "overlap both records emit ordered events"
    );
    kani::cover!(
        !r.direct_a
            && !r.direct_b
            && r.a[0] == r.initial[0]
            && r.b == r.initial[1]
            && r.a[1] != r.b
            && r.events == 2,
        "overlap restoring initial state still emits two events"
    );
    kani::cover!(
        r.direct_a && r.direct_b && r.a[1] != r.b && r.events == 2,
        "overlap direct final state still has transient commit changes"
    );
    kani::cover!(r.byte == 0, "overlap prefix guard is reachable");
    kani::cover!(r.byte == 17, "overlap suffix guard is reachable");
    kani::cover!(r.byte == 8, "overlap preserved first word end is reachable");
    kani::cover!(r.byte == 9, "overlap last writer start is reachable");
}

#[kani::proof]
#[kani::unwind(65)]
fn window_history_mixed_matches_reference() {
    let r = history::<true>();
    kani::cover!(
        r.direct_a && r.alias_a,
        "mixed history direct source alias is reachable"
    );
    kani::cover!(
        !r.direct_a && r.alias_a,
        "mixed history log-only source alias is reachable"
    );
    kani::cover!(
        r.direct_a && !r.alias_a,
        "mixed history direct external source is reachable"
    );
    kani::cover!(
        !r.direct_a && !r.alias_a,
        "mixed history log-only external source is reachable"
    );
    kani::cover!(
        r.a[1] != r.b,
        "wide pool overrides later enqueued narrow value"
    );
    kani::cover!(
        !r.first && !r.second && r.events == 0,
        "mixed history no changes emit no events"
    );
    kani::cover!(
        r.first && !r.second && r.events == 1,
        "mixed history only narrow record changes"
    );
    kani::cover!(
        !r.first && r.second && r.events == 1,
        "mixed history only wide record changes"
    );
    kani::cover!(
        r.first && r.second && r.events == 2,
        "mixed history emits narrow then wide events"
    );
    kani::cover!(
        r.direct_a && r.a[1] != r.b && r.events == 2,
        "mixed history restores direct staging with two events"
    );
    kani::cover!(
        !r.direct_a && r.a == r.initial && r.a[1] != r.b && r.events == 2,
        "mixed history restores initial state with two events"
    );
    kani::cover!(r.byte == 0, "mixed history prefix guard is reachable");
    kani::cover!(r.byte == 17, "mixed history suffix guard is reachable");
}

struct Observation {
    direct_a: bool,
    direct_b: bool,
    alias_a: bool,
    alias_b: bool,
    initial: [u64; 2],
    a: [u64; 2],
    b: u64,
    first: bool,
    second: bool,
    events: usize,
    byte: usize,
}

fn history<const MIXED: bool>() -> Observation {
    let initial: [u64; 2] = kani::any();
    let external_a: [u64; 2] = kani::any();
    let external_b: u64 = kani::any();
    let direct_a: bool = kani::any();
    let direct_b = !MIXED && kani::any::<bool>();
    let alias_a: bool = kani::any();
    let alias_b = !MIXED && kani::any::<bool>();
    let source_a = guarded::<2, 17>(&external_a);
    let source_b = guarded::<1, 9>(&[external_b]);
    let source_a_before = source_a;
    let source_b_before = source_b;
    let mut destination = guarded::<2, 18>(&initial);
    destination[17] = kani::any();
    let before = destination;
    let a = if alias_a { initial } else { external_a };
    let after_a = if direct_a { a } else { initial };
    let b = if alias_b { after_a[0] } else { external_b };
    let staging_words = [after_a[0], if direct_b { b } else { after_a[1] }];
    let final_words = [a[0], if MIXED { a[1] } else { b }];
    let mut log = WriteLogBuffer::with_capacity(usize::from(MIXED), 1);
    unsafe {
        let dst = destination.as_mut_ptr().add(1);
        let src_a = if alias_a {
            dst.cast_const()
        } else {
            source_a.as_ptr().add(1)
        };
        wide_ops::wide_window_store(
            &mut log,
            dst,
            src_a,
            0,
            128 | (128u64 << 32),
            1 | ((direct_a as u64) << 32),
        );
        if MIXED {
            (log.grow_push_narrow)(&mut log, 9, external_b, 8);
        } else {
            let src_b = if alias_b {
                dst.cast_const()
            } else {
                source_b.as_ptr().add(1)
            };
            wide_ops::wide_window_store(
                &mut log,
                dst,
                src_b,
                64,
                64 | (128u64 << 32),
                1 | ((direct_b as u64) << 32),
            );
        }
    }
    let records = if MIXED { 1 } else { 2 };
    assert!(
        log.wide_count() == records
            && log.narrow_count() == u32::from(MIXED)
            && log.wide_capacity() >= records as usize,
        "simulator window history semantics mismatch"
    );
    let entries = log.wide_entries_slice();
    let saved = [entries[0], entries[if MIXED { 0 } else { 1 }]];
    assert!(
        entries[0].offset == 1 && entries[0].native_bytes == 16,
        "simulator window history semantics mismatch"
    );
    let payload_byte = usize::from(kani::any::<u8>() & 15);
    assert!(
        entries[0].payload[payload_byte] == a[payload_byte / 8].to_le_bytes()[payload_byte % 8],
        "simulator window history semantics mismatch"
    );
    if MIXED {
        let n = &log.narrow_entries_slice()[0];
        assert!(
            n.offset == 9 && n.width_class == 8 && n.mask_xz == 0 && n.payload == b,
            "simulator window history semantics mismatch"
        );
    } else {
        let byte = usize::from(kani::any::<u8>() & 7);
        assert!(
            entries[1].offset == 9
                && entries[1].native_bytes == 8
                && entries[1].payload[byte] == b.to_le_bytes()[byte],
            "simulator window history semantics mismatch"
        );
    }
    let (first, second) = if MIXED {
        (staging_words[1] != b, staging_words[0] != a[0] || b != a[1])
    } else {
        (staging_words != a, a[1] != b)
    };
    let (first_event, second_event) = if MIXED {
        ((9, 8), (1, 16))
    } else {
        ((1, 16), (9, 8))
    };
    let expected_events = match (first, second) {
        (true, true) => [Some(first_event), Some(second_event)],
        (true, false) => [Some(first_event), None],
        (false, true) => [Some(second_event), None],
        (false, false) => [None, None],
    };
    let capacity = (log.narrow_capacity(), log.wide_capacity());
    let mut plain = destination;
    let mut marked = destination;
    ff_commit_from_log(&mut plain, &log);
    let mut events = 0;
    ff_commit_from_log_marking(&mut marked, &log, &mut |off, nb| {
        assert!(
            expected_events.get(events).copied().flatten() == Some((off, nb)),
            "simulator window history semantics mismatch"
        );
        events += 1;
    });
    assert!(
        events == usize::from(first) + usize::from(second),
        "simulator window history semantics mismatch"
    );
    let byte = usize::from(kani::any::<u8>() % 18);
    let staged = modeled_byte(&before, staging_words, byte);
    let expected = modeled_byte(&before, final_words, byte);
    assert!(
        destination[byte] == staged && plain[byte] == expected && marked[byte] == expected,
        "simulator window history semantics mismatch"
    );
    assert!(
        source_a == source_a_before
            && source_b == source_b_before
            && log.wide_count() == records
            && log.narrow_count() == u32::from(MIXED)
            && capacity == (log.narrow_capacity(), log.wide_capacity()),
        "simulator window history semantics mismatch"
    );
    let observed_entry = usize::from(kani::any::<bool>());
    let observed_byte = usize::from(kani::any::<u8>() % 56);
    if observed_entry < records as usize {
        let entry = &log.wide_entries_slice()[observed_entry];
        assert!(
            entry.offset == saved[observed_entry].offset
                && entry.native_bytes == saved[observed_entry].native_bytes
                && entry.payload[observed_byte] == saved[observed_entry].payload[observed_byte],
            "simulator window history semantics mismatch"
        );
    }
    if MIXED {
        let n = &log.narrow_entries_slice()[0];
        assert!(
            n.offset == 9 && n.width_class == 8 && n.mask_xz == 0 && n.payload == b,
            "simulator window history semantics mismatch"
        );
    }
    Observation {
        direct_a,
        direct_b,
        alias_a,
        alias_b,
        initial,
        a,
        b,
        first,
        second,
        events,
        byte,
    }
}

fn modeled_byte(before: &[u8; 18], words: [u64; 2], byte: usize) -> u8 {
    if byte == 0 || byte == 17 {
        before[byte]
    } else {
        words[(byte - 1) / 8].to_le_bytes()[(byte - 1) % 8]
    }
}

fn guarded<const W: usize, const B: usize>(words: &[u64; W]) -> [u8; B] {
    let mut bytes = [0u8; B];
    bytes[0] = kani::any();
    // SAFETY: callers provide W*8 payload bytes after the prefix, plus an optional guard.
    unsafe {
        std::ptr::copy_nonoverlapping(
            words.as_ptr().cast::<u8>(),
            bytes.as_mut_ptr().add(1),
            W * 8,
        );
    }
    bytes
}
