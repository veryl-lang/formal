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

/// Integral-power special cases under the established X/Z-first interpretation.
/// For X/Z bases and a known zero exponent, the precedence of the general
/// arithmetic X/Z sentence and the explicit zero-power identity in IEEE 1800-2023
/// 11.4.3 remains unresolved here. Equivalence for that case is qualified by this
/// interpretation, as in the native and wide power harnesses.
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

/// IEEE1800-2023 11.4.3: exact division by signed units, or zero dividend.
/// Known normalized words1..128; division truncates toward zero and minimum/-1
/// wraps at the result width. This closed-form specification is independent of
/// production's sign/magnitude BigInt conversion and arithmetic APIs.
pub fn division_identity(a: u128, b: u128, width: usize, signed: bool) -> (u128, u128) {
    assert!((1..=128).contains(&width));
    let mask = u128::MAX >> (128 - width);
    assert!((a | b) & !mask == 0 && b != 0);
    assert!(a == 0 || (signed && (b == 1 || b == mask)));
    (
        if a == 0 {
            0
        } else if b == 1 {
            a
        } else {
            a.wrapping_neg() & mask
        },
        0,
    )
}

/// IEEE1800-2023 Table11-4: known positive powers with exponent1 or base0/1.
/// The exponent retains its source width/sign; the base is already prepared.
/// General modular exponentiation and X/Z inputs are outside this identity.
pub fn positive_power_identity(
    base: u128,
    exponent: u128,
    width: usize,
    exponent_width: usize,
    exponent_signed: bool,
) -> u128 {
    assert!((1..=128).contains(&width) && (1..=128).contains(&exponent_width));
    assert!(base & !(u128::MAX >> (128 - width)) == 0);
    assert!(exponent & !(u128::MAX >> (128 - exponent_width)) == 0);
    assert!(exponent != 0 && !(exponent_signed && exponent >> (exponent_width - 1) & 1 != 0));
    assert!(exponent == 1 || base <= 1);
    base
}

/// IEEE1800-2023 11.4.3: known division classes, after operand preparation.
#[derive(Clone, Copy)]
pub enum DivisionClass {
    Equal,
    UnsignedSmaller,
    UnsignedPowerOfTwo,
}

/// Scalar bit-coordinate specification, independent of the numeric API's limb
/// shifts, comparisons and sign/magnitude representation. Widths1..128; normalized
/// known words; equal nonzero operands permit either mode, other classes unsigned.
/// For a single-hot divisor, locate its bit and select dividend coordinates.
/// No production operation, allocation model or restoring-division summary is used.
pub fn division_class_bit(
    a: u128,
    b: u128,
    width: usize,
    signed: bool,
    position: usize,
    remainder: bool,
    class: DivisionClass,
) -> bool {
    assert!((1..=128).contains(&width) && position < 128);
    assert!((a | b) & !(u128::MAX >> (128 - width)) == 0 && b != 0);
    match class {
        DivisionClass::Equal => {
            assert!(a == b);
            !remainder && position == 0
        }
        DivisionClass::UnsignedSmaller => {
            assert!(!signed && a < b);
            remainder && position < width && a >> position & 1 != 0
        }
        DivisionClass::UnsignedPowerOfTwo => {
            assert!(!signed && b.is_power_of_two());
            let mut selected = false;
            for bit in 0..width {
                if b >> bit & 1 != 0 {
                    selected = if remainder {
                        position < bit && a >> position & 1 != 0
                    } else {
                        position + bit < width && a >> (position + bit) & 1 != 0
                    };
                }
            }
            selected
        }
    }
}

/// IEEE1800-2023 11.4.3: multiplication modulo the prepared result width.
/// Known normalized words, widths1..128; factor has exactly one set bit.
/// Signedness does not change the low-width product of the prepared bit patterns.
/// The supplied factor position is checked against the actual bit pattern.
/// Select source coordinates independently of the numeric API's limb shifts.
pub fn one_hot_product_bit(
    number: u128,
    factor: u128,
    factor_position: usize,
    width: usize,
    position: usize,
) -> bool {
    assert!((1..=128).contains(&width) && position < 128);
    assert!((number | factor) & !(u128::MAX >> (128 - width)) == 0);
    assert!(factor_position < width);
    assert!(factor == 1u128 << factor_position);
    position < width
        && position >= factor_position
        && number >> (position - factor_position) & 1 != 0
}

/// Modular word form of the single-hot multiplication identity. Its agreement
/// with the independent scalar coordinate specification is checked separately.
pub fn one_hot_product_word(
    number: u128,
    factor: u128,
    factor_position: usize,
    width: usize,
) -> u128 {
    assert!((1..=128).contains(&width) && factor_position < width);
    let mask = u128::MAX >> (128 - width);
    assert!((number | factor) & !mask == 0 && factor == 1u128 << factor_position);
    (number << factor_position) & mask
}

