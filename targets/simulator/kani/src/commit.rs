//! Both real commit wrappers against owned byte-replacement and event references.
//! Single narrow records cover classes 1/2/4/8; wide lengths are fixed at
//! 3/7/15/17/56. Separate fixed sequences check same-pool and mixed-pool ordering.
//! Invalid spans are rejected as whole records. Events describe each changing entry,
//! not just the final buffer difference. No general FF scheduler or source NBA theorem.

use veryl_simulator::ir::{WriteLogBuffer, ff_commit_from_log, ff_commit_from_log_marking};

#[kani::proof]
#[kani::unwind(9)]
fn narrow_commit_matches_reference() {
    let before: [u8; 16] = kani::any();
    let payload: u64 = kani::any();
    let offset = u32::from(kani::any::<u8>() % 17);
    let width = [1u16, 2, 4, 8][usize::from(kani::any::<u8>() & 3)];
    let mut log = WriteLogBuffer::with_capacity(1, 0);
    unsafe {
        (log.grow_push_narrow)(&mut log, offset, payload, width as u32);
    }
    let mut plain = before;
    let mut marked = before;
    ff_commit_from_log(&mut plain, &log);
    let mut count = 0usize;
    ff_commit_from_log_marking(&mut marked, &log, &mut |off, nb| {
        assert!(
            off == offset as usize && nb == width as usize,
            "simulator commit semantics mismatch"
        );
        count += 1;
    });
    let observed = usize::from(kani::any::<u8>() % 16);
    let valid = offset as usize + width as usize <= 16;
    let expected =
        if valid && observed >= offset as usize && observed - (offset as usize) < width as usize {
            payload.to_le_bytes()[observed - offset as usize]
        } else {
            before[observed]
        };
    assert!(
        plain[observed] == expected && marked[observed] == expected,
        "simulator commit semantics mismatch"
    );
    let mut differs = false;
    for i in 0..8 {
        if valid && i < width as usize && before[offset as usize + i] != payload.to_le_bytes()[i] {
            differs = true;
        }
    }
    assert!(
        count == usize::from(differs),
        "simulator commit semantics mismatch"
    );
    let entry = &log.narrow_entries_slice()[0];
    assert!(
        log.narrow_count() == 1
            && log.wide_count() == 0
            && entry.offset == offset
            && entry.width_class == width
            && entry.payload == payload,
        "simulator commit semantics mismatch"
    );
    kani::cover!(width == 1 && valid, "commit one byte is reachable");
    kani::cover!(width == 2 && valid, "commit two bytes is reachable");
    kani::cover!(width == 4 && valid, "commit four bytes is reachable");
    kani::cover!(
        width == 8 && offset == 1,
        "commit unaligned eight bytes is reachable"
    );
    kani::cover!(!valid, "commit outside range is reachable");
    kani::cover!(count == 0 && valid, "commit unchanged value is reachable");
    kani::cover!(count == 1, "commit changed value is reachable");
}

#[kani::proof]
#[kani::unwind(4)]
fn wide_three_commit_matches_reference() {
    wide::<3>();
}
#[kani::proof]
#[kani::unwind(8)]
fn wide_seven_commit_matches_reference() {
    wide::<7>();
}
#[kani::proof]
#[kani::unwind(16)]
fn wide_fifteen_commit_matches_reference() {
    wide::<15>();
}
#[kani::proof]
#[kani::unwind(18)]
fn large_seventeen_commit_matches_reference() {
    wide::<17>();
}
#[kani::proof]
#[kani::unwind(57)]
fn large_fifty_six_commit_matches_reference() {
    wide::<56>();
}

