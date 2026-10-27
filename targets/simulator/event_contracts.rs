use std::{fs, path::Path};
use veryl_formal_runner::{Contract, Mutation};
pub fn mutations(repo: &Path, selected: &[&'static Contract]) -> Vec<Mutation> {
    let path = "veryl/crates/simulator/src/simulator.rs";
    let original = fs::read_to_string(repo.join(path)).unwrap();
    let mut result = Vec::new();
    for &contract in selected.iter().filter(|c| c.group == "event-state") {
        let names = match contract.name {
            "event-interpreter" => [
                "event-interpreter-early-commit",
                "event-interpreter-no-assertion",
                "event-interpreter-reset-polarity",
                "event-interpreter-drop-edge",
            ],
            "event-cranelift" => [
                "event-cranelift-early-commit",
                "event-cranelift-no-assertion",
                "event-cranelift-reset-polarity",
                "event-cranelift-drop-edge",
            ],
            "event-aot-c" => [
                "event-aot-c-early-commit",
                "event-aot-c-no-assertion",
                "event-aot-c-reset-polarity",
                "event-aot-c-drop-edge",
            ],
            _ => unreachable!(),
        };
        for ((decl, old, new), name) in [
            (
                "    fn step_event_inner(",
                "self.eval_event_stmts(event);",
                "self.eval_event_stmts(event);self.commit_event_log();",
            ),
            (
                "    fn eval_assertion_edges(",
                "if window.assertion_edge {",
                "if false {",
            ),
            (
                "    pub fn set_reset_level(",
                "asserted != self.ir.reset_active_low(id)",
                "asserted == self.ir.reset_active_low(id)",
            ),
            (
                "    fn step_event_inner(",
                "for event in events {\n            self.eval_event_stmts(event);",
                "for event in events.iter().take(1) {\n            self.eval_event_stmts(event);",
            ),
        ]
        .into_iter()
        .zip(names)
        {
            let start = original.find(decl).unwrap();
            let end = start + original[start..].find("\n    }").unwrap() + 6;
            let body = &original[start..end];
            assert_eq!(body.matches(old).count(), 1, "mutation anchor: {name}");
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
