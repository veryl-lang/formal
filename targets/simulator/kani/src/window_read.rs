//! Bounded physical-plane reads with a preclipped source/destination interval.
//! Source width 130 has exactly 24 accessible bytes after an unaligned prefix.
//! All source storage bits, including padding, and all initial destination bytes
//! are arbitrary. Output windows 13/65/130 have exact native extents 2/16/24,
//! guarded on both sides. Active spans require start+count <= 130 and
//! skip+count <= window; start, skip and count range over all u64 values meeting
//! those premises, without wrapping either sum. The caller's bounds lowering
//! and alias routing are excluded.
//! Count-zero roots instead admit arbitrary u64 source widths/start/skip and a
//! null source; fill is an arbitrary repeated u64 plane pattern. A zero-window
//! root admits arbitrary u64 parameters and null or guarded live destinations.
//! Only those no-source/no-access paths admit null pointers. Active source and
//! destination allocations are disjoint. Every logical output bit follows an
//! independent bit-origin model; padding is zero, guards and sources preserved.
//! IEEE 1800-2023 11.5.1 supplies the in-range/out-of-range distinction, while
//! the caller chooses payload/XZ-plane fill values. This is not a source-level
//! selection, JIT bounds, arbitrary aliasing, or four-state encoding theorem.
//! All real helper operations execute without stubs on a little-endian 64-bit
//! target; explicit covers witness nonempty spans and every stated boundary.
use veryl_formal_semantics::words;
use veryl_simulator::wide_ops;
#[cfg(not(all(target_endian = "little", target_pointer_width = "64")))]
compile_error!("read-plane proofs require a little-endian 64-bit target");

#[kani::proof]
#[kani::unwind(27)]
fn window_read13_matches_reference() {
    let r = read::<13, 2, 4, false>();
    kani::cover!(r.count == 0, "read13 empty copied span is reachable");
    kani::cover!(r.count == 1, "read13 one copied bit is reachable");
    kani::cover!(r.count > 1, "read13 multiple copied bits are reachable");
    kani::cover!(r.count == 13, "read13 full output span is reachable");
    kani::cover!(
        r.skip > 0 && r.count > 0,
        "read13 leading fill precedes copied bits"
    );
    kani::cover!(
        r.count > 0 && r.skip + r.count < 13,
        "read13 trailing fill follows copied bits"
    );
    kani::cover!(
        r.start < 64 && r.start + r.count > 64,
        "read13 first source word boundary is reachable"
    );
    kani::cover!(
        r.start < 128 && r.start + r.count > 128,
        "read13 second source word boundary is reachable"
    );
    kani::cover!(
        r.count > 0 && r.start + r.count == 130,
        "read13 last logical source bit is reachable"
    );
    kani::cover!(
        r.count > 0 && r.fill != 0,
        "read13 nonzero fill and copied data coexist"
    );
    kani::cover!(r.byte == 0, "read13 prefix guard is reachable");
    kani::cover!(r.byte == 3, "read13 suffix guard is reachable");
    kani::cover!(r.byte == 2, "read13 native padding byte is reachable");
}

#[kani::proof]
#[kani::unwind(27)]
fn window_read65_matches_reference() {
    let r = read::<65, 16, 18, false>();
    kani::cover!(r.count == 0, "read65 empty copied span is reachable");
    kani::cover!(r.count == 1, "read65 one copied bit is reachable");
    kani::cover!(r.count > 1, "read65 multiple copied bits are reachable");
    kani::cover!(r.count == 65, "read65 full output span is reachable");
    kani::cover!(
        r.skip > 0 && r.count > 0,
        "read65 leading fill precedes copied bits"
    );
    kani::cover!(
        r.count > 0 && r.skip + r.count < 65,
        "read65 trailing fill follows copied bits"
    );
    kani::cover!(
        r.start < 64 && r.start + r.count > 64,
        "read65 first source word boundary is reachable"
    );
    kani::cover!(
        r.start < 128 && r.start + r.count > 128,
        "read65 second source word boundary is reachable"
    );
    kani::cover!(
        r.count > 0 && r.start + r.count == 130,
        "read65 last logical source bit is reachable"
    );
    kani::cover!(
        r.count > 0 && r.fill != 0,
        "read65 nonzero fill and copied data coexist"
    );
    kani::cover!(r.byte == 0, "read65 prefix guard is reachable");
    kani::cover!(r.byte == 17, "read65 suffix guard is reachable");
    kani::cover!(r.byte == 16, "read65 native padding byte is reachable");
    kani::cover!(
        r.skip < 64 && r.skip + r.count > 64,
        "read65 destination word boundary is reachable"
    );
}

