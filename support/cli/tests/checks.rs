use clap::ValueEnum;
use std::{collections::VecDeque, fs, path::Path};
use veryl_formal_ci::{Commands, Environment, Result, checks::*, error};

#[path = "../../../targets/analyzer/kani/src/bin/check_contracts/analyzer.rs"]
mod analyzer;

#[path = "../../../targets/simulator/contracts.rs"]
mod simulator_catalog;

#[derive(Default)]
struct Recording {
    calls: Vec<Invocation>,
    fail_at: Option<usize>,
}
impl Executor for Recording {
    fn execute(&mut self, _: &Path, step: &Invocation) -> Result<()> {
        self.calls.push(step.clone());
        if self.fail_at == Some(self.calls.len()) {
            Err(error("selected step failed"))
        } else {
            Ok(())
        }
    }
}

#[test]
fn rejects_unknown_checks_suites_and_modes() {
    for (check, suite) in [
        ("unknown", "single"),
        ("value-contracts", "core"),
        ("value-contracts", "single"),
        ("simulator-contracts", "single"),
        ("cache-contracts", "; true"),
    ] {
        assert!(plan(check, suite, Mode::All).is_err());
    }
    assert!(Mode::from_str("mutation", false).is_err());
    assert!(Mode::from_str("--mode positive", false).is_err());
}

#[test]
fn positive_mode_retains_tests_and_only_changes_adapter_arguments() {
    for shard in SHARDS {
        let (check, suite) = (shard.check, shard.suite);
        let all = plan(check, suite, Mode::All).unwrap();
        let positive = plan(check, suite, Mode::Positive).unwrap();
        assert_eq!(all.steps.len(), positive.steps.len());
        let mut changed = 0;
        for (a, mut b) in all.steps.into_iter().zip(positive.steps) {
            if b.args.ends_with(&["--mode".into(), "positive".into()]) {
                changed += 1;
                assert!(a.args.contains(&"--example".into()) || a.args.contains(&"--bin".into()));
                b.args.truncate(b.args.len() - 2);
            }
            assert_eq!(a, b);
        }
        assert!(changed > 0);
    }
}

