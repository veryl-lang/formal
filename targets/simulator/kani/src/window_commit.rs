//! Real window stores followed by both real commit wrappers. The fixed binary
//! windows are 128 bits at bit 63 in 192 bits, and 512 bits at bit 64 in 576 bits.
//! Every source/destination bit, direct/log-only mode and disjoint/exact-base alias
//! is quantified. Pointers have byte offset 1; disjoint sources expose exactly
//! 16/64 readable bytes. Log offsets address the same guarded destination at base 1.
//! Empty wide capacity 1 admits the first record and forces growth for two chunks.
//! Final bytes, staging, ordered per-record change events, source guards and live
//! log offsets, lengths and payloads are checked. Events compare the staging state,
//! so direct stores produce no commit event, while unchanged log-only stores also
//! produce none. Reference bits use independent bit origins; change predicates
//! use a scalar 128-bit slice or equality of independently generated input words.
//! No production operation or constructor is stubbed. Execution is single-threaded
//! and allocation succeeds. Other windows, base offsets, existing histories,
//! partial overlaps, four-state execution and source NBA scheduling are excluded.
use veryl_formal_semantics::words;
use veryl_simulator::{
    ir::{WriteLogBuffer, ff_commit_from_log, ff_commit_from_log_marking},
    wide_ops,
};

#[cfg(not(all(target_endian = "little", target_pointer_width = "64")))]
compile_error!("window commit proofs require a little-endian 64-bit target");

#[kani::proof]
#[kani::unwind(66)]
fn window_commit_unaligned_matches_reference() {
    let sw: [u64; 2] = kani::any();
    let dw: [u64; 3] = kani::any();
    let source = source_bytes::<2, 17>(&sw);
    let source_before = source;
    let mut destination = destination_bytes::<3, 26>(&dw);
    let before = destination;
    let direct: bool = kani::any();
    let alias: bool = kani::any();
    let mut log = WriteLogBuffer::with_capacity(0, 1);
    unsafe {
        let dst = destination.as_mut_ptr().add(1);
        let src = if alias {
            dst.cast_const()
        } else {
            source.as_ptr().add(1)
        };
        wide_ops::wide_window_store(
            &mut log,
            dst,
            src,
            63,
            128 | (192u64 << 32),
            1 | ((direct as u64) << 32),
        );
    }
    assert!(
        log.wide_count() == 1 && log.narrow_count() == 0,
        "simulator window commit semantics mismatch"
    );
    let entry = &log.wide_entries_slice()[0];
    assert!(
        entry.offset == 1 && entry.native_bytes == 24,
        "simulator window commit semantics mismatch"
    );
    let input = if alias {
        &before[1..17]
    } else {
        &source_before[1..]
    };
    // Extract destination bits 63..191 into a scalar without using word-store code.
    let old_window = u128::from(dw[0] >> 63) | (u128::from(dw[1]) << 1) | (u128::from(dw[2]) << 65);
    let input_words = if alias { [dw[0], dw[1]] } else { sw };
    let input_value = u128::from(input_words[0]) | (u128::from(input_words[1]) << 64);
    let differs = old_window != input_value;
    let events = [
        if !direct && differs {
            Some((1, 24))
        } else {
            None
        },
        None,
    ];
    let r = commit_and_check(destination, &before, &log, direct, events, |byte, bit| {
        if (1..25).contains(&byte) {
            words::window_stored_bit(&before[1..25], input, 192, 63, 128, (byte - 1) * 8 + bit)
        } else {
            words::bit(&before, byte * 8 + bit)
        }
    });
    assert!(
        source == source_before,
        "simulator window commit semantics mismatch"
    );
    kani::cover!(
        direct && alias,
        "unaligned commit direct alias is reachable"
    );
    kani::cover!(
        !direct && alias,
        "unaligned commit log-only alias is reachable"
    );
    kani::cover!(
        direct && !alias,
        "unaligned commit direct external is reachable"
    );
    kani::cover!(
        !direct && !alias,
        "unaligned commit log-only external is reachable"
    );
    kani::cover!(
        !direct && differs && r.events == 1,
        "unaligned commit emits a change event"
    );
    kani::cover!(
        !direct && !differs && r.events == 0,
        "unaligned unchanged commit emits no event"
    );
    kani::cover!(
        direct && differs && r.events == 0,
        "unaligned direct commit emits no duplicate event"
    );
    kani::cover!(r.byte == 0, "unaligned commit prefix guard is reachable");
    kani::cover!(r.byte == 25, "unaligned commit suffix guard is reachable");
    kani::cover!(
        r.byte == 24 && r.bit == 6,
        "unaligned commit last source bit is reachable"
    );
}

