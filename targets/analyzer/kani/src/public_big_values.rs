//! Public Value operations over bounded mathematical BigUint APIs.
//! Numeric values use opaque handles: the API stubs implement exact u128 math,
//! leaving Veryl's operation bodies and metadata/four-state decisions unchanged.
//! These proofs assume the numerical API contract, not the library's allocation
//! implementation. Real-library integration remains in the native comparisons.
//! Public constructors remain finite comparisons (big-constructor-boundaries).
//! gen_mask has a separate exhaustive comparison of every input in 0..=128.
//! Native helper comparisons and storage proofs are both required in CI.
//! The handle bridge, numeric API, and scalar reference have separate universal proofs.
//! Pool capacity permits 63 registrations per trace; capacity and decoder bounds
//! are asserted, never assumed. Every handle-using proof initializes the pool.
//! Operation harnesses use unwind3; the 128-bit scalar fold uses unwind129.
//! Inputs are normalized four-state values; condition/source widths are 65..=128.
//! Each arbitrary observation position 0..127 quantifies an entire bit plane,
//! together with bit-length, storage, width, sign and source-preservation checks.
//! IEEE 1800-2023 11.4.7 (truth), 11.4.11/Table 11-20 (conditional),
//! 11.8.2 (extension), and 11.5.1 (missing selected bits read X).
//! These are evaluated-value contracts, not proofs of source expression effects.
use veryl_analyzer::value::Value;
use veryl_formal_semantics::{
    comparison::Operand,
    conditional::conditional_logic_bit,
    logic4::{Logic4, read_logic},
    resize::extended_logic_bit,
    selection::selected_logic_bit,
};

// A bit belongs to the known-one set exactly when its payload plane is set
// and its X/Z plane is clear. The independent scalar fold theorem below proves
// this set-based classification for every pair of 128-bit planes. Normalized
// padding is zero, so folding to 128 also classifies declared widths65..128.
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
fn big_truth_reference_matches_scalar_fold() {
    let planes = (kani::any(), kani::any());
    assert!(
        truth(planes) == veryl_formal_semantics::gates::logical_truth(planes, 128),
        "public BigUint semantics mismatch"
    );
    kani::cover!(
        truth(planes) == Logic4::Zero,
        "big truth reference zero is reachable"
    );
    kani::cover!(
        truth(planes) == Logic4::One,
        "big truth reference one is reachable"
    );
    kani::cover!(
        truth(planes) == Logic4::X,
        "big truth reference X is reachable"
    );
}