#[test]
fn value_shards_partition_every_proof_and_production_mutation() {
    use std::collections::{BTreeMap, BTreeSet};
    use veryl_formal_runner::Options;

    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let all: Vec<_> = analyzer::CONTRACTS.iter().collect();
    // Only named arithmetic-partition diagnostics may be ignored during mutations.
    // Required positive witnesses still come from each contract's own cover list.
    let special_covers: BTreeSet<_> = all
        .iter()
        .filter(|c| c.name.starts_with("special256-") && !c.name.ends_with("-api"))
        .flat_map(|c| c.covers.iter().copied())
        .collect();
    let special_diagnostics: BTreeSet<_> = analyzer::AUXILIARY_CONDITIONAL_COVERS
        .iter()
        .copied()
        .filter(|c| c.starts_with("special "))
        .collect();
    assert_eq!(special_diagnostics, special_covers);
    let division_covers: BTreeSet<_> = all
        .iter()
        .filter(|c| c.name.starts_with("division256-") && !c.name.ends_with("-api"))
        .flat_map(|c| c.covers.iter().copied())
        .collect();
    let division_diagnostics: BTreeSet<_> = analyzer::AUXILIARY_CONDITIONAL_COVERS
        .iter()
        .copied()
        .filter(|c| c.starts_with("division class "))
        .collect();
    assert_eq!(division_diagnostics, division_covers);
    let multiplication_covers: BTreeSet<_> = all
        .iter()
        .filter(|c| c.name.starts_with("multiply256-") && !c.name.ends_with("-api"))
        .flat_map(|c| c.covers.iter().copied())
        .collect();
    let multiplication_diagnostics: BTreeSet<_> = analyzer::AUXILIARY_CONDITIONAL_COVERS
        .iter()
        .copied()
        .filter(|c| c.starts_with("multiplication class "))
        .collect();
    assert_eq!(multiplication_diagnostics, multiplication_covers);
    let opposite_covers: BTreeSet<_> = all
        .iter()
        .filter(|c| c.name.starts_with("opposite256-") && !c.name.ends_with("-api"))
        .flat_map(|c| c.covers.iter().copied())
        .collect();
    let opposite_diagnostics: BTreeSet<_> = analyzer::AUXILIARY_CONDITIONAL_COVERS
        .iter()
        .copied()
        .filter(|c| c.starts_with("opposite class "))
        .collect();
    assert_eq!(opposite_diagnostics, opposite_covers);
    let signed_small_covers: BTreeSet<_> = all
        .iter()
        .filter(|c| c.name.starts_with("signedsmall256-") && !c.name.ends_with("-api"))
        .flat_map(|c| c.covers.iter().copied())
        .collect();
    let signed_small_diagnostics: BTreeSet<_> = analyzer::AUXILIARY_CONDITIONAL_COVERS
        .iter()
        .copied()
        .filter(|c| c.starts_with("small signed class "))
        .collect();
    assert_eq!(signed_small_covers, signed_small_diagnostics);
    let signed_power_covers: BTreeSet<_> = all
        .iter()
        .filter(|c| c.name.starts_with("signedpow256-") && !c.name.ends_with("-api"))
        .flat_map(|c| c.covers.iter().copied())
        .collect();
    let signed_power_diagnostics: BTreeSet<_> = analyzer::AUXILIARY_CONDITIONAL_COVERS
        .iter()
        .copied()
        .filter(|c| c.starts_with("power signed class "))
        .collect();
    assert_eq!(signed_power_covers, signed_power_diagnostics);
    let ones_covers: BTreeSet<_> = all
        .iter()
        .filter(|c| c.name.starts_with("ones256-") && !c.name.ends_with("-api"))
        .flat_map(|c| c.covers.iter().copied())
        .collect();
    let ones_diagnostics: BTreeSet<_> = analyzer::AUXILIARY_CONDITIONAL_COVERS
        .iter()
        .copied()
        .filter(|c| c.starts_with("ones class "))
        .collect();
    assert_eq!(ones_covers, ones_diagnostics);
    let ones_power_covers: BTreeSet<_> = all
        .iter()
        .filter(|c| c.name.starts_with("powerones256-") && !c.name.ends_with("-api"))
        .flat_map(|c| c.covers.iter().copied())
        .collect();
    let ones_power_diagnostics: BTreeSet<_> = analyzer::AUXILIARY_CONDITIONAL_COVERS
        .iter()
        .copied()
        .filter(|c| c.starts_with("ones power class "))
        .collect();
    assert_eq!(ones_power_covers, ones_power_diagnostics);
    let hot_power_covers: BTreeSet<_> = all
        .iter()
        .filter(|c| c.name.starts_with("powerhot256-") && !c.name.ends_with("-api"))
        .flat_map(|c| c.covers.iter().copied())
        .collect();
    let hot_power_diagnostics: BTreeSet<_> = analyzer::AUXILIARY_CONDITIONAL_COVERS
        .iter()
        .copied()
        .filter(|c| c.starts_with("single hot power class "))
        .collect();
    assert_eq!(hot_power_covers, hot_power_diagnostics);
    let unitq_covers: BTreeSet<_> = all
        .iter()
        .filter(|c| c.name.starts_with("unitq256-") && !c.name.ends_with("-api"))
        .flat_map(|c| c.covers.iter().copied())
        .collect();
    let unitq_diagnostics: BTreeSet<_> = analyzer::AUXILIARY_CONDITIONAL_COVERS
        .iter()
        .copied()
        .filter(|c| c.starts_with("unit quotient class "))
        .collect();
    assert_eq!(unitq_covers, unitq_diagnostics);
    let powerq_covers: BTreeSet<_> = all
        .iter()
        .filter(|c| c.name.starts_with("powerq256-") && !c.name.ends_with("-api"))
        .flat_map(|c| c.covers.iter().copied())
        .collect();
    let powerq_diagnostics: BTreeSet<_> = analyzer::AUXILIARY_CONDITIONAL_COVERS
        .iter()
        .copied()
        .filter(|c| c.starts_with("power quotient class "))
        .collect();
    assert_eq!(powerq_covers, powerq_diagnostics);
    let signedq_covers: BTreeSet<_> = all
        .iter()
        .filter(|c| c.name.starts_with("signedq256-") && !c.name.ends_with("-api"))
        .flat_map(|c| c.covers.iter().copied())
        .collect();
    let signedq_diagnostics: BTreeSet<_> = analyzer::AUXILIARY_CONDITIONAL_COVERS
        .iter()
        .copied()
        .filter(|c| c.starts_with("signed quotient class "))
        .collect();
    assert_eq!(signedq_covers, signedq_diagnostics);
    let signed_context_budget: BTreeSet<_> = all
        .iter()
        .filter(|c| analyzer::uses_signed_quotient_context_timeout(c))
        .map(|c| c.name)
        .collect();
    let expected_signed_context_budget: BTreeSet<_> = [
        "signedq256-div-positive-positive-context",
        "signedq256-div-positive-negative-context",
        "signedq256-div-negative-positive-context",
        "signedq256-div-negative-negative-context",
        "signedq256-rem-positive-positive-context",
        "signedq256-rem-positive-negative-context",
        "signedq256-rem-negative-positive-context",
        "signedq256-rem-negative-negative-context",
    ]
    .into_iter()
    .collect();
    assert_eq!(signed_context_budget, expected_signed_context_budget);

    // Conditional diagnostics belong only to their matching public boundary proofs.
    // They must never become mandatory witnesses for unrelated contracts.
    for contract in &all {
        for witness in contract.covers {
            if witness.starts_with("public less equal width4 ") {
                assert_eq!(contract.group, "public-comparison-sequences");
                assert!(matches!(
                    contract.name,
                    "public-less-equal-left-width4" | "public-less-equal-right-width4"
                ));
            }
        }
    }
    for contract in &all {
        assert_eq!(
            analyzer::uses_extended_logical_timeout(contract),
            contract.name.starts_with("signedq256-")
                || contract.name.starts_with("powerq256-")
                || contract.name.starts_with("unitq256-")
                || contract.name.starts_with("powerhot256-")
                || contract.name.starts_with("powerones256-")
                || contract.name.starts_with("ones256-")
                || contract.name.starts_with("signedpow256-")
                || contract.name.starts_with("signedsmall256-")
                || contract.name.starts_with("opposite256-")
                || contract.name.starts_with("multiply256-")
                || contract.name.starts_with("division256-")
                || contract.name.starts_with("special256-")
                || matches!(
                    contract.name,
                    "public-less-equal-left-width4"
                        | "public-less-equal-right-width4"
                        | "unit256-div-one-reference"
                        | "identity256-mul-right"
                        | "identity256-mul-right-context"
                        | "identity256-mul-left"
                        | "identity256-mul-left-context"
                        | "identity256-div-one"
                        | "identity256-div-one-context"
                        | "identity256-rem-one"
                        | "identity256-rem-one-context"
                        | "power256-unknown"
                        | "power256-unknown-context"
                        | "power256-negative"
                        | "power256-negative-context"
                        | "power256-zero"
                        | "power256-zero-context"
                        | "exception256-mul-context"
                        | "exception256-div-context"
                        | "exception256-rem-context"
                        | "logical256-and-native-big"
                        | "logical256-and-big-native"
                        | "logical256-and-big-big"
                        | "logical256-or-native-big"
                        | "logical256-or-big-native"
                        | "logical256-or-big-big"
                        | "conversion256-bigint"
                        | "numeric256-order-api"
                        | "ordering256-less-unsigned"
                        | "ordering256-less-signed"
                        | "ordering256-less-unknown"
                        | "ordering256-less-eq-unsigned"
                        | "ordering256-less-eq-signed"
                        | "ordering256-less-eq-unknown"
                        | "ordering256-greater-unsigned"
                        | "ordering256-greater-signed"
                        | "ordering256-greater-unknown"
                        | "ordering256-greater-eq-unsigned"
                        | "ordering256-greater-eq-signed"
                        | "ordering256-greater-eq-unknown"
                        | "equality256-eq"
                        | "equality256-ne"
                        | "equality256-wild-eq"
                        | "equality256-wild-ne"
                        | "arithmetic256-add"
                        | "arithmetic256-sub"
                        | "arithmetic256-neg"
                        | "assignment256-big-native"
                        | "assignment256-big-big"
                        | "bitwise256-and"
                        | "bitwise256-or"
                        | "bitwise256-xor"
                        | "bitwise256-xnor"
                        | "shift256-left"
                        | "shift256-right"
                        | "shift256-arith-left"
                        | "shift256-arith-right-signed"
                        | "shift256-arith-right-unsigned"
                        | "public-logical-and-width65"
                        | "public-logical-or-native-continuation"
                        | "public-logical-or-width65"
                        | "public-conditional-wide-native-native"
                        | "public-conditional-wide-native-big"
                        | "public-conditional-wide-big-native"
                        | "public-conditional-wide-big-big"
                )
        );
    }
    // Kani's default harness filter is a substring match.
    for contract in &all {
        assert_eq!(
            all.iter()
                .filter(|other| other.harness.contains(contract.harness))
                .count(),
            1,
            "ambiguous Kani harness filter: {}",
            contract.harness,
        );
    }
    let mutations = analyzer::mutations(repo, &all);
    let mutation_count = mutations.len();
    let expected: BTreeMap<_, _> = mutations
        .into_iter()
        .map(|m| ((m.contract.name, m.name, m.path), m.source))
        .collect();
    assert_eq!(
        expected.len(),
        mutation_count,
        "duplicate mutation identity"
    );
    let mut contracts = BTreeSet::new();
    let mut mutations = BTreeSet::new();
    let mut proof_shards = 0;
    for shard in SHARDS
        .iter()
        .filter(|s| s.check == "value-contracts" && s.suite != "native")
    {
        proof_shards += 1;
        let p = plan(shard.check, shard.suite, Mode::All).unwrap();
        assert!(p.kani);
        assert_eq!(p.steps.len(), 1);
        let step = &p.steps[0];
        let manifest = step
            .args
            .iter()
            .position(|a| a == "--manifest-path")
            .unwrap();
        assert_eq!(step.args[manifest + 1], "targets/analyzer/kani/Cargo.toml");
        let args = step.args.iter().position(|a| a == "--").unwrap() + 1;
        let options = Options::parse(step.args[args..].iter().cloned()).unwrap();
        assert!(!options.contracts.is_empty());
        assert!(options.groups.is_empty());
        let selected = options.select("analyzer", &analyzer::CONTRACTS).unwrap();
        for contract in &selected {
            assert!(contracts.insert(contract.name), "overlapping proof shard");
            assert_eq!(
                analyzer::uses_explicit_covers(contract),
                shard.suite == "nonlinear"
                    || matches!(
                        contract.group,
                        "expression-context"
                            | "public-wide-resize"
                            | "public-big-values"
                            | "public-getters"
                            | "public-native-sequences"
                            | "public-comparison-sequences"
                            | "public-gate-sequences"
                    )
            );
            if matches!(
                contract.group,
                "public-wide-resize"
                    | "public-getters"
                    | "public-native-sequences"
                    | "public-big-values"
            ) {
                let expected_suite = if contract.group == "public-big-values" {
                    if contract.name.starts_with("signedq256-") {
                        if contract.name.starts_with("signedq256-rem") {
                            "big-signed-remainder-power-quotient"
                        } else {
                            "big-signed-division-power-quotient"
                        }
                    } else if contract.name.starts_with("powerq256-") {
                        "big-power-quotient"
                    } else if contract.name.starts_with("unitq256-") {
                        "big-unit-quotient"
                    } else if contract.name.starts_with("powerhot256-") {
                        "big-single-hot-power"
                    } else if contract.name.starts_with("powerones256-") {
                        "big-all-ones-power"
                    } else if contract.name.starts_with("ones256-") {
                        "big-all-ones-multiplication"
                    } else if contract.name.starts_with("signedpow256-") {
                        "big-signed-power-two-division"
                    } else if contract.name.starts_with("signedsmall256-") {
                        "big-signed-smaller-division"
                    } else if contract.name.starts_with("opposite256-") {
                        "big-opposite-division"
                    } else if contract.name.starts_with("multiply256-") {
                        "big-power-two-multiplication"
                    } else if contract.name.starts_with("division256-") {
                        if contract.name.starts_with("division256-self-")
                            || contract.name == "division256-byte-api"
                        {
                            "big-self-division"
                        } else {
                            "big-unsigned-division-classes"
                        }
                    } else if contract.name.starts_with("special256-") {
                        if contract.name.starts_with("special256-power-") {
                            "big-positive-power"
                        } else {
                            "big-division-identities"
                        }
                    } else if contract.name.starts_with("identity256-")
                        || contract.name.starts_with("unit256-")
                    {
                        "big-unit-identities"
                    } else if contract.name.starts_with("power256-") {
                        "big-power-special"
                    } else if contract.name.starts_with("exception256-") {
                        "big-nonlinear-exceptions"
                    } else if contract.name.starts_with("logical256-") {
                        "big-logical"
                    } else if contract.name.starts_with("ordering256-")
                        || contract.name == "numeric256-order-api"
                        || contract.name == "conversion256-bigint"
                        || [
                            "wiring256-less",
                            "wiring256-less-eq",
                            "wiring256-greater",
                            "wiring256-greater-eq",
                        ]
                        .contains(&contract.name)
                    {
                        "big-ordering"
                    } else if contract.name.starts_with("wiring256-")
                        || contract.name.starts_with("equality256-")
                        || contract.name.starts_with("arithmetic256-")
                        || contract.name.starts_with("numeric256-add")
                    {
                        "big-context-arithmetic"
                    } else if contract.name.contains("256") {
                        "big-operators"
                    } else {
                        "big-values"
                    }
                } else if contract.name.contains("shift") || contract.name.contains("big-amount") {
                    "shifts"
                } else {
                    "storage"
                };
                assert_eq!(shard.suite, expected_suite);
                let witness = match contract.name {
                    "signedq256-byte-api" => "signed quotient API positive positive is reachable",
                    "signedq256-quotient-word-api" => {
                        "signed quotient word unit width is reachable"
                    }
                    "signedq256-remainder-word-api" => {
                        "signed quotient word unit width is reachable"
                    }
                    "signedq256-domain-api" => "signed quotient domain native native is reachable",
                    "signedq256-div-positive" => {
                        "signed quotient class minimum nonpower divisor is reachable"
                    }
                    "signedq256-div-negative" => {
                        "signed quotient class minimum nonpower divisor is reachable"
                    }
                    "signedq256-rem-positive" => {
                        "signed quotient class minimum nonpower divisor is reachable"
                    }
                    "signedq256-rem-negative" => {
                        "signed quotient class minimum nonpower divisor is reachable"
                    }
                    "signedq256-div-positive-positive-context" => {
                        "signed quotient class positive positive context is reachable"
                    }
                    "signedq256-div-positive-negative-context" => {
                        "signed quotient class positive negative context is reachable"
                    }
                    "signedq256-div-negative-positive-context" => {
                        "signed quotient class negative positive context is reachable"
                    }
                    "signedq256-div-negative-negative-context" => {
                        "signed quotient class negative negative context is reachable"
                    }
                    "signedq256-rem-positive-positive-context" => {
                        "signed quotient class positive positive context is reachable"
                    }
                    "signedq256-rem-positive-negative-context" => {
                        "signed quotient class positive negative context is reachable"
                    }
                    "signedq256-rem-negative-positive-context" => {
                        "signed quotient class negative positive context is reachable"
                    }
                    "signedq256-rem-negative-negative-context" => {
                        "signed quotient class negative negative context is reachable"
                    }
                    "powerq256-byte-api" => "power quotient API unit quotient is reachable",
                    "powerq256-product-api" => "power quotient product unit quotient is reachable",
                    "powerq256-euclidean-api" => "power quotient Euclidean unit width is reachable",
                    "powerq256-domain-api" => "power quotient domain native native is reachable",
                    "powerq256-div" => "power quotient class minimum nonunit quotient is reachable",
                    "powerq256-div-context" => "power quotient class original context is reachable",
                    "powerq256-rem" => "power quotient class minimum nonunit quotient is reachable",
                    "powerq256-rem-context" => "power quotient class original context is reachable",
                    "unitq256-byte-api" => "unit quotient API unit divisor is reachable",
                    "unitq256-euclidean-api" => "unit quotient Euclidean unit width is reachable",
                    "unitq256-domain-api" => "unit quotient domain native native is reachable",
                    "unitq256-div" => "unit quotient class minimum width is reachable",
                    "unitq256-div-context" => "unit quotient class original context is reachable",
                    "unitq256-rem" => "unit quotient class minimum width is reachable",
                    "unitq256-rem-context" => "unit quotient class original context is reachable",
                    "powerhot256-expansion-api" => "single hot membership unit width is reachable",
                    "powerhot256-byte-api" => "single hot power API zero exponent is reachable",
                    "powerhot256-coordinates-api" => {
                        "single hot power coordinates unit width is reachable"
                    }
                    "powerhot256-successor-api" => {
                        "single hot power recurrence unit base is reachable"
                    }
                    "powerhot256-domain-api" => {
                        "single hot power domain native native is reachable"
                    }
                    "powerhot256-prepared-signed" => {
                        "single hot power class minimum signed width is reachable"
                    }
                    "powerhot256-prepared-unsigned" => {
                        "single hot power class maximum unsigned width is reachable"
                    }
                    "powerhot256-context-signed" => {
                        "single hot power class signed original context is reachable"
                    }
                    "powerhot256-context-unsigned" => {
                        "single hot power class unsigned original context is reachable"
                    }
                    "powerones256-byte-api" => "ones power API zero exponent is reachable",
                    "powerones256-successor-api" => "ones power recurrence unit width is reachable",
                    "powerones256-domain-api" => "ones power domain native native is reachable",
                    "powerones256-prepared" => "ones power class minimum signed base is reachable",
                    "powerones256-context" => "ones power class original context is reachable",
                    "powerones256-prepared-witness" => {
                        "ones power class witness signed base is reachable"
                    }
                    "powerones256-context-witness" => {
                        "ones power class witness native exponent is reachable"
                    }
                    "ones256-byte-api" => "ones API unit factor is reachable",
                    "ones256-inverse-api" => "ones inverse unit width is reachable",
                    "ones256-word-api" => "ones word minimum width is reachable",
                    "ones256-right-domain-api" => "ones domain native native is reachable",
                    "ones256-left-domain-api" => "ones domain native native is reachable",
                    "ones256-right" => "ones class minimum signed width is reachable",
                    "ones256-right-context" => "ones class original context is reachable",
                    "ones256-left" => "ones class minimum signed width is reachable",
                    "ones256-left-context" => "ones class original context is reachable",
                    "signedpow256-byte-api" => "power signed API zero dividend is reachable",
                    "signedpow256-coordinates-api" => {
                        "power signed coordinates unit divisor is reachable"
                    }
                    "signedpow256-word-api" => "power signed word minimum width is reachable",
                    "signedpow256-domain-api" => "power signed domain native native is reachable",
                    "signedpow256-div" => "power signed class minimum nonunit divisor is reachable",
                    "signedpow256-div-nonnegative-context" => {
                        "power signed class nonnegative context is reachable"
                    }
                    "signedpow256-div-negative-context" => {
                        "power signed class negative context is reachable"
                    }
                    "signedpow256-rem" => "power signed class minimum nonunit divisor is reachable",
                    "signedpow256-rem-nonnegative-context" => {
                        "power signed class nonnegative context is reachable"
                    }
                    "signedpow256-rem-negative-context" => {
                        "power signed class negative context is reachable"
                    }
                    "signedsmall256-conversion-api" => {
                        "small signed conversion minimum width is reachable"
                    }
                    "signedsmall256-byte-api" => "small signed API zero dividend is reachable",
                    "signedsmall256-word-api" => "small signed word minimum width is reachable",
                    "signedsmall256-domain-api" => "small signed domain native native is reachable",
                    "signedsmall256-div" => "small signed class minimum positive pair is reachable",
                    "signedsmall256-div-nonnegative-context" => {
                        "small signed class nonnegative context is reachable"
                    }
                    "signedsmall256-div-negative-context" => {
                        "small signed class negative context is reachable"
                    }
                    "signedsmall256-rem" => "small signed class minimum positive pair is reachable",
                    "signedsmall256-rem-nonnegative-context" => {
                        "small signed class nonnegative context is reachable"
                    }
                    "signedsmall256-rem-negative-context" => {
                        "small signed class negative context is reachable"
                    }
                    "opposite256-conversion-api" => {
                        "opposite conversion minimum width is reachable"
                    }
                    "opposite256-byte-api" => "opposite API positive dividend is reachable",
                    "opposite256-context-domain-api" => {
                        "opposite class minimum positive dividend is reachable"
                    }
                    "opposite256-div" => "opposite class minimum positive dividend is reachable",
                    "opposite256-div-positive-context" => {
                        "opposite class positive context is reachable"
                    }
                    "opposite256-div-negative-context" => {
                        "opposite class negative context is reachable"
                    }
                    "opposite256-rem" => "opposite class minimum positive dividend is reachable",
                    "opposite256-rem-positive-context" => {
                        "opposite class positive context is reachable"
                    }
                    "opposite256-rem-negative-context" => {
                        "opposite class negative context is reachable"
                    }
                    "multiply256-byte-api" => "multiplication API zero number is reachable",
                    "multiply256-coordinates-api" => "multiplication API factor one is reachable",
                    "multiply256-right" => "multiplication class minimum signed width is reachable",
                    "multiply256-right-context" => {
                        "multiplication class minimum signed width is reachable"
                    }
                    "multiply256-left" => "multiplication class minimum signed width is reachable",
                    "multiply256-left-context" => {
                        "multiplication class minimum signed width is reachable"
                    }
                    "division256-self-signed-domain-api" => {
                        "division class native native sources is reachable"
                    }
                    "division256-byte-api" => "division API equal words are reachable",
                    "division256-pow2-api" => "division API zero shift is reachable",
                    "division256-self-div-unsigned" => "division class minimum width is reachable",
                    "division256-self-div-unsigned-context" => {
                        "division class minimum width is reachable"
                    }
                    "division256-self-div-signed" => "division class minimum width is reachable",
                    "division256-self-div-signed-context" => {
                        "division class minimum width is reachable"
                    }
                    "division256-self-rem-unsigned" => "division class minimum width is reachable",
                    "division256-self-rem-unsigned-context" => {
                        "division class minimum width is reachable"
                    }
                    "division256-self-rem-signed" => "division class minimum width is reachable",
                    "division256-self-rem-signed-context" => {
                        "division class minimum width is reachable"
                    }
                    "division256-smaller-div" => "division class minimum width is reachable",
                    "division256-smaller-div-context" => {
                        "division class minimum width is reachable"
                    }
                    "division256-smaller-rem" => "division class minimum width is reachable",
                    "division256-smaller-rem-context" => {
                        "division class minimum width is reachable"
                    }
                    "division256-pow2-div" => "division class minimum width is reachable",
                    "division256-pow2-div-context" => "division class minimum width is reachable",
                    "division256-pow2-rem" => "division class minimum width is reachable",
                    "division256-pow2-rem-context" => "division class minimum width is reachable",
                    "special256-signed-api" => "special API zero over upper divisor is reachable",
                    "special256-power-api" => "special API bit128 modulus is reachable",
                    "special256-signed-div-unit" => "special minimum width is reachable",
                    "special256-signed-div-unit-context" => "special minimum width is reachable",
                    "special256-signed-rem-unit" => "special minimum width is reachable",
                    "special256-signed-rem-unit-context" => "special minimum width is reachable",
                    "special256-zero-div" => "special minimum width is reachable",
                    "special256-zero-div-context" => "special minimum width is reachable",
                    "special256-zero-rem" => "special minimum width is reachable",
                    "special256-zero-rem-context" => "special minimum width is reachable",
                    "special256-power-one" => "special minimum width is reachable",
                    "special256-power-one-context" => "special minimum width is reachable",
                    "special256-power-unit-base" => "special minimum width is reachable",
                    "special256-power-unit-base-context" => "special minimum width is reachable",

                    "unit256-div-one-reference" => "unit division reference upper bit is reachable",
                    "unit256-api" => "unit API bit255 is reachable",
                    "identity256-mul-right"
                    | "identity256-mul-right-context"
                    | "identity256-mul-left"
                    | "identity256-mul-left-context"
                    | "identity256-div-one"
                    | "identity256-div-one-context"
                    | "identity256-rem-one"
                    | "identity256-rem-one-context" => "unit identity upper bit is reachable",

                    "power256-zero-api" => "power bridge negative zero is reachable",
                    "power256-unknown"
                    | "power256-unknown-context"
                    | "power256-negative"
                    | "power256-negative-context"
                    | "power256-zero"
                    | "power256-zero-context" => {
                        "wide power exponent wider than result is reachable"
                    }

                    "exception256-mul"
                    | "exception256-mul-context"
                    | "exception256-div"
                    | "exception256-div-context"
                    | "exception256-rem"
                    | "exception256-rem-context" => "wide exception only lhs unknown is reachable",

                    "arithmetic256-plus" => "wide plus upper X is reachable",
                    "wiring256-plus" => "wide plus X sign extension is reachable",
                    "logical256-truth-reference" => {
                        "logical truth full zero extension is reachable"
                    }
                    "logical256-and-native-big"
                    | "logical256-and-big-native"
                    | "logical256-and-big-big"
                    | "logical256-or-native-big"
                    | "logical256-or-big-native"
                    | "logical256-or-big-big" => "wide logical unknown is reachable",

                    "conversion256-bigint" => "wide signed conversion minimum is reachable",
                    "numeric256-order-api" => "ordering API high half difference is reachable",
                    "ordering256-less-unsigned"
                    | "ordering256-less-signed"
                    | "ordering256-less-unknown"
                    | "ordering256-less-eq-unsigned"
                    | "ordering256-less-eq-signed"
                    | "ordering256-less-eq-unknown"
                    | "ordering256-greater-unsigned"
                    | "ordering256-greater-signed"
                    | "ordering256-greater-unknown"
                    | "ordering256-greater-eq-unsigned"
                    | "ordering256-greater-eq-signed"
                    | "ordering256-greater-eq-unknown" => {
                        if contract.name.ends_with("-unknown") {
                            "wide ordering upper unknown is reachable"
                        } else {
                            "wide ordering known true is reachable"
                        }
                    }
                    "wiring256-less"
                    | "wiring256-less-eq"
                    | "wiring256-greater"
                    | "wiring256-greater-eq" => "wiring ordering signed operands are reachable",
                    "numeric256-add-api" => "addition API carry into high half is reachable",
                    "numeric256-add" => "numeric addition carry into bit128 is reachable",
                    "equality256-eq" => "wide equality upper unknown is reachable",
                    "equality256-ne" => "wide equality upper unknown is reachable",
                    "equality256-wild-eq" => "wide equality upper unknown is reachable",
                    "equality256-wild-ne" => "wide equality upper unknown is reachable",
                    "arithmetic256-add" => "wide arithmetic carry boundary is reachable",
                    "arithmetic256-sub" => "wide arithmetic carry boundary is reachable",
                    "arithmetic256-neg" => "wide arithmetic carry boundary is reachable",
                    "wiring256-and" => "wiring native signed source reaches wide context",
                    "wiring256-or" => "wiring native signed source reaches wide context",
                    "wiring256-xor" => "wiring native signed source reaches wide context",
                    "wiring256-xnor" => "wiring native signed source reaches wide context",
                    "wiring256-not" => "wiring native signed source reaches wide context",
                    "wiring256-left" => "wiring native signed source reaches wide context",
                    "wiring256-right" => "wiring native signed source reaches wide context",
                    "wiring256-arith-left" => "wiring native signed source reaches wide context",
                    "wiring256-arith-right" => "wiring native signed source reaches wide context",
                    "wiring256-eq" => "wiring native signed source reaches wide context",
                    "wiring256-ne" => "wiring native signed source reaches wide context",
                    "wiring256-wild-eq" => "wiring native signed source reaches wide context",
                    "wiring256-wild-ne" => "wiring native signed source reaches wide context",
                    "wiring256-add" => "wiring native signed source reaches wide context",
                    "wiring256-sub" => "wiring native signed source reaches wide context",
                    "wiring256-neg" => "wiring native signed source reaches wide context",

                    "reduction256-and" => "wide reduction maximum widths are reachable",
                    "reduction256-nand" => "wide reduction maximum widths are reachable",
                    "reduction256-or" => "wide reduction maximum widths are reachable",
                    "reduction256-nor" => "wide reduction maximum widths are reachable",
                    "reduction256-xor" => "wide reduction maximum widths are reachable",
                    "reduction256-xnor" => "wide reduction maximum widths are reachable",
                    "reduction256-logic-not" => "wide reduction maximum widths are reachable",

                    "assignment256-high" => "assignment bit254 intermediate is reachable",
                    "bitwise256-and" => "wide bitwise lower boundary is reachable",
                    "bitwise256-or" => "wide bitwise lower boundary is reachable",
                    "bitwise256-xor" => "wide bitwise lower boundary is reachable",
                    "bitwise256-xnor" => "wide bitwise lower boundary is reachable",
                    "not256" => "wide NOT sign expansion is reachable",
                    "shift256-left" => "wide shift oversized amount is reachable",
                    "shift256-right" => "wide shift oversized amount is reachable",
                    "shift256-arith-left" => "wide shift oversized amount is reachable",
                    "shift256-arith-right-signed" => "wide shift oversized amount is reachable",
                    "shift256-arith-right-unsigned" => "wide shift oversized amount is reachable",

                    "numeric256-shifts" => "numeric256 left crosses128 is reachable",
                    "assignment256-native-native" => "wide assignment full window is reachable",
                    "assignment256-native-big" => "wide assignment full window is reachable",
                    "assignment256-big-native" => "wide assignment full window is reachable",
                    "assignment256-big-big" => "wide assignment full window is reachable",

                    "numeric256-api" => "numeric256 high difference is reachable",
                    "conditional-expansion-value-summary" => {
                        "wide expansion value summary borrowed sign is reachable"
                    }
                    "conditional-expansion-summary" => {
                        "expansion summary native Z fill is reachable"
                    }
                    "big-numeric-wide-api" => "wide numeric API zero zero is reachable",
                    "public-big-truncation" => "big truncation native low X is reachable",
                    "public-big-expansion-zero" => {
                        "big expansion borrowed signed source is reachable"
                    }
                    "public-big-expansion" => "big expansion borrowed signed source is reachable",
                    "public-big-selection-wide" => "big wide selection full width65 is reachable",
                    "public-conditional-wide-native-native" => {
                        "wide conditional known zero is reachable"
                    }
                    "public-conditional-wide-native-big" => {
                        "wide conditional known zero is reachable"
                    }
                    "public-conditional-wide-big-native" => {
                        "wide conditional known zero is reachable"
                    }
                    "public-conditional-wide-big-big" => "wide conditional known zero is reachable",
                    "big-numeric-api" => "big numeric API word crossing is reachable",
                    "big-numeric-bridge" => "big numeric bridge inline word is reachable",
                    "big-truth-reference" => "big truth reference X is reachable",
                    "public-big-truth" => "big truth known one dominates X is reachable",
                    "public-big-conditional-native" => {
                        "big conditional mixed signs and known one are reachable"
                    }
                    "public-native-sign-extension-wide" => {
                        "big sign extension interior is reachable"
                    }
                    "public-big-selection-native" => {
                        "big selection crosses word boundary is reachable"
                    }
                    "public-native-zero-extension-wide" => {
                        "public native wide zero extension interior widths are reachable"
                    }
                    "public-native-zero-fill-extension-wide" => {
                        "public native zero fill extension signed interior Z is reachable"
                    }
                    "public-big-getters-128" => {
                        "public BigUint getters high Z and padding are reachable"
                    }
                    "public-big-amount-decoding-128" => {
                        "public BigUint upper Z amount is reachable"
                    }
                    name if name.starts_with("public-word64-big-amount-")
                        || name.starts_with("public-word32-big-amount-")
                        || name.starts_with("public-shift-big-amount-width-") =>
                    {
                        "public wide amount trace unsigned Z source is reachable"
                    }
                    name if name.starts_with("public-shift-widening-4-to-") => {
                        "public shift widening signed Z fill is reachable"
                    }
                    name if name.starts_with("public-small-shift-continuation-width-")
                        || name.starts_with("public-shift-boundary-width-") =>
                    {
                        "public shift continuation signed Z fill is reachable"
                    }
                    _ => panic!("explicit public contracts need a scope witness policy"),
                };
                assert!(contract.covers.contains(&witness));
            }
            if contract.group == "public-gate-sequences" {
                let suite = if contract.name.starts_with("public-reduction-")
                    || contract.name.starts_with("reduction-")
                    || contract.name.starts_with("public-logical-not-")
                {
                    "reductions"
                } else {
                    "logic"
                };
                assert_eq!(shard.suite, suite);
                assert!(!contract.covers.is_empty());
                let witnesses: &[&str] = match contract.name {
                    "public-conditional-native" => &[
                        "public native conditional condition width 64 is reachable",
                        "public native conditional high bit determines true",
                        "public native conditional high bit determines unknown",
                    ],
                    "public-bitwise-widened-native-trace" => &[
                        "public bitwise trace LHS width 64 is reachable",
                        "public bitwise trace RHS width 64 is reachable",
                        "public bitwise trace widened context is reachable",
                        "public bitwise trace signed Z extension is reachable",
                        "public bitwise trace u128 padding is reachable",
                    ],
                    "public-bitwise-widened-mutation-witness" => &[
                        "public bitwise witness signed LHS only is reachable",
                        "public bitwise witness signed RHS only is reachable",
                        "public bitwise witness u128 padding is reachable",
                    ],
                    "public-reduction-nand-native-continuation" => &[
                        "public native reduction NAND width 1 zero unchanged is reachable",
                        "public native reduction NAND width 64 signed one is reachable",
                        "public native reduction NAND widened X and Z is reachable",
                        "public native reduction NAND widened zero dominates X and Z is reachable",
                    ],
                    "public-reduction-nand-width65" | "public-reduction-nand-width128" => &[
                        "public wide reduction NAND width 1 zero unchanged is reachable",
                        "public wide reduction NAND width 64 signed one is reachable",
                        "public wide reduction NAND widened X and Z is reachable",
                        "public wide reduction NAND widened zero dominates X and Z is reachable",
                    ],
                    "public-logical-not-native-continuation"
                    | "public-logical-not-width65"
                    | "public-logical-not-width128" => &[
                        "public logical NOT width 1 zero unchanged is reachable",
                        "public logical NOT width 64 signed high one is reachable",
                        "public logical NOT widened X and Z is reachable",
                        "public logical NOT widened one dominates X and Z is reachable",
                    ],
                    "equality256-eq"
                    | "equality256-ne"
                    | "equality256-wild-eq"
                    | "equality256-wild-ne"
                    | "arithmetic256-add"
                    | "arithmetic256-sub"
                    | "arithmetic256-neg"
                    | "assignment256-big-native"
                    | "assignment256-big-big"
                    | "bitwise256-and"
                    | "bitwise256-or"
                    | "bitwise256-xor"
                    | "bitwise256-xnor"
                    | "shift256-left"
                    | "shift256-right"
                    | "shift256-arith-left"
                    | "shift256-arith-right-signed"
                    | "shift256-arith-right-unsigned"
                    | "public-logical-and-width65"
                    | "public-logical-or-native-continuation"
                    | "public-logical-or-width65" => {
                        &["public logical binary full continuation is reachable"]
                    }
                    "public-logical-and-width65-witness"
                    | "public-logical-or-native-witness"
                    | "public-logical-or-width65-witness" => &[
                        "public logical binary witness zero result is reachable",
                        "public logical binary witness one result is reachable",
                        "public logical binary witness X result is reachable",
                    ],
                    "public-logical-binary-domain" => &[
                        "public logical binary unsigned narrow LHS and high RHS are reachable",
                        "public logical binary signed high LHS and narrow RHS are reachable",
                        "public logical binary signed LHS known ones dominate X and Z",
                        "public logical binary signed RHS zero and unknown LHS are reachable",
                        "public logical binary zero LHS and Z RHS are reachable",
                        "public logical binary both unknown widened sources are reachable",
                        "public logical binary both zero unchanged sources are reachable",
                    ],
                    "logical-truth-unrolled-reference" => &[
                        "logical unrolled truth width 64 is reachable",
                        "logical unrolled truth X is reachable",
                    ],
                    "reduction-xor-reference" => &[
                        "reduction XOR reference width 64 is reachable",
                        "reduction XOR reference X is reachable",
                    ],
                    "public-reduction-xor-native-continuation" => &[
                        "public native reduction XOR width 1 zero unchanged is reachable",
                        "public native reduction XOR width 64 signed odd parity is reachable",
                        "public native reduction XOR width 64 even parity is reachable",
                        "public native reduction XOR widened X and Z is reachable",
                        "public native reduction XOR widened unknown dominates one is reachable",
                    ],
                    "public-reduction-xnor-native-continuation" => &[
                        "public native reduction XNOR width 1 zero unchanged is reachable",
                        "public native reduction XNOR width 64 signed odd parity is reachable",
                        "public native reduction XNOR width 64 even parity is reachable",
                        "public native reduction XNOR widened X and Z is reachable",
                        "public native reduction XNOR widened unknown dominates one is reachable",
                    ],
                    "public-reduction-xnor-width65" | "public-reduction-xnor-width128" => &[
                        "public wide reduction XNOR width 1 zero unchanged is reachable",
                        "public wide reduction XNOR width 64 signed odd parity is reachable",
                        "public wide reduction XNOR width 64 even parity is reachable",
                        "public wide reduction XNOR widened X and Z is reachable",
                        "public wide reduction XNOR widened unknown dominates one is reachable",
                    ],
                    "public-reduction-xor-width65" | "public-reduction-xor-width128" => &[
                        "public wide reduction XOR width 1 zero unchanged is reachable",
                        "public wide reduction XOR width 64 signed odd parity is reachable",
                        "public wide reduction XOR width 64 even parity is reachable",
                        "public wide reduction XOR widened X and Z is reachable",
                        "public wide reduction XOR widened unknown dominates one is reachable",
                    ],
                    "reduction-or-reference" => &[
                        "reduction OR reference width 64 is reachable",
                        "reduction OR reference X is reachable",
                    ],
                    "public-reduction-or-native-continuation" => &[
                        "public native reduction OR width 1 zero unchanged is reachable",
                        "public native reduction OR width 64 signed high one is reachable",
                        "public native reduction OR widened X and Z is reachable",
                        "public native reduction OR widened one dominates X and Z is reachable",
                    ],
                    "public-reduction-nor-native-continuation" => &[
                        "public native reduction NOR width 1 zero unchanged is reachable",
                        "public native reduction NOR width 64 signed high one is reachable",
                        "public native reduction NOR widened X and Z is reachable",
                        "public native reduction NOR widened one dominates X and Z is reachable",
                    ],
                    "public-reduction-nor-width65" | "public-reduction-nor-width128" => &[
                        "public wide reduction NOR width 1 zero unchanged is reachable",
                        "public wide reduction NOR width 64 signed high one is reachable",
                        "public wide reduction NOR widened X and Z is reachable",
                        "public wide reduction NOR widened one dominates X and Z is reachable",
                    ],
                    "public-reduction-or-width65" | "public-reduction-or-width128" => &[
                        "public wide reduction OR width 1 zero unchanged is reachable",
                        "public wide reduction OR width 64 signed high one is reachable",
                        "public wide reduction OR widened X and Z is reachable",
                        "public wide reduction OR widened one dominates X and Z is reachable",
                    ],
                    "reduction-and-reference" => &[
                        "reduction AND reference width 64 is reachable",
                        "reduction AND reference X is reachable",
                    ],
                    "public-reduction-and-width65" | "public-reduction-and-width128" => &[
                        "public wide reduction AND width 1 zero unchanged is reachable",
                        "public wide reduction AND width 64 signed one is reachable",
                        "public wide reduction AND widened X and Z condition is reachable",
                        "public wide reduction AND widened zero dominates X and Z is reachable",
                    ],
                    "public-reduction-and-native-continuation" => &[
                        "public reduction AND source width 64 is reachable",
                        "public reduction AND widened NOT context is reachable",
                        "public reduction AND zero dominates unknown is reachable",
                    ],
                    "public-logical-and-native-continuation" => &[
                        "public native logical AND LHS width 64 is reachable",
                        "public native logical AND RHS width 64 is reachable",
                        "public native logical AND widened context is reachable",
                        "public native logical AND known ones dominate unknown is reachable",
                        "public native logical AND signed LHS only is reachable",
                        "public native logical AND signed RHS only is reachable",
                    ],
                    "public-logical-and-mutation-witness" => &[
                        "logical AND witness signed LHS only is reachable",
                        "logical AND witness signed RHS only is reachable",
                        "logical AND witness zero dominates RHS unknown is reachable",
                    ],
                    "logical-truth-reference" => &[
                        "logical truth reference width 64 is reachable",
                        "logical truth reference X is reachable",
                    ],
                    _ => panic!("public gate contracts need a scope witness policy"),
                };
                for witness in witnesses {
                    assert!(contract.covers.contains(witness));
                }
            }
            if contract.group == "public-comparison-sequences" {
                let suite = if contract.name.contains("greater") {
                    "comparison-greater"
                } else if contract.name.contains("less") {
                    "comparison-less"
                } else {
                    "comparison-equality"
                };
                assert_eq!(shard.suite, suite);
                if matches!(
                    contract.name,
                    "public-less-equal-left-width4" | "public-less-equal-right-width4"
                ) || contract.name.starts_with("public-small-less-equal-width-")
                    || contract
                        .name
                        .starts_with("public-small-wildcard-equal-width-")
                    || contract
                        .name
                        .starts_with("public-small-wildcard-not-equal-width-")
                    || contract.name.starts_with("public-small-less-width-")
                    || contract.name.starts_with("public-small-greater-width-")
                    || contract
                        .name
                        .starts_with("public-small-greater-equal-width-")
                {
                    for witness in [
                        "public comparison unsigned sources are reachable",
                        "public comparison signed RHS only is reachable",
                        "public comparison signed LHS only is reachable",
                        "public comparison signed sources are reachable",
                        "public comparison X result is reachable",
                        "public comparison true result is reachable",
                        "public comparison false result is reachable",
                    ] {
                        assert!(contract.covers.contains(&witness));
                    }
                    if contract
                        .name
                        .starts_with("public-small-wildcard-equal-width-")
                    {
                        for witness in [
                            "public width4 wildcard equality RHS X matches",
                            "public width4 wildcard equality RHS Z matches",
                            "public width4 wildcard equality LHS X remains significant",
                            "public width4 wildcard equality LHS Z remains significant",
                            "public width4 wildcard equality known mismatch dominates LHS unknown",
                            "public width4 wildcard equality known mismatch survives RHS wildcard",
                        ] {
                            assert!(contract.covers.contains(&witness));
                        }
                    }
                    if contract
                        .name
                        .starts_with("public-small-wildcard-not-equal-width-")
                    {
                        for witness in [
                            "public width4 wildcard inequality RHS X matches",
                            "public width4 wildcard inequality RHS Z matches",
                            "public width4 wildcard inequality LHS X remains significant",
                            "public width4 wildcard inequality LHS Z remains significant",
                            "public width4 wildcard inequality known mismatch dominates LHS unknown",
                            "public width4 wildcard inequality known mismatch survives RHS wildcard",
                        ] {
                            assert!(contract.covers.contains(&witness));
                        }
                    }
                    let boundary_witnesses: &[&str] = match contract.name {
                        "public-less-equal-left-width4" => &[
                            "public less equal width4 narrow RHS is reachable",
                            "public less equal width4 equal widths are reachable",
                        ],
                        "public-less-equal-right-width4" => &[
                            "public less equal width4 narrow LHS is reachable",
                            "public less equal width4 three bit LHS is reachable",
                        ],
                        _ => &[],
                    };
                    for witness in boundary_witnesses {
                        assert!(contract.covers.contains(witness));
                    }
                } else {
                    let sizing_witness =
                        if contract.name == "public-small-wildcard-equality-continuation" {
                            "public comparison sign extension is reachable"
                        } else {
                            "public comparison partition sizing is reachable"
                        };
                    assert!(contract.covers.contains(&sizing_witness));
                    if contract.name.ends_with("-unsigned-continuation") {
                        for witness in [
                            "public comparison unsigned sources are reachable",
                            "public comparison signed RHS only is reachable",
                            "public comparison signed LHS only is reachable",
                        ] {
                            assert!(contract.covers.contains(&witness));
                        }
                    } else if contract.name.ends_with("-signed-continuation") {
                        assert!(
                            contract
                                .covers
                                .contains(&"public comparison signed sources are reachable")
                        );
                    }
                }
            }
            if contract.group == "public-nonlinear" {
                assert!(!contract.covers.is_empty());
                assert!(
                    contract
                        .covers
                        .contains(&"nonlinear remainder numeric case is reachable")
                );
            }
        }
        for mutant in analyzer::mutations(repo, &selected) {
            let key = (mutant.contract.name, mutant.name, mutant.path);
            assert!(mutations.insert(key), "overlapping mutation shard");
            assert_eq!(expected.get(&key), Some(&mutant.source));
        }
    }
    // Both partitions are required before the direct checks can omit either context.
    for family in [
        "public-inequality-native",
        "public-small-wildcard-inequality",
        "public-small-less",
    ] {
        for context in ["unsigned", "signed"] {
            let name = format!("{family}-{context}-continuation");
            assert!(
                contracts.contains(name.as_str()),
                "missing comparison partition: {name}"
            );
        }
    }
    // Every fixed width pair is required before its direct result checks are omitted.
    for (family, limit) in [("less-equal", 3), ("greater", 4), ("greater-equal", 4)] {
        for left in 1..=limit {
            for right in 1..=limit {
                let name = format!("public-small-{family}-width-{left}-by-{right}");
                assert!(
                    contracts.contains(name.as_str()),
                    "missing comparison width pair: {name}"
                );
            }
        }
    }
    // The original two Less contexts plus these boundaries cover widths1..4.
    for left in 1..=4 {
        for right in 1..=4 {
            if left == 4 || right == 4 {
                let name = format!("public-small-less-width-{left}-by-{right}");
                assert!(
                    contracts.contains(name.as_str()),
                    "missing Less boundary: {name}"
                );
            }
        }
    }
    // The wildcard equality continuation covers1..3; these complete widths1..4.
    assert!(contracts.contains("public-small-wildcard-equality-continuation"));
    for left in 1..=4 {
        for right in 1..=4 {
            if left == 4 || right == 4 {
                let name = format!("public-small-wildcard-equal-width-{left}-by-{right}");
                assert!(
                    contracts.contains(name.as_str()),
                    "missing wildcard boundary: {name}"
                );
            }
        }
    }
    // Both wildcard inequality contexts cover1..3; these complete widths1..4.
    for left in 1..=4 {
        for right in 1..=4 {
            if left == 4 || right == 4 {
                let name = format!("public-small-wildcard-not-equal-width-{left}-by-{right}");
                assert!(
                    contracts.contains(name.as_str()),
                    "missing wildcard inequality boundary: {name}"
                );
            }
        }
    }
    // Both native passes are required before direct result checks are omitted.
    assert!(contracts.contains("public-bitwise-native"));
    assert!(contracts.contains("public-bitwise-widened-native-trace"));
    assert!(contracts.contains("public-bitwise-widened-mutation-witness"));
    assert!(contracts.contains("public-logical-and-native-continuation"));
    assert!(contracts.contains("logical-truth-reference"));
    assert!(contracts.contains("public-logical-and-mutation-witness"));
    assert!(contracts.contains("public-reduction-and-native-continuation"));
    let getter = all
        .iter()
        .find(|c| c.name == "public-big-getters-128")
        .unwrap();
    for witness in [
        "public BigUint getters u64 constructor zero is reachable",
        "public BigUint getters u64 constructor one is reachable",
        "public BigUint getters u64 constructor X is reachable",
        "public BigUint getters u64 constructor Z is reachable",
    ] {
        assert!(getter.covers.contains(&witness));
    }
    // Wide reduction plane checks compose with scalar models and the getter theorem.
    for name in [
        "public-reduction-nor-width65",
        "public-reduction-nor-width128",
        "public-reduction-xor-width65",
        "public-reduction-xor-width128",
        "public-reduction-xnor-width65",
        "public-reduction-xnor-width128",
        "public-logical-not-native-continuation",
        "public-logical-not-width65",
        "public-logical-not-width128",
        "logical-truth-unrolled-reference",
        "public-logical-binary-domain",
        "public-logical-and-width65-witness",
        "public-logical-or-native-witness",
        "public-logical-or-width65-witness",
        "public-logical-and-width65",
        "public-logical-or-native-continuation",
        "public-logical-or-width65",
        "reduction-xor-reference",
        "reduction-or-reference",
        "public-big-getters-128",
    ] {
        assert!(
            contracts.contains(name),
            "missing reduction proof dependency: {name}"
        );
    }
    // These disjoint boundaries complete the existing1..3 matrix through width4.
    assert!(contracts.contains("public-less-equal-left-width4"));
    assert!(contracts.contains("public-less-equal-right-width4"));
    // Conditional summaries are valid only together with the full expansion
    // domain partition and both independent summary-conformance theorems.
    for name in [
        "public-resize-native",
        "public-native-zero-extension-wide",
        "public-native-zero-fill-extension-wide",
        "public-native-sign-extension-wide",
        "public-big-expansion-zero",
        "public-big-expansion",
        "conditional-expansion-summary",
        "conditional-expansion-value-summary",
        "big-numeric-wide-api",
        "public-conditional-wide-native-native",
        "public-conditional-wide-native-big",
        "public-conditional-wide-big-native",
        "public-conditional-wide-big-big",
    ] {
        assert!(
            contracts.contains(name),
            "missing conditional composition dependency: {name}"
        );
    }
    assert_eq!(proof_shards, 33);
    assert_eq!(contracts, all.iter().map(|c| c.name).collect());
    assert_eq!(mutations, expected.keys().copied().collect());
}

