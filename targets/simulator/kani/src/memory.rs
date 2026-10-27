//! Binary unsafe helper contracts with valid whole-word extents, disjoint storage or
//! exact alias; partial overlap is excluded. Copy/resize use up to four words and
//! bounded misaligned offsets; mask admits any u16 width, with zero as a no-op sentinel.
//! Separate source allocations end at their required extent. Logical-width masking
//! is separate from resize. These are not four-state or source-lowering proofs.

use veryl_simulator::wide_ops;

#[kani::proof]
#[kani::unwind(5)]
fn wide_mask_matches_reference() {
    use veryl_formal_semantics::words;
    let mut destination: [u8; 48] = kani::any();
    let before = destination;
    let count = usize::from(kani::any::<u8>() % 5);
    let width = usize::from(kani::any::<u16>());
    let offset = 8 + usize::from(kani::any::<u8>() & 7);
    let observed = usize::from(kani::any::<u8>() % 48);
    let bit = usize::from(kani::any::<u8>() & 7);
    unsafe {
        let dst = if count == 0 {
            std::ptr::null_mut()
        } else {
            destination.as_mut_ptr().add(offset)
        };
        wide_ops::wide_apply_mask(
            dst,
            std::ptr::null(),
            wide_ops::pack_nb_width(count * 8, width),
        );
    }
    let expected = if observed >= offset && observed - offset < count * 8 {
        words::masked_storage_bit(&before[offset..], width, (observed - offset) * 8 + bit)
    } else {
        words::bit(&before, observed * 8 + bit)
    };
    assert!(
        words::bit(&destination, observed * 8 + bit) == expected,
        "simulator mask semantics mismatch"
    );
    kani::cover!(count == 0, "mask empty storage is reachable");
    kani::cover!(
        count == 4 && width == 0,
        "mask zero width sentinel is reachable"
    );
    kani::cover!(count == 4 && width == 63, "mask width 63 is reachable");
    kani::cover!(count == 4 && width == 64, "mask word boundary is reachable");
    kani::cover!(count == 4 && width == 65, "mask width 65 is reachable");
    kani::cover!(
        count == 4 && width == 256,
        "mask full capacity is reachable"
    );
    kani::cover!(
        count == 1 && width == 65535,
        "mask above capacity is reachable"
    );
    kani::cover!(
        count == 4 && offset == 15,
        "mask unaligned destination is reachable"
    );
    kani::cover!(
        count == 4 && observed < offset,
        "mask prefix guard is reachable"
    );
    kani::cover!(
        count == 4 && observed >= offset + 32,
        "mask suffix guard is reachable"
    );
}

#[kani::proof]
#[kani::unwind(5)]
fn wide_copy_matches_reference() {
    let source_data: [u8; 40] = kani::any();
    let mut destination: [u8; 48] = kani::any();
    let before = destination;
    let words = usize::from(kani::any::<u8>() % 5);
    let src_offset = usize::from(kani::any::<u8>() & 7);
    let dst_offset = 8 + usize::from(kani::any::<u8>() & 7);
    let observed = usize::from(kani::any::<u8>() % 48);
    let inplace: bool = kani::any();
    let nb = words * 8;
    let source = source_data[..src_offset + nb].to_vec();
    unsafe {
        let dst = if nb == 0 {
            std::ptr::null_mut()
        } else {
            destination.as_mut_ptr().add(dst_offset)
        };
        let src = if nb == 0 {
            std::ptr::null()
        } else if inplace {
            dst as *const u8
        } else {
            source.as_ptr().add(src_offset)
        };
        wide_ops::wide_copy(dst, src, nb as u32);
    }
    let expected = if !inplace && observed >= dst_offset && observed - dst_offset < nb {
        source_data[src_offset + observed - dst_offset]
    } else {
        before[observed]
    };
    assert!(
        destination[observed] == expected,
        "simulator copy semantics mismatch"
    );
    let source_observed = usize::from(kani::any::<u8>() % 40);
    if source_observed < source.len() {
        assert!(
            source[source_observed] == source_data[source_observed],
            "simulator copy semantics mismatch"
        );
    }
    kani::cover!(words == 0, "copy empty buffer is reachable");
    kani::cover!(words == 4 && !inplace, "copy four words is reachable");
    kani::cover!(words == 4 && inplace, "copy exact alias is reachable");
    kani::cover!(
        words > 0 && src_offset == 7,
        "copy unaligned source is reachable"
    );
    kani::cover!(
        words > 0 && dst_offset == 15,
        "copy unaligned destination is reachable"
    );
    kani::cover!(
        words == 4 && observed < dst_offset,
        "copy prefix guard is reachable"
    );
    kani::cover!(
        words == 4 && observed >= dst_offset + nb,
        "copy suffix guard is reachable"
    );
}