// Numeric API boundary: assume num-bigint's exact nonnegative arithmetic and
// cloning contracts, just as native u128 operators are primitive in the model.
// These adapters do not substitute Value::truth/conditional/expand/select. Thus
// these proofs establish Veryl control/metadata/four-state semantics conditional
// on the numeric API, not memory safety of every num-bigint allocation path.
// Actual-library integration remains in the native boundary comparisons.
// Domain/path guard failures have distinct messages: mutation classification
// must never accept them as semantic counterexamples.
// Opaque inline handles eliminate library allocation paths from the numeric
// abstraction. A symbolic nonzero tag prevents an ordinary unmodelled literal
// from being silently interpreted as a handle. Raw ZERO remains numeric zero.
// This store is proof-only, with explicit bounds; failures are not semantic CEs.
const CAPACITY: usize = 64;
static mut VALUES: [u128; CAPACITY] = [0; CAPACITY];
static mut NEXT: usize = 1;
static mut TAG: u32 = 0;
fn init_numeric() {
    let tag: u16 = kani::any();
    kani::assume(tag != 0);
    unsafe {
        TAG = u32::from(tag) << 16;
        NEXT = 1;
    }
}
fn plane(bits: u128) -> num_bigint::BigUint {
    unsafe {
        let index = NEXT;
        assert!(index < CAPACITY, "BigUint numeric model domain exceeded");
        VALUES[index] = bits;
        NEXT += 1;
        num_bigint::BigUint::new_const(TAG | index as u32)
    }
}
fn read_plane(value: &num_bigint::BigUint) -> u128 {
    let mut digits = value.iter_u64_digits();
    let raw = digits.next().unwrap_or(0);
    assert!(
        digits.next().is_none() && raw <= u64::from(u32::MAX),
        "BigUint numeric model domain exceeded"
    );
    if raw == 0 {
        return 0;
    }
    unsafe {
        let index = (raw & 0xffff) as usize;
        assert!(
            raw >> 16 == u64::from(TAG >> 16) && index > 0 && index < NEXT && index < CAPACITY,
            "BigUint numeric model domain exceeded"
        );
        VALUES[index]
    }
}
fn numeric_from_u64(bits: u64) -> num_bigint::BigUint {
    plane(u128::from(bits))
}
fn numeric_from_u128(bits: u128) -> num_bigint::BigUint {
    plane(bits)
}
fn numeric_is_zero(a: &num_bigint::BigUint) -> bool {
    read_plane(a) == 0
}
fn numeric_one() -> num_bigint::BigUint {
    plane(1)
}
fn numeric_bits(a: &num_bigint::BigUint) -> u64 {
    u64::from(128 - read_plane(a).leading_zeros())
}
fn numeric_bit(a: &num_bigint::BigUint, position: u64) -> bool {
    position < 128 && (read_plane(a) >> position & 1 != 0)
}
fn numeric_to_u64(a: &num_bigint::BigUint) -> Option<u64> {
    let bits = read_plane(a);
    if bits <= u128::from(u64::MAX) {
        Some(bits as u64)
    } else {
        None
    }
}
fn numeric_getter(a: &num_bigint::BigUint) -> u128 {
    read_plane(a)
}
fn numeric_clone(a: &num_bigint::BigUint) -> num_bigint::BigUint {
    plane(read_plane(a))
}
// Kani requires early-bound lifetimes matching the numeric trait impls.
fn numeric_and<'a: 'a, 'b: 'b>(
    a: &'a num_bigint::BigUint,
    b: &'b num_bigint::BigUint,
) -> num_bigint::BigUint {
    plane(read_plane(a) & read_plane(b))
}
fn numeric_xor<'a: 'a, 'b: 'b>(
    a: &'a num_bigint::BigUint,
    b: &'b num_bigint::BigUint,
) -> num_bigint::BigUint {
    plane(read_plane(a) ^ read_plane(b))
}
fn numeric_xor_owned(a: num_bigint::BigUint, b: num_bigint::BigUint) -> num_bigint::BigUint {
    numeric_xor(&a, &b)
}
fn numeric_xor_right_owned<'a: 'a>(
    a: &'a num_bigint::BigUint,
    b: num_bigint::BigUint,
) -> num_bigint::BigUint {
    numeric_xor(a, &b)
}
fn numeric_or_owned(a: num_bigint::BigUint, b: num_bigint::BigUint) -> num_bigint::BigUint {
    plane(read_plane(&a) | read_plane(&b))
}
fn numeric_or_assign<'a: 'a>(a: &mut num_bigint::BigUint, b: &'a num_bigint::BigUint) {
    *a = plane(read_plane(a) | read_plane(b));
}
fn numeric_and_assign<'a: 'a>(a: &mut num_bigint::BigUint, b: &'a num_bigint::BigUint) {
    *a = numeric_and(a, b);
}
fn numeric_shr_assign(a: &mut num_bigint::BigUint, shift: usize) {
    *a = plane(if shift >= 128 {
        0
    } else {
        read_plane(a) >> shift
    });
}
// Native branches/results cannot enter BigUint branch expansion or cloning.
// These guards fail closed, avoiding symbolic exploration of impossible paths.
fn reject_branch_mask(_: usize) -> num_bigint::BigUint {
    panic!("unexpected BigUint model path");
}
fn reject_branch_clone(
    _: &veryl_analyzer::value::ValueBigUint,
) -> veryl_analyzer::value::ValueBigUint {
    panic!("unexpected BigUint model path");
}
fn reject_source_to_native(
    _: &veryl_analyzer::value::ValueBigUint,
) -> Option<veryl_analyzer::value::ValueU64> {
    panic!("unexpected BigUint model path");
}
fn reject_native_conversion(_: u128) -> num_bigint::BigUint {
    panic!("unexpected BigUint model path");
}
kani::stub_set!(
    numeric_biguint,
    stub(
        <num_bigint::BigUint as std::clone::Clone>::clone,
        numeric_clone
    ),
    stub(
        <&num_bigint::BigUint as std::ops::BitAnd<&num_bigint::BigUint>>::bitand,
        numeric_and
    ),
    stub(
        <&num_bigint::BigUint as std::ops::BitXor<&num_bigint::BigUint>>::bitxor,
        numeric_xor
    ),
    stub(
        <num_bigint::BigUint as std::ops::BitXor<num_bigint::BigUint>>::bitxor,
        numeric_xor_owned
    ),
    stub(
        <&num_bigint::BigUint as std::ops::BitXor<num_bigint::BigUint>>::bitxor,
        numeric_xor_right_owned
    ),
    stub(
        <num_bigint::BigUint as std::ops::BitOr<num_bigint::BigUint>>::bitor,
        numeric_or_owned
    ),
    stub(
        <num_bigint::BigUint as std::ops::BitOrAssign<&num_bigint::BigUint>>::bitor_assign,
        numeric_or_assign
    ),
    stub(
        <num_bigint::BigUint as std::ops::BitAndAssign<&num_bigint::BigUint>>::bitand_assign,
        numeric_and_assign
    ),
    stub(
        <num_bigint::BigUint as std::ops::ShrAssign<usize>>::shr_assign,
        numeric_shr_assign
    ),
    stub(
        veryl_analyzer::value::u128_to_biguint,
        reject_native_conversion
    ),
    stub(
        <num_bigint::BigUint as std::convert::From<u64>>::from,
        numeric_from_u64
    ),
    stub(
        <num_bigint::BigUint as std::convert::From<u128>>::from,
        numeric_from_u128
    ),
    stub(
        <num_bigint::BigUint as num_traits::Zero>::is_zero,
        numeric_is_zero
    ),
    stub(<num_bigint::BigUint as num_traits::One>::one, numeric_one),
    stub(num_bigint::BigUint::bits, numeric_bits),
    stub(num_bigint::BigUint::bit, numeric_bit),
    stub(
        <num_bigint::BigUint as num_traits::ToPrimitive>::to_u64,
        numeric_to_u64
    ),
    stub(veryl_analyzer::value::biguint_to_u128, numeric_getter),
);

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric_biguint)]
fn bounded_biguint_numeric_bridge_preserves_bits() {
    init_numeric();
    let native: bool = kani::any();
    let bits: u128 = if native {
        u128::from(kani::any::<u64>())
    } else {
        kani::any()
    };
    let value = if native {
        num_bigint::BigUint::from(bits as u64)
    } else {
        plane(bits)
    };
    let position = usize::from(kani::any::<u8>() & 127);
    assert!(
        value.bits() <= 128
            && value.bit(position as u64) == (bits >> position & 1 != 0)
            && read_plane(&value) == bits,
        "public BigUint semantics mismatch"
    );
    kani::cover!(bits == 0, "big numeric bridge zero is reachable");
    kani::cover!(
        bits == u128::MAX,
        "big numeric bridge two words are reachable"
    );
    kani::cover!(
        bits == 1u128 << 64,
        "big numeric bridge word boundary is reachable"
    );
    kani::cover!(
        native && bits == u128::from(u64::MAX),
        "big numeric bridge inline word is reachable"
    );
}

