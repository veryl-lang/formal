//! Bounded read planes for every source width 1..130, partitioned into native
//! widths 1..64, two-word widths 65..128, and tail widths 129..130. Storage has
//! the declared ABI capacity: the next power-of-two byte type below 65 bits,
//! then whole 64-bit words. Each source extent ends at its allocation boundary;
//! the unused prefix and all source/padding bytes are arbitrary and preserved.
//! Windows 13/65/130 have guarded native extents 2/16/24. Arbitrary u64 start,
//! skip and count satisfy nonwrapping source/output interval bounds. Fill is an
//! arbitrary repeated u64 pattern. Source and destination never overlap.
//! Independent bit origins check every output byte, clean destination padding,
//! both guards and the source frame. All real helper operations execute without
//! stubs on a little-endian 64-bit target. IEEE 1800-2023 11.5.1 motivates the
//! logical in-range/out-of-range distinction; ABI capacities and physical-plane
//! fill are separate implementation contracts. No bounds lowering, arbitrary
//! aliasing, source program, or complete four-state encoding theorem is claimed.
use veryl_formal_semantics::words;
use veryl_simulator::wide_ops;
#[cfg(not(all(target_endian = "little", target_pointer_width = "64")))]
compile_error!("source-width read proofs require a little-endian 64-bit target");

#[kani::proof]
#[kani::unwind(27)]
fn window_read13_source_native_matches_reference() {
    let r = read::<13, 2, 4, 0>();
    kani::cover!(
        r.width == 1,
        "source13 native minimum source width is reachable"
    );
    kani::cover!(
        r.width == 64,
        "source13 native maximum source width is reachable"
    );
    kani::cover!(r.count == 0, "source13 native empty copy is reachable");
    kani::cover!(r.count == 1, "source13 native one copied bit is reachable");
    kani::cover!(
        r.count > 1,
        "source13 native multiple copied bits are reachable"
    );
    kani::cover!(
        r.count > 0 && r.start + r.count == r.width,
        "source13 native last logical source bit is reachable"
    );
    kani::cover!(
        r.count > 0 && r.skip > 0,
        "source13 native leading fill is reachable"
    );
    kani::cover!(
        r.count > 0 && r.skip + r.count < 13,
        "source13 native trailing fill is reachable"
    );
    kani::cover!(
        r.width == 64 && r.count == 13,
        "source13 native largest permitted copy is reachable"
    );
    kani::cover!(
        r.count > 0 && r.fill != 0,
        "source13 native nonzero fill coexists with copying"
    );
    kani::cover!(r.byte == 0, "source13 native prefix guard is reachable");
    kani::cover!(r.byte == 3, "source13 native suffix guard is reachable");
    kani::cover!(
        r.byte == 2,
        "source13 native last native destination byte is reachable"
    );
    kani::cover!(r.width == 8, "source13 native source width 8 is reachable");
    kani::cover!(r.width == 9, "source13 native source width 9 is reachable");
    kani::cover!(
        r.width == 16,
        "source13 native source width 16 is reachable"
    );
    kani::cover!(
        r.width == 17,
        "source13 native source width 17 is reachable"
    );
    kani::cover!(
        r.width == 32,
        "source13 native source width 32 is reachable"
    );
    kani::cover!(
        r.width == 33,
        "source13 native source width 33 is reachable"
    );
}

