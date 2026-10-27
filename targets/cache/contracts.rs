use std::{fs, path::Path};
use veryl_formal_runner::{Contract, Mutation};
pub static NATIVE: [Contract; 3] = [
    Contract {
        name: "blob-integrity",
        group: "store",
        harness: "corrupted_blobs_miss_and_are_repaired",
        message: "cache integrity output mismatch",
        covers: &[],
    },
    Contract {
        name: "metadata",
        group: "store",
        harness: "metadata_survives_warm_keep_invalidation_and_deletion",
        message: "cache metadata output mismatch",
        covers: &[],
    },
    Contract {
        name: "keys",
        group: "store",
        harness: "global_keys_separate_boundaries_and_clear_old_entries",
        message: "cache key output mismatch",
        covers: &[],
    },
];
pub fn catalog(_: bool) -> &'static [Contract] {
    &NATIVE
}
pub fn mutations(repo: &Path, selected: &[&'static Contract]) -> Vec<Mutation> {
    let path = "veryl/crates/cache/src/lib.rs";
    let original = fs::read_to_string(repo.join(path)).unwrap();
    let mut mutations = Vec::new();
    for &contract in selected {
        let specs = match contract.name {
            "blob-integrity" => vec![
                (
                    "cache-no-digest",
                    "if content_hash(&data) != Path::new(rel).file_stem()?.to_str()? {",
                    "if false {",
                ),
                (
                    "cache-no-repair",
                    "if fs::read(&path).ok().as_deref() != Some(data.as_slice()) {",
                    "if !path.exists() {",
                ),
            ],
            "metadata" => vec![
                (
                    "cache-keep-drops-dependencies",
                    "self.next_files.insert(src.to_string(), entry.clone());",
                    "let mut entry=entry.clone();entry.dependents.clear();self.next_files.insert(src.to_string(),entry);",
                ),
                (
                    "cache-no-invalidation",
                    "entry.fragment = None;",
                    "let _ = &entry.fragment;",
                ),
            ],
            "keys" => vec![
                (
                    "cache-key-no-boundaries",
                    "hasher.update(&(part.len() as u64).to_le_bytes());",
                    "hasher.update(&[]);",
                ),
                (
                    "cache-key-ignore-mismatch",
                    "manifest.global_key == global_key",
                    "true",
                ),
            ],
            _ => unreachable!(),
        };
        for (name, old, new) in specs {
            assert_eq!(original.matches(old).count(), 1, "mutation anchor: {name}");
            mutations.push(Mutation {
                name,
                contract,
                path,
                source: original.replacen(old, new, 1),
            });
        }
    }
    mutations
}