fn wide<const CAP: usize>() {
    let source: [u8; CAP] = kani::any();
    let before: [u8; 64] = kani::any();
    let nb = CAP;
    let offset = u32::from(kani::any::<u8>() % 65);
    let mut log = WriteLogBuffer::with_capacity(0, 1);
    unsafe {
        (log.grow_push_wide)(&mut log, offset, source.as_ptr(), nb as u32);
    }
    let recorded = log.wide_entries_slice()[0].payload;
    let mut plain = before;
    let mut marked = before;
    ff_commit_from_log(&mut plain, &log);
    let mut count = 0usize;
    ff_commit_from_log_marking(&mut marked, &log, &mut |off, size| {
        assert!(
            off == offset as usize && size == nb,
            "simulator commit semantics mismatch"
        );
        count += 1;
    });
    let valid = offset as usize + nb <= 64;
    let observed = usize::from(kani::any::<u8>() % 64);
    let expected = if valid && observed >= offset as usize && observed - (offset as usize) < nb {
        source[observed - offset as usize]
    } else {
        before[observed]
    };
    assert!(
        plain[observed] == expected && marked[observed] == expected,
        "simulator commit semantics mismatch"
    );
    let mut differs = false;
    for i in 0..CAP {
        if valid && i < nb && before[offset as usize + i] != source[i] {
            differs = true;
        }
    }
    assert!(
        count == usize::from(differs),
        "simulator commit semantics mismatch"
    );
    let entry = &log.wide_entries_slice()[0];
    let log_observed = usize::from(kani::any::<u8>() % 56);
    assert!(
        log.wide_count() == 1
            && log.narrow_count() == 0
            && entry.offset == offset
            && entry.native_bytes as usize == nb
            && entry.payload[log_observed] == recorded[log_observed],
        "simulator commit semantics mismatch"
    );
    kani::cover!(
        offset as usize + nb == 64,
        "wide commit exact end is reachable"
    );
    kani::cover!(valid && offset == 1, "wide commit unaligned is reachable");
    kani::cover!(!valid, "wide commit outside range is reachable");
    kani::cover!(valid && count == 0, "wide commit unchanged is reachable");
    kani::cover!(count == 1, "wide commit changed is reachable");
}

#[kani::proof]
#[kani::unwind(4)]
fn narrow_commit_order_matches_reference() {
    order::<0>();
}
#[kani::proof]
#[kani::unwind(4)]
fn mixed_commit_order_matches_reference() {
    order::<1>();
}

#[kani::proof]
#[kani::unwind(4)]
fn wide_commit_order_matches_reference() {
    order::<2>();
}

fn order<const CASE: u8>() {
    let with_wide = CASE == 1;
    let before: [u8; 4] = kani::any();
    let a: u8 = kani::any();
    let b: u8 = kani::any();
    let c: u8 = kani::any();
    let mut log = WriteLogBuffer::with_capacity(
        if CASE == 2 { 0 } else { 2 },
        if CASE == 0 {
            0
        } else if CASE == 1 {
            1
        } else {
            2
        },
    );
    unsafe {
        if with_wide {
            (log.grow_push_wide)(&mut log, 1, &c, 1);
        }
        if CASE == 2 {
            (log.grow_push_wide)(&mut log, 1, &a, 1);
            (log.grow_push_wide)(&mut log, 1, &b, 1);
        } else {
            (log.grow_push_narrow)(&mut log, 1, a as u64, 1);
            (log.grow_push_narrow)(&mut log, 1, b as u64, 1);
        }
    }
    let mut plain = before;
    let mut marked = before;
    ff_commit_from_log(&mut plain, &log);
    let mut count = 0;
    ff_commit_from_log_marking(&mut marked, &log, &mut |off, nb| {
        assert!(off == 1 && nb == 1, "simulator commit order mismatch");
        count += 1;
    });
    let observed = usize::from(kani::any::<u8>() & 3);
    let expected = if observed == 1 {
        if with_wide { c } else { b }
    } else {
        before[observed]
    };
    let changes =
        usize::from(a != before[1]) + usize::from(b != a) + usize::from(with_wide && c != b);
    assert!(
        plain[observed] == expected && marked[observed] == expected && count == changes,
        "simulator commit order mismatch"
    );
    kani::cover!(a != b, "commit repeated byte differs is reachable");
    kani::cover!(
        a != before[1] && b == before[1] && (!with_wide || c == before[1]),
        "commit restored value still marks is reachable"
    );
    kani::cover!(count == 0, "commit all writes unchanged is reachable");
    if with_wide {
        kani::cover!(
            c != b,
            "commit wide pool wins over later narrow push is reachable"
        );
    }
}
