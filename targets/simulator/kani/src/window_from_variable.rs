//! Variable source-offset stores into a logical 130-bit, native 24-byte value.
//! Requested widths cover every u32; starts and source offsets cover every u64.
//! Active source intervals must fit 192 readable bits. The effective copied-width
//! partitions are 0..64 and65..130. Eight operation roots cover all
//! four direct/alias Boolean pairs in each partition. Small roots separate active/no-op inputs:
//! active starts0..129 and source offsets0..191 are generated as bytes and widened
//! to u64, covering every valid active input; no-op metadata stays arbitrary u64.
//! All storage bits, including padding, are arbitrary. Sources are disjoint or
//! exact aliases of destination; snapshots define RHS values before direct writes.
//! Independent bit origins check every value and payload bit. Change events compare
//! global192-bit scalar projections, separately tested against individual bits.
//! Real stores, appends and both real commits check staging, final values, guards,
//! sources, record spans and payloads, ordered notifications and live-record frames.
//! A checked exact record count is assigned its equal model value before commits
//! to keep symbolic producer histories out of the commit-loop bounds.
//! Two domain-witness roots use the same generator with both Booleans arbitrary
//! (operation roots fix each of the four possible pairs). Domain roots return
//! before production calls. Operation roots retain every assertion and all actual
//! event/live-record endpoint covers. Together these establish the full theorem.
//! Notification observation uses one-threaded globals fully reset before each
//! marking commit; no captured references or external observers are admitted.
//! IEEE1800-2023 11.5.1 restricts writes to logical in-range bits. Flags, physical
//! padding, log spans covering storage words and capacities are separate ABI rules.
//! One preallocated record avoids growth. A panic stub checks that wide-pool growth
//! is unreachable; reaching it fails with a message distinct from semantic checks.
//! Word loops unfold four times; the stable CBMC memcmp loop unfolds27 times.
//! Unwinding assertions check both bounds, including the25-byte source frame.
//! Temporary SmallVec construction uses the checked0..3-word inline summary;
//! window_scratch_refines_constructor compares it with the real constructor.
//! Its wrapper rejects construction in the no-op phase with an unconditional
//! panic, establishing that all no-op metadata returns before scratch allocation.
//! Exact allocation extents, successful allocation, one thread and a little-endian
//! 64-bit target are premises. Other widths, partial pointer overlaps, longer
//! histories, producer/source lowering and X/Z are outside this theorem.
use veryl_formal_semantics::words;
use veryl_simulator::{
    ir::{WriteLogBuffer, ff_commit_from_log, ff_commit_from_log_marking},
    wide_ops,
};
#[cfg(not(all(target_endian = "little", target_pointer_width = "64")))]
compile_error!("variable source-offset proofs require a little-endian 64-bit target");

#[kani::proof]
#[kani::unwind(4)]
#[kani::stub(veryl_simulator::ir::WriteLogBuffer::grow_wide_to, reject_wide_growth)]
#[kani::stub(smallvec::SmallVec::<[u64; 16]>::from_elem, guarded_word_scratch)]
fn source_offset_small_log_external_matches_reference() {
    let o = if kani::any::<bool>() {
        check::<false, true, true, 0>()
    } else {
        check::<false, false, true, 0>()
    };
    covers_log_outputs(&o);
}

#[kani::proof]
#[kani::unwind(4)]
#[kani::stub(veryl_simulator::ir::WriteLogBuffer::grow_wide_to, reject_wide_growth)]
#[kani::stub(smallvec::SmallVec::<[u64; 16]>::from_elem, guarded_word_scratch)]
fn source_offset_small_direct_external_matches_reference() {
    let o = if kani::any::<bool>() {
        check::<false, true, true, 1>()
    } else {
        check::<false, false, true, 1>()
    };
    covers_direct_outputs(&o);
}

