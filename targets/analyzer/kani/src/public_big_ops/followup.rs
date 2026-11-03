//! Prepared equality/arithmetic kernels and original-operand preparation wiring.
//! Relational kernels and signed conversion are in ordering.rs.
//! Inputs use canonical storage: widths1..64 in U64, widths65..128 in BigUint.
//! Numeric results are checked by the kernels. Wiring separately overapproximates
//! numeric API reads, checking actual operand identity and width/sign arguments.
use super::*;

#[kani::proof]
#[kani::unwind(257)]
fn numeric256_addition_matches_scalar_carry() {
    let a = N {
        lo: kani::any(),
        hi: kani::any(),
    };
    let b = N {
        lo: kani::any(),
        hi: kani::any(),
    };
    let p = usize::from(kani::any::<u8>());
    let mut carry = false;
    let mut selected = false;
    for i in 0..256 {
        let x = a.bit(i);
        let y = b.bit(i);
        let sum = x ^ y ^ carry;
        let next = (x && y) || (x && carry) || (y && carry);
        if i == p {
            selected = sum;
        }
        carry = next;
    }
    let (sum, overflow) = a.overflowing_add(b);
    assert!(
        sum.bit(p) == selected && overflow == carry,
        "wide numeric addition mismatch"
    );
    kani::cover!(
        a.lo == u128::MAX && b.lo == 1 && a.hi == 0 && b.hi == 0,
        "numeric addition carry into bit128 is reachable"
    );
    kani::cover!(
        overflow && sum == N::ZERO,
        "numeric addition carry beyond bit255 is reachable"
    );
}

// Equal-width calls preserve even invalid high bits for result assertions.
// Unequal-width preparation uses the separately proved expansion summary.
pub(super) fn expanded(value: &Value, target: usize, sign: bool) -> std::borrow::Cow<'_, Value> {
    if value.width() == target {
        std::borrow::Cow::Borrowed(value)
    } else {
        prepared(value, target, sign)
    }
}
pub(super) fn wide_input(w: usize) -> (Value, Operand) {
    let (v, a) = input(w);
    let Value::BigUint(v) = v else {
        panic!("wide operator semantics mismatch")
    };
    (Value::BigUint(v), a)
}

// IEEE1800-2023 11.4.5/11.4.6. Every normalized prepared65..128-bit
// operand pair and both operand sign flags; scalar reference folds all bits.
// Result widths1..128 and independent outer signs are quantified. The
// existing expansion contract summarizes the actual boolean-result expansion.
fn equality<const K: u8>() {
    use veryl_formal_semantics::comparison::{Comparison, compared_logic};
    init();
    let w = usize::from(kani::any::<u8>() & 63) + 65;
    let out = usize::from(kani::any::<u8>() & 127) + 1;
    let outer: bool = kani::any();
    let (x, a) = wide_input(w);
    let (y, b) = wide_input(w);
    let (op, kind) = match K {
        0 => (Op::Eq, Comparison::Equal),
        1 => (Op::Ne, Comparison::NotEqual),
        2 => (Op::EqWildcard, Comparison::WildcardEqual),
        _ => (Op::NeWildcard, Comparison::WildcardNotEqual),
    };
    let ret = op.eval_value_binary(&x, &y, out, outer, &mut MaskCache::default());
    let expected = compared_logic(a, b, kind);
    let planes = match expected {
        Logic4::Zero => (0u128, 0u128),
        Logic4::One => (1, 0),
        Logic4::X => (0, 1),
        Logic4::Z => (1, 1),
    };
    assert!(
        (ret.payload_u128(), ret.mask_xz_u128()) == planes,
        "wide operator semantics mismatch"
    );
    result(&ret, out, false, 0, expected);
    preserve(&x, a);
    preserve(&y, b);
    kani::cover!(
        w == 128 && (a.planes.1 | b.planes.1) >> 64 != 0,
        "wide equality upper unknown is reachable"
    );
    kani::cover!(
        a.planes.1 == 0 && b.planes.1 == 0 && a.planes.0 == b.planes.0,
        "wide equality known equal is reachable"
    );
    kani::cover!(
        w == 65 && a.signed && !b.signed,
        "wide equality mixed sign is reachable"
    );
    kani::cover!(
        w == 65 && out == 128,
        "wide equality widens boolean result is reachable"
    );
    kani::cover!(
        w == 128 && out == 1,
        "wide equality narrow result keeps full comparison is reachable"
    );
}

