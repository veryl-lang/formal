//! Finite binary helper comparisons with guarded, misaligned allocations and
//! explicit pre-call alias snapshots. Broader lengths, patterns, window/log
//! histories, and heap cases supplement the narrower Kani domains; they do not
//! turn sampled payloads or arbitrary partial overlap into symbolic guarantees.

use veryl_formal_semantics::words;
use veryl_simulator::wide_ops;

const GUARD: u8 = 0xa5;

#[path = "wide_memory/commit.rs"]
mod commit;
#[path = "wide_memory/window.rs"]
mod window;
#[path = "wide_memory/window_log.rs"]
mod window_log;

#[test]
fn bitwise_and_buffers_and_alias() {
    bitwise_comparisons(words::StorageBitOp::And);
}
#[test]
fn bitwise_or_buffers_and_alias() {
    bitwise_comparisons(words::StorageBitOp::Or);
}
#[test]
fn bitwise_xor_buffers_and_alias() {
    bitwise_comparisons(words::StorageBitOp::Xor);
}
#[test]
fn bitwise_xnor_buffers_and_alias() {
    bitwise_comparisons(words::StorageBitOp::Xnor);
}
#[test]
fn bitwise_and_not_buffers_and_alias() {
    bitwise_comparisons(words::StorageBitOp::AndNot);
}
#[test]
fn bitwise_not_buffers_and_alias() {
    bitwise_comparisons(words::StorageBitOp::Not);
}

fn bitwise_comparisons(op: words::StorageBitOp) {
    use words::StorageBitOp;
    for count in 0..=8 {
        let nb = count * 8;
        for a_kind in 0..6 {
            let left = pattern(nb, a_kind);
            for b_kind in 0..if op == StorageBitOp::Not { 1 } else { 6 } {
                let right = pattern(nb, b_kind);
                for alias in 0..if op == StorageBitOp::Not { 2 } else { 4 } {
                    let alias_a = alias == 1 || alias == 3;
                    let alias_b = alias == 2 || alias == 3;
                    let initial = if alias_a {
                        left.clone()
                    } else if alias_b {
                        right.clone()
                    } else {
                        vec![GUARD; nb]
                    };
                    let x = if alias_a { &initial } else { &left };
                    let y = if alias_b { &initial } else { &right };
                    let mut expected = vec![0; nb];
                    for position in 0..nb * 8 {
                        if words::storage_bit(op, words::bit(x, position), words::bit(y, position))
                        {
                            expected[position / 8] |= 1 << (position % 8);
                        }
                    }
                    for offset in 0..8 {
                        let s = 8 + offset;
                        let t = 8 + (7 - offset);
                        let dst_offset = (offset + 3) % 8;
                        let d = 8 + dst_offset;
                        let mut a = vec![GUARD; s + nb];
                        let mut b = vec![GUARD; t + nb];
                        a[s..].copy_from_slice(&left);
                        b[t..].copy_from_slice(&right);
                        let a_before = a.clone();
                        let b_before = b.clone();
                        let mut dst = guarded(nb, dst_offset);
                        dst[d..d + nb].copy_from_slice(&initial);
                        unsafe {
                            let output = if nb == 0 {
                                std::ptr::null_mut()
                            } else {
                                dst.as_mut_ptr().add(d)
                            };
                            let x = if nb == 0 {
                                std::ptr::null()
                            } else if alias_a {
                                output as *const u8
                            } else {
                                a.as_ptr().add(s)
                            };
                            let y = if nb == 0 {
                                std::ptr::null()
                            } else if alias_b {
                                output as *const u8
                            } else {
                                b.as_ptr().add(t)
                            };
                            match op {
                                StorageBitOp::And => wide_ops::wide_band(output, x, y, nb as u32),
                                StorageBitOp::Or => wide_ops::wide_bor(output, x, y, nb as u32),
                                StorageBitOp::Xor => wide_ops::wide_bxor(output, x, y, nb as u32),
                                StorageBitOp::Xnor => {
                                    wide_ops::wide_bxor_not(output, x, y, nb as u32)
                                }
                                StorageBitOp::AndNot => {
                                    wide_ops::wide_band_not(output, x, y, nb as u32)
                                }
                                StorageBitOp::Not => wide_ops::wide_bnot(output, x, nb as u32),
                            }
                        }
                        assert_eq!(
                            &dst[d..d + nb],
                            expected,
                            "simulator bitwise semantics mismatch"
                        );
                        assert_eq!(a, a_before, "simulator bitwise semantics mismatch");
                        assert_eq!(b, b_before, "simulator bitwise semantics mismatch");
                        check_guard(&dst, d, nb, "simulator bitwise semantics mismatch");
                    }
                }
            }
        }
    }
}

