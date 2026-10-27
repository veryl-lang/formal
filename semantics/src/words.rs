//! Binary little-endian storage, independent of production word loops. Width zero is
//! a mask no-op sentinel. Resize fills capacity; logical-width masking is separate.
//! Arithmetic is modular, shifts use bit origins, and clipped windows preserve padding.
//! No four-state plane handling or source context resolution is implied.

pub fn bit(bytes: &[u8], position: usize) -> bool {
    bytes[position / 8] & (1 << (position % 8)) != 0
}

pub fn window_stored_bit(
    destination: &[u8],
    source: &[u8],
    destination_width: usize,
    amount: u64,
    width: usize,
    position: usize,
) -> bool {
    let position_u64 = position as u64;
    if position < destination_width
        && position_u64 >= amount
        && position_u64 - amount < width as u64
    {
        bit(source, (position_u64 - amount) as usize)
    } else {
        bit(destination, position)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StorageBitOp {
    And,
    Or,
    Xor,
    Xnor,
    AndNot,
    Not,
}

pub fn storage_bit(op: StorageBitOp, a: bool, b: bool) -> bool {
    use crate::gates::{self, BitOp};
    use crate::logic4::{Logic4, logic_from_planes};
    let a = logic_from_planes(a, false);
    let b = logic_from_planes(b, false);
    let result = match op {
        StorageBitOp::And => gates::binary(a, b, BitOp::And),
        StorageBitOp::Or => gates::binary(a, b, BitOp::Or),
        StorageBitOp::Xor => gates::binary(a, b, BitOp::Xor),
        StorageBitOp::Xnor => gates::binary(a, b, BitOp::Xnor),
        StorageBitOp::AndNot => gates::binary(a, gates::not(b), BitOp::And),
        StorageBitOp::Not => gates::not(a),
    };
    result == Logic4::One
}

pub fn logical_shifted_bit(source: &[u8], amount: u64, left: bool, position: usize) -> bool {
    let capacity = source.len() * 8;
    assert!(position < capacity);
    if amount >= capacity as u64 {
        return false;
    }
    let amount = amount as usize;
    if left {
        position >= amount && bit(source, position - amount)
    } else {
        amount < capacity - position && bit(source, position + amount)
    }
}

pub fn arithmetic_shifted_bit(source: &[u8], width: usize, amount: u64, position: usize) -> bool {
    assert!(width > 0 && width <= source.len() * 8);
    if position >= width {
        false
    } else if amount >= (width - position) as u64 {
        bit(source, width - 1)
    } else {
        bit(source, position + amount as usize)
    }
}

pub fn resized_bit(source: &[u8], source_width: usize, signed: bool, position: usize) -> bool {
    if position < source_width {
        bit(source, position)
    } else {
        signed && source_width > 0 && bit(source, source_width - 1)
    }
}

/// Width zero is the storage helper's no-op sentinel, not a zero-bit value.
pub fn masked_storage_bit(source: &[u8], width: usize, position: usize) -> bool {
    (width == 0 || position < width) && bit(source, position)
}

pub fn resize(source: &[u8], source_width: usize, signed: bool, bytes: usize) -> Vec<u8> {
    let mut result = vec![0; bytes];
    for position in 0..bytes * 8 {
        if resized_bit(source, source_width, signed, position) {
            result[position / 8] |= 1 << (position % 8);
        }
    }
    result
}

pub fn add_sub(a: &[u8], b: &[u8], subtract: bool) -> Vec<u8> {
    assert_eq!(a.len(), b.len());
    let mut result = vec![0; a.len()];
    let mut carry = subtract;
    for position in 0..a.len() * 8 {
        let x = bit(a, position);
        let y = bit(b, position) ^ subtract;
        if x ^ y ^ carry {
            result[position / 8] |= 1 << (position % 8);
        }
        carry = (x && y) || (carry && (x || y));
    }
    result
}

pub fn arithmetic_byte(a: &[u8], b: &[u8], position: usize, subtract: bool) -> u8 {
    let mut carry = u16::from(subtract);
    let mut result = 0;
    for i in 0..=position {
        let y = if subtract { !b[i] } else { b[i] };
        let sum = a[i] as u16 + y as u16 + carry;
        result = sum as u8;
        carry = sum >> 8;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn window_store_matches_integer_oracle() {
        for dst_width in [0usize, 1, 7, 8, 63, 64, 65, 127, 128] {
            for width in [0usize, 1, 2, 7, 8, 63, 64, 65, 127, 128, u32::MAX as usize] {
                for amount in (0..=129).chain([1u64 << 32, u64::MAX]) {
                    let count = if amount >= dst_width as u64 {
                        0
                    } else {
                        width.min(dst_width - amount as usize)
                    };
                    let low = if count == 128 {
                        u128::MAX
                    } else {
                        (1u128 << count) - 1
                    };
                    for source in [
                        0u128,
                        1,
                        u128::MAX,
                        u128::MAX / 3,
                        1u128 << 64,
                        (1u128 << 127) | 7,
                    ] {
                        let before = source.rotate_left(37) ^ (u128::MAX / 3);
                        let expected = if count == 0 {
                            before
                        } else {
                            (before & !(low << amount)) | ((source & low) << amount)
                        };
                        for position in 0..128 {
                            assert_eq!(
                                window_stored_bit(
                                    &before.to_le_bytes(),
                                    &source.to_le_bytes(),
                                    dst_width,
                                    amount,
                                    width,
                                    position
                                ),
                                expected & (1u128 << position) != 0
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn storage_gates_match_binary_truth_tables() {
        for (op, table) in [
            (StorageBitOp::And, 0b1000),
            (StorageBitOp::Or, 0b1110),
            (StorageBitOp::Xor, 0b0110),
            (StorageBitOp::Xnor, 0b1001),
            (StorageBitOp::AndNot, 0b0100),
            (StorageBitOp::Not, 0b0011),
        ] {
            for a in [false, true] {
                for b in [false, true] {
                    let index = (usize::from(a) * 2) + usize::from(b);
                    assert_eq!(storage_bit(op, a, b), table & (1 << index) != 0);
                }
            }
        }
    }

    #[test]
    fn arithmetic_shift_matches_integer_oracle() {
        for width in [1usize, 7, 8, 31, 32, 63, 64, 65, 127, 128] {
            let mask = if width == 128 {
                u128::MAX
            } else {
                (1u128 << width) - 1
            };
            for value in [
                0,
                1,
                u128::MAX,
                u128::MAX / 3,
                1u128 << (width - 1),
                (1u128 << (width - 1)) - 1,
            ] {
                let source = value.to_le_bytes();
                let signed = (((value & mask) << (128 - width)) as i128) >> (128 - width);
                for amount in (0..=129).chain([1u64 << 32, 1u64 << 63, u64::MAX]) {
                    let expected = if amount >= width as u64 {
                        if signed < 0 { mask } else { 0 }
                    } else {
                        (signed >> amount) as u128 & mask
                    };
                    for position in 0..128 {
                        assert_eq!(
                            arithmetic_shifted_bit(&source, width, amount, position),
                            expected & (1u128 << position) != 0
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn logical_shift_matches_integer_oracle() {
        for value in (0..128)
            .map(|bit| 1u128 << bit)
            .chain([0, u128::MAX, u128::MAX / 3])
        {
            let source = value.to_le_bytes();
            for amount in (0..=129).chain([1u64 << 32, 1u64 << 63, u64::MAX]) {
                for left in [false, true] {
                    let expected = if amount >= 128 {
                        0
                    } else if left {
                        value << amount
                    } else {
                        value >> amount
                    };
                    for position in 0..128 {
                        assert_eq!(
                            logical_shifted_bit(&source, amount, left, position),
                            expected & (1u128 << position) != 0
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn byte_arithmetic_matches_integer_oracle() {
        for a in 0..=u8::MAX {
            for b in 0..=u8::MAX {
                assert_eq!(arithmetic_byte(&[a], &[b], 0, false), a.wrapping_add(b));
                assert_eq!(arithmetic_byte(&[a], &[b], 0, true), a.wrapping_sub(b));
            }
        }
    }
}
