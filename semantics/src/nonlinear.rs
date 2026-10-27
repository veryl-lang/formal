//! IEEE 1800-2023 rule mapping:
//! SV-PRODUCT (11.4.3, 11.6.1): Multiply integral values at the prepared width, discarding excess high bits; X/Z poisons the result.
//! SV-DIVREM (11.4.3, 11.4.3.1): Integer quotient truncates toward zero; remainder follows dividend sign; zero divisor or X/Z yields all X.
//! SV-POWER (11.4.3, Table 11-4, 11.6.1): Keep exponent self-determined; apply integral positive/zero/negative-power cases and X/Z propagation.
//!
//! Signed quotient truncates toward zero; remainder follows the dividend. Fixed-width
//! minimum / -1 wraps to the minimum pattern. Exponent sizing is self-determined.
//! This definition does not itself establish arbitrary-width production equivalence.

#[derive(Clone, Copy, Debug)]
pub enum Nonlinear {
    Multiply,
    Divide,
    Remainder,
    Power,
}

pub fn product(mut a: u128, mut b: u128, width: usize) -> u128 {
    let mask = u128::MAX >> (128 - width);
    let mut result = 0u128;
    for _ in 0..width {
        if b & 1 != 0 {
            result = result.wrapping_add(a);
        }
        a = a.wrapping_add(a);
        b >>= 1;
    }
    result & mask
}

fn magnitude(x: u128, width: usize, signed: bool) -> (u128, bool) {
    let mask = u128::MAX >> (128 - width);
    let negative = signed && x >> (width - 1) & 1 != 0;
    (
        if negative {
            (!x).wrapping_add(1) & mask
        } else {
            x
        },
        negative,
    )
}

pub fn quotient_remainder(a: u128, b: u128, width: usize, signed: bool) -> (u128, u128) {
    let mask = u128::MAX >> (128 - width);
    let (a, an) = magnitude(a, width, signed);
    let (b, bn) = magnitude(b, width, signed);
    assert!(b != 0);
    let mut quotient = 0u128;
    let mut remainder = 0u128;
    for index in (0..width).rev() {
        let carry = remainder >> 127 != 0;
        remainder = (remainder << 1) | ((a >> index) & 1);
        quotient <<= 1;
        if carry || remainder >= b {
            remainder = remainder.wrapping_sub(b);
            quotient |= 1;
        }
    }
    if an != bn {
        quotient = (!quotient).wrapping_add(1) & mask;
    }
    if an {
        remainder = (!remainder).wrapping_add(1) & mask;
    }
    (quotient, remainder)
}

pub fn planes(
    a: (u128, u128),
    b: (u128, u128),
    width: usize,
    signed: bool,
    exponent_width: usize,
    exponent_signed: bool,
    kind: Nonlinear,
) -> (u128, u128) {
    assert!((1..=128).contains(&width));
    let mask = u128::MAX >> (128 - width);
    if a.1 != 0 || b.1 != 0 {
        return (0, mask);
    }
    let result = match kind {
        Nonlinear::Multiply => product(a.0, b.0, width),
        Nonlinear::Divide | Nonlinear::Remainder => {
            if b.0 == 0 {
                return (0, mask);
            }
            let (q, r) = quotient_remainder(a.0, b.0, width, signed);
            if matches!(kind, Nonlinear::Divide) {
                q
            } else {
                r
            }
        }
        Nonlinear::Power => {
            let negative = exponent_signed && b.0 >> (exponent_width - 1) & 1 != 0;
            if negative {
                if a.0 == 0 {
                    return (0, mask);
                }
                if a.0 == 1 {
                    1
                } else if signed && a.0 == mask {
                    if b.0 & 1 == 0 { 1 } else { mask }
                } else {
                    0
                }
            } else {
                let mut result = 1;
                let mut base = a.0;
                let mut exponent = b.0;
                for _ in 0..exponent_width {
                    if exponent & 1 != 0 {
                        result = product(result, base, width);
                    }
                    exponent >>= 1;
                    base = product(base, base, width);
                }
                result & mask
            }
        }
    };
    (result, 0)
}

pub fn power_exception(
    a: (u128, u128),
    b: (u128, u128),
    width: usize,
    signed: bool,
    exponent_width: usize,
    exponent_signed: bool,
) -> (u128, u128) {
    let mask = u128::MAX >> (128 - width);
    if a.1 != 0 || b.1 != 0 {
        return (0, mask);
    }
    let negative = exponent_signed && b.0 >> (exponent_width - 1) & 1 != 0;
    assert!(negative || b.0 == 0);
    if !negative {
        return (1, 0);
    }
    match (a.0, signed, b.0 & 1) {
        (0, _, _) => (0, mask),
        (1, _, _) => (1, 0),
        (base, true, odd) if base == mask => (if odd == 0 { 1 } else { mask }, 0),
        _ => (0, 0),
    }
}

pub fn small_power_planes(
    a: (u128, u128),
    b: (u128, u128),
    width: usize,
    signed: bool,
    exponent_width: usize,
    exponent_signed: bool,
) -> (u128, u128) {
    let negative = exponent_signed && b.0 >> (exponent_width - 1) & 1 != 0;
    if a.1 != 0 || b.1 != 0 || negative {
        return power_exception(a, b, width, signed, exponent_width, exponent_signed);
    }
    assert!(b.0 <= 7);
    let mut result = 1;
    for _ in 0..b.0 {
        result = product(result, a.0, width);
    }
    (result, 0)
}