#[test]
fn arithmetic_right_shift_storage_and_alias() {
    let layouts = (1..=8).map(|count| (count * 8, count * 64)).chain(
        [1usize, 63, 65, 127, 129, 130, 191, 193, 255, 257, 511]
            .map(|width| (width.div_ceil(64).max(3) * 8, width)),
    );
    for (nb, width) in layouts {
        let mut amounts: Vec<u64> = (0..=(nb * 8) as u64 + 1).collect();
        amounts.extend([u32::MAX as u64, 1u64 << 32, 1u64 << 63, u64::MAX]);
        for kind in 0..6 {
            let mut bytes = pattern(nb, kind);
            for position in width..nb * 8 {
                bytes[position / 8] &= !(1 << (position % 8));
            }
            for &amount in &amounts {
                let expected = arithmetic_shift_reference(&bytes, width, amount);
                for offset in 0..8 {
                    let s = 8 + offset;
                    let d = 8 + (7 - offset);
                    let mut source = vec![GUARD; s + nb];
                    source[s..].copy_from_slice(&bytes);
                    let before = source.clone();
                    let mut dst = guarded(nb, 7 - offset);
                    unsafe {
                        wide_ops::wide_ashr(
                            dst.as_mut_ptr().add(d),
                            source.as_ptr().add(s),
                            amount,
                            wide_ops::pack_nb_width(nb, width),
                        );
                    }
                    assert_eq!(
                        &dst[d..d + nb],
                        expected,
                        "simulator arithmetic shift semantics mismatch"
                    );
                    assert_eq!(
                        source, before,
                        "simulator arithmetic shift semantics mismatch"
                    );
                    check_guard(&dst, d, nb, "simulator arithmetic shift semantics mismatch");
                    let mut inplace = guarded(nb, offset);
                    inplace[s..s + nb].copy_from_slice(&bytes);
                    unsafe {
                        let ptr = inplace.as_mut_ptr().add(s);
                        wide_ops::wide_ashr(ptr, ptr, amount, wide_ops::pack_nb_width(nb, width));
                    }
                    assert_eq!(
                        &inplace[s..s + nb],
                        expected,
                        "simulator arithmetic shift semantics mismatch"
                    );
                    check_guard(
                        &inplace,
                        s,
                        nb,
                        "simulator arithmetic shift semantics mismatch",
                    );
                }
            }
        }
    }
}

fn arithmetic_shift_reference(bytes: &[u8], width: usize, amount: u64) -> Vec<u8> {
    let mut result = vec![0; bytes.len()];
    for position in 0..bytes.len() * 8 {
        if words::arithmetic_shifted_bit(bytes, width, amount, position) {
            result[position / 8] |= 1 << (position % 8);
        }
    }
    result
}

