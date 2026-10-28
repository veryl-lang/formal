//! Finite native/gate/SV comparisons of expression values and sequential traces.
//! Mutants target context boundaries, gate extension, and emitted assignments.
use std::{fs, path::Path};
use veryl_formal_runner::{Contract, Mutation};
pub static NATIVE: [Contract; 3] = [
    Contract {
        name: "mixed-expression-cross",
        group: "expression-cross",
        harness: "mixed_expressions::mixed_width_signed_casts_match_all_routes",
        message: "mixed expression output mismatch",
        covers: &[],
    },
    Contract {
        name: "sequential-cross",
        group: "sequential-cross",
        harness: "sequential::pure_calls_and_ff_reads_match_native_gate_and_sv",
        message: "sequential cross-target output mismatch",
        covers: &[],
    },
    Contract {
        name: "ram-clock-cross",
        group: "sequential-cross",
        harness: "ram_clocks::masked_ram_and_simultaneous_clocks",
        message: "RAM clock cross output mismatch",
        covers: &[],
    },
];
pub fn catalog(_: bool) -> &'static [Contract] {
    &NATIVE
}
pub fn mutations(repo: &Path, selected: &[&'static Contract]) -> Vec<Mutation> {
    let mut result = Vec::new();
    for &contract in selected {
        if contract.name == "mixed-expression-cross" {
            for (name, path, declaration, old, new) in [
                (
                    "mixed-cast-context-leak",
                    "veryl/crates/analyzer/src/ir/expression.rs",
                    "",
                    "if *op != Op::As && !op.binary_op_self_determined() {",
                    "if !op.binary_op_self_determined() {",
                ),
                (
                    "mixed-cast-loses-sign",
                    "veryl/crates/analyzer/src/ir/op.rs",
                    "",
                    "result.signed = x.signed;",
                    "result.signed = false;",
                ),
                (
                    "mixed-conditional-sign-union",
                    "veryl/crates/analyzer/src/ir/expression.rs",
                    "",
                    "signed: y.signed & z.signed,",
                    "signed: y.signed | z.signed,",
                ),
                (
                    "mixed-gate-zero-extension",
                    "veryl/crates/synthesizer/src/conv/expression.rs",
                    "fn resize(",
                    "if signed && !nets.is_empty()",
                    "if false && !nets.is_empty()",
                ),
                (
                    "mixed-sv-short-cast",
                    "veryl/crates/emitter/src/emitter.rs",
                    "    fn expression02(",
                    "                    self.base_less(&x.base_less);",
                    "                    self.str(\"1\");",
                ),
            ] {
                let original = fs::read_to_string(repo.join(path)).unwrap();
                let (start, end) = if declaration.is_empty() {
                    (0, original.len())
                } else {
                    let start = original.find(declaration).unwrap();
                    let boundary = if declaration.starts_with("    ") {
                        "\n    }\n"
                    } else {
                        "\n}"
                    };
                    (
                        start,
                        start + original[start..].find(boundary).unwrap() + boundary.len(),
                    )
                };
                let body = &original[start..end];
                assert_eq!(body.matches(old).count(), 1, "mutation anchor {name}");
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
            continue;
        }
        if contract.name == "ram-clock-cross" {
            for (name, path, old, new) in [
                (
                    "ram-clock-native-early-commit",
                    "veryl/crates/simulator/src/simulator.rs",
                    "for event in events {\n            self.eval_event_stmts(event);",
                    "for event in events {\n            self.eval_event_stmts(event);self.commit_event_log();",
                ),
                (
                    "ram-clock-gate-write-mask",
                    "veryl/crates/synthesizer/src/conv/statement.rs",
                    "mask[bit] = m;",
                    "mask[bit] = NET_CONST1;",
                ),
                (
                    "ram-clock-sv-blocking",
                    "veryl/crates/emitter/src/emitter.rs",
                    "fn assignment(&mut self, arg: &Assignment) {\n        let is_nba = self.in_always_ff",
                    "fn assignment(&mut self, arg: &Assignment) {\n        let is_nba = false && self.in_always_ff",
                ),
            ] {
                let original = fs::read_to_string(repo.join(path)).unwrap();
                let (start, end) = if path.ends_with("/simulator.rs") {
                    let start = original.find("    fn step_event_inner(").unwrap();
                    (
                        start,
                        start + original[start..].find("\n    }").unwrap() + 6,
                    )
                } else {
                    (0, original.len())
                };
                let body = &original[start..end];
                assert_eq!(body.matches(old).count(), 1, "mutation anchor {name}");
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
            continue;
        }

        for (name, path, old, new) in [
            (
                "cross-gate-prior-state",
                "veryl/crates/synthesizer/src/conv/expression.rs",
                "ctx.ff_snapshot_reads",
                "false",
            ),
            (
                "cross-sv-blocking",
                "veryl/crates/emitter/src/emitter.rs",
                "fn assignment(&mut self, arg: &Assignment) {\n        let is_nba = self.in_always_ff",
                "fn assignment(&mut self, arg: &Assignment) {\n        let is_nba = false && self.in_always_ff",
            ),
        ] {
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
