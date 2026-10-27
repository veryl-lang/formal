use std::{fs, path::Path};
use veryl_formal_runner::{Contract, Mutation};
pub fn mutations(repo: &Path, selected: &[&'static Contract]) -> Vec<Mutation> {
    let mut result = Vec::new();
    for &contract in selected
        .iter()
        .filter(|c| c.group == "gate-max" || c.group == "external-wide")
    {
        if contract.group == "gate-max" {
            let names = match contract.name {
                "gate-max-interpreter" => [
                    "gate-max-interpreter-low-cap",
                    "gate-max-interpreter-high-cap",
                    "gate-max-interpreter-initial",
                ],
                "gate-max-cranelift" => [
                    "gate-max-cranelift-low-cap",
                    "gate-max-cranelift-high-cap",
                    "gate-max-cranelift-initial",
                ],
                "gate-max-aot-c" => [
                    "gate-max-aot-c-low-cap",
                    "gate-max-aot-c-high-cap",
                    "gate-max-aot-c-initial",
                ],
                _ => unreachable!(),
            };
            let path = "veryl/crates/simulator/src/ir/opt/event_gate.rs";
            let original = fs::read_to_string(repo.join(path)).unwrap();
            for ((old, new), name) in [
                (
                    "EVENT_GATE_REARM_CAP: u32 = 1 << 16",
                    "EVENT_GATE_REARM_CAP: u32 = 1 << 15",
                ),
                (
                    "EVENT_GATE_REARM_CAP: u32 = 1 << 16",
                    "EVENT_GATE_REARM_CAP: u32 = 1 << 17",
                ),
                (
                    "EVENT_GATE_REARM_FIRES: u32 = 1024",
                    "EVENT_GATE_REARM_FIRES: u32 = 512",
                ),
            ]
            .into_iter()
            .zip(names)
            {
                assert_eq!(original.matches(old).count(), 1);
                result.push(Mutation {
                    name,
                    contract,
                    path,
                    source: original.replacen(old, new, 1),
                });
            }
        } else {
            let names = match contract.name {
                "external-wide-interpreter" => [
                    "external-wide-interpreter-set",
                    "external-wide-interpreter-mark",
                ],
                "external-wide-cranelift" => [
                    "external-wide-cranelift-set",
                    "external-wide-cranelift-mark",
                ],
                "external-wide-aot-c" => ["external-wide-aot-c-set", "external-wide-aot-c-mark"],
                _ => unreachable!(),
            };
            let path = "veryl/crates/simulator/src/simulator.rs";
            let original = fs::read_to_string(repo.join(path)).unwrap();
            for (decl, name) in ["    pub fn set_var_by_id(", "    pub fn mark_ff_written("]
                .into_iter()
                .zip(names)
            {
                let start = original.find(decl).unwrap();
                let end = start + original[start..].find("\n    }").unwrap() + 6;
                let body = &original[start..end];
                let old = "self.invalidate_event_gates();";
                assert_eq!(body.matches(old).count(), 1);
                result.push(Mutation {
                    name,
                    contract,
                    path,
                    source: format!(
                        "{}{}{}",
                        &original[..start],
                        body.replace(old, ""),
                        &original[end..]
                    ),
                });
            }
        }
    }
    result
}
