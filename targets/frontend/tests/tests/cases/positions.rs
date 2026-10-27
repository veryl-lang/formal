//! Finite Unicode/newline token and Finder fixtures against an independent character
//! scan. Parser columns count scalars; byte spans count UTF-8 bytes. Synthetic
//! native-boundary metadata does not require allocating enormous source files.
//! No arbitrary tokenization, multiline Finder, or source-provenance theorem.

use veryl_parser::{
    Parser,
    finder::Finder,
    resource_table::{self, StrId},
    token_collector::TokenCollector,
    token_range::TokenRange,
    veryl_token::{Token, TokenSource},
    veryl_walker::VerylWalker,
};

fn location(source: &str, offset: usize) -> (u32, u32) {
    let mut line = 1;
    let mut column = 1;
    for c in source[..offset].chars() {
        if c == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    (line, column)
}
fn sources() -> Vec<String> {
    let mut result = Vec::new();
    for word in ["a", "é", "λ", "日本", "😀", "e\u{301}", "𐐷"] {
        for windows in [false, true] {
            let mut source = format!(
                "/* {word} */ /* pair {word} */\n// {word}\n\tmodule Unicode(a:input logic,result:output logic){{always_comb{{result=a;$display(\"{word}\");        result=a;}}}} // tail\n"
            );
            if windows {
                source = source.replace('\n', "\r\n");
            }
            result.push(source);
        }
    }
    let multiline = "/* α\n 日本😀 */ /* λ */\nmodule Multi(a:input logic,result:output logic){assign result=a;}\n";
    result.push(multiline.into());
    result.push(multiline.replace('\n', "\r\n"));
    result
}
#[test]
fn unicode_token_metadata_and_finder_use_scalar_columns() {
    let mut points = 0;
    let sources = sources();
    let count = sources.len();
    for source in sources {
        let parser = Parser::parse(&source, &"unicode.veryl").unwrap();
        let mut collector = TokenCollector::new(true);
        collector.veryl(&parser.veryl);
        for token in &collector.tokens {
            if token.length == 0 {
                continue;
            }
            let start = token.pos as usize;
            let end = start + token.length as usize;
            assert!(
                source.is_char_boundary(start) && source.is_char_boundary(end),
                "frontend source-position output mismatch: invalid byte boundary"
            );
            assert_eq!(
                &source[start..end],
                token.to_string(),
                "frontend source-position output mismatch: token bytes"
            );
            assert_eq!(
                (token.line, token.column),
                location(&source, start),
                "frontend source-position output mismatch: coordinates"
            );
            let (line, col) = location(&source, end);
            assert_eq!(
                (token.end_line(), token.end_column()),
                (line, col - 1),
                "frontend source-position output mismatch: token extent"
            );
            let span: miette::SourceSpan = TokenRange::from(token).into();
            assert_eq!(
                (span.offset(), span.len()),
                (start, end - start),
                "frontend source-position output mismatch: byte span"
            );
        }
        let mut tokens = TokenCollector::new(false);
        tokens.veryl(&parser.veryl);
        for (line, chars) in source.lines().enumerate() {
            let chars = chars.strip_suffix('\r').unwrap_or(chars);
            for column in 1..=chars.chars().count() + 2 {
                let mut finder = Finder::new();
                finder.line = line as u32 + 1;
                finder.column = column as u32;
                finder.veryl(&parser.veryl);
                let want = tokens
                    .tokens
                    .iter()
                    .find(|t| {
                        t.length > 0
                            && t.line == line as u32 + 1
                            && t.column <= column as u32
                            && column < (t.column as usize + t.to_string().chars().count())
                    })
                    .map(|t| t.id);
                assert_eq!(
                    finder.token.map(|t| t.id),
                    want,
                    "frontend source-position output mismatch: finder: line={} column={column}",
                    line + 1
                );
                points += 1;
            }
        }
    }
    eprintln!("source positions complete sources={count} queries={points}");
}
#[test]
fn token_ranges_include_the_last_tokens_characters() {
    let path = resource_table::insert_path(std::path::Path::new("range.veryl"));
    let source = TokenSource::File {
        path,
        text: Default::default(),
    };
    let beg = Token::new("begin", 2, 4, 5, 10, source);
    let end = Token::new("日本😀end", 3, 7, 13, 30, source);
    let range = TokenRange { beg, end };
    for line in 1..=4 {
        for column in 1..=20 {
            let point = (line, column);
            let want = ((2, 4)..=(3, 12)).contains(&point);
            assert_eq!(
                range.include(path, line, column),
                want,
                "source range extent mismatch: line={line} column={column}"
            );
        }
    }
    let other = resource_table::insert_path(std::path::Path::new("other.veryl"));
    assert!(
        !range.include(other, 2, 4),
        "source range extent mismatch: wrong path"
    );
}
#[test]
fn byte_spans_use_widened_native_boundaries() {
    for (begin, end, length) in [
        (0, 0, 0),
        (0, 1, 1),
        (0, u32::MAX, 1),
        (1, u32::MAX, u32::MAX),
        (u32::MAX, u32::MAX, u32::MAX),
        (123, 456, 789),
    ] {
        let expected = u64::from(end) - u64::from(begin) + u64::from(length);
        if expected > usize::MAX as u64 {
            continue;
        }
        let make = |pos, length| Token {
            id: Default::default(),
            text: StrId::default(),
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
        let owned: miette::SourceSpan = range.into();
        let borrowed: miette::SourceSpan = (&range).into();
        assert_eq!(
            (owned.offset() as u64, owned.len() as u64),
            (u64::from(begin), expected),
            "source span arithmetic mismatch"
        );
        assert_eq!(
            owned, borrowed,
            "source span arithmetic mismatch: owned/borrowed"
        );
    }
}