#[kani::proof]
#[kani::unwind(27)]
fn window_read130_matches_reference() {
    let r = read::<130, 24, 26, false>();
    kani::cover!(r.count == 0, "read130 empty copied span is reachable");
    kani::cover!(r.count == 1, "read130 one copied bit is reachable");
    kani::cover!(r.count > 1, "read130 multiple copied bits are reachable");
    kani::cover!(r.count == 130, "read130 full output span is reachable");
    kani::cover!(
        r.skip > 0 && r.count > 0,
        "read130 leading fill precedes copied bits"
    );
    kani::cover!(
        r.count > 0 && r.skip + r.count < 130,
        "read130 trailing fill follows copied bits"
    );
    kani::cover!(
        r.start < 64 && r.start + r.count > 64,
        "read130 first source word boundary is reachable"
    );
    kani::cover!(
        r.start < 128 && r.start + r.count > 128,
        "read130 second source word boundary is reachable"
    );
    kani::cover!(
        r.count > 0 && r.start + r.count == 130,
        "read130 last logical source bit is reachable"
    );
    kani::cover!(
        r.count > 0 && r.fill != 0,
        "read130 nonzero fill and copied data coexist"
    );
    kani::cover!(r.byte == 0, "read130 prefix guard is reachable");
    kani::cover!(r.byte == 25, "read130 suffix guard is reachable");
    kani::cover!(r.byte == 24, "read130 native padding byte is reachable");
    kani::cover!(
        r.skip < 64 && r.skip + r.count > 64,
        "read130 destination word boundary is reachable"
    );
    kani::cover!(
        r.skip < 128 && r.skip + r.count > 128,
        "read130 last destination word is reachable"
    );
}

#[kani::proof]
#[kani::unwind(27)]
fn window_fill13_does_not_read_source() {
    let r = read::<13, 2, 4, true>();
    kani::cover!(r.start == u64::MAX, "fill13 maximum start is reachable");
    kani::cover!(r.skip == u64::MAX, "fill13 maximum skip is reachable");
    kani::cover!(
        r.source_width == 0,
        "fill13 empty source width is reachable"
    );
    kani::cover!(
        r.source_width == u64::MAX,
        "fill13 maximum source width is reachable"
    );
    kani::cover!(r.fill == 0, "fill13 zero fill is reachable");
    kani::cover!(r.fill == u64::MAX, "fill13 one fill is reachable");
    kani::cover!(
        r.fill == 1u64 << 63,
        "fill13 single high fill bit is reachable"
    );
    kani::cover!(r.byte == 0, "fill13 prefix guard is reachable");
    kani::cover!(r.byte == 3, "fill13 suffix guard is reachable");
    kani::cover!(r.byte == 2, "fill13 native padding byte is reachable");
}

#[kani::proof]
#[kani::unwind(27)]
fn window_fill65_does_not_read_source() {
    let r = read::<65, 16, 18, true>();
    kani::cover!(r.start == u64::MAX, "fill65 maximum start is reachable");
    kani::cover!(r.skip == u64::MAX, "fill65 maximum skip is reachable");
    kani::cover!(
        r.source_width == 0,
        "fill65 empty source width is reachable"
    );
    kani::cover!(
        r.source_width == u64::MAX,
        "fill65 maximum source width is reachable"
    );
    kani::cover!(r.fill == 0, "fill65 zero fill is reachable");
    kani::cover!(r.fill == u64::MAX, "fill65 one fill is reachable");
    kani::cover!(
        r.fill == 1u64 << 63,
        "fill65 single high fill bit is reachable"
    );
    kani::cover!(r.byte == 0, "fill65 prefix guard is reachable");
    kani::cover!(r.byte == 17, "fill65 suffix guard is reachable");
    kani::cover!(r.byte == 16, "fill65 native padding byte is reachable");
}

