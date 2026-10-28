//! Normalized native branches, resolved width/sign, and arbitrary four-state conditions;
//! Public dispatch adds fixed (3, 2)->5 and (64, 32)->64 layouts to one bit.
//! Their requested widths are 5 and 1 respectively; common Z follows Table 11-20.
//! No general context propagation or optimizer theorem follows.

use veryl_analyzer::value::Value;
use veryl_analyzer::value::ValueU64;
use veryl_formal_semantics::conditional::conditional_logic_bit;
use veryl_formal_semantics::gates::logical_truth;
use veryl_formal_semantics::logic4::{Logic4, read_logic};

fn operand() -> ValueU64 {
    let width = u32::from(kani::any::<u8>() & 63) + 1;
    let mask = u64::MAX >> (64 - width);
    ValueU64 {
        payload: kani::any::<u64>() & mask,
        mask_xz: kani::any::<u64>() & mask,
        width,
        signed: kani::any(),
    }
}

#[kani::proof]
#[kani::unwind(1)]
fn public_conditional_one_bit_matches_reference_semantics() {
    fn bit() -> ValueU64 {
        ValueU64 {
            payload: u64::from(kani::any::<bool>()),
            mask_xz: u64::from(kani::any::<bool>()),
            width: 1,
            signed: kani::any(),
        }
    }
    let c = bit();
    let a = bit();
    let b = bit();
    let signed: bool = kani::any();
    let condition = read_logic(c.payload as u128, c.mask_xz as u128, 0);
    let expected = conditional_logic_bit(
        condition,
        ((a.payload as u128, a.mask_xz as u128), 1, a.signed),
        ((b.payload as u128, b.mask_xz as u128), 1, b.signed),
        1,
        signed,
        0,
    );
    let common_z = a.payload == 1 && a.mask_xz == 1 && b.payload == 1 && b.mask_xz == 1;
    let output = Value::U64(c).conditional(&Value::U64(a), &Value::U64(b), 1, signed);
    let Value::U64(output) = output else {
        panic!("unexpected BigUint result");
    };
    assert!(
        read_logic(output.payload as u128, output.mask_xz as u128, 0) == expected,
        "public conditional semantics mismatch"
    );
    assert!(
        output.width == 1 && output.signed == signed && output.payload <= 1 && output.mask_xz <= 1,
        "public conditional semantics mismatch"
    );
    kani::cover!(
        condition == Logic4::Zero,
        "public condition zero is reachable"
    );
    kani::cover!(
        condition == Logic4::One,
        "public condition one is reachable"
    );
    kani::cover!(condition == Logic4::X, "public condition X is reachable");
    kani::cover!(condition == Logic4::Z, "public condition Z is reachable");
    kani::cover!(
        condition == Logic4::X && common_z && expected == Logic4::X,
        "public common Z becomes X"
    );
}

#[kani::proof]
#[kani::unwind(65)]
fn native_condition_truth_matches_reference_semantics() {
    let c = operand();
    let expected = logical_truth((c.payload as u128, c.mask_xz as u128), c.width as usize);
    let actual = c.truth();
    let decoded = match actual {
        Some(true) => Logic4::One,
        Some(false) => Logic4::Zero,
        None => Logic4::X,
    };
    assert!(decoded == expected, "condition truth semantics mismatch");
    kani::cover!(c.width == 1, "one-bit condition is reachable");
    kani::cover!(c.width == 64, "width-64 condition is reachable");
    kani::cover!(actual == Some(true), "true condition is reachable");
    kani::cover!(actual == Some(false), "false condition is reachable");
    kani::cover!(
        actual.is_none() && c.payload == 0,
        "ambiguous X condition is reachable"
    );
    kani::cover!(
        c.width == 1 && c.mask_xz == 1 && c.payload == 1 && actual.is_none(),
        "ambiguous Z condition is reachable"
    );
    kani::cover!(
        c.mask_xz != 0 && actual == Some(true),
        "condition known one dominates XZ"
    );
}

