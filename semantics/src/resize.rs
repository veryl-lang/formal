//! IEEE 1800-2023 rule mapping:
//! SV-TRUNC (10.7): Discard excess high bits when narrowing an assignment.
//! SV-EXTEND (10.7, 11.8.2, 11.8.3, 11.8.4): Extend using the applicable signedness, including X/Z sign bits.
//!
//! Sign extension requires the caller-resolved sign mode. This bit model does not
//! establish propagation through an enclosing expression or destination-type conversion.

use crate::logic4::{Logic4, read_logic};

pub fn truncated_logic_bit(input: (u128, u128), width: usize, position: usize) -> Logic4 {
    if position < width {
        read_logic(input.0, input.1, position)
    } else {
        Logic4::Zero
    }
}

pub fn extended_logic_bit(
    input: (u128, u128),
    source_width: usize,
    output_width: usize,
    signed: bool,
    use_sign: bool,
    position: usize,
) -> Logic4 {
    assert!((1..=128).contains(&source_width));
    if position >= output_width {
        Logic4::Zero
    } else if position < source_width {
        read_logic(input.0, input.1, position)
    } else if signed && use_sign {
        read_logic(input.0, input.1, source_width - 1)
    } else {
        Logic4::Zero
    }
}

pub fn reference_planes(width: usize, bit: impl Fn(usize) -> Logic4) -> (u128, u128) {
    assert!((1..=128).contains(&width));
    let mut payload = 0;
    let mut mask_xz = 0;
    for position in 0..width {
        let (p, xz) = match bit(position) {
            Logic4::Zero => (false, false),
            Logic4::One => (true, false),
            Logic4::X => (false, true),
            Logic4::Z => (true, true),
        };
        payload |= u128::from(p) << position;
        mask_xz |= u128::from(xz) << position;
    }
    (payload, mask_xz)
}

#[cfg(test)]
mod tests {
    use super::reference_planes;
    use crate::logic4::Logic4::{One, X, Z, Zero};

    // The explicit encoding table is independent of read_logic. Keep plane
    // packing checked when compiler comparisons migrate to per-bit proofs.
    #[test]
    fn reference_planes_encodes_all_small_four_state_vectors() {
        let encoding = [Zero, One, X, Z];
        // Four-bit small selections can return seven bits, including missing X.
        for width in 1..=7 {
            let limit = 1u128 << width;
            for payload in 0..limit {
                for mask_xz in 0..limit {
                    let actual = reference_planes(width, |position| {
                        let p = (payload >> position) & 1;
                        let xz = (mask_xz >> position) & 1;
                        encoding[(p + 2 * xz) as usize]
                    });
                    assert_eq!(actual, (payload, mask_xz));
                }
            }
        }
    }
}
