use std::{fs, path::Path};
use veryl_formal_runner::{Contract, Mutation};
pub static NATIVE: [Contract; 4] = [
    Contract {
        name: "effectful-member",
        group: "aggregate-dynamic",
        harness: "effectful_member_indices_stay_inside_the_static_prefix",
        message: "effectful member output mismatch",
        covers: &[],
    },
    Contract {
        name: "packed-selector-calls",
        group: "aggregate-dynamic",
        harness: "packed_selector_calls_preserve_count_and_missing_bits",
        message: "packed selector call output mismatch",
        covers: &[],
    },
    Contract {
        name: "dynamic-aggregate",
        group: "aggregate-dynamic",
        harness: "recursive_member_domains_preserve_dynamic_signed_and_unknown_indices",
        message: "dynamic aggregate output mismatch",
        covers: &[],
    },
    Contract {
        name: "aggregate-selector-calls",
        group: "aggregate-dynamic",
        harness: "member_selector_calls_are_evaluated_once",
        message: "dynamic aggregate call output mismatch",
        covers: &[],
    },
];
pub fn catalog(_: bool) -> &'static [Contract] {
    &NATIVE
}
pub fn mutations(repo: &Path, selected: &[&'static Contract]) -> Vec<Mutation> {
    let mut result = Vec::new();
    for &contract in selected {
        let specs = match contract.name {
            "effectful-member" => vec![
                (
                    "effectful-member-prefix",
                    "veryl/crates/analyzer/src/ir/expression.rs",
                    "if let Some(domain) = comptime.member_select_domain {",
                    "if let Some(domain) = None::<crate::ir::MemberSelectDomain> {",
                ),
                (
                    "effectful-member-repeat",
                    "veryl/crates/analyzer/src/ir/expression.rs",
                    "let (beg, end) = evaluated.eval_value(context, &r#type, false)?;",
                    "let (beg, end) = select.eval_value(context, &r#type, false)?;",
                ),
            ],
            "dynamic-aggregate" => vec![
                (
                    "aggregate-unknown-decode",
                    "veryl/crates/analyzer/src/ir/expression.rs",
                    "if unknown {",
                    "if false {",
                ),
                (
                    "aggregate-no-runtime-bounds",
                    "veryl/crates/analyzer/src/ir/comptime.rs",
                    "if let Some(valid) = valid_coordinate {",
                    "if let Some(valid) = None::<Expression> {",
                ),
                (
                    "aggregate-offset",
                    "veryl/crates/analyzer/src/ir/comptime.rs",
                    "Value::new(x.pos as u64, 32, false)",
                    "Value::new(x.pos as u64 + 1, 32, false)",
                ),
            ],
            "packed-selector-calls" => vec![(
                "packed-redecode-calls",
                "veryl/crates/analyzer/src/ir/expression.rs",
                "let (beg, end) = evaluated.eval_value(context, &r#type, false)?;",
                "let (beg, end) = select.eval_value(context, &r#type, false)?;",
            )],
            "aggregate-selector-calls" => vec![
                (
                    "aggregate-repeat-calls",
                    "veryl/crates/analyzer/src/ir/comptime.rs",
                    "Factor::Value(_) => true,",
                    "Factor::FunctionCall(_) | Factor::Value(_) => true,",
                ),
                (
                    "aggregate-redecode-calls",
                    "veryl/crates/analyzer/src/ir/expression.rs",
                    "let (beg, end) = evaluated.eval_value(context, &r#type, false)?;",
                    "let (beg, end) = select.eval_value(context, &r#type, false)?;",
                ),
            ],
            _ => unreachable!(),
        };
        for (name, path, old, new) in specs {
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
