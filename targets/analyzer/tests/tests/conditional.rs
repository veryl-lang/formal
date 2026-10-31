//! Public native dispatch is proved in targets/analyzer/kani/src/conditional.rs.
//! public_big_values.rs adds all nonzero widths1..128 and contexts0..128.
//! Retain numeric integration, unsized-literal paths and independent model checks.
//! Public constructors and actual BigUint truth remain checked in big_helpers.rs.

use veryl_analyzer::value::Value;
use veryl_formal_semantics::comparison::Operand;
use veryl_formal_semantics::conditional::{conditional_logic_bit, merged_logic};
use veryl_formal_semantics::gates::logical_truth;
use veryl_formal_semantics::logic4::Logic4;
use veryl_formal_semantics::resize::reference_planes;

fn check(condition: Operand, a: Operand, b: Operand, context_width: usize, signed: bool) {
    let c = Value::from_u128(
        condition.planes.0,
        condition.planes.1,
        condition.width,
        condition.signed,
    );
    let x = Value::from_u128(a.planes.0, a.planes.1, a.width, a.signed);
    let y = Value::from_u128(b.planes.0, b.planes.1, b.width, b.signed);
    let result = c.conditional(&x, &y, context_width, signed);
    let width = context_width.max(a.width).max(b.width);
    let truth = logical_truth(condition.planes, condition.width);
    let expected = reference_planes(width, |position| {
        conditional_logic_bit(
            truth,
            (a.planes, a.width, a.signed),
            (b.planes, b.width, b.signed),
            width,
            signed,
            position,
        )
    });
    assert_eq!(
        (result.payload_u128(), result.mask_xz_u128()),
        expected,
        "condition={condition:?}, true={a:?}, false={b:?}, context={context_width}, signed={signed}"
    );
    assert_eq!(result.width(), width);
    assert_eq!(result.signed(), signed);
    assert_eq!(matches!(result, Value::U64(_)), width <= 64);
    for (value, before) in [(&c, condition), (&x, a), (&y, b)] {
        assert_eq!((value.payload_u128(), value.mask_xz_u128()), before.planes);
        assert_eq!(
            (value.width(), value.signed()),
            (before.width, before.signed)
        );
    }
}

// Kani covers all nonzero condition/branch widths1..128 and contexts0..128,
// using numeric API contracts. Retain real-library integration for all branch
// storage layouts, both source signs, caller signs and ambiguous conditions.
#[test]
fn conditional_native_biguint_boundaries() {
    fn patterns(width: usize) -> [(u128, u128); 6] {
        let mask = u128::MAX >> (128 - width);
        let sign = 1u128 << (width - 1);
        [
            (0, 0),
            (mask, 0),
            (0, mask),
            (mask, mask),
            (1, mask ^ 1),
            (sign, sign),
        ]
    }
    for (aw, bw, context) in [
        (1, 1, 65),
        (64, 64, 128),
        (1, 65, 0),
        (65, 1, 0),
        (65, 128, 0),
        (128, 65, 128),
    ] {
        for (ai, bi) in [(0, 0), (1, 0), (0, 1), (2, 3), (3, 3), (4, 5)] {
            for (planes, width) in [
                ((0, 0), 1),
                ((1, 0), 1),
                ((0, 1), 1),
                ((1, 1), 1),
                ((1, 2), 2),
                ((1, 1), 128),
                ((1u128 << 100, 0), 128),
                ((0, 1u128 << 100), 128),
            ] {
                for cs in [false, true] {
                    for as_ in [false, true] {
                        for bs in [false, true] {
                            let c = Operand {
                                planes,
                                width,
                                signed: cs,
                            };
                            let a = Operand {
                                planes: patterns(aw)[ai],
                                width: aw,
                                signed: as_,
                            };
                            let b = Operand {
                                planes: patterns(bw)[bi],
                                width: bw,
                                signed: bs,
                            };
                            for signed in [false, true] {
                                check(c, a, b, context, signed);
                            }
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn conditional_reference_scalar_truth_table() {
    use Logic4::{One as I, X, Z, Zero as O};
    let bits = [O, I, X, Z];
    let consensus = [[O, X, X, X], [X, I, X, X], [X, X, X, X], [X, X, X, X]];
    for condition in bits {
        for (i, a) in bits.into_iter().enumerate() {
            for (j, b) in bits.into_iter().enumerate() {
                let expected = match condition {
                    O => b,
                    I => a,
                    _ => consensus[i][j],
                };
                assert_eq!(merged_logic(condition, a, b), expected);
            }
        }
    }
}

#[test]
fn conditional_materializes_all_bit_sentinels() {
    for width in [1, 4, 64, 65, 128] {
        let mask = u128::MAX >> (128 - width);
        for (literal, expected) in [
            ("'0", (0, 0)),
            ("'1", (mask, 0)),
            ("'x", (0, mask)),
            ("'z", (mask, mask)),
        ] {
            let branch: Value = literal.parse().unwrap();
            for c in ["1'b0", "1'b1", "1'bx", "1'bz"] {
                let condition: Value = c.parse().unwrap();
                let result = condition.conditional(&branch, &branch, width, false);
                assert_eq!(
                    (result.payload_u128(), result.mask_xz_u128()),
                    if matches!(c, "1'bx" | "1'bz") && literal == "'z" {
                        (0, mask)
                    } else {
                        expected
                    },
                    "{c}, {literal}, width={width}"
                );
                assert_eq!(result.width(), width);
                assert!(!result.signed());
            }
        }
    }
}
