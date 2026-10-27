//! Finite native/gate/SV trace catalog. Mutants separately corrupt the
//! synthesizer's prior-state reads and emitted SV's nonblocking assignments.
use std::{fs, path::Path};
use veryl_formal_runner::{Contract, Mutation};
pub static NATIVE: [Contract; 2] = [
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