#[test]
fn value_native_retains_all_finite_checks_without_installing_kani() {
    use std::collections::BTreeSet;
    let p = plan("value-contracts", "native", Mode::All).unwrap();
    assert!(!p.kani);
    assert!(!p.workspaces.contains("targets/analyzer/kani ->"));
    let examples: BTreeSet<_> = p
        .steps
        .iter()
        .filter_map(|s| {
            s.args
                .iter()
                .position(|a| a == "--example")
                .map(|i| s.args[i + 1].as_str())
        })
        .collect();
    assert_eq!(
        examples,
        BTreeSet::from([
            "check_aggregate_contracts",
            "check_selection_contracts",
            "check_big_helper_contracts",
            "check_ir_contracts",
            "check_expression_contracts",
            "check_graph_contracts",
            "check_dynamic_aggregate_contracts",
            "check_nonlinear_wide_contracts",
        ])
    );
    let tests: Vec<_> = p.steps.iter().filter(|s| s.args[0] == "test").collect();
    assert_eq!(tests.len(), 3);
    assert!(
        tests
            .iter()
            .any(|s| s.args.contains(&"support/Cargo.toml".into()))
    );
    assert!(
        tests
            .iter()
            .any(|s| s.args.contains(&"veryl-formal-contract-tests".into()))
    );
    assert!(
        tests
            .iter()
            .any(|s| s.args.ends_with(&["--lib".into(), "_contract".into()]))
    );
    assert_eq!(p.steps.iter().filter(|s| s.args[0] == "fmt").count(), 4);
    let mut exec = Recording::default();
    setup(Path::new("repo"), &p, &Environment::new(), &mut exec).unwrap();
    assert!(exec.calls.is_empty());
}

