use super::{GUARD, pattern};
use veryl_formal_semantics::write_log::{NarrowRecord, WideRecord, committed};
use veryl_simulator::ir::{WriteLogBuffer, ff_commit_from_log, ff_commit_from_log_marking};

#[derive(Debug, PartialEq, Eq)]
struct Snapshot {
    counts: (u32, u32),
    capacities: (usize, usize),
    narrow: Vec<(u32, u16, u16, u64)>,
    wide: Vec<(u32, u8, [u8; 3], [u8; 56])>,
}
fn snapshot(log: &WriteLogBuffer) -> Snapshot {
    Snapshot {
        counts: (log.narrow_count(), log.wide_count()),
        capacities: (log.narrow_capacity(), log.wide_capacity()),
        narrow: log
            .narrow_entries_slice()
            .iter()
            .map(|e| (e.offset, e.mask_xz, e.width_class, e.payload))
            .collect(),
        wide: log
            .wide_entries_slice()
            .iter()
            .map(|e| (e.offset, e.native_bytes, e._pad, e.payload))
            .collect(),
    }
}
fn make_log(narrow: &[NarrowRecord], wide: &[WideRecord], wide_first: bool) -> WriteLogBuffer {
    let mut log = WriteLogBuffer::with_capacity(narrow.len(), wide.len());
    for is_wide in [wide_first, !wide_first] {
        if is_wide {
            for entry in wide {
                let blank = [0u8; 1];
                let data = if entry.payload.is_empty() {
                    &blank[..]
                } else {
                    &entry.payload[..entry.payload.len().min(56)]
                };
                unsafe {
                    let idx = log.wide_count() as usize;
                    (log.grow_push_wide)(&mut log, entry.offset, data.as_ptr(), data.len() as u32);
                    (*log.wide_entries_ptr.add(idx)).native_bytes = entry.payload.len() as u8;
                }
            }
        } else {
            for entry in narrow {
                let valid_class = if [1, 2, 4, 8].contains(&entry.width_class) {
                    entry.width_class
                } else {
                    1
                };
                unsafe {
                    let idx = log.narrow_count() as usize;
                    (log.grow_push_narrow)(
                        &mut log,
                        entry.offset,
                        entry.payload,
                        valid_class as u32,
                    );
                    (*log.narrow_entries_ptr.add(idx)).width_class = entry.width_class;
                }
            }
        }
    }
    log
}
fn compare(before: &[u8], narrow: &[NarrowRecord], wide: &[WideRecord], log: &WriteLogBuffer) {
    let (expected, events) = committed(before, narrow, wide);
    let saved = snapshot(log);
    let mut plain = vec![GUARD; before.len() + 32];
    let mut marked = plain.clone();
    plain[16..16 + before.len()].copy_from_slice(before);
    marked[16..16 + before.len()].copy_from_slice(before);
    ff_commit_from_log(&mut plain[16..16 + before.len()], log);
    let mut actual_events = Vec::new();
    ff_commit_from_log_marking(&mut marked[16..16 + before.len()], log, &mut |off, nb| {
        actual_events.push((off, nb))
    });
    assert_eq!(
        &plain[16..16 + before.len()],
        expected,
        "simulator commit semantics mismatch"
    );
    assert_eq!(
        &marked[16..16 + before.len()],
        expected,
        "simulator commit semantics mismatch"
    );
    assert_eq!(actual_events, events, "simulator commit semantics mismatch");
    assert!(
        plain[..16]
            .iter()
            .chain(&plain[16 + before.len()..])
            .chain(&marked[..16])
            .chain(&marked[16 + before.len()..])
            .all(|&b| b == GUARD),
        "simulator commit semantics mismatch"
    );
    assert_eq!(snapshot(log), saved, "simulator commit semantics mismatch");
}

#[test]
fn entry_sizes_and_bounds() {
    for len in [
        0usize, 1, 2, 3, 4, 7, 8, 9, 15, 16, 17, 31, 32, 55, 56, 57, 64, 65, 97,
    ] {
        let offsets: Vec<_> = (0..=len as u32 + 1).chain([u32::MAX]).collect();
        for before_kind in 0..6 {
            let before = pattern(len, before_kind);
            for kind in 0..6 {
                let payload = u64::from_le_bytes(pattern(8, kind).try_into().unwrap());
                for &offset in &offsets {
                    for width_class in [0u16, 1, 2, 3, 4, 5, 7, 8, 9, u16::MAX] {
                        let narrow = [NarrowRecord {
                            offset,
                            width_class,
                            payload,
                        }];
                        let log = make_log(&narrow, &[], false);
                        compare(&before, &narrow, &[], &log);
                    }
                    for size in (0..=57).chain([255]) {
                        let wide = [WideRecord {
                            offset,
                            payload: pattern(size, kind),
                        }];
                        let log = make_log(&[], &wide, true);
                        compare(&before, &[], &wide, &log);
                    }
                }
            }
        }
    }
}