#[kani::proof]
#[kani::unwind(66)]
fn window_commit_chunks_matches_reference() {
    let sw: [u64; 8] = kani::any();
    let dw: [u64; 9] = kani::any();
    let source = source_bytes::<8, 65>(&sw);
    let source_before = source;
    let mut destination = destination_bytes::<9, 74>(&dw);
    let before = destination;
    let direct: bool = kani::any();
    let alias: bool = kani::any();
    let mut log = WriteLogBuffer::with_capacity(0, 1);
    unsafe {
        let dst = destination.as_mut_ptr().add(1);
        let src = if alias {
            dst.cast_const()
        } else {
            source.as_ptr().add(1)
        };
        wide_ops::wide_window_store(
            &mut log,
            dst,
            src,
            64,
            512 | (576u64 << 32),
            1 | ((direct as u64) << 32),
        );
    }
    assert!(
        log.wide_count() == 2 && log.wide_capacity() >= 2 && log.narrow_count() == 0,
        "simulator window commit semantics mismatch"
    );
    let entries = log.wide_entries_slice();
    assert!(
        entries[0].offset == 9
            && entries[0].native_bytes == 56
            && entries[1].offset == 65
            && entries[1].native_bytes == 8,
        "simulator window commit semantics mismatch"
    );
    let input = if alias {
        &before[1..65]
    } else {
        &source_before[1..]
    };
    let mut first_differs = false;
    for word in 0..7 {
        first_differs |= dw[word + 1] != if alias { dw[word] } else { sw[word] };
    }
    let second_differs = dw[8] != if alias { dw[7] } else { sw[7] };
    let first = !direct && first_differs;
    let second = !direct && second_differs;
    let events = match (first, second) {
        (true, true) => [Some((9, 56)), Some((65, 8))],
        (true, false) => [Some((9, 56)), None],
        (false, true) => [Some((65, 8)), None],
        (false, false) => [None, None],
    };
    let r = commit_and_check(destination, &before, &log, direct, events, |byte, bit| {
        let value = if (9..73).contains(&byte) {
            input[byte - 9]
        } else {
            before[byte]
        };
        value & (1 << bit) != 0
    });
    assert!(
        source == source_before,
        "simulator window commit semantics mismatch"
    );
    kani::cover!(direct && alias, "chunk commit direct alias is reachable");
    kani::cover!(!direct && alias, "chunk commit log-only alias is reachable");
    kani::cover!(
        direct && !alias,
        "chunk commit direct external is reachable"
    );
    kani::cover!(
        !direct && !alias,
        "chunk commit log-only external is reachable"
    );
    kani::cover!(
        first && second && r.events == 2,
        "chunk commit emits two ordered events"
    );
    kani::cover!(
        first && !second && r.events == 1,
        "chunk commit changes only first record"
    );
    kani::cover!(
        !first && second && r.events == 1,
        "chunk commit changes only second record"
    );
    kani::cover!(
        !direct && !first_differs && !second_differs && r.events == 0,
        "chunk unchanged commit emits no event"
    );
    kani::cover!(
        direct && (first_differs || second_differs) && r.events == 0,
        "chunk direct commit emits no duplicate event"
    );
    kani::cover!(r.byte == 0, "chunk commit prefix guard is reachable");
    kani::cover!(r.byte == 73, "chunk commit suffix guard is reachable");
    kani::cover!(r.byte == 64, "chunk commit first record end is reachable");
    kani::cover!(
        r.byte == 65,
        "chunk commit second record start is reachable"
    );
}

struct Observation {
    byte: usize,
    bit: usize,
    events: usize,
}

fn commit_and_check<const N: usize>(
    staging: [u8; N],
    before: &[u8; N],
    log: &WriteLogBuffer,
    direct: bool,
    expected_events: [Option<(usize, usize)>; 2],
    expected_bit: impl Fn(usize, usize) -> bool,
) -> Observation {
    let record_count = log.wide_count();
    let capacity = log.wide_capacity();
    let saved = [
        log.wide_entries_slice()[0],
        log.wide_entries_slice()[if record_count == 2 { 1 } else { 0 }],
    ];
    let mut plain = staging;
    let mut marked = staging;
    ff_commit_from_log(&mut plain, log);
    let mut count = 0;
    ff_commit_from_log_marking(&mut marked, log, &mut |off, nb| {
        assert!(
            expected_events.get(count).copied().flatten() == Some((off, nb)),
            "simulator window commit semantics mismatch"
        );
        count += 1;
    });
    let expected_count =
        usize::from(expected_events[0].is_some()) + usize::from(expected_events[1].is_some());
    assert!(
        count == expected_count,
        "simulator window commit semantics mismatch"
    );
    let byte = usize::from(kani::any::<u8>()) % N;
    let bit = usize::from(kani::any::<u8>() & 7);
    let expected = expected_bit(byte, bit);
    let staged = if direct {
        expected
    } else {
        words::bit(before, byte * 8 + bit)
    };
    assert!(
        words::bit(&staging, byte * 8 + bit) == staged
            && words::bit(&plain, byte * 8 + bit) == expected
            && words::bit(&marked, byte * 8 + bit) == expected,
        "simulator window commit semantics mismatch"
    );
    assert!(
        log.wide_count() == record_count
            && log.wide_capacity() == capacity
            && log.narrow_count() == 0,
        "simulator window commit semantics mismatch"
    );
    let entry = usize::from(kani::any::<bool>());
    let payload_byte = usize::from(kani::any::<u8>() % 56);
    if entry < record_count as usize {
        let after = &log.wide_entries_slice()[entry];
        assert!(
            after.offset == saved[entry].offset
                && after.native_bytes == saved[entry].native_bytes
                && after.payload[payload_byte] == saved[entry].payload[payload_byte],
            "simulator window commit semantics mismatch"
        );
    }
    Observation {
        byte,
        bit,
        events: count,
    }
}

fn source_bytes<const W: usize, const B: usize>(words: &[u64; W]) -> [u8; B] {
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

fn destination_bytes<const W: usize, const B: usize>(words: &[u64; W]) -> [u8; B] {
    let mut bytes = source_bytes::<W, B>(words);
    bytes[B - 1] = kani::any();
    bytes
}