#[test]
fn simulator_shards_keep_their_disjoint_group_selection() {
    for suite in ["core", "bitwise", "window-log"] {
        let plan = plan("simulator-contracts", suite, Mode::All).unwrap();
        let adapters: Vec<_> = plan
            .steps
            .iter()
            .filter(|s| s.args.contains(&"run".into()))
            .collect();
        assert_eq!(adapters.len(), 2);
        let groups = |s: &Invocation| {
            s.args
                .windows(2)
                .filter(|p| p[0] == "--group")
                .map(|p| p[1].clone())
                .collect::<Vec<_>>()
        };
        assert_eq!(groups(adapters[0]), groups(adapters[1]));
        assert!(
            groups(adapters[0]).contains(
                &match suite {
                    "core" => "memory",
                    x => x,
                }
                .into()
            )
        );
    }
    let core = plan("simulator-backend-contracts", "core", Mode::All).unwrap();
    assert!(
        core.steps
            .iter()
            .any(|s| s.args.contains(&"check_optimization_contracts".into()))
    );
    for suite in [
        "four-state",
        "array-index",
        "packed-write",
        "packed-read",
        "packed-layout",
    ] {
        let plan = plan("simulator-backend-contracts", suite, Mode::All).unwrap();
        assert!(
            !plan
                .steps
                .iter()
                .any(|s| s.args.contains(&"check_optimization_contracts".into()))
        );
        assert!(
            plan.steps
                .iter()
                .all(|s| s.env["VERYL_AOT_CACHE_DIR"] == "$ROOT/target/formal-aot-cache")
        );
    }
}

