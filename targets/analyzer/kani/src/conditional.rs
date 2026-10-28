//! Normalized native branches, resolved width/sign, and arbitrary four-state conditions;
//! Public small dispatch covers widths 1..=2 and requests 0,1,4; wider fixed
//! (3,2)->5 and (64,32)->64 layouts request widths 5 and 1 respectively.
//! Common Z follows Table 11-20.
//! No general context propagation or optimizer theorem follows.

use veryl_analyzer::value::Value;
use veryl_analyzer::value::ValueU64;
use veryl_formal_semantics::comparison::Operand;
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

// Each fixed layout admits all normalized four-state values and signs. The
// complete family covers conditions/branches of widths 1..=2 and requests 0,1,4.
// Construct through the public API and check its inputs before and after use.
fn check_small_input(value: &Value, expected: Operand) {
    assert!(
        matches!(value, Value::U64(_))
            && value.payload_u128() == expected.planes.0
            && value.mask_xz_u128() == expected.planes.1
            && value.width() == expected.width
            && value.signed() == expected.signed,
        "public small conditional semantics mismatch"
    );
}

fn small_input<const W: usize>() -> (Value, Operand) {
    let mask = (1u128 << W) - 1;
    let expected = Operand {
        planes: (
            u128::from(kani::any::<u8>()) & mask,
            u128::from(kani::any::<u8>()) & mask,
        ),
        width: W,
        signed: kani::any(),
    };
    let actual = Value::from_u128(expected.planes.0, expected.planes.1, W, expected.signed);
    check_small_input(&actual, expected);
    (actual, expected)
}

struct SmallConditionalWitness {
    kind: Logic4,
    common_z: bool,
    known_dominance: bool,
    signed_extension: bool,
    sign_suppression: bool,
    upper_padding: bool,
}

fn small_layout<const C: usize, const A: usize, const B: usize, const R: usize>()
-> SmallConditionalWitness {
    let (c, cr) = small_input::<C>();
    let (a, ar) = small_input::<A>();
    let (b, br) = small_input::<B>();
    let signed: bool = kani::any();
    let position = usize::from(kani::any::<u8>() & 127);
    let width = R.max(A).max(B);
    let condition = logical_truth(cr.planes, C);
    let expected = conditional_logic_bit(
        condition,
        (ar.planes, A, ar.signed),
        (br.planes, B, br.signed),
        width,
        signed,
        position,
    );
    let result = c.conditional(&a, &b, R, signed);
    assert!(
        matches!(result, Value::U64(_))
            && result.width() == width
            && result.signed() == signed
            && read_logic(result.payload_u128(), result.mask_xz_u128(), position) == expected,
        "public small conditional semantics mismatch"
    );
    check_small_input(&c, cr);
    check_small_input(&a, ar);
    check_small_input(&b, br);
    SmallConditionalWitness {
        // Preserve the raw one-bit Z case as well as decoded multi-bit truth.
        kind: if C == 1 {
            read_logic(cr.planes.0, cr.planes.1, 0)
        } else {
            condition
        },
        common_z: condition == Logic4::X
            && position == 0
            && ar.planes.0 & 1 == 1
            && ar.planes.1 & 1 == 1
            && br.planes.0 & 1 == 1
            && br.planes.1 & 1 == 1
            && expected == Logic4::X,
        known_dominance: condition == Logic4::One && cr.planes.1 != 0,
        signed_extension: condition == Logic4::One
            && signed
            && ar.signed
            && A < width
            && position == width - 1,
        sign_suppression: condition == Logic4::One
            && !signed
            && ar.signed
            && A < width
            && position == width - 1
            && expected == Logic4::Zero,
        upper_padding: position == 127 && expected == Logic4::Zero,
    }
}

macro_rules! small_conditional_proof {
    ($name:ident, $a:literal, $b:literal) => {
        #[kani::proof]
        #[kani::unwind(3)]
        fn $name() {
            let two_bit: bool = kani::any();
            let context = match kani::any::<u8>() & 3 {
                0 => 0,
                1 => 1,
                _ => 4,
            };
            let witness = match (two_bit, context) {
                (false, 0) => small_layout::<1, $a, $b, 0>(),
                (false, 1) => small_layout::<1, $a, $b, 1>(),
                (false, 4) => small_layout::<1, $a, $b, 4>(),
                (true, 0) => small_layout::<2, $a, $b, 0>(),
                (true, 1) => small_layout::<2, $a, $b, 1>(),
                (true, 4) => small_layout::<2, $a, $b, 4>(),
                _ => unreachable!(),
            };
            kani::cover!(
                witness.kind == Logic4::Zero,
                "public condition zero is reachable"
            );
            kani::cover!(
                witness.kind == Logic4::One,
                "public condition one is reachable"
            );
            kani::cover!(witness.kind == Logic4::X, "public condition X is reachable");
            kani::cover!(witness.kind == Logic4::Z, "public condition Z is reachable");
            kani::cover!(
                witness.kind == Logic4::X && witness.common_z,
                "public common Z becomes X"
            );
            kani::cover!(!two_bit, "small conditional one-bit condition is reachable");
            kani::cover!(two_bit, "small conditional two-bit condition is reachable");
            kani::cover!(context == 0, "small conditional zero context is reachable");
            kani::cover!(context == 1, "small conditional unit context is reachable");
            kani::cover!(context == 4, "small conditional wide context is reachable");
            kani::cover!(
                witness.known_dominance,
                "small conditional known one dominates unknown"
            );
            kani::cover!(
                witness.signed_extension,
                "small conditional signed extension is reachable"
            );
            kani::cover!(
                witness.sign_suppression,
                "small conditional sign suppression is reachable"
            );
            kani::cover!(
                witness.upper_padding,
                "small conditional u128 padding is reachable"
            );
        }
    };
}

small_conditional_proof!(
    public_conditional_small_11_matches_reference_semantics,
    1,
    1
);
small_conditional_proof!(
    public_conditional_small_12_matches_reference_semantics,
    1,
    2
);
small_conditional_proof!(
    public_conditional_small_21_matches_reference_semantics,
    2,
    1
);
small_conditional_proof!(
    public_conditional_small_22_matches_reference_semantics,
    2,
    2
);