#[test]
fn mixed_order_and_marking() {
    let narrow = [1, 2].map(|payload| NarrowRecord {
        offset: 1,
        width_class: 1,
        payload,
    });
    let wide = [WideRecord {
        offset: 1,
        payload: vec![3],
    }];
    for wide_first in [false, true] {
        compare(
            &[0; 4],
            &narrow,
            &wide,
            &make_log(&narrow, &wide, wide_first),
        );
    }
    for len in [0usize, 1, 8, 17, 64, 97] {
        for seed in 0..32usize {
            for size in 0..=12 {
                let mut narrow = Vec::new();
                let mut wide = Vec::new();
                for i in 0..size {
                    let nb = [1usize, 2, 3, 4, 7, 8, 15, 16, 17, 31, 32, 55, 56][(seed + i) % 13];
                    let offset = [
                        0,
                        (seed * 7 + i * 11) as u32 % (len as u32 + 1),
                        len.saturating_sub(nb) as u32,
                        len as u32,
                        u32::MAX,
                    ][(seed + i) % 5];
                    if (seed + i) % 2 == 0 {
                        narrow.push(NarrowRecord {
                            offset,
                            width_class: [0, 1, 2, 3, 4, 8, 9][(seed + i) % 7],
                            payload: (seed as u64)
                                .wrapping_mul(0x123456789abcdef)
                                .wrapping_add(i as u64),
                        });
                    } else {
                        wide.push(WideRecord {
                            offset,
                            payload: pattern(nb, (seed + i) % 6),
                        });
                    }
                }
                for wide_first in [false, true] {
                    let log = make_log(&narrow, &wide, wide_first);
                    compare(&pattern(len, seed % 6), &narrow, &wide, &log);
                }
            }
        }
    }
}

#[test]
fn restoration_and_live_prefix() {
    for kind in 0..6 {
        let before = pattern(64, kind);
        let narrow = [
            NarrowRecord {
                offset: 1,
                width_class: 1,
                payload: (before[1] ^ 255) as u64,
            },
            NarrowRecord {
                offset: 1,
                width_class: 1,
                payload: before[1] as u64,
            },
        ];
        compare(&before, &narrow, &[], &make_log(&narrow, &[], false));
        for size in [1, 3, 7, 15, 17, 56] {
            let original = before[1..1 + size].to_vec();
            let wide = [
                WideRecord {
                    offset: 1,
                    payload: original.iter().map(|b| b ^ 255).collect(),
                },
                WideRecord {
                    offset: 1,
                    payload: original,
                },
            ];
            compare(&before, &[], &wide, &make_log(&[], &wide, true));
        }
        for active_narrow in [false, true] {
            for active_wide in [false, true] {
                let old_narrow = [
                    NarrowRecord {
                        offset: 0,
                        width_class: 1,
                        payload: 255,
                    },
                    NarrowRecord {
                        offset: 1,
                        width_class: 1,
                        payload: 254,
                    },
                ];
                let old_wide = [
                    WideRecord {
                        offset: 2,
                        payload: vec![253],
                    },
                    WideRecord {
                        offset: 3,
                        payload: vec![252],
                    },
                ];
                let mut log = make_log(&old_narrow, &old_wide, true);
                log.reset();
                let mut narrow = Vec::new();
                let mut wide = Vec::new();
                if active_narrow {
                    unsafe {
                        (log.grow_push_narrow)(&mut log, 0, 0x11, 1);
                    }
                    narrow.push(NarrowRecord {
                        offset: 0,
                        width_class: 1,
                        payload: 0x11,
                    });
                }
                if active_wide {
                    unsafe {
                        (log.grow_push_wide)(&mut log, 2, &0x22, 1);
                    }
                    wide.push(WideRecord {
                        offset: 2,
                        payload: vec![0x22],
                    });
                }
                compare(&before, &narrow, &wide, &log);
            }
        }
    }
}

#[test]
fn marking_sees_each_byte() {
    for kind in 0..6 {
        let before = pattern(64, kind);
        for size in 1..=56 {
            let original = before[1..1 + size].to_vec();
            for changed in 0..=size {
                let mut payload = original.clone();
                if changed < size {
                    payload[changed] ^= 1;
                }
                let wide = [WideRecord { offset: 1, payload }];
                compare(&before, &[], &wide, &make_log(&[], &wide, true));
            }
        }
    }
}
