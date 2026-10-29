//! Normalized native branches, resolved width/sign, and arbitrary four-state conditions;
//! Public dispatch covers condition widths 1..=2, branch widths 1..=64 and
//! context requests 0..=64, including constructor/getter/input-preservation checks.
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

// BigUint conversion/cloning/masks, heap allocation, and mask-cache access are
// forbidden in these small native proofs. Each replacement panics, proving it unreachable
// for every admitted input. Conditional selection, expansion, constructors, and getters run
// unchanged. Kani 0.68 otherwise expands unreachable BigUint paths before solving.
fn reject_conditional_biguint(_: u128) -> num_bigint::BigUint {
    panic!("public conditional semantics mismatch");
}
fn reject_conditional_cache(
    _: &mut veryl_analyzer::value::MaskCache,
    _: usize,
) -> &num_bigint::BigUint {
    panic!("public conditional semantics mismatch");
}
fn reject_conditional_from_u64(_: u64) -> num_bigint::BigUint {
    panic!("public conditional semantics mismatch");
}
fn reject_conditional_clone(
    _: &veryl_analyzer::value::ValueBigUint,
) -> veryl_analyzer::value::ValueBigUint {
    panic!("public conditional semantics mismatch");
}
fn reject_conditional_mask(_: usize) -> num_bigint::BigUint {
    panic!("public conditional semantics mismatch");
}
unsafe fn reject_conditional_allocation(_: std::alloc::Layout) -> *mut u8 {
    panic!("public conditional semantics mismatch");
}

kani::stub_set!(
    small_native_conditional,
    stub(
        veryl_analyzer::value::u128_to_biguint,
        reject_conditional_biguint
    ),
    stub(
        veryl_analyzer::value::MaskCache::get,
        reject_conditional_cache
    ),
    stub(
        <num_bigint::BigUint as std::convert::From<u64>>::from,
        reject_conditional_from_u64
    ),
    stub(
        <veryl_analyzer::value::ValueBigUint as std::clone::Clone>::clone,
        reject_conditional_clone
    ),
    stub(
        veryl_analyzer::value::ValueBigUint::gen_mask,
        reject_conditional_mask
    ),
    stub(std::alloc::alloc, reject_conditional_allocation),
    stub(std::alloc::alloc_zeroed, reject_conditional_allocation),
);

fn public_native_conditional_input(width: usize) -> (Value, Operand) {
    let mask = u64::MAX >> (64 - width);
    let before = Operand {
        planes: (
            u128::from(kani::any::<u64>() & mask),
            u128::from(kani::any::<u64>() & mask),
        ),
        width,
        signed: kani::any(),
    };
    let input = Value::from_u128(before.planes.0, before.planes.1, width, before.signed);
    check_native_conditional_input(&input, before);
    let Value::U64(native) = input else {
        panic!("public conditional semantics mismatch");
    };
    (Value::U64(native), before)
}

fn check_native_conditional_input(input: &Value, before: Operand) {
    assert!(
        matches!(input, Value::U64(_))
            && input.width() == before.width
            && input.signed() == before.signed
            && input.payload_u128() == before.planes.0
            && input.mask_xz_u128() == before.planes.1,
        "public conditional semantics mismatch"
    );
}

// Public construction, truth decoding and conditional selection. Conditions have
// every normalized four-state value of widths 1..=2, including known-one with X/Z.
// Branch widths span 1..=64 independently; requests 0..=64 and all signs are free.
// IEEE 1800 11.4.11/Table 11-20 and 11.8.2 are interpreted by independent models.
#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(small_native_conditional)]
fn public_conditional_native_matches_reference_semantics() {
    let cw = usize::from(kani::any::<u8>() & 1) + 1;
    let aw = usize::from(kani::any::<u8>() & 63) + 1;
    let bw = usize::from(kani::any::<u8>() & 63) + 1;
    let context = usize::from(kani::any::<u8>() & 127).min(64);
    let signed: bool = kani::any();
    let position = usize::from(kani::any::<u8>() & 127);
    let (c, cr) = public_native_conditional_input(cw);
    let (a, ar) = public_native_conditional_input(aw);
    let (b, br) = public_native_conditional_input(bw);
    let width = context.max(aw).max(bw);
    let truth = logical_truth(cr.planes, cw);
    let expected = conditional_logic_bit(
        truth,
        (ar.planes, aw, ar.signed),
        (br.planes, bw, br.signed),
        width,
        signed,
        position,
    );
    let result = c.conditional(&a, &b, context, signed);
    let Value::U64(native) = result else {
        panic!("public conditional semantics mismatch");
    };
    let mask = u64::MAX >> (64 - width);
    assert!(
        native.payload & !mask == 0 && native.mask_xz & !mask == 0,
        "public conditional semantics mismatch"
    );
    let result = Value::U64(native);
    assert!(
        result.width() == width
            && result.signed() == signed
            && read_logic(result.payload_u128(), result.mask_xz_u128(), position) == expected,
        "public conditional semantics mismatch"
    );
    check_native_conditional_input(&c, cr);
    check_native_conditional_input(&a, ar);
    check_native_conditional_input(&b, br);
    kani::cover!(
        cw == 1,
        "public native conditional condition width 1 is reachable"
    );
    kani::cover!(
        cw == 2,
        "public native conditional condition width 2 is reachable"
    );
    kani::cover!(
        aw == 1,
        "public native conditional true width 1 is reachable"
    );
    kani::cover!(
        aw == 64,
        "public native conditional true width 64 is reachable"
    );
    kani::cover!(
        bw == 1,
        "public native conditional false width 1 is reachable"
    );
    kani::cover!(
        bw == 64,
        "public native conditional false width 64 is reachable"
    );
    kani::cover!(
        context == 0,
        "public native conditional context 0 is reachable"
    );
    kani::cover!(
        context == 1,
        "public native conditional context 1 is reachable"
    );
    kani::cover!(
        context == 64,
        "public native conditional context 64 is reachable"
    );
    kani::cover!(
        truth == Logic4::Zero,
        "public native conditional false condition is reachable"
    );
    kani::cover!(
        truth == Logic4::One,
        "public native conditional true condition is reachable"
    );
    kani::cover!(
        truth == Logic4::X,
        "public native conditional ambiguous condition is reachable"
    );
    kani::cover!(
        cr.planes.1 != 0 && truth == Logic4::One,
        "public native conditional known one dominates unknown"
    );
    kani::cover!(
        truth == Logic4::X
            && position < aw
            && position < bw
            && read_logic(ar.planes.0, ar.planes.1, position) == Logic4::Z
            && read_logic(br.planes.0, br.planes.1, position) == Logic4::Z
            && expected == Logic4::X,
        "public native conditional common Z becomes X"
    );
    kani::cover!(
        context > aw.max(bw),
        "public native conditional context widens branches"
    );
    kani::cover!(
        context < aw.max(bw),
        "public native conditional context preserves branch width"
    );
    kani::cover!(
        truth == Logic4::One
            && ar.signed
            && signed
            && position >= aw
            && position < width
            && expected == Logic4::One,
        "public native conditional sign extension is reachable"
    );
    kani::cover!(
        truth == Logic4::Zero && br.signed && !signed && position >= bw && position < width,
        "public native conditional sign suppression is reachable"
    );
    kani::cover!(
        signed,
        "public native conditional signed result is reachable"
    );
    kani::cover!(
        !signed,
        "public native conditional unsigned result is reachable"
    );
    kani::cover!(
        position == 127,
        "public native conditional u128 padding is reachable"
    );
}
