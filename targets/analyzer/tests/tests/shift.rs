//! The initial unsigned native four-call shift prefix, including amount decoding,
//! is proved in targets/analyzer/kani/src/public_shift.rs. Execute every call;
//! later source/amount signs and native/BigUint contexts remain comparisons.
use veryl_analyzer::ir::Op;
use veryl_analyzer::value::{MaskCache, Value};
use veryl_formal_semantics::logic4::Logic4;
use veryl_formal_semantics::resize::{extended_logic_bit, reference_planes};
use veryl_formal_semantics::shift::{Shift, shifted_logic_bit};

const SHIFTS: [(Op, Shift); 4] = [
    (Op::LogicShiftL, Shift::LogicalLeft),
    (Op::LogicShiftR, Shift::LogicalRight),
    (Op::ArithShiftL, Shift::ArithmeticLeft),
    (Op::ArithShiftR, Shift::ArithmeticRight),
];

#[test]
fn shift_context_widens_before_shifting() {
    for (source_width, width) in [(4, 8), (4, 64), (4, 65), (64, 65), (65, 128)] {
        let mask = u128::MAX >> (128 - source_width);
        let sign = 1u128 << (source_width - 1);
        for source in [(1, 0), (mask, 0), (sign, sign), (1, sign)] {
            for signed in [false, true] {
                let input = Value::from_u128(source.0, source.1, source_width, signed);
                let prepared = reference_planes(width, |position| {
                    extended_logic_bit(source, source_width, width, signed, signed, position)
                });
                for amount in [0, 1, source_width, width, width + 1] {
                    let rhs = Value::new(amount as u64, 64, false);
                    for (op, kind) in SHIFTS {
                        let output = op.eval_value_binary(
                            &input,
                            &rhs,
                            width,
                            signed,
                            &mut MaskCache::default(),
                        );
                        let expected = reference_planes(width, |position| {
                            shifted_logic_bit(prepared, width, Some(amount), kind, signed, position)
                        });
                        assert_eq!(
                            (output.payload_u128(), output.mask_xz_u128()),
                            expected,
                            "{kind:?}, source_width={source_width}, width={width}, signed={signed}, amount={amount}"
                        );
                        assert_eq!(output.width(), width);
                        assert_eq!(
                            output.signed(),
                            signed
                                && matches!(kind, Shift::ArithmeticLeft | Shift::ArithmeticRight)
                        );
                        assert_eq!(matches!(output, Value::U64(_)), width <= 64);
                    }
                }
            }
        }
    }
}

fn check_shift(source: (u128, u128), width: usize, amount: (u128, u128), amount_width: usize) {
    let mut cache = MaskCache::default();
    let native = (1..=64).contains(&width)
        && ((source.0 | source.1) >> width) == 0
        && amount_width == 64
        && ((amount.0 | amount.1) >> 64) == 0;
    let mut calls = 0;
    let mut prefix_matches = true;
    let decoded = if amount.1 != 0 {
        None
    } else {
        Some(usize::try_from(amount.0).unwrap_or(usize::MAX))
    };
    for signed in [false, true] {
        let input = Value::from_u128(source.0, source.1, width, signed);
        for amount_signed in [false, true] {
            let rhs = Value::from_u128(amount.0, amount.1, amount_width, amount_signed);
            for (op, kind) in SHIFTS {
                let output = op.eval_value_binary(&input, &rhs, width, signed, &mut cache);
                let covered = prefix_matches
                    && native
                    && !signed
                    && !amount_signed
                    && SHIFTS
                        .get(calls)
                        .is_some_and(|&(expected, _)| op == expected);
                prefix_matches = covered;
                calls += 1;
                if covered {
                    continue;
                }
                let expected = reference_planes(width, |position| {
                    shifted_logic_bit(source, width, decoded, kind, signed, position)
                });
                assert_eq!(
                    (output.payload_u128(), output.mask_xz_u128()),
                    expected,
                    "{kind:?}, width={width}, source={source:?}, amount={amount:?}, signed={signed}"
                );
                assert_eq!(output.width(), width);
                let expected_signed = decoded.is_some()
                    && signed
                    && matches!(kind, Shift::ArithmeticLeft | Shift::ArithmeticRight);
                assert_eq!(output.signed(), expected_signed);
                assert_eq!(matches!(output, Value::U64(_)), width <= 64);
            }
            assert_eq!((rhs.payload_u128(), rhs.mask_xz_u128()), amount);
            assert_eq!(rhs.width(), amount_width);
            assert_eq!(rhs.signed(), amount_signed);
        }
        assert_eq!((input.payload_u128(), input.mask_xz_u128()), source);
        assert_eq!(input.width(), width);
        assert_eq!(input.signed(), signed);
    }
}

#[test]
fn shift_exhaustive_four_state_small_widths() {
    for width in 1..=4 {
        let mask = (1u128 << width) - 1;
        for state in 0..(1u128 << (2 * width)) {
            for amount in 0..=width + 1 {
                check_shift(
                    (state & mask, state >> width),
                    width,
                    (amount as u128, 0),
                    64,
                );
            }
            for amount in [(0, 1), (1, 1), (u64::MAX as u128, 0)] {
                check_shift((state & mask, state >> width), width, amount, 64);
            }
        }
    }
}

#[test]
fn shift_native_biguint_boundaries() {
    for width in [1, 2, 31, 32, 63, 64, 65, 127, 128] {
        let mask = u128::MAX >> (128 - width);
        let sign = 1u128 << (width - 1);
        for source in [
            (0, 0),
            (mask, 0),
            (0, mask),
            (mask, mask),
            (sign, 0),
            (0, sign),
            (sign, sign),
            (
                0xaaaa_aaaa_aaaa_aaaa_aaaa_aaaa_aaaa_aaaa & mask,
                0x5555_5555_5555_5555_5555_5555_5555_5555 & mask,
            ),
        ] {
            for amount in [0, 1, width - 1, width, width + 1, 63, 64, 65, usize::MAX] {
                check_shift(source, width, (amount as u128, 0), 64);
            }
            for amount in [
                (1, 0),
                (1u128 << 100, 0),
                (0, 1u128 << 100),
                (1u128 << 100, 1u128 << 100),
            ] {
                check_shift(source, width, amount, 128);
            }
        }
    }
}

#[test]
fn shift_reference_truth_table_and_extreme_amounts() {
    for (p, m, bit) in [
        (0, 0, Logic4::Zero),
        (1, 0, Logic4::One),
        (0, 1, Logic4::X),
        (1, 1, Logic4::Z),
    ] {
        for (_, kind) in SHIFTS {
            assert_eq!(shifted_logic_bit((p, m), 1, Some(0), kind, true, 0), bit);
            assert_eq!(shifted_logic_bit((p, m), 1, None, kind, true, 0), Logic4::X);
            assert_eq!(
                shifted_logic_bit((p, m), 1, None, kind, true, 1),
                Logic4::Zero
            );
            let fill = if kind == Shift::ArithmeticRight {
                bit
            } else {
                Logic4::Zero
            };
            assert_eq!(
                shifted_logic_bit((p, m), 1, Some(usize::MAX), kind, true, 0),
                fill
            );
            assert_eq!(
                shifted_logic_bit((p, m), 1, Some(usize::MAX), kind, false, 0),
                Logic4::Zero
            );
        }
    }
}
