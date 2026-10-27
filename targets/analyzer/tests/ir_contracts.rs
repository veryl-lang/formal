//! Finite analyzer IR control and function-call execution contracts.
use std::fs;
use std::path::Path;
use veryl_formal_runner::{Contract, Mutation};
pub static NATIVE: [Contract; 3] = [
    Contract {
        name: "ir-loop-trace",
        group: "ir-execution",
        harness: "loop_and_branch_preserve_each_write",
        message: "IR execution trace mismatch",
        covers: &[],
    },
    Contract {
        name: "ir-actual-trace",
        group: "ir-execution",
        harness: "nested_calls_capture_actuals_before_binding",
        message: "IR execution trace mismatch",
        covers: &[],
    },
    Contract {
        name: "ir-constant-control",
        group: "ir-execution",
        harness: "constant_function_folding_preserves_control_flow",
        message: "IR folded control mismatch",
        covers: &[],
    },
];
pub fn catalog(_: bool) -> &'static [Contract] {
    &NATIVE
}
pub fn mutations(repo: &Path, selected: &[&'static Contract]) -> Vec<Mutation> {
    let mut result = Vec::new();
    for &contract in selected {
        let changes: &[(&str, &str, &str, &str, &str)] = match contract.name {
            "ir-loop-trace" => &[
                (
                    "ir-loop-trace-branch-inverted",
                    "veryl/crates/analyzer/src/ir/statement.rs",
                    "impl IfStatement {",
                    "if cond.is_true() {",
                    "if !cond.is_true() {",
                ),
                (
                    "ir-loop-trace-last-iteration-dropped",
                    "veryl/crates/analyzer/src/ir/statement.rs",
                    "    pub fn eval_iter(",
                    "Some((start..end).collect())",
                    "Some((start..end.saturating_sub(1)).collect())",
                ),
            ],
            "ir-actual-trace" => &[
                (
                    "ir-actual-trace-actual-order-reversed",
                    "veryl/crates/analyzer/src/ir/function.rs",
                    "    fn eval_value_inner(",
                    "for (path, expr) in &self.inputs {",
                    "for (path, expr) in self.inputs.iter().rev() {",
                ),
                (
                    "ir-actual-trace-actual-values-aliased",
                    "veryl/crates/analyzer/src/ir/function.rs",
                    "    fn eval_value_inner(",
                    "for (id, value) in evaluated_inputs {",
                    "let first = evaluated_inputs.first().map(|(_,v)|v.clone()); for (id, value) in evaluated_inputs { let value = first.clone().unwrap_or(value);",
                ),
            ],
            "ir-constant-control" => &[
                (
                    "ir-constant-control-branch-inverted",
                    "veryl/crates/analyzer/src/ir/statement.rs",
                    "impl IfStatement {",
                    "if cond.is_true() {",
                    "if !cond.is_true() {",
                ),
                (
                    "ir-constant-control-last-iteration-dropped",
                    "veryl/crates/analyzer/src/ir/statement.rs",
                    "    pub fn eval_iter(",
                    "Some((start..end).collect())",
                    "Some((start..end.saturating_sub(1)).collect())",
                ),
            ],
            _ => unreachable!(),
        };
        for &(name, path, decl, old, new) in changes {
            let original = fs::read_to_string(repo.join(path)).unwrap();
            let start = original.find(decl).unwrap();
            let suffix = if decl.starts_with("impl") {
                "\n}"
            } else {
                "\n    }"
            };
            let end = start + original[start..].find(suffix).unwrap() + suffix.len();
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
