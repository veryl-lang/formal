//! Initialized dual-slot histories in the 130-bit, 24-byte wide-log route.
//! Two source-offset writes update next in place, read RHS values from disjoint
//! 24-byte sources or old current, then use both real commit wrappers. IEEE
//! 1800-2023 4.6/10.4.2 give ordered NBA updates and pre-commit RHS evaluation.
//! Clause 11.5.1 restricts a partially out-of-range write to logical in-range bits.
//! Separate current/next initialization, IR classification and routing, valid
//! pointers, successful allocation, and single-threaded execution are premises.
//! All 192 storage bits are arbitrary, including 62 padding bits. A fixed pair
//! of overlapping windows crosses bits 63/64, or clips at bits 129/130. The
//! first pair emits wide records of 16 then 8 bytes; the tail pair emits two
//! 16-byte records. Their exact capacities avoid pool growth. Independent bit
//! origins check staging, records, final memory, and per-record notifications;
//! guarded sources and log frames are preserved. No operations are stubbed.
//! Other windows, logical widths, longer histories, partial pointer overlaps,
//! four-state values and producer layout selection are outside this theorem.
use veryl_formal_semantics::words;
use veryl_simulator::{
    ir::{WriteLogBuffer, ff_commit_from_log, ff_commit_from_log_marking},
    wide_ops,
};

#[cfg(not(all(target_endian = "little", target_pointer_width = "64")))]
compile_error!("wide dual-slot proofs require a little-endian 64-bit target");

