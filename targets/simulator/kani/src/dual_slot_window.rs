//! Two source-offset partial writes with initialized, separate current/next slots.
//! Logical widths 61/128 occupy 8/16 native bytes. Current and next initially have
//! identical arbitrary bits, including padding. Both writes update next directly
//! and log replacements for current. Each RHS is arbitrary disjoint data or an
//! exact alias of current, which stays unchanged until commit. Partial updates
//! therefore forward through next while RHS reads retain the pre-commit value.
//! This conditional transition follows IEEE 1800-2023 4.6 and 10.4.2; IR analysis,
//! next initialization and routing to this layout remain caller preconditions.
//! Windows (amount,width,source bit) are (5,13,17) then (11,9,3) for width 61,
//! and (60,12,7) then (68,10,57) for width 128. The latter crosses a word boundary.
//! Disjoint source pointers expose 8/16 bytes; current and next are adjacent in an 18/34
//! byte allocation with outer guards. Log base is 1, while next starts at 9/17.
//! Narrow capacities 2/3 avoid growth. Real stores and both real commits check
//! staging, final current/next, ordered per-record notifications, guards and live
//! log/source frames against scalar masks and an independent bit-origin model.
//! Constructors and operations are not stubbed. Allocation succeeds on one thread.
//! Other windows, packed/log-only layouts, uninitialized next slots, longer
//! histories, partial pointer overlaps and four-state execution are excluded.
use veryl_formal_semantics::words;
use veryl_simulator::{
    ir::{WriteLogBuffer, ff_commit_from_log, ff_commit_from_log_marking},
    wide_ops,
};

#[cfg(not(all(target_endian = "little", target_pointer_width = "64")))]
compile_error!("dual-slot proofs require a little-endian 64-bit target");

#[kani::proof]
#[kani::unwind(18)]
fn dual_slot_partial61_matches_reference() {
    let r = history::<false, 9, 18>();
    kani::cover!(
        !r.alias_a && !r.alias_b,
        "dual61 disjoint sources are reachable"
    );
    kani::cover!(
        r.alias_a && r.alias_b,
        "dual61 both current aliases are reachable"
    );
    kani::cover!(
        !r.alias_a && r.alias_b,
        "dual61 second current alias is reachable"
    );
    kani::cover!(
        r.alias_a && !r.alias_b,
        "dual61 first current alias is reachable"
    );
    kani::cover!(
        r.alias_b && ((r.a ^ r.current) >> 3) & 511 != 0,
        "dual61 RHS current differs from updated next"
    );
    kani::cover!(r.events == 0, "dual61 unchanged records emit no events");
    kani::cover!(
        r.changes[0] && !r.changes[1] && r.events == 1,
        "dual61 only first record changes"
    );
    kani::cover!(
        !r.changes[0] && r.changes[1] && r.events == 1,
        "dual61 only second record changes"
    );
    kani::cover!(r.events == 2, "dual61 both records change");
    kani::cover!(
        r.a != r.current && r.b == r.current && r.events == 2,
        "dual61 restoring current still emits two events"
    );
    kani::cover!(
        ((r.a ^ r.current) >> 5) & 63 != 0 && (r.a ^ r.b) & (63 << 5) == 0,
        "dual61 first-only bits survive the second update"
    );
    kani::cover!(
        (r.a ^ r.b) & (127 << 11) != 0,
        "dual61 last writer replaces overlapping bits"
    );
    kani::cover!(r.byte == 0, "dual61 prefix guard is reachable");
    kani::cover!(r.byte == 17, "dual61 suffix guard is reachable");
    kani::cover!(r.byte == 8, "dual61 current padding is reachable");
    kani::cover!(r.byte == 16, "dual61 next padding is reachable");
}

