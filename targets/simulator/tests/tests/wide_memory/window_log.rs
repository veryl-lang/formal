use super::window::{effective_width, expected_window};
use super::{GUARD, check_guard, guarded, pattern};
use veryl_formal_semantics::write_log::{WideRecord, window_records};
use veryl_simulator::wide_ops;

macro_rules! assert_records {
    ($log:ident,$wide:expr,$narrow:expr) => {{
        let expected_narrow: &[(u32, u16, u16, u64)] = $narrow;
        assert_eq!(
            $log.wide_count() as usize,
            $wide.len(),
            "simulator window log semantics mismatch"
        );
        assert_eq!(
            $log.narrow_count() as usize,
            expected_narrow.len(),
            "simulator window log semantics mismatch"
        );
        assert!(
            $log.wide_capacity() >= $log.wide_count() as usize,
            "simulator window log semantics mismatch"
        );
        let actual: Vec<WideRecord> = $log
            .wide_entries_slice()
            .iter()
            .take($log.wide_count() as usize)
            .map(|entry| {
                assert!(
                    entry.native_bytes <= 56,
                    "simulator window log semantics mismatch"
                );
                WideRecord {
                    offset: entry.offset,
                    payload: entry.payload[..entry.native_bytes as usize].to_vec(),
                }
            })
            .collect();
        assert_eq!(actual, $wide, "simulator window log semantics mismatch");
        let narrow: Vec<_> = $log
            .narrow_entries_slice()
            .iter()
            .take($log.narrow_count() as usize)
            .map(|entry| {
                (
                    entry.offset,
                    entry.mask_xz,
                    entry.width_class,
                    entry.payload,
                )
            })
            .collect();
        assert_eq!(
            narrow, expected_narrow,
            "simulator window log semantics mismatch"
        );
    }};
}

macro_rules! seed_wide {
    ($log:ident) => {{
        let mut dst = [0x72u8; 8];
        let src = [0x35u8; 8];
        unsafe {
            wide_ops::wide_window_store(
                &mut $log,
                dst.as_mut_ptr(),
                src.as_ptr(),
                0,
                64 | (64u64 << 32),
                16,
            );
        }
        assert_eq!(dst, [0x72; 8], "simulator window log semantics mismatch");
        WideRecord {
            offset: 16,
            payload: vec![0x35; 8],
        }
    }};
}

