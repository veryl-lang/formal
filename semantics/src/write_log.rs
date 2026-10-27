//! Internal record format and commit contract, not source NBA scheduling. Window
//! records carry at most 56 payload bytes with checked offsets. Commit processes narrow
//! records before wide records, preserving insertion order within each pool. Invalid
//! spans are skipped atomically; each entry changing its current span emits an event.

use crate::words;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WideRecord {
    pub offset: u32,
    pub payload: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NarrowRecord {
    pub offset: u32,
    pub width_class: u16,
    pub payload: u64,
}

pub fn committed(
    before: &[u8],
    narrow: &[NarrowRecord],
    wide: &[WideRecord],
) -> (Vec<u8>, Vec<(usize, usize)>) {
    let mut state = before.to_vec();
    let mut changed = Vec::new();
    let narrow_bytes: Vec<_> = narrow
        .iter()
        .filter(|entry| [1, 2, 4, 8].contains(&entry.width_class))
        .map(|entry| {
            (
                entry.offset,
                entry.payload.to_le_bytes()[..entry.width_class as usize].to_vec(),
            )
        })
        .collect();
    for (offset, bytes) in narrow_bytes
        .iter()
        .map(|(off, bytes)| (*off, &bytes[..]))
        .chain(
            wide.iter()
                .filter(|entry| !entry.payload.is_empty() && entry.payload.len() <= 56)
                .map(|entry| (entry.offset, &entry.payload[..])),
        )
    {
        let start = offset as usize;
        let Some(end) = start
            .checked_add(bytes.len())
            .filter(|&end| end <= state.len())
        else {
            continue;
        };
        if state[start..end] != *bytes {
            changed.push((start, bytes.len()));
        }
        for (i, byte) in bytes.iter().enumerate() {
            state[start + i] = *byte;
        }
    }
    (state, changed)
}

pub fn window_records(
    before: &[u8],
    source: &[u8],
    dst_width: usize,
    amount: u64,
    width: usize,
    base: u32,
) -> Option<Vec<WideRecord>> {
    if width == 0 || amount >= dst_width as u64 {
        return Some(Vec::new());
    }
    assert!(before.len().is_multiple_of(8) && dst_width <= before.len() * 8);
    let mut records: Vec<WideRecord> = Vec::new();
    for byte in 0..before.len() {
        let word_start = (byte / 8) * 64;
        let touched = (word_start..word_start + 64)
            .any(|p| p < dst_width && p as u64 >= amount && p as u64 - amount < width as u64);
        if !touched {
            continue;
        }
        let address = u32::try_from(u64::from(base) + byte as u64).ok()?;
        if records.last().is_none_or(|r| r.payload.len() == 56) {
            records.push(WideRecord {
                offset: address,
                payload: Vec::new(),
            });
        }
        let mut value = 0u8;
        for bit in 0..8 {
            if words::window_stored_bit(before, source, dst_width, amount, width, byte * 8 + bit) {
                value |= 1 << bit;
            }
        }
        records.last_mut().unwrap().payload.push(value);
    }
    Some(records)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn commit_matches_literal_states_and_events() {
        let narrow = [
            NarrowRecord {
                offset: 1,
                width_class: 2,
                payload: 0x1234,
            },
            NarrowRecord {
                offset: 1,
                width_class: 1,
                payload: 0,
            },
            NarrowRecord {
                offset: 3,
                width_class: 2,
                payload: u64::MAX,
            },
            NarrowRecord {
                offset: 0,
                width_class: 3,
                payload: u64::MAX,
            },
        ];
        let wide = [
            WideRecord {
                offset: 2,
                payload: vec![0x56, 0x78],
            },
            WideRecord {
                offset: 0,
                payload: vec![],
            },
            WideRecord {
                offset: 0,
                payload: vec![255; 57],
            },
            WideRecord {
                offset: u32::MAX,
                payload: vec![255],
            },
        ];
        assert_eq!(
            committed(&[0; 4], &narrow, &wide),
            (vec![0, 0, 0x56, 0x78], vec![(1, 2), (1, 1), (2, 2)])
        );
        let restored = [
            WideRecord {
                offset: 0,
                payload: vec![1],
            },
            WideRecord {
                offset: 0,
                payload: vec![0],
            },
            WideRecord {
                offset: 0,
                payload: vec![0],
            },
        ];
        assert_eq!(
            committed(&[0], &[], &restored),
            (vec![0], vec![(0, 1), (0, 1)])
        );
        assert_eq!(committed(&[], &narrow, &wide), (vec![], vec![]));
    }

    #[test]
    fn records_match_literal_frames() {
        let source: Vec<u8> = (0..64).collect();
        assert_eq!(
            window_records(&[0; 64], &source, 512, 0, 512, 7),
            Some(vec![
                WideRecord {
                    offset: 7,
                    payload: (0..56).collect()
                },
                WideRecord {
                    offset: 63,
                    payload: (56..64).collect()
                },
            ])
        );
        assert_eq!(
            window_records(&[255; 8], &[0; 8], 1, 0, 1, 0),
            Some(vec![WideRecord {
                offset: 0,
                payload: vec![254, 255, 255, 255, 255, 255, 255, 255]
            }])
        );
        assert_eq!(
            window_records(&[0; 24], &[255; 8], 192, 64, 64, u32::MAX - 15),
            Some(vec![WideRecord {
                offset: u32::MAX - 7,
                payload: vec![255; 8]
            }])
        );
        assert_eq!(
            window_records(&[0; 24], &[255; 8], 192, 64, 64, u32::MAX - 14),
            None
        );
        assert_eq!(
            window_records(&[], &[], 0, u64::MAX, usize::MAX, u32::MAX),
            Some(Vec::new())
        );
    }
}
