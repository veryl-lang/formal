//! Finite selected output windows through 512 bits using independent source-bit
//! origins. Result width and source storage width are checked separately.

use super::{Backend, packed_read::run};
#[test]
fn interpreter_scalar_wide_packed_read() {
    run(
        Backend::Interpret,
        &[65, 128, 129, 256, 512],
        false,
        false,
        true,
        "packed_read_wide::interpreter_scalar_wide_packed_read",
    );
}
#[test]
fn cranelift_scalar_wide_packed_read() {
    run(
        Backend::Cranelift,
        &[65, 128, 129, 256, 512],
        false,
        false,
        true,
        "packed_read_wide::cranelift_scalar_wide_packed_read",
    );
}
#[test]
fn interpreter_array_wide_packed_read() {
    run(
        Backend::Interpret,
        &[65, 128, 129, 256, 512],
        true,
        false,
        true,
        "packed_read_wide::interpreter_array_wide_packed_read",
    );
}
#[test]
fn cranelift_array_wide_packed_read() {
    run(
        Backend::Cranelift,
        &[65, 128],
        true,
        false,
        true,
        "packed_read_wide::cranelift_array_wide_packed_read",
    );
}
#[test]
fn fallback_array_wide_packed_read() {
    run(
        Backend::Cranelift,
        &[129, 256, 512],
        true,
        true,
        true,
        "packed_read_wide::fallback_array_wide_packed_read",
    );
}
