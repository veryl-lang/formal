//! Finite source-to-dependency-graph and summary contracts.
use std::fs;
use std::path::Path;
use veryl_formal_runner::{Contract, Mutation};
pub static NATIVE: [Contract; 4] = [
    Contract {
        name: "graph-direct",
        group: "dependency-graph",
        harness: "direct_three_node_graphs_match_transitive_closure",
        message: "dependency graph outcome mismatch",
        covers: &[],
    },
    Contract {
        name: "graph-hierarchy",
        group: "dependency-graph",
        harness: "hierarchical_three_node_graphs_match_transitive_closure",
        message: "dependency graph outcome mismatch",
        covers: &[],
    },
    Contract {
        name: "graph-summary",
        group: "dependency-graph",
        harness: "child_summary_edges_match_feedback_probes",
        message: "dependency graph outcome mismatch",
        covers: &[],
    },
    Contract {
        name: "graph-incomplete",
        group: "dependency-graph",
        harness: "incomplete_effect_does_not_claim_a_complete_negative",
        message: "dependency graph outcome mismatch",
        covers: &[],
    },
];
pub fn catalog(_: bool) -> &'static [Contract] {
    &NATIVE
}
pub fn mutations(repo: &Path, selected: &[&'static Contract]) -> Vec<Mutation> {
    let path = "veryl/crates/analyzer/src/comb_loop_detect.rs";
    let original = fs::read_to_string(repo.join(path)).unwrap();
    let start = original.find("pub fn check_with_completeness(").unwrap();
    let end = start + original[start..].find("\n}").unwrap() + 2;
    let body = &original[start..end];
    assert_eq!(body.matches("check_inner(ir)").count(), 1);
    let mut result = Vec::new();
    for &contract in selected {
        let names = match contract.name {
            "graph-direct" => ["graph-direct-no-diagnostics", "graph-direct-incomplete"],
            "graph-hierarchy" => [
                "graph-hierarchy-no-diagnostics",
                "graph-hierarchy-incomplete",
            ],
            "graph-summary" => ["graph-summary-no-diagnostics", "graph-summary-incomplete"],
            "graph-incomplete" => [
                "graph-incomplete-no-diagnostics",
                "graph-incomplete-claimed-complete",
            ],
            _ => unreachable!(),
        };
        for (kind, name) in names.into_iter().enumerate() {
            let replacement = if kind == 0 {
                "{ let (_, complete)=check_inner(ir); (Vec::new(),complete) }"
            } else if contract.name == "graph-incomplete" {
                "{ let (errors,_)=check_inner(ir); (errors,true) }"
            } else {
                "{ let (errors,_)=check_inner(ir); (errors,false) }"
            };
            result.push(Mutation {
                name,
                contract,
                path,
                source: format!(
                    "{}{}{}",
                    &original[..start],
                    body.replace("check_inner(ir)", replacement),
                    &original[end..]
                ),
            });
        }
    }
    result
}
