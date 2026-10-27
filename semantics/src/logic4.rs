//! IEEE 1800-2023 rule mapping:
//! SV-LOGIC (6.3.1): Distinguish 0, 1, X, and Z.
//!
//! The payload/mask encoding is an internal representation, not an IEEE requirement.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Logic4 {
    Zero,
    One,
    X,
    Z,
}

pub fn logic_from_planes(payload: bool, mask_xz: bool) -> Logic4 {
    match (payload, mask_xz) {
        (false, false) => Logic4::Zero,
        (true, false) => Logic4::One,
        (false, true) => Logic4::X,
        (true, true) => Logic4::Z,
    }
}

pub fn read_logic(payload: u128, mask_xz: u128, position: usize) -> Logic4 {
    assert!(position < 128);
    logic_from_planes(
        (payload >> position) & 1 != 0,
        (mask_xz >> position) & 1 != 0,
    )
}