#[kani::proof]
#[kani::unwind(27)]
fn window_read13_source_big_matches_reference() {
    let r = read::<13, 2, 4, 1>();
    kani::cover!(
        r.width == 65,
        "source13 big minimum source width is reachable"
    );
    kani::cover!(
        r.width == 128,
        "source13 big maximum source width is reachable"
    );
    kani::cover!(r.count == 0, "source13 big empty copy is reachable");
    kani::cover!(r.count == 1, "source13 big one copied bit is reachable");
    kani::cover!(
        r.count > 1,
        "source13 big multiple copied bits are reachable"
    );
    kani::cover!(
        r.count > 0 && r.start + r.count == r.width,
        "source13 big last logical source bit is reachable"
    );
    kani::cover!(
        r.count > 0 && r.skip > 0,
        "source13 big leading fill is reachable"
    );
    kani::cover!(
        r.count > 0 && r.skip + r.count < 13,
        "source13 big trailing fill is reachable"
    );
    kani::cover!(
        r.width == 128 && r.count == 13,
        "source13 big largest permitted copy is reachable"
    );
    kani::cover!(
        r.count > 0 && r.fill != 0,
        "source13 big nonzero fill coexists with copying"
    );
    kani::cover!(r.byte == 0, "source13 big prefix guard is reachable");
    kani::cover!(r.byte == 3, "source13 big suffix guard is reachable");
    kani::cover!(
        r.byte == 2,
        "source13 big last native destination byte is reachable"
    );
    kani::cover!(
        r.start < 64 && r.start + r.count > 64,
        "source13 big first source word crossing is reachable"
    );
}

#[kani::proof]
#[kani::unwind(27)]
fn window_read13_source_tail_matches_reference() {
    let r = read::<13, 2, 4, 2>();
    kani::cover!(
        r.width == 129,
        "source13 tail minimum source width is reachable"
    );
    kani::cover!(
        r.width == 130,
        "source13 tail maximum source width is reachable"
    );
    kani::cover!(r.count == 0, "source13 tail empty copy is reachable");
    kani::cover!(r.count == 1, "source13 tail one copied bit is reachable");
    kani::cover!(
        r.count > 1,
        "source13 tail multiple copied bits are reachable"
    );
    kani::cover!(
        r.count > 0 && r.start + r.count == r.width,
        "source13 tail last logical source bit is reachable"
    );
    kani::cover!(
        r.count > 0 && r.skip > 0,
        "source13 tail leading fill is reachable"
    );
    kani::cover!(
        r.count > 0 && r.skip + r.count < 13,
        "source13 tail trailing fill is reachable"
    );
    kani::cover!(
        r.width == 130 && r.count == 13,
        "source13 tail largest permitted copy is reachable"
    );
    kani::cover!(
        r.count > 0 && r.fill != 0,
        "source13 tail nonzero fill coexists with copying"
    );
    kani::cover!(r.byte == 0, "source13 tail prefix guard is reachable");
    kani::cover!(r.byte == 3, "source13 tail suffix guard is reachable");
    kani::cover!(
        r.byte == 2,
        "source13 tail last native destination byte is reachable"
    );
    kani::cover!(
        r.start < 64 && r.start + r.count > 64,
        "source13 tail first source word crossing is reachable"
    );
    kani::cover!(
        r.start < 128 && r.start + r.count > 128,
        "source13 tail second source word crossing is reachable"
    );
}

#[kani::proof]
#[kani::unwind(27)]
fn window_read65_source_native_matches_reference() {
    let r = read::<65, 16, 18, 0>();
    kani::cover!(
        r.width == 1,
        "source65 native minimum source width is reachable"
    );
    kani::cover!(
        r.width == 64,
        "source65 native maximum source width is reachable"
    );
    kani::cover!(r.count == 0, "source65 native empty copy is reachable");
    kani::cover!(r.count == 1, "source65 native one copied bit is reachable");
    kani::cover!(
        r.count > 1,
        "source65 native multiple copied bits are reachable"
    );
    kani::cover!(
        r.count > 0 && r.start + r.count == r.width,
        "source65 native last logical source bit is reachable"
    );
    kani::cover!(
        r.count > 0 && r.skip > 0,
        "source65 native leading fill is reachable"
    );
    kani::cover!(
        r.count > 0 && r.skip + r.count < 65,
        "source65 native trailing fill is reachable"
    );
    kani::cover!(
        r.width == 64 && r.count == 64,
        "source65 native largest permitted copy is reachable"
    );
    kani::cover!(
        r.count > 0 && r.fill != 0,
        "source65 native nonzero fill coexists with copying"
    );
    kani::cover!(r.byte == 0, "source65 native prefix guard is reachable");
    kani::cover!(r.byte == 17, "source65 native suffix guard is reachable");
    kani::cover!(
        r.byte == 16,
        "source65 native last native destination byte is reachable"
    );
    kani::cover!(r.width == 8, "source65 native source width 8 is reachable");
    kani::cover!(r.width == 9, "source65 native source width 9 is reachable");
    kani::cover!(
        r.width == 16,
        "source65 native source width 16 is reachable"
    );
    kani::cover!(
        r.width == 17,
        "source65 native source width 17 is reachable"
    );
    kani::cover!(
        r.width == 32,
        "source65 native source width 32 is reachable"
    );
    kani::cover!(
        r.width == 33,
        "source65 native source width 33 is reachable"
    );
    kani::cover!(
        r.skip < 64 && r.skip + r.count > 64,
        "source65 native first destination word crossing is reachable"
    );
}

