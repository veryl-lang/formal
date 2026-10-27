use std::fs;
use std::path::PathBuf;
use veryl_formal_runner::{Backend, Contract, Mode, Mutation, Options, Runner, USAGE};

static CONTRACTS: [Contract; 4] = [
    Contract {
        name: "aggregate-width",
        group: "aggregate",
        harness: "struct_width_native_boundaries",
        message: "aggregate width semantics mismatch",
        covers: &[],
    },
    Contract {
        name: "aggregate-spans",
        group: "aggregate",
        harness: "flat_struct_and_union_spans",
        message: "aggregate member span semantics mismatch",
        covers: &[],
    },
    Contract {
        name: "aggregate-nested",
        group: "aggregate",
        harness: "nested_struct_union_packed_and_unpacked_coordinates",
        message: "aggregate nested coordinate semantics mismatch",
        covers: &[],
    },
    Contract {
        name: "aggregate-invalid-storage",
        group: "aggregate",
        harness: "aggregate_invalid_storage_does_not_wrap",
        message: "aggregate invalid storage semantics mismatch",
        covers: &[],
    },
];

fn replace_once(source: &str, old: &str, new: &str) -> String {
    assert_eq!(
        source.matches(old).count(),
        1,
        "mutation site changed: {old}"
    );
    source.replacen(old, new, 1)
}
fn main() {
    let options = Options::parse(std::env::args().skip(1)).unwrap_or_else(|error| {
        eprintln!("{error}\n{USAGE}");
        std::process::exit(2)
    });
    if options.help {
        println!(
            "{USAGE}\nThis adapter runs finite native reference comparisons, not Kani proofs."
        );
        return;
    }
    let selected = options
        .select("analyzer", &CONTRACTS)
        .unwrap_or_else(|error| {
            eprintln!("{error}");
            std::process::exit(2)
        });
    if options.list {
        for c in selected {
            println!("analyzer\taggregate\t{}\t{}", c.name, c.harness);
        }
        return;
    }
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let repo = manifest.ancestors().nth(4).unwrap();
    let path = "veryl/crates/analyzer/src/ir/comptime.rs";
    let source = fs::read_to_string(repo.join(path)).unwrap();
    let structure_start = source.find("impl TypeKindStruct {").unwrap();
    let union_start = source.find("impl TypeKindUnion {").unwrap();
    let mut mutations = Vec::new();
    if options.mode != Mode::Positive {
        for (name, contract_name, scope, old, new) in [
            (
                "struct-sum-wraps",
                "aggregate-width",
                "struct",
                "ret.checked_add(x.width()?)?",
                "ret.wrapping_add(x.width()?)",
            ),
            (
                "struct-sum-keeps-last",
                "aggregate-width",
                "struct",
                "ret.checked_add(x.width()?)?",
                "x.width()?",
            ),
            (
                "struct-offset-reverses-fields",
                "aggregate-spans",
                "struct",
                "for x in self.members.iter().rev()",
                "for x in self.members.iter()",
            ),
            (
                "struct-offset-loses-prefix",
                "aggregate-spans",
                "struct",
                "x.pos += offset;",
                "x.pos += 0;",
            ),
            (
                "struct-offset-uses-unit-stride",
                "aggregate-spans",
                "struct",
                "offset = next_offset;",
                "offset += 1;",
            ),
            (
                "union-offset-shifts-storage",
                "aggregate-spans",
                "union",
                "ret.append(&mut x.expand_struct_union(path, part_select, base));",
                "let mut paths = x.expand_struct_union(path, part_select, base); for path in &mut paths { path.part_select[part_select.len()].pos += 1; } ret.append(&mut paths);",
            ),
            (
                "nested-coordinate-adds-stride",
                "aggregate-nested",
                "path",
                "Expression::Binary(Box::new(expr), Op::Mul, Box::new(index), comptime.clone())",
                "Expression::Binary(Box::new(expr), Op::Add, Box::new(index), comptime.clone())",
            ),
            (
                "nested-coordinate-loses-member-offset",
                "aggregate-nested",
                "path",
                "Value::new(x.pos as u64, 32, false)",
                "Value::new(0, 32, false)",
            ),
            (
                "nested-coordinate-keeps-consumed-dimensions",
                "aggregate-nested",
                "path",
                "let width = if is_select {",
                "let width = if false {",
            ),
            (
                "nested-range-ignores-element-width",
                "aggregate-nested",
                "path",
                "let rw = range_width?;",
                "let rw = 1usize;",
            ),
            (
                "aggregate-offset-wraps",
                "aggregate-invalid-storage",
                "struct",
                "let Some(next_offset) = offset.checked_add(width) else {\n                return vec![];\n            };",
                "let next_offset=offset.wrapping_add(width);",
            ),
            (
                "aggregate-keeps-partial-overflow-paths",
                "aggregate-invalid-storage",
                "struct",
                "let Some(next_offset) = offset.checked_add(width) else {\n                return vec![];\n            };",
                "let Some(next_offset)=offset.checked_add(width) else {return ret;};",
            ),
        ] {
            let contract = CONTRACTS.iter().find(|c| c.name == contract_name).unwrap();
            if !selected.iter().any(|c| c.name == contract_name) {
                continue;
            }
            let range = match scope {
                "struct" => structure_start..union_start,
                "union" => union_start..source.find("impl TypeKindMember {").unwrap(),
                _ => 0..structure_start,
            };
            let replacement = replace_once(&source[range.clone()], old, new);
            let mut changed = source.clone();
            changed.replace_range(range, &replacement);
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
            test_target: "aggregate",
        },
        options.mode,
        &selected,
        &mutations,
    );
}
