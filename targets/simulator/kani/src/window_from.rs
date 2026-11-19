//! Source-offset binary stores through the real JIT helper and both real commits.
//! Fixed (destination width, amount, requested width, source bit) cases are
//! (13,5,12,63), (128,63,128,17), and (130,63,128,17). The first uses exactly
//! two destination bytes and one narrow record; the second uses sixteen bytes
//! and two narrow records; the third uses twenty-four bytes and one wide record.
//! All data and direct/log-only modes are arbitrary. The 128/130-bit cases also
//! admit exact-base aliases; the short case has a disjoint 16-byte source only.
//! Pointers have byte offset 1 and destination suffix guards. Log base is 1.
//! The two-record case starts with narrow capacity 2; other logs have one slot.
//! Pool growth is excluded. Independent bit origins
//! and scalar u128 masks check staging, final bits, ordered change notifications,
//! payload/record fields, source frames and guards. Padding above logical width
//! remains unchanged. Commit starts from the post-store staging state.
//! A separate no-op harness quantifies all u32 widths, all u64 destination/source
//! bit positions and flags, null data pointers and null/live logs with both pools
//! prefilled. Its explicit assumption is width zero or amount outside the target.
//! Active cases use real operations and constructors. The no-op proof replaces
//! active-only scratch construction with unconditional failure: its assertion
//! must be unreachable for verification to pass. No behavior of that constructor
//! is assumed. Setup allocation is real and succeeds on a single thread.
//! Other active windows/source offsets, short-source aliases,
//! partial pointer overlaps, four-state execution and source NBA scheduling are
//! excluded. Safety and unwinding assertions remain enabled.
use veryl_formal_semantics::words;
use veryl_simulator::{
    ir::{WriteLogBuffer, ff_commit_from_log, ff_commit_from_log_marking},
    wide_ops,
};

#[cfg(not(all(target_endian = "little", target_pointer_width = "64")))]
compile_error!("source-offset proofs require a little-endian 64-bit target");

#[kani::proof]
#[kani::unwind(65)]
fn window_from_short_matches_reference() {
    let r = active::<0, 4>();
    kani::cover!(r.direct, "offset short direct write is reachable");
    kani::cover!(!r.direct, "offset short log-only write is reachable");
    kani::cover!(
        r.first && r.events == 1,
        "offset short changed commit emits one event"
    );
    kani::cover!(
        !r.direct && !r.first && r.events == 0,
        "offset short unchanged commit emits no event"
    );
    kani::cover!(
        r.direct && r.events == 0,
        "offset short direct commit emits no event"
    );
    kani::cover!(r.byte == 0, "offset short prefix guard is reachable");
    kani::cover!(r.byte == 3, "offset short suffix guard is reachable");
    kani::cover!(
        r.byte == 2 && r.bit == 4,
        "offset short clipped last bit is reachable"
    );
    kani::cover!(
        r.byte == 2 && r.bit == 7,
        "offset short native padding is reachable"
    );
}

#[kani::proof]
#[kani::unwind(65)]
fn window_from_native128_matches_reference() {
    let r = active::<1, 18>();
    kani::cover!(
        r.direct && r.alias,
        "offset native direct alias is reachable"
    );
    kani::cover!(
        !r.direct && r.alias,
        "offset native log-only alias is reachable"
    );
    kani::cover!(
        r.direct && !r.alias,
        "offset native direct external is reachable"
    );
    kani::cover!(
        !r.direct && !r.alias,
        "offset native log-only external is reachable"
    );
    kani::cover!(
        r.first && r.second && r.events == 2,
        "offset native emits two ordered events"
    );
    kani::cover!(
        r.first && !r.second && r.events == 1,
        "offset native changes only first word"
    );
    kani::cover!(
        !r.first && r.second && r.events == 1,
        "offset native changes only second word"
    );
    kani::cover!(
        !r.direct && !r.first && !r.second && r.events == 0,
        "offset native unchanged commit emits no event"
    );
    kani::cover!(
        r.direct && r.events == 0,
        "offset native direct commit emits no event"
    );
    kani::cover!(r.byte == 0, "offset native prefix guard is reachable");
    kani::cover!(r.byte == 17, "offset native suffix guard is reachable");
    kani::cover!(
        r.byte == 14 && r.bit == 6,
        "offset native source word boundary is reachable"
    );
    kani::cover!(
        r.byte == 16 && r.bit == 7,
        "offset native logical last bit is reachable"
    );
}