#[test]
fn arithmetic_right_shift_preparation_and_mask() {
    for width in [
        1usize, 2, 7, 8, 31, 32, 63, 64, 65, 127, 128, 129, 130, 191, 192, 193, 255, 256, 257, 511,
        512,
    ] {
        let nb = width.div_ceil(64) * 8;
        let capacity = nb * 8;
        let mut amounts: Vec<u64> = (0..=capacity as u64 + 1).collect();
        amounts.extend([u32::MAX as u64, 1u64 << 32, 1u64 << 63, u64::MAX]);
        for kind in 0..6 {
            let bytes = pattern(nb, kind);
            for &amount in &amounts {
                let expected = arithmetic_shift_reference(&bytes, width, amount);
                for offset in 0..8 {
                    let s = 8 + offset;
                    let d = 8 + (7 - offset);
                    let mut source = vec![GUARD; s + nb];
                    source[s..].copy_from_slice(&bytes);
                    let before = source.clone();
                    let mut prepared = guarded(nb, offset);
                    let mut dst = guarded(nb, 7 - offset);
                    let info = wide_ops::pack_nb_width(nb, width) as u64 | (1u64 << 32);
                    unsafe {
                        wide_ops::wide_resize(
                            prepared.as_mut_ptr().add(s),
                            source.as_ptr().add(s),
                            info,
                            nb as u32,
                        );
                        wide_ops::wide_ashr(
                            dst.as_mut_ptr().add(d),
                            prepared.as_ptr().add(s),
                            amount,
                            wide_ops::pack_nb_width(nb, capacity),
                        );
                        wide_ops::wide_apply_mask(
                            dst.as_mut_ptr().add(d),
                            std::ptr::null(),
                            wide_ops::pack_nb_width(nb, width),
                        );
                    }
                    assert_eq!(
                        &dst[d..d + nb],
                        expected,
                        "simulator prepared shift semantics mismatch"
                    );
                    assert_eq!(
                        source, before,
                        "simulator prepared shift semantics mismatch"
                    );
                    check_guard(
                        &prepared,
                        s,
                        nb,
                        "simulator prepared shift semantics mismatch",
                    );
                    check_guard(&dst, d, nb, "simulator prepared shift semantics mismatch");
                    let mut inplace = guarded(nb, offset);
                    inplace[s..s + nb].copy_from_slice(&bytes);
                    unsafe {
                        let ptr = inplace.as_mut_ptr().add(s);
                        wide_ops::wide_resize(ptr, ptr, info, nb as u32);
                        wide_ops::wide_ashr(
                            ptr,
                            ptr,
                            amount,
                            wide_ops::pack_nb_width(nb, capacity),
                        );
                        wide_ops::wide_apply_mask(
                            ptr,
                            std::ptr::null(),
                            wide_ops::pack_nb_width(nb, width),
                        );
                    }
                    assert_eq!(
                        &inplace[s..s + nb],
                        expected,
                        "simulator prepared shift semantics mismatch"
                    );
                    check_guard(
                        &inplace,
                        s,
                        nb,
                        "simulator prepared shift semantics mismatch",
                    );
                }
            }
        }
    }
}

#[test]
fn arithmetic_right_shift_empty_metadata() {
    for count in 0..=8 {
        let nb = count * 8;
        for kind in 0..6 {
            for offset in 0..8 {
                let start = 8 + offset;
                let mut dst = guarded(nb, offset);
                dst[start..start + nb].copy_from_slice(&pattern(nb, kind));
                let before = dst.clone();
                unsafe {
                    wide_ops::wide_ashr(
                        dst.as_mut_ptr().add(start),
                        std::ptr::null(),
                        u64::MAX,
                        wide_ops::pack_nb_width(nb, 0),
                    );
                }
                assert_eq!(dst, before, "simulator empty shift semantics mismatch");
            }
        }
    }
    for width in [0, 1, 63, 65535] {
        unsafe {
            wide_ops::wide_ashr(
                std::ptr::null_mut(),
                std::ptr::null(),
                u64::MAX,
                wide_ops::pack_nb_width(0, width),
            );
        }
    }
}

#[test]
fn logical_left_shift_boundaries_and_alias() {
    logical_shift::<true>();
}

#[test]
fn logical_right_shift_boundaries_and_alias() {
    logical_shift::<false>();
}

