//! Public native resize domains are proved in targets/analyzer/kani/src/resize.rs.
//! Retain native/BigUint boundaries and independent reference truth tables.

use veryl_analyzer::value::Value;
use veryl_formal_semantics::logic4::{Logic4, read_logic};
use veryl_formal_semantics::resize::{extended_logic_bit, reference_planes, truncated_logic_bit};

fn check_width_changes(payload: u128, mask_xz: u128, source_width: usize, width: usize) {
    for signed in [false, true] {
        let input = Value::from_u128(payload, mask_xz, source_width, signed);
        let mut truncated = input.clone();
        truncated.trunc(width);
        let truncated_width = width.min(source_width);
        let expected = reference_planes(truncated_width, |position| {
            truncated_logic_bit((payload, mask_xz), truncated_width, position)
        });
        assert_eq!(
            (truncated.payload_u128(), truncated.mask_xz_u128()),
            expected,
            "truncation: source={source_width}, target={width}, signed={signed}"
        );
        assert_eq!(truncated.width(), truncated_width);
        assert_eq!(truncated.signed(), signed);
        assert_eq!(matches!(truncated, Value::U64(_)), truncated_width <= 64);

        for use_sign in [false, true] {
            let expanded = input.expand(width, use_sign);
            let expanded_width = width.max(source_width);
            let expected = reference_planes(expanded_width, |position| {
                extended_logic_bit(
                    (payload, mask_xz),
                    source_width,
                    expanded_width,
                    signed,
                    use_sign,
                    position,
                )
            });
            assert_eq!(
                (expanded.payload_u128(), expanded.mask_xz_u128()),
                expected,
                "extension: source={source_width}, target={width}, signed={signed}, use_sign={use_sign}"
            );
            assert_eq!(expanded.width(), expanded_width);
            assert_eq!(
                expanded.signed(),
                signed && (width <= source_width || use_sign)
            );
            assert_eq!(
                matches!(expanded.as_ref(), Value::U64(_)),
                expanded_width <= 64
            );
            assert_eq!(input.payload_u128(), payload);
            assert_eq!(input.mask_xz_u128(), mask_xz);
            assert_eq!(input.width(), source_width);
            assert_eq!(input.signed(), signed);
        }
    }
}

#[test]
fn resize_native_biguint_boundaries() {
    for source_width in [1, 2, 31, 32, 63, 64, 65, 127, 128] {
        let mask = u128::MAX >> (128 - source_width);
        let sign_bit = 1u128 << (source_width - 1);
        for width in [1, 2, 31, 32, 63, 64, 65, 127, 128] {
            // Public native resize quantifies all four-state values and both flags.
            if source_width <= 64 && width <= 64 {
                continue;
            }
            for (payload, mask_xz) in [
                (0, 0),
                (mask, 0),
                (0, mask),
                (mask, mask),
                (sign_bit, 0),
                (0, sign_bit),
                (sign_bit, sign_bit),
                (
                    0xaaaa_aaaa_aaaa_aaaa_aaaa_aaaa_aaaa_aaaa & mask,
                    0x5555_5555_5555_5555_5555_5555_5555_5555 & mask,
                ),
            ] {
                check_width_changes(payload, mask_xz, source_width, width);
            }
        }
    }
}

#[test]
fn resize_reference_sign_bit_truth_table() {
    for (payload, mask_xz, sign) in [
        (0, 0, Logic4::Zero),
        (1, 0, Logic4::One),
        (0, 1, Logic4::X),
        (1, 1, Logic4::Z),
    ] {
        for position in 0..4 {
            assert_eq!(
                extended_logic_bit((payload, mask_xz), 1, 4, true, true, position),
                sign
            );
            let zero_extended = if position == 0 { sign } else { Logic4::Zero };
            assert_eq!(
                extended_logic_bit((payload, mask_xz), 1, 4, true, false, position),
                zero_extended
            );
            assert_eq!(
                extended_logic_bit((payload, mask_xz), 1, 4, false, true, position),
                zero_extended
            );
        }
        let (p, xz) = reference_planes(4, |_| sign);
        assert_eq!(read_logic(p, xz, 3), sign);
        assert_eq!(
            extended_logic_bit((payload, mask_xz), 1, 4, true, true, 4),
            Logic4::Zero
        );
    }
}
