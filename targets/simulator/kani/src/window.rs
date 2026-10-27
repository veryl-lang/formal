//! Direct binary writes with arbitrary payloads but fixed metadata families:
//! (destination width, amount, requested width) = (192,63,129), (192,64,64), or
//! (130,129,u32::MAX). Separate and exact-alias sources use pre-call snapshots;
//! partial overlap is excluded. No-op metadata is separately symbolic with null
//! pointers. General nonempty windows, heap storage, and logs are not proved here.

use veryl_formal_semantics::words;
use veryl_simulator::wide_ops;

#[kani::proof]
#[kani::unwind(4)]
fn wide_window_cross_words_matches_reference() {
    window::<false, 0>();
}

#[kani::proof]
#[kani::unwind(4)]
fn wide_window_cross_words_alias_matches_reference() {
    window::<true, 0>();
}

#[kani::proof]
#[kani::unwind(4)]
fn wide_window_aligned_matches_reference() {
    window::<false, 1>();
}
#[kani::proof]
#[kani::unwind(4)]
fn wide_window_aligned_alias_matches_reference() {
    window::<true, 1>();
}
#[kani::proof]
#[kani::unwind(4)]
fn wide_window_clipped_matches_reference() {
    window::<false, 2>();
}
#[kani::proof]
#[kani::unwind(4)]
fn wide_window_clipped_alias_matches_reference() {
    window::<true, 2>();
}

fn window<const ALIAS: bool, const CASE: u8>() {
    let source_data: [u8; 25] = kani::any();
    let mut destination: [u8; 26] = kani::any();
    let before = destination;
    let dst_width = if CASE == 2 { 130 } else { 192 };
    let width = if CASE == 0 {
        129
    } else if CASE == 1 {
        64
    } else {
        u32::MAX as usize
    };
    let amount: u64 = if CASE == 0 {
        63
    } else if CASE == 1 {
        64
    } else {
        129
    };
    let effective = width.min(dst_width - amount as usize);
    let src_nb = effective.div_ceil(64) * 8;
    let source = source_data[..1 + src_nb].to_vec();
    let observed = usize::from(kani::any::<u8>() % 26);
    let bit = usize::from(kani::any::<u8>() & 7);
    unsafe {
        let dst = if effective == 0 {
            std::ptr::null_mut()
        } else {
            destination.as_mut_ptr().add(1)
        };
        let src = if effective == 0 {
            std::ptr::null()
        } else if ALIAS {
            dst as *const u8
        } else {
            source.as_ptr().add(1)
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
    let original = if ALIAS {
        &before[1..25]
    } else {
        &source_data[1..1 + src_nb]
    };
    let inside = observed > 0 && observed < 25;
    let position = observed.saturating_sub(1) * 8 + bit;
    let expected = if inside {
        words::window_stored_bit(&before[1..25], original, dst_width, amount, width, position)
    } else {
        words::bit(&before, observed * 8 + bit)
    };
    assert!(
        words::bit(&destination, observed * 8 + bit) == expected,
        "simulator window store semantics mismatch"
    );
    let source_observed = usize::from(kani::any::<u8>() % 25);
    if source_observed < source.len() {
        assert!(
            source[source_observed] == source_data[source_observed],
            "simulator window store semantics mismatch"
        );
    }
    kani::cover!(
        inside && position == amount as usize && words::bit(original, 0),
        "window first written bit is reachable"
    );
    kani::cover!(
        inside
            && position == amount as usize + effective - 1
            && words::bit(original, effective - 1),
        "window last written bit is reachable"
    );
    kani::cover!(
        inside && position == amount as usize - 1,
        "window preceding bit is reachable"
    );
    kani::cover!(
        effective > 0 && observed == 0,
        "window prefix guard is reachable"
    );
    kani::cover!(
        effective > 0 && observed == 25,
        "window suffix guard is reachable"
    );
    if CASE == 0 {
        kani::cover!(
            inside && position == 127 && words::bit(original, 64),
            "window source word boundary is reachable"
        );
        if ALIAS {
            kani::cover!(
                inside && position == 126 && words::bit(original, 0) != words::bit(original, 63),
                "window alias snapshot is reachable"
            );
        }
    } else if CASE == 1 {
        kani::cover!(
            inside && position == 128,
            "window following bit is reachable"
        );
    } else {
        kani::cover!(
            inside && position == 130 && words::bit(&before[1..25], 130),
            "window dirty padding is preserved"
        );
    }
}

#[kani::proof]
#[kani::unwind(2)]
fn wide_window_noop_preserves_storage() {
    let width = kani::any::<u32>() as usize;
    let dst_width = kani::any::<u32>() as usize;
    let amount: u64 = kani::any();
    kani::assume(width == 0 || amount >= dst_width as u64);
    let mut destination: [u8; 26] = kani::any();
    let before = destination;
    let observed = usize::from(kani::any::<u8>() % 26);
    unsafe {
        let dst = if dst_width == 0 {
            std::ptr::null_mut()
        } else {
            destination.as_mut_ptr().add(1)
        };
        wide_ops::wide_window_store(
            std::ptr::null_mut(),
            dst,
            std::ptr::null(),
            amount,
            width as u64 | ((dst_width as u64) << 32),
            kani::any(),
        );
    }
    assert!(
        destination[observed] == before[observed],
        "simulator window noop semantics mismatch"
    );
    kani::cover!(
        width == 0 && dst_width == 192 && amount == 0,
        "window zero width is reachable"
    );
    kani::cover!(
        dst_width == 0 && width > 0,
        "window empty destination is reachable"
    );
    kani::cover!(
        dst_width == 192 && amount == 192 && width > 0,
        "window outside destination is reachable"
    );
    kani::cover!(
        amount == u64::MAX && width > 0,
        "window maximum amount is reachable"
    );
}
