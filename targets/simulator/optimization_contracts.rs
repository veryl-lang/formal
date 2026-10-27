//! Finite pass-isolation catalog. Mutations corrupt transformed values rather
//! than merely disabling a pass; detection requires the named output comparison.
use std::{fs, path::Path};
use veryl_formal_runner::{Contract, Mutation};
pub static NATIVE: [Contract; 5] = [
    Contract {
        name: "dead-let",
        group: "pass-shapes",
        harness: "pass_shapes::dead_let_outputs",
        message: "pass shape output mismatch",
        covers: &[],
    },
    Contract {
        name: "lane-merge",
        group: "pass-shapes",
        harness: "pass_shapes::merged_bit_lane_outputs",
        message: "pass shape output mismatch",
        covers: &[],
    },
    Contract {
        name: "opt-interpreter",
        group: "optimization",
        harness: "optimization::interpreter_optimization",
        message: "optimization output mismatch",
        covers: &[],
    },
    Contract {
        name: "opt-cranelift",
        group: "optimization",
        harness: "optimization::cranelift_optimization",
        message: "optimization output mismatch",
        covers: &[],
    },
    Contract {
        name: "opt-aot-c",
        group: "optimization",
        harness: "optimization::aot_c_optimization",
        message: "optimization output mismatch",
        covers: &[],
    },
];
pub fn catalog(_: bool) -> &'static [Contract] {
    &NATIVE
}
pub fn mutations(repo: &Path, selected: &[&'static Contract]) -> Vec<Mutation> {
    let mut result = Vec::new();
    for &contract in selected {
        if matches!(contract.name, "dead-let" | "lane-merge") {
            let (name, path, old, new) = if contract.name == "dead-let" {
                (
                    "dead-let-live-read",
                    "veryl/crates/simulator/src/ir/opt/dead_var_dce.rs",
                    "&& s.reads == 0",
                    "&& s.reads <= 1",
                )
            } else {
                (
                    "lane-merge-xor-to-or",
                    "veryl/crates/simulator/src/ir/opt/lane_vector.rs",
                    "Op::BitOr | Op::BitAnd | Op::BitXor => *op,",
                    "Op::BitOr | Op::BitAnd | Op::BitXor => if *op == Op::BitXor {Op::BitOr}else{*op},",
                )
            };
            let original = fs::read_to_string(repo.join(path)).unwrap();
            assert_eq!(original.matches(old).count(), 1, "mutation anchor {name}");
            result.push(Mutation {
                name,
                contract,
                path,
                source: original.replace(old, new),
            });
            continue;
        }

        let names = match contract.name {
            "opt-interpreter" => ["opt-interpreter-priority", "opt-interpreter-rhs"],
            "opt-cranelift" => ["opt-cranelift-priority", "opt-cranelift-rhs"],
            "opt-aot-c" => ["opt-aot-c-priority", "opt-aot-c-rhs"],
            _ => unreachable!(),
        };
        for (name,(path,old,new)) in names.into_iter().zip([
            ("veryl/crates/simulator/src/ir/opt/version_split.rs","true_expr: Box::new(te),","true_expr: Box::new(fe_.clone()),"),
            ("veryl/crates/simulator/src/ir/opt/comb_fusion.rs","let mut repl = Some(canonical_wrap(expr, w));\n        let (l, r) = stmts.split_at_mut(r_idx);","let mut repl = Some(ProtoExpression::Value { value: Value::new(0,w,false), width:w, expr_context: ExpressionContext {width:w,signed:false} });\n        let (l, r) = stmts.split_at_mut(r_idx);"),
        ]){
            let original=fs::read_to_string(repo.join(path)).unwrap();assert_eq!(original.matches(old).count(),1,"mutation anchor {name}");
            result.push(Mutation{name,contract,path,source:original.replace(old,new)});
        }
    }
    result
}