// Check the abstraction against scalar bit definitions, independently of the
// particular numeric states exercised by the four Value operations.
#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric_biguint)]
fn bounded_biguint_numeric_api_follows_scalar_bits() {
    init_numeric();
    let x: u128 = kani::any();
    let y: u128 = kani::any();
    let position = usize::from(kani::any::<u8>() & 127);
    let shift: usize = kani::any();
    let a = plane(x);
    let b = plane(y);
    let left = x >> position & 1 != 0;
    let right = y >> position & 1 != 0;
    let and = &a & &b;
    let xor = a.clone() ^ b.clone();
    let or = a.clone() | b.clone();
    let mut and_assigned = a.clone();
    and_assigned &= &b;
    let mut or_assigned = a.clone();
    or_assigned |= &b;
    let mut shifted = a.clone();
    shifted >>= shift;
    let expected_shift =
        shift < 128 && position < 128 - shift && (x >> (position + shift) & 1 != 0);
    assert!(
        and.bit(position as u64) == (left && right)
            && xor.bit(position as u64) == (left != right)
            && or.bit(position as u64) == (left || right)
            && and_assigned.bit(position as u64) == (left && right)
            && or_assigned.bit(position as u64) == (left || right)
            && shifted.bit(position as u64) == expected_shift,
        "public BigUint semantics mismatch"
    );
    assert!(
        a.bits() <= 128
            && ((position as u64) < a.bits()) == (x >> position != 0)
            && <num_bigint::BigUint as num_traits::Zero>::is_zero(&a) == (x == 0)
            && read_plane(&a) == x
            && read_plane(&b) == y,
        "public BigUint semantics mismatch"
    );
    let narrowed = <num_bigint::BigUint as num_traits::ToPrimitive>::to_u64(&a);
    assert!(
        if x >> 64 == 0 {
            narrowed == Some(x as u64)
        } else {
            narrowed.is_none()
        },
        "public BigUint semantics mismatch"
    );
    kani::cover!(
        x == 0 && y == 0 && shift == usize::MAX,
        "big numeric API zero and maximum shift are reachable"
    );
    kani::cover!(
        x == u128::MAX && y == u128::MAX && position == 127 && shift == 0,
        "big numeric API two full words are reachable"
    );
    kani::cover!(
        x == 1u128 << 64 && y == 1u128 << 63 && position == 63 && shift == 1,
        "big numeric API word crossing is reachable"
    );
    kani::cover!(
        x == 1u128 << 127 && y == (1u128 << 127) - 1 && position == 127,
        "big numeric API opposite high bits are reachable"
    );
}