fn logical_shift<const LEFT: bool>() {
    for count in 0..=8 {
        let nb = count * 8;
        let capacity = nb * 8;
        let mut amounts: Vec<u64> = (0..=capacity as u64 + 1).collect();
        amounts.extend([
            capacity as u64 + 63,
            capacity as u64 + 64,
            capacity as u64 + 65,
            u32::MAX as u64,
            1u64 << 32,
            1u64 << 63,
            u64::MAX,
        ]);
        amounts.sort_unstable();
        amounts.dedup();
        for kind in 0..6 {
            let bytes = pattern(nb, kind);
            for &amount in &amounts {
                let mut expected = vec![0u8; nb];
                for position in 0..capacity {
                    if words::logical_shifted_bit(&bytes, amount, LEFT, position) {
                        expected[position / 8] |= 1 << (position % 8);
                    }
                }
                for offset in 0..8 {
                    let s = 8 + offset;
                    let d = 8 + (7 - offset);
                    let mut source = vec![GUARD; s + nb];
                    source[s..].copy_from_slice(&bytes);
                    let source_before = source.clone();
                    let mut dst = guarded(nb, 7 - offset);
                    unsafe {
                        let output = if nb == 0 {
                            std::ptr::null_mut()
                        } else {
                            dst.as_mut_ptr().add(d)
                        };
                        let input = if nb == 0 {
                            std::ptr::null()
                        } else {
                            source.as_ptr().add(s)
                        };
                        if LEFT {
                            wide_ops::wide_shl(output, input, amount, nb as u32);
                        } else {
                            wide_ops::wide_lshr(output, input, amount, nb as u32);
                        }
                    }
                    assert_eq!(
                        &dst[d..d + nb],
                        expected,
                        "simulator logical shift semantics mismatch"
                    );
                    assert_eq!(
                        source, source_before,
                        "simulator logical shift semantics mismatch"
                    );
                    check_guard(&dst, d, nb, "simulator logical shift semantics mismatch");
                    let mut inplace = guarded(nb, offset);
                    inplace[s..s + nb].copy_from_slice(&bytes);
                    unsafe {
                        let ptr = if nb == 0 {
                            std::ptr::null_mut()
                        } else {
                            inplace.as_mut_ptr().add(s)
                        };
                        if LEFT {
                            wide_ops::wide_shl(ptr, ptr, amount, nb as u32);
                        } else {
                            wide_ops::wide_lshr(ptr, ptr, amount, nb as u32);
                        }
                    }
                    assert_eq!(
                        &inplace[s..s + nb],
                        expected,
                        "simulator logical shift semantics mismatch"
                    );
                    check_guard(
                        &inplace,
                        s,
                        nb,
                        "simulator logical shift semantics mismatch",
                    );
                }
            }
        }
    }
}

