//! IEEE 1800-2023 rule mapping:
//! SV-WRITE (11.5.1): Ignore missing destinations; partially overlapping writes update existing bits only.
//!
//! Endpoints are resolved storage coordinates. Reversed endpoints are a defensive no-op.
//! The RHS is already prepared; this model does not choose assignment sizing or casts.

use crate::logic4::{Logic4, logic_from_planes};

pub fn assigned_plane_bit(dst: u128, rhs: u128, position: usize, beg: usize, end: usize) -> bool {
    assert!(position < 128);
    if end <= position && position <= beg {
        // The range predicate establishes 0 <= position - end < 128.
        (rhs >> (position - end)) & 1 != 0
    } else {
        (dst >> position) & 1 != 0
    }
}

pub fn assigned_logic_bit(
    dst: (u128, u128),
    rhs: (u128, u128),
    position: usize,
    beg: usize,
    end: usize,
) -> Logic4 {
    logic_from_planes(
        assigned_plane_bit(dst.0, rhs.0, position, beg, end),
        assigned_plane_bit(dst.1, rhs.1, position, beg, end),
    )
}

pub fn reference_plane(dst: u128, rhs: u128, width: usize, beg: usize, end: usize) -> u128 {
    assert!((1..=128).contains(&width));
    let mut result = 0;
    for position in 0..width {
        result |= u128::from(assigned_plane_bit(dst, rhs, position, beg, end)) << position;
    }
    result
}