// Finite native comparison big-mask-exhaustive checks every width in 0..=128.
// The scalar mask is encoded through the separately proved numeric bridge.
fn enumerated_mask(width: usize) -> num_bigint::BigUint {
    assert!(width <= 128, "BigUint numeric model domain exceeded");
    plane(if width == 0 {
        0
    } else {
        u128::MAX >> (128 - width)
    })
}
fn big_input(width: usize, position: usize) -> (Value, Operand) {
    let mask = u128::MAX >> (128 - width);
    let before = Operand {
        planes: (kani::any::<u128>(), kani::any::<u128>()),
        width,
        signed: kani::any(),
    };
    kani::assume((before.planes.0 | before.planes.1) & !mask == 0);
    let value = Value::BigUint(veryl_analyzer::value::ValueBigUint {
        payload: Box::new(plane(before.planes.0)),
        mask_xz: Box::new(plane(before.planes.1)),
        width: width as u32,
        signed: before.signed,
    });
    check_input(&value, before, position);
    let Value::BigUint(big) = value else {
        panic!("public BigUint semantics mismatch")
    };
    (Value::BigUint(big), before)
}
fn native_input(width: usize, position: usize) -> (Value, Operand) {
    let mask = u64::MAX >> (64 - width);
    let before = Operand {
        planes: (
            u128::from(kani::any::<u64>() & mask),
            u128::from(kani::any::<u64>() & mask),
        ),
        width,
        signed: kani::any(),
    };
    let value = Value::from_u128(before.planes.0, before.planes.1, width, before.signed);
    check_input(&value, before, position);
    let Value::U64(native) = value else {
        panic!("public BigUint semantics mismatch")
    };
    (Value::U64(native), before)
}
fn check_input(value: &Value, before: Operand, position: usize) {
    assert!(
        value.width() == before.width && value.signed() == before.signed,
        "public BigUint semantics mismatch"
    );
    match value {
        Value::U64(n) => assert!(
            before.width <= 64
                && u128::from(n.payload) == before.planes.0
                && u128::from(n.mask_xz) == before.planes.1,
            "public BigUint semantics mismatch"
        ),
        Value::BigUint(n) => assert!(
            before.width > 64
                && n.payload.bits() <= before.width as u64
                && n.mask_xz.bits() <= before.width as u64
                && n.payload.bit(position as u64) == (before.planes.0 >> position & 1 != 0)
                && n.mask_xz.bit(position as u64) == (before.planes.1 >> position & 1 != 0),
            "public BigUint semantics mismatch"
        ),
    }
}
fn check_result(value: &Value, width: usize, signed: bool, position: usize, expected: Logic4) {
    assert!(
        value.width() == width
            && value.signed() == signed
            && matches!(value, Value::U64(_)) == (width <= 64),
        "public BigUint semantics mismatch"
    );
    if let Value::BigUint(n) = value {
        assert!(
            n.payload.bits() <= width as u64 && n.mask_xz.bits() <= width as u64,
            "public BigUint semantics mismatch"
        );
    }
    // Execute public getters as well as the numeric operation: no representation
    // assumption is needed to compose a producer with a separate getter proof.
    assert!(
        read_logic(value.payload_u128(), value.mask_xz_u128(), position) == expected,
        "public BigUint semantics mismatch"
    );
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric_biguint)]
fn public_big_truth_matches_reference_semantics() {
    init_numeric();
    let width = usize::from(kani::any::<u8>() & 63) + 65;
    let position = usize::from(kani::any::<u8>() & 127);
    let (value, before) = big_input(width, position);
    let expected = truth(before.planes);
    let actual = match value.truth() {
        Some(true) => Logic4::One,
        Some(false) => Logic4::Zero,
        None => Logic4::X,
    };
    assert!(actual == expected, "public BigUint semantics mismatch");
    check_input(&value, before, position);
    kani::cover!(
        width == 65 && !before.signed && expected == Logic4::Zero,
        "big truth zero endpoint is reachable"
    );
    kani::cover!(
        width == 128 && before.signed && before.planes == (1u128 << 127, 0),
        "big truth high one is reachable"
    );
    kani::cover!(
        width == 128 && !before.signed && before.planes == (0, 1u128 << 127),
        "big truth high X is reachable"
    );
    kani::cover!(
        width == 128 && before.signed && before.planes == (1u128 << 127, 1u128 << 127),
        "big truth high Z is reachable"
    );
    kani::cover!(
        width == 127 && before.planes == (1, 1u128 << 100),
        "big truth known one dominates X is reachable"
    );
}

