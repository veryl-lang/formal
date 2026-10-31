//! Joint numeric-kernel/expansion/wiring domains. Keep actual-library value
//! comparisons at the smallest width and native/BigUint storage boundaries.
use veryl_analyzer::value::Value;
use veryl_formal_semantics::comparison::Operand;
pub fn normalized(a: Operand) -> bool {
    (1..=128).contains(&a.width)
        && ((a.planes.0 | a.planes.1) & !(u128::MAX >> (128 - a.width))) == 0
}
pub fn boundary(width: usize) -> bool {
    matches!(width, 1 | 64 | 65 | 128)
}
pub fn shape(v: &Value, width: usize, signed: bool) {
    assert_eq!((v.width(), v.signed()), (width, signed));
    assert_eq!(matches!(v, Value::U64(_)), width <= 64);
    if let Value::BigUint(n) = v {
        assert!(n.payload.bits() <= width as u64 && n.mask_xz.bits() <= width as u64);
    }
}
