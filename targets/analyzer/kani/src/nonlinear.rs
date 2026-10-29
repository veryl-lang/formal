//! General multiply/divide/remainder use widths 1..=8; general power uses base widths
//! 1..=8 and exponent widths 1..=3. Full-native identity/exception proofs restrict
//! operand classes: multiplication RHS 0/1, division RHS 0/1 or signed all-ones, and
//! power unknown/negative/zero exponents. Public dispatch here is one bit;
//! arithmetic.rs also proves normalized small Mul/Div/Rem cache prefixes. These do not
//! establish arbitrary-input native division or arbitrary-width BigUint correctness.

use veryl_analyzer::ir::Op;
use veryl_analyzer::value::{MaskCache, Value, ValueU64};
use veryl_formal_semantics::nonlinear::{Nonlinear, planes};

#[kani::proof]
#[kani::unwind(3)]
fn public_nonlinear_one_bit_matches_reference_semantics() {
    let a = ValueU64 {
        payload: u64::from(kani::any::<bool>()),
        mask_xz: u64::from(kani::any::<bool>()),
        width: 1,
        signed: kani::any(),
    };
    let b = ValueU64 {
        payload: u64::from(kani::any::<bool>()),
        mask_xz: u64::from(kani::any::<bool>()),
        width: 1,
        signed: kani::any(),
    };
    let signed: bool = kani::any();
    let mode = kani::any::<u8>() & 3;
    let (op, kind) = match mode {
        0 => (Op::Mul, Nonlinear::Multiply),
        1 => (Op::Div, Nonlinear::Divide),
        2 => (Op::Rem, Nonlinear::Remainder),
        _ => (Op::Pow, Nonlinear::Power),
    };
    let expected = planes(
        (a.payload as u128, a.mask_xz as u128),
        (b.payload as u128, b.mask_xz as u128),
        1,
        if mode == 3 { a.signed } else { signed },
        1,
        b.signed,
        kind,
    );
    let result = op.eval_value_binary(
        &Value::U64(a.clone()),
        &Value::U64(b.clone()),
        1,
        signed,
        &mut MaskCache::default(),
    );
    assert!(
        (result.payload_u128(), result.mask_xz_u128()) == expected,
        "public nonlinear semantics mismatch"
    );
    assert!(
        result.width() == 1
            && result.signed()
                == if mode == 3 {
                    b.mask_xz == 0 && a.signed
                } else {
                    signed
                },
        "public nonlinear semantics mismatch"
    );
    kani::cover!(mode == 0, "public multiply is reachable");
    kani::cover!(mode == 1, "public divide is reachable");
    kani::cover!(mode == 2, "public remainder is reachable");
    kani::cover!(mode == 3, "public power is reachable");
    kani::cover!(
        mode == 3 && b.mask_xz != 0 && b.signed && b.payload == 1,
        "public unknown negative exponent is reachable"
    );
    kani::cover!(
        mode == 3 && a.payload == 0 && a.mask_xz == 0 && b.payload == 0 && b.mask_xz == 0,
        "public zero to zero is reachable"
    );
}

fn operand(width: u32) -> ValueU64 {
    let mask = u64::MAX >> (64 - width);
    ValueU64 {
        payload: kani::any::<u64>() & mask,
        mask_xz: kani::any::<u64>() & mask,
        width,
        signed: kani::any(),
    }
}