// IEEE1800-2023 11.4.3: modular add/subtract/negate; any X/Z poisons
// the whole result. All prepared65..128-bit planes, operand flags and caller
// signs are quantified. The scalar ripple-carry oracle is independent of the
// production complement/mask sequence and the two-half numeric addition model.
fn arithmetic<const K: u8>() {
    use veryl_formal_semantics::arithmetic::{Arithmetic, arithmetic_bit};
    init();
    let w = usize::from(kani::any::<u8>() & 63) + 65;
    let (x, a) = wide_input(w);
    let (y, b) = wide_input(w);
    let sign: bool = kani::any();
    let p = usize::from(kani::any::<u8>() & 127);
    let mut cache = MaskCache::default();
    let (ret, kind) = match K {
        0 => (
            Op::Add.eval_value_binary(&x, &y, w, sign, &mut cache),
            Arithmetic::Add,
        ),
        1 => (
            Op::Sub.eval_value_binary(&x, &y, w, sign, &mut cache),
            Arithmetic::Subtract,
        ),
        _ => (
            Op::Sub.eval_value_unary(&x, w, sign, &mut cache),
            Arithmetic::Negate,
        ),
    };
    let expected = arithmetic_bit(a.planes, b.planes, w, kind, p);
    result(&ret, w, if K == 2 { a.signed } else { sign }, p, expected);
    preserve(&x, a);
    preserve(&y, b);
    kani::cover!(
        w == 128
            && a.planes.1 == 0
            && b.planes.1 == 0
            && a.planes.0 == u128::MAX
            && b.planes.0 == 1,
        "wide arithmetic carry boundary is reachable"
    );
    kani::cover!(
        a.planes.1 >> 64 != 0,
        "wide arithmetic upper unknown is reachable"
    );
    kani::cover!(
        w == 65 && sign && a.signed,
        "wide arithmetic signed lower boundary is reachable"
    );
}

// Wiring obligations do not depend on arithmetic identities. Erase numeric
// stores and allow each load to vary independently over all256-bit numbers.
// This overapproximates every consistent bounded numeric execution, including
// value-dependent branches. No numeric result claim is made in these harnesses.
fn wiring_read(_: &num_bigint::BigUint) -> N {
    N {
        lo: kani::any(),
        hi: kani::any(),
    }
}
fn wiring_write(_: N) -> num_bigint::BigUint {
    num_bigint::BigUint::ZERO
}
fn wiring_shift(_: N, _: usize) -> N {
    N::ZERO
}
fn wiring_add(_: N, _: N) -> N {
    N::ZERO
}

