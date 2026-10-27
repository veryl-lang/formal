//! Native orchestration of CBMC checks on actual generated C expressions.
use std::fs;
use std::path::Path;
use veryl_formal_runner::{Contract, Mutation};
pub static NATIVE: [Contract; 12] = [
    Contract {
        name: "c-cast-store",
        group: "generated-c",
        harness: "emitted_cast_store_preserves_dirty_padding_and_guards",
        message: "generated C expression mismatch",
        covers: &[],
    },
    Contract {
        name: "c-signed-tree-8",
        group: "generated-c",
        harness: "emitted_signed_tree_8",
        message: "generated C expression mismatch",
        covers: &[],
    },
    Contract {
        name: "c-signed-tree-32",
        group: "generated-c",
        harness: "emitted_signed_tree_32",
        message: "generated C expression mismatch",
        covers: &[],
    },
    Contract {
        name: "c-add-8",
        group: "generated-c",
        harness: "emitted_addition_8",
        message: "generated C expression mismatch",
        covers: &[],
    },
    Contract {
        name: "c-add-32",
        group: "generated-c",
        harness: "emitted_addition_32",
        message: "generated C expression mismatch",
        covers: &[],
    },
    Contract {
        name: "c-add-64",
        group: "generated-c",
        harness: "emitted_addition_64",
        message: "generated C expression mismatch",
        covers: &[],
    },
    Contract {
        name: "c-xor-8",
        group: "generated-c",
        harness: "emitted_xor_8",
        message: "generated C expression mismatch",
        covers: &[],
    },
    Contract {
        name: "c-xor-32",
        group: "generated-c",
        harness: "emitted_xor_32",
        message: "generated C expression mismatch",
        covers: &[],
    },
    Contract {
        name: "c-xor-64",
        group: "generated-c",
        harness: "emitted_xor_64",
        message: "generated C expression mismatch",
        covers: &[],
    },
    Contract {
        name: "c-shift-8",
        group: "generated-c",
        harness: "emitted_shift_8",
        message: "generated C expression mismatch",
        covers: &[],
    },
    Contract {
        name: "c-shift-32",
        group: "generated-c",
        harness: "emitted_shift_32",
        message: "generated C expression mismatch",
        covers: &[],
    },
    Contract {
        name: "c-shift-64",
        group: "generated-c",
        harness: "emitted_shift_64",
        message: "generated C expression mismatch",
        covers: &[],
    },
];
pub fn catalog(_: bool) -> &'static [Contract] {
    &NATIVE
}
pub fn mutations(repo: &Path, selected: &[&'static Contract]) -> Vec<Mutation> {
    let path = "veryl/crates/simulator/src/backend/aot_c/emit.rs";
    let original = fs::read_to_string(repo.join(path)).unwrap();
    let mut result = Vec::new();
    for &contract in selected {
        let changes: &[(&str, &str, &str)] = match contract.name {
            "c-cast-store" => &[
                (
                    "c-cast-store-preserve",
                    "let pos_mask = value_mask << lo;",
                    "let pos_mask = (value_mask << lo) | 1;",
                ),
                (
                    "c-cast-store-cast-bit",
                    "Op::As => Some(xs),",
                    "Op::As => Some(format!(\"(({xs}) ^ 1ULL)\")),",
                ),
            ],
            "c-signed-tree-8" => &[
                (
                    "c-signed-tree-8-order",
                    "let is_signed_cmp = signed\n",
                    "let is_signed_cmp = false && signed\n",
                ),
                (
                    "c-signed-tree-8-sum",
                    "Op::Add => Some(\"+\"),",
                    "Op::Add => Some(\"-\"),",
                ),
            ],
            "c-signed-tree-32" => &[
                (
                    "c-signed-tree-32-order",
                    "let is_signed_cmp = signed\n",
                    "let is_signed_cmp = false && signed\n",
                ),
                (
                    "c-signed-tree-32-sum",
                    "Op::Add => Some(\"+\"),",
                    "Op::Add => Some(\"-\"),",
                ),
            ],
            "c-add-8" => &[
                (
                    "c-add-8-fault-1",
                    "Op::Add => Some(\"+\"),",
                    "Op::Add => Some(\"-\"),",
                ),
                (
                    "c-add-8-fault-2",
                    "Op::Add => Some(\"+\"),",
                    "Op::Add => Some(\"^\"),",
                ),
            ],
            "c-add-32" => &[
                (
                    "c-add-32-fault-1",
                    "Op::Add => Some(\"+\"),",
                    "Op::Add => Some(\"-\"),",
                ),
                (
                    "c-add-32-fault-2",
                    "Op::Add => Some(\"+\"),",
                    "Op::Add => Some(\"^\"),",
                ),
            ],
            "c-add-64" => &[
                (
                    "c-add-64-fault-1",
                    "Op::Add => Some(\"+\"),",
                    "Op::Add => Some(\"-\"),",
                ),
                (
                    "c-add-64-fault-2",
                    "Op::Add => Some(\"+\"),",
                    "Op::Add => Some(\"^\"),",
                ),
            ],
            "c-xor-8" => &[
                (
                    "c-xor-8-fault-1",
                    "Op::BitXor => Some(\"^\"),",
                    "Op::BitXor => Some(\"|\"),",
                ),
                (
                    "c-xor-8-fault-2",
                    "Op::BitXor => Some(\"^\"),",
                    "Op::BitXor => Some(\"&\"),",
                ),
            ],
            "c-xor-32" => &[
                (
                    "c-xor-32-fault-1",
                    "Op::BitXor => Some(\"^\"),",
                    "Op::BitXor => Some(\"|\"),",
                ),
                (
                    "c-xor-32-fault-2",
                    "Op::BitXor => Some(\"^\"),",
                    "Op::BitXor => Some(\"&\"),",
                ),
            ],
            "c-xor-64" => &[
                (
                    "c-xor-64-fault-1",
                    "Op::BitXor => Some(\"^\"),",
                    "Op::BitXor => Some(\"|\"),",
                ),
                (
                    "c-xor-64-fault-2",
                    "Op::BitXor => Some(\"^\"),",
                    "Op::BitXor => Some(\"&\"),",
                ),
            ],
            "c-shift-8" => &[
                (
                    "c-shift-8-fault-1",
                    "Op::LogicShiftR => Some(\">>\"),",
                    "Op::LogicShiftR => Some(\"<<\"),",
                ),
                (
                    "c-shift-8-fault-2",
                    "\"(((__uint128_t)({ys})) >= 64 ? 0 : (({xs}) {c_op} ({ys})))\"",
                    "\"(((__uint128_t)({ys})) >= 1 ? 0 : (({xs}) {c_op} ({ys})))\"",
                ),
            ],
            "c-shift-32" => &[
                (
                    "c-shift-32-fault-1",
                    "Op::LogicShiftR => Some(\">>\"),",
                    "Op::LogicShiftR => Some(\"<<\"),",
                ),
                (
                    "c-shift-32-fault-2",
                    "\"(((__uint128_t)({ys})) >= 64 ? 0 : (({xs}) {c_op} ({ys})))\"",
                    "\"(((__uint128_t)({ys})) >= 1 ? 0 : (({xs}) {c_op} ({ys})))\"",
                ),
            ],
            "c-shift-64" => &[
                (
                    "c-shift-64-fault-1",
                    "Op::LogicShiftR => Some(\">>\"),",
                    "Op::LogicShiftR => Some(\"<<\"),",
                ),
                (
                    "c-shift-64-fault-2",
                    "\"(((__uint128_t)({ys})) >= 64 ? 0 : (({xs}) {c_op} ({ys})))\"",
                    "\"(((__uint128_t)({ys})) >= 1 ? 0 : (({xs}) {c_op} ({ys})))\"",
                ),
            ],
            _ => unreachable!(),
        };
        let start = if contract.name == "c-cast-store" {
            0
        } else {
            original.find("fn emit_expr_inner(").unwrap()
        };
        let end = if contract.name == "c-cast-store" {
            original.len()
        } else {
            start + original[start..].find("\n}").unwrap() + 2
        };
        let body = &original[start..end];
        for &(name, old, new) in changes {
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
