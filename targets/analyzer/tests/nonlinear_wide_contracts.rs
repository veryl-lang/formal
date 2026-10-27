use std::{fs, path::Path};
use veryl_formal_runner::{Contract, Mutation};
pub static NATIVE: [Contract; 3] = [
    Contract {
        name: "wide-product",
        group: "nonlinear-wide",
        harness: "products_match_bit_vectors",
        message: "wide nonlinear output mismatch",
        covers: &[],
    },
    Contract {
        name: "wide-divrem",
        group: "nonlinear-wide",
        harness: "division_and_remainder_match_bit_vectors",
        message: "wide nonlinear output mismatch",
        covers: &[],
    },
    Contract {
        name: "wide-power",
        group: "nonlinear-wide",
        harness: "small_powers_match_bit_vectors",
        message: "wide nonlinear output mismatch",
        covers: &[],
    },
];
pub fn catalog(_: bool) -> &'static [Contract] {
    &NATIVE
}
pub fn mutations(repo: &Path, selected: &[&'static Contract]) -> Vec<Mutation> {
    let path = "veryl/crates/analyzer/src/ir/op.rs";
    let original = fs::read_to_string(repo.join(path)).unwrap();
    let mut result = Vec::new();
    for &contract in selected {
        let edits = match contract.name {
            "wide-product" => vec![
                (
                    "wide-product-add",
                    "let mut payload = x.payload() * y.payload();",
                    "let mut payload = x.payload() + y.payload();",
                ),
                (
                    "wide-product-truncate",
                    "let mut payload = x.payload() * y.payload();",
                    "let mut payload = (x.payload() * y.payload()) & b1();",
                ),
            ],
            "wide-divrem" => vec![
                (
                    "wide-divrem-quotient",
                    "let payload = x / y;",
                    "let payload = x % y;",
                ),
                (
                    "wide-divrem-remainder",
                    "let payload = x % y;",
                    "let payload = x / y;",
                ),
            ],
            "wide-power" => vec![
                (
                    "wide-power-zero",
                    "mag.modpow(exp, &modulus)",
                    "mag.modpow(&b0(), &modulus)",
                ),
                (
                    "wide-power-one",
                    "mag.modpow(exp, &modulus)",
                    "mag.modpow(&b1(), &modulus)",
                ),
            ],
            _ => unreachable!(),
        };
        for (name, old, new) in edits {
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