// BigUint condition, independently sized native branches, context 0..=64.
#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric_biguint)]
#[kani::stub(veryl_analyzer::value::ValueBigUint::gen_mask, reject_branch_mask)]
#[kani::stub(<veryl_analyzer::value::ValueBigUint as std::clone::Clone>::clone, reject_branch_clone)]
fn public_big_condition_matches_reference_semantics() {
    init_numeric();
    let width = usize::from(kani::any::<u8>() & 63) + 65;
    let position = usize::from(kani::any::<u8>() & 127);
    let (c, cr) = big_input(width, position);
    let (a, ar) = native_input(usize::from(kani::any::<u8>() & 63) + 1, position);
    let (b, br) = native_input(usize::from(kani::any::<u8>() & 63) + 1, position);
    let context = usize::from(kani::any::<u8>() & 127).min(64);
    let signed = kani::any();
    let output_width = context.max(ar.width).max(br.width);
    let expected = conditional_logic_bit(
        truth(cr.planes),
        (ar.planes, ar.width, ar.signed),
        (br.planes, br.width, br.signed),
        output_width,
        signed,
        position,
    );
    let result = c.conditional(&a, &b, context, signed);
    let Value::U64(native) = result else {
        panic!("public BigUint semantics mismatch")
    };
    let result = Value::U64(native);
    check_result(&result, output_width, signed, position, expected);
    check_input(&c, cr, position);
    check_input(&a, ar, position);
    check_input(&b, br, position);
    kani::cover!(
        width == 65 && context == 0 && truth(cr.planes) == Logic4::Zero,
        "big conditional zero endpoint is reachable"
    );
    kani::cover!(
        width == 128 && cr.planes == (1u128 << 127, 0) && ar.width == 64 && br.width == 1 && signed,
        "big conditional high one is reachable"
    );
    kani::cover!(
        width == 128
            && cr.planes == (0, 1u128 << 100)
            && ar.width == 1
            && br.width == 64
            && !signed,
        "big conditional high X is reachable"
    );
    kani::cover!(
        width == 128 && cr.planes == (1u128 << 127, 1u128 << 127) && context == 64,
        "big conditional high Z is reachable"
    );
    kani::cover!(
        cr.planes == (1, 1u128 << 64) && cr.signed && ar.signed && !br.signed && !signed,
        "big conditional mixed signs and known one are reachable"
    );
}