#[kani::proof]
#[kani::unwind(27)]
fn window_read65_source_big_matches_reference() {
    let r = read::<65, 16, 18, 1>();
    kani::cover!(
        r.width == 65,
        "source65 big minimum source width is reachable"
    );
    kani::cover!(
        r.width == 128,
        "source65 big maximum source width is reachable"
    );
    kani::cover!(r.count == 0, "source65 big empty copy is reachable");
    kani::cover!(r.count == 1, "source65 big one copied bit is reachable");
    kani::cover!(
        r.count > 1,
        "source65 big multiple copied bits are reachable"
    );
    kani::cover!(
        r.count > 0 && r.start + r.count == r.width,
        "source65 big last logical source bit is reachable"
    );
    kani::cover!(
        r.count > 0 && r.skip > 0,
        "source65 big leading fill is reachable"
    );
    kani::cover!(
        r.count > 0 && r.skip + r.count < 65,
        "source65 big trailing fill is reachable"
    );
    kani::cover!(
        r.width == 128 && r.count == 65,
        "source65 big largest permitted copy is reachable"
    );
    kani::cover!(
        r.count > 0 && r.fill != 0,
        "source65 big nonzero fill coexists with copying"
    );
    kani::cover!(r.byte == 0, "source65 big prefix guard is reachable");
    kani::cover!(r.byte == 17, "source65 big suffix guard is reachable");
    kani::cover!(
        r.byte == 16,
        "source65 big last native destination byte is reachable"
    );
    kani::cover!(
        r.start < 64 && r.start + r.count > 64,
        "source65 big first source word crossing is reachable"
    );
    kani::cover!(
        r.skip < 64 && r.skip + r.count > 64,
        "source65 big first destination word crossing is reachable"
    );
}

#[kani::proof]
#[kani::unwind(27)]
fn window_read65_source_tail_matches_reference() {
    let r = read::<65, 16, 18, 2>();
    kani::cover!(
        r.width == 129,
        "source65 tail minimum source width is reachable"
    );
    kani::cover!(
        r.width == 130,
        "source65 tail maximum source width is reachable"
    );
    kani::cover!(r.count == 0, "source65 tail empty copy is reachable");
    kani::cover!(r.count == 1, "source65 tail one copied bit is reachable");
    kani::cover!(
        r.count > 1,
        "source65 tail multiple copied bits are reachable"
    );
    kani::cover!(
        r.count > 0 && r.start + r.count == r.width,
        "source65 tail last logical source bit is reachable"
    );
    kani::cover!(
        r.count > 0 && r.skip > 0,
        "source65 tail leading fill is reachable"
    );
    kani::cover!(
        r.count > 0 && r.skip + r.count < 65,
        "source65 tail trailing fill is reachable"
    );
    kani::cover!(
        r.width == 130 && r.count == 65,
        "source65 tail largest permitted copy is reachable"
    );
    kani::cover!(
        r.count > 0 && r.fill != 0,
        "source65 tail nonzero fill coexists with copying"
    );
    kani::cover!(r.byte == 0, "source65 tail prefix guard is reachable");
    kani::cover!(r.byte == 17, "source65 tail suffix guard is reachable");
    kani::cover!(
        r.byte == 16,
        "source65 tail last native destination byte is reachable"
    );
    kani::cover!(
        r.start < 64 && r.start + r.count > 64,
        "source65 tail first source word crossing is reachable"
    );
    kani::cover!(
        r.start < 128 && r.start + r.count > 128,
        "source65 tail second source word crossing is reachable"
    );
    kani::cover!(
        r.skip < 64 && r.skip + r.count > 64,
        "source65 tail first destination word crossing is reachable"
    );
}

