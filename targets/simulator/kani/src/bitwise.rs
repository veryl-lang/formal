//! Binary scalar truth tables over up to four words. Source offsets range 0..=7;
//! destination offset is one. Exact left/right/double alias and separate sources are
//! modeled from pre-call values. Partial overlap, logical-width masking, and X/Z plane
//! handling are outside these unsafe kernel contracts.

use veryl_formal_semantics::words::{self, StorageBitOp};
use veryl_simulator::wide_ops;

#[kani::proof]
#[kani::unwind(5)]
fn wide_band_matches_reference() {
    bitwise::<0>();
}
#[kani::proof]
#[kani::unwind(5)]
fn wide_bor_matches_reference() {
    bitwise::<1>();
}
#[kani::proof]
#[kani::unwind(5)]
fn wide_bxor_matches_reference() {
    bitwise::<2>();
}
#[kani::proof]
#[kani::unwind(5)]
fn wide_bxor_not_matches_reference() {
    bitwise::<3>();
}
#[kani::proof]
#[kani::unwind(5)]
fn wide_band_not_matches_reference() {
    bitwise::<4>();
}
#[kani::proof]
#[kani::unwind(5)]
fn wide_bnot_matches_reference() {
    bitwise::<5>();
}

fn bitwise<const OP: usize>() {
    let kind = [
        StorageBitOp::And,
        StorageBitOp::Or,
        StorageBitOp::Xor,
        StorageBitOp::Xnor,
        StorageBitOp::AndNot,
        StorageBitOp::Not,
    ][OP];
    let a_data: [u8; 40] = kani::any();
    let b_data: [u8; 40] = kani::any();
    let mut destination: [u8; 34] = kani::any();
    let before = destination;
    let count = usize::from(kani::any::<u8>() % 5);
    let nb = count * 8;
    let a_offset = usize::from(kani::any::<u8>() & 7);
    let b_offset = usize::from(kani::any::<u8>() & 7);
    let dst_offset = 1;
    let alias_a: bool = kani::any();
    let alias_b = if OP == 5 { false } else { kani::any::<bool>() };
    let a = a_data[..a_offset + nb].to_vec();
    let b = b_data[..b_offset + nb].to_vec();
    let observed = usize::from(kani::any::<u8>() % 34);
    let bit = usize::from(kani::any::<u8>() & 7);
    unsafe {
        let dst = if nb == 0 {
            std::ptr::null_mut()
        } else {
            destination.as_mut_ptr().add(dst_offset)
        };
        let x = if nb == 0 {
            std::ptr::null()
        } else if alias_a {
            dst as *const u8
        } else {
            a.as_ptr().add(a_offset)
        };
        let y = if nb == 0 {
            std::ptr::null()
        } else if alias_b {
            dst as *const u8
        } else {
            b.as_ptr().add(b_offset)
        };
        match OP {
            0 => wide_ops::wide_band(dst, x, y, nb as u32),
            1 => wide_ops::wide_bor(dst, x, y, nb as u32),
            2 => wide_ops::wide_bxor(dst, x, y, nb as u32),
            3 => wide_ops::wide_bxor_not(dst, x, y, nb as u32),
            4 => wide_ops::wide_band_not(dst, x, y, nb as u32),
            5 => wide_ops::wide_bnot(dst, x, nb as u32),
            _ => unreachable!(),
        }
    }
    let x = if alias_a {
        &before[dst_offset..dst_offset + nb]
    } else {
        &a_data[a_offset..a_offset + nb]
    };
    let y = if alias_b {
        &before[dst_offset..dst_offset + nb]
    } else {
        &b_data[b_offset..b_offset + nb]
    };
    let inside = observed >= dst_offset && observed - dst_offset < nb;
    let position = observed.saturating_sub(dst_offset) * 8 + bit;
    let x_bit = inside && words::bit(x, position);
    let y_bit = OP != 5 && inside && words::bit(y, position);
    let expected = if inside {
        words::storage_bit(kind, x_bit, y_bit)
    } else {
        words::bit(&before, observed * 8 + bit)
    };
    assert!(
        words::bit(&destination, observed * 8 + bit) == expected,
        "simulator bitwise semantics mismatch"
    );
    let source_observed = usize::from(kani::any::<u8>() % 40);
    if source_observed < a.len() {
        assert!(
            a[source_observed] == a_data[source_observed],
            "simulator bitwise semantics mismatch"
        );
    }
    if source_observed < b.len() {
        assert!(
            b[source_observed] == b_data[source_observed],
            "simulator bitwise semantics mismatch"
        );
    }
    kani::cover!(count == 0, "bitwise empty storage is reachable");
    kani::cover!(count == 1, "bitwise one word is reachable");
    kani::cover!(
        count == 4 && !alias_a && !alias_b,
        "bitwise four disjoint words are reachable"
    );
    kani::cover!(
        count == 4 && alias_a && !alias_b,
        "bitwise left alias is reachable"
    );
    kani::cover!(
        count == 4 && a_offset == 7 && !alias_a,
        "bitwise unaligned left source is reachable"
    );
    kani::cover!(
        count == 4 && inside && position == 64,
        "bitwise word boundary is reachable"
    );
    kani::cover!(
        count == 4 && inside && position == 255,
        "bitwise last bit is reachable"
    );
    kani::cover!(
        count == 4 && observed < dst_offset,
        "bitwise prefix guard is reachable"
    );
    kani::cover!(
        count == 4 && observed >= dst_offset + nb,
        "bitwise suffix guard is reachable"
    );
    if OP == 5 {
        kani::cover!(inside && !x_bit, "bitwise unary zero is reachable");
        kani::cover!(inside && x_bit, "bitwise unary one is reachable");
    } else {
        kani::cover!(
            count == 4 && !alias_a && alias_b,
            "bitwise right alias is reachable"
        );
        kani::cover!(
            count == 4 && alias_a && alias_b,
            "bitwise double alias is reachable"
        );
        kani::cover!(
            count == 4 && b_offset == 7 && !alias_b,
            "bitwise unaligned right source is reachable"
        );
        kani::cover!(
            inside && !x_bit && !y_bit,
            "bitwise inputs zero zero are reachable"
        );
        kani::cover!(
            inside && !x_bit && y_bit,
            "bitwise inputs zero one are reachable"
        );
        kani::cover!(
            inside && x_bit && !y_bit,
            "bitwise inputs one zero are reachable"
        );
        kani::cover!(
            inside && x_bit && y_bit,
            "bitwise inputs one one are reachable"
        );
    }
}
