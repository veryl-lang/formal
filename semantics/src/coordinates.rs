//! Internal resolved endpoints use widened arithmetic. Clamped and checked endpoints
//! are different contracts: checked forms reject zero indexed widths, negative lower
//! positions, and native overflow. Colon preserves supplied order. Callers validate
//! cardinality, storage, signed/X/Z expression decoding, and configured size limits.
//! Selection intervals use forward coordinates and tail products; representable missing
//! positions are left for downstream value selection rather than clipped here.

#[derive(Clone, Copy, Debug)]
pub enum Select {
    Colon,
    Plus,
    Minus,
    Step,
}

pub fn endpoints(op: Select, base: usize, bound: usize, array: bool) -> (usize, usize) {
    let clamp = |x: u128| x.min(usize::MAX as u128) as usize;
    let a = base as u128;
    let b = bound as u128;
    let (high, low) = match op {
        Select::Colon => return (base, bound),
        Select::Plus => ((a + b).saturating_sub(1), a),
        Select::Minus => (a, (a + 1).saturating_sub(b)),
        Select::Step => ((a * b + b).saturating_sub(1), a * b),
    };
    if array {
        (clamp(low), clamp(high))
    } else {
        (clamp(high), clamp(low))
    }
}

pub fn checked_endpoints(
    op: Select,
    base: usize,
    bound: usize,
    array: bool,
) -> Option<(usize, usize)> {
    if matches!(op, Select::Colon) {
        return Some((base, bound));
    }
    let a = base as u128;
    let b = bound as u128;
    let extra = b.checked_sub(1)?;
    let (high, low) = match op {
        Select::Colon => unreachable!(),
        Select::Plus => (a + extra, a),
        Select::Minus => (a, a.checked_sub(extra)?),
        Select::Step => (a * b + extra, a * b),
    };
    let pair = (usize::try_from(high).ok()?, usize::try_from(low).ok()?);
    Some(if array { (pair.1, pair.0) } else { pair })
}

pub fn selection_interval(
    shape: &[Option<usize>],
    element: usize,
    coords: &[usize],
    range: Option<(Select, usize)>,
    array: bool,
    limit: usize,
) -> Option<(usize, usize)> {
    let count = usize::try_from(crate::layout::total(shape)? as u128 * element as u128).ok()?;
    if count == 0 {
        return None;
    }
    if coords.is_empty() {
        return (range.is_none() && count - 1 <= limit).then_some((count - 1, 0));
    }
    let raw = match range {
        Some((op, bound)) => checked_endpoints(op, *coords.last()?, bound, array)?,
        None => (*coords.last()?, *coords.last()?),
    };
    let result = if shape.is_empty() && coords.len() == 1 && !array {
        raw
    } else {
        if coords.len() > shape.len() {
            return None;
        }
        let mut prefix = 0u128;
        for (i, coord) in coords[..coords.len() - 1].iter().enumerate() {
            prefix = prefix
                .checked_mul(shape[i]? as u128)?
                .checked_add(*coord as u128)?;
        }
        let axis = shape[coords.len() - 1]? as u128;
        let tail = crate::layout::total(&shape[coords.len()..])? as u128 * element as u128;
        let (low, high) = if array { raw } else { (raw.1, raw.0) };
        let first = prefix
            .checked_mul(axis)?
            .checked_add(low as u128)?
            .checked_mul(tail)?;
        let last = prefix
            .checked_mul(axis)?
            .checked_add(high as u128)?
            .checked_add(1)?
            .checked_mul(tail)?
            .checked_sub(1)?;
        let pair = (usize::try_from(first).ok()?, usize::try_from(last).ok()?);
        if array { pair } else { (pair.1, pair.0) }
    };
    let (low, high) = if array { result } else { (result.1, result.0) };
    high.checked_sub(low)?.checked_add(1)?;
    (result.0 <= limit && result.1 <= limit).then_some(result)
}