/// IEEE1800-2023 11.4.3/11.6.1: division truncates toward zero, and remainder
/// takes the dividend sign. Known prepared opposite, nonzero signed operands
/// have quotient-1 and remainder0. Widths1..128; the minimum signed value has
/// no representable positive counterpart, so it cannot satisfy this domain.
pub fn opposite_division_word(a: u128, b: u128, width: usize, remainder: bool) -> u128 {
    assert!((1..=128).contains(&width));
    let mask = u128::MAX >> (128 - width);
    assert!((a | b) & !mask == 0);
    let shift = 128 - width;
    let x = ((a << shift) as i128) >> shift;
    let y = ((b << shift) as i128) >> shift;
    assert!(x != 0 && y != 0 && x.signum() != y.signum() && x.checked_add(y) == Some(0));
    if remainder { 0 } else { mask }
}

/// IEEE1800-2023 11.4.3/11.6.1: bounded signed two's-complement encoding.
/// Positive magnitudes are below the sign bit; the minimum signed value is
/// permitted only with a negative sign. Zero has one bit pattern for either sign.
pub fn signed_magnitude_word(magnitude: u128, negative: bool, width: usize) -> u128 {
    assert!((1..=128).contains(&width));
    let top = 1u128 << (width - 1);
    assert!(magnitude <= top && (magnitude < top || negative));
    if negative {
        magnitude.wrapping_neg() & (u128::MAX >> (128 - width))
    } else {
        magnitude
    }
}

/// IEEE1800-2023 11.4.3: signed division truncates toward zero. For known
/// normalized words with |a| < |b|, the quotient is zero and remainder is a.
/// Widths1..128; zero dividends and minimum signed divisors are included.
pub fn signed_smaller_division_word(a: u128, b: u128, width: usize, remainder: bool) -> u128 {
    assert!((1..=128).contains(&width));
    let mask = u128::MAX >> (128 - width);
    assert!((a | b) & !mask == 0);
    let shift = 128 - width;
    let x = ((a << shift) as i128) >> shift;
    let y = ((b << shift) as i128) >> shift;
    assert!(x.unsigned_abs() < y.unsigned_abs());
    if remainder { a } else { 0 }
}

/// Scalar coordinates for division by 2^shift, independent of numeric limbs.
/// All128-bit magnitudes; shifts0..127. Quotient selects higher coordinates,
/// remainder selects coordinates below the divisor's unique set bit.
pub fn power_two_division_magnitude_bit(
    magnitude: u128,
    shift: usize,
    position: usize,
    remainder: bool,
) -> bool {
    assert!(shift < 128 && position < 128);
    if remainder {
        position < shift && magnitude >> position & 1 != 0
    } else {
        position + shift < 128 && magnitude >> (position + shift) & 1 != 0
    }
}

/// IEEE1800-2023 11.4.3/11.6.1: known signed division by a power-of-two
/// magnitude, widths1..128. The supplied shift is checked against the actual
/// decoded divisor magnitude. Truncate toward zero, retain the dividend sign for
/// remainder, then encode modulo2^width. This includes minimum/-1 overflow.
pub fn signed_power_two_division_word(
    a: u128,
    b: u128,
    shift: usize,
    width: usize,
    remainder: bool,
) -> u128 {
    assert!((1..=128).contains(&width));
    let mask = u128::MAX >> (128 - width);
    assert!((a | b) & !mask == 0);
    let padding = 128 - width;
    let x = ((a << padding) as i128) >> padding;
    let y = ((b << padding) as i128) >> padding;
    let divisor = y.unsigned_abs();
    assert!(shift < width && divisor == 1u128 << shift);
    let magnitude = if remainder {
        x.unsigned_abs() & (divisor - 1)
    } else {
        x.unsigned_abs() >> shift
    };
    let negative = if remainder { x < 0 } else { (x < 0) != (y < 0) };
    (if negative {
        magnitude.wrapping_neg()
    } else {
        magnitude
    }) & mask
}

/// IEEE1800-2023 11.4.3/11.6.1: multiplication by an all-ones prepared word
/// equals modular negation, regardless of the resolved signed mode. Known
/// normalized words, widths1..128; no production or numeric limb APIs are used.
pub fn all_ones_product_word(number: u128, factor: u128, width: usize) -> u128 {
    assert!((1..=128).contains(&width));
    let mask = u128::MAX >> (128 - width);
    assert!((number | factor) & !mask == 0 && factor == mask);
    number.wrapping_neg() & mask
}

/// Scalar subtraction carry for 0-number, independent of native negation and
/// the numeric API's limb subtraction. A bit is inverted exactly when some
/// less significant input bit is nonzero. Bits beyond the width are zero.
pub fn negated_word_bit(number: u128, width: usize, position: usize) -> bool {
    assert!((1..=128).contains(&width) && position < 128);
    assert!(number & !(u128::MAX >> (128 - width)) == 0);
    if position >= width {
        return false;
    }
    let lower = if position == 0 {
        0
    } else {
        u128::MAX >> (128 - position)
    };
    (number >> position & 1 != 0) ^ (number & lower != 0)
}

