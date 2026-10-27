//! Source expression composition and mutations of real context boundaries.
//! Named output mismatches must detect cast leakage and signedness corruption;
//! parse/build failures alone do not qualify.
use std::{fs, path::Path};
use veryl_formal_runner::{Contract, Mutation};
pub static NATIVE: [Contract; 2] = [
    Contract {
        name: "expression-small",
        group: "expression-trees",
        harness: "source_expression_trees_small_two_state",
        message: "expression tree output mismatch",
        covers: &[],
    },
    Contract {
        name: "expression-four-state",
        group: "expression-trees",
        harness: "source_expression_trees_four_state_and_word_boundaries",
        message: "expression tree output mismatch",
        covers: &[],
    },
];
pub fn catalog(_: bool) -> &'static [Contract] {
    &NATIVE
}
pub fn mutations(repo: &Path, selected: &[&'static Contract]) -> Vec<Mutation> {
    let mut result = Vec::new();
    for &contract in selected {
        let names = if contract.name == "expression-small" {
            [
                "expr-small-cast-leak",
                "expr-small-cast-sign",
                "expr-small-conditional-sign",
            ]
        } else {
            [
                "expr-four-cast-leak",
                "expr-four-cast-sign",
                "expr-four-conditional-sign",
            ]
        };
        for (name, (path, old, new)) in names.into_iter().zip([
            (
                "veryl/crates/analyzer/src/ir/expression.rs",
                "if *op != Op::As && !op.binary_op_self_determined() {",
                "if !op.binary_op_self_determined() {",
            ),
            (
                "veryl/crates/analyzer/src/ir/op.rs",
                "result.signed = x.signed;",
                "result.signed = false;",
            ),
            (
                "veryl/crates/analyzer/src/ir/expression.rs",
                "signed: y.signed & z.signed,",
                "signed: y.signed | z.signed,",
            ),
        ]) {
            let original = fs::read_to_string(repo.join(path)).unwrap();
            assert_eq!(
                original.matches(old).count(),
                1,
                "mutation site changed: {name}"
            );
            result.push(Mutation {
                name,
                contract,
                path,
                source: original.replace(old, new),
            });
        }
    }
    result
}