static mut LEFT: *const Value = std::ptr::null();
static mut RIGHT: *const Value = std::ptr::null();
static mut PREP_WIDTH: usize = 0;
static mut OUT_WIDTH: usize = 0;
static mut PREP_SIGN: bool = false;
static mut FIRST_CALLS: usize = 0;
static mut CALLS: usize = 0;
static mut COMPARISON: bool = false;
fn skeleton(width: usize, signed: bool) -> Value {
    if width <= 64 {
        Value::new(0, width, signed)
    } else {
        Value::BigUint(ValueBigUint {
            payload: Box::new(num_bigint::BigUint::ZERO),
            mask_xz: Box::new(num_bigint::BigUint::ZERO),
            width: width as u32,
            signed,
        })
    }
}
fn original(width: usize, signed: bool) -> Value {
    if width <= 64 {
        let m = u64::MAX >> (64 - width);
        Value::U64(veryl_analyzer::value::ValueU64 {
            payload: kani::any::<u64>() & m,
            mask_xz: kani::any::<u64>() & m,
            width: width as u32,
            signed,
        })
    } else {
        skeleton(width, signed)
    }
}
fn wiring_expand(value: &Value, width: usize, sign: bool) -> std::borrow::Cow<'_, Value> {
    unsafe {
        let i = CALLS;
        CALLS += 1;
        if i < FIRST_CALLS {
            let expected = if i == 0 { LEFT } else { RIGHT };
            assert!(
                std::ptr::eq(value, expected) && width == PREP_WIDTH && sign == PREP_SIGN,
                "wide preparation wiring mismatch"
            );
            let flag = value.signed() && (value.width() == width || sign);
            std::borrow::Cow::Owned(skeleton(width, flag))
        } else {
            assert!(
                i == FIRST_CALLS
                    && width == OUT_WIDTH
                    && !sign
                    && value.width() == if COMPARISON { 1 } else { PREP_WIDTH },
                "wide preparation wiring mismatch"
            );
            if width == value.width() {
                std::borrow::Cow::Borrowed(value)
            } else {
                std::borrow::Cow::Owned(skeleton(width, value.signed()))
            }
        }
    }
}
// IEEE1800-2023 11.8.2,11.4.4/5/6/8/10. Original source widths1..128,
// all source signs; nonnarrowing65..128 contexts. Equality prepares at the
// common operand width independently of result width1..128 and outer sign;
// at least one original operand is BigUint for comparison wiring contracts.
// Relational callers supply the resolved sign (both source signs); unlike
// equality, the low-level API does not ignore an arbitrary caller sign. Caller
// expression-context derivation remains covered by separate context contracts.
// Numeric API relaxation checks argument wiring only. The separately proved
// expansion summaries and universal prepared kernels discharge the value part.
fn wiring<const K: u8>() {
    let ordering = (16..=19).contains(&K);
    let comparison = (9..=12).contains(&K) || ordering;
    let shift = (5..=8).contains(&K);
    let binary = K < 4 || (9..=14).contains(&K) || ordering;
    let sw = usize::from(kani::any::<u8>() & 127) + 1;
    let tw = usize::from(kani::any::<u8>() & 127) + 1;
    let requested = usize::from(kani::any::<u8>() & 63) + 65;
    let prep = if comparison { sw.max(tw) } else { requested };
    kani::assume(prep >= 65 && sw <= prep && (!binary || tw <= prep));
    let out = if comparison {
        usize::from(kani::any::<u8>() & 127) + 1
    } else {
        prep
    };
    let a: bool = kani::any();
    let b: bool = kani::any();
    let outer: bool = kani::any();
    let sign = if K < 4 || comparison {
        a && b
    } else if shift {
        a
    } else {
        outer
    };
    let x = original(sw, a);
    let y = original(tw, b);
    unsafe {
        LEFT = &x;
        RIGHT = &y;
        PREP_WIDTH = prep;
        OUT_WIDTH = out;
        PREP_SIGN = sign;
        FIRST_CALLS = if binary { 2 } else { 1 };
        CALLS = 0;
        COMPARISON = comparison;
    }
    let op = match K {
        0 => Op::BitAnd,
        1 => Op::BitOr,
        2 => Op::BitXor,
        3 => Op::BitXnor,
        4 => Op::BitNot,
        5 => Op::LogicShiftL,
        6 => Op::LogicShiftR,
        7 => Op::ArithShiftL,
        8 => Op::ArithShiftR,
        9 => Op::Eq,
        10 => Op::Ne,
        11 => Op::EqWildcard,
        12 => Op::NeWildcard,
        13 => Op::Add,
        14 => Op::Sub,
        16 => Op::Less,
        17 => Op::LessEq,
        18 => Op::Greater,
        19 => Op::GreaterEq,
        _ => Op::Sub,
    };
    let mut cache = MaskCache::default();
    let ret = if binary || shift {
        op.eval_value_binary(
            &x,
            &y,
            out,
            if comparison && !ordering { outer } else { sign },
            &mut cache,
        )
    } else {
        op.eval_value_unary(&x, out, sign, &mut cache)
    };
    let calls = unsafe { CALLS };
    let first = if binary { 2 } else { 1 };
    assert!(
        calls >= first
            && calls <= first + usize::from(comparison || shift)
            && (!comparison || calls == first + 1)
            && ret.width() == out,
        "wide preparation wiring mismatch"
    );
    let expected = if K < 4 || comparison {
        false
    } else if shift {
        K >= 7 && calls == first + 1 && a
    } else if K == 4 || K == 15 {
        a && (sw == prep || sign)
    } else {
        sign
    };
    assert!(
        ret.signed() == expected
            && x.width() == sw
            && x.signed() == a
            && y.width() == tw
            && y.signed() == b,
        "wide preparation wiring mismatch"
    );
    kani::cover!(
        sw < 65 && prep == 128 && a,
        "wiring native signed source reaches wide context"
    );
    kani::cover!(
        sw > 64 && prep == 128,
        "wiring BigUint source reaches wide context"
    );
    if binary {
        kani::cover!(
            sw < 65 && tw > 64 && a && !b,
            "wiring mixed storage and signs are reachable"
        );
    }
    if shift {
        kani::cover!(
            tw > 64 && calls == first,
            "wiring unknown wide amount is reachable"
        );
        kani::cover!(
            tw > 64 && calls == first + 1,
            "wiring known wide amount is reachable"
        );
    }
    if comparison {
        if ordering {
            kani::cover!(
                out == 1 && sign,
                "wiring ordering signed operands are reachable"
            );
        } else {
            kani::cover!(
                out == 1 && outer != sign,
                "wiring comparison ignores outer sign"
            );
        }
        kani::cover!(
            out == 128 && prep == 65,
            "wiring comparison separates result width"
        );
    }
}

