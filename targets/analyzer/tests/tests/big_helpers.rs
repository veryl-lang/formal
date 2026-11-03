//! Numeric boundary comparisons retained alongside the compositional Kani proofs.
//! Masks enumerate every input in the substituted helper's domain. Constructors
//! cover finite boundary patterns, not arbitrary u128 values. Operation proofs
//! start with canonical BigUint values and do not subsume these constructor checks.
use veryl_analyzer::value::{Value, ValueBigUint};

#[test]
fn biguint_masks_all_supported_widths() {
    for width in 0..=128 {
        let actual = ValueBigUint::gen_mask(width);
        assert!(
            actual.bits() <= width as u64,
            "public BigUint helper semantics mismatch"
        );
        // This scalar definition does not repeat the production word-packing loop
        // or the closed-form u128 expression used by the Kani replacement.
        for position in 0..=128 {
            assert!(
                actual.bit(position as u64) == (position < width),
                "public BigUint helper semantics mismatch"
            );
        }
    }
}

#[test]
fn biguint_public_constructor_boundaries() {
    for width in 65..=128 {
        let mask = u128::MAX >> (128 - width);
        let sign = 1u128 << (width - 1);
        // Include every BigUint constructor pattern removed from conditional and
        // selection comparisons, plus all intermediate widths and sign states.
        for (p, x) in [
            (0, 0),
            (mask, 0),
            (0, mask),
            (mask, mask),
            (sign, 0),
            (0, sign),
            (sign, sign),
            (1, mask ^ 1),
            (1, 1),
            (1, 1u128 << 64),
            (1u128 << 100, 0),
            (0, 1u128 << 100),
            (1u128 << 100, 1u128 << 100),
            (
                0xaaaa_aaaa_aaaa_aaaa_aaaa_aaaa_aaaa_aaaa,
                0x5555_5555_5555_5555_5555_5555_5555_5555,
            ),
        ] {
            let (p, x) = (p & mask, x & mask);
            for signed in [false, true] {
                let input = Value::from_u128(p, x, width, signed);
                assert!(
                    input.width() == width
                        && input.signed() == signed
                        && input.payload_u128() == p
                        && input.mask_xz_u128() == x,
                    "public BigUint helper semantics mismatch"
                );
                let expected_truth =
                    match veryl_formal_semantics::gates::logical_truth((p, x), width) {
                        veryl_formal_semantics::logic4::Logic4::One => Some(true),
                        veryl_formal_semantics::logic4::Logic4::Zero => Some(false),
                        _ => None,
                    };
                assert!(
                    input.truth() == expected_truth,
                    "public BigUint helper semantics mismatch"
                );
                let Value::BigUint(big) = &input else {
                    panic!("public BigUint helper semantics mismatch");
                };
                assert!(
                    big.payload.bits() <= width as u64 && big.mask_xz.bits() <= width as u64,
                    "public BigUint helper semantics mismatch"
                );
                for position in 0..128 {
                    assert!(
                        big.payload.bit(position) == (p >> position & 1 != 0)
                            && big.mask_xz.bit(position) == (x >> position & 1 != 0),
                        "public BigUint helper semantics mismatch"
                    );
                }
                // Finite real-library bridge for intermediates up to bit255.
                // Width128 shifted by128 must retain the high half before clipping.
                for (plane, number) in [(p, &big.payload), (x, &big.mask_xz)] {
                    let shifted = number.as_ref() << width;
                    assert!(
                        shifted.bits() <= 2 * width as u64,
                        "public BigUint helper semantics mismatch"
                    );
                    for position in 0..256 {
                        let expected = position >= width
                            && position - width < 128
                            && (plane >> (position - width)) & 1 != 0;
                        assert!(
                            shifted.bit(position as u64) == expected,
                            "public BigUint helper semantics mismatch"
                        );
                    }
                    assert!(
                        (&shifted >> width) == **number,
                        "public BigUint helper semantics mismatch"
                    );
                }
                assert!(
                    input.clone() == input,
                    "public BigUint helper semantics mismatch"
                );
            }
        }
    }
}

// Finite exhaustive two-key histories over the supported cache key domain,
// plus a full-table traversal. This does not quantify arbitrary HashMap states.
#[test]
fn biguint_mask_cache_width_histories() {
    fn check(cache: &mut veryl_analyzer::value::MaskCache, width: usize) {
        let mask = cache.get(width);
        assert!(
            mask.bits() == width as u64,
            "public BigUint helper semantics mismatch"
        );
        for p in 0..=128 {
            assert!(
                mask.bit(p) == (p < width as u64),
                "public BigUint helper semantics mismatch"
            );
        }
    }
    for a in 0..=128 {
        for b in 0..=128 {
            let mut cache = veryl_analyzer::value::MaskCache::default();
            check(&mut cache, a);
            check(&mut cache, b);
            check(&mut cache, a);
        }
    }
    let mut cache = veryl_analyzer::value::MaskCache::default();
    for width in 0..=128 {
        check(&mut cache, width);
    }
    for width in (0..=128).rev() {
        check(&mut cache, width);
    }
}

// Finite bridge to actual BigUint addition, including the carry beyond the
// bounded256-bit model. The oracle is a scalar carry chain over native halves.
#[test]
fn biguint_addition_word_boundaries() {
    fn number(n: u128) -> ValueBigUint {
        let Value::BigUint(v) = Value::from_u128(n, 0, 128, false) else {
            panic!("public BigUint helper semantics mismatch")
        };
        v
    }
    let words = [
        0u128,
        1,
        u128::MAX,
        1u128 << 127,
        0xaaaa_aaaa_aaaa_aaaa_aaaa_aaaa_aaaa_aaaa,
    ];
    for al in words {
        for ah in words {
            for bl in words {
                for bh in words {
                    let a = number(al).payload.as_ref() | (number(ah).payload.as_ref() << 128usize);
                    let b = number(bl).payload.as_ref() | (number(bh).payload.as_ref() << 128usize);
                    let sum = &a + &b;
                    let mut carry = false;
                    for p in 0..256 {
                        let x = if p < 128 {
                            al >> p & 1 != 0
                        } else {
                            ah >> (p - 128) & 1 != 0
                        };
                        let y = if p < 128 {
                            bl >> p & 1 != 0
                        } else {
                            bh >> (p - 128) & 1 != 0
                        };
                        let expected = x ^ y ^ carry;
                        let next = (x && y) || (x && carry) || (y && carry);
                        assert!(
                            sum.bit(p as u64) == expected,
                            "public BigUint helper semantics mismatch"
                        );
                        carry = next;
                    }
                    assert!(
                        sum.bit(256) == carry && sum.bits() <= 257,
                        "public BigUint helper semantics mismatch"
                    );
                }
            }
        }
    }
}
