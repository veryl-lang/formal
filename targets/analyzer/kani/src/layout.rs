//! Total products use two arbitrary known native extents. General indexing/ranges
//! use up to three axes with extents 0..=3 or unresolved, and coordinates 0..=7.
//! Separate native-boundary and overflowing [a, 2] families have narrower shapes.
//! No arbitrary-rank native-width indexing or source dimension-resolution theorem follows.

use veryl_analyzer::ir::ShapeRef;
use veryl_formal_semantics::layout;

#[kani::proof]
#[kani::unwind(4)]
fn shape_total_matches_reference_semantics() {
    let a: usize = kani::any();
    let b: usize = kani::any();
    let data = [Some(a), Some(b)];
    let expected = usize::try_from(a as u128 * b as u128).ok();
    assert!(
        ShapeRef::new(&data).total() == expected,
        "shape total semantics mismatch"
    );
    kani::cover!(a == 0, "zero first extent is reachable");
    kani::cover!(b == 0, "zero final extent is reachable");
    kani::cover!(a == 1 && b == 1, "unit shape total is reachable");
    kani::cover!(
        a == usize::MAX && b == 1,
        "maximum first extent is reachable"
    );
    kani::cover!(
        a == 1 && b == usize::MAX,
        "maximum final extent is reachable"
    );
    kani::cover!(
        a == usize::MAX && b == 2,
        "shape product overflow is reachable"
    );
    kani::cover!(
        a == usize::MAX / 2 && b == 2,
        "largest doubled fitting shape is reachable"
    );
    kani::cover!(
        a == usize::MAX / 2 + 1 && b == 2,
        "first doubled overflowing shape is reachable"
    );
    kani::cover!(a == 2 && b == 3, "nontrivial shape product is reachable");
}

fn small_shape() -> [Option<usize>; 4] {
    fn extent() -> Option<usize> {
        let code = kani::any::<u8>() & 7;
        if code == 7 {
            None
        } else {
            Some((code & 3) as usize)
        }
    }
    [extent(), extent(), extent(), extent()]
}
fn coordinates() -> [usize; 4] {
    [
        usize::from(kani::any::<u8>() & 7),
        usize::from(kani::any::<u8>() & 7),
        usize::from(kani::any::<u8>() & 7),
        usize::from(kani::any::<u8>() & 7),
    ]
}

#[kani::proof]
#[kani::unwind(4)]
fn shape_index_matches_reference_semantics() {
    let data = small_shape();
    let indices = coordinates();
    let rank = usize::from(kani::any::<u8>() & 3);
    let arity = usize::from(kani::any::<u8>() % 5);
    let shape = &data[..rank];
    let indices = &indices[..arity];
    assert!(
        ShapeRef::new(shape).calc_index(indices) == layout::index(shape, indices),
        "shape index semantics mismatch"
    );
    kani::cover!(rank == 0, "scalar index shape is reachable");
    kani::cover!(
        rank == 1 && data[0] == Some(1) && arity == 0,
        "unit scalar coercion is reachable"
    );
    kani::cover!(
        rank == 3
            && (data[0] == Some(2) && data[1] == Some(3) && data[2] == Some(2))
            && arity == 3
            && (indices[0] == 1 && indices[1] == 2 && indices[2] == 1),
        "three-dimensional last element is reachable"
    );
    kani::cover!(rank == 2 && arity == 1, "missing coordinate is reachable");
    kani::cover!(arity > rank, "excess coordinates are reachable");
    kani::cover!(
        rank > 0 && data[rank - 1].is_none(),
        "unknown index extent is reachable"
    );
    kani::cover!(
        rank > 0 && data[rank - 1] == Some(0),
        "zero index extent is reachable"
    );
    kani::cover!(
        rank == 2
            && (data[0] == Some(2) && data[1] == Some(3))
            && arity == 2
            && (indices[0] == 0 && indices[1] == 3),
        "inner coordinate spill is reachable"
    );
    kani::cover!(
        rank == 2
            && (data[0] == Some(2) && data[1] == Some(3))
            && arity == 2
            && (indices[0] == 2 && indices[1] == 0),
        "outer coordinate spill is reachable"
    );
    kani::cover!(
        rank == 2
            && (data[0] == Some(2) && data[1] == Some(3))
            && arity == 2
            && (indices[0] == 0 && indices[1] == 0),
        "row-major first element is reachable"
    );
    kani::cover!(
        rank == 2
            && (data[0] == Some(2) && data[1] == Some(3))
            && arity == 2
            && (indices[0] == 1 && indices[1] == 0),
        "row-major second row is reachable"
    );
    kani::cover!(
        rank == 2 && data[0] == Some(0) && data[1].is_none(),
        "unknown alongside zero extent is reachable"
    );
}

