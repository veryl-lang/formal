//! Shared finite domain for wide-amount caller proofs and constructor checks.
//! Each native boundary caller harness instantiates one entry. The native test
//! checks the real Value constructor for both signs before omitting matching traces.
pub const WIDE_SHIFT_AMOUNTS: [(u128, u128); 4] = [
    (1, 0),
    (1u128 << 100, 0),
    (0, 1u128 << 100),
    (1u128 << 100, 1u128 << 100),
];