// Complete the signed native-source partition with MSB 1/X/Z. Preserve the
// clone/truncation and preceding zero-extension call from the executable trace.
#[kani::proof]
#[kani::unwind(3)]
#[kani::stub(veryl_analyzer::value::ValueBigUint::gen_mask, enumerated_mask)]
#[kani::use_stub_set(numeric_biguint)]
#[kani::stub(<veryl_analyzer::value::ValueBigUint as std::clone::Clone>::clone, reject_branch_clone)]
#[kani::stub(
    veryl_analyzer::value::ValueBigUint::to_value_u64,
    reject_source_to_native
)]
fn public_native_sign_extension_wide_matches_reference_semantics() {
    init_numeric();
    let position = usize::from(kani::any::<u8>() & 127);
    let source = usize::from(kani::any::<u8>() & 63) + 1;
    let target = usize::from(kani::any::<u8>() & 63) + 65;
    let (value, before) = native_input(source, position);
    kani::assume(
        before.signed && (before.planes.0 | before.planes.1) & (1u128 << (source - 1)) != 0,
    );
    let mut truncated = value.clone();
    truncated.trunc(target);
    let Value::U64(native) = truncated else {
        panic!("public BigUint semantics mismatch");
    };
    let truncated = Value::U64(native);
    check_input(&truncated, before, position);
    drop(value.expand(target, false));
    let result = value.expand(target, true).into_owned();
    let Value::BigUint(big) = result else {
        panic!("public BigUint semantics mismatch");
    };
    let result = Value::BigUint(big);
    let expected = extended_logic_bit(before.planes, source, target, true, true, position);
    check_result(&result, target, true, position, expected);
    check_input(&value, before, position);
    let msb = read_logic(before.planes.0, before.planes.1, source - 1);
    kani::cover!(
        source == 1 && target == 65 && msb == Logic4::One,
        "big sign extension one is reachable"
    );
    kani::cover!(
        source == 64 && target == 128 && msb == Logic4::X,
        "big sign extension X is reachable"
    );
    kani::cover!(
        source == 64 && target == 65 && msb == Logic4::Z,
        "big sign extension Z is reachable"
    );
    kani::cover!(
        source == 32 && target == 127 && msb == Logic4::Z && position == 100,
        "big sign extension interior is reachable"
    );
}

// All endpoint pairs with a result of 0..=64 bits, including reversed and
// usize::MAX endpoints. Subtract only after ordering, preventing host overflow.
#[kani::proof]
#[kani::unwind(3)]
#[kani::stub(veryl_analyzer::value::ValueBigUint::gen_mask, enumerated_mask)]
#[kani::use_stub_set(numeric_biguint)]
fn public_big_selection_matches_reference_semantics() {
    init_numeric();
    let width = usize::from(kani::any::<u8>() & 63) + 65;
    let position = usize::from(kani::any::<u8>() & 127);
    let (value, before) = big_input(width, position);
    let beg: usize = kani::any();
    let end: usize = kani::any();
    kani::assume(beg < end || beg - end < 64);
    let original = value.clone();
    let result = value.select(beg, end);
    let output_width = if beg < end { 0 } else { beg - end + 1 };
    let expected = selected_logic_bit(before.planes, width, beg, end, position);
    let Value::U64(native) = result else {
        panic!("public BigUint semantics mismatch")
    };
    let result = Value::U64(native);
    check_result(&result, output_width, false, position, expected);
    check_input(&value, before, position);
    check_input(&original, before, position);
    kani::cover!(
        width == 65 && beg == 64 && end == 63,
        "big selection crosses word boundary is reachable"
    );
    kani::cover!(
        width == 128 && beg == 127 && end == 64 && before.signed,
        "big selection upper word is reachable"
    );
    kani::cover!(
        beg == width + 2 && end == width - 1 && expected == Logic4::X,
        "big selection partial overlap is reachable"
    );
    kani::cover!(
        beg == usize::MAX && end == usize::MAX - 3,
        "big selection maximum endpoint is reachable"
    );
    kani::cover!(
        beg == 0 && end == usize::MAX,
        "big selection reversed endpoints are reachable"
    );
    kani::cover!(
        width == 128 && beg == 127 && end == 127 && before.planes == (1u128 << 127, 1u128 << 127),
        "big selection high Z is reachable"
    );
}