#[kani::proof]
#[kani::unwind(27)]
fn window_read130_source_native_matches_reference() {
    let r = read::<130, 24, 26, 0>();
    kani::cover!(
        r.width == 1,
        "source130 native minimum source width is reachable"
    );
    kani::cover!(
        r.width == 64,
        "source130 native maximum source width is reachable"
    );
    kani::cover!(r.count == 0, "source130 native empty copy is reachable");
    kani::cover!(r.count == 1, "source130 native one copied bit is reachable");
    kani::cover!(
        r.count > 1,
        "source130 native multiple copied bits are reachable"
    );
    kani::cover!(
        r.count > 0 && r.start + r.count == r.width,
        "source130 native last logical source bit is reachable"
    );
    kani::cover!(
        r.count > 0 && r.skip > 0,
        "source130 native leading fill is reachable"
    );
    kani::cover!(
        r.count > 0 && r.skip + r.count < 130,
        "source130 native trailing fill is reachable"
    );
    kani::cover!(
        r.width == 64 && r.count == 64,
        "source130 native largest permitted copy is reachable"
    );
    kani::cover!(
        r.count > 0 && r.fill != 0,
        "source130 native nonzero fill coexists with copying"
    );
    kani::cover!(r.byte == 0, "source130 native prefix guard is reachable");
    kani::cover!(r.byte == 25, "source130 native suffix guard is reachable");
    kani::cover!(
        r.byte == 24,
        "source130 native last native destination byte is reachable"
    );
    kani::cover!(r.width == 8, "source130 native source width 8 is reachable");
    kani::cover!(r.width == 9, "source130 native source width 9 is reachable");
    kani::cover!(
        r.width == 16,
        "source130 native source width 16 is reachable"
    );
    kani::cover!(
        r.width == 17,
        "source130 native source width 17 is reachable"
    );
    kani::cover!(
        r.width == 32,
        "source130 native source width 32 is reachable"
    );
    kani::cover!(
        r.width == 33,
        "source130 native source width 33 is reachable"
    );
    kani::cover!(
        r.skip < 64 && r.skip + r.count > 64,
        "source130 native first destination word crossing is reachable"
    );
    kani::cover!(
        r.skip < 128 && r.skip + r.count > 128,
        "source130 native second destination word crossing is reachable"
    );
}

#[kani::proof]
#[kani::unwind(27)]
fn window_read130_source_big_matches_reference() {
    let r = read::<130, 24, 26, 1>();
    kani::cover!(
        r.width == 65,
        "source130 big minimum source width is reachable"
    );
    kani::cover!(
        r.width == 128,
        "source130 big maximum source width is reachable"
    );
    kani::cover!(r.count == 0, "source130 big empty copy is reachable");
    kani::cover!(r.count == 1, "source130 big one copied bit is reachable");
    kani::cover!(
        r.count > 1,
        "source130 big multiple copied bits are reachable"
    );
    kani::cover!(
        r.count > 0 && r.start + r.count == r.width,
        "source130 big last logical source bit is reachable"
    );
    kani::cover!(
        r.count > 0 && r.skip > 0,
        "source130 big leading fill is reachable"
    );
    kani::cover!(
        r.count > 0 && r.skip + r.count < 130,
        "source130 big trailing fill is reachable"
    );
    kani::cover!(
        r.width == 128 && r.count == 128,
        "source130 big largest permitted copy is reachable"
    );
    kani::cover!(
        r.count > 0 && r.fill != 0,
        "source130 big nonzero fill coexists with copying"
    );
    kani::cover!(r.byte == 0, "source130 big prefix guard is reachable");
    kani::cover!(r.byte == 25, "source130 big suffix guard is reachable");
    kani::cover!(
        r.byte == 24,
        "source130 big last native destination byte is reachable"
    );
    kani::cover!(
        r.start < 64 && r.start + r.count > 64,
        "source130 big first source word crossing is reachable"
    );
    kani::cover!(
        r.skip < 64 && r.skip + r.count > 64,
        "source130 big first destination word crossing is reachable"
    );
    kani::cover!(
        r.skip < 128 && r.skip + r.count > 128,
        "source130 big second destination word crossing is reachable"
    );
}