#[kani::proof]
#[kani::unwind(57)]
fn dual_slot_wide130_matches_reference() {
    let r = history::<false>();
    kani::cover!(
        !r.alias_a && !r.alias_b,
        "dual130 disjoint sources are reachable"
    );
    kani::cover!(
        r.alias_a && r.alias_b,
        "dual130 both old-current aliases are reachable"
    );
    kani::cover!(
        r.alias_a && !r.alias_b,
        "dual130 first old-current alias is reachable"
    );
    kani::cover!(
        !r.alias_a && r.alias_b,
        "dual130 second old-current alias is reachable"
    );
    kani::cover!(
        r.old_rhs_distinct,
        "dual130 old-current RHS differs from updated next"
    );
    kani::cover!(r.events == 0, "dual130 unchanged records emit no events");
    kani::cover!(
        r.changes[0] && !r.changes[1] && r.events == 1,
        "dual130 only the first wide record changes"
    );
    kani::cover!(
        !r.changes[0] && r.changes[1] && r.events == 1,
        "dual130 only the second wide record changes"
    );
    kani::cover!(
        r.events == 2,
        "dual130 two ordered wide events are reachable"
    );
    kani::cover!(
        r.restores && r.events == 2,
        "dual130 restoration still emits two wide events"
    );
    kani::cover!(
        r.first_only_changed,
        "dual130 first-only bits survive the last writer"
    );
    kani::cover!(
        r.overlap_changed,
        "dual130 last writer replaces overlapping bits"
    );
    kani::cover!(r.byte == 0, "dual130 prefix guard is reachable");
    kani::cover!(r.byte == 49, "dual130 suffix guard is reachable");
    kani::cover!(r.byte == 24, "dual130 current padding byte is reachable");
    kani::cover!(r.byte == 48, "dual130 next padding byte is reachable");
}
#[kani::proof]
#[kani::unwind(57)]
fn dual_slot_tail130_matches_reference() {
    let r = history::<true>();
    kani::cover!(
        !r.alias_a && !r.alias_b,
        "tail130 disjoint sources are reachable"
    );
    kani::cover!(
        r.alias_a && r.alias_b,
        "tail130 both old-current aliases are reachable"
    );
    kani::cover!(
        r.alias_a && !r.alias_b,
        "tail130 first old-current alias is reachable"
    );
    kani::cover!(
        !r.alias_a && r.alias_b,
        "tail130 second old-current alias is reachable"
    );
    kani::cover!(
        r.old_rhs_distinct,
        "tail130 old-current RHS differs from updated next"
    );
    kani::cover!(r.events == 0, "tail130 unchanged records emit no events");
    kani::cover!(
        r.changes[0] && !r.changes[1] && r.events == 1,
        "tail130 only the first wide record changes"
    );
    kani::cover!(
        !r.changes[0] && r.changes[1] && r.events == 1,
        "tail130 only the second wide record changes"
    );
    kani::cover!(
        r.events == 2,
        "tail130 two ordered wide events are reachable"
    );
    kani::cover!(
        r.restores && r.events == 2,
        "tail130 restoration still emits two wide events"
    );
    kani::cover!(
        r.first_only_changed,
        "tail130 first-only bits survive the last writer"
    );
    kani::cover!(
        r.overlap_changed,
        "tail130 last writer replaces overlapping bits"
    );
    kani::cover!(
        r.logical_last_set,
        "tail130 last logical bit can be written"
    );
    kani::cover!(
        r.dirty_padding,
        "tail130 nonzero storage padding is preserved"
    );
    kani::cover!(r.byte == 0, "tail130 prefix guard is reachable");
    kani::cover!(r.byte == 49, "tail130 suffix guard is reachable");
    kani::cover!(r.byte == 24, "tail130 current padding byte is reachable");
    kani::cover!(r.byte == 48, "tail130 next padding byte is reachable");
}
struct Observation {
    alias_a: bool,
    alias_b: bool,
    old_rhs_distinct: bool,
    changes: [bool; 2],
    events: usize,
    restores: bool,
    first_only_changed: bool,
    overlap_changed: bool,
    logical_last_set: bool,
    dirty_padding: bool,
    byte: usize,
}
fn source(data: [u64; 3]) -> [u8; 26] {
    let mut result = [0u8; 26];
    result[0] = kani::any();
    result[25] = kani::any();
    for (i, word) in data.into_iter().enumerate() {
        result[1 + i * 8..1 + (i + 1) * 8].copy_from_slice(&word.to_le_bytes());
    }
    result
}
fn patch(old: &[u8; 24], rhs: &[u8; 24], amount: u64, width: usize, source_bit: usize) -> [u8; 24] {
    let mut result = *old;
    // The reference assigns individual logical bits; it has no production
    // word decomposition, packed width/flags, staging pointer, or log record.
    for i in 0..width {
        let bit = amount as usize + i;
        if bit < 130 {
            let value =
                words::window_stored_from_bit(old, rhs, 130, amount, width, source_bit, bit);
            result[bit / 8] =
                (result[bit / 8] & !(1 << (bit % 8))) | (u8::from(value) << (bit % 8));
        }
    }
    result
}
fn bit(bytes: &[u8; 24], index: usize) -> bool {
    bytes[index / 8] & (1 << (index % 8)) != 0
}
fn field(bytes: &[u8; 24], start: usize, width: usize) -> u16 {
    let mut result = 0u16;
    for i in 0..width {
        result |= u16::from(bit(bytes, start + i)) << i;
    }
    result
}
fn history<const TAIL: bool>() -> Observation {
    let initial: [u64; 3] = kani::any();
    let rhs_a: [u64; 3] = kani::any();
    let rhs_b: [u64; 3] = kani::any();
    let alias_a: bool = kani::any();
    let alias_b: bool = kani::any();
    let external_a = source(rhs_a);
    let external_b = source(rhs_b);
    let external_a_before = external_a;
    let external_b_before = external_b;
    let mut old = [0u8; 24];
    for (i, word) in initial.into_iter().enumerate() {
        old[i * 8..(i + 1) * 8].copy_from_slice(&word.to_le_bytes());
    }
    let mut input_a = [0u8; 24];
    let mut input_b = [0u8; 24];
    input_a.copy_from_slice(if alias_a { &old } else { &external_a[1..25] });
    input_b.copy_from_slice(if alias_b { &old } else { &external_b[1..25] });
    let (amount_a, width_a, source_a, amount_b, width_b, source_b) = if TAIL {
        (124, 12, 63, 126, 8, 123)
    } else {
        (60, 12, 7, 68, 10, 57)
    };
    let a = patch(&old, &input_a, amount_a, width_a, source_a);
    let b = patch(&a, &input_b, amount_b, width_b, source_b);
    let offsets = if TAIL { [9u32, 9] } else { [1u32, 9] };
    let starts = if TAIL { [8usize, 8] } else { [0usize, 8] };
    let lengths = if TAIL { [16usize, 16] } else { [16usize, 8] };
    let changes = [
        a[starts[0]..starts[0] + lengths[0]] != old[starts[0]..starts[0] + lengths[0]],
        b[starts[1]..starts[1] + lengths[1]] != a[starts[1]..starts[1] + lengths[1]],
    ];
    let mut expected_events = [None; 2];
    let mut expected_count = 0;
    for i in 0..2 {
        if changes[i] {
            expected_events[expected_count] = Some((offsets[i] as usize, lengths[i]));
            expected_count += 1;
        }
    }
    let mut memory = [0u8; 50];
    memory[0] = kani::any();
    memory[49] = kani::any();
    memory[1..25].copy_from_slice(&old);
    memory[25..49].copy_from_slice(&old);
    let before = memory;
    let mut log = WriteLogBuffer::with_capacity(0, 2);
    unsafe {
        // Each source is valid for 24 bytes; current and next are disjoint live
        // 24-byte slots. Both raw slot pointers derive from this one base.
        let base = memory.as_mut_ptr();
        let current = base.add(1).cast_const();
        let next = base.add(25);
        let src_a = if alias_a {
            current
        } else {
            external_a.as_ptr().add(1)
        };
        let src_b = if alias_b {
            current
        } else {
            external_b.as_ptr().add(1)
        };
        wide_ops::wide_window_store_from(
            &mut log,
            next,
            src_a,
            amount_a,
            (130u64 << 32) | width_a as u64,
            1 | (1u64 << 32),
            source_a as u64,
        );
        wide_ops::wide_window_store_from(
            &mut log,
            next,
            src_b,
            amount_b,
            (130u64 << 32) | width_b as u64,
            1 | (1u64 << 32),
            source_b as u64,
        );
    }
    assert!(
        log.narrow_count() == 0 && log.wide_count() == 2,
        "simulator wide dual-slot window semantics mismatch"
    );
    let entries = log.wide_entries_slice();
    // Keep the saved state independent of the over-aligned record aggregate.
    // Every logical field, including all padding/payload bytes, is preserved.
    let saved_offsets = [entries[0].offset, entries[1].offset];
    let saved_sizes = [entries[0].native_bytes, entries[1].native_bytes];
    let saved_padding = [entries[0]._pad, entries[1]._pad];
    let saved_payloads = [entries[0].payload, entries[1].payload];
    let record = usize::from(kani::any::<u8>()) % 2;
    let payload_byte = usize::from(kani::any::<u8>()) % 16;
    let expected_payload = if record == 0 { a } else { b };
    for i in 0..2 {
        assert!(
            entries[i].offset == offsets[i] && usize::from(entries[i].native_bytes) == lengths[i],
            "simulator wide dual-slot window semantics mismatch"
        );
    }
    if payload_byte < lengths[record] {
        assert!(
            entries[record].payload[payload_byte]
                == expected_payload[starts[record] + payload_byte],
            "simulator wide dual-slot window semantics mismatch"
        );
    }
    let capacity = (log.narrow_capacity(), log.wide_capacity());
    let mut plain = memory;
    let mut marked = memory;
    ff_commit_from_log(&mut plain, &log);
    let mut events = 0;
    ff_commit_from_log_marking(&mut marked, &log, &mut |off, size| {
        assert!(
            expected_events.get(events).copied().flatten() == Some((off, size)),
            "simulator wide dual-slot window semantics mismatch"
        );
        events += 1;
    });
    assert!(
        events == expected_count,
        "simulator wide dual-slot window semantics mismatch"
    );
    let byte = usize::from(kani::any::<u8>()) % 50;
    let staged = if byte == 0 || byte == 49 {
        before[byte]
    } else if byte < 25 {
        old[byte - 1]
    } else {
        b[byte - 25]
    };
    let final_byte = if byte == 0 || byte == 49 {
        before[byte]
    } else if byte < 25 {
        b[byte - 1]
    } else {
        b[byte - 25]
    };
    assert!(
        memory[byte] == staged
            && plain[byte] == final_byte
            && marked[byte] == final_byte
            && external_a == external_a_before
            && external_b == external_b_before,
        "simulator wide dual-slot window semantics mismatch"
    );
    assert!(
        log.narrow_count() == 0
            && log.wide_count() == 2
            && capacity == (log.narrow_capacity(), log.wide_capacity()),
        "simulator wide dual-slot window semantics mismatch"
    );
    let after = &log.wide_entries_slice()[record];
    assert!(
        after.offset == saved_offsets[record],
        "simulator wide dual-slot window semantics mismatch"
    );
    assert!(
        after.native_bytes == saved_sizes[record],
        "simulator wide dual-slot window semantics mismatch"
    );
    assert!(
        after._pad == saved_padding[record],
        "simulator wide dual-slot window semantics mismatch"
    );
    assert!(
        after.payload == saved_payloads[record],
        "simulator wide dual-slot window semantics mismatch"
    );
    let clipped_b = width_b.min(130 - amount_b as usize);
    Observation {
        alias_a,
        alias_b,
        old_rhs_distinct: alias_b
            && field(&old, source_b, clipped_b) != field(&a, source_b, clipped_b),
        changes,
        events,
        restores: a != old && b == old,
        first_only_changed: bit(&a, amount_a as usize + 1) != bit(&old, amount_a as usize + 1)
            && bit(&b, amount_a as usize + 1) == bit(&a, amount_a as usize + 1),
        overlap_changed: bit(&b, amount_b as usize) != bit(&a, amount_b as usize),
        logical_last_set: bit(&b, 129),
        dirty_padding: bit(&old, 191) && bit(&b, 191),
        byte,
    }
}