#[kani::proof]
#[kani::unwind(4)]
fn shape_range_matches_reference_semantics() {
    let data = small_shape();
    let indices = coordinates();
    let rank = usize::from(kani::any::<u8>() & 3);
    let arity = usize::from(kani::any::<u8>() % 5);
    let shape = &data[..rank];
    let prefix = &indices[..arity];
    assert!(
        ShapeRef::new(shape).calc_range(prefix) == layout::range_three(shape, prefix),
        "shape range semantics mismatch"
    );
    kani::cover!(rank == 0 && arity == 0, "scalar range is reachable");
    kani::cover!(
        rank == 1 && data[0] == Some(1) && arity == 0,
        "unit whole range is reachable"
    );
    kani::cover!(
        rank == 3 && (data[0] == Some(2) && data[1] == Some(3) && data[2] == Some(2)) && arity == 0,
        "three-dimensional whole range is reachable"
    );
    kani::cover!(
        rank == 3
            && (data[0] == Some(2) && data[1] == Some(3) && data[2] == Some(2))
            && arity == 1
            && prefix[0] == 1,
        "outer prefix block is reachable"
    );
    kani::cover!(
        rank == 3
            && (data[0] == Some(2) && data[1] == Some(3) && data[2] == Some(2))
            && arity == 2
            && (prefix[0] == 1 && prefix[1] == 2),
        "inner prefix block is reachable"
    );
    kani::cover!(
        rank == 3
            && (data[0] == Some(2) && data[1] == Some(3) && data[2] == Some(2))
            && arity == 3
            && (prefix[0] == 1 && prefix[1] == 2 && prefix[2] == 1),
        "full coordinate singleton is reachable"
    );
    kani::cover!(
        rank > 0 && data[rank - 1].is_none(),
        "unknown range extent is reachable"
    );
    kani::cover!(
        rank > 0 && data[rank - 1] == Some(0),
        "empty range is reachable"
    );
    kani::cover!(
        rank == 2 && (data[0] == Some(2) && data[1] == Some(3)) && arity == 1 && prefix[0] == 2,
        "invalid prefix is reachable"
    );
    kani::cover!(arity > rank, "excess prefix is reachable");
}

#[kani::proof]
#[kani::unwind(4)]
fn shape_native_boundary_matches_reference_semantics() {
    let extent: usize = kani::any();
    let coordinate: usize = kani::any();
    let data = [if kani::any::<bool>() {
        None
    } else {
        Some(extent)
    }];
    let coordinates = [coordinate, 0];
    let arity = usize::from(kani::any::<u8>() % 3);
    let shape = ShapeRef::new(&data);
    assert!(
        shape.total() == layout::total(&data)
            && shape.calc_index(&coordinates[..1]) == layout::index(&data, &coordinates[..1])
            && shape.calc_range(&coordinates[..arity])
                == layout::range(&data, &coordinates[..arity]),
        "shape native boundary semantics mismatch"
    );
    assert!(
        ShapeRef::new(&[]).total() == Some(1)
            && ShapeRef::new(&[Some(usize::MAX), Some(usize::MAX), Some(0)]).total() == Some(0),
        "shape native boundary semantics mismatch"
    );
    kani::cover!(
        data[0].is_some() && (extent == 0),
        "native empty extent is reachable"
    );
    kani::cover!(
        data[0].is_some() && (extent == 1 && coordinate == 0),
        "native unit coordinate is reachable"
    );
    kani::cover!(
        data[0].is_some() && (extent == usize::MAX && coordinate == usize::MAX - 1 && arity == 1),
        "maximum valid native coordinate is reachable"
    );
    kani::cover!(
        data[0].is_some() && (extent == usize::MAX && coordinate == usize::MAX && arity == 1),
        "maximum invalid native coordinate is reachable"
    );
    kani::cover!(
        data[0].is_some() && (extent == usize::MAX && arity == 0),
        "maximum native whole range is reachable"
    );
    kani::cover!(
        data[0].is_some() && (arity == 2),
        "native excess prefix is reachable"
    );
    kani::cover!(
        data[0].is_some() && (extent == 0 && arity == 0),
        "native empty whole range is reachable"
    );
    kani::cover!(data[0].is_none(), "native unresolved extent is reachable");
}

#[kani::proof]
#[kani::unwind(4)]
fn shape_overflow_rejected() {
    let a: usize = kani::any();
    let b = 2usize;
    kani::assume(a > usize::MAX / 2);
    let data = [Some(a), Some(b)];
    let indices = [kani::any::<usize>(), kani::any::<usize>()];
    let arity = usize::from(kani::any::<u8>() % 3);
    let shape = ShapeRef::new(&data);
    assert!(
        shape.total().is_none()
            && shape.calc_index(&indices).is_none()
            && shape.calc_range(&indices[..arity]).is_none(),
        "shape overflow semantics mismatch"
    );
    kani::cover!(a == usize::MAX, "maximum overflow extent is reachable");
    kani::cover!(
        a == usize::MAX / 2 + 1,
        "first native overflow extent is reachable"
    );
    kani::cover!(arity == 0, "overflow whole selection is reachable");
    kani::cover!(
        arity == 1 && indices[0] == 0,
        "overflow partial selection is reachable"
    );
    kani::cover!(
        arity == 2 && (indices[0] == 0 && indices[1] == 0),
        "overflow full selection is reachable"
    );
}
