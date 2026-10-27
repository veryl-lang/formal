//! Finite non-innermost packed ranges scaled by remaining element extents,
//! including selected multi-axis suffixes. Independent physical-bit enumeration
//! checks row-local clipping; no arbitrary-rank or arbitrary-stride theorem.

use super::Backend;
use super::packed_ranges::run;

const SHAPES: &[(&[usize], usize)] = &[
    (&[3, 4, 3], 1),
    (&[2, 3, 4, 5], 2),
    (&[2, 3, 65], 1),
    (&[2, 3, 2, 17], 1),
];

#[test]
fn interpreter_packed_stride_read() {
    run(
        Backend::Interpret,
        SHAPES,
        &[true, false],
        false,
        "packed_stride_ranges::interpreter_packed_stride_read",
    );
}

#[test]
fn cranelift_packed_stride_read() {
    run(
        Backend::Cranelift,
        SHAPES,
        &[true, false],
        false,
        "packed_stride_ranges::cranelift_packed_stride_read",
    );
}

#[test]
fn interpreter_packed_stride_write() {
    run(
        Backend::Interpret,
        SHAPES,
        &[true, false],
        true,
        "packed_stride_ranges::interpreter_packed_stride_write",
    );
}

#[test]
fn cranelift_packed_stride_write() {
    run(
        Backend::Cranelift,
        &SHAPES[..2],
        &[true, false],
        true,
        "packed_stride_ranges::cranelift_packed_stride_write",
    );
}

#[test]
fn cranelift_wide_packed_stride_write() {
    run(
        Backend::Cranelift,
        &SHAPES[2..],
        &[false],
        true,
        "packed_stride_ranges::cranelift_wide_packed_stride_write",
    );
}

#[test]
fn fallback_packed_stride_write() {
    run(
        Backend::Cranelift,
        &SHAPES[2..],
        &[true],
        true,
        "packed_stride_ranges::fallback_packed_stride_write",
    );
}
