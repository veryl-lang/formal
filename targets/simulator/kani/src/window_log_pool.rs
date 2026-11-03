//! Growth of a full pool through one real wide_window_store call. The initial pool
//! has capacity/count 64, arbitrary offsets, lengths 1..56 and all 56 payload bytes;
//! padding stays default. One live narrow record, an arbitrary 192-bit destination
//! and a disjoint or exactly aliased 8-byte source are present. Direct/log-only
//! modes are symbolic. Base offsets are bounded by u32::MAX-15. Appending one
//! 64-bit window at bit offset 64 triggers real growth (currently to 128 slots)
//! and preserves both record pools, source data and destination guard bytes.
//! Other capacities/counts, arbitrary histories, chunk splitting, partial overlap,
//! four-state plane interpretation and commit/scheduling are outside scope.
//! Execution is single-threaded and allocations succeed; allocation failure is
//! outside this contract.
//! Word-based input generation spans every payload byte pattern on this target
//! while reducing standard-library initialization loops.
use veryl_simulator::{ir::WriteLogBuffer, wide_ops};
#[cfg(not(all(target_endian = "little", target_pointer_width = "64")))]
compile_error!("full-pool growth proof requires a little-endian 64-bit target");
#[kani::proof]
#[kani::unwind(129)]
fn window_log_full_pool_growth_matches_reference() {
    let offsets: [u32; 64] = kani::any();
    let sizes: [u8; 64] = kani::any();
    let data: [[u64; 7]; 64] = kani::any();
    let narrow: u64 = kani::any();
    let base: u32 = kani::any();
    kani::assume(base <= u32::MAX - 15);
    let direct: bool = kani::any();
    let alias: bool = kani::any();
    let mut destination: [u8; 26] = kani::any();
    let before = destination;
    let source: [u8; 9] = kani::any();
    let source_before = source;
    let mut log = WriteLogBuffer::with_capacity(1, 64);
    unsafe {
        (log.grow_push_narrow)(&mut log, 3, narrow, 8);
    }
    for i in 0..64 {
        unsafe {
            let entry = &mut *log.wide_entries_ptr.add(i);
            entry.offset = offsets[i];
            entry.native_bytes = sizes[i] % 56 + 1; // SAFETY: seven initialized u64s occupy 56 bytes. Integer arrays have no
            // padding; the stack input and owned log allocation are disjoint.
            std::ptr::copy_nonoverlapping(
                data[i].as_ptr().cast::<u8>(),
                entry.payload.as_mut_ptr(),
                56,
            );
        }
    }
    log.wide_count = 64;
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
            u64::from(base) | ((direct as u64) << 32),
        );
    }
    assert!(
        log.wide_count() == 65 && log.wide_capacity() >= 65 && log.narrow_count() == 1,
        "simulator window log semantics mismatch"
    );
    let record = usize::from(kani::any::<u8>() & 63);
    let byte = usize::from(kani::any::<u8>() % 56);
    let old = &log.wide_entries_slice()[record];
    assert!(
        old.offset == offsets[record]
            && old.native_bytes == sizes[record] % 56 + 1
            && old.payload[byte] == data[record][byte / 8].to_le_bytes()[byte % 8],
        "simulator window log semantics mismatch"
    );
    let copied = usize::from(kani::any::<u8>() & 7);
    let original = if alias {
        before[copied + 1]
    } else {
        source[copied + 1]
    };
    let new = &log.wide_entries_slice()[64];
    assert!(
        new.offset == base + 8 && new.native_bytes == 8 && new.payload[copied] == original,
        "simulator window log semantics mismatch"
    );
    let n = &log.narrow_entries_slice()[0];
    assert!(
        n.offset == 3 && n.width_class == 8 && n.mask_xz == 0 && n.payload == narrow,
        "simulator window log semantics mismatch"
    );
    let out = usize::from(kani::any::<u8>() % 26);
    let expected = if direct && (9..17).contains(&out) {
        if alias {
            before[out - 8]
        } else {
            source[out - 8]
        }
    } else {
        before[out]
    };
    assert!(
        destination[out] == expected && source == source_before,
        "simulator window log semantics mismatch"
    );
    kani::cover!(
        record == 0 && byte == 0,
        "first old record byte is reachable"
    );
    kani::cover!(
        record == 63 && byte == 55,
        "last old record byte is reachable"
    );
    kani::cover!(old.native_bytes == 1, "one-byte old record is reachable");
    kani::cover!(
        old.native_bytes == 56,
        "full-length old record is reachable"
    );
    kani::cover!(
        data[record][byte / 8].to_le_bytes()[byte % 8] != 0,
        "nonzero old history is reachable"
    );
    kani::cover!(
        base == u32::MAX - 15,
        "maximum safe full pool base is reachable"
    );
    kani::cover!(direct && alias, "full pool direct alias is reachable");
    kani::cover!(!direct && alias, "full pool log-only alias is reachable");
    kani::cover!(direct && !alias, "full pool direct external is reachable");
    kani::cover!(
        !direct && !alias,
        "full pool log-only external is reachable"
    );
}
