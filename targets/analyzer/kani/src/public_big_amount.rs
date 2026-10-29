//! Unsigned count decoding for independent canonical BigUint planes up to 128
//! bits, metadata widths 65..=128, and either sign. Raw data may exceed metadata
//! width: any X/Z yields None, otherwise the magnitude saturates at usize::MAX.
//! Input packing is checked against independent u128 bits; decoding, allocation
//! and getters execute unchanged. This is an API-value contract, not a proof of
//! the Veryl u128-to-BigUint constructor or a complete shift call.
//! IEEE 1800 11.4.10 interprets shift counts as unsigned. Unwind 3 is checked.
#[kani::proof]
#[kani::unwind(3)]
fn public_big_amount_decoding_matches_reference() {
    use veryl_analyzer::value::{Value, ValueBigUint};
    let payload: u128 = kani::any();
    let xz: u128 = kani::any();
    let width = u32::from(kani::any::<u8>() & 63) + 65;
    let signed: bool = kani::any();
    let big = ValueBigUint {
        payload: Box::new(num_bigint::BigUint::from(payload)),
        mask_xz: Box::new(num_bigint::BigUint::from(xz)),
        width,
        signed,
    };
    let position = usize::from(kani::any::<u8>() & 127);
    assert!(
        big.payload.bits() <= 128
            && big.mask_xz.bits() <= 128
            && big.payload.bit(position as u64) == (payload >> position & 1 != 0)
            && big.mask_xz.bit(position as u64) == (xz >> position & 1 != 0),
        "public BigUint amount semantics mismatch"
    );
    let value = Value::BigUint(big);
    let expected = if xz != 0 {
        None
    } else {
        Some(usize::try_from(payload).unwrap_or(usize::MAX))
    };
    assert!(
        value.to_usize_saturating() == expected,
        "public BigUint amount semantics mismatch"
    );
    assert!(
        value.width() == width as usize
            && value.signed() == signed
            && value.payload_u128() == payload
            && value.mask_xz_u128() == xz,
        "public BigUint amount semantics mismatch"
    );
    kani::cover!(
        payload == 0 && xz == 0 && width == 65 && !signed,
        "public BigUint amount zero is reachable"
    );
    kani::cover!(
        payload == usize::MAX as u128 && xz == 0 && width == 128 && signed,
        "public BigUint amount usize maximum is reachable"
    );
    kani::cover!(
        payload == 1u128 << 100 && xz == 0 && width == 128 && signed,
        "public BigUint oversized amount is reachable"
    );
    kani::cover!(
        payload == 0 && xz == 1u128 << 100 && width == 128 && !signed,
        "public BigUint upper X amount is reachable"
    );
    kani::cover!(
        payload == 1u128 << 100 && xz == 1u128 << 100 && width == 128 && signed,
        "public BigUint upper Z amount is reachable"
    );
}