#[test]
fn cache_cli_checks_use_the_binary_built_by_the_plan() {
    let p = plan("cache-contracts", "single", Mode::All).unwrap();
    let build = p
        .steps
        .iter()
        .position(|s| s.args.first().map(String::as_str) == Some("build"))
        .unwrap();
    let cli = p
        .steps
        .iter()
        .position(|s| s.env.contains_key("VERYL_BIN"))
        .unwrap();
    assert!(build < cli);
    assert_eq!(p.steps[cli].env["VERYL_BIN"], "$ROOT/target/debug/veryl");
    assert!(p.steps[cli].args.contains(&"--ignored".into()));
}

#[test]
fn setup_installs_pinned_kani_and_preserves_archive_permissions() {
    let mut exec = Recording::default();
    setup(
        Path::new("repo"),
        &plan("frontend-contracts", "single", Mode::All).unwrap(),
        &Environment::new(),
        &mut exec,
    )
    .unwrap();
    assert_eq!(
        exec.calls[0].args,
        [
            "install",
            "--locked",
            "--force",
            "kani-verifier",
            "--version",
            KANI_VERSION
        ]
    );
    assert_eq!(exec.calls[1].args, ["kani", "setup"]);
    assert_eq!(exec.calls[1].env["TAR_OPTIONS"], "--no-same-owner");
    assert_eq!(exec.calls[2].args, ["kani", "--version"]);
}

