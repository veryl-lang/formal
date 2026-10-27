//! Finite source graphs and hierarchy summaries compared with independent transitive
//! closure. Three-node graph enumeration and selected feedback/incomplete-effect
//! fixtures do not establish arbitrary graph size or general summary completeness.

use veryl_analyzer::{Analyzer, AnalyzerError, Context, comb_loop_detect, ir};
use veryl_metadata::Metadata;
use veryl_parser::Parser;

fn analyze(source: &str, invalid: bool) -> (bool, bool) {
    let metadata = Metadata::create_default("prj").unwrap();
    let analyzer = Analyzer::new(&metadata);
    analyzer.clear();
    let parser = Parser::parse(source, &"").unwrap();
    let mut context = Context::default();
    let mut air = ir::Ir::default();
    let mut errors = analyzer.analyze_pass1("prj", &parser.veryl);
    errors.extend(Analyzer::analyze_post_pass1());
    errors.extend(analyzer.analyze_pass2(&parser.veryl, &mut context, Some(&mut air)));
    errors.extend(Analyzer::analyze_post_pass2(&air));
    if !invalid {
        assert!(
            errors.iter().all(|e| matches!(
                e,
                AnalyzerError::CombinationalLoop { .. } | AnalyzerError::UnassignVariable { .. }
            )),
            "unexpected graph fixture diagnostics: {errors:?}"
        );
    }
    let (errors, complete) = comb_loop_detect::check_with_completeness(&air);
    (
        errors
            .iter()
            .any(|e| matches!(e, AnalyzerError::CombinationalLoop { .. })),
        complete,
    )
}
fn cyclic(mask: usize) -> bool {
    let mut reach = [[false; 3]; 3];
    for (row, targets) in reach.iter_mut().enumerate() {
        for (column, edge) in targets.iter_mut().enumerate() {
            *edge = mask & (1 << (row * 3 + column)) != 0;
        }
    }
    for middle in 0..3 {
        for begin in 0..3 {
            for end in 0..3 {
                reach[begin][end] |= reach[begin][middle] && reach[middle][end];
            }
        }
    }
    (0..3).any(|i| reach[i][i])
}
fn source(mask: usize, hierarchical: bool) -> String {
    let mut source="module Top(data:input logic,result:output logic<3>){var v0:logic;var v1:logic;var v2:logic;".to_string();
    for row in 0..3 {
        let expression = std::iter::once("data".to_string())
            .chain(
                (0..3)
                    .filter(|&column| mask & (1 << (row * 3 + column)) != 0)
                    .map(|column| format!("v{column}")),
            )
            .collect::<Vec<_>>()
            .join("^");
        source += &if hierarchical {
            format!("inst u{row}:WireLeaf(i:({expression}),o:v{row});")
        } else {
            format!("assign v{row}={expression};")
        };
    }
    source +=
        "assign result={v2,v1,v0};}module WireLeaf(i:input logic,o:output logic){assign o=i;}";
    source
}
fn compare(hierarchical: bool) {
    for mask in 0..512 {
        assert_eq!(
            analyze(&source(mask, hierarchical), false),
            (cyclic(mask), true),
            "dependency graph outcome mismatch: mask={mask} hierarchy={hierarchical}"
        );
    }
}
#[test]
fn direct_three_node_graphs_match_transitive_closure() {
    compare(false);
}
#[test]
fn hierarchical_three_node_graphs_match_transitive_closure() {
    compare(true);
}

#[test]
fn child_summary_edges_match_feedback_probes() {
    for mask in 0..512 {
        for input in 0..3 {
            for output in 0..3 {
                let mut code="module Leaf(i0:input logic,i1:input logic,i2:input logic,o0:output logic,o1:output logic,o2:output logic){".to_string();
                for row in 0..3 {
                    let parts = (0..3)
                        .filter(|&column| mask & (1 << (row * 3 + column)) != 0)
                        .map(|column| format!("i{column}"))
                        .collect::<Vec<_>>();
                    let expression = if parts.is_empty() {
                        "1'b0".into()
                    } else {
                        parts.join("^")
                    };
                    code += &format!("assign o{row}={expression};");
                }
                code += "}module Top(data:input logic<3>,result:output logic<3>){var v0:logic;var v1:logic;var v2:logic;inst leaf:Leaf(";
                for column in 0..3 {
                    code += &format!(
                        "i{column}:{},",
                        if column == input {
                            format!("v{output}")
                        } else {
                            format!("data[{column}]")
                        }
                    );
                }
                code += "o0:v0,o1:v1,o2:v2);assign result={v2,v1,v0};}";
                assert_eq!(
                    analyze(&code, false),
                    (mask & (1 << (output * 3 + input)) != 0, true),
                    "dependency graph outcome mismatch: summary mask={mask} input={input} output={output}"
                );
            }
        }
    }
}

#[test]
fn incomplete_effect_does_not_claim_a_complete_negative() {
    for independent in [false, true] {
        let cycle = if independent {
            "var x:logic;var y:logic;assign x=y^1'b1;assign y=x^1'b1;assign witness=x;"
        } else {
            "assign witness=0;"
        };
        let code = format!(
            "module Top(n:input logic<32>,o:output logic,witness:output logic){{var a:logic;var b:logic;always_comb{{for _index in 0..n{{a=b;missing_function();b=a;}}o=b;}}{cycle}}}"
        );
        assert_eq!(
            analyze(&code, true),
            (independent, false),
            "dependency graph outcome mismatch: incomplete independent={independent}"
        );
    }
}
