//! Finite executable integer references over least-significant-bit-first vectors.
pub fn negate(input: &[bool]) -> Vec<bool> {
    let mut carry = true;
    input
        .iter()
        .map(|&bit| {
            let result = !bit ^ carry;
            carry &= !bit;
            result
        })
        .collect()
}
pub fn multiply(a: &[bool], b: &[bool]) -> Vec<bool> {
    assert_eq!(a.len(), b.len());
    let mut result = vec![false; a.len()];
    for (shift, &set) in b.iter().enumerate() {
        if set {
            let mut carry = false;
            for (out, &bit) in result[shift..].iter_mut().zip(a) {
                let sum = *out ^ bit ^ carry;
                carry = (*out && bit) || (carry && (*out ^ bit));
                *out = sum;
            }
        }
    }
    result
}
pub fn divide_remainder(a: &[bool], b: &[bool], signed: bool) -> Option<(Vec<bool>, Vec<bool>)> {
    assert!(!a.is_empty() && a.len() == b.len());
    if !b.contains(&true) {
        return None;
    }
    let negative_a = signed && *a.last().unwrap();
    let negative_b = signed && *b.last().unwrap();
    let dividend = if negative_a { negate(a) } else { a.to_vec() };
    let mut divisor = if negative_b { negate(b) } else { b.to_vec() };
    divisor.push(false);
    let mut quotient = vec![false; a.len()];
    let mut remainder = vec![false; a.len() + 1];
    for index in (0..a.len()).rev() {
        remainder.rotate_right(1);
        remainder[0] = dividend[index];
        if remainder.iter().rev().cmp(divisor.iter().rev()) != std::cmp::Ordering::Less {
            let mut borrow = false;
            for (out, &bit) in remainder.iter_mut().zip(&divisor) {
                let difference = *out ^ bit ^ borrow;
                borrow = (!*out && bit) || (borrow && !(*out ^ bit));
                *out = difference;
            }
            assert!(!borrow);
            quotient[index] = true;
        }
    }
    remainder.pop();
    if negative_a != negative_b {
        quotient = negate(&quotient);
    }
    if negative_a {
        remainder = negate(&remainder);
    }
    Some((quotient, remainder))
}
pub fn bytes(bits: &[bool]) -> Vec<u8> {
    let mut out = vec![0; bits.len().div_ceil(64) * 8];
    for (i, &bit) in bits.iter().enumerate() {
        out[i / 8] |= u8::from(bit) << (i % 8);
    }
    out
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn small_integer_crosscheck() {
        for a in 0u16..16 {
            for b in 0u16..16 {
                let bits = |v: u16| (0..8).map(|i| v & (1 << i) != 0).collect::<Vec<_>>();
                assert_eq!(bytes(&multiply(&bits(a), &bits(b)))[0], (a * b) as u8);
                let result = divide_remainder(&bits(a), &bits(b), false);
                if let Some(quotient) = a.checked_div(b) {
                    let (q, r) = result.unwrap();
                    assert_eq!(
                        (bytes(&q)[0], bytes(&r)[0]),
                        (quotient as u8, a.checked_rem(b).unwrap() as u8)
                    );
                } else {
                    assert!(result.is_none());
                }
            }
        }
    }
}