#[kani::proof]
#[kani::unwind(18)]
fn dual_slot_partial128_matches_reference() {
    let r = history::<true, 17, 34>();
    kani::cover!(
        !r.alias_a && !r.alias_b,
        "dual128 disjoint sources are reachable"
    );
    kani::cover!(
        r.alias_a && r.alias_b,
        "dual128 both current aliases are reachable"
    );
    kani::cover!(
        !r.alias_a && r.alias_b,
        "dual128 second current alias is reachable"
    );
    kani::cover!(
        r.alias_a && !r.alias_b,
        "dual128 first current alias is reachable"
    );
    kani::cover!(
        r.alias_b && ((r.a ^ r.current) >> 57) & 1023 != 0,
        "dual128 RHS current differs from updated next"
    );
    kani::cover!(r.events == 0, "dual128 unchanged records emit no events");
    kani::cover!(r.events == 1, "dual128 one changing record is reachable");
    kani::cover!(r.events == 2, "dual128 two changing records are reachable");
    kani::cover!(
        r.events == 3,
        "dual128 three ordered change events are reachable"
    );
    kani::cover!(
        r.a != r.current && r.b == r.current && r.events == 2,
        "dual128 restoring current still emits two events"
    );
    kani::cover!(
        (r.a ^ r.current) & (255u128 << 60) != 0 && (r.a ^ r.b) & (255u128 << 60) == 0,
        "dual128 first-only bits survive the second update"
    );
    kani::cover!(
        (r.a ^ r.b) & (15u128 << 68) != 0,
        "dual128 last writer replaces overlapping bits"
    );
    kani::cover!(r.byte == 0, "dual128 prefix guard is reachable");
    kani::cover!(r.byte == 33, "dual128 suffix guard is reachable");
    kani::cover!(r.byte == 16, "dual128 current last byte is reachable");
    kani::cover!(r.byte == 32, "dual128 next last byte is reachable");
}

struct Observation {
    alias_a: bool,
    alias_b: bool,
    current: u128,
    a: u128,
    b: u128,
    changes: [bool; 3],
    events: usize,
    byte: usize,
}

