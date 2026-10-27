//! IEEE 1800-2023 rule mapping:
//! SV-READ (11.5.1): Missing source bits read X for four-state values.
//!
//! Endpoints are resolved storage coordinates; missing four-state source bits read X.
//! Two-state conversion, declared range direction, and selector evaluation are caller concerns.

use crate::logic4::{Logic4, read_logic};

pub fn selected_logic_bit(
    input: (u128, u128),
    source_width: usize,
    beg: usize,
    end: usize,
    position: usize,
) -> Logic4 {
    assert!((1..=128).contains(&source_width));
    if beg < end || position > beg - end {
        Logic4::Zero
    } else if end >= source_width || position >= source_width - end {
        Logic4::X
    } else {
        read_logic(input.0, input.1, end + position)
    }
}
