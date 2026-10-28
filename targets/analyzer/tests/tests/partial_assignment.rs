//! Public Value dispatch and conversions complement the native assignment proof.
//! Native bit/metadata semantics are proved in
//! `targets/analyzer/kani/src/partial_assignment.rs`. Reference truth tables and
//! native/BigUint integration remain executable checks.

use veryl_analyzer::value::Value;
use veryl_formal_semantics::logic4::{Logic4, read_logic};
use veryl_formal_semantics::partial_assignment::{assigned_logic_bit, reference_plane};

#[test]
fn assignment_contract_reference_semantics_four_state_truth_table() {
    for (payload, mask_xz, expected) in [
        (0, 0, Logic4::Zero),
        (1, 0, Logic4::One),
        (0, 1, Logic4::X),
        (1, 1, Logic4::Z),
    ] {
        assert_eq!(read_logic(payload, mask_xz, 0), expected);
        assert_eq!(
            assigned_logic_bit((0, 0), (payload, mask_xz), 0, 0, 0),
            expected
        );
        assert_eq!(
            assigned_logic_bit((payload, mask_xz), (0, 0), 0, 1, 1),
            expected
        );
    }
}

fn width_mask(width: usize) -> u128 {
    u128::MAX >> (128 - width)
}

#[test]
fn assignment_contract_native_biguint_boundaries_and_conversions() {
    let patterns = [
        0,
        u128::MAX,
        0xaaaa_aaaa_aaaa_aaaa_aaaa_aaaa_aaaa_aaaa,
        0x5555_5555_5555_5555_5555_5555_5555_5555,
        1u128 << 63,
        1u128 << 64,
        1u128 << 127,
    ];
    for width in [1, 4, 63, 64, 65, 127, 128] {
        for rhs_width in [1, 4, 63, 64, 65, 127, 128] {
            for beg in 0..width {
                // Exercise every in-bounds field, including fields crossing bit 64.
                for end in 0..=beg {
                    for pattern in patterns {
                        for signed in [false, true] {
                            let dst_payload = !pattern & width_mask(width);
                            let dst_xz = pattern & width_mask(width);
                            let rhs_payload = pattern & width_mask(rhs_width);
                            let rhs_xz = !pattern & width_mask(rhs_width);
                            let mut actual = Value::from_u128(dst_payload, dst_xz, width, signed);
                            let rhs = Value::from_u128(rhs_payload, rhs_xz, rhs_width, !signed);
                            actual.assign(rhs, beg, end);
                            assert_eq!(
                                actual.payload_u128(),
                                reference_plane(dst_payload, rhs_payload, width, beg, end,),
                                "payload: dst={width}, rhs={rhs_width}, [{beg}:{end}]"
                            );
                            assert_eq!(
                                actual.mask_xz_u128(),
                                reference_plane(dst_xz, rhs_xz, width, beg, end,),
                                "mask_xz: dst={width}, rhs={rhs_width}, [{beg}:{end}]"
                            );
                            assert_eq!(actual.width(), width);
                            assert_eq!(actual.signed(), signed);
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn assignment_contract_rhs_high_bits_do_not_leak_into_neighbours() {
    let mut actual = Value::from_u128(61, 0, 14, false);
    actual.assign(Value::from_u128(4092, 0, 14, false), 7, 4);
    assert_eq!(actual.payload_u128(), 205);
    assert_eq!(actual.mask_xz_u128(), 0);
}

#[test]
fn assignment_contract_out_of_bounds_fields_clip_without_leaking() {
    for width in [2, 63, 64, 65, 128] {
        for (beg, end) in [
            (width, width),
            (width + 2, width + 1),
            (width + 2, width - 1),
            (usize::MAX, 0),
            (usize::MAX, usize::MAX),
            (0, 1),
        ] {
            for pattern in [0, u128::MAX, 0xaaaa_aaaa_aaaa_aaaa_aaaa_aaaa_aaaa_aaaa] {
                let dst_payload = pattern & width_mask(width);
                let dst_xz = !pattern & width_mask(width);
                let rhs_payload = !pattern;
                let rhs_xz = pattern;
                let mut actual = Value::from_u128(dst_payload, dst_xz, width, true);
                actual.assign(Value::from_u128(rhs_payload, rhs_xz, 128, false), beg, end);
                assert_eq!(
                    actual.payload_u128(),
                    reference_plane(dst_payload, rhs_payload, width, beg, end),
                    "payload: width={width}, [{beg}:{end}]"
                );
                assert_eq!(
                    actual.mask_xz_u128(),
                    reference_plane(dst_xz, rhs_xz, width, beg, end),
                    "mask_xz: width={width}, [{beg}:{end}]"
                );
                assert_eq!(actual.width(), width);
                assert!(actual.signed());
            }
        }
    }
}
