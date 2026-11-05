//! Variable binary window store: source widths 0..=64, destination logical
//! widths 0..=192 in a 192-bit allocation, and every u64 starting position.
//! All payload bits, safe u32 base offsets, direct/log-only writes and disjoint/
//! exact-base aliases are quantified; pointers have byte offset 1. One empty wide
//! slot and one live narrow record check record extent, payload, both frames and
//! guards, including no-op and logical-end clipping. Allocation succeeds.
//! SmallVec<[u64;16]>::from_elem uses a checked inline summary for at most three
//! elements; a separate, unstubbed constructor proof compares lengths, spill state
//! and every live element for all four sizes and arbitrary u64 elements. Finite
//! constant-size branches cover that full summary domain. Pointer identity, OOM
//! and constructor-proof teardown are excluded. Other capacities/histories,
//! source offsets, partial overlaps, four-state interpretation and commit are
//! outside scope. This is an internal memory protocol, not a scheduler theorem.
use veryl_simulator::{ir::WriteLogBuffer, wide_ops};
#[cfg(not(all(target_endian = "little", target_pointer_width = "64")))]
compile_error!("variable window proof requires a little-endian 64-bit target");
#[kani::proof]
#[kani::unwind(17)]
#[kani::stub(smallvec::SmallVec::<[u64;16]>::from_elem, generic_word_scratch)]
fn window_log_variable_matches_reference() {
    use veryl_formal_semantics::words;
    let sw: u64 = kani::any();
    let dw: [u64; 3] = kani::any();
    let mut source = [0u8; 9];
    source[0] = kani::any();
    let mut destination = [0u8; 26];
    destination[0] = kani::any();
    destination[25] = kani::any();
    unsafe {
        std::ptr::copy_nonoverlapping(
            (&sw as *const u64).cast::<u8>(),
            source.as_mut_ptr().add(1),
            8,
        );
        std::ptr::copy_nonoverlapping(
            dw.as_ptr().cast::<u8>(),
            destination.as_mut_ptr().add(1),
            24,
        );
    }
    let source_before = source;
    let before = destination;
    let width = usize::from(kani::any::<u8>() % 65);
    let dst_width = usize::from(kani::any::<u8>() % 193);
    let amount: u64 = kani::any();
    let base: u32 = kani::any();
    kani::assume(base <= u32::MAX - 23);
    let direct: bool = kani::any();
    let alias: bool = kani::any();
    let narrow: u64 = kani::any();
    let mut log = WriteLogBuffer::with_capacity(1, 1);
    unsafe {
        (log.grow_push_narrow)(&mut log, 3, narrow, 8);
        let dst = destination.as_mut_ptr().add(1);
        let src = if alias {
            dst as *const u8
        } else {
            source.as_ptr().add(1)
        };
        wide_ops::wide_window_store(
            &mut log,
            dst,
            src,
            amount,
            width as u64 | ((dst_width as u64) << 32),
            u64::from(base) | ((direct as u64) << 32),
        );
    }
    let active = width > 0 && amount < dst_width as u64;
    assert!(
        log.wide_count() == u32::from(active) && log.narrow_count() == 1,
        "simulator window log semantics mismatch"
    );
    let n = &log.narrow_entries_slice()[0];
    assert!(
        n.offset == 3 && n.width_class == 8 && n.mask_xz == 0 && n.payload == narrow,
        "simulator window log semantics mismatch"
    );
    let original = if alias {
        &before[1..9]
    } else {
        &source_before[1..]
    };
    if active {
        let e = &log.wide_entries_slice()[0];
        assert!(
            e.offset >= base
                && e.offset - base <= 16
                && (e.offset - base).is_multiple_of(8)
                && matches!(e.native_bytes, 8 | 16)
                && e.offset - base + u32::from(e.native_bytes) <= 24,
            "simulator window log semantics mismatch"
        );
        let delta = (e.offset - base) as usize;
        let word = usize::from(kani::any::<u8>() % 3);
        let stored = word * 8 >= delta && word * 8 - delta < usize::from(e.native_bytes);
        assert!(
            stored == words::window_touches_word(word, dst_width, amount, width),
            "simulator window log semantics mismatch"
        );
        let byte = usize::from(kani::any::<u8>() & 15);
        let bit = usize::from(kani::any::<u8>() & 7);
        if byte < usize::from(e.native_bytes) {
            let expected = words::window_stored_bit(
                &before[1..25],
                original,
                dst_width,
                amount,
                width,
                (delta + byte) * 8 + bit,
            );
            assert!(
                (e.payload[byte] & (1 << bit) != 0) == expected,
                "simulator window log semantics mismatch"
            );
        }
    }
    let observed = usize::from(kani::any::<u8>() % 26);
    let bit = usize::from(kani::any::<u8>() & 7);
    let expected = if direct && (1..25).contains(&observed) {
        words::window_stored_bit(
            &before[1..25],
            original,
            dst_width,
            amount,
            width,
            (observed - 1) * 8 + bit,
        )
    } else {
        words::bit(&before, observed * 8 + bit)
    };
    assert!(
        words::bit(&destination, observed * 8 + bit) == expected && source == source_before,
        "simulator window log semantics mismatch"
    );
    kani::cover!(width == 0, "zero source window is reachable");
    kani::cover!(dst_width == 0, "zero destination width is reachable");
    kani::cover!(amount == u64::MAX, "maximum window amount is reachable");
    kani::cover!(
        active && amount == 63 && width == 64 && dst_width == 192,
        "unaligned full window crosses a word"
    );
    kani::cover!(
        active && amount + width as u64 > dst_width as u64,
        "window is clipped at logical end"
    );
    kani::cover!(active && width == 1, "single-bit window is reachable");
    kani::cover!(
        active && amount == 191 && dst_width == 192,
        "last logical bit is reachable"
    );
    kani::cover!(
        active && direct && alias,
        "direct alias variable window is reachable"
    );
    kani::cover!(
        active && !direct && alias,
        "log-only alias variable window is reachable"
    );
    kani::cover!(
        active && direct && !alias,
        "direct external variable window is reachable"
    );
    kani::cover!(
        active && !direct && !alias,
        "log-only external variable window is reachable"
    );
}

