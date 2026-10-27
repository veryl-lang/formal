//! Internal layout from resolved member widths: structs use declaration-order suffix
//! sums; union members start at zero and are assumed to have equal widths. Spans are
//! (low, width), requiring a representable aggregate before allocation. No source type
//! legality, arbitrary recursive layout, or numeric rebasing of unresolved types is implied.

pub fn struct_width(widths: &[Option<usize>]) -> Option<usize> {
    let sum = widths
        .iter()
        .try_fold(0u128, |sum, width| Some(sum + (*width)? as u128))?;
    usize::try_from(sum).ok()
}

pub fn member_span(widths: &[Option<usize>], member: usize, union: bool) -> Option<(usize, usize)> {
    let width = (*widths.get(member)?)?;
    let low = if union {
        0
    } else {
        struct_width(&widths[member + 1..])?
    };
    Some((low, width))
}