#[kani::proof]
#[kani::unwind(4)]
#[kani::stub(veryl_simulator::ir::WriteLogBuffer::grow_wide_to, reject_wide_growth)]
#[kani::stub(smallvec::SmallVec::<[u64; 16]>::from_elem, guarded_word_scratch)]
fn source_offset_small_log_alias_matches_reference() {
    let o = if kani::any::<bool>() {
        check::<false, true, true, 2>()
    } else {
        check::<false, false, true, 2>()
    };
    covers_log_outputs(&o);
}

#[kani::proof]
#[kani::unwind(4)]
#[kani::stub(veryl_simulator::ir::WriteLogBuffer::grow_wide_to, reject_wide_growth)]
#[kani::stub(smallvec::SmallVec::<[u64; 16]>::from_elem, guarded_word_scratch)]
fn source_offset_small_direct_alias_matches_reference() {
    let o = if kani::any::<bool>() {
        check::<false, true, true, 3>()
    } else {
        check::<false, false, true, 3>()
    };
    covers_direct_outputs(&o);
}

#[kani::proof]
#[kani::unwind(4)]
#[kani::stub(veryl_simulator::ir::WriteLogBuffer::grow_wide_to, reject_wide_growth)]
#[kani::stub(smallvec::SmallVec::<[u64; 16]>::from_elem, guarded_word_scratch)]
fn source_offset_large_log_external_matches_reference() {
    let o = check::<true, true, true, 0>();
    covers_log_outputs(&o);
}

#[kani::proof]
#[kani::unwind(4)]
#[kani::stub(veryl_simulator::ir::WriteLogBuffer::grow_wide_to, reject_wide_growth)]
#[kani::stub(smallvec::SmallVec::<[u64; 16]>::from_elem, guarded_word_scratch)]
fn source_offset_large_direct_external_matches_reference() {
    let o = check::<true, true, true, 1>();
    covers_direct_outputs(&o);
}

#[kani::proof]
#[kani::solver(kissat)]
#[kani::unwind(4)]
#[kani::stub(veryl_simulator::ir::WriteLogBuffer::grow_wide_to, reject_wide_growth)]
#[kani::stub(smallvec::SmallVec::<[u64; 16]>::from_elem, guarded_word_scratch)]
fn source_offset_large_log_alias_matches_reference() {
    let o = check::<true, true, true, 2>();
    covers_log_outputs(&o);
}

