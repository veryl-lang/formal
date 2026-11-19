//! Fixed 512-bit source-offset store spanning two real wide-log records.
//! Source bits1..511 replace destination bits2..512 (half-open intervals).
//! Every storage bit and both guards are arbitrary. Four roots partition all
//! direct/log-only and external/exact-alias choices. The earlier overlapping
//! RHS makes cross-word snapshot preservation observable. No other geometry,
//! pointer overlap, histories, allocation failure, source lowering or X/Z claim.
//! IEEE1800-2023 11.5.1 supplies bit origins; record spans/capacities are ABI rules.
//! Independent bit origins check every staging/final/payload bit. A separately
//! proved four128-bit-limb integer model classifies changes in each record.
//! One preallocated pair avoids growth. Real stores/appends and both commits run;
//! a checked exact count is normalized to2 before commits. A distinct panic
//! rejects growth. An inline constructor summary for0..8 words has its own
//! unstubbed refinement proof; the positive operation uses exactly8 words. All unwinding/safety checks stay
//! enabled: word loops unwind9, the built-in memcmp loop67 (source frame65).
//! Notifications use one-threaded, fully reset scalar globals. Exact64-byte
//! source extents, successful allocation and a little-endian64-bit target are
//! premises. Live record fields/payloads, guards and external sources are frames.
use veryl_formal_semantics::words;
use veryl_simulator::{
    ir::{WriteLogBuffer, ff_commit_from_log, ff_commit_from_log_marking},
    wide_ops,
};
#[cfg(not(all(target_endian = "little", target_pointer_width = "64")))]
compile_error!("chunked source-offset proofs require a little-endian64-bit target");
macro_rules! operation_root {
    ($name:ident,$flags:literal) => {
        #[kani::proof]
        #[kani::unwind(9)]
        #[kani::stub(veryl_simulator::ir::WriteLogBuffer::grow_wide_to, reject_growth)]
        #[kani::stub(smallvec::SmallVec::<[u64;16]>::from_elem,eight_word_scratch)]
        fn $name() {
            let o = check::<$flags>();
            covers(&o);
        }
    };
}
operation_root!(chunked_source_log_external_matches_reference, 0);
operation_root!(chunked_source_direct_external_matches_reference, 1);
operation_root!(chunked_source_log_alias_matches_reference, 2);
operation_root!(chunked_source_direct_alias_matches_reference, 3);
struct Observation {
    first: bool,
    last: bool,
    byte: usize,
    bit: usize,
    frame: usize,
}
fn covers(o: &Observation) {
    kani::cover!(!o.first && !o.last, "offset chunks have no changed record");
    kani::cover!(
        o.first && !o.last,
        "offset chunks change only the first record"
    );
    kani::cover!(
        !o.first && o.last,
        "offset chunks change only the last record"
    );
    kani::cover!(o.first && o.last, "offset chunks change both records");
    kani::cover!(o.byte == 0, "offset chunks prefix guard is reachable");
    kani::cover!(o.byte == 65, "offset chunks suffix guard is reachable");
    kani::cover!(
        o.byte == 64 && o.bit == 7,
        "offset chunks last written bit is reachable"
    );
    kani::cover!(
        o.byte == 1 && o.bit == 0,
        "offset chunks retained low bit is reachable"
    );
    kani::cover!(
        o.frame == 0,
        "offset chunks first live payload byte is reachable"
    );
    kani::cover!(
        o.frame == 55,
        "offset chunks last live payload byte is reachable"
    );
}
fn check<const FLAGS: u8>() -> Observation {
    let mut memory: [u8; 66] = kani::any();
    let before = memory;
    let source: [u8; 65] = kani::any();
    let source_before = source;
    let direct = FLAGS & 1 != 0;
    let alias = FLAGS & 2 != 0;
    let input = if alias {
        &before[1..65]
    } else {
        &source_before[1..65]
    };
    let expected = words::shifted_window512(&before[1..65], input);
    let mut old = [0u128; 4];
    for (i, v) in old.iter_mut().enumerate() {
        *v = u128::from_le_bytes(before[1 + i * 16..17 + i * 16].try_into().unwrap());
    }
    let first = expected[0] != old[0]
        || expected[1] != old[1]
        || expected[2] != old[2]
        || expected[3] as u64 != old[3] as u64;
    let last = expected[3] >> 64 != old[3] >> 64;
    let byte = usize::from(kani::any::<u8>()) % 66;
    let bit = usize::from(kani::any::<u8>() & 7);
    let frame = usize::from(kani::any::<u8>()) % 56;
    let mut log = WriteLogBuffer::with_capacity(0, 2);
    let capacity = (log.narrow_capacity(), log.wide_capacity());
    unsafe {
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
            2,
            (512u64 << 32) | 510,
            1 | (u64::from(direct) << 32),
            1,
        );
    }
    assert!(
        log.narrow_count() == 0
            && log.wide_count() == 2
            && capacity == (log.narrow_capacity(), log.wide_capacity()),
        "simulator chunked source-offset semantics mismatch"
    );
    // Normalize only the count whose exact value was just checked.
    log.wide_count = 2;
    let wanted = expected_bit(&before, input, byte, bit);
    assert!(
        words::bit(&memory, byte * 8 + bit)
            == if direct {
                wanted
            } else {
                words::bit(&before, byte * 8 + bit)
            },
        "simulator chunked source-offset semantics mismatch"
    );
    let mut offsets = [0u32; 2];
    let mut sizes = [0u8; 2];
    let mut pads = [[0u8; 3]; 2];
    let mut payloads = [[0u8; 56]; 2];
    let payload_byte = usize::from(kani::any::<u8>()) % 56;
    for i in 0..2 {
        let e = &log.wide_entries_slice()[i];
        let start = i * 56;
        let length = if i == 0 { 56 } else { 8 };
        assert!(
            e.offset as usize == 1 + start && usize::from(e.native_bytes) == length,
            "simulator chunked source-offset semantics mismatch"
        );
        if payload_byte < length {
            assert!(
                words::bit(&e.payload, payload_byte * 8 + bit)
                    == expected_bit(&before, input, 1 + start + payload_byte, bit),
                "simulator chunked source-offset semantics mismatch"
            );
        }
        offsets[i] = e.offset;
        sizes[i] = e.native_bytes;
        pads[i] = e._pad;
        payloads[i] = e.payload;
    }
    let mut plain = memory;
    let mut marked = memory;
    ff_commit_from_log(&mut plain, &log);
    unsafe {
        FIRST_EVENT = !direct && first;
        LAST_EVENT = !direct && last;
        SEEN = 0;
    }
    let mut observer = observe_change;
    ff_commit_from_log_marking(&mut marked, &log, &mut observer);
    assert!(
        words::bit(&plain, byte * 8 + bit) == wanted
            && words::bit(&marked, byte * 8 + bit) == wanted
            && source == source_before
            && unsafe { SEEN } == usize::from(!direct && first) + usize::from(!direct && last),
        "simulator chunked source-offset semantics mismatch"
    );
    assert!(
        log.narrow_count() == 0
            && log.wide_count() == 2
            && capacity == (log.narrow_capacity(), log.wide_capacity()),
        "simulator chunked source-offset semantics mismatch"
    );
    for i in 0..2 {
        let e = &log.wide_entries_slice()[i];
        assert!(
            e.offset == offsets[i]
                && e.native_bytes == sizes[i]
                && e._pad == pads[i]
                && e.payload[frame] == payloads[i][frame],
            "simulator chunked source-offset semantics mismatch"
        );
    }
    Observation {
        first,
        last,
        byte,
        bit,
        frame,
    }
}
fn expected_bit(before: &[u8; 66], source: &[u8], byte: usize, bit: usize) -> bool {
    if byte == 0 || byte == 65 {
        return words::bit(before, byte * 8 + bit);
    }
    words::window_stored_from_bit(&before[1..65], source, 512, 2, 510, 1, (byte - 1) * 8 + bit)
}
static mut FIRST_EVENT: bool = false;
static mut LAST_EVENT: bool = false;
static mut SEEN: usize = 0;
fn observe_change(offset: usize, size: usize) {
    // One thread; every observer field is reset before the marking commit.
    unsafe {
        let first = FIRST_EVENT && SEEN == 0;
        assert!(
            (if first { FIRST_EVENT } else { LAST_EVENT })
                && SEEN < usize::from(FIRST_EVENT) + usize::from(LAST_EVENT)
                && offset == if first { 1 } else { 57 }
                && size == if first { 56 } else { 8 },
            "simulator chunked source-offset semantics mismatch"
        );
        SEEN += 1;
    }
}
fn reject_growth(_: &mut WriteLogBuffer, _: usize) {
    panic!("bounded chunked source-offset store reached wide-pool growth");
}

