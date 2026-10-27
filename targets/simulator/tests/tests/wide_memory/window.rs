use super::{GUARD, check_guard, guarded, pattern};
use veryl_formal_semantics::words;
use veryl_simulator::wide_ops;

pub(super) fn effective_width(dst_width: usize, width: usize, amount: u64) -> usize {
    if amount >= dst_width as u64 {
        0
    } else {
        width.min(dst_width - amount as usize)
    }
}

pub(super) fn expected_window(
    before: &[u8],
    source: &[u8],
    dst_width: usize,
    width: usize,
    amount: u64,
) -> Vec<u8> {
    let mut expected = vec![0; before.len()];
    for position in 0..before.len() * 8 {
        if words::window_stored_bit(before, source, dst_width, amount, width, position) {
            expected[position / 8] |= 1 << (position % 8);
        }
    }
    expected
}

fn compare(dst_width: usize, width: usize, amount: u64, src_kind: usize, dst_kind: usize) {
    let dst_nb = dst_width.div_ceil(64) * 8;
    let effective = effective_width(dst_width, width, amount);
    let src_nb = effective.div_ceil(64) * 8;
    let source_data = pattern(src_nb, src_kind);
    let before_data = pattern(dst_nb, dst_kind);
    for alias in [false, true] {
        let original = if alias { &before_data } else { &source_data };
        let expected = expected_window(&before_data, original, dst_width, width, amount);
        for offset in 0..8 {
            let s = 8 + offset;
            let d = 8 + (7 - offset);
            let mut source = vec![GUARD; s + src_nb];
            source[s..].copy_from_slice(&source_data);
            let source_before = source.clone();
            let mut dst = guarded(dst_nb, 7 - offset);
            dst[d..d + dst_nb].copy_from_slice(&before_data);
            unsafe {
                let output = if dst_nb == 0 {
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
                let log_offset = if offset % 2 == 0 { 0 } else { u32::MAX as u64 };
                wide_ops::wide_window_store(
                    std::ptr::null_mut(),
                    output,
                    input,
                    amount,
                    width as u64 | ((dst_width as u64) << 32),
                    (1u64 << 32) | log_offset,
                );
            }
            assert_eq!(
                &dst[d..d + dst_nb],
                expected,
                "simulator window store semantics mismatch"
            );
            assert_eq!(
                source, source_before,
                "simulator window store semantics mismatch"
            );
            check_guard(&dst, d, dst_nb, "simulator window store semantics mismatch");
        }
    }
}

#[test]
fn null_noops() {
    for dst_width in [0u32, 1, 64, 130, 192, u32::MAX] {
        for amount in [0u64, dst_width as u64, u64::MAX] {
            unsafe {
                wide_ops::wide_window_store(
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null(),
                    amount,
                    (dst_width as u64) << 32,
                    1u64 << 32,
                );
            }
        }
        for width in [1u32, 64, u32::MAX] {
            unsafe {
                wide_ops::wide_window_store(
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null(),
                    dst_width as u64,
                    width as u64 | ((dst_width as u64) << 32),
                    1u64 << 32,
                );
            }
        }
    }
}

#[test]
fn small_disjoint_and_alias() {
    for dst_width in 0..=8 {
        for width in 0..=8 {
            for amount in (0..=dst_width as u64 + 1).chain([u32::MAX as u64, 1u64 << 32, u64::MAX])
            {
                for src_kind in 0..6 {
                    for dst_kind in 0..6 {
                        compare(dst_width, width, amount, src_kind, dst_kind);
                    }
                }
            }
        }
    }
}

#[test]
fn word_boundaries_clipping_and_heap() {
    for dst_width in [
        0usize, 1, 63, 64, 65, 127, 128, 129, 191, 192, 193, 255, 256, 257, 447, 448, 449, 511,
        512, 513, 1023, 1024, 1025, 1088,
    ] {
        for width in [
            0usize,
            1,
            2,
            63,
            64,
            65,
            127,
            128,
            129,
            191,
            192,
            193,
            255,
            256,
            257,
            447,
            448,
            449,
            511,
            512,
            513,
            1023,
            1024,
            1025,
            1088,
            u32::MAX as usize,
        ] {
            for amount in [
                0u64,
                1,
                2,
                7,
                8,
                31,
                32,
                62,
                63,
                64,
                65,
                127,
                128,
                129,
                191,
                192,
                193,
                255,
                256,
                257,
                447,
                448,
                449,
                511,
                512,
                513,
                1023,
                1024,
                1025,
                1087,
                1088,
                1089,
                u32::MAX as u64,
                1u64 << 32,
                1u64 << 63,
                u64::MAX,
            ] {
                for kind in 0..6 {
                    compare(dst_width, width, amount, kind, (kind + 3) % 6);
                }
            }
        }
    }
}

#[test]
fn shared_allocation_overlap_uses_snapshot() {
    for dst_width in [
        1usize, 63, 64, 65, 127, 128, 129, 191, 192, 193, 511, 512, 1025,
    ] {
        let dst_nb = dst_width.div_ceil(64) * 8;
        for width in [
            0usize,
            1,
            2,
            63,
            64,
            65,
            127,
            128,
            129,
            191,
            192,
            193,
            511,
            512,
            1025,
            u32::MAX as usize,
        ] {
            for amount in [
                0u64,
                1,
                63,
                64,
                65,
                127,
                128,
                191,
                192,
                511,
                512,
                1023,
                u64::MAX,
            ] {
                let effective = effective_width(dst_width, width, amount);
                let src_nb = effective.div_ceil(64) * 8;
                for delta in [-9isize, -8, -7, -1, 0, 1, 7, 8, 9] {
                    let d = 33usize;
                    let s = d.checked_add_signed(delta).unwrap();
                    for kind in 0..6 {
                        let mut storage = pattern(d + dst_nb + 9 + 16, kind);
                        let before = storage.clone();
                        let expected = expected_window(
                            &before[d..d + dst_nb],
                            &before[s..s + src_nb],
                            dst_width,
                            width,
                            amount,
                        );
                        unsafe {
                            let dst = if effective == 0 {
                                std::ptr::null_mut()
                            } else {
                                storage.as_mut_ptr().add(d)
                            };
                            let src = if effective == 0 {
                                std::ptr::null()
                            } else {
                                storage.as_ptr().add(s)
                            };
                            wide_ops::wide_window_store(
                                std::ptr::null_mut(),
                                dst,
                                src,
                                amount,
                                width as u64 | ((dst_width as u64) << 32),
                                1u64 << 32,
                            );
                        }
                        assert_eq!(
                            &storage[d..d + dst_nb],
                            expected,
                            "simulator window overlap semantics mismatch"
                        );
                        assert_eq!(
                            &storage[..d],
                            &before[..d],
                            "simulator window overlap semantics mismatch"
                        );
                        assert_eq!(
                            &storage[d + dst_nb..],
                            &before[d + dst_nb..],
                            "simulator window overlap semantics mismatch"
                        );
                    }
                }
            }
        }
    }
}