#[kani::proof]
#[kani::unwind(9)]
fn native_multiplication_matches_reference_semantics() {
    let width = u32::from(kani::any::<u8>() & 7) + 1;
    let a = operand(width);
    let b = operand(width);
    let signed: bool = kani::any();
    let output = a.multiply(&b, width as usize, signed);
    let expected = planes(
        (a.payload as u128, a.mask_xz as u128),
        (b.payload as u128, b.mask_xz as u128),
        width as usize,
        signed,
        width as usize,
        b.signed,
        Nonlinear::Multiply,
    );
    assert!(
        (output.payload as u128, output.mask_xz as u128) == expected
            && output.width == width
            && output.signed == signed,
        "multiplication semantics mismatch"
    );
    kani::cover!(width == 1, "one-bit multiplication is reachable");
    kani::cover!(width == 8, "eight-bit multiplication is reachable");
    kani::cover!(signed, "signed multiplication is reachable");
    kani::cover!(!signed, "unsigned multiplication is reachable");
    kani::cover!(
        a.mask_xz == 0 && b.mask_xz == 0 && a.payload == 0,
        "zero multiplication is reachable"
    );
    kani::cover!(
        a.mask_xz == 0 && b.mask_xz == 0 && b.payload == 1,
        "identity multiplication is reachable"
    );
    kani::cover!(
        width == 8 && a.mask_xz == 0 && b.mask_xz == 0 && a.payload == 128 && b.payload == 2,
        "multiplication carry truncation is reachable"
    );
    kani::cover!(
        a.mask_xz != 0 && b.mask_xz == 0,
        "multiplication unknown LHS is reachable"
    );
    kani::cover!(
        a.mask_xz == 0 && b.mask_xz != 0,
        "multiplication unknown RHS is reachable"
    );
    kani::cover!(
        (a.payload & a.mask_xz) != 0,
        "multiplication source Z is reachable"
    );
}

#[kani::proof]
#[kani::unwind(9)]
fn native_division_remainder_matches_reference_semantics() {
    let width = u32::from(kani::any::<u8>() & 7) + 1;
    let a = operand(width);
    let b = operand(width);
    let signed: bool = kani::any();
    let remainder: bool = kani::any();
    let output = a.divide_remainder(&b, width as usize, signed, remainder);
    let expected = planes(
        (a.payload as u128, a.mask_xz as u128),
        (b.payload as u128, b.mask_xz as u128),
        width as usize,
        signed,
        width as usize,
        b.signed,
        if remainder {
            Nonlinear::Remainder
        } else {
            Nonlinear::Divide
        },
    );
    assert!(
        (output.payload as u128, output.mask_xz as u128) == expected
            && output.width == width
            && output.signed == signed,
        "division remainder semantics mismatch"
    );
    kani::cover!(!remainder, "division is reachable");
    kani::cover!(remainder, "remainder is reachable");
    kani::cover!(width == 1, "one-bit division is reachable");
    kani::cover!(width == 8, "eight-bit division is reachable");
    kani::cover!(signed, "signed division is reachable");
    kani::cover!(!signed, "unsigned division is reachable");
    kani::cover!(
        b.payload == 0 && b.mask_xz == 0,
        "zero divisor is reachable"
    );
    kani::cover!(a.mask_xz != 0, "division unknown LHS is reachable");
    kani::cover!(b.mask_xz != 0, "division unknown RHS is reachable");
    kani::cover!(
        width == 8
            && signed
            && a.payload == 249
            && b.payload == 3
            && a.mask_xz == 0
            && b.mask_xz == 0
            && !remainder,
        "negative quotient is reachable"
    );
    kani::cover!(
        width == 8
            && signed
            && a.payload == 249
            && b.payload == 3
            && a.mask_xz == 0
            && b.mask_xz == 0
            && remainder,
        "negative remainder is reachable"
    );
    kani::cover!(
        width == 8
            && signed
            && a.payload == 128
            && b.payload == 255
            && a.mask_xz == 0
            && b.mask_xz == 0,
        "small minimum divided by minus one is reachable"
    );
}

