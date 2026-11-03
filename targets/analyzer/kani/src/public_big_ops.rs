//! Bounded numeric API: operands/results 1..128, intermediates up to 256 bits.
//! Actual assignment and Op arms execute under numeric and mask-cache APIs.
//! Bitwise/shift kernels quantify prepared operands. followup.rs checks original
//! preparation argument wiring separately; expansion has independent contracts.
//! Numeric domain/overflow/path failures are distinct from semantic mismatches.
//! Real-library constructors, masks/cache and operation traces remain native.
//! This is not a proof of num-bigint allocation or arbitrary HashMap states.
use veryl_analyzer::{
    ir::Op,
    value::{MaskCache, Value, ValueBigUint},
};
use veryl_formal_semantics::{
    bits256::Bits256 as N,
    comparison::Operand,
    logic4::{Logic4, read_logic},
};
const CAP: usize = 32;
static mut VALUES: [N; CAP] = [N::ZERO; CAP];
static mut NEXT: usize = 1;
static mut TAG: u32 = 0;
static mut CACHE: [num_bigint::BigUint; 129] = [const { num_bigint::BigUint::ZERO }; 129];
fn init() {
    let t: u16 = kani::any();
    kani::assume(t != 0);
    unsafe {
        TAG = u32::from(t) << 16;
        NEXT = 1;
        ordering::reset();
    }
}
fn put(n: N) -> num_bigint::BigUint {
    unsafe {
        let i = NEXT;
        assert!(i < CAP, "256-bit numeric model domain exceeded");
        VALUES[i] = n;
        NEXT += 1;
        num_bigint::BigUint::new_const(TAG | i as u32)
    }
}
fn get(n: &num_bigint::BigUint) -> N {
    let mut it = n.iter_u64_digits();
    let raw = it.next().unwrap_or(0);
    assert!(
        it.next().is_none() && raw <= u64::from(u32::MAX),
        "256-bit numeric model domain exceeded"
    );
    if raw == 0 {
        return N::ZERO;
    }
    unsafe {
        let i = (raw & 0xffff) as usize;
        assert!(
            raw >> 16 == u64::from(TAG >> 16) && i > 0 && i < NEXT && i < CAP,
            "256-bit numeric model domain exceeded"
        );
        VALUES[i]
    }
}
fn clone_number(n: &num_bigint::BigUint) -> num_bigint::BigUint {
    let _ = get(n);
    num_bigint::BigUint::new_const(n.iter_u64_digits().next().unwrap_or(0) as u32)
}
fn from64(n: u64) -> num_bigint::BigUint {
    put(N::from_u128(u128::from(n)))
}
fn from128(n: u128) -> num_bigint::BigUint {
    put(N::from_u128(n))
}
fn bits(n: &num_bigint::BigUint) -> u64 {
    get(n).bits()
}
fn bit(n: &num_bigint::BigUint, p: u64) -> bool {
    p < 256 && get(n).bit(p as usize)
}
fn zero(n: &num_bigint::BigUint) -> bool {
    get(n) == N::ZERO
}
fn one() -> num_bigint::BigUint {
    from64(1)
}
fn eq(a: &num_bigint::BigUint, b: &num_bigint::BigUint) -> bool {
    get(a) == get(b)
}
fn to64(n: &num_bigint::BigUint) -> Option<u64> {
    let n = get(n);
    if n.hi == 0 && n.lo <= u128::from(u64::MAX) {
        Some(n.lo as u64)
    } else {
        None
    }
}
fn low64(n: &num_bigint::BigUint) -> u64 {
    get(n).lo as u64
}
fn getter(n: &num_bigint::BigUint) -> u128 {
    get(n).lo
}
fn popcount(n: &num_bigint::BigUint) -> u64 {
    let n = get(n);
    u64::from(n.lo.count_ones()) + u64::from(n.hi.count_ones())
}
fn mask(w: usize) -> num_bigint::BigUint {
    assert!(w <= 128, "256-bit numeric model domain exceeded");
    put(N::mask(w))
}
// A mathematical memoized lookup contract. Its real implementation is checked
// separately by finite complete width-pair and full-table histories in native CI.
fn cached_mask(_: &mut MaskCache, w: usize) -> &num_bigint::BigUint {
    assert!(w <= 128, "256-bit numeric model domain exceeded");
    let value = mask(w);
    unsafe {
        CACHE[w] = value;
        &*std::ptr::addr_of!(CACHE[w])
    }
}
fn and_oo(a: num_bigint::BigUint, b: num_bigint::BigUint) -> num_bigint::BigUint {
    put(get(&a).and(get(&b)))
}
fn and_ro<'a: 'a>(a: &'a num_bigint::BigUint, b: num_bigint::BigUint) -> num_bigint::BigUint {
    put(get(&a).and(get(&b)))
}
fn and_or<'a: 'a>(a: num_bigint::BigUint, b: &'a num_bigint::BigUint) -> num_bigint::BigUint {
    put(get(&a).and(get(&b)))
}
fn and_rr<'a: 'a, 'b: 'b>(
    a: &'a num_bigint::BigUint,
    b: &'b num_bigint::BigUint,
) -> num_bigint::BigUint {
    put(get(&a).and(get(&b)))
}
fn and_assign_o(a: &mut num_bigint::BigUint, b: num_bigint::BigUint) {
    *a = put(get(a).and(get(&b)));
}
fn and_assign_r<'a: 'a>(a: &mut num_bigint::BigUint, b: &'a num_bigint::BigUint) {
    *a = put(get(a).and(get(&b)));
}
fn or_oo(a: num_bigint::BigUint, b: num_bigint::BigUint) -> num_bigint::BigUint {
    put(get(&a).or(get(&b)))
}
fn or_ro<'a: 'a>(a: &'a num_bigint::BigUint, b: num_bigint::BigUint) -> num_bigint::BigUint {
    put(get(&a).or(get(&b)))
}
fn or_or<'a: 'a>(a: num_bigint::BigUint, b: &'a num_bigint::BigUint) -> num_bigint::BigUint {
    put(get(&a).or(get(&b)))
}
fn or_rr<'a: 'a, 'b: 'b>(
    a: &'a num_bigint::BigUint,
    b: &'b num_bigint::BigUint,
) -> num_bigint::BigUint {
    put(get(&a).or(get(&b)))
}
fn or_assign_o(a: &mut num_bigint::BigUint, b: num_bigint::BigUint) {
    *a = put(get(a).or(get(&b)));
}
fn or_assign_r<'a: 'a>(a: &mut num_bigint::BigUint, b: &'a num_bigint::BigUint) {
    *a = put(get(a).or(get(&b)));
}
fn xor_oo(a: num_bigint::BigUint, b: num_bigint::BigUint) -> num_bigint::BigUint {
    put(get(&a).xor(get(&b)))
}
fn xor_ro<'a: 'a>(a: &'a num_bigint::BigUint, b: num_bigint::BigUint) -> num_bigint::BigUint {
    put(get(&a).xor(get(&b)))
}
fn xor_or<'a: 'a>(a: num_bigint::BigUint, b: &'a num_bigint::BigUint) -> num_bigint::BigUint {
    put(get(&a).xor(get(&b)))
}
fn xor_rr<'a: 'a, 'b: 'b>(
    a: &'a num_bigint::BigUint,
    b: &'b num_bigint::BigUint,
) -> num_bigint::BigUint {
    put(get(&a).xor(get(&b)))
}
fn xor_assign_o(a: &mut num_bigint::BigUint, b: num_bigint::BigUint) {
    *a = put(get(a).xor(get(&b)));
}
fn xor_assign_r<'a: 'a>(a: &mut num_bigint::BigUint, b: &'a num_bigint::BigUint) {
    *a = put(get(a).xor(get(&b)));
}
fn left_usize(a: num_bigint::BigUint, b: usize) -> num_bigint::BigUint {
    put(get(&a).checked_shl(b as usize))
}
fn left_ref_usize<'a: 'a>(a: &'a num_bigint::BigUint, b: usize) -> num_bigint::BigUint {
    put(get(a).checked_shl(b as usize))
}
fn left_assign_usize(a: &mut num_bigint::BigUint, b: usize) {
    *a = put(get(a).checked_shl(b as usize));
}
fn left_u32(a: num_bigint::BigUint, b: u32) -> num_bigint::BigUint {
    put(get(&a).checked_shl(b as usize))
}
fn left_ref_u32<'a: 'a>(a: &'a num_bigint::BigUint, b: u32) -> num_bigint::BigUint {
    put(get(a).checked_shl(b as usize))
}
fn left_assign_u32(a: &mut num_bigint::BigUint, b: u32) {
    *a = put(get(a).checked_shl(b as usize));
}
fn right_usize(a: num_bigint::BigUint, b: usize) -> num_bigint::BigUint {
    put(get(&a).shr(b as usize))
}
fn right_ref_usize<'a: 'a>(a: &'a num_bigint::BigUint, b: usize) -> num_bigint::BigUint {
    put(get(a).shr(b as usize))
}
fn right_assign_usize(a: &mut num_bigint::BigUint, b: usize) {
    *a = put(get(a).shr(b as usize));
}
fn right_u32(a: num_bigint::BigUint, b: u32) -> num_bigint::BigUint {
    put(get(&a).shr(b as usize))
}
fn right_ref_u32<'a: 'a>(a: &'a num_bigint::BigUint, b: u32) -> num_bigint::BigUint {
    put(get(a).shr(b as usize))
}
fn right_assign_u32(a: &mut num_bigint::BigUint, b: u32) {
    *a = put(get(a).shr(b as usize));
}
fn add_oo(a: num_bigint::BigUint, b: num_bigint::BigUint) -> num_bigint::BigUint {
    put(get(&a).checked_add(get(&b)))
}
fn add_ro<'a: 'a>(a: &'a num_bigint::BigUint, b: num_bigint::BigUint) -> num_bigint::BigUint {
    put(get(a).checked_add(get(&b)))
}
fn add_or<'a: 'a>(a: num_bigint::BigUint, b: &'a num_bigint::BigUint) -> num_bigint::BigUint {
    put(get(&a).checked_add(get(b)))
}
fn add_rr<'a: 'a, 'b: 'b>(
    a: &'a num_bigint::BigUint,
    b: &'b num_bigint::BigUint,
) -> num_bigint::BigUint {
    put(get(a).checked_add(get(b)))
}
fn add_assign_o(a: &mut num_bigint::BigUint, b: num_bigint::BigUint) {
    *a = put(get(a).checked_add(get(&b)));
}
fn add_assign_r<'a: 'a>(a: &mut num_bigint::BigUint, b: &'a num_bigint::BigUint) {
    *a = put(get(a).checked_add(get(b)));
}
kani::stub_set!(
    numeric256,
    stub(
        <num_bigint::BigUint as std::ops::Add<num_bigint::BigUint>>::add,
        add_oo
    ),
    stub(
        <&num_bigint::BigUint as std::ops::Add<num_bigint::BigUint>>::add,
        add_ro
    ),
    stub(
        <num_bigint::BigUint as std::ops::Add<&num_bigint::BigUint>>::add,
        add_or
    ),
    stub(
        <&num_bigint::BigUint as std::ops::Add<&num_bigint::BigUint>>::add,
        add_rr
    ),
    stub(
        <num_bigint::BigUint as std::ops::AddAssign<num_bigint::BigUint>>::add_assign,
        add_assign_o
    ),
    stub(
        <num_bigint::BigUint as std::ops::AddAssign<&num_bigint::BigUint>>::add_assign,
        add_assign_r
    ),
    stub(
        <num_bigint::BigUint as std::clone::Clone>::clone,
        clone_number
    ),
    stub(
        <num_bigint::BigUint as std::convert::From<u64>>::from,
        from64
    ),
    stub(
        <num_bigint::BigUint as std::convert::From<u128>>::from,
        from128
    ),
    stub(num_bigint::BigUint::bits, bits),
    stub(num_bigint::BigUint::bit, bit),
    stub(num_bigint::BigUint::count_ones, popcount),
    stub(<num_bigint::BigUint as num_traits::Zero>::is_zero, zero),
    stub(<num_bigint::BigUint as num_traits::One>::one, one),
    stub(num_bigint::BigUint::eq, eq),
    stub(
        num_bigint::BigUint::cmp,
        crate::public_big_ops::ordering::unsigned_cmp
    ),
    stub(
        num_bigint::BigInt::from_biguint,
        crate::public_big_ops::ordering::from_biguint
    ),
    stub(
        num_bigint::BigInt::cmp,
        crate::public_big_ops::ordering::signed_cmp
    ),
    stub(
        num_bigint::BigInt::eq,
        crate::public_big_ops::ordering::signed_eq
    ),
    stub(
        <num_bigint::BigUint as num_traits::ToPrimitive>::to_u64,
        to64
    ),
    stub(veryl_analyzer::value::biguint_to_u128, getter),
    stub(veryl_analyzer::value::biguint_low_u64, low64),
    stub(veryl_analyzer::value::u128_to_biguint, from128),
    stub(veryl_analyzer::value::ValueBigUint::gen_mask, mask),
    stub(veryl_analyzer::value::MaskCache::get, cached_mask),
    stub(
        <num_bigint::BigUint as std::ops::BitAnd<num_bigint::BigUint>>::bitand,
        and_oo
    ),
    stub(
        <&num_bigint::BigUint as std::ops::BitAnd<num_bigint::BigUint>>::bitand,
        and_ro
    ),
    stub(
        <num_bigint::BigUint as std::ops::BitAnd<&num_bigint::BigUint>>::bitand,
        and_or
    ),
    stub(
        <&num_bigint::BigUint as std::ops::BitAnd<&num_bigint::BigUint>>::bitand,
        and_rr
    ),
    stub(
        <num_bigint::BigUint as std::ops::BitAndAssign<num_bigint::BigUint>>::bitand_assign,
        and_assign_o
    ),
    stub(
        <num_bigint::BigUint as std::ops::BitAndAssign<&num_bigint::BigUint>>::bitand_assign,
        and_assign_r
    ),
    stub(
        <num_bigint::BigUint as std::ops::BitOr<num_bigint::BigUint>>::bitor,
        or_oo
    ),
    stub(
        <&num_bigint::BigUint as std::ops::BitOr<num_bigint::BigUint>>::bitor,
        or_ro
    ),
    stub(
        <num_bigint::BigUint as std::ops::BitOr<&num_bigint::BigUint>>::bitor,
        or_or
    ),
    stub(
        <&num_bigint::BigUint as std::ops::BitOr<&num_bigint::BigUint>>::bitor,
        or_rr
    ),
    stub(
        <num_bigint::BigUint as std::ops::BitOrAssign<num_bigint::BigUint>>::bitor_assign,
        or_assign_o
    ),
    stub(
        <num_bigint::BigUint as std::ops::BitOrAssign<&num_bigint::BigUint>>::bitor_assign,
        or_assign_r
    ),
    stub(
        <num_bigint::BigUint as std::ops::BitXor<num_bigint::BigUint>>::bitxor,
        xor_oo
    ),
    stub(
        <&num_bigint::BigUint as std::ops::BitXor<num_bigint::BigUint>>::bitxor,
        xor_ro
    ),
    stub(
        <num_bigint::BigUint as std::ops::BitXor<&num_bigint::BigUint>>::bitxor,
        xor_or
    ),
    stub(
        <&num_bigint::BigUint as std::ops::BitXor<&num_bigint::BigUint>>::bitxor,
        xor_rr
    ),
    stub(
        <num_bigint::BigUint as std::ops::BitXorAssign<num_bigint::BigUint>>::bitxor_assign,
        xor_assign_o
    ),
    stub(
        <num_bigint::BigUint as std::ops::BitXorAssign<&num_bigint::BigUint>>::bitxor_assign,
        xor_assign_r
    ),
    stub(
        <num_bigint::BigUint as std::ops::Shl<usize>>::shl,
        left_usize
    ),
    stub(
        <&num_bigint::BigUint as std::ops::Shl<usize>>::shl,
        left_ref_usize
    ),
    stub(
        <num_bigint::BigUint as std::ops::ShlAssign<usize>>::shl_assign,
        left_assign_usize
    ),
    stub(<num_bigint::BigUint as std::ops::Shl<u32>>::shl, left_u32),
    stub(
        <&num_bigint::BigUint as std::ops::Shl<u32>>::shl,
        left_ref_u32
    ),
    stub(
        <num_bigint::BigUint as std::ops::ShlAssign<u32>>::shl_assign,
        left_assign_u32
    ),
    stub(
        <num_bigint::BigUint as std::ops::Shr<usize>>::shr,
        right_usize
    ),
    stub(
        <&num_bigint::BigUint as std::ops::Shr<usize>>::shr,
        right_ref_usize
    ),
    stub(
        <num_bigint::BigUint as std::ops::ShrAssign<usize>>::shr_assign,
        right_assign_usize
    ),
    stub(<num_bigint::BigUint as std::ops::Shr<u32>>::shr, right_u32),
    stub(
        <&num_bigint::BigUint as std::ops::Shr<u32>>::shr,
        right_ref_u32
    ),
    stub(
        <num_bigint::BigUint as std::ops::ShrAssign<u32>>::shr_assign,
        right_assign_u32
    ),
);

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
fn numeric256_equality_and_high_bits_match_scalar() {
    init();
    let x = N {
        lo: kani::any(),
        hi: kani::any(),
    };
    let y = N {
        lo: kani::any(),
        hi: kani::any(),
    };
    let a = put(x);
    let b = put(y);
    let p = usize::from(kani::any::<u8>());
    assert!(
        (a == b) == (x.lo == y.lo && x.hi == y.hi)
            && (a != b) == (x.lo != y.lo || x.hi != y.hi)
            && a.bit(p as u64) == x.bit(p)
            && get(&a.clone()) == x
            && num_traits::ToPrimitive::to_u64(&a)
                == if x.hi == 0 && x.lo <= u128::from(u64::MAX) {
                    Some(x.lo as u64)
                } else {
                    None
                }
            && getter(&a) == x.lo
            && low64(&a) == x.lo as u64,
        "wide numeric semantics mismatch"
    );
    kani::cover!(
        x.hi != 0 && x.lo == y.lo && x.hi != y.hi,
        "numeric256 high difference is reachable"
    );
    kani::cover!(x == y && x.hi != 0, "numeric256 high equality is reachable");
}