#[test]
fn mask_width_sentinel_padding_and_guards() {
    for count in 0..=8 {
        let nb = count * 8;
        for width in [
            0, 1, 7, 8, 63, 64, 65, 127, 128, 129, 255, 256, 257, 511, 512, 513, 65535,
        ] {
            for kind in 0..6 {
                let bytes = pattern(nb, kind);
                for offset in 0..8 {
                    let mut dst = guarded(nb, offset);
                    let start = 8 + offset;
                    dst[start..start + nb].copy_from_slice(&bytes);
                    unsafe {
                        wide_ops::wide_apply_mask(
                            dst.as_mut_ptr().add(start),
                            std::ptr::null(),
                            wide_ops::pack_nb_width(nb, width),
                        );
                    }
                    for position in 0..nb * 8 {
                        assert_eq!(
                            words::bit(&dst[start..start + nb], position),
                            words::masked_storage_bit(&bytes, width, position),
                            "simulator mask semantics mismatch"
                        );
                    }
                    check_guard(&dst, start, nb, "simulator mask semantics mismatch");
                }
            }
        }
    }
    unsafe {
        wide_ops::wide_apply_mask(
            std::ptr::null_mut(),
            std::ptr::null(),
            wide_ops::pack_nb_width(0, 65),
        );
    }
}
fn pattern(n: usize, kind: usize) -> Vec<u8> {
    (0..n)
        .map(|i| match kind {
            0 => 0,
            1 => 0xff,
            2 => 0x55,
            3 => 0xaa,
            4 => (i * 73 + 19) as u8,
            _ => {
                if i % 8 == 7 {
                    0x80
                } else {
                    1
                }
            }
        })
        .collect()
}
fn guarded(n: usize, offset: usize) -> Vec<u8> {
    vec![GUARD; 8 + offset + n + 8]
}
fn check_guard(buffer: &[u8], start: usize, n: usize, message: &str) {
    assert!(
        buffer[..start]
            .iter()
            .chain(&buffer[start + n..])
            .all(|x| *x == GUARD),
        "{message}: guard corruption"
    );
}
#[test]
fn copy_disjoint_unaligned_and_exact_alias() {
    for count in 0..=8 {
        let nb = count * 8;
        for kind in 0..6 {
            let source = pattern(nb, kind);
            for src_offset in 0..8 {
                let mut src = guarded(nb, src_offset);
                let s = 8 + src_offset;
                src[s..s + nb].copy_from_slice(&source);
                let source_before = src.clone();
                for dst_offset in 0..8 {
                    let mut dst = guarded(nb, dst_offset);
                    let d = 8 + dst_offset;
                    unsafe {
                        wide_ops::wide_copy(
                            dst.as_mut_ptr().add(d),
                            src.as_ptr().add(s),
                            nb as u32,
                        );
                    }
                    assert_eq!(&dst[d..d + nb], source, "simulator copy semantics mismatch");
                    check_guard(&dst, d, nb, "simulator copy semantics mismatch");
                    assert_eq!(src, source_before, "simulator copy semantics mismatch");
                }
                let before = src.clone();
                unsafe {
                    let ptr = src.as_mut_ptr().add(s);
                    wide_ops::wide_copy(ptr, ptr, nb as u32);
                }
                assert_eq!(src, before, "simulator copy semantics mismatch");
            }
        }
    }
    unsafe {
        wide_ops::wide_copy(std::ptr::null_mut(), std::ptr::null(), 0);
    }
}
#[test]
fn resize_minimal_source_dirty_padding_and_guards() {
    for width in [
        0usize, 1, 2, 7, 8, 31, 32, 63, 64, 65, 127, 128, 129, 191, 192, 193, 255, 256, 257, 319,
        320,
    ] {
        let src_nb = width.div_ceil(64) * 8;
        for dst_words in 0..=6 {
            let dst_nb = dst_words * 8;
            for signed in [false, true] {
                for kind in 0..6 {
                    let bytes = pattern(src_nb, kind);
                    let expected = words::resize(&bytes, width, signed, dst_nb);
                    for src_offset in 0..8 {
                        let mut src = vec![0x39; src_offset + src_nb];
                        src[src_offset..].copy_from_slice(&bytes);
                        let source_before = src.clone();
                        for dst_offset in 0..8 {
                            let mut dst = guarded(dst_nb, dst_offset);
                            let d = 8 + dst_offset;
                            let info = wide_ops::pack_nb_width(src_nb, width) as u64
                                | ((signed as u64) << 32);
                            unsafe {
                                let ptr = if width == 0 {
                                    std::ptr::null()
                                } else {
                                    src.as_ptr().add(src_offset)
                                };
                                wide_ops::wide_resize(
                                    dst.as_mut_ptr().add(d),
                                    ptr,
                                    info,
                                    dst_nb as u32,
                                );
                            }
                            assert_eq!(
                                &dst[d..d + dst_nb],
                                expected,
                                "simulator resize semantics mismatch: width={width}, dst={dst_nb}, signed={signed}, kind={kind}"
                            );
                            check_guard(&dst, d, dst_nb, "simulator resize semantics mismatch");
                            assert_eq!(src, source_before, "simulator resize semantics mismatch");
                        }
                    }
                }
            }
        }
    }
}
#[test]
fn resize_exact_alias_grows_and_shrinks() {
    for width in [
        0usize, 1, 63, 64, 65, 127, 128, 129, 191, 192, 255, 256, 257, 320,
    ] {
        let src_nb = width.div_ceil(64) * 8;
        for dst_words in 0..=6 {
            let dst_nb = dst_words * 8;
            for signed in [false, true] {
                for kind in 0..6 {
                    let bytes = pattern(src_nb, kind);
                    let expected = words::resize(&bytes, width, signed, dst_nb);
                    for offset in 0..8 {
                        let n = src_nb.max(dst_nb);
                        let mut buffer = guarded(n, offset);
                        let start = 8 + offset;
                        buffer[start..start + src_nb].copy_from_slice(&bytes);
                        let before = buffer.clone();
                        let info =
                            wide_ops::pack_nb_width(src_nb, width) as u64 | ((signed as u64) << 32);
                        unsafe {
                            let ptr = buffer.as_mut_ptr().add(start);
                            wide_ops::wide_resize(ptr, ptr, info, dst_nb as u32);
                        }
                        assert_eq!(
                            &buffer[start..start + dst_nb],
                            expected,
                            "simulator inplace resize semantics mismatch"
                        );
                        assert_eq!(
                            &buffer[..start],
                            &before[..start],
                            "simulator inplace resize semantics mismatch"
                        );
                        assert_eq!(
                            &buffer[start + dst_nb..],
                            &before[start + dst_nb..],
                            "simulator inplace resize semantics mismatch"
                        );
                    }
                }
            }
        }
    }
}
#[test]
fn modular_add_sub_unaligned_and_alias() {
    for count in 1..=8 {
        let nb = count * 8;
        for kind in 0..6 {
            let a = pattern(nb, kind);
            let b = pattern(nb, (kind + 3) % 6);
            for subtract in [false, true] {
                let expected = words::add_sub(&a, &b, subtract);
                for (position, byte) in expected.iter().enumerate() {
                    assert_eq!(
                        *byte,
                        words::arithmetic_byte(&a, &b, position, subtract),
                        "reference arithmetic models disagree"
                    );
                }
                for offset in 0..8 {
                    for alias in 0..3 {
                        let mut left = vec![0; offset + nb];
                        left[offset..].copy_from_slice(&a);
                        let mut right = vec![0; 7 - offset + nb];
                        right[7 - offset..].copy_from_slice(&b);
                        let mut dst = guarded(nb, offset);
                        let start = 8 + offset;
                        if alias == 1 {
                            dst[start..start + nb].copy_from_slice(&a);
                        }
                        if alias == 2 {
                            dst[start..start + nb].copy_from_slice(&b);
                        }
                        unsafe {
                            let d = dst.as_mut_ptr().add(start);
                            let x = if alias == 1 {
                                d as *const u8
                            } else {
                                left.as_ptr().add(offset)
                            };
                            let y = if alias == 2 {
                                d as *const u8
                            } else {
                                right.as_ptr().add(7 - offset)
                            };
                            if subtract {
                                wide_ops::wide_sub(d, x, y, nb as u32);
                            } else {
                                wide_ops::wide_add(d, x, y, nb as u32);
                            }
                        }
                        assert_eq!(
                            &dst[start..start + nb],
                            expected,
                            "simulator arithmetic semantics mismatch"
                        );
                        check_guard(&dst, start, nb, "simulator arithmetic semantics mismatch");
                        assert_eq!(
                            &left[offset..],
                            a,
                            "simulator arithmetic semantics mismatch"
                        );
                        assert_eq!(
                            &right[7 - offset..],
                            b,
                            "simulator arithmetic semantics mismatch"
                        );
                    }
                }
            }
        }
    }
    unsafe {
        wide_ops::wide_add(std::ptr::null_mut(), std::ptr::null(), std::ptr::null(), 0);
        wide_ops::wide_sub(std::ptr::null_mut(), std::ptr::null(), std::ptr::null(), 0);
    }
}