#[kani::proof]
#[kani::unwind(4)]
#[kani::stub(veryl_simulator::ir::WriteLogBuffer::grow_wide_to, reject_wide_growth)]
#[kani::stub(smallvec::SmallVec::<[u64; 16]>::from_elem, guarded_word_scratch)]
fn source_offset_large_direct_alias_matches_reference() {
    let o = check::<true, true, true, 3>();
    covers_direct_outputs(&o);
}
#[kani::proof]
#[kani::unwind(4)]
fn small_source_domain_is_reachable() {
    let o = if kani::any::<bool>() {
        check::<false, true, false, 4>()
    } else {
        check::<false, false, false, 4>()
    };
    covers_inputs(&o);
    covers_small(&o);
}
#[kani::proof]
#[kani::unwind(4)]
fn large_source_domain_is_reachable() {
    let o = check::<true, true, false, 4>();
    covers_inputs(&o);
    covers_large(&o);
}
struct Observation {
    amount: u64,
    width: u32,
    source_bit: u64,
    copied: usize,
    direct: bool,
    alias: bool,
    changed: bool,
    events: usize,
    byte: usize,
    dirty_padding: bool,
    frame_byte: usize,
}
fn covers_inputs(o: &Observation) {
    kani::cover!(
        o.copied > 0 && o.direct && o.alias,
        "variable source direct alias is reachable"
    );
    kani::cover!(
        o.copied > 0 && o.direct && !o.alias,
        "variable source direct disjoint data is reachable"
    );
    kani::cover!(
        o.copied > 0 && !o.direct && o.alias,
        "variable source log-only alias is reachable"
    );
    kani::cover!(
        o.copied > 0 && !o.direct && !o.alias,
        "variable source log-only disjoint data is reachable"
    );
    kani::cover!(
        o.copied > 0 && o.amount == 0,
        "variable source destination first bit is reachable"
    );
    kani::cover!(
        o.copied > 0 && o.amount < 64 && o.amount + o.copied as u64 > 64,
        "variable source first destination word crossing is reachable"
    );
    kani::cover!(
        o.copied > 0 && o.amount < 128 && o.amount + o.copied as u64 > 128,
        "variable source second destination word crossing is reachable"
    );
    kani::cover!(
        o.copied > 0 && o.source_bit < 64 && o.source_bit + o.copied as u64 > 64,
        "variable source first source word crossing is reachable"
    );
    kani::cover!(
        o.copied > 0 && o.source_bit < 128 && o.source_bit + o.copied as u64 > 128,
        "variable source second source word crossing is reachable"
    );
    kani::cover!(
        o.copied > 0 && o.source_bit + o.copied as u64 == 192,
        "variable source last allocated source bit is reachable"
    );
    kani::cover!(
        o.copied > 0 && o.amount + o.copied as u64 == 130,
        "variable source last logical destination bit is reachable"
    );
    kani::cover!(
        o.copied > 0 && o.width == u32::MAX,
        "variable source maximum requested width is reachable"
    );
    kani::cover!(
        o.copied > 0 && !o.changed,
        "variable source unchanged active store is reachable"
    );
    kani::cover!(
        o.copied > 0
            && o.alias
            && o.source_bit < o.amount
            && o.source_bit + o.copied as u64 > o.amount,
        "variable source overlapping earlier RHS is reachable"
    );
    kani::cover!(
        o.copied > 0
            && o.alias
            && o.source_bit > o.amount
            && o.source_bit < o.amount + o.copied as u64,
        "variable source overlapping later RHS is reachable"
    );
    kani::cover!(
        o.dirty_padding,
        "variable source nonzero padding is preserved"
    );
    kani::cover!(o.byte == 0, "variable source prefix guard is reachable");
    kani::cover!(o.byte == 25, "variable source suffix guard is reachable");
    kani::cover!(
        o.byte == 24,
        "variable source last storage byte is reachable"
    );
}
fn covers_record_frame(o: &Observation) {
    kani::cover!(
        o.copied > 0 && o.frame_byte == 0,
        "variable source first live record byte is reachable"
    );
    kani::cover!(
        o.copied > 0 && o.frame_byte == 55,
        "variable source last live record byte is reachable"
    );
}
fn covers_log_outputs(o: &Observation) {
    covers_record_frame(o);
    kani::cover!(
        o.events == 1,
        "variable source changed log-only notification is reachable"
    );
}
fn covers_direct_outputs(o: &Observation) {
    covers_record_frame(o);
    kani::cover!(
        o.direct && o.changed && o.events == 0,
        "variable source changed direct store emits no commit notification"
    );
}
fn covers_small(o: &Observation) {
    kani::cover!(o.width == 0, "variable source zero width is reachable");
    kani::cover!(
        o.amount == u64::MAX,
        "variable source maximum destination start is reachable"
    );
    kani::cover!(
        o.source_bit == u64::MAX,
        "variable source no-op maximum source offset is reachable"
    );
    kani::cover!(o.copied == 1, "variable source one copied bit is reachable");
    kani::cover!(
        o.copied == 64,
        "variable source 64 copied bits are reachable"
    );
}
fn covers_large(o: &Observation) {
    kani::cover!(
        o.copied == 65,
        "variable source 65 copied bits are reachable"
    );
    kani::cover!(
        o.copied == 130,
        "variable source 130 copied bits are reachable"
    );
}
fn check<const LARGE: bool, const ACTIVE: bool, const RUN: bool, const FLAGS: u8>() -> Observation {
    let mut memory: [u8; 26] = kani::any();
    let before = memory;
    let source: [u8; 25] = kani::any();
    let source_before = source;
    let amount = if ACTIVE {
        u64::from(kani::any::<u8>() % 130)
    } else {
        kani::any::<u64>()
    };
    let width: u32 = kani::any();
    let source_bit = if ACTIVE {
        u64::from(kani::any::<u8>())
    } else {
        kani::any::<u64>()
    };
    let direct = if FLAGS < 4 {
        FLAGS & 1 != 0
    } else {
        kani::any::<bool>()
    };
    let alias = if FLAGS < 4 {
        FLAGS & 2 != 0
    } else {
        kani::any::<bool>()
    };
    if ACTIVE {
        kani::assume(width > 0);
    } else {
        kani::assume(width == 0 || amount >= 130);
    }
    let copied = if amount >= 130 {
        0
    } else {
        (width as usize).min(130 - amount as usize)
    };
    kani::assume((copied > 64) == LARGE);
    assert!(
        (copied > 0) == ACTIVE,
        "variable source proof partition mismatch"
    );
    if ACTIVE {
        kani::assume(source_bit <= 192 - copied as u64);
    }
    let input = if alias {
        &before[1..25]
    } else {
        &source_before[1..25]
    };
    let changed = ACTIVE
        && words::project_u192(&before[1..25], amount as usize, copied)
            != words::project_u192(input, source_bit as usize, copied);
    let start = if ACTIVE {
        (amount as usize / 64) * 8
    } else {
        0
    };
    let end = if ACTIVE {
        (amount as usize + copied).div_ceil(64) * 8
    } else {
        0
    };
    let length = end - start;
    let byte = usize::from(kani::any::<u8>()) % 26;
    let bit = usize::from(kani::any::<u8>() & 7);
    let frame_byte = usize::from(kani::any::<u8>()) % 56;
    let input_observation = Observation {
        frame_byte,
        amount,
        width,
        source_bit,
        copied,
        direct,
        alias,
        changed,
        events: 0,
        byte,
        dirty_padding: before[24] & 0x80 != 0,
    };
    if !RUN {
        return input_observation;
    }
    let mut log = WriteLogBuffer::with_capacity(0, 1);
    let capacity = (log.narrow_capacity(), log.wide_capacity());
    unsafe {
        OP_ACTIVE = ACTIVE;
        // Each active source exposes complete words through the selected bit.
        // An exact alias derives from the same destination base; no partial
        // allocation overlap is admitted. Guards are outside the 24-byte span.
        let dst = memory.as_mut_ptr().add(1);
        let src = if alias {
            dst.cast_const()
        } else {
            source.as_ptr().add(1)
        };
        wide_ops::wide_window_store_from(
            &mut log,
            dst,
            src,
            amount,
            (130u64 << 32) | u64::from(width),
            1 | (u64::from(direct) << 32),
            source_bit,
        );
    }
    assert!(
        log.narrow_count() == 0
            && log.wide_count() == u32::from(ACTIVE)
            && capacity == (log.narrow_capacity(), log.wide_capacity()),
        "simulator variable source-offset semantics mismatch"
    );
    // This assigns the equal count whose producer postcondition was just checked.
    // Its constant bound avoids expanding symbolic producer histories again in
    // the real commit loops. An incorrect count still fails the assertion above.
    log.wide_count = u32::from(ACTIVE);
    let wanted = if ACTIVE {
        expected_bit(&before, input, amount, width, source_bit, byte, bit)
    } else {
        words::bit(&before, byte * 8 + bit)
    };
    assert!(
        words::bit(&memory, byte * 8 + bit)
            == if direct {
                wanted
            } else {
                words::bit(&before, byte * 8 + bit)
            },
        "simulator variable source-offset semantics mismatch"
    );
    let mut saved_offset = 0;
    let mut saved_size = 0;
    let mut saved_padding = [0u8; 3];
    let mut saved_payload = [0u8; 56];
    if ACTIVE {
        let entry = &log.wide_entries_slice()[0];
        assert!(
            entry.offset as usize == 1 + start && usize::from(entry.native_bytes) == length,
            "simulator variable source-offset semantics mismatch"
        );
        let payload_byte = usize::from(kani::any::<u8>()) % 24;
        if payload_byte < length {
            assert!(
                words::bit(&entry.payload, payload_byte * 8 + bit)
                    == expected_bit(
                        &before,
                        input,
                        amount,
                        width,
                        source_bit,
                        1 + start + payload_byte,
                        bit,
                    ),
                "simulator variable source-offset semantics mismatch"
            );
        }
        saved_offset = entry.offset;
        saved_size = entry.native_bytes;
        saved_padding = entry._pad;
        saved_payload = entry.payload;
    }
    let mut plain = memory;
    let mut marked = memory;
    ff_commit_from_log(&mut plain, &log);
    // Reset the complete observer state for this one-threaded transition. A
    // function item avoids captured-reference aliases in the trait-object model.
    unsafe {
        EXPECT_NOTIFY = !direct && changed;
        EXPECTED_OFFSET = 1 + start;
        EXPECTED_SIZE = length;
        OBSERVED_COUNT = 0;
    }
    let mut observer = observe_change;
    ff_commit_from_log_marking(&mut marked, &log, &mut observer);
    let events = unsafe { OBSERVED_COUNT };
    assert!(
        words::bit(&plain, byte * 8 + bit) == wanted
            && words::bit(&marked, byte * 8 + bit) == wanted
            && events == usize::from(!direct && changed)
            && source == source_before,
        "simulator variable source-offset semantics mismatch"
    );
    assert!(
        log.narrow_count() == 0
            && log.wide_count() == u32::from(ACTIVE)
            && capacity == (log.narrow_capacity(), log.wide_capacity()),
        "simulator variable source-offset semantics mismatch"
    );
    if ACTIVE {
        let after = &log.wide_entries_slice()[0];
        assert!(
            after.offset == saved_offset
                && after.native_bytes == saved_size
                && after._pad == saved_padding
                && after.payload[frame_byte] == saved_payload[frame_byte],
            "simulator variable source-offset semantics mismatch"
        );
    }
    Observation {
        frame_byte,
        amount,
        width,
        source_bit,
        copied,
        direct,
        alias,
        changed,
        events,
        byte,
        dirty_padding: before[24] & 0x80 != 0,
    }
}