fn history<const WIDE: bool, const S: usize, const B: usize>() -> Observation {
    let initial: [u64; 2] = kani::any();
    let rhs_a: [u64; 2] = kani::any();
    let rhs_b: [u64; 2] = kani::any();
    let alias_a: bool = kani::any();
    let alias_b: bool = kani::any();
    let nb = if WIDE { 16 } else { 8 };
    let dw = if WIDE { 128 } else { 61 };
    let source_a = source_bytes::<S>(&rhs_a, nb);
    let source_b = source_bytes::<S>(&rhs_b, nb);
    let source_a_before = source_a;
    let source_b_before = source_b;
    let mut memory = [0u8; B];
    memory[0] = kani::any();
    memory[B - 1] = kani::any();
    unsafe {
        // SAFETY: the two native slots are disjoint and surrounded by guards.
        std::ptr::copy_nonoverlapping(
            initial.as_ptr().cast::<u8>(),
            memory.as_mut_ptr().add(1),
            nb,
        );
        std::ptr::copy_nonoverlapping(
            initial.as_ptr().cast::<u8>(),
            memory.as_mut_ptr().add(1 + nb),
            nb,
        );
    }
    let before = memory;
    let current = if WIDE {
        packed(initial)
    } else {
        u128::from(initial[0])
    };
    let input_a = if alias_a { current } else { packed(rhs_a) };
    let input_b = if alias_b { current } else { packed(rhs_b) };
    let (amount_a, width_a, bit_a, amount_b, width_b, bit_b) = if WIDE {
        (60, 12, 7, 68, 10, 57)
    } else {
        (5, 13, 17, 11, 9, 3)
    };
    let a = patch(current, input_a, amount_a, width_a, bit_a);
    let b = patch(a, input_b, amount_b, width_b, bit_b);
    let records = if WIDE { 3 } else { 2 };
    let offsets = if WIDE { [1u32, 9, 9] } else { [1u32, 1, 0] };
    let payloads = if WIDE {
        [a as u64, (a >> 64) as u64, (b >> 64) as u64]
    } else {
        [a as u64, b as u64, 0]
    };
    let changes = if WIDE {
        [
            a as u64 != current as u64,
            (a >> 64) != (current >> 64),
            (a >> 64) != (b >> 64),
        ]
    } else {
        [a != current, a != b, false]
    };
    let mut expected_events = [None; 3];
    let mut expected_count = 0;
    for i in 0..records {
        if changes[i] {
            expected_events[expected_count] = Some((offsets[i] as usize, 8));
            expected_count += 1;
        }
    }
    let mut log = WriteLogBuffer::with_capacity(records, 0);
    unsafe {
        let base = memory.as_mut_ptr();
        let old = base.add(1).cast_const();
        let next = base.add(1 + nb);
        let src_a = if alias_a {
            old
        } else {
            source_a.as_ptr().add(1)
        };
        wide_ops::wide_window_store_from(
            &mut log,
            next,
            src_a,
            amount_a,
            width_a | (dw << 32),
            1 | (1u64 << 32),
            bit_a,
        );
        let src_b = if alias_b {
            old
        } else {
            source_b.as_ptr().add(1)
        };
        wide_ops::wide_window_store_from(
            &mut log,
            next,
            src_b,
            amount_b,
            width_b | (dw << 32),
            1 | (1u64 << 32),
            bit_b,
        );
    }
    assert!(
        log.narrow_count() == records as u32 && log.wide_count() == 0,
        "simulator dual-slot partial window semantics mismatch"
    );
    let entries = log.narrow_entries_slice();
    let saved = [entries[0], entries[1], entries[if WIDE { 2 } else { 1 }]];
    for i in 0..records {
        assert!(
            entries[i].offset == offsets[i]
                && entries[i].width_class == 8
                && entries[i].mask_xz == 0
                && entries[i].payload == payloads[i],
            "simulator dual-slot partial window semantics mismatch"
        );
    }
    let bit = usize::from(kani::any::<u8>()) % (nb * 8);
    assert!(
        ((a >> bit) & 1 != 0)
            == words::window_stored_from_bit(
                &current.to_le_bytes(),
                &input_a.to_le_bytes(),
                dw as usize,
                amount_a,
                width_a as usize,
                bit_a as usize,
                bit
            )
            && ((b >> bit) & 1 != 0)
                == words::window_stored_from_bit(
                    &a.to_le_bytes(),
                    &input_b.to_le_bytes(),
                    dw as usize,
                    amount_b,
                    width_b as usize,
                    bit_b as usize,
                    bit
                ),
        "simulator dual-slot partial window semantics mismatch"
    );
    let capacity = (log.narrow_capacity(), log.wide_capacity());
    let mut plain = memory;
    let mut marked = memory;
    ff_commit_from_log(&mut plain, &log);
    let mut events = 0;
    ff_commit_from_log_marking(&mut marked, &log, &mut |off, size| {
        assert!(
            expected_events.get(events).copied().flatten() == Some((off, size)),
            "simulator dual-slot partial window semantics mismatch"
        );
        events += 1;
    });
    assert!(
        events == expected_count,
        "simulator dual-slot partial window semantics mismatch"
    );
    let byte = usize::from(kani::any::<u8>()) % B;
    let staged = state_byte(&before, current, b, nb, byte);
    let final_byte = state_byte(&before, b, b, nb, byte);
    assert!(
        memory[byte] == staged
            && plain[byte] == final_byte
            && marked[byte] == final_byte
            && source_a == source_a_before
            && source_b == source_b_before
            && log.narrow_count() == records as u32
            && log.wide_count() == 0
            && capacity == (log.narrow_capacity(), log.wide_capacity()),
        "simulator dual-slot partial window semantics mismatch"
    );
    let observed = usize::from(kani::any::<u8>()) % records;
    let after = &log.narrow_entries_slice()[observed];
    assert!(
        after.offset == saved[observed].offset
            && after.width_class == saved[observed].width_class
            && after.mask_xz == saved[observed].mask_xz
            && after.payload == saved[observed].payload,
        "simulator dual-slot partial window semantics mismatch"
    );
    Observation {
        alias_a,
        alias_b,
        current,
        a,
        b,
        changes,
        events,
        byte,
    }
}

fn packed(words: [u64; 2]) -> u128 {
    u128::from(words[0]) | (u128::from(words[1]) << 64)
}

// Callers use nonempty masks wholly inside the logical destination and source.
fn patch(old: u128, source: u128, amount: u64, width: u64, source_bit: u64) -> u128 {
    let low = (1u128 << width) - 1;
    (old & !(low << amount)) | (((source >> source_bit) & low) << amount)
}

fn state_byte<const B: usize>(
    before: &[u8; B],
    current: u128,
    next: u128,
    nb: usize,
    byte: usize,
) -> u8 {
    if byte == 0 || byte == B - 1 {
        before[byte]
    } else {
        let value = if byte <= nb { current } else { next };
        (value >> (((byte - 1) % nb) * 8)) as u8
    }
}

fn source_bytes<const S: usize>(data: &[u64; 2], nb: usize) -> [u8; S] {
    let mut bytes = [0u8; S];
    bytes[0] = kani::any();
    // SAFETY: callers provide nb bytes after the prefix and data contains 16.
    unsafe {
        std::ptr::copy_nonoverlapping(data.as_ptr().cast::<u8>(), bytes.as_mut_ptr().add(1), nb);
    }
    bytes
}