// All 256 bits and arbitrary shifts are checked by scalar index transport, not
// by another two-half shift implementation. checked_shl rejects overflow, while
// the pure shl operation has explicitly modular256 semantics.
#[kani::proof]
#[kani::unwind(3)]
fn numeric256_shifts_follow_scalar_coordinates() {
    let x = N {
        lo: kani::any(),
        hi: kani::any(),
    };
    let y = N {
        lo: kani::any(),
        hi: kani::any(),
    };
    let p = usize::from(kani::any::<u8>());
    let k: usize = kani::any();
    let left = k <= p && x.bit(p - k);
    let right = k < 256 - p && x.bit(p + k);
    assert!(
        x.shl(k).bit(p) == left
            && x.shr(k).bit(p) == right
            && x.and(y).bit(p) == (x.bit(p) && y.bit(p))
            && x.or(y).bit(p) == (x.bit(p) || y.bit(p))
            && x.xor(y).bit(p) == (x.bit(p) != y.bit(p)),
        "wide numeric semantics mismatch"
    );
    let w = usize::from(kani::any::<u16>());
    kani::assume(w <= 256);
    kani::cover!(w == 0, "numeric256 zero mask is reachable");
    kani::cover!(w == 256, "numeric256 full mask is reachable");
    assert!(
        N::mask(w).bit(p) == (p < w),
        "wide numeric semantics mismatch"
    );
    kani::cover!(
        p == 128 && k == 1 && x.lo >> 127 == 1,
        "numeric256 left crosses128 is reachable"
    );
    kani::cover!(
        p == 127 && k == 1 && x.hi & 1 == 1,
        "numeric256 right crosses128 is reachable"
    );
    kani::cover!(
        p == 255 && k == 128 && x.lo >> 127 == 1,
        "numeric256 bit255 is reachable"
    );
    kani::cover!(
        k == usize::MAX && x.hi != 0,
        "numeric256 oversized shift is reachable"
    );
}
fn input(w: usize) -> (Value, Operand) {
    let mask = u128::MAX >> (128 - w);
    let before = Operand {
        planes: (kani::any::<u128>() & mask, kani::any::<u128>() & mask),
        width: w,
        signed: kani::any(),
    };
    // Reserve both planes on either storage path. Stable handle indices avoid
    // making every later numeric lookup depend on the source representation.
    let payload = from128(before.planes.0);
    let xz = from128(before.planes.1);
    let v = if w <= 64 {
        Value::from_u128(before.planes.0, before.planes.1, w, before.signed)
    } else {
        Value::BigUint(ValueBigUint {
            payload: Box::new(payload),
            mask_xz: Box::new(xz),
            width: w as u32,
            signed: before.signed,
        })
    };
    preserve(&v, before);
    (v, before)
}
fn preserve(v: &Value, before: Operand) {
    assert!(
        v.width() == before.width
            && v.signed() == before.signed
            && v.payload_u128() == before.planes.0
            && v.mask_xz_u128() == before.planes.1
            && matches!(v, Value::U64(_)) == (before.width <= 64),
        "wide operator semantics mismatch"
    );
}
fn result(v: &Value, width: usize, signed: bool, position: usize, expected: Logic4) {
    assert!(
        v.width() == width && v.signed() == signed && matches!(v, Value::U64(_)) == (width <= 64),
        "wide operator semantics mismatch"
    );
    if let Value::BigUint(n) = v {
        let payload = get(&n.payload);
        let xz = get(&n.mask_xz);
        let outside = !(u128::MAX >> (128 - width));
        assert!(
            payload.hi == 0 && xz.hi == 0 && ((payload.lo | xz.lo) & outside) == 0,
            "wide operator semantics mismatch"
        );
    }
    assert!(
        read_logic(v.payload_u128(), v.mask_xz_u128(), position) == expected,
        "wide operator semantics mismatch"
    );
}
// IEEE 1800-2023 11.5.1. All widths 1..128 across four storage pairs;
// arbitrary usize endpoints include reversed, clipped and ignored writes.
fn assignment<const DB: bool, const RB: bool>() {
    init();
    let dw = usize::from(kani::any::<u8>() & 63) + if DB { 65 } else { 1 };
    let rw = usize::from(kani::any::<u8>() & 63) + if RB { 65 } else { 1 };
    let (dst, d) = input(dw);
    let (rhs, r) = input(rw);
    let mut dst = if DB {
        let Value::BigUint(x) = dst else {
            panic!("wide operator semantics mismatch");
        };
        Value::BigUint(x)
    } else {
        let Value::U64(x) = dst else {
            panic!("wide operator semantics mismatch");
        };
        Value::U64(x)
    };
    let rhs = if RB {
        let Value::BigUint(x) = rhs else {
            panic!("wide operator semantics mismatch");
        };
        Value::BigUint(x)
    } else {
        let Value::U64(x) = rhs else {
            panic!("wide operator semantics mismatch");
        };
        Value::U64(x)
    };
    let beg = kani::any::<usize>();
    let end = kani::any::<usize>();
    let p = usize::from(kani::any::<u8>() & 127);
    dst.assign(rhs, beg, end);
    let expected = if p < dw {
        veryl_formal_semantics::partial_assignment::assigned_logic_bit(
            d.planes, r.planes, p, beg, end,
        )
    } else {
        Logic4::Zero
    };
    result(&dst, dw, d.signed, p, expected);
    kani::cover!(
        beg == dw - 1 && end == 0,
        "wide assignment full window is reachable"
    );
    kani::cover!(
        beg == end && end == dw - 1,
        "wide assignment last bit is reachable"
    );
    kani::cover!(
        r.planes.0 >> (rw - 1) & 1 == 1 && end == dw - 1,
        "wide assignment shifted high RHS is reachable"
    );
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
fn assignment_native_native_matches_scalar_bits() {
    assignment::<false, false>();
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
fn assignment_native_big_matches_scalar_bits() {
    assignment::<false, true>();
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
fn assignment_big_native_matches_scalar_bits() {
    assignment::<true, false>();
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
fn assignment_big_big_matches_scalar_bits() {
    assignment::<true, true>();
}

// Observational composition of the separately proved expansion contracts
// (native expansion, BigUint expansion and native-to-BigUint conversion).
// These operators do not inspect Cow ownership. No narrowing contexts here.
fn prepared(value: &Value, target: usize, sign: bool) -> std::borrow::Cow<'_, Value> {
    let source = value.width();
    assert!(
        (1..=128).contains(&source) && (source..=128).contains(&target),
        "256-bit expansion summary domain exceeded"
    );
    let (p, m) = veryl_formal_semantics::resize::expanded_planes(
        (value.payload_u128(), value.mask_xz_u128()),
        source,
        target,
        value.signed(),
        sign,
    );
    let signed = value.signed() && (source == target || sign);
    let out = if target <= 64 {
        Value::from_u128(p, m, target, signed)
    } else {
        Value::BigUint(ValueBigUint {
            payload: Box::new(from128(p)),
            mask_xz: Box::new(from128(m)),
            width: target as u32,
            signed,
        })
    };
    std::borrow::Cow::Owned(out)
}
// Wide-only version fixes the output representation without symbolic enum paths.
fn prepared_wide(value: &Value, target: usize, sign: bool) -> std::borrow::Cow<'_, Value> {
    let source = value.width();
    assert!(
        (1..=128).contains(&source) && (65..=128).contains(&target) && source <= target,
        "256-bit expansion summary domain exceeded"
    );
    // Production borrows on equal widths. Preserve the complete numeric value,
    // including invalid high bits, so result assertions detect clipping bugs.
    if source == target {
        return std::borrow::Cow::Borrowed(value);
    }
    let (p, m) = veryl_formal_semantics::resize::expanded_planes(
        (value.payload_u128(), value.mask_xz_u128()),
        source,
        target,
        value.signed(),
        sign,
    );
    std::borrow::Cow::Owned(Value::BigUint(ValueBigUint {
        payload: Box::new(from128(p)),
        mask_xz: Box::new(from128(m)),
        width: target as u32,
        signed: value.signed() && (source == target || sign),
    }))
}
// IEEE 1800-2023 11.5.1: the shifted RHS reaches bit 254 before clipping.
#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
fn assignment_high_intermediate_matches_scalar_bits() {
    init();
    let (mut dst, d) = input(128);
    let (rhs, r) = input(128);
    let p = usize::from(kani::any::<u8>() & 127);
    dst.assign(rhs, 127, 127);
    let expected = veryl_formal_semantics::partial_assignment::assigned_logic_bit(
        d.planes, r.planes, p, 127, 127,
    );
    result(&dst, 128, d.signed, p, expected);
    kani::cover!(
        r.planes.0 >> 127 != 0,
        "assignment bit254 intermediate is reachable"
    );
}
// IEEE 1800-2023 11.4.8 and 11.8.2. Factor context preparation from the
// pointwise operation: the existing expansion proofs map every normalized
// source width1..128 into the prepared 65..128 domain quantified here. Both
// prepared operands have arbitrary four-state planes and independent flags.
// Equal-width calls after preparation are mechanized here. followup.rs checks
// original mixed-width argument wiring under numeric erasure; expansion has
// independent value contracts. Actual-library integration traces remain native.
fn bitwise<const K: u8>() {
    use veryl_formal_semantics::{
        gates::{self, BitOp},
        resize::extended_logic_bit,
    };
    init();
    let w = usize::from(kani::any::<u8>() & 63) + 65;
    let sw = w;
    let tw = w;
    let (x, a) = input(w);
    let (y, b) = input(w);
    let Value::BigUint(x) = x else {
        panic!("wide operator semantics mismatch")
    };
    let Value::BigUint(y) = y else {
        panic!("wide operator semantics mismatch")
    };
    let x = Value::BigUint(x);
    let y = Value::BigUint(y);
    let sign = a.signed && b.signed;
    let (op, kind) = match K {
        0 => (Op::BitAnd, BitOp::And),
        1 => (Op::BitOr, BitOp::Or),
        2 => (Op::BitXor, BitOp::Xor),
        _ => (Op::BitXnor, BitOp::Xnor),
    };
    let p = usize::from(kani::any::<u8>() & 127);
    let ret = op.eval_value_binary(&x, &y, w, sign, &mut MaskCache::default());
    let expected = if p < w {
        gates::binary(
            extended_logic_bit(a.planes, sw, w, a.signed, sign, p),
            extended_logic_bit(b.planes, tw, w, b.signed, sign, p),
            kind,
        )
    } else {
        Logic4::Zero
    };
    result(&ret, w, false, p, expected);
    preserve(&x, a);
    preserve(&y, b);
    kani::cover!(w == 65, "wide bitwise lower boundary is reachable");
    kani::cover!(
        w == 128 && a.planes.1 != 0 && b.planes.1 != 0 && (a.planes.1 | b.planes.1) >> 64 != 0,
        "wide bitwise upper unknown is reachable"
    );
    kani::cover!(
        a.signed && b.signed,
        "wide bitwise signed operands are reachable"
    );
}
fn bit_not() {
    init();
    let w = usize::from(kani::any::<u8>() & 63) + 65;
    let sw = usize::from(kani::any::<u8>() & 127) + 1;
    kani::assume(sw <= w);
    let (x, a) = input(sw);
    let sign: bool = kani::any();
    let p = usize::from(kani::any::<u8>() & 127);
    let ret = Op::BitNot.eval_value_unary(&x, w, sign, &mut MaskCache::default());
    let expected = if p < w {
        veryl_formal_semantics::gates::not(veryl_formal_semantics::resize::extended_logic_bit(
            a.planes, sw, w, a.signed, sign, p,
        ))
    } else {
        Logic4::Zero
    };
    result(&ret, w, a.signed && (w == sw || sign), p, expected);
    preserve(&x, a);
    kani::cover!(
        sw < 65 && w == 128 && sign && a.signed,
        "wide NOT sign expansion is reachable"
    );
    kani::cover!(a.planes.1 != 0 && w == sw, "wide NOT unknown is reachable");
}
// IEEE 1800-2023 11.4.10,11.8.2. As with bitwise, quantify every prepared
// widths 65..128 value and compose the independently proved nonnarrowing source
// expansion (widths 1..128). followup.rs checks original context arguments
// separately. Effective expression sign equals prepared source sign.
// Amount widths 1..128 have arbitrary planes/sign and saturate at usize::MAX.
// Arithmetic right is partitioned by effective sign; the union retains all
// values, widths and amounts while avoiding one oversized solver instance.
fn shift<const K: u8, const SIGN: u8>() {
    use veryl_formal_semantics::shift::{Shift, shifted_resized_logic_bit};
    init();
    let w = usize::from(kani::any::<u8>() & 63) + 65;
    let sw = w;
    let aw = usize::from(kani::any::<u8>() & 127) + 1;
    let (x, mut a) = input(sw);
    let Value::BigUint(mut x) = x else {
        panic!("wide operator semantics mismatch")
    };
    if SIGN < 2 {
        x.signed = SIGN == 1;
        a.signed = SIGN == 1;
    }
    let x = Value::BigUint(x);
    let (y, b) = input(aw);
    let amount = if b.planes.1 != 0 {
        None
    } else {
        Some(b.planes.0.min(usize::MAX as u128) as usize)
    };
    let (op, kind) = match K {
        0 => (Op::LogicShiftL, Shift::LogicalLeft),
        1 => (Op::LogicShiftR, Shift::LogicalRight),
        2 => (Op::ArithShiftL, Shift::ArithmeticLeft),
        _ => (Op::ArithShiftR, Shift::ArithmeticRight),
    };
    let p = usize::from(kani::any::<u8>() & 127);
    let ret = op.eval_value_binary(&x, &y, w, a.signed, &mut MaskCache::default());
    result(
        &ret,
        w,
        K >= 2 && amount.is_some() && a.signed,
        p,
        shifted_resized_logic_bit(a.planes, sw, w, amount, kind, a.signed, p),
    );
    preserve(&x, a);
    preserve(&y, b);
    kani::cover!(
        amount == Some(usize::MAX) && aw == 128,
        "wide shift oversized amount is reachable"
    );
    kani::cover!(
        amount.is_none() && aw > 64 && b.planes.1 >> 64 != 0,
        "wide shift unknown amount is reachable"
    );
    kani::cover!(
        w == 128 && a.signed == (SIGN != 0),
        "wide shift source sign partition is reachable"
    );
    kani::cover!(
        w == 128 && amount == Some(128) && a.planes.0 >> 127 != 0,
        "wide shift bit255 intermediate input is reachable"
    );
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, prepared_wide)]
fn wide_bitwise_and_matches_scalar_bits() {
    bitwise::<0>();
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, prepared_wide)]
fn wide_bitwise_or_matches_scalar_bits() {
    bitwise::<1>();
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, prepared_wide)]
fn wide_bitwise_xor_matches_scalar_bits() {
    bitwise::<2>();
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, prepared_wide)]
fn wide_bitwise_xnor_matches_scalar_bits() {
    bitwise::<3>();
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, prepared_wide)]
fn wide_bit_not_matches_scalar_bits() {
    bit_not();
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, prepared_wide)]
fn wide_shift_left_matches_scalar_bits() {
    shift::<0, 2>();
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, prepared_wide)]
fn wide_shift_right_matches_scalar_bits() {
    shift::<1, 2>();
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, prepared_wide)]
fn wide_shift_arith_left_matches_scalar_bits() {
    shift::<2, 2>();
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, prepared_wide)]
fn wide_shift_arith_right_signed_matches_scalar_bits() {
    shift::<3, 1>();
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, prepared_wide)]
fn wide_shift_arith_right_unsigned_matches_scalar_bits() {
    shift::<3, 0>();
}