#[kani::proof]
#[kani::unwind(129)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, expanded)]
fn wide_equality_eq_matches_scalar() {
    equality::<0>();
}

#[kani::proof]
#[kani::unwind(129)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, expanded)]
fn wide_equality_ne_matches_scalar() {
    equality::<1>();
}

#[kani::proof]
#[kani::unwind(129)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, expanded)]
fn wide_equality_wild_eq_matches_scalar() {
    equality::<2>();
}

#[kani::proof]
#[kani::unwind(129)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, expanded)]
fn wide_equality_wild_ne_matches_scalar() {
    equality::<3>();
}

#[kani::proof]
#[kani::unwind(129)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, expanded)]
fn wide_arithmetic_add_matches_scalar() {
    arithmetic::<0>();
}

#[kani::proof]
#[kani::unwind(129)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, expanded)]
fn wide_arithmetic_sub_matches_scalar() {
    arithmetic::<1>();
}

#[kani::proof]
#[kani::unwind(129)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(veryl_analyzer::value::Value::expand, expanded)]
fn wide_arithmetic_neg_matches_scalar() {
    arithmetic::<2>();
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(crate::public_big_ops::get, wiring_read)]
#[kani::stub(crate::public_big_ops::put, wiring_write)]
#[kani::stub(veryl_formal_semantics::bits256::Bits256::checked_shl, wiring_shift)]
#[kani::stub(veryl_formal_semantics::bits256::Bits256::checked_add, wiring_add)]
#[kani::stub(veryl_analyzer::value::Value::expand, wiring_expand)]
fn wide_preparation_and_wiring() {
    wiring::<0>();
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(crate::public_big_ops::get, wiring_read)]
#[kani::stub(crate::public_big_ops::put, wiring_write)]
#[kani::stub(veryl_formal_semantics::bits256::Bits256::checked_shl, wiring_shift)]
#[kani::stub(veryl_formal_semantics::bits256::Bits256::checked_add, wiring_add)]
#[kani::stub(veryl_analyzer::value::Value::expand, wiring_expand)]
fn wide_preparation_or_wiring() {
    wiring::<1>();
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(crate::public_big_ops::get, wiring_read)]
#[kani::stub(crate::public_big_ops::put, wiring_write)]
#[kani::stub(veryl_formal_semantics::bits256::Bits256::checked_shl, wiring_shift)]
#[kani::stub(veryl_formal_semantics::bits256::Bits256::checked_add, wiring_add)]
#[kani::stub(veryl_analyzer::value::Value::expand, wiring_expand)]
fn wide_preparation_xor_wiring() {
    wiring::<2>();
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(crate::public_big_ops::get, wiring_read)]
#[kani::stub(crate::public_big_ops::put, wiring_write)]
#[kani::stub(veryl_formal_semantics::bits256::Bits256::checked_shl, wiring_shift)]
#[kani::stub(veryl_formal_semantics::bits256::Bits256::checked_add, wiring_add)]
#[kani::stub(veryl_analyzer::value::Value::expand, wiring_expand)]
fn wide_preparation_xnor_wiring() {
    wiring::<3>();
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(crate::public_big_ops::get, wiring_read)]
#[kani::stub(crate::public_big_ops::put, wiring_write)]
#[kani::stub(veryl_formal_semantics::bits256::Bits256::checked_shl, wiring_shift)]
#[kani::stub(veryl_formal_semantics::bits256::Bits256::checked_add, wiring_add)]
#[kani::stub(veryl_analyzer::value::Value::expand, wiring_expand)]
fn wide_preparation_not_wiring() {
    wiring::<4>();
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(crate::public_big_ops::get, wiring_read)]
#[kani::stub(crate::public_big_ops::put, wiring_write)]
#[kani::stub(veryl_formal_semantics::bits256::Bits256::checked_shl, wiring_shift)]
#[kani::stub(veryl_formal_semantics::bits256::Bits256::checked_add, wiring_add)]
#[kani::stub(veryl_analyzer::value::Value::expand, wiring_expand)]
fn wide_preparation_left_wiring() {
    wiring::<5>();
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(crate::public_big_ops::get, wiring_read)]
#[kani::stub(crate::public_big_ops::put, wiring_write)]
#[kani::stub(veryl_formal_semantics::bits256::Bits256::checked_shl, wiring_shift)]
#[kani::stub(veryl_formal_semantics::bits256::Bits256::checked_add, wiring_add)]
#[kani::stub(veryl_analyzer::value::Value::expand, wiring_expand)]
fn wide_preparation_right_wiring() {
    wiring::<6>();
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(crate::public_big_ops::get, wiring_read)]
#[kani::stub(crate::public_big_ops::put, wiring_write)]
#[kani::stub(veryl_formal_semantics::bits256::Bits256::checked_shl, wiring_shift)]
#[kani::stub(veryl_formal_semantics::bits256::Bits256::checked_add, wiring_add)]
#[kani::stub(veryl_analyzer::value::Value::expand, wiring_expand)]
fn wide_preparation_arith_left_wiring() {
    wiring::<7>();
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(crate::public_big_ops::get, wiring_read)]
#[kani::stub(crate::public_big_ops::put, wiring_write)]
#[kani::stub(veryl_formal_semantics::bits256::Bits256::checked_shl, wiring_shift)]
#[kani::stub(veryl_formal_semantics::bits256::Bits256::checked_add, wiring_add)]
#[kani::stub(veryl_analyzer::value::Value::expand, wiring_expand)]
fn wide_preparation_arith_right_wiring() {
    wiring::<8>();
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(crate::public_big_ops::get, wiring_read)]
#[kani::stub(crate::public_big_ops::put, wiring_write)]
#[kani::stub(veryl_formal_semantics::bits256::Bits256::checked_shl, wiring_shift)]
#[kani::stub(veryl_formal_semantics::bits256::Bits256::checked_add, wiring_add)]
#[kani::stub(veryl_analyzer::value::Value::expand, wiring_expand)]
fn wide_preparation_eq_wiring() {
    wiring::<9>();
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(crate::public_big_ops::get, wiring_read)]
#[kani::stub(crate::public_big_ops::put, wiring_write)]
#[kani::stub(veryl_formal_semantics::bits256::Bits256::checked_shl, wiring_shift)]
#[kani::stub(veryl_formal_semantics::bits256::Bits256::checked_add, wiring_add)]
#[kani::stub(veryl_analyzer::value::Value::expand, wiring_expand)]
fn wide_preparation_ne_wiring() {
    wiring::<10>();
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(crate::public_big_ops::get, wiring_read)]
#[kani::stub(crate::public_big_ops::put, wiring_write)]
#[kani::stub(veryl_formal_semantics::bits256::Bits256::checked_shl, wiring_shift)]
#[kani::stub(veryl_formal_semantics::bits256::Bits256::checked_add, wiring_add)]
#[kani::stub(veryl_analyzer::value::Value::expand, wiring_expand)]
fn wide_preparation_wild_eq_wiring() {
    wiring::<11>();
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(crate::public_big_ops::get, wiring_read)]
#[kani::stub(crate::public_big_ops::put, wiring_write)]
#[kani::stub(veryl_formal_semantics::bits256::Bits256::checked_shl, wiring_shift)]
#[kani::stub(veryl_formal_semantics::bits256::Bits256::checked_add, wiring_add)]
#[kani::stub(veryl_analyzer::value::Value::expand, wiring_expand)]
fn wide_preparation_wild_ne_wiring() {
    wiring::<12>();
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(crate::public_big_ops::get, wiring_read)]
#[kani::stub(crate::public_big_ops::put, wiring_write)]
#[kani::stub(veryl_formal_semantics::bits256::Bits256::checked_shl, wiring_shift)]
#[kani::stub(veryl_formal_semantics::bits256::Bits256::checked_add, wiring_add)]
#[kani::stub(veryl_analyzer::value::Value::expand, wiring_expand)]
fn wide_preparation_add_wiring() {
    wiring::<13>();
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(crate::public_big_ops::get, wiring_read)]
#[kani::stub(crate::public_big_ops::put, wiring_write)]
#[kani::stub(veryl_formal_semantics::bits256::Bits256::checked_shl, wiring_shift)]
#[kani::stub(veryl_formal_semantics::bits256::Bits256::checked_add, wiring_add)]
#[kani::stub(veryl_analyzer::value::Value::expand, wiring_expand)]
fn wide_preparation_sub_wiring() {
    wiring::<14>();
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(crate::public_big_ops::get, wiring_read)]
#[kani::stub(crate::public_big_ops::put, wiring_write)]
#[kani::stub(veryl_formal_semantics::bits256::Bits256::checked_shl, wiring_shift)]
#[kani::stub(veryl_formal_semantics::bits256::Bits256::checked_add, wiring_add)]
#[kani::stub(veryl_analyzer::value::Value::expand, wiring_expand)]
fn wide_preparation_neg_wiring() {
    wiring::<15>();
}

