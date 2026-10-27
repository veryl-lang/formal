use clap::ValueEnum;
use std::{collections::VecDeque, fs, path::Path};
use veryl_formal_ci::{Commands, Environment, Result, checks::*, error};

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
    for (check, suite) in [
        ("value-contracts", "single"),
        ("simulator-contracts", "window-log"),
        ("simulator-backend-contracts", "core"),
        ("synthesizer-contracts", "single"),
        ("emitter-contracts", "single"),
        ("generated-c-contracts", "single"),
        ("frontend-contracts", "single"),
        ("cache-contracts", "single"),
    ] {
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
    let p = plan("value-contracts", "single", Mode::All).unwrap();
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
    assert!(verify(Path::new("repo"), &candidate, &p, &mut exec, &mut commands).is_err());
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
        assert!(verify(Path::new("repo"), &candidate, &p, &mut exec, &mut commands).is_err());
        assert_eq!(exec.calls.is_empty(), before);
    }
}

#[test]
fn plan_metadata_is_multiline_and_can_be_read_by_github_actions() {
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().join("output");
    let env = Environment::from([("GITHUB_OUTPUT".into(), output.to_str().unwrap().into())]);
    let p = plan("value-contracts", "single", Mode::All).unwrap();
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
    assert_eq!(all.len(), 15);
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
    for shard in all {
        let check = shard["check"].as_str().unwrap();
        let suite = shard["suite"].as_str().unwrap();
        assert!(plan(check, suite, Mode::All).is_ok());
        assert!(plan(check, suite, Mode::Positive).is_ok());
        assert!(shard["timeout"].as_u64().unwrap() > 0);
    }
    assert!(matrix("unknown").is_err());
}