#[kani::proof]
#[kani::unwind(27)]
fn window_read130_source_tail_matches_reference() {
    let r = read::<130, 24, 26, 2>();
    kani::cover!(
        r.width == 129,
        "source130 tail minimum source width is reachable"
    );
    kani::cover!(
        r.width == 130,
        "source130 tail maximum source width is reachable"
    );
    kani::cover!(r.count == 0, "source130 tail empty copy is reachable");
    kani::cover!(r.count == 1, "source130 tail one copied bit is reachable");
    kani::cover!(
        r.count > 1,
        "source130 tail multiple copied bits are reachable"
    );
    kani::cover!(
        r.count > 0 && r.start + r.count == r.width,
        "source130 tail last logical source bit is reachable"
    );
    kani::cover!(
        r.count > 0 && r.skip > 0,
        "source130 tail leading fill is reachable"
    );
    kani::cover!(
        r.count > 0 && r.skip + r.count < 130,
        "source130 tail trailing fill is reachable"
    );
    kani::cover!(
        r.width == 130 && r.count == 130,
        "source130 tail largest permitted copy is reachable"
    );
    kani::cover!(
        r.count > 0 && r.fill != 0,
        "source130 tail nonzero fill coexists with copying"
    );
    kani::cover!(r.byte == 0, "source130 tail prefix guard is reachable");
    kani::cover!(r.byte == 25, "source130 tail suffix guard is reachable");
    kani::cover!(
        r.byte == 24,
        "source130 tail last native destination byte is reachable"
    );
    kani::cover!(
        r.start < 64 && r.start + r.count > 64,
        "source130 tail first source word crossing is reachable"
    );
    kani::cover!(
        r.start < 128 && r.start + r.count > 128,
        "source130 tail second source word crossing is reachable"
    );
    kani::cover!(
        r.skip < 64 && r.skip + r.count > 64,
        "source130 tail first destination word crossing is reachable"
    );
    kani::cover!(
        r.skip < 128 && r.skip + r.count > 128,
        "source130 tail second destination word crossing is reachable"
    );
}

struct Observation {
    width: u64,
    start: u64,
    skip: u64,
    count: u64,
    fill: u64,
    byte: usize,
}
fn read<const WINDOW: usize, const NB: usize, const B: usize, const BAND: u8>() -> Observation {
    let width = match BAND {
        0 => usize::from(kani::any::<u8>() & 63) + 1,
        1 => usize::from(kani::any::<u8>() & 63) + 65,
        _ => usize::from(kani::any::<u8>() & 1) + 129,
    };
    let capacity = match BAND {
        0 => width.max(8).next_power_of_two() / 8,
        1 => 16,
        _ => 24,
    };
    let source: [u8; 25] = kani::any();
    let source_before = source;
    let origin = source.len() - capacity;
    let mut memory: [u8; B] = kani::any();
    let before = memory;
    let start: u64 = kani::any();
    let skip: u64 = kani::any();
    let count: u64 = kani::any();
    let fill: u64 = kani::any();
    kani::assume(start <= width as u64 && skip <= WINDOW as u64);
    kani::assume(count <= width as u64 - start && count <= WINDOW as u64 - skip);
    unsafe {
        wide_ops::wide_window_read(
            memory.as_mut_ptr().add(1),
            source.as_ptr().add(origin),
            width as u64,
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
                &source[origin..],
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
        BAND < 3 && B == NB + 2 && memory[byte] == expected && source == source_before,
        "simulator variable-source read-plane semantics mismatch"
    );
    Observation {
        width: width as u64,
        start,
        skip,
        count,
        fill,
        byte,
    }
}
