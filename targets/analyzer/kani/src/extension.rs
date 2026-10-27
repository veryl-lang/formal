//! Arbitrary clean 64-bit multiplication compared with a widened u128 product modulo
//! 64 bits. This relies on verifier primitive arithmetic; it is not a theorem about
//! the primitives themselves and does not cover arbitrary-input division/remainder.

use veryl_analyzer::value::ValueU64;
#[kani::proof]
fn native_full_multiply_widened() {
    let a: u64 = kani::any();
    let b: u64 = kani::any();
    let signed: bool = kani::any();
    let lhs = ValueU64 {
        payload: a,
        mask_xz: 0,
        width: 64,
        signed,
    };
    let rhs = ValueU64 {
        payload: b,
        mask_xz: 0,
        width: 64,
        signed,
    };
    let got = lhs.multiply(&rhs, 64, signed);
    let expected = (a as u128 * b as u128) as u64;
    assert!(
        got.payload == expected && got.mask_xz == 0 && got.width == 64 && got.signed == signed,
        "full native nonlinear output mismatch"
    );
    kani::cover!(
        a >> 63 != 0 && b >> 63 != 0,
        "both full native operands use high bits"
    );
    kani::cover!(
        (a as u128 * b as u128) > u64::MAX as u128,
        "full native multiply truncates overflow"
    );
}