#[kani::proof]
#[kani::unwind(17)]
fn window_scratch_refines_constructor() {
    let n = usize::from(kani::any::<u8>() & 3);
    let elem: u64 = kani::any();
    let actual = match n {
        0 => smallvec::SmallVec::<[u64; 16]>::from_elem(elem, 0),
        1 => smallvec::SmallVec::from_elem(elem, 1),
        2 => smallvec::SmallVec::from_elem(elem, 2),
        _ => smallvec::SmallVec::from_elem(elem, 3),
    };
    let modeled = generic_word_scratch::<[u64; 16]>(elem, n);
    assert!(
        actual.len() == n && modeled.len() == n && !actual.spilled() && !modeled.spilled(),
        "simulator window log semantics mismatch"
    );
    let i = usize::from(kani::any::<u8>() & 3);
    if i < n {
        assert!(
            actual[i] == elem && modeled[i] == elem,
            "simulator window log semantics mismatch"
        );
    }
    kani::cover!(n == 0, "empty scratch is reachable");
    kani::cover!(n == 3 && i == 2, "last bounded scratch word is reachable");
    // Teardown is outside the constructor equivalence contract.
    std::mem::forget(actual);
    std::mem::forget(modeled);
}

fn generic_word_scratch<A: smallvec::Array>(elem: A::Item, n: usize) -> smallvec::SmallVec<A>
where
    A::Item: Clone,
{
    assert!(
        n <= 3 && A::size() >= 3,
        "scratch summary precondition violated"
    );
    let mut buf = std::mem::MaybeUninit::<A>::uninit();
    // SAFETY: Array guarantees contiguous items; the checked bound fits A.
    // Each live slot is initialized before ownership passes to SmallVec.
    unsafe {
        let ptr = buf.as_mut_ptr().cast::<A::Item>();
        if n > 0 {
            ptr.write(elem.clone());
        }
        if n > 1 {
            ptr.add(1).write(elem.clone());
        }
        if n > 2 {
            ptr.add(2).write(elem);
        }
        smallvec::SmallVec::from_buf_and_len_unchecked(buf, n)
    }
}
