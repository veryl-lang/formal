//! Internal row-major storage after dimension resolution. Unknown extents make total
//! size unknown; otherwise zero extents dominate overflow and the empty product is one.
//! Coordinates must be axis-valid. Prefix ranges cover remaining contiguous axes.
//! Scalar index conventions are retained; ranges still reject excess coordinates.
//! Declared ranges, aggregate offsets, and backend byte addresses are outside this model.

pub fn total(shape: &[Option<usize>]) -> Option<usize> {
    if shape.iter().any(Option::is_none) {
        return None;
    }
    if shape.contains(&Some(0)) {
        return Some(0);
    }
    let mut result = 1usize;
    for extent in shape {
        let product = result as u128 * (*extent)? as u128;
        result = usize::try_from(product).ok()?;
    }
    Some(result)
}

pub fn index(shape: &[Option<usize>], coordinates: &[usize]) -> Option<usize> {
    if shape.is_empty() || (shape.len() == 1 && shape[0] == Some(1) && coordinates.is_empty()) {
        return Some(0);
    }
    if coordinates.len() != shape.len() {
        return None;
    }
    let count = total(shape)?;
    if count == 0 {
        return None;
    }
    let mut result = 0u128;
    for (coordinate, extent) in coordinates.iter().zip(shape) {
        let extent = (*extent)?;
        if *coordinate >= extent {
            return None;
        }
        result = result * extent as u128 + *coordinate as u128;
    }
    usize::try_from(result).ok()
}

pub fn range(shape: &[Option<usize>], prefix: &[usize]) -> Option<(usize, usize)> {
    if prefix.len() > shape.len() {
        return None;
    }
    let count = total(shape)?;
    if count == 0 {
        return None;
    }
    let mut position = 0u128;
    for (coordinate, extent) in prefix.iter().zip(shape) {
        let extent = (*extent)?;
        if *coordinate >= extent {
            return None;
        }
        position = position * extent as u128 + *coordinate as u128;
    }
    let block = total(&shape[prefix.len()..])? as u128;
    let first = position * block;
    Some((
        usize::try_from(first).ok()?,
        usize::try_from(first + block - 1).ok()?,
    ))
}

pub fn range_three(shape: &[Option<usize>], prefix: &[usize]) -> Option<(usize, usize)> {
    if shape.len() > 3 || prefix.len() > shape.len() {
        return None;
    }
    let a = shape.first().copied().unwrap_or(Some(1))?;
    let b = shape.get(1).copied().unwrap_or(Some(1))?;
    let c = shape.get(2).copied().unwrap_or(Some(1))?;
    if a == 0 || b == 0 || c == 0 {
        return None;
    }
    let total = (a as u128).checked_mul(b as u128)?.checked_mul(c as u128)?;
    usize::try_from(total).ok()?;
    let x = prefix.first().copied().unwrap_or(0);
    let y = prefix.get(1).copied().unwrap_or(0);
    let z = prefix.get(2).copied().unwrap_or(0);
    if (!prefix.is_empty() && x >= a)
        || (prefix.len() > 1 && y >= b)
        || (prefix.len() > 2 && z >= c)
    {
        return None;
    }
    let first = (x as u128 * b as u128 + y as u128) * c as u128 + z as u128;
    let length = match prefix.len() {
        0 => total,
        1 => b as u128 * c as u128,
        2 => c as u128,
        _ => 1,
    };
    Some((
        usize::try_from(first).ok()?,
        usize::try_from(first + length - 1).ok()?,
    ))
}