// All ownership variants must preserve the same exact sum. The independent
// scalar-carry theorem above establishes overflowing_add's specification.
// This API contract admits all256-bit pairs whose sum fits the bounded model.
#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
fn numeric256_addition_api_preserves_high_half() {
    init();
    let x = N {
        lo: kani::any(),
        hi: kani::any(),
    };
    let y = N {
        lo: kani::any(),
        hi: kani::any(),
    };
    let (sum, overflow) = x.overflowing_add(y);
    kani::assume(!overflow);
    let a = put(x);
    let b = put(y);
    assert!(
        get(&(a.clone() + b.clone())) == sum
            && get(&(&a + b.clone())) == sum
            && get(&(a.clone() + &b)) == sum
            && get(&(&a + &b)) == sum,
        "wide numeric addition API mismatch"
    );
    let mut c = a.clone();
    c += b.clone();
    assert!(get(&c) == sum, "wide numeric addition API mismatch");
    let mut d = a.clone();
    d += &b;
    assert!(
        get(&d) == sum && get(&a) == x && get(&b) == y,
        "wide numeric addition API mismatch"
    );
    kani::cover!(
        x.lo == u128::MAX && y.lo == 1 && x.hi == 0 && y.hi == 0,
        "addition API carry into high half is reachable"
    );
    kani::cover!(sum.hi >> 127 != 0, "addition API bit255 is reachable");
    kani::cover!(sum == N::ZERO, "addition API zero sum is reachable");
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(crate::public_big_ops::get, wiring_read)]
#[kani::stub(crate::public_big_ops::put, wiring_write)]
#[kani::stub(veryl_formal_semantics::bits256::Bits256::checked_shl, wiring_shift)]
#[kani::stub(veryl_formal_semantics::bits256::Bits256::checked_add, wiring_add)]
#[kani::stub(veryl_analyzer::value::Value::expand, wiring_expand)]
fn wide_preparation_less_wiring() {
    wiring::<16>();
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(crate::public_big_ops::get, wiring_read)]
#[kani::stub(crate::public_big_ops::put, wiring_write)]
#[kani::stub(veryl_formal_semantics::bits256::Bits256::checked_shl, wiring_shift)]
#[kani::stub(veryl_formal_semantics::bits256::Bits256::checked_add, wiring_add)]
#[kani::stub(veryl_analyzer::value::Value::expand, wiring_expand)]
fn wide_preparation_less_eq_wiring() {
    wiring::<17>();
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(crate::public_big_ops::get, wiring_read)]
#[kani::stub(crate::public_big_ops::put, wiring_write)]
#[kani::stub(veryl_formal_semantics::bits256::Bits256::checked_shl, wiring_shift)]
#[kani::stub(veryl_formal_semantics::bits256::Bits256::checked_add, wiring_add)]
#[kani::stub(veryl_analyzer::value::Value::expand, wiring_expand)]
fn wide_preparation_greater_wiring() {
    wiring::<18>();
}

#[kani::proof]
#[kani::unwind(3)]
#[kani::use_stub_set(numeric256)]
#[kani::stub(crate::public_big_ops::get, wiring_read)]
#[kani::stub(crate::public_big_ops::put, wiring_write)]
#[kani::stub(veryl_formal_semantics::bits256::Bits256::checked_shl, wiring_shift)]
#[kani::stub(veryl_formal_semantics::bits256::Bits256::checked_add, wiring_add)]
#[kani::stub(veryl_analyzer::value::Value::expand, wiring_expand)]
fn wide_preparation_greater_eq_wiring() {
    wiring::<19>();
}