#[kani::proof]
#[kani::unwind(1)]
fn native_conditional_matches_reference_semantics() {
    let a = operand();
    let b = operand();
    let requested = usize::from(kani::any::<u8>() & 63) + 1;
    let signed: bool = kani::any();
    let condition: Option<bool> = kani::any();
    let width = requested.max(a.width as usize).max(b.width as usize);
    let position = usize::from(kani::any::<u8>() & 63);
    let x = a.expand(width, signed);
    let y = b.expand(width, signed);
    let output = a.conditional(&b, condition, requested, signed);
    let truth = match condition {
        Some(true) => Logic4::One,
        Some(false) => Logic4::Zero,
        None => Logic4::X,
    };
    let expected = conditional_logic_bit(
        truth,
        (
            (a.payload as u128, a.mask_xz as u128),
            a.width as usize,
            a.signed,
        ),
        (
            (b.payload as u128, b.mask_xz as u128),
            b.width as usize,
            b.signed,
        ),
        width,
        signed,
        position,
    );
    assert!(
        read_logic(output.payload as u128, output.mask_xz as u128, position) == expected,
        "conditional semantics mismatch"
    );
    assert!(
        output.width as usize == width && output.signed == signed,
        "conditional semantics mismatch"
    );
    let observed = position < width;
    let a_bit = read_logic(x.payload as u128, x.mask_xz as u128, position);
    let b_bit = read_logic(y.payload as u128, y.mask_xz as u128, position);
    kani::cover!(width == 1, "one-bit conditional is reachable");
    kani::cover!(width == 64, "width-64 conditional is reachable");
    kani::cover!(
        position >= width,
        "conditional upper stored bit is reachable"
    );
    kani::cover!(condition == Some(true), "true arm selection is reachable");
    kani::cover!(condition == Some(false), "false arm selection is reachable");
    kani::cover!(condition.is_none(), "ambiguous arm merge is reachable");
    kani::cover!(
        condition == Some(true) && expected == Logic4::Z,
        "selected true Z is reachable"
    );
    kani::cover!(
        condition == Some(false) && expected == Logic4::Z,
        "selected false Z is reachable"
    );
    kani::cover!(
        condition.is_none() && observed && a_bit == Logic4::Zero && b_bit == Logic4::Zero,
        "merged known zero is reachable"
    );
    kani::cover!(
        condition.is_none() && expected == Logic4::One,
        "merged known one is reachable"
    );
    kani::cover!(
        condition.is_none()
            && observed
            && a_bit == Logic4::Z
            && b_bit == Logic4::Z
            && expected == Logic4::X,
        "merged common Z becomes X"
    );
    kani::cover!(
        condition.is_none() && a_bit == Logic4::X && b_bit == Logic4::Z && expected == Logic4::X,
        "merged X versus Z is reachable"
    );
    kani::cover!(
        condition.is_none()
            && a_bit == Logic4::Zero
            && b_bit == Logic4::One
            && expected == Logic4::X,
        "merged disagreement X is reachable"
    );
    kani::cover!(
        condition == Some(true)
            && signed
            && a.signed
            && position >= a.width as usize
            && expected == Logic4::One,
        "conditional sign extension is reachable"
    );
    kani::cover!(
        condition == Some(true)
            && !signed
            && a.signed
            && (a.payload >> (a.width - 1)) & 1 == 1
            && position >= a.width as usize
            && observed
            && expected == Logic4::Zero,
        "conditional disabled sign extension is reachable"
    );
    kani::cover!(
        condition == Some(false) && a.width > b.width && requested < a.width as usize,
        "unselected arm determines width"
    );
}

// Fixed layouts avoid symbolic dispatch into the BigUint fallback;
// values, all four states, signedness, and the observed bit remain symbolic.
// W is the expected output width; R is the requested context width.
// This proves value dispatch with resolved sizing, not expression-tree traversal.
fn public_layout<const A: u32, const B: u32, const W: usize, const R: usize>() {
    fn input(width: u32) -> ValueU64 {
        let mask = u64::MAX >> (64 - width);
        ValueU64 {
            payload: kani::any::<u64>() & mask,
            mask_xz: kani::any::<u64>() & mask,
            width,
            signed: kani::any(),
        }
    }
    let c = input(1);
    let a = input(A);
    let b = input(B);
    let signed: bool = kani::any();
    let position = usize::from(kani::any::<u8>() & 63);
    let condition = read_logic(c.payload as u128, c.mask_xz as u128, 0);
    let expected = conditional_logic_bit(
        condition,
        ((a.payload as u128, a.mask_xz as u128), A as usize, a.signed),
        ((b.payload as u128, b.mask_xz as u128), B as usize, b.signed),
        W,
        signed,
        position,
    );
    let extended_signed = b.signed && signed;
    let output = Value::U64(c).conditional(&Value::U64(a), &Value::U64(b), R, signed);
    let Value::U64(output) = output else {
        panic!("unexpected BigUint result");
    };
    let mask = u64::MAX >> (64 - W);
    assert!(
        read_logic(output.payload as u128, output.mask_xz as u128, position) == expected
            && output.width as usize == W
            && output.signed == signed
            && output.payload & !mask == 0
            && output.mask_xz & !mask == 0,
        "public conditional layout semantics mismatch"
    );
    kani::cover!(
        condition == Logic4::X,
        "public layout condition X is reachable"
    );
    kani::cover!(
        condition == Logic4::Z,
        "public layout condition Z is reachable"
    );
    kani::cover!(
        condition == Logic4::Zero && extended_signed && position == W - 1,
        "public layout signed extension is reachable"
    );
    kani::cover!(
        condition == Logic4::Zero && !extended_signed && position == W - 1,
        "public layout unsigned extension is reachable"
    );
}

#[kani::proof]
#[kani::unwind(1)]
fn public_conditional_widened_matches_reference_semantics() {
    public_layout::<3, 2, 5, 5>();
}

#[kani::proof]
#[kani::unwind(1)]
fn public_conditional_word_matches_reference_semantics() {
    public_layout::<64, 32, 64, 1>();
}
