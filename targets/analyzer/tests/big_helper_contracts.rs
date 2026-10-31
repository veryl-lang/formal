//! Exhaustive finite mask comparison and retained public constructor boundaries.
use std::{fs, path::Path};
use veryl_formal_runner::{Contract, Mutation};
pub static NATIVE: [Contract; 2] = [
    Contract {
        name: "big-mask-exhaustive",
        group: "big-helpers",
        harness: "biguint_masks_all_supported_widths",
        message: "public BigUint helper semantics mismatch",
        covers: &[],
    },
    Contract {
        name: "big-constructor-boundaries",
        group: "big-helpers",
        harness: "biguint_public_constructor_boundaries",
        message: "public BigUint helper semantics mismatch",
        covers: &[],
    },
];
pub fn catalog(_: bool) -> &'static [Contract] {
    &NATIVE
}
pub fn mutations(repo: &Path, selected: &[&'static Contract]) -> Vec<Mutation> {
    let path = "veryl/crates/analyzer/src/value.rs";
    let original = fs::read_to_string(repo.join(path)).unwrap();
    let mut result = Vec::new();
    if let Some(&contract) = selected.iter().find(|c| c.name == "big-mask-exhaustive") {
        let old = "ret.push(0xffffffff);";
        assert_eq!(original.matches(old).count(), 1, "mutation site changed");
        result.push(Mutation {
            name: "big-mask-loses-word-MSB",
            contract,
            path,
            source: original.replacen(old, "ret.push(0x7fffffff);", 1),
        });
    }
    if let Some(&contract) = selected.iter().find(|c| c.name == "big-mask-exhaustive") {
        let old = "ret.push((1u32 << remaining) - 1);";
        assert_eq!(original.matches(old).count(), 1, "mutation site changed");
        result.push(Mutation {
            name: "big-mask-loses-partial-low-bit",
            contract,
            path,
            source: original.replacen(old, "ret.push(((1u32 << remaining) - 1) & !1);", 1),
        });
    }
    if let Some(&contract) = selected
        .iter()
        .find(|c| c.name == "big-constructor-boundaries")
    {
        let old = "let hi = (v >> 64) as u64;";
        assert_eq!(original.matches(old).count(), 1, "mutation site changed");
        result.push(Mutation {
            name: "big-conversion-drops-high-word",
            contract,
            path,
            source: original.replacen(old, "let hi = 0u64;", 1),
        });
    }
    if let Some(&contract) = selected
        .iter()
        .find(|c| c.name == "big-constructor-boundaries")
    {
        let old = "let lo = v as u64;";
        assert_eq!(original.matches(old).count(), 1, "mutation site changed");
        result.push(Mutation {
            name: "big-conversion-drops-low-word",
            contract,
            path,
            source: original.replacen(old, "let lo = 0u64;", 1),
        });
    }
    result
}
