//! Public small selection domains are proved in targets/analyzer/kani/src/selection.rs.
//! Retain wider/BigUint and distant endpoints plus independent model checks.

use veryl_analyzer::value::Value;
use veryl_formal_semantics::logic4::Logic4;
use veryl_formal_semantics::resize::reference_planes;
use veryl_formal_semantics::selection::selected_logic_bit;

fn check_selection(payload: u128, mask_xz: u128, width: usize, beg: usize, end: usize) {
    for signed in [false, true] {
        let input = Value::from_u128(payload, mask_xz, width, signed);
        let original = input.clone();
        let result = input.select(beg, end);
        let output_width = if beg < end { 0 } else { beg - end + 1 };
        let expected = if output_width == 0 {
            (0, 0)
        } else {
            reference_planes(output_width, |position| {
                selected_logic_bit((payload, mask_xz), width, beg, end, position)
            })
        };
        assert_eq!(
            (result.payload_u128(), result.mask_xz_u128()),
            expected,
            "width={width}, [{beg}:{end}], signed={signed}"
        );
        assert_eq!(result.width(), output_width);
        assert!(!result.signed());
        assert_eq!(input, original);
        assert_eq!(matches!(result, Value::U64(_)), output_width <= 64);
    }
}

// Match the public small proof's complete domain before dropping a native case.
fn covered_small_selection(payload: u128, xz: u128, width: usize, beg: usize, end: usize) -> bool {
    (1..=4).contains(&width)
        && payload < (1u128 << width)
        && xz < (1u128 << width)
        && beg <= width + 2
        && end <= width + 2
}

#[test]
fn selection_native_biguint_boundaries() {
    for width in [1, 2, 31, 32, 63, 64, 65, 127, 128] {
        let mask = u128::MAX >> (128 - width);
        for (payload, mask_xz) in [
            (0, 0),
            (mask, 0),
            (0, mask),
            (mask, mask),
            (
                0xaaaa_aaaa_aaaa_aaaa_aaaa_aaaa_aaaa_aaaa & mask,
                0x5555_5555_5555_5555_5555_5555_5555_5555 & mask,
            ),
        ] {
            for (beg, end) in [
                (0, 0),
                (width - 1, 0),
                (width - 1, width - 1),
                (width, width),
                (width + 3, width),
                (width + 2, width - 1),
                (63, 63),
                (64, 63),
                (64, 64),
                (65, 64),
                (127, 64),
                (usize::MAX, usize::MAX),
                (usize::MAX, usize::MAX - 3),
                (0, 1),
                (0, usize::MAX),
            ] {
                // Native results cannot represent more than 64 stored bits.
                if width <= 64 && beg >= end && beg - end >= 64 {
                    continue;
                }
                if covered_small_selection(payload, mask_xz, width, beg, end) {
                    continue;
                }
                check_selection(payload, mask_xz, width, beg, end);
            }
        }
    }
}

#[test]
fn selection_reference_truth_table_and_missing_bits() {
    for (payload, mask_xz, expected) in [
        (0, 0, Logic4::Zero),
        (1, 0, Logic4::One),
        (0, 1, Logic4::X),
        (1, 1, Logic4::Z),
    ] {
        assert_eq!(selected_logic_bit((payload, mask_xz), 1, 0, 0, 0), expected);
        assert_eq!(
            selected_logic_bit((payload, mask_xz), 1, 1, 0, 1),
            Logic4::X
        );
        assert_eq!(
            selected_logic_bit((payload, mask_xz), 1, 0, 0, 1),
            Logic4::Zero
        );
        assert_eq!(
            selected_logic_bit((payload, mask_xz), 1, 0, 1, 0),
            Logic4::Zero
        );
        assert_eq!(
            selected_logic_bit((payload, mask_xz), 1, usize::MAX, usize::MAX, 0),
            Logic4::X
        );
    }
}
