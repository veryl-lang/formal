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

#[cfg(test)]
mod tests {
    use super::{Logic4, read_logic};

    #[test]
    fn plane_decoding_covers_every_u128_position() {
        for position in 0..128 {
            let bit = 1u128 << position;
            for (p, xz, expected) in [
                (false, false, Logic4::Zero),
                (true, false, Logic4::One),
                (false, true, Logic4::X),
                (true, true, Logic4::Z),
            ] {
                for (payload_background, xz_background) in [
                    (0, 0),
                    (u128::MAX, 0),
                    (0, u128::MAX),
                    (u128::MAX, u128::MAX),
                ] {
                    let payload = (payload_background & !bit) | if p { bit } else { 0 };
                    let mask_xz = (xz_background & !bit) | if xz { bit } else { 0 };
                    assert_eq!(read_logic(payload, mask_xz, position), expected);
                }
            }
        }
    }
}
