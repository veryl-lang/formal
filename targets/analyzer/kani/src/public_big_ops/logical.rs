//! IEEE 1800-2023 11.4.7/11.8.2: evaluated-value &&/|| contracts.
//! Normalized four-state operands have independent widths1..128 and signs;
//! at least one source is BigUint. All three storage pairs and result widths
//! 1..128 are covered. The caller sign is arbitrary and ignored by these ops.
//! Actual Op bodies run under numeric256 and separately proved expansion APIs.
//! Expansion observations check operand identity, zero extension at the common
//! source width, and independent unsigned result extension. Cache histories and
//! real numeric libraries remain finite integration comparisons. Allocation,
//! arbitrary ASTs, short-circuit evaluation, and side effects are outside scope.
use super::*;
use veryl_formal_semantics::gates::{self, BitOp};

// Set classification is justified against the scalar fold below, including
// zero padding. It is not a call to production's truth or logical operation.
fn truth(planes: (u128, u128)) -> Logic4 {
    if planes.0 & !planes.1 != 0 {
        Logic4::One
    } else if planes.1 != 0 {
        Logic4::X
    } else {
        Logic4::Zero
    }
}

#[kani::proof]
#[kani::unwind(129)]
fn logical_truth_and_zero_extension_match_scalar() {
    let width = usize::from(kani::any::<u8>() & 127) + 1;
    let target = usize::from(kani::any::<u8>() & 127) + 1;
    kani::assume(target >= width);
    let mask = u128::MAX >> (128 - width);
    let p = (kani::any::<u128>() & mask, kani::any::<u128>() & mask);
    let expected = truth(p);
    assert!(
        gates::logical_truth(p, width) == expected && gates::logical_truth(p, target) == expected,
        "wide logical truth mismatch"
    );
    kani::cover!(
        width == 1 && target == 128,
        "logical truth full zero extension is reachable"
    );
    kani::cover!(
        width == 65 && expected == Logic4::Zero,
        "logical truth wide zero is reachable"
    );
    kani::cover!(
        width == 128 && p == (1u128 << 127, 0),
        "logical truth upper one is reachable"
    );
    kani::cover!(
        width == 128 && p == (0, 1u128 << 127),
        "logical truth upper X is reachable"
    );
    kani::cover!(
        width == 128 && p == (1u128 << 127, 1u128 << 127),
        "logical truth upper Z is reachable"
    );
    kani::cover!(
        width == 127 && p == (1, 1u128 << 100),
        "logical truth one dominates unknown is reachable"
    );
}

static mut SOURCES: [*const Value; 2] = [std::ptr::null(); 2];
static mut COMMON: usize = 0;
static mut OUTPUT: usize = 0;
static mut CALLS: usize = 0;

fn observed_expand(value: &Value, width: usize, signed: bool) -> std::borrow::Cow<'_, Value> {
    unsafe {
        let call = CALLS;
        assert!(call < 3 && !signed, "wide operator semantics mismatch");
        if call < 2 {
            assert!(
                std::ptr::eq(value, SOURCES[call]) && width == COMMON,
                "wide operator semantics mismatch"
            );
        } else {
            assert!(
                value.width() == 1 && !value.signed() && width == OUTPUT,
                "wide operator semantics mismatch"
            );
        }
        CALLS += 1;
    }
    super::followup::expanded(value, width, signed)
}

fn logical<const AND: bool, const LAYOUT: u8>() {
    init();
    let left_width = usize::from(kani::any::<u8>() & 63) + if LAYOUT == 0 { 1 } else { 65 };
    let right_width = usize::from(kani::any::<u8>() & 63) + if LAYOUT == 1 { 1 } else { 65 };
    let output = usize::from(kani::any::<u8>() & 127) + 1;
    let (x, a) = input(left_width);
    let (y, b) = input(right_width);
    let caller_sign: bool = kani::any();
    unsafe {
        SOURCES = [&x, &y];
        COMMON = left_width.max(right_width);
        OUTPUT = output;
        CALLS = 0;
    }
    let op = if AND { Op::LogicAnd } else { Op::LogicOr };
    let result_value = op.eval_value_binary(&x, &y, output, caller_sign, &mut MaskCache::default());
    assert!(unsafe { CALLS } == 3, "wide operator semantics mismatch");
    let at = truth(a.planes);
    let bt = truth(b.planes);
    let expected = gates::binary(at, bt, if AND { BitOp::And } else { BitOp::Or });
    let planes = match expected {
        Logic4::Zero => (0, 0),
        Logic4::One => (1, 0),
        Logic4::X => (0, 1),
        Logic4::Z => unreachable!(),
    };
    assert!(
        (result_value.payload_u128(), result_value.mask_xz_u128()) == planes,
        "wide operator semantics mismatch"
    );
    result(&result_value, output, false, 0, expected);
    preserve(&x, a);
    preserve(&y, b);
    kani::cover!(expected == Logic4::Zero, "wide logical false is reachable");
    kani::cover!(expected == Logic4::One, "wide logical true is reachable");
    kani::cover!(expected == Logic4::X, "wide logical unknown is reachable");
    kani::cover!(
        at == Logic4::Zero && bt == Logic4::X,
        "wide logical zero with unknown is reachable"
    );
    kani::cover!(
        at == Logic4::One && bt == Logic4::X,
        "wide logical one with unknown is reachable"
    );
    kani::cover!(
        a.planes.0 & !a.planes.1 != 0 && a.planes.1 != 0,
        "wide logical known one with XZ is reachable"
    );
    kani::cover!(
        left_width == (if LAYOUT == 0 { 64 } else { 128 })
            && right_width == (if LAYOUT == 1 { 64 } else { 128 })
            && output == 1,
        "wide logical upper source widths narrow result is reachable"
    );
    kani::cover!(
        left_width == (if LAYOUT == 0 { 1 } else { 65 })
            && right_width == (if LAYOUT == 1 { 1 } else { 65 })
            && output == 128,
        "wide logical lower source widths wide result is reachable"
    );
    kani::cover!(
        a.signed && !b.signed && caller_sign,
        "wide logical independent signs are reachable"
    );
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, observed_expand)]
fn wide_logical_and_native_big_matches_scalar() {
    logical::<true, 0>();
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, observed_expand)]
fn wide_logical_and_big_native_matches_scalar() {
    logical::<true, 1>();
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, observed_expand)]
fn wide_logical_and_big_big_matches_scalar() {
    logical::<true, 2>();
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, observed_expand)]
fn wide_logical_or_native_big_matches_scalar() {
    logical::<false, 0>();
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, observed_expand)]
fn wide_logical_or_big_native_matches_scalar() {
    logical::<false, 1>();
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, observed_expand)]
fn wide_logical_or_big_big_matches_scalar() {
    logical::<false, 2>();
}