#[kani::proof]
#[kani::unwind(27)]
fn window_fill130_does_not_read_source() {
    let r = read::<130, 24, 26, true>();
    kani::cover!(r.start == u64::MAX, "fill130 maximum start is reachable");
    kani::cover!(r.skip == u64::MAX, "fill130 maximum skip is reachable");
    kani::cover!(
        r.source_width == 0,
        "fill130 empty source width is reachable"
    );
    kani::cover!(
        r.source_width == u64::MAX,
        "fill130 maximum source width is reachable"
    );
    kani::cover!(r.fill == 0, "fill130 zero fill is reachable");
    kani::cover!(r.fill == u64::MAX, "fill130 one fill is reachable");
    kani::cover!(
        r.fill == 1u64 << 63,
        "fill130 single high fill bit is reachable"
    );
    kani::cover!(r.byte == 0, "fill130 prefix guard is reachable");
    kani::cover!(r.byte == 25, "fill130 suffix guard is reachable");
    kani::cover!(r.byte == 24, "fill130 native padding byte is reachable");
}

#[kani::proof]
#[kani::unwind(4)]
fn window_read_empty_does_not_access_pointers() {
    let source_width: u64 = kani::any();
    let start: u64 = kani::any();
    let skip: u64 = kani::any();
    let count: u64 = kani::any();
    let fill: u64 = kani::any();
    let live: bool = kani::any();
    let mut memory: [u8; 3] = kani::any();
    let before = memory;
    unsafe {
        let dst = if live {
            memory.as_mut_ptr().add(1)
        } else {
            std::ptr::null_mut()
        };
        wide_ops::wide_window_read(
            dst,
            std::ptr::null(),
            source_width,
            0,
            start,
            skip,
            count,
            fill,
        );
    }
    assert!(
        memory == before,
        "simulator window read-plane semantics mismatch"
    );
    kani::cover!(live, "empty read live destination is reachable");
    kani::cover!(!live, "empty read null destination is reachable");
    kani::cover!(
        source_width == 0,
        "empty read zero source width is reachable"
    );
    kani::cover!(
        source_width == u64::MAX,
        "empty read maximum source width is reachable"
    );
    kani::cover!(start == u64::MAX, "empty read maximum start is reachable");
    kani::cover!(skip == u64::MAX, "empty read maximum skip is reachable");
    kani::cover!(count == u64::MAX, "empty read maximum count is reachable");
    kani::cover!(fill == u64::MAX, "empty read nonzero fill is reachable");
}
struct Observation {
    start: u64,
    skip: u64,
    count: u64,
    fill: u64,
    source_width: u64,
    byte: usize,
}
fn read<const WINDOW: usize, const NB: usize, const B: usize, const FILL: bool>() -> Observation {
    let source: [u8; 25] = kani::any();
    let source_before = source;
    let mut memory: [u8; B] = kani::any();
    let before = memory;
    let start: u64 = kani::any();
    let skip: u64 = kani::any();
    let count = if FILL { 0 } else { kani::any::<u64>() };
    let source_width = if FILL { kani::any::<u64>() } else { 130 };
    let fill: u64 = kani::any();
    if !FILL {
        kani::assume(start <= 130);
        kani::assume(skip <= WINDOW as u64);
        kani::assume(count <= 130 - start);
        kani::assume(count <= WINDOW as u64 - skip);
    }
    unsafe {
        // The active source exposes exactly 24 bytes from this unaligned
        // pointer. Count zero may not dereference its null source pointer.
        let src = if FILL {
            std::ptr::null()
        } else {
            source.as_ptr().add(1)
        };
        wide_ops::wide_window_read(
            memory.as_mut_ptr().add(1),
            src,
            source_width,
            WINDOW as u64,
            start,
            skip,
            count,
            fill,
        );
    }
    let byte = usize::from(kani::any::<u8>()) % B;
    let expected = if byte == 0 || byte == B - 1 {
        before[byte]
    } else {
        let mut value = 0u8;
        for bit in 0..8 {
            value |= u8::from(words::window_read_bit(
                &source[1..],
                WINDOW,
                start,
                skip,
                count,
                fill,
                (byte - 1) * 8 + bit,
            )) << bit;
        }
        value
    };
    assert!(
        B == NB + 2 && memory[byte] == expected && source == source_before,
        "simulator window read-plane semantics mismatch"
    );
    Observation {
        start,
        skip,
        count,
        fill,
        source_width,
        byte,
    }
}