#[kani::proof]
#[kani::unwind(65)]
fn window_from_wide130_matches_reference() {
    let r = active::<2, 26>();
    kani::cover!(r.direct && r.alias, "offset wide direct alias is reachable");
    kani::cover!(
        !r.direct && r.alias,
        "offset wide log-only alias is reachable"
    );
    kani::cover!(
        r.direct && !r.alias,
        "offset wide direct external is reachable"
    );
    kani::cover!(
        !r.direct && !r.alias,
        "offset wide log-only external is reachable"
    );
    kani::cover!(
        r.first && r.events == 1,
        "offset wide changed commit emits one event"
    );
    kani::cover!(
        !r.direct && !r.first && r.events == 0,
        "offset wide unchanged commit emits no event"
    );
    kani::cover!(
        r.direct && r.events == 0,
        "offset wide direct commit emits no event"
    );
    kani::cover!(r.byte == 0, "offset wide prefix guard is reachable");
    kani::cover!(r.byte == 25, "offset wide suffix guard is reachable");
    kani::cover!(
        r.byte == 17 && r.bit == 1,
        "offset wide clipped last bit is reachable"
    );
    kani::cover!(
        r.byte == 24 && r.bit == 7,
        "offset wide native padding is reachable"
    );
}

struct Observation {
    direct: bool,
    alias: bool,
    first: bool,
    second: bool,
    events: usize,
    byte: usize,
    bit: usize,
}