#[test]
fn cached_kani_restores_its_own_toolchain_without_reinstalling_the_bundle() {
    let dir = tempfile::tempdir().unwrap();
    let bundle = dir.path().join(format!(".kani/kani-{KANI_VERSION}"));
    fs::create_dir_all(&bundle).unwrap();
    fs::write(
        bundle.join("rust-toolchain-version"),
        "nightly-2026-08-21-x86_64-unknown-linux-gnu\n",
    )
    .unwrap();
    let env = Environment::from([
        ("HOME".into(), dir.path().to_str().unwrap().into()),
        ("KANI_CACHE_HIT".into(), "true".into()),
    ]);
    let mut exec = Recording::default();
    let p = plan("value-contracts", "storage", Mode::All).unwrap();
    setup(Path::new("repo"), &p, &env, &mut exec).unwrap();
    assert_eq!(exec.calls[0].program, "rustup");
    assert_eq!(
        exec.calls[0].args[2],
        "nightly-2026-08-21-x86_64-unknown-linux-gnu"
    );
    assert_eq!(exec.calls[1].program, "ln");
    assert_eq!(
        exec.calls[1].args[2],
        bundle.join("toolchain").to_str().unwrap()
    );
    assert!(
        !exec
            .calls
            .iter()
            .any(|s| s.args.contains(&"install".into()) && s.program == "cargo")
    );
    for invalid in ["../escape", "nightly --flag", ""] {
        fs::write(bundle.join("rust-toolchain-version"), invalid).unwrap();
        let mut exec = Recording::default();
        assert!(setup(Path::new("repo"), &p, &env, &mut exec).is_err());
        assert!(exec.calls.is_empty());
    }
}

