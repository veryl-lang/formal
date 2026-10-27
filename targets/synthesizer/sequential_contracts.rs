//! Finite sequential reconstruction catalog and production mutations.
//! Mutants corrupt prior-state reads, reset values, write masks, RAM data routing,
//! or clock connections. Only the selected independent state comparison qualifies as
//! detection; reconstruction failures and unrelated panics do not.

use std::{fs, path::Path};
use veryl_formal_runner::{Contract, Mutation};
pub static NATIVE: [Contract; 1] = [Contract {
    name: "sequential-techmap",
    group: "sequential",
    harness: "sequential_techmap",
    message: "synthesizer sequential output mismatch",
    covers: &[],
}];
pub fn catalog(_: bool) -> &'static [Contract] {
    &NATIVE
}
pub fn mutations(repo: &Path, selected: &[&'static Contract]) -> Vec<Mutation> {
    let mut result = Vec::new();
    for &contract in selected {
        let names = match contract.name {
            "sequential-techmap" => [
                "sequential-techmap-blocking",
                "sequential-techmap-reset",
                "sequential-techmap-mask",
                "sequential-techmap-data",
                "sequential-techmap-ff-clock",
                "sequential-techmap-ram-clock",
            ],
            _ => unreachable!(),
        };
        for ((path, old, new), name) in [
            (
                "veryl/crates/synthesizer/src/conv/expression.rs",
                "ctx.ff_snapshot_reads",
                "false",
            ),
            (
                "veryl/crates/synthesizer/src/conv.rs",
                "self.ffs[*ff_idx].reset_value = *v;",
                "self.ffs[*ff_idx].reset_value = !*v;",
            ),
            (
                "veryl/crates/synthesizer/src/conv/statement.rs",
                "mask[bit] = m;",
                "mask[bit] = NET_CONST1;",
            ),
            (
                "veryl/crates/synthesizer/src/conv/statement.rs",
                "port.data[bit] = data;",
                "port.data[bit] = NET_CONST0;",
            ),
            (
                "veryl/crates/synthesizer/src/conv.rs",
                "self.ffs.push(FfCell {\n                            clock: clock_net,",
                "self.ffs.push(FfCell {\n                            clock: NET_CONST0,",
            ),
            (
                "veryl/crates/synthesizer/src/conv.rs",
                "clock: b.clock,",
                "clock: NET_CONST0,",
            ),
        ]
        .into_iter()
        .zip(names)
        {
            let original = fs::read_to_string(repo.join(path)).unwrap();
            assert_eq!(original.matches(old).count(), 1, "mutation anchor {name}");
            result.push(Mutation {
                name,
                contract,
                path,
                source: original.replacen(old, new, 1),
            });
        }
    }
    result
}