#[kani::proof]
#[kani::unwind(5)]
fn wide_resize_matches_reference() {
    use veryl_formal_semantics::words;
    let data: [u8; 40] = kani::any();
    let mut destination: [u8; 48] = kani::any();
    let before = destination;
    let src_width = usize::from(kani::any::<u16>() % 257);
    let src_nb = src_width.div_ceil(64) * 8;
    let dst_words = usize::from(kani::any::<u8>() % 5);
    let src_offset = usize::from(kani::any::<u8>() & 7);
    let dst_offset = 8 + usize::from(kani::any::<u8>() & 7);
    let observed = usize::from(kani::any::<u8>() % 48);
    let bit = usize::from(kani::any::<u8>() & 7);
    let inplace: bool = kani::any();
    let signed: bool = kani::any();
    let source = data[..src_offset + src_nb].to_vec();
    let info = wide_ops::pack_nb_width(src_nb, src_width) as u64 | ((signed as u64) << 32);
    unsafe {
        let backing = destination.as_mut_ptr().add(dst_offset);
        let dst = if dst_words == 0 && !inplace {
            std::ptr::null_mut()
        } else {
            backing
        };
        let src = if src_width == 0 {
            std::ptr::null()
        } else if inplace {
            backing as *const u8
        } else {
            source.as_ptr().add(src_offset)
        };
        wide_ops::wide_resize(dst, src, info, (dst_words * 8) as u32);
    }
    let original = if inplace {
        &before[dst_offset..]
    } else {
        &data[src_offset..src_offset + src_nb]
    };
    let expected = if observed >= dst_offset && observed - dst_offset < dst_words * 8 {
        words::resized_bit(
            original,
            src_width,
            signed,
            (observed - dst_offset) * 8 + bit,
        )
    } else {
        words::bit(&before, observed * 8 + bit)
    };
    assert!(
        words::bit(&destination, observed * 8 + bit) == expected,
        "simulator resize semantics mismatch"
    );
    let source_observed = usize::from(kani::any::<u8>() % 40);
    if source_observed < source.len() {
        assert!(
            source[source_observed] == data[source_observed],
            "simulator resize semantics mismatch"
        );
    }
    kani::cover!(
        src_width == 0 && dst_words == 4,
        "resize empty source is reachable"
    );
    kani::cover!(
        src_width == 256 && dst_words == 0 && !inplace,
        "resize empty destination is reachable"
    );
    kani::cover!(
        src_width == 1 && dst_words == 4 && signed && words::bit(original, 0),
        "resize signed one-bit growth is reachable"
    );
    kani::cover!(src_width == 63, "resize width 63 is reachable");
    kani::cover!(src_width == 64, "resize width 64 is reachable");
    kani::cover!(src_width == 65, "resize width 65 is reachable");
    kani::cover!(
        src_width == 129 && dst_words == 4,
        "resize above native width is reachable"
    );
    kani::cover!(
        src_width == 256 && dst_words == 1,
        "resize shrinking is reachable"
    );
    kani::cover!(
        src_width == 65 && dst_words == 4 && inplace && signed,
        "resize exact alias is reachable"
    );
    kani::cover!(
        src_width > 0 && src_offset == 7 && !inplace,
        "resize unaligned source is reachable"
    );
    kani::cover!(
        dst_words > 0 && dst_offset == 15,
        "resize unaligned destination is reachable"
    );
    kani::cover!(
        dst_words == 4 && observed < dst_offset,
        "resize prefix guard is reachable"
    );
    kani::cover!(
        dst_words == 4 && observed >= dst_offset + 32,
        "resize suffix guard is reachable"
    );
}