// IEEE 1800-2023 11.4.9/11.4.7: scalar four-state fold, independently of
// production mask comparisons and popcount. BigUint sources 65..128, all signs,
// result widths 1..128, all normalized four-state planes.
fn reduction<const K: u8>() {
    use veryl_formal_semantics::gates::{self, Reduction};
    init();
    let sw = usize::from(kani::any::<u8>() & 63) + 65;
    let w = usize::from(kani::any::<u8>() & 127) + 1;
    let (x, a) = input(sw);
    let (op, kind) = match K {
        0 => (Op::BitAnd, Reduction::And),
        1 => (Op::BitNand, Reduction::Nand),
        2 => (Op::BitOr, Reduction::Or),
        3 => (Op::BitNor, Reduction::Nor),
        4 => (Op::BitXor, Reduction::Xor),
        5 => (Op::BitXnor, Reduction::Xnor),
        _ => (Op::LogicNot, Reduction::Nor),
    };
    let p = usize::from(kani::any::<u8>() & 127);
    let ret = op.eval_value_unary(&x, w, a.signed, &mut MaskCache::default());
    let expected = if p == 0 {
        gates::reduced_logic(a.planes, sw, kind)
    } else {
        Logic4::Zero
    };
    result(&ret, w, false, p, expected);
    preserve(&x, a);
    kani::cover!(
        sw == 128 && w == 128,
        "wide reduction maximum widths are reachable"
    );
    kani::cover!(
        a.planes.1 != 0 && p == 0,
        "wide reduction unknown source is reachable"
    );
}

