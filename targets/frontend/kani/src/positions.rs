//! Ordered u32 begin/end metadata and arbitrary u32 final-token length on a 64-bit
//! target. Owned and borrowed spans must match widened endpoint arithmetic.
//! This is metadata arithmetic, not source validity, provenance, tokenization,
//! UTF-16 positioning, or representability on a 32-bit target.

use veryl_parser::token_range::TokenRange;
use veryl_parser::veryl_token::{Token, TokenSource};

#[kani::proof]
fn byte_span_matches_widened_interval() {
    assert_eq!(usize::BITS, 64, "proof target requires 64-bit usize");
    let begin: u32 = kani::any();
    let end: u32 = kani::any();
    let length: u32 = kani::any();
    kani::assume(begin <= end);
    let make = |pos, length| Token {
        id: Default::default(),
        text: Default::default(),
        line: 1,
        column: 1,
        pos,
        length,
        source: TokenSource::Builtin,
    };
    let range = TokenRange {
        beg: make(begin, 0),
        end: make(end, length),
    };
    let expected_end = u64::from(end) + u64::from(length);
    let expected_length = expected_end - u64::from(begin);
    let borrowed: miette::SourceSpan = (&range).into();
    let owned: miette::SourceSpan = range.into();
    assert_eq!(
        (borrowed.offset() as u64, borrowed.len() as u64),
        (u64::from(begin), expected_length),
        "frontend byte-span output mismatch"
    );
    assert_eq!(owned, borrowed, "frontend byte-span output mismatch");
    kani::cover!(begin == end && length == 0, "empty byte interval");
    kani::cover!(
        expected_length > u64::from(u32::MAX),
        "span exceeds 32-bit length"
    );
    kani::cover!(begin == u32::MAX && length > 0, "native metadata endpoint");
}