fn active<const CASE: u8, const B: usize>() -> Observation {
    let source: [u8; 17] = kani::any();
    let source_before = source;
    let mut destination: [u8; B] = kani::any();
    let before = destination;
    let direct: bool = kani::any();
    let alias = CASE != 0 && kani::any::<bool>();
    let (dw, amount, width, source_bit) = if CASE == 0 {
        (13, 5, 12, 63)
    } else if CASE == 1 {
        (128, 63, 128, 17)
    } else {
        (130, 63, 128, 17)
    };
    let input = if alias {
        &before[1..17]
    } else {
        &source_before[1..]
    };
    let source_value = u128::from(read_word(input, 0)) | (u128::from(read_word(input, 8)) << 64);
    let old_value = if CASE == 0 {
        u128::from(u16::from_le_bytes([before[1], before[2]]))
    } else {
        u128::from(read_word(&before, 1)) | (u128::from(read_word(&before, 9)) << 64)
    };
    let expected_native = if CASE == 0 {
        let selected = (source_value >> 63) & 255;
        (old_value & !(255u128 << 5)) | (selected << 5)
    } else if CASE == 1 {
        let mask = (1u128 << 65) - 1;
        (old_value & !(mask << 63)) | (((source_value >> 17) & mask) << 63)
    } else {
        0
    };
    let first = !direct
        && if CASE < 2 {
            expected_native as u64 != old_value as u64
        } else {
            let old_window = (old_value >> 63) | (u128::from(read_word(&before, 17) & 3) << 65);
            old_window != ((source_value >> 17) & ((1u128 << 67) - 1))
        };
    let second = !direct && CASE == 1 && (expected_native >> 64) != (old_value >> 64);
    let first_event = (
        1,
        if CASE == 0 {
            2
        } else if CASE == 1 {
            8
        } else {
            24
        },
    );
    let expected_events = match (first, second) {
        (true, true) => [Some(first_event), Some((9, 8))],
        (true, false) => [Some(first_event), None],
        (false, true) => [Some((9, 8)), None],
        (false, false) => [None, None],
    };
    let mut log = WriteLogBuffer::with_capacity(
        if CASE == 1 { 2 } else { usize::from(CASE == 0) },
        usize::from(CASE == 2),
    );
    unsafe {
        let dst = destination.as_mut_ptr().add(1);
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
            width | (dw << 32),
            1 | ((direct as u64) << 32),
            source_bit,
        );
    }
    let narrow_count = if CASE == 0 {
        1
    } else if CASE == 1 {
        2
    } else {
        0
    };
    let wide_count = u32::from(CASE == 2);
    assert!(
        log.narrow_count() == narrow_count && log.wide_count() == wide_count,
        "simulator source-offset window semantics mismatch"
    );
    let narrow = if CASE < 2 {
        let entries = log.narrow_entries_slice();
        let saved = [entries[0], entries[if CASE == 1 { 1 } else { 0 }]];
        for i in 0..narrow_count as usize {
            let e = &entries[i];
            let expected = (expected_native >> (i * 64)) as u64;
            assert!(
                e.offset == 1 + (i as u32) * 8
                    && e.width_class == first_event.1 as u16
                    && e.mask_xz == 0
                    && e.payload == expected,
                "simulator source-offset window semantics mismatch"
            );
        }
        Some(saved)
    } else {
        None
    };
    let wide = if CASE == 2 {
        let e = log.wide_entries_slice()[0];
        assert!(
            e.offset == 1 && e.native_bytes == 24,
            "simulator source-offset window semantics mismatch"
        );
        let byte = usize::from(kani::any::<u8>() % 24);
        let bit = usize::from(kani::any::<u8>() & 7);
        assert!(
            (e.payload[byte] & (1 << bit) != 0)
                == words::window_stored_from_bit(
                    &before[1..25],
                    input,
                    dw as usize,
                    amount,
                    width as usize,
                    source_bit as usize,
                    byte * 8 + bit
                ),
            "simulator source-offset window semantics mismatch"
        );
        Some(e)
    } else {
        None
    };
    let capacity = (log.narrow_capacity(), log.wide_capacity());
    let mut plain = destination;
    let mut marked = destination;
    ff_commit_from_log(&mut plain, &log);
    let mut events = 0;
    ff_commit_from_log_marking(&mut marked, &log, &mut |off, nb| {
        assert!(
            expected_events.get(events).copied().flatten() == Some((off, nb)),
            "simulator source-offset window semantics mismatch"
        );
        events += 1;
    });
    assert!(
        events == usize::from(first) + usize::from(second),
        "simulator source-offset window semantics mismatch"
    );
    let byte = usize::from(kani::any::<u8>()) % B;
    let bit = usize::from(kani::any::<u8>() & 7);
    let expected = if byte > 0 && byte < B - 1 {
        words::window_stored_from_bit(
            &before[1..B - 1],
            input,
            dw as usize,
            amount,
            width as usize,
            source_bit as usize,
            (byte - 1) * 8 + bit,
        )
    } else {
        words::bit(&before, byte * 8 + bit)
    };
    let staged = if direct {
        expected
    } else {
        words::bit(&before, byte * 8 + bit)
    };
    assert!(
        words::bit(&destination, byte * 8 + bit) == staged
            && words::bit(&plain, byte * 8 + bit) == expected
            && words::bit(&marked, byte * 8 + bit) == expected,
        "simulator source-offset window semantics mismatch"
    );
    assert!(
        source == source_before
            && log.narrow_count() == narrow_count
            && log.wide_count() == wide_count
            && capacity == (log.narrow_capacity(), log.wide_capacity()),
        "simulator source-offset window semantics mismatch"
    );
    if let Some(saved) = narrow {
        let i = usize::from(kani::any::<bool>());
        if i < narrow_count as usize {
            let e = &log.narrow_entries_slice()[i];
            assert!(
                e.offset == saved[i].offset
                    && e.width_class == saved[i].width_class
                    && e.mask_xz == saved[i].mask_xz
                    && e.payload == saved[i].payload,
                "simulator source-offset window semantics mismatch"
            );
        }
    }
    if let Some(saved) = wide {
        let e = &log.wide_entries_slice()[0];
        let i = usize::from(kani::any::<u8>() % 56);
        assert!(
            e.offset == saved.offset
                && e.native_bytes == saved.native_bytes
                && e.payload[i] == saved.payload[i],
            "simulator source-offset window semantics mismatch"
        );
    }
    Observation {
        direct,
        alias,
        first,
        second,
        events,
        byte,
        bit,
    }
}

