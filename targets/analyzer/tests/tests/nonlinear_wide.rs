//! Finite wide arithmetic comparisons against independent Boolean-vector shift/add
//! and restoring division. Widths and deterministic patterns are defined below;
//! positive powers are bounded. No arbitrary-width symbolic arithmetic theorem.

use veryl_analyzer::{
    ir::Op,
    value::{MaskCache, Value},
};
use veryl_formal_semantics::bitvector;
fn patterns(width: usize) -> Vec<Vec<bool>> {
    (0..8)
        .map(|kind| {
            (0..width)
                .map(|i| match kind {
                    0 => false,
                    1 => i == 0,
                    2 => true,
                    3 => i == width - 1,
                    4 => i % 2 == 0,
                    5 => i % 2 == 1,
                    6 => i == 0 || i == width - 1,
                    _ => (i * 37 + i / 7) % 11 < 5,
                })
                .collect()
        })
        .collect()
}
fn compare(op: Op, a: &[bool], b: &[bool], signed: bool, expected: Option<Vec<bool>>) {
    let width = a.len();
    let lhs = Value::from_le_bytes(
        &bitvector::bytes(a),
        &vec![0; width.div_ceil(64) * 8],
        width,
        signed,
    );
    let rhs = Value::from_le_bytes(
        &bitvector::bytes(b),
        &vec![0; b.len().div_ceil(64) * 8],
        b.len(),
        if op == Op::Pow { false } else { signed },
    );
    let got = op.eval_value_binary(&lhs, &rhs, width, signed, &mut MaskCache::default());
    let mut p = vec![0; width.div_ceil(64) * 8];
    let mut m = p.clone();
    got.write_payload_to_bytes(&mut p);
    got.write_mask_xz_to_bytes(&mut m);
    let want = expected
        .as_ref()
        .map(|v| bitvector::bytes(v))
        .unwrap_or_else(|| vec![0; p.len()]);
    let mask = if expected.is_none() {
        bitvector::bytes(&vec![true; width])
    } else {
        vec![0; p.len()]
    };
    assert_eq!(
        (p, m),
        (want, mask),
        "wide nonlinear output mismatch: {op:?} width={width} signed={signed}"
    );
    assert_eq!(
        (got.width(), got.signed()),
        (width, signed),
        "wide nonlinear output mismatch: metadata"
    );
}
#[test]
fn products_match_bit_vectors() {
    let mut count = 0;
    for width in [1, 7, 31, 63, 64, 65, 127, 128, 129, 257, 512] {
        let cases = patterns(width);
        for signed in [false, true] {
            for a in &cases {
                for b in &cases {
                    compare(Op::Mul, a, b, signed, Some(bitvector::multiply(a, b)));
                    count += 1;
                }
            }
        }
    }
    eprintln!("wide product complete comparisons={count}");
}
#[test]
fn division_and_remainder_match_bit_vectors() {
    let mut count = 0;
    for width in [1, 7, 31, 63, 64, 65, 127, 128, 129, 257, 512] {
        let cases = patterns(width);
        for signed in [false, true] {
            for a in &cases {
                for b in &cases {
                    let expected = bitvector::divide_remainder(a, b, signed);
                    compare(
                        Op::Div,
                        a,
                        b,
                        signed,
                        expected.as_ref().map(|(q, _)| q.clone()),
                    );
                    compare(Op::Rem, a, b, signed, expected.map(|(_, r)| r));
                    count += 2;
                }
            }
        }
    }
    eprintln!("wide division complete comparisons={count}");
}

#[test]
fn small_powers_match_bit_vectors() {
    let mut count = 0;
    for width in [1, 7, 31, 63, 64, 65, 127, 128, 129, 257, 512] {
        for signed in [false, true] {
            for a in patterns(width) {
                let mut power = vec![false; width];
                power[0] = true;
                for exponent in 0..8 {
                    let b: Vec<_> = (0..3).map(|bit| exponent & (1 << bit) != 0).collect();
                    compare(Op::Pow, &a, &b, signed, Some(power.clone()));
                    power = bitvector::multiply(&power, &a);
                    count += 1;
                }
            }
        }
    }
    eprintln!("wide power complete comparisons={count}");
}