fn expected_bit(
    before: &[u8; 26],
    source: &[u8],
    amount: u64,
    width: u32,
    source_bit: u64,
    byte: usize,
    bit: usize,
) -> bool {
    if byte == 0 || byte == 25 {
        return words::bit(before, byte * 8 + bit);
    }
    words::window_stored_from_bit(
        &before[1..25],
        source,
        130,
        amount,
        width as usize,
        source_bit as usize,
        (byte - 1) * 8 + bit,
    )
}

fn reject_wide_growth(_: &mut WriteLogBuffer, _: usize) {
    panic!("bounded source-offset store reached wide-pool growth");
}

static mut EXPECT_NOTIFY: bool = false;
static mut EXPECTED_OFFSET: usize = 0;
static mut EXPECTED_SIZE: usize = 0;
static mut OBSERVED_COUNT: usize = 0;

fn observe_change(offset: usize, size: usize) {
    // The harness is single-threaded and resets every field before each commit.
    unsafe {
        assert!(
            EXPECT_NOTIFY
                && OBSERVED_COUNT == 0
                && offset == EXPECTED_OFFSET
                && size == EXPECTED_SIZE,
            "simulator variable source-offset semantics mismatch"
        );
        OBSERVED_COUNT += 1;
    }
}

static mut OP_ACTIVE: bool = false;

fn guarded_word_scratch<A: smallvec::Array>(elem: A::Item, n: usize) -> smallvec::SmallVec<A>
where
    A::Item: Clone,
{
    if !unsafe { OP_ACTIVE } {
        panic!("no-op source-offset store reached scratch construction");
    }
    crate::window_log_variable::generic_word_scratch(elem, n)
}
