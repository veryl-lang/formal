use std::fs;
use std::path::PathBuf;
use veryl_formal_runner::{Backend, Contract, Mode, Mutation, Options, Runner, USAGE};
static CONTRACTS: [Contract; 6] = [
    Contract {
        name: "selection-storage",
        group: "selection-layout",
        harness: "selection_storage_boundaries",
        message: "selection layout semantics mismatch",
        covers: &[],
    },
    Contract {
        name: "selection-geometry",
        group: "selection-layout",
        harness: "selection_small_coordinate_geometry",
        message: "selection layout semantics mismatch",
        covers: &[],
    },
    Contract {
        name: "selection-limits",
        group: "selection-layout",
        harness: "selection_limits_and_scalar_offsets",
        message: "selection layout semantics mismatch",
        covers: &[],
    },
    Contract {
        name: "selection-rebased-storage",
        group: "selection-layout",
        harness: "rebased_selection_rejects_empty_storage_and_missing_range_base",
        message: "rebased selection storage semantics mismatch",
        covers: &[],
    },
    Contract {
        name: "selection-metadata",
        group: "selection-layout",
        harness: "selection_metadata_widths_do_not_wrap",
        message: "selection metadata semantics mismatch",
        covers: &[],
    },
    Contract {
        name: "selection-unpacked-storage",
        group: "selection-layout",
        harness: "unpacked_selection_rejects_empty_or_unrepresentable_storage",
        message: "unpacked selection storage semantics mismatch",
        covers: &[],
    },
];
fn replace_once(s: &str, old: &str, new: &str) -> String {
    assert_eq!(s.matches(old).count(), 1, "mutation site changed: {old}");
    s.replacen(old, new, 1)
}
fn main() {
    let options = Options::parse(std::env::args().skip(1)).unwrap_or_else(|e| {
        eprintln!("{e}\n{USAGE}");
        std::process::exit(2)
    });
    if options.help {
        println!(
            "{USAGE}\nThis adapter runs finite native reference comparisons, not Kani proofs."
        );
        return;
    }
    let selected = options.select("analyzer", &CONTRACTS).unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(2)
    });
    if options.list {
        for c in selected {
            println!("analyzer\tselection-layout\t{}\t{}", c.name, c.harness);
        }
        return;
    }
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let repo = manifest.ancestors().nth(4).unwrap();
    let variable = "veryl/crates/analyzer/src/ir/variable.rs";
    let comptime = "veryl/crates/analyzer/src/ir/comptime.rs";
    let mut mutations = Vec::new();
    if options.mode != Mode::Positive {
        for (name, contract_name, path, old, new) in [
            (
                "selection-empty-becomes-singleton",
                "selection-storage",
                variable,
                "if storage == 0 {\n            return None;\n        }",
                "if storage == 0 { return Some((0,0)); }",
            ),
            (
                "selection-overflow-becomes-singleton",
                "selection-storage",
                variable,
                "let storage = if is_array {",
                "if if is_array { r#type.array.iter().all(Option::is_some) && r#type.total_array().is_none() } else { r#type.width().iter().all(Option::is_some) && r#type.total_width().is_none() } { return Some((0,0)); } let storage = if is_array {",
            ),
            (
                "selection-offset-wraps",
                "selection-storage",
                variable,
                "x.checked_mul(base)?.checked_add(base.checked_sub(1)?)?",
                "x.wrapping_mul(base).wrapping_add(base-1)",
            ),
            (
                "selection-interval-count-overflows",
                "selection-storage",
                variable,
                "high.checked_sub(low)?.checked_add(1)?;",
                "high.checked_sub(low)?;",
            ),
            (
                "selection-end-off-by-one",
                "selection-geometry",
                variable,
                "x.checked_mul(base)?.checked_add(base.checked_sub(1)?)?",
                "x.checked_mul(base)?.checked_add(base)?",
            ),
            (
                "selection-array-order-ignored",
                "selection-geometry",
                variable,
                "let (low, high) = if array { pair } else { (pair.1, pair.0) };",
                "let (low, high) = if false { pair } else { (pair.1, pair.0) };",
            ),
            (
                "selection-missing-range-base-is-whole",
                "selection-geometry",
                variable,
                "if self.1.is_some() {\n                return None;\n            }",
                "if false { return None; }",
            ),
            (
                "selection-scalar-limit-bypassed",
                "selection-limits",
                variable,
                "return finish(context, (x, y), is_array);",
                "return Some((x,y));",
            ),
            (
                "selection-whole-limit-bypassed",
                "selection-limits",
                variable,
                "return finish(context, (storage - 1, 0), false);",
                "return Some((storage-1,0));",
            ),
            (
                "rebased-empty-becomes-selection",
                "selection-rebased-storage",
                comptime,
                "        if self.base.total_width()? == 0 || (select.1.is_some() && select.0.is_empty()) {",
                "        if self.base.total_width()==Some(0) { return Some(VarSelect::default()); }\n        if self.base.total_width()? == 0 || (select.1.is_some() && select.0.is_empty()) {",
            ),
            (
                "rebased-missing-range-base-accepted",
                "selection-rebased-storage",
                comptime,
                "        if self.base.total_width()? == 0 || (select.1.is_some() && select.0.is_empty()) {",
                "        if select.1.is_some() && select.0.is_empty() {return Some(VarSelect::default());}\n        if self.base.total_width()? == 0 || (select.1.is_some() && select.0.is_empty()) {",
            ),
            (
                "rebased-zero-range-accepted",
                "selection-rebased-storage",
                comptime,
                "if width == Some(0) {\n                    return None;\n                }",
                "if width == Some(0) {return Some(VarSelect::default());}",
            ),
            (
                "rebased-empty-field-accepted",
                "selection-rebased-storage",
                comptime,
                "if width == 0 {\n                return None;\n            }",
                "if width == 0 {return Some(VarSelect::default());}",
            ),
            (
                "selection-metadata-width-wraps",
                "selection-metadata",
                variable,
                "let width = beg.checked_sub(end)?.checked_add(1)?;",
                "let width = beg.checked_sub(end)?.wrapping_add(1);",
            ),
            (
                "selection-metadata-zero-width-accepted",
                "selection-metadata",
                variable,
                "if !matches!(op, VarSelectOp::Colon) && !end_value.is_xz() && end == 0 {",
                "if false {",
            ),
            (
                "selection-metadata-missing-base-accepted",
                "selection-metadata",
                variable,
                "if self.1.is_some() && self.0.is_empty() {\n            return None;\n        }",
                "if false { return None; }",
            ),
            (
                "unpacked-empty-becomes-singleton",
                "selection-unpacked-storage",
                variable,
                "        if shape.total()? == 0 {\n            return None;\n        }",
                "        if shape.total()==Some(0) {return Some(ArraySelectionRange{source_start:0,result_start:0,length:1});}\n        if shape.total()? == 0 { return None; }",
            ),
            (
                "unpacked-overflow-becomes-singleton",
                "selection-unpacked-storage",
                variable,
                "        if shape.total()? == 0 {\n            return None;\n        }",
                "        if shape.total().is_none() && shape.iter().all(Option::is_some) {return Some(ArraySelectionRange{source_start:0,result_start:0,length:1});}\n        if shape.total()? == 0 { return None; }",
            ),
        ] {
            let contract = CONTRACTS.iter().find(|c| c.name == contract_name).unwrap();
            if !selected.iter().any(|c| c.name == contract_name) {
                continue;
            }
            let original = fs::read_to_string(repo.join(path)).unwrap();
            let start = if contract_name == "selection-unpacked-storage" {
                original.find("    pub fn eval_selection(").unwrap()
            } else if path == variable {
                original.find("impl VarSelect {").unwrap()
            } else {
                0
            };
            let changed = replace_once(&original[start..], old, new);
            let changed = format!("{}{changed}", &original[..start]);
            mutations.push(Mutation {
                name,
                contract,
                path,
                source: changed,
            });
        }
    }
    Runner {
        target: "analyzer",
        repo,
        manifest: &manifest,
        files: &[
            "Cargo.toml",
            "Cargo.lock",
            "README.md",
            "LICENSE-MIT",
            "LICENSE-APACHE",
        ],
        directories: &["veryl", "targets", "semantics", "support"],
    }
    .run_with_backend(
        Backend::NativeTests {
            package: "veryl-formal-contract-tests",
            test_target: "selection_layout",
        },
        options.mode,
        &selected,
        &mutations,
    );
}
