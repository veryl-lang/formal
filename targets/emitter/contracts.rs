//! Finite generated-SV comparisons through an independent simulator.
use std::fs;
use std::path::Path;
use veryl_formal_runner::{Contract, Mutation};
pub static NATIVE: [Contract; 3] = [
    Contract {
        name: "scalar",
        group: "emitter-scalar",
        harness: "scalars::emitted_precedence_casts_and_widths",
        message: "emitter scalar output mismatch",
        covers: &[],
    },
    Contract {
        name: "select",
        group: "emitter-select",
        harness: "selects::emitted_indexed_selects_preserve_xz_and_boundaries",
        message: "emitter selection output mismatch",
        covers: &[],
    },
    Contract {
        name: "conditional",
        group: "spec-conditional",
        harness: "conditional::conditional_four_state_consensus_matches_external_sv_and_values",
        message: "external conditional output mismatch",
        covers: &[],
    },
];
pub fn catalog(_: bool) -> &'static [Contract] {
    &NATIVE
}
pub fn mutations(repo: &Path, selected: &[&'static Contract]) -> Vec<Mutation> {
    let path = "veryl/crates/emitter/src/emitter.rs";
    let original = fs::read_to_string(repo.join(path)).unwrap();
    let mut result = Vec::new();
    for &contract in selected {
        let changes: &[(&str, &str, &str, &str)] = match contract.name {
            "scalar" => &[
                (
                    "scalar-drop-grouping",
                    "    fn factor(",
                    "                self.l_paren(&x.l_paren);\n                self.expression(&x.expression);\n                self.r_paren(&x.r_paren);",
                    "                self.expression(&x.expression);",
                ),
                (
                    "scalar-short-size-cast",
                    "    fn expression02(",
                    "                    self.base_less(&x.base_less);",
                    "                    self.str(\"1\");",
                ),
                (
                    "scalar-unsigned-type-cast",
                    "    fn expression02(",
                    "CastingType::I32(_) => self.str(\"int'(\"),",
                    "CastingType::I32(_) => self.str(\"$unsigned(\"),",
                ),
            ],
            "select" => &[
                (
                    "select-step-add",
                    "    fn select(",
                    "self.str(\"*(\");",
                    "self.str(\"+(\");",
                ),
                (
                    "select-ascending-to-descending",
                    "    fn select(",
                    "self.select_operator(&x.select_operator);",
                    "if matches!(&*x.select_operator, SelectOperator::PlusColon(_)) {self.str(\"-:\");} else {self.select_operator(&x.select_operator);}",
                ),
                (
                    "select-descending-to-ascending",
                    "    fn select(",
                    "self.select_operator(&x.select_operator);",
                    "if matches!(&*x.select_operator, SelectOperator::MinusColon(_)) {self.str(\"+:\");} else {self.select_operator(&x.select_operator);}",
                ),
            ],
            "conditional" => &[
                (
                    "conditional-unknown-as-false",
                    "    fn if_expression(",
                    "x.question.question_token.replace(\") ? (\")",
                    "x.question.question_token.replace(\") === 1'b1 ? (\")",
                ),
                (
                    "conditional-always-false",
                    "    fn if_expression(",
                    "x.question.question_token.replace(\") ? (\")",
                    "x.question.question_token.replace(\") && 1'b0 ? (\")",
                ),
            ],
            _ => unreachable!(),
        };
        for &(name, decl, old, new) in changes {
            let start = original.find(decl).unwrap();
            let end = start + original[start..].find("\n    }\n").unwrap() + 6;
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