#[test]
fn records_and_modes() {
    for (dst_width, width, amount) in [
        (1usize, 1usize, 0u64),
        (65, 2, 63),
        (192, 129, 63),
        (130, u32::MAX as usize, 129),
        (192, 64, 64),
        (192, 0, 0),
        (0, 129, 0),
        (192, 129, 192),
        (192, u32::MAX as usize, u64::MAX),
        (448, 448, 0),
        (512, 512, 0),
        (449, u32::MAX as usize, 1),
        (576, 512, 64),
        (1024, 1024, 0),
        (1088, 1088, 0),
        (1025, u32::MAX as usize, 1),
        (1088, 1024, 63),
    ] {
        let nb = dst_width.div_ceil(64) * 8;
        let effective = effective_width(dst_width, width, amount);
        let src_nb = effective.div_ceil(64) * 8;
        for kind in 0..6 {
            let before = pattern(nb, (kind + 3) % 6);
            let source_data = pattern(src_nb, kind);
            for alias in [false, true] {
                let original = if alias { &before } else { &source_data };
                let expected = expected_window(&before, original, dst_width, width, amount);
                for base in [0, 1, 7, 8, 65535, u32::MAX - nb as u32] {
                    let added =
                        window_records(&before, original, dst_width, amount, width, base).unwrap();
                    for direct in [false, true] {
                        for offset in 0..8 {
                            let s = 8 + offset;
                            let d = 8 + (7 - offset);
                            let mut source = vec![GUARD; s + src_nb];
                            source[s..].copy_from_slice(&source_data);
                            let source_before = source.clone();
                            let mut dst = guarded(nb, 7 - offset);
                            dst[d..d + nb].copy_from_slice(&before);
                            let mut log = Default::default();
                            let mut records = vec![seed_wide!(log)];
                            records.extend(added.iter().cloned());
                            unsafe {
                                let output = if nb == 0 {
                                    std::ptr::null_mut()
                                } else {
                                    dst.as_mut_ptr().add(d)
                                };
                                let input = if effective == 0 {
                                    std::ptr::null()
                                } else if alias {
                                    output as *const u8
                                } else {
                                    source.as_ptr().add(s)
                                };
                                wide_ops::wide_window_store(
                                    &mut log,
                                    output,
                                    input,
                                    amount,
                                    width as u64 | ((dst_width as u64) << 32),
                                    base as u64 | ((direct as u64) << 32),
                                );
                            }
                            assert_eq!(
                                &dst[d..d + nb],
                                if direct { &expected } else { &before },
                                "simulator window log semantics mismatch"
                            );
                            assert_eq!(
                                source, source_before,
                                "simulator window log semantics mismatch"
                            );
                            check_guard(&dst, d, nb, "simulator window log semantics mismatch");
                            assert_records!(log, records, &[]);
                            source.fill(0xa7);
                            dst[d..d + nb].fill(0xe3);
                            assert_records!(log, records, &[]);
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn growth_and_append_preserve_history() {
    for direct in [false, true] {
        for alias in [false, true] {
            for kind in 0..6 {
                let mut log = Default::default();
                let mut records = vec![seed_wide!(log)];
                let initial_capacity = log.wide_capacity();
                unsafe {
                    (log.grow_push_narrow)(&mut log, 3, 0x8877665544332211, 8);
                }
                let narrow = [(3, 0, 8, 0x8877665544332211)];
                let mut dst = guarded(136, 1);
                let d = 9;
                dst[d..d + 136].copy_from_slice(&pattern(136, kind));
                for step in 0..40 {
                    let before = dst[d..d + 136].to_vec();
                    let mut source = vec![GUARD; 129];
                    source[1..].copy_from_slice(&pattern(128, (kind + step) % 6));
                    let source_before = source.clone();
                    let original = if alias { &before[..] } else { &source[1..] };
                    let base = 64 + step as u32 * 192;
                    records
                        .extend(window_records(&before, original, 1088, 63, 1024, base).unwrap());
                    let expected = expected_window(&before, original, 1088, 1024, 63);
                    unsafe {
                        let output = dst.as_mut_ptr().add(d);
                        let input = if alias {
                            output as *const u8
                        } else {
                            source.as_ptr().add(1)
                        };
                        wide_ops::wide_window_store(
                            &mut log,
                            output,
                            input,
                            63,
                            1024 | (1088u64 << 32),
                            base as u64 | ((direct as u64) << 32),
                        );
                    }
                    assert_eq!(
                        &dst[d..d + 136],
                        if direct { &expected } else { &before },
                        "simulator window log semantics mismatch"
                    );
                    assert_eq!(
                        source, source_before,
                        "simulator window log semantics mismatch"
                    );
                    check_guard(&dst, d, 136, "simulator window log semantics mismatch");
                    assert_records!(log, records, &narrow);
                    source.fill(0xa7);
                    dst[d..d + 136].copy_from_slice(&pattern(136, (step + 1) % 6));
                    assert_records!(log, records, &narrow);
                }
                assert!(
                    log.wide_capacity() >= 121,
                    "simulator window log semantics mismatch"
                );
                assert!(
                    log.wide_capacity() > initial_capacity,
                    "simulator window log semantics mismatch"
                );
            }
        }
    }
}

#[test]
fn prefilled_noops_preserve_entries() {
    let mut log = Default::default();
    let records = vec![seed_wide!(log)];
    unsafe {
        (log.grow_push_narrow)(&mut log, 3, 0x8877665544332211, 8);
    }
    let narrow = [(3, 0, 8, 0x8877665544332211)];
    for dst_width in [0u32, 1, 130, 192, u32::MAX] {
        for flags in [0u64, 1u64 << 32, u64::MAX] {
            for (width, amount) in [
                (0u32, 0u64),
                (0, u64::MAX),
                (1, dst_width as u64),
                (u32::MAX, u64::MAX),
            ] {
                unsafe {
                    wide_ops::wide_window_store(
                        &mut log,
                        std::ptr::null_mut(),
                        std::ptr::null(),
                        amount,
                        width as u64 | ((dst_width as u64) << 32),
                        flags,
                    );
                }
                assert_records!(log, records, &narrow);
            }
        }
    }
}