#[kani::proof]
#[kani::unwind(2)]
#[kani::stub(smallvec::SmallVec::<[u64;16]>::from_elem, reject_active_scratch)]
fn window_from_noop_preserves_prefilled_logs() {
    let packed_width: u64 = kani::any();
    let width = packed_width as u32;
    let dw = (packed_width >> 32) as u32;
    let amount: u64 = kani::any();
    let source_bit: u64 = kani::any();
    let flags: u64 = kani::any();
    let data: [u64; 7] = kani::any();
    let narrow_data: u64 = kani::any();
    let len = u32::from(kani::any::<u8>() % 56 + 1);
    let mut log = WriteLogBuffer::with_capacity(1, 1);
    unsafe {
        // SAFETY: both capacity-one pools own initialized entries. Populate a
        // valid arbitrary live prefix without testing the append paths here.
        let n = &mut *log.narrow_entries_ptr;
        n.offset = kani::any();
        n.width_class = 8;
        n.mask_xz = 0;
        n.payload = narrow_data;
        let w = &mut *log.wide_entries_ptr;
        w.offset = kani::any();
        w.native_bytes = len as u8;
        std::ptr::copy_nonoverlapping(data.as_ptr().cast::<u8>(), w.payload.as_mut_ptr(), 56);
    }
    log.narrow_count = 1;
    log.wide_count = 1;
    let n = log.narrow_entries_slice()[0];
    let w = log.wide_entries_slice()[0];
    let capacity = (log.narrow_capacity(), log.wide_capacity());
    let null_log: bool = kani::any();
    let log_ptr = if null_log {
        std::ptr::null_mut()
    } else {
        &mut log as *mut _
    };
    // These branches partition the original no-op domain. Explicit call sites
    // avoid expanding the large allocation paths behind infeasible guards.
    if width == 0 {
        unsafe {
            wide_ops::wide_window_store_from(
                log_ptr,
                std::ptr::null_mut(),
                std::ptr::null(),
                amount,
                u64::from(dw) << 32,
                flags,
                source_bit,
            );
        }
    } else if amount >= packed_width >> 32 {
        unsafe {
            wide_ops::wide_window_store_from(
                log_ptr,
                std::ptr::null_mut(),
                std::ptr::null(),
                amount,
                packed_width,
                flags,
                source_bit,
            );
        }
    } else {
        kani::assume(false);
    }
    assert!(
        log.narrow_count() == 1
            && log.wide_count() == 1
            && capacity == (log.narrow_capacity(), log.wide_capacity()),
        "simulator source-offset window semantics mismatch"
    );
    let after_n = &log.narrow_entries_slice()[0];
    let after_w = &log.wide_entries_slice()[0];
    let i = usize::from(kani::any::<u8>() % 56);
    assert!(
        after_n.offset == n.offset
            && after_n.width_class == n.width_class
            && after_n.mask_xz == n.mask_xz
            && after_n.payload == n.payload
            && after_w.offset == w.offset
            && after_w.native_bytes == w.native_bytes
            && after_w.payload[i] == w.payload[i],
        "simulator source-offset window semantics mismatch"
    );
    kani::cover!(
        width == 0 && dw > 0,
        "offset noop zero source width is reachable"
    );
    kani::cover!(
        dw == 0 && width > 0,
        "offset noop empty destination is reachable"
    );
    kani::cover!(
        width > 0 && amount == u64::from(dw),
        "offset noop exact destination end is reachable"
    );
    kani::cover!(
        amount == u64::MAX,
        "offset noop maximum amount is reachable"
    );
    kani::cover!(
        source_bit == u64::MAX,
        "offset noop maximum source bit is reachable"
    );
    kani::cover!(null_log, "offset noop null log is reachable");
    kani::cover!(
        !null_log && len == 56 && narrow_data != 0,
        "offset noop nonzero prefilled logs are reachable"
    );
}

fn reject_active_scratch<A: smallvec::Array>(_: A::Item, _: usize) -> smallvec::SmallVec<A>
where
    A::Item: Clone,
{
    panic!("no-op reached active scratch construction");
}

fn read_word(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
        bytes[offset + 4],
        bytes[offset + 5],
        bytes[offset + 6],
        bytes[offset + 7],
    ])
}