fn eight_word_scratch<A: smallvec::Array>(elem: A::Item, n: usize) -> smallvec::SmallVec<A>
where
    A::Item: Clone,
{
    assert!(
        n <= 8 && A::size() >= 8,
        "bounded scratch summary precondition violated"
    );
    // Equal under the checked precondition; expose the finite length range.
    let n = usize::from(n as u8 & 15);
    let mut buf = std::mem::MaybeUninit::<A>::uninit();
    // SAFETY: Array guarantees contiguous items. Exactly the first n slots are
    // initialized, and n<=8 fits A. Only the [u64;16] constructor is stubbed.
    unsafe {
        let ptr = buf.as_mut_ptr().cast::<A::Item>();
        for i in 0..8 {
            if i < n {
                ptr.add(i).write(elem.clone());
            }
        }
        smallvec::SmallVec::from_buf_and_len_unchecked(buf, n)
    }
}
#[kani::proof]
#[kani::unwind(17)]
fn eight_word_scratch_refines_constructor() {
    let n = usize::from(kani::any::<u8>() % 9);
    let elem: u64 = kani::any();
    let actual = match n {
        0 => smallvec::SmallVec::<[u64; 16]>::from_elem(elem, 0),
        1 => smallvec::SmallVec::<[u64; 16]>::from_elem(elem, 1),
        2 => smallvec::SmallVec::<[u64; 16]>::from_elem(elem, 2),
        3 => smallvec::SmallVec::<[u64; 16]>::from_elem(elem, 3),
        4 => smallvec::SmallVec::<[u64; 16]>::from_elem(elem, 4),
        5 => smallvec::SmallVec::<[u64; 16]>::from_elem(elem, 5),
        6 => smallvec::SmallVec::<[u64; 16]>::from_elem(elem, 6),
        7 => smallvec::SmallVec::<[u64; 16]>::from_elem(elem, 7),
        _ => smallvec::SmallVec::<[u64; 16]>::from_elem(elem, 8),
    };
    let modeled = eight_word_scratch::<[u64; 16]>(elem, n);
    assert!(
        actual.len() == n && modeled.len() == n && !actual.spilled() && !modeled.spilled(),
        "simulator chunked source-offset semantics mismatch"
    );
    let i = usize::from(kani::any::<u8>() & 7);
    if i < n {
        assert!(
            actual[i] == elem && modeled[i] == elem,
            "simulator chunked source-offset semantics mismatch"
        );
    }
    kani::cover!(n == 0, "bounded scratch size0 is reachable");
    kani::cover!(n == 1, "bounded scratch size1 is reachable");
    kani::cover!(n == 2, "bounded scratch size2 is reachable");
    kani::cover!(n == 3, "bounded scratch size3 is reachable");
    kani::cover!(n == 4, "bounded scratch size4 is reachable");
    kani::cover!(n == 5, "bounded scratch size5 is reachable");
    kani::cover!(n == 6, "bounded scratch size6 is reachable");
    kani::cover!(n == 7, "bounded scratch size7 is reachable");
    kani::cover!(n == 8, "bounded scratch size8 is reachable");
    kani::cover!(
        n == 8 && i == 0,
        "eight-word scratch first live word is reachable"
    );
    kani::cover!(
        n == 8 && i == 7,
        "eight-word scratch last live word is reachable"
    );
    // Teardown is outside this constructor-equivalence contract.
    std::mem::forget(actual);
    std::mem::forget(modeled);
}
#[kani::proof]
#[kani::unwind(5)]
fn shifted_512_integer_refines_bit_origins() {
    let old: [u8; 64] = kani::any();
    let source: [u8; 64] = kani::any();
    let expected = words::shifted_window512(&old, &source);
    let pos = usize::from(kani::any::<u16>() & 511);
    let scalar_bit = expected[pos / 128] >> (pos % 128) & 1 != 0;
    assert!(
        scalar_bit == words::window_stored_from_bit(&old, &source, 512, 2, 510, 1, pos),
        "simulator chunked source-offset model mismatch"
    );
    kani::cover!(pos == 1, "shifted512 last retained bit is reachable");
    kani::cover!(pos == 2, "shifted512 first copied bit is reachable");
    kani::cover!(pos == 128, "shifted512 limb boundary is reachable");
    kani::cover!(pos == 511, "shifted512 last copied bit is reachable");
}