#[test]
fn setup_uses_icarus_only_for_emitter_and_no_tools_for_native_cache_checks() {
    let mut exec = Recording::default();
    setup(
        Path::new("repo"),
        &plan("cache-contracts", "single", Mode::All).unwrap(),
        &Environment::new(),
        &mut exec,
    )
    .unwrap();
    assert!(exec.calls.is_empty());
    setup(
        Path::new("repo"),
        &plan("emitter-contracts", "single", Mode::All).unwrap(),
        &Environment::new(),
        &mut exec,
    )
    .unwrap();
    assert_eq!(exec.calls.len(), 2);
    assert_eq!(exec.calls[1].args, ["apt-get", "install", "-y", "iverilog"]);
}

struct Integrity {
    replies: VecDeque<String>,
    calls: usize,
}
impl Commands for Integrity {
    fn run(&mut self, _: &Path, _: &str, _: &[&str]) -> Result<String> {
        self.calls += 1;
        Ok(self
            .replies
            .pop_front()
            .expect("unexpected integrity query"))
    }
}

#[test]
fn execution_stops_on_failure_and_still_checks_source_integrity() {
    let candidate = "a".repeat(40);
    let mut commands = Integrity {
        replies: [
            candidate.clone(),
            String::new(),
            candidate.clone(),
            String::new(),
        ]
        .into(),
        calls: 0,
    };
    let mut exec = Recording {
        fail_at: Some(2),
        ..Default::default()
    };
    let p = plan("cache-contracts", "single", Mode::All).unwrap();
    assert!(
        verify(
            Path::new("repo"),
            &candidate,
            None,
            &p,
            &mut exec,
            &mut commands
        )
        .is_err()
    );
    assert_eq!(exec.calls.len(), 2);
    assert_eq!(commands.calls, 4);
}

