//! Finite source-to-gate comparisons over small two-state operands.
use std::fs;
use std::path::Path;
use veryl_formal_runner::{Contract, Mutation};
pub static NATIVE: [Contract; 1] = [Contract {
    name: "source-techmap",
    group: "source-gates",
    harness: "source_to_technology_mapped_gates",
    message: "synthesizer source output mismatch",
    covers: &[],
}];
pub fn catalog(kani: bool) -> &'static [Contract] {
    assert!(!kani);
    &NATIVE
}
pub fn mutations(repo: &Path, selected: &[&'static Contract]) -> Vec<Mutation> {
    let path = "veryl/crates/synthesizer/src/conv/expression.rs";
    let original = fs::read_to_string(repo.join(path)).unwrap();
    let mut result = Vec::new();
    for &contract in selected {
        let names = match contract.name {
            "source-techmap" => [
                "source-techmap-wrong-add",
                "source-techmap-unsigned-compare",
                "source-techmap-shift-fill",
                "source-techmap-zero-extension",
            ],
            _ => unreachable!(),
        };
        for (kind, name) in names.into_iter().enumerate() {
            let (decl, old, new) = match kind {
                0 => (
                    "fn synth_binary(",
                    "arith::ripple_add(ctx, &xs, &ys, NET_CONST0)",
                    "arith::ripple_sub(ctx, &xs, &ys)",
                ),
                1 => (
                    "fn synth_binary(",
                    "let signed = x.comptime().expr_context.signed && y.comptime().expr_context.signed;",
                    "let signed = false;",
                ),
                2 => (
                    "fn synth_binary(",
                    "let signed_ext = matches!(op, Op::ArithShiftR);",
                    "let signed_ext = false;",
                ),
                _ => (
                    "fn resize(",
                    "if signed && !nets.is_empty()",
                    "if false && !nets.is_empty()",
                ),
            };
            let start = original.find(decl).unwrap();
            let end = start + original[start..].find("\n}").unwrap() + 2;
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
