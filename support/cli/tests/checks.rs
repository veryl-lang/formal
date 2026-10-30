use clap::ValueEnum;
use std::{collections::VecDeque, fs, path::Path};
use veryl_formal_ci::{Commands, Environment, Result, checks::*, error};

#[path = "../../../targets/analyzer/kani/src/bin/check_contracts/analyzer.rs"]
mod analyzer;

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
                        "public-wide-resize"
                            | "public-getters"
                            | "public-native-sequences"
                            | "public-comparison-sequences"
                    )
            );
            if matches!(
                contract.group,
                "public-wide-resize" | "public-getters" | "public-native-sequences"
            ) {
                let expected_suite =
                    if contract.name.contains("shift") || contract.name.contains("big-amount") {
                        "shifts"
                    } else {
                        "storage"
                    };
                assert_eq!(shard.suite, expected_suite);
                let witness = match contract.name {
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
            if contract.group == "public-comparison-sequences" {
                assert_eq!(shard.suite, "logic");
                if contract.name.starts_with("public-small-less-equal-width-") {
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
    // All nine width pairs are required before direct checks omit LessEq.
    for left in 1..=3 {
        for right in 1..=3 {
            let name = format!("public-small-less-equal-width-{left}-by-{right}");
            assert!(
                contracts.contains(name.as_str()),
                "missing comparison width pair: {name}"
            );
        }
    }
    assert_eq!(proof_shards, 5);
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
    assert_eq!(all.len(), 20);
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
    assert_eq!(rows("simulator").len(), 10);
    assert_eq!(rows("value").len(), 6);
    for shard in all {
        let check = shard["check"].as_str().unwrap();
        let suite = shard["suite"].as_str().unwrap();
        assert!(plan(check, suite, Mode::All).is_ok());
        assert!(plan(check, suite, Mode::Positive).is_ok());
        assert!(shard["timeout"].as_u64().unwrap() > 0);
    }
    assert!(matrix("unknown").is_err());
}