#[test]
fn dirty_sources_fail_before_execution_or_after_success() {
    let candidate = "a".repeat(40);
    let p = plan("cache-contracts", "single", Mode::All).unwrap();
    for before in [true, false] {
        let mut replies = VecDeque::from([
            candidate.clone(),
            if before {
                " M source.rs".into()
            } else {
                String::new()
            },
        ]);
        if !before {
            replies.extend([candidate.clone(), " M source.rs".into()]);
        }
        let mut commands = Integrity { replies, calls: 0 };
        let mut exec = Recording::default();
        assert!(
            verify(
                Path::new("repo"),
                &candidate,
                None,
                &p,
                &mut exec,
                &mut commands
            )
            .is_err()
        );
        assert_eq!(exec.calls.is_empty(), before);
    }
}

#[test]
fn plan_metadata_is_multiline_and_can_be_read_by_github_actions() {
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().join("output");
    let env = Environment::from([("GITHUB_OUTPUT".into(), output.to_str().unwrap().into())]);
    let p = plan("value-contracts", "storage", Mode::All).unwrap();
    describe(&p, &env).unwrap();
    let text = fs::read_to_string(output).unwrap();
    let mut lines = text.lines();
    assert_eq!(lines.next(), Some("kani=true"));
    assert_eq!(
        lines.next(),
        Some(format!("kani-version={KANI_VERSION}").as_str())
    );
    assert_eq!(lines.next(), Some("workspaces<<FORMAL_WORKSPACES"));
    let mut workspaces = Vec::new();
    for line in lines.by_ref() {
        if line == "FORMAL_WORKSPACES" {
            break;
        }
        workspaces.push(line);
    }
    assert_eq!(workspaces.join("\n"), p.workspaces);
    assert!(lines.next().is_none());
}

#[test]
fn streaming_executor_propagates_real_process_failure() {
    assert!(
        Streaming
            .execute(Path::new("."), &Invocation::new("rustc", &["--version"]))
            .is_ok()
    );
    assert!(
        Streaming
            .execute(
                Path::new("."),
                &Invocation::new("rustc", &["--formal-invalid-option"])
            )
            .is_err()
    );
}

#[test]
fn cli_plan_accepts_a_root_with_spaces_and_rejects_bad_arguments() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("root with spaces");
    fs::create_dir(&root).unwrap();
    let call = |args: &[&str]| {
        std::process::Command::new(env!("CARGO_BIN_EXE_formal-ci"))
            .args(["--root", root.to_str().unwrap()])
            .args(args)
            .env_remove("GITHUB_OUTPUT")
            .output()
            .unwrap()
    };
    let good = call(&["plan", "cache-contracts", "single", "--mode", "positive"]);
    assert!(good.status.success());
    assert!(
        String::from_utf8(good.stdout)
            .unwrap()
            .contains("check_contracts")
    );
    for args in [
        vec!["plan", "cache-contracts"],
        vec!["plan", "cache-contracts", "single", "--mode", "mutation"],
        vec!["plan", "cache-contracts", "single", "extra"],
        vec!["setup", "cache-contracts", "single", "--mode", "positive"],
        vec!["run", "cache-contracts", "unknown"],
    ] {
        assert!(!call(&args).status.success(), "{args:?}");
    }
    assert!(!call(&["run", "cache-contracts", "single"]).status.success());
}

#[test]
fn matrix_preserves_all_shards_and_manual_target_partitions() {
    use std::collections::BTreeSet;
    let rows = |target| {
        matrix(target).unwrap()["include"]
            .as_array()
            .unwrap()
            .clone()
    };
    let identity = |v: &serde_json::Value| {
        (
            v["check"].as_str().unwrap().to_owned(),
            v["suite"].as_str().unwrap().to_owned(),
        )
    };
    let all = rows("all");
    assert_eq!(all.len(), 49);
    let expected: BTreeSet<_> = all.iter().map(identity).collect();
    assert_eq!(all.len(), expected.len());
    let mut selected = BTreeSet::new();
    for target in [
        "value",
        "simulator",
        "synthesizer",
        "emitter",
        "frontend",
        "cache",
    ] {
        let subset = rows(target);
        assert!(!subset.is_empty());
        for shard in subset {
            assert!(
                selected.insert(identity(&shard)),
                "overlapping target selection"
            );
        }
    }
    assert_eq!(selected, expected);
    assert_eq!(rows("simulator").len(), 11);
    assert_eq!(rows("value").len(), 34);
    for shard in all {
        let check = shard["check"].as_str().unwrap();
        let suite = shard["suite"].as_str().unwrap();
        assert!(plan(check, suite, Mode::All).is_ok());
        assert!(plan(check, suite, Mode::Positive).is_ok());
        assert!(shard["timeout"].as_u64().unwrap() > 0);
    }
    assert!(matrix("unknown").is_err());
}

#[test]
fn simulator_kernel_shards_select_known_contracts_and_partition_proofs() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap();
    let all = simulator_catalog::catalog(true);
    let mut assigned = std::collections::BTreeSet::new();
    let mut mutations = std::collections::BTreeSet::new();
    for shard in SHARDS.iter().filter(|s| s.check == "simulator-contracts") {
        let p = plan(shard.check, shard.suite, Mode::All).unwrap();
        for step in p.steps.iter().filter(|s| {
            s.args
                .iter()
                .any(|a| a == "check_contracts" || a == "check_simulator_contracts")
        }) {
            let proof = step.args.iter().any(|a| a == "check_contracts");
            let groups: Vec<_> = step
                .args
                .windows(2)
                .filter(|p| p[0] == "--group")
                .map(|p| p[1].as_str())
                .collect();
            let selected: Vec<_> = simulator_catalog::catalog(proof)
                .iter()
                .filter(|c| groups.contains(&c.group))
                .collect();
            assert!(
                !selected.is_empty(),
                "empty adapter selection: {}",
                shard.suite
            );
            if proof {
                for c in &selected {
                    assert!(
                        assigned.insert(c.name),
                        "overlapping simulator proof: {}",
                        c.name
                    );
                }
                for m in simulator_catalog::mutations(root, &selected, true) {
                    assert!(
                        mutations.insert(m.name),
                        "overlapping simulator mutation: {}",
                        m.name
                    );
                }
            }
        }
    }
    assert_eq!(assigned, all.iter().map(|c| c.name).collect());
    assert_eq!(
        mutations,
        simulator_catalog::mutations(root, &all.iter().collect::<Vec<_>>(), true)
            .iter()
            .map(|m| m.name)
            .collect()
    );
}

#[test]
fn literal_and_metadata_registry_use_the_proof_diagnostics() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap();
    for (name, file) in [
        ("literal-leaf-context", "expression_context.rs"),
        ("resolved-type-copy-update", "metadata.rs"),
        ("literal-metadata-copy", "metadata.rs"),
    ] {
        let source = fs::read_to_string(root.join("targets/analyzer/kani/src").join(file)).unwrap();
        let contract = analyzer::CONTRACTS.iter().find(|c| c.name == name).unwrap();
        for message in std::iter::once(contract.message).chain(contract.covers.iter().copied()) {
            assert!(
                source.contains(&format!("{message:?}")),
                "missing proof diagnostic for {name}: {message}"
            );
        }
    }
}