#[kani::proof]
#[kani::unwind(129)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, prepared)]
fn wide_reduction_and_matches_scalar_bits() {
    reduction::<0>();
}

#[kani::proof]
#[kani::unwind(129)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, prepared)]
fn wide_reduction_nand_matches_scalar_bits() {
    reduction::<1>();
}

#[kani::proof]
#[kani::unwind(129)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, prepared)]
fn wide_reduction_or_matches_scalar_bits() {
    reduction::<2>();
}

#[kani::proof]
#[kani::unwind(129)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, prepared)]
fn wide_reduction_nor_matches_scalar_bits() {
    reduction::<3>();
}

#[kani::proof]
#[kani::unwind(129)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, prepared)]
fn wide_reduction_xor_matches_scalar_bits() {
    reduction::<4>();
}

#[kani::proof]
#[kani::unwind(129)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, prepared)]
fn wide_reduction_xnor_matches_scalar_bits() {
    reduction::<5>();
}

#[kani::proof]
#[kani::unwind(129)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, prepared)]
fn wide_reduction_logic_not_matches_scalar_bits() {
    reduction::<6>();
}

mod exceptions;
mod followup;
mod identities;
mod logical;
mod plus;
mod power;

mod ordering;

mod special;

mod division;

mod multiplication;

mod opposite;

mod signed_small;

mod signed_power_two;

mod ones;

mod ones_power;

mod hot_power;

mod unit_quotient;

mod power_quotient;

mod signed_power_quotient;