#[kani::proof]
#[kani::unwind(9)]
fn native_power_matches_reference_semantics() {
    let width = u32::from(kani::any::<u8>() & 7) + 1;
    let ew = u32::from(kani::any::<u8>() % 3) + 1;
    let a = operand(width);
    let b = operand(ew);
    let output = a.power(&b, width as usize);
    let expected = veryl_formal_semantics::nonlinear::small_power_planes(
        (a.payload as u128, a.mask_xz as u128),
        (b.payload as u128, b.mask_xz as u128),
        width as usize,
        a.signed,
        ew as usize,
        b.signed,
    );
    assert!(
        (output.payload as u128, output.mask_xz as u128) == expected
            && output.width == width
            && output.signed == (b.mask_xz == 0 && a.signed),
        "power semantics mismatch"
    );
    kani::cover!(width == 1, "one-bit power is reachable");
    kani::cover!(width == 8, "eight-bit power is reachable");
    kani::cover!(ew == 1, "one-bit exponent is reachable");
    kani::cover!(ew == 3, "three-bit exponent is reachable");
    kani::cover!(
        b.payload == 0 && b.mask_xz == 0,
        "zero exponent is reachable"
    );
    kani::cover!(
        b.payload == 7 && !b.signed && b.mask_xz == 0,
        "positive exponent seven is reachable"
    );
    kani::cover!(
        b.signed && b.payload >> (ew - 1) & 1 != 0 && b.payload & 1 != 0 && b.mask_xz == 0,
        "negative odd exponent is reachable"
    );
    kani::cover!(
        b.signed && b.payload >> (ew - 1) & 1 != 0 && b.payload & 1 == 0 && b.mask_xz == 0,
        "negative even exponent is reachable"
    );
    kani::cover!(
        a.payload == 0
            && a.mask_xz == 0
            && b.signed
            && b.payload >> (ew - 1) & 1 != 0
            && b.mask_xz == 0,
        "zero to negative power is reachable"
    );
    kani::cover!(
        a.signed
            && a.payload == (u64::MAX >> (64 - width))
            && a.mask_xz == 0
            && b.signed
            && b.payload >> (ew - 1) & 1 != 0
            && b.payload & 1 != 0
            && b.mask_xz == 0,
        "minus one negative odd power is reachable"
    );
    kani::cover!(
        a.signed
            && a.payload == (u64::MAX >> (64 - width))
            && a.mask_xz == 0
            && b.signed
            && b.payload >> (ew - 1) & 1 != 0
            && b.payload & 1 == 0
            && b.mask_xz == 0,
        "minus one negative even power is reachable"
    );
    kani::cover!(
        a.mask_xz != 0 && a.payload == 0,
        "power source X is reachable"
    );
    kani::cover!((a.payload & a.mask_xz) != 0, "power source Z is reachable");
    kani::cover!(b.mask_xz != 0, "unknown exponent is reachable");
    kani::cover!(
        b.mask_xz != 0 && b.signed && b.payload >> (ew - 1) & 1 != 0,
        "unknown negative exponent is reachable"
    );
}

#[kani::proof]
#[kani::unwind(1)]
fn native_multiplication_identities_full_width() {
    let width = u32::from(kani::any::<u8>() & 63) + 1;
    let a = operand(width);
    let mut b = operand(width);
    b.payload = u64::from(kani::any::<bool>());
    let signed: bool = kani::any();
    let output = a.multiply(&b, width as usize, signed);
    let unknown = a.mask_xz != 0 || b.mask_xz != 0;
    let expected = if unknown {
        (0, u64::MAX >> (64 - width))
    } else {
        (if b.payload == 0 { 0 } else { a.payload }, 0)
    };
    assert!(
        (output.payload, output.mask_xz) == expected
            && output.width == width
            && output.signed == signed,
        "multiplication identity semantics mismatch"
    );
    kani::cover!(width == 64, "native identity width 64 is reachable");
    kani::cover!(
        a.mask_xz == 0 && b.mask_xz == 0 && b.payload == 0,
        "native multiplication by zero is reachable"
    );
    kani::cover!(
        a.mask_xz == 0 && b.mask_xz == 0 && b.payload == 1,
        "native multiplication by one is reachable"
    );
    kani::cover!(
        a.mask_xz != 0 && b.payload == 0 && b.mask_xz == 0,
        "unknown times zero is reachable"
    );
    kani::cover!(
        b.mask_xz != 0,
        "native multiplication RHS unknown is reachable"
    );
    kani::cover!(
        (a.payload & a.mask_xz) != 0,
        "native multiplication Z is reachable"
    );
    kani::cover!(signed, "native signed multiplication identity is reachable");
}

