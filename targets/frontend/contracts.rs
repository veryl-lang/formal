//! Bounded frontend metadata, formatting, and source-map contracts.
use std::fs;
use std::path::Path;
use veryl_formal_runner::{Contract, Mutation};
pub static KANI: [Contract; 1] = [Contract {
    name: "byte-span",
    group: "positions",
    harness: "byte_span_matches_widened_interval",
    message: "frontend byte-span output mismatch",
    covers: &[
        "empty byte interval",
        "span exceeds 32-bit length",
        "native metadata endpoint",
    ],
}];
pub static NATIVE: [Contract; 7] = [
    Contract {
        name: "format-roundtrip",
        group: "format",
        harness: "formatting::formatting_preserves_normalized_ast_comments_and_idempotence",
        message: "formatter semantics mismatch",
        covers: &[],
    },
    Contract {
        name: "unicode-positions",
        group: "positions",
        harness: "positions::unicode_token_metadata_and_finder_use_scalar_columns",
        message: "frontend source-position output mismatch",
        covers: &[],
    },
    Contract {
        name: "range-positions",
        group: "positions",
        harness: "positions::token_ranges_include_the_last_tokens_characters",
        message: "source range extent mismatch",
        covers: &[],
    },
    Contract {
        name: "byte-span",
        group: "positions",
        harness: "positions::byte_spans_use_widened_native_boundaries",
        message: "source span arithmetic mismatch",
        covers: &[],
    },
    Contract {
        name: "source-map",
        group: "mapping",
        harness: "mapping::source_map_roundtrip_uses_one_based_predecessor_coordinates",
        message: "source map coordinate mismatch",
        covers: &[],
    },
    Contract {
        name: "lsp-coordinates",
        group: "lsp",
        harness: "lsp::utf16_positions_match_independent_encoded_prefixes",
        message: "LSP coordinate output mismatch",
        covers: &[],
    },
    Contract {
        name: "lsp-wire",
        group: "lsp",
        harness: "lsp::lsp_definition_diagnostics_and_semantic_tokens_use_utf16",
        message: "LSP wire output mismatch",
        covers: &[],
    },
];
pub fn catalog(kani: bool) -> &'static [Contract] {
    if kani { &KANI } else { &NATIVE }
}
pub fn mutations(repo: &Path, selected: &[&'static Contract]) -> Vec<Mutation> {
    let mut result = Vec::new();
    for &contract in selected {
        let changes: &[(&str, &str, &str, &str, &str)] = match contract.name {
            "format-roundtrip" => &[
                (
                    "format-roundtrip-drop-grouping",
                    "veryl/crates/formatter/src/formatter.rs",
                    "    fn factor(",
                    "                self.l_paren(&x.l_paren);\n                self.expression(&x.expression);\n                self.r_paren(&x.r_paren);",
                    "                self.expression(&x.expression);",
                ),
                (
                    "format-roundtrip-drop-comments",
                    "veryl/crates/formatter/src/formatter.rs",
                    "    fn emit_trailing_comments(",
                    "        if x.comments.is_empty() {",
                    "        if true {",
                ),
                (
                    "format-roundtrip-toggle-newlines",
                    "veryl/crates/formatter/src/formatter.rs",
                    "    pub fn format(",
                    "self.newline = self.format_opt.newline_style.newline_str(raw_input);",
                    "self.newline = if raw_input.contains(\"\\r\\n\") {\"\\n\"} else {\"\\r\\n\"};",
                ),
            ],
            "unicode-positions" => &[
                (
                    "unicode-positions-comment-offset",
                    "veryl/crates/parser/src/veryl_token.rs",
                    "fn split_comment_token(",
                    "pos: token.pos + pos as u32,",
                    "pos: pos as u32 + length,",
                ),
                (
                    "unicode-positions-comment-byte-columns",
                    "veryl/crates/parser/src/veryl_token.rs",
                    "fn split_comment_token(",
                    "column += 1;",
                    "column += character.len_utf8() as u32;",
                ),
                (
                    "unicode-positions-finder-byte-columns",
                    "veryl/crates/parser/src/finder.rs",
                    "    fn veryl_token(",
                    "self.column <= arg.token.end_column()",
                    "self.column < arg.token.column + arg.token.length",
                ),
                (
                    "unicode-positions-token-byte-columns",
                    "veryl/crates/parser/src/veryl_token.rs",
                    "    pub fn end_column(",
                    "text.chars().count() as u32 - 1",
                    "text.len() as u32 - 1",
                ),
            ],
            "range-positions" => &[
                (
                    "range-positions-last-start-only",
                    "veryl/crates/parser/src/token_range.rs",
                    "    pub fn include(",
                    "(self.end.end_line(), self.end.end_column())",
                    "(self.end.line, self.end.column)",
                ),
                (
                    "range-positions-ignore-path",
                    "veryl/crates/parser/src/token_range.rs",
                    "    pub fn include(",
                    "self.beg.source == path",
                    "true",
                ),
            ],
            "byte-span" => &[
                (
                    "byte-span-premature-wrap",
                    "veryl/crates/parser/src/token_range.rs",
                    "impl From<&TokenRange> for miette::SourceSpan",
                    "x.end.pos as usize - x.beg.pos as usize + x.end.length as usize",
                    "x.end.pos.wrapping_sub(x.beg.pos).wrapping_add(x.end.length) as usize",
                ),
                (
                    "byte-span-omit-last-length",
                    "veryl/crates/parser/src/token_range.rs",
                    "impl From<&TokenRange> for miette::SourceSpan",
                    "x.end.pos as usize - x.beg.pos as usize + x.end.length as usize",
                    "x.end.pos as usize - x.beg.pos as usize",
                ),
            ],
            "source-map" => &[
                (
                    "source-map-dst-column-origin",
                    "veryl/crates/sourcemap/src/sourcemap.rs",
                    "    pub fn add(",
                    "let dst_column = dst_column - 1;",
                    "let dst_column = dst_column;",
                ),
                (
                    "source-map-lookup-column-origin",
                    "veryl/crates/sourcemap/src/sourcemap.rs",
                    "    pub fn lookup(",
                    "lookup_token(line - 1, column - 1)",
                    "lookup_token(line - 1, column)",
                ),
            ],
            "lsp-coordinates" => &[
                (
                    "lsp-byte-units",
                    "veryl/crates/languageserver/src/coordinates.rs",
                    "pub fn byte_position(",
                    ".map(char::len_utf16)",
                    ".map(char::len_utf8)",
                ),
                (
                    "lsp-input-units",
                    "veryl/crates/languageserver/src/coordinates.rs",
                    "pub fn parser_column(",
                    "units += c.len_utf16();",
                    "units += 1;",
                ),
                (
                    "lsp-output-units",
                    "veryl/crates/languageserver/src/coordinates.rs",
                    "pub fn utf16_column(",
                    ".map(char::len_utf16)",
                    ".map(char::len_utf8)",
                ),
            ],
            "lsp-wire" => &[
                (
                    "lsp-input-no-conversion",
                    "veryl/crates/languageserver/src/server.rs",
                    "    fn source_column(",
                    "parser_column(&text, line - 1, column - 1)",
                    "{let _ = &text; column}",
                ),
                (
                    "lsp-diagnostic-byte-columns",
                    "veryl/crates/languageserver/src/server.rs",
                    "fn byte_range(",
                    "let (line, column) = byte_position(rope, offset);",
                    "let (line, _) = byte_position(rope, offset);let column=(offset.min(rope.len_bytes())-rope.line_to_byte(line as usize)) as u32;",
                ),
                (
                    "lsp-location-scalar-columns",
                    "veryl/crates/languageserver/src/server.rs",
                    "fn to_location(",
                    "let range = byte_range(text, token.pos as usize, token.length as usize);",
                    "let _ = text;let range=Range::new(Position::new(token.line-1,token.column-1),Position::new(token.line-1,token.column-1+token.length));",
                ),
                (
                    "lsp-semantic-scalar-columns",
                    "veryl/crates/languageserver/src/server.rs",
                    "    fn semantic_tokens(",
                    "let token_column = range.start.character;",
                    "let token_column = token.column-1;",
                ),
            ],
            _ => unreachable!(),
        };
        for &(name, path, decl, old, new) in changes {
            let original = fs::read_to_string(repo.join(path)).unwrap();
            let start = original.find(decl).unwrap();
            let closing = if !decl.starts_with("    ") {
                "\n}"
            } else {
                "\n    }"
            };
            let end = start + original[start..].find(closing).unwrap() + closing.len();
            let body = &original[start..end];
            assert_eq!(
                body.matches(old).count(),
                1,
                "mutation site changed: {name}"
            );
            result.push(Mutation {
                name,
                contract,
                path,
                source: format!(
                    "{}{}{}",
                    &original[..start],
                    body.replace(old, new),
                    &original[end..]
                ),
            });
        }
    }
    result
}
