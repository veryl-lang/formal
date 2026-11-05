//! One real 512-bit window store at bit offset 64 in a 576-bit allocation.
//! Arbitrary bytes, safe u32 base offsets, direct/log-only and disjoint/exact-base
//! aliases are admitted. Capacity 2 starts with one arbitrary wide record and one
//! narrow record, so appending chunks of 56 and 8 bytes forces in-call growth.
//! Checks offsets/order/payload, both histories, source and destination guards.
//! Source/destination pointers have byte offset 1. Input words span all byte
//! patterns on the required little-endian 64-bit target. Allocation succeeds;
//! execution is single-threaded. Other windows/capacities, partial overlaps,
//! four-state interpretation, OOM and scheduling/commit are outside scope.
use veryl_simulator::{ir::WriteLogBuffer, wide_ops};
#[cfg(not(all(target_endian = "little", target_pointer_width = "64")))]
compile_error!("chunk proof requires a little-endian 64-bit target");
#[kani::proof]
#[kani::unwind(66)]
fn window_log_chunk_split_matches_reference() {
    let sw: [u64; 8] = kani::any();
    let dw: [u64; 9] = kani::any();
    let old: [u64; 7] = kani::any();
    let mut source = [0u8; 65];
    source[0] = kani::any();
    let mut destination = [0u8; 74];
    destination[0] = kani::any();
    destination[73] = kani::any();
    // SAFETY: initialized integer arrays have no padding; buffers are disjoint.
    unsafe {
        std::ptr::copy_nonoverlapping(sw.as_ptr().cast::<u8>(), source.as_mut_ptr().add(1), 64);
        std::ptr::copy_nonoverlapping(
            dw.as_ptr().cast::<u8>(),
            destination.as_mut_ptr().add(1),
            72,
        );
    }
    let source_before = source;
    let before = destination;
    let direct: bool = kani::any();
    let alias: bool = kani::any();
    let base: u32 = kani::any();
    kani::assume(base <= u32::MAX - 71);
    let old_offset: u32 = kani::any();
    let old_len = kani::any::<u8>() % 56 + 1;
    let narrow: u64 = kani::any();
    let mut log = WriteLogBuffer::with_capacity(1, 2);
    unsafe {
        (log.grow_push_narrow)(&mut log, 3, narrow, 8);
        let e = &mut *log.wide_entries_ptr;
        e.offset = old_offset;
        e.native_bytes = old_len;
        // SAFETY: seven initialized u64s supply 56 bytes to the owned record.
        std::ptr::copy_nonoverlapping(old.as_ptr().cast::<u8>(), e.payload.as_mut_ptr(), 56);
    }
    log.wide_count = 1;
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
            512 | (576u64 << 32),
            u64::from(base) | ((direct as u64) << 32),
        );
    }
    assert!(
        log.wide_count() == 3 && log.wide_capacity() >= 3 && log.narrow_count() == 1,
        "simulator window log semantics mismatch"
    );
    let i = usize::from(kani::any::<u8>() % 56);
    let previous = &log.wide_entries_slice()[0];
    assert!(
        previous.offset == old_offset
            && previous.native_bytes == old_len
            && previous.payload[i] == old[i / 8].to_le_bytes()[i % 8],
        "simulator window log semantics mismatch"
    );
    let chunk = usize::from(kani::any::<bool>());
    let len = if chunk == 0 { 56 } else { 8 };
    let byte = usize::from(kani::any::<u8>()) % len;
    let original = if alias {
        &before[1..65]
    } else {
        &source_before[1..]
    };
    let e = &log.wide_entries_slice()[chunk + 1];
    assert!(
        e.offset == base + 8 + 56 * chunk as u32
            && e.native_bytes as usize == len
            && e.payload[byte] == original[chunk * 56 + byte],
        "simulator window log semantics mismatch"
    );
    let n = &log.narrow_entries_slice()[0];
    assert!(
        n.offset == 3 && n.width_class == 8 && n.mask_xz == 0 && n.payload == narrow,
        "simulator window log semantics mismatch"
    );
    let out = usize::from(kani::any::<u8>() % 74);
    let expected = if direct && (9..73).contains(&out) {
        original[out - 9]
    } else {
        before[out]
    };
    assert!(
        destination[out] == expected && source == source_before,
        "simulator window log semantics mismatch"
    );
    kani::cover!(
        chunk == 0 && byte == 0,
        "first chunk begins at first source byte"
    );
    kani::cover!(chunk == 0 && byte == 55, "first chunk ends at byte55");
    kani::cover!(chunk == 1 && byte == 0, "second chunk begins at byte56");
    kani::cover!(chunk == 1 && byte == 7, "second chunk ends at byte63");
    kani::cover!(original[0] != original[56], "chunk payloads differ");
    kani::cover!(
        base == u32::MAX - 71,
        "highest safe chunk base is reachable"
    );
    kani::cover!(direct && alias, "direct alias chunk split is reachable");
    kani::cover!(!direct && alias, "log-only alias chunk split is reachable");
    kani::cover!(direct && !alias, "direct external chunk split is reachable");
    kani::cover!(
        !direct && !alias,
        "log-only external chunk split is reachable"
    );
    kani::cover!(out == 0, "chunk prefix guard is reachable");
    kani::cover!(out == 73, "chunk suffix guard is reachable");
}
