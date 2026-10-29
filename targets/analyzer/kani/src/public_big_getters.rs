//! Public getter projection for canonical BigUint planes bounded by 128 bits.
//! Both planes, metadata widths 65..=128 and signs vary independently. Data may
//! occupy bits above the declared width: these getters return raw low 128 bits.
//! Input packing is checked bitwise against independent u128 inputs before the
//! public getters are checked. No production conversion is used as an oracle.
//! Unwind 3 covers construction/collection of at most two u64 digits, with all
//! safety and unwinding assertions enabled. This does not prove a producer's
//! arithmetic or resizing behavior.
use veryl_analyzer::value::{Value, ValueBigUint};
#[kani::proof]
#[kani::unwind(3)]
fn public_big_getters_match_reference_bits() {
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
        big.payload.bits() <= 128 && big.mask_xz.bits() <= 128,
        "public BigUint getter semantics mismatch"
    );
    assert!(
        big.payload.bit(position as u64) == (payload >> position & 1 != 0)
            && big.mask_xz.bit(position as u64) == (xz >> position & 1 != 0),
        "public BigUint getter semantics mismatch"
    );
    let value = Value::BigUint(big);
    assert!(
        value.payload_u128() == payload
            && value.mask_xz_u128() == xz
            && value.width() == width as usize
            && value.signed() == signed,
        "public BigUint getter semantics mismatch"
    );
    kani::cover!(
        payload == 0 && xz == 0 && width == 65 && !signed,
        "public BigUint getters empty digits are reachable"
    );
    kani::cover!(
        payload == 1u128 << 127 && xz == 0 && width == 128 && signed,
        "public BigUint getters high one is reachable"
    );
    kani::cover!(
        payload == 0 && xz == 1u128 << 127 && width == 128 && !signed,
        "public BigUint getters high X is reachable"
    );
    kani::cover!(
        payload == u128::MAX && xz == u128::MAX && width == 127 && signed,
        "public BigUint getters high Z and padding are reachable"
    );
}
