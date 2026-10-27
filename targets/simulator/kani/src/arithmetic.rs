//! Arbitrary binary operands in three words at byte offset one, split by add/sub
//! and disjoint/left-alias/right-alias modes. Pre-call snapshots define modular results.
//! Checks guards and separate source preservation. Other lengths, partial overlap,
//! and simultaneous alias of both operands are outside the proof domains.

use veryl_formal_semantics::words;
use veryl_simulator::wide_ops;

#[kani::proof]
#[kani::unwind(193)]
fn wide_add_three_words_matches_reference() {
    arithmetic::<false, 0>();
}

#[kani::proof]
#[kani::unwind(193)]
fn wide_sub_three_words_matches_reference() {
    arithmetic::<true, 0>();
}

#[kani::proof]
#[kani::unwind(193)]
fn wide_add_left_alias_matches_reference() {
    arithmetic::<false, 1>();
}

#[kani::proof]
#[kani::unwind(193)]
fn wide_add_right_alias_matches_reference() {
    arithmetic::<false, 2>();
}

#[kani::proof]
#[kani::unwind(193)]
fn wide_sub_left_alias_matches_reference() {
    arithmetic::<true, 1>();
}

#[kani::proof]
#[kani::unwind(193)]
fn wide_sub_right_alias_matches_reference() {
    arithmetic::<true, 2>();
}

fn arithmetic<const SUBTRACT: bool, const ALIAS: u8>() {
    let a: [u8; 26] = kani::any();
    let b: [u8; 26] = kani::any();
    let a_before = a;
    let b_before = b;
    let mut dst: [u8; 26] = kani::any();
    let before = dst;
    let x = if ALIAS == 1 {
        &before[1..25]
    } else {
        &a[1..25]
    };
    let y = if ALIAS == 2 {
        &before[1..25]
    } else {
        &b[1..25]
    };
    let expected = words::add_sub(x, y, SUBTRACT);
    unsafe {
        let output = dst.as_mut_ptr().add(1);
        let left = if ALIAS == 1 {
            output as *const u8
        } else {
            a.as_ptr().add(1)
        };
        let right = if ALIAS == 2 {
            output as *const u8
        } else {
            b.as_ptr().add(1)
        };
        if SUBTRACT {
            wide_ops::wide_sub(output, left, right, 24);
        } else {
            wide_ops::wide_add(output, left, right, 24);
        }
    }
    for i in 0..24 {
        assert!(
            dst[i + 1] == expected[i],
            "simulator arithmetic semantics mismatch"
        );
    }
    assert!(
        dst[0] == before[0] && dst[25] == before[25],
        "simulator arithmetic semantics mismatch"
    );
    for i in 0..26 {
        assert!(
            a[i] == a_before[i] && b[i] == b_before[i],
            "simulator arithmetic semantics mismatch"
        );
    }
    let low_a = u64::from_le_bytes(x[..8].try_into().unwrap());
    let low_b = u64::from_le_bytes(y[..8].try_into().unwrap());
    let middle_a = u64::from_le_bytes(x[8..16].try_into().unwrap());
    let middle_b = u64::from_le_bytes(y[8..16].try_into().unwrap());
    let low128_a = u128::from_le_bytes(x[..16].try_into().unwrap());
    let low128_b = u128::from_le_bytes(y[..16].try_into().unwrap());
    let first = if SUBTRACT {
        low_a < low_b
    } else {
        low_a as u128 + low_b as u128 > u64::MAX as u128
    };
    let second = if SUBTRACT {
        low128_a < low128_b
    } else {
        low128_a.checked_add(low128_b).is_none()
    };
    let chain = first && middle_b == 0 && middle_a == if SUBTRACT { 0 } else { u64::MAX };
    kani::cover!(first, "arithmetic first-word carry or borrow is reachable");
    kani::cover!(
        second,
        "arithmetic second-word carry or borrow is reachable"
    );
    kani::cover!(chain, "arithmetic carry or borrow chain is reachable");
}