#[kani::proof]
#[kani::unwind(1)]
fn native_division_exceptional_full_width() {
    let width = u32::from(kani::any::<u8>() & 63) + 1;
    let a = operand(width);
    let b = operand(width);
    let signed: bool = kani::any();
    let remainder: bool = kani::any();
    let mask = u64::MAX >> (64 - width);
    kani::assume(b.payload == 0 || b.payload == 1 || (signed && b.payload == mask));
    let output = a.divide_remainder(&b, width as usize, signed, remainder);
    let unknown = a.mask_xz != 0 || b.mask_xz != 0 || b.payload == 0;
    let payload = if remainder {
        0
    } else if b.payload == 1 {
        a.payload
    } else {
        (((1u128 << width) - a.payload as u128) & mask as u128) as u64
    };
    let expected = if unknown { (0, mask) } else { (payload, 0) };
    assert!(
        (output.payload, output.mask_xz) == expected
            && output.width == width
            && output.signed == signed,
        "division exception semantics mismatch"
    );
    kani::cover!(width == 64, "native division width 64 is reachable");
    kani::cover!(
        b.payload == 0 && b.mask_xz == 0,
        "native zero divisor is reachable"
    );
    kani::cover!(
        b.payload == 1 && b.mask_xz == 0,
        "native divisor one is reachable"
    );
    kani::cover!(
        signed && b.payload == mask && b.mask_xz == 0,
        "native divisor minus one is reachable"
    );
    kani::cover!(
        width == 64
            && signed
            && a.payload == 1u64 << 63
            && b.payload == u64::MAX
            && !unknown
            && !remainder,
        "native division overflow is reachable"
    );
    kani::cover!(
        width == 64
            && signed
            && a.payload == 1u64 << 63
            && b.payload == u64::MAX
            && !unknown
            && remainder,
        "native remainder overflow is reachable"
    );
    kani::cover!(a.mask_xz != 0, "native division LHS unknown is reachable");
    kani::cover!(b.mask_xz != 0, "native division RHS unknown is reachable");
}

#[kani::proof]
#[kani::unwind(1)]
fn native_power_exceptional_full_width() {
    let width = u32::from(kani::any::<u8>() & 63) + 1;
    let ew = u32::from(kani::any::<u8>() & 63) + 1;
    let a = operand(width);
    let b = operand(ew);
    let negative = b.signed && b.payload >> (ew - 1) & 1 != 0;
    kani::assume(b.mask_xz != 0 || negative || b.payload == 0);
    let output = a.power(&b, width as usize);
    let expected = veryl_formal_semantics::nonlinear::power_exception(
        (a.payload as u128, a.mask_xz as u128),
        (b.payload as u128, b.mask_xz as u128),
        width as usize,
        a.signed,
        ew as usize,
        b.signed,
    );
    assert!(
        (output.payload as u128, output.mask_xz as u128) == expected
            && output.width == width
            && output.signed == (b.mask_xz == 0 && a.signed),
        "power exception semantics mismatch"
    );
    kani::cover!(width == 64, "native power width 64 is reachable");
    kani::cover!(ew == 64, "native exponent width 64 is reachable");
    kani::cover!(
        a.payload == 0 && a.mask_xz == 0 && b.payload == 0 && b.mask_xz == 0,
        "native zero to zero is reachable"
    );
    kani::cover!(
        a.payload == 0 && a.mask_xz == 0 && negative && b.mask_xz == 0,
        "native zero negative power is reachable"
    );
    kani::cover!(
        a.payload == 1 && a.mask_xz == 0 && negative && b.mask_xz == 0,
        "native one negative power is reachable"
    );
    kani::cover!(
        a.signed
            && a.payload == (u64::MAX >> (64 - width))
            && a.mask_xz == 0
            && negative
            && b.payload & 1 != 0
            && b.mask_xz == 0,
        "native minus one odd power is reachable"
    );
    kani::cover!(
        a.signed
            && a.payload == (u64::MAX >> (64 - width))
            && a.mask_xz == 0
            && negative
            && b.payload & 1 == 0
            && b.mask_xz == 0,
        "native minus one even power is reachable"
    );
    kani::cover!(
        width == 64 && a.payload == 2 && a.mask_xz == 0 && negative && b.mask_xz == 0,
        "native other negative power is reachable"
    );
    kani::cover!(a.mask_xz != 0, "native unknown base is reachable");
    kani::cover!(b.mask_xz != 0, "native unknown exponent is reachable");
    kani::cover!(
        ew == 64 && b.signed && b.payload == 1u64 << 63 && b.mask_xz == 0,
        "native minimum exponent is reachable"
    );
    kani::cover!(
        negative && b.mask_xz != 0,
        "native unknown negative exponent is reachable"
    );
}
