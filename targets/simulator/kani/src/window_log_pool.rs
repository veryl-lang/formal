//! Append frames for a pool with capacity 64 and any live count 0..64. Live
//! records have arbitrary offsets, lengths 1..56 and all 56 stored payload bytes;
//! unused slots also admit arbitrary length bytes. Padding stays default. One
//! narrow record, an arbitrary 192-bit destination and an 8-byte disjoint or
//! exactly aliased source are present. Both direct/log-only modes are symbolic.
//! One 64-bit window at bit offset 64 is appended, with base <= u32::MAX-15.
//! Separate harnesses cover spare capacity and real growth (currently to 128 slots).
//! All live records, the narrow record, source and destination guards are checked.
//! Other capacities, arbitrary histories/chunking, partial overlaps, four-state
//! interpretation and commit/scheduling are excluded. Execution is single-threaded
//! and allocations succeed. Word generation spans all payload byte patterns with
//! fewer initialization loops on a little-endian 64-bit target. Pointer/refcount
//! identity and allocation failure
//! are outside scope.
use veryl_simulator::{ir::WriteLogBuffer, wide_ops};
#[cfg(not(all(target_endian = "little", target_pointer_width = "64")))]
compile_error!("pool append proofs require a little-endian 64-bit target");

struct Observation {
    record: usize,
    byte: usize,
    old_length: u8,
    nonzero: bool,
    unused_header: u8,
    base: u32,
    direct: bool,
    alias: bool,
}

#[kani::proof]
#[kani::unwind(129)]
fn window_log_full_pool_growth_matches_reference() {
    let r = append(64);
    kani::cover!(
        r.record == 0 && r.byte == 0,
        "first old record byte is reachable"
    );
    kani::cover!(
        r.record == 63 && r.byte == 55,
        "last old record byte is reachable"
    );
    kani::cover!(r.old_length == 1, "one-byte old record is reachable");
    kani::cover!(r.old_length == 56, "full-length old record is reachable");
    kani::cover!(r.nonzero, "nonzero old history is reachable");
    kani::cover!(
        r.base == u32::MAX - 15,
        "maximum safe full pool base is reachable"
    );
    kani::cover!(r.direct && r.alias, "full pool direct alias is reachable");
    kani::cover!(
        !r.direct && r.alias,
        "full pool log-only alias is reachable"
    );
    kani::cover!(
        r.direct && !r.alias,
        "full pool direct external is reachable"
    );
    kani::cover!(
        !r.direct && !r.alias,
        "full pool log-only external is reachable"
    );
}

#[kani::proof]
#[kani::unwind(65)]
fn window_log_prefix_append_matches_reference() {
    let count = u32::from(kani::any::<u8>() & 63);
    let r = append(count);
    kani::cover!(count == 0, "empty wide prefix is reachable");
    kani::cover!(count == 1, "one live record prefix is reachable");
    kani::cover!(count == 63, "last spare slot is reachable");
    kani::cover!(
        r.record == 0 && r.record < count as usize && r.byte == 0,
        "first live prefix byte is reachable"
    );
    kani::cover!(
        count == 63 && r.record == 62 && r.byte == 55 && r.old_length == 56,
        "last live prefix byte is reachable"
    );
    kani::cover!(
        r.record < count as usize && r.nonzero,
        "nonzero live prefix is reachable"
    );
    kani::cover!(r.unused_header == 255, "dirty unused slot is overwritten");
    kani::cover!(
        r.base == u32::MAX - 15,
        "maximum safe prefix base is reachable"
    );
    kani::cover!(r.direct && r.alias, "prefix direct alias is reachable");
    kani::cover!(!r.direct && r.alias, "prefix log-only alias is reachable");
    kani::cover!(r.direct && !r.alias, "prefix direct external is reachable");
    kani::cover!(
        !r.direct && !r.alias,
        "prefix log-only external is reachable"
    );
}

fn append(count: u32) -> Observation {
    assert!(count <= 64, "simulator window log semantics mismatch");
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
            entry.native_bytes = if i < count as usize {
                sizes[i] % 56 + 1
            } else {
                sizes[i]
            };
            // SAFETY: seven initialized u64s occupy 56 bytes. Integer arrays have no
            // padding; the stack input and owned log allocation are disjoint.
            std::ptr::copy_nonoverlapping(
                data[i].as_ptr().cast::<u8>(),
                entry.payload.as_mut_ptr(),
                56,
            );
        }
    }
    log.wide_count = count;
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
        log.wide_count() == count + 1
            && log.wide_capacity() >= count as usize + 1
            && log.narrow_count() == 1,
        "simulator window log semantics mismatch"
    );
    let record = usize::from(kani::any::<u8>() & 63);
    let byte = usize::from(kani::any::<u8>() % 56);
    let old = &log.wide_entries_slice()[record];
    if record < count as usize {
        assert!(
            old.offset == offsets[record]
                && old.native_bytes == sizes[record] % 56 + 1
                && old.payload[byte] == data[record][byte / 8].to_le_bytes()[byte % 8],
            "simulator window log semantics mismatch"
        );
    }
    let copied = usize::from(kani::any::<u8>() & 7);
    let original = if alias {
        before[copied + 1]
    } else {
        source[copied + 1]
    };
    let new = &log.wide_entries_slice()[count as usize];
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
    Observation {
        record,
        byte,
        old_length: old.native_bytes,
        nonzero: data[record][byte / 8].to_le_bytes()[byte % 8] != 0,
        unused_header: if count < 64 { sizes[count as usize] } else { 0 },
        base,
        direct,
        alias,
    }
}