/// IEEE1800-2023 11.4.3/Table11-4,11.6.1: positive integral powers of an
/// all-ones prepared word alternate between that word and1 modulo2^width.
/// The exponent keeps its original width/sign; the base must be known, and
/// result/exponent widths1..128. Negative/zero exponents have separate contracts.
pub fn all_ones_power_word(
    base: u128,
    exponent: u128,
    width: usize,
    exponent_width: usize,
    exponent_signed: bool,
) -> u128 {
    assert!((1..=128).contains(&width) && (1..=128).contains(&exponent_width));
    let mask = u128::MAX >> (128 - width);
    assert!(base == mask && exponent & !(u128::MAX >> (128 - exponent_width)) == 0);
    assert!(
        exponent != 0 && (!exponent_signed || ((exponent << (128 - exponent_width)) as i128) >= 0)
    );
    if exponent & 1 != 0 { mask } else { 1 }
}

/// IEEE1800-2023 11.4.3/11.6.1: powers of a single-hot prepared word.
/// For widths1..128 and any nonnegative integral exponent, a result bit exists
/// exactly at a multiple of the base position whose quotient is the exponent.
/// This scalar coordinate definition avoids multiplying large exponents.
/// Signed minimum bases have the same residue modulo2^width.
pub fn one_hot_power_bit(
    base_position: usize,
    exponent: u128,
    width: usize,
    position: usize,
) -> bool {
    assert!((1..=128).contains(&width) && base_position < width);
    if position >= width {
        return false;
    }
    if base_position == 0 {
        return position == 0;
    }
    let p = position as u8;
    let k = base_position as u8;
    p % k == 0 && u128::from(p / k) == exponent
}

/// IEEE1800-2023 11.4.3/11.6.1: unsigned quotient-one division and remainder.
/// Known normalized words, widths1..128, divisor>0 and divisor<=dividend<2*divisor.
/// Express the upper bound with subtraction to retain divisors above2^127.
/// Quotient is1; remainder bits use independent scalar subtraction borrows.
pub fn unit_quotient_bit(a: u128, b: u128, width: usize, position: usize, remainder: bool) -> bool {
    assert!((1..=128).contains(&width) && position < 128);
    assert!((a | b) & !(u128::MAX >> (128 - width)) == 0);
    assert!(b != 0 && a >= b && a - b < b);
    if position >= width {
        return false;
    }
    if !remainder {
        return position == 0;
    }
    let lower = if position == 0 {
        0
    } else {
        u128::MAX >> (128 - position)
    };
    let borrow = (a & lower) < (b & lower);
    ((a >> position) ^ (b >> position) ^ u128::from(borrow)) & 1 != 0
}

/// IEEE1800-2023 11.4.3/11.6.1: unsigned division with quotient2^shift.
/// Known normalized words, widths1..128. The divisor is nonzero and its exact
/// shifted product fits below the dividend; remainder is smaller than divisor.
/// Quotient coordinates select shift; remainder uses scalar subtraction borrows.
pub fn power_quotient_bit(
    a: u128,
    b: u128,
    shift: usize,
    width: usize,
    position: usize,
    remainder: bool,
) -> bool {
    assert!((1..=128).contains(&width) && shift < width && position < 128);
    assert!((a | b) & !(u128::MAX >> (128 - width)) == 0);
    assert!(b != 0 && b <= a >> shift);
    let product = b << shift;
    assert!(a - product < b);
    if position >= width {
        return false;
    }
    if !remainder {
        return position == shift;
    }
    let lower = if position == 0 {
        0
    } else {
        u128::MAX >> (128 - position)
    };
    ((a >> position) ^ (product >> position) ^ u128::from((a & lower) < (product & lower))) & 1 != 0
}

/// IEEE1800-2023 11.4.3/11.6.1: signed quotient-magnitude2^shift.
/// Known normalized words1..128; nonzero divisor, truncation toward zero and
/// remainder following dividend sign. The quotient-magnitude class is checked
/// on decoded signed magnitudes; minimum/-1 encodes modulo the result width.
pub fn signed_power_quotient_word(
    a: u128,
    b: u128,
    shift: usize,
    width: usize,
    remainder: bool,
) -> u128 {
    assert!((1..=128).contains(&width) && shift < width);
    let mask = u128::MAX >> (128 - width);
    assert!((a | b) & !mask == 0);
    let padding = 128 - width;
    let x = ((a << padding) as i128) >> padding;
    let y = ((b << padding) as i128) >> padding;
    let ma = x.unsigned_abs();
    let mb = y.unsigned_abs();
    assert!(mb != 0 && mb <= ma >> shift);
    let rem = ma - (mb << shift);
    assert!(rem < mb);
    let magnitude = if remainder { rem } else { 1u128 << shift };
    let negative = if remainder { x < 0 } else { (x < 0) != (y < 0) };
    (if negative {
        magnitude.wrapping_neg()
    } else {
        magnitude
    }) & mask
}
