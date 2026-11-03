//! Bounded unary-plus composition under the SV-UPLUS identity interpretation.
//! IEEE 1800-2023 11.4.3/Table11-6 and11.8.2: preserve normalized four-state
//! planes after width/sign preparation. The precedence of Table11-6's identity
//! and the general arithmetic X/Z rule remains unresolved as documented in
//! semantics/arithmetic.rs. These are interpretation-equivalence proofs, not
//! an unqualified standard-conformance claim.
//! Prepared operands are65..128 bits. Original sources are1..128 bits with
//! independent signs and nonnarrowing requested widths65..128. Numeric APIs,
//! cloning and expansion rely on their separate contracts. Real-library
//! boundaries, sentinel literals, and interpretation anchors remain direct.
use super::*;

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, super::prepared_wide)]
fn wide_unary_plus_preserves_prepared_planes() {
    init();
    let width = usize::from(kani::any::<u8>() & 63) + 65;
    let (x, a) = super::followup::wide_input(width);
    let caller_sign: bool = kani::any();
    let position = usize::from(kani::any::<u8>() & 127);
    let ret = Op::Add.eval_value_unary(&x, width, caller_sign, &mut MaskCache::default());
    result(
        &ret,
        width,
        a.signed,
        position,
        read_logic(a.planes.0, a.planes.1, position),
    );
    assert!(
        (ret.payload_u128(), ret.mask_xz_u128()) == a.planes,
        "wide operator semantics mismatch"
    );
    preserve(&x, a);
    kani::cover!(
        width == 65 && a.planes == (0, 0),
        "wide plus minimum zero is reachable"
    );
    kani::cover!(
        width == 128 && a.planes == (1u128 << 127, 0),
        "wide plus upper one is reachable"
    );
    kani::cover!(
        width == 128 && a.planes == (0, 1u128 << 127),
        "wide plus upper X is reachable"
    );
    kani::cover!(
        width == 128 && a.planes == (1u128 << 127, 1u128 << 127),
        "wide plus upper Z is reachable"
    );
    kani::cover!(
        a.planes.1 != 0 && a.planes.1 != u128::MAX >> (128 - width),
        "wide plus partial unknown is reachable"
    );
    kani::cover!(
        a.signed && !caller_sign,
        "wide plus equal width retains sign is reachable"
    );
}

static mut SOURCE: *const Value = std::ptr::null();
static mut TARGET: usize = 0;
static mut SIGN: bool = false;
static mut CALLS: usize = 0;
fn observed_expand(value: &Value, width: usize, signed: bool) -> std::borrow::Cow<'_, Value> {
    unsafe {
        assert!(
            CALLS == 0 && std::ptr::eq(value, SOURCE) && width == TARGET && signed == SIGN,
            "wide operator semantics mismatch"
        );
        CALLS += 1;
    }
    // Execute the separately proved summary, including values, rather than
    // replacing numeric reads with arbitrary wiring-only values.
    prepared_wide(value, width, signed)
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, observed_expand)]
fn wide_unary_plus_original_preparation_matches_scalar() {
    use veryl_formal_semantics::resize::extended_logic_bit;
    init();
    let source = usize::from(kani::any::<u8>() & 127) + 1;
    let width = usize::from(kani::any::<u8>() & 63) + 65;
    kani::assume(source <= width);
    let (x, a) = input(source);
    let caller_sign: bool = kani::any();
    let position = usize::from(kani::any::<u8>() & 127);
    unsafe {
        SOURCE = &x;
        TARGET = width;
        SIGN = caller_sign;
        CALLS = 0;
    }
    let ret = Op::Add.eval_value_unary(&x, width, caller_sign, &mut MaskCache::default());
    assert!(unsafe { CALLS } == 1, "wide operator semantics mismatch");
    let expected = extended_logic_bit(a.planes, source, width, a.signed, caller_sign, position);
    result(
        &ret,
        width,
        a.signed && (source == width || caller_sign),
        position,
        expected,
    );
    preserve(&x, a);
    kani::cover!(
        source == 1 && width == 128 && a.signed && caller_sign,
        "wide plus native sign extension is reachable"
    );
    kani::cover!(
        source == 64 && width == 65 && a.signed && !caller_sign,
        "wide plus native zero extension is reachable"
    );
    kani::cover!(
        source == 65
            && width == 128
            && a.signed
            && caller_sign
            && a.planes == (0, 1u128 << 64)
            && position == 127,
        "wide plus X sign extension is reachable"
    );
    kani::cover!(
        source == 65
            && width == 128
            && a.signed
            && caller_sign
            && a.planes == (1u128 << 64, 1u128 << 64)
            && position == 127,
        "wide plus Z sign extension is reachable"
    );
    kani::cover!(
        source == width && width == 128 && a.signed && !caller_sign,
        "wide plus equal width preserves signed metadata is reachable"
    );
    kani::cover!(
        !a.signed && caller_sign && source < width,
        "wide plus unsigned source stays unsigned is reachable"
    );
    kani::cover!(
        source == 127 && width == 128 && a.signed && !caller_sign && position == 127,
        "wide plus disabled sign extension is reachable"
    );
    kani::cover!(
        source == 128 && width == 128 && a.planes.1 != 0,
        "wide plus maximum unknown input is reachable"
    );
}
