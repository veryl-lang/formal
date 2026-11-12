//! Oracle assurance: the independent 192-bit scalar projection equals bit origins.
//! The source is exactly 24 little-endian bytes, with no X/Z or signed values.
//! Every source bit, nonempty interval inside 192 bits and every output bit are
//! symbolic. Empty intervals expose no source bytes and admit every usize start.
//! This checks reference-model code, not a new Veryl language/production feature.
//! No production helper is an oracle. Whole output bits above the selected width
//! must be zero. A little-endian 64-bit host is the campaign's supported target.
use veryl_formal_semantics::words;
#[kani::proof]
#[kani::unwind(4)]
fn projected192_matches_bit_origins() {
    let source: [u8; 24] = kani::any();
    let empty: bool = kani::any();
    let start = if empty {
        kani::any::<usize>()
    } else {
        usize::from(kani::any::<u8>()) % 192
    };
    let width = if empty {
        0
    } else {
        1 + usize::from(kani::any::<u8>())
    };
    if !empty {
        kani::assume(width <= 192 - start);
    }
    let result = if empty {
        words::project_u192(&[], start, 0)
    } else {
        words::project_u192(&source, start, width)
    };
    let pos = usize::from(kani::any::<u8>()) % 192;
    let actual = if pos < 128 {
        result.0 >> pos & 1 != 0
    } else {
        result.1 >> (pos - 128) & 1 != 0
    };
    let expected = pos < width && words::bit(&source, start + pos);
    assert!(
        actual == expected,
        "independent 192-bit projection mismatch"
    );
    kani::cover!(
        empty && start == usize::MAX,
        "projection192 empty maximum start is reachable"
    );
    kani::cover!(
        !empty && start == 0 && width == 192 && pos == 191,
        "projection192 full high bit is reachable"
    );
    kani::cover!(
        !empty && start == 127 && width == 65 && pos == 64,
        "projection192 crossing128 is reachable"
    );
    kani::cover!(
        !empty && start == 128 && width == 64 && pos == 63,
        "projection192 high-only source is reachable"
    );
    kani::cover!(
        !empty && start == 191 && width == 1 && pos == 0,
        "projection192 final source bit is reachable"
    );
    kani::cover!(
        !empty && start == 64 && width == 128 && pos == 127,
        "projection192 full low limb is reachable"
    );
    kani::cover!(
        !empty && start == 63 && width == 129 && pos == 128,
        "projection192 first high output bit is reachable"
    );
    kani::cover!(
        !empty && width == 1 && pos == 191,
        "projection192 zero upper output is reachable"
    );
}
