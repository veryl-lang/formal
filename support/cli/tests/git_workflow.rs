use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use veryl_formal_ci::{
    Commands, Environment, Process, Result,
    nightly::{self, CHECKS},
};

struct Fixture {
    _directory: tempfile::TempDir,
    seed: PathBuf,
    remote: PathBuf,
    root: PathBuf,
    pinned: String,
    candidate: String,
    formal: String,
}

fn git(root: &Path, args: &[&str]) -> String {
    Process.run(root, "git", args).unwrap()
}
fn identity(root: &Path) {
    git(root, &["config", "user.name", "Formal tests"]);
    git(root, &["config", "user.email", "test@example.invalid"]);
    git(root, &["config", "commit.gpgsign", "false"]);
}

impl Fixture {
    fn new() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let base = directory.path();
        let upstream = base.join("upstream");
        fs::create_dir(&upstream).unwrap();
        git(&upstream, &["init", "--initial-branch=master"]);
        identity(&upstream);
        fs::write(upstream.join("source.rs"), "old\n").unwrap();
        git(&upstream, &["add", "source.rs"]);
        git(&upstream, &["commit", "-m", "Old source"]);
        let pinned = git(&upstream, &["rev-parse", "HEAD"]);
        fs::write(upstream.join("source.rs"), "new\n").unwrap();
        git(&upstream, &["add", "source.rs"]);
        git(&upstream, &["commit", "-m", "New source"]);
        let candidate = git(&upstream, &["rev-parse", "HEAD"]);
        let seed = base.join("seed");
        fs::create_dir(&seed).unwrap();
        git(&seed, &["init", "--initial-branch=master"]);
        identity(&seed);
        git(
            &seed,
            &[
                "-c",
                "protocol.file.allow=always",
                "submodule",
                "add",
                upstream.to_str().unwrap(),
                "veryl",
            ],
        );
        git(&seed.join("veryl"), &["checkout", "--detach", &pinned]);
        fs::write(seed.join("proof.txt"), "original proof\n").unwrap();
        git(&seed, &["add", "veryl", ".gitmodules", "proof.txt"]);
        git(&seed, &["commit", "-m", "Pin Veryl"]);
        let formal = git(&seed, &["rev-parse", "HEAD"]);
        let remote = base.join("remote.git");
        git(
            base,
            &[
                "init",
                "--bare",
                "--initial-branch=master",
                remote.to_str().unwrap(),
            ],
        );
        git(
            &seed,
            &["remote", "add", "origin", remote.to_str().unwrap()],
        );
        git(&seed, &["push", "origin", "HEAD:master"]);
        let root = base.join("checkout");
        git(
            base,
            &[
                "-c",
                "protocol.file.allow=always",
                "clone",
                "--recurse-submodules",
                remote.to_str().unwrap(),
                root.to_str().unwrap(),
            ],
        );
        Self {
            _directory: directory,
            seed,
            remote,
            root,
            pinned,
            candidate,
            formal,
        }
    }

    fn env(&self) -> Environment {
        let needs = Value::Object(
            CHECKS
                .into_iter()
                .map(|name| (name.into(), json!({"result":"success"})))
                .collect(),
        );
        [
            ("VERYL_CANDIDATE", self.candidate.clone()),
            ("VERYL_PINNED", self.pinned.clone()),
            ("FORMAL_SHA", self.formal.clone()),
            ("DEFAULT_BRANCH", "master".into()),
            ("GITHUB_REPOSITORY", "test/formal".into()),
            ("CHECK_RESULTS", needs.to_string()),
        ]
        .into_iter()
        .map(|(k, v)| (k.into(), v))
        .collect()
    }

    fn advance_branch(&self) -> String {
        fs::write(self.seed.join("proof.txt"), "maintainer change\n").unwrap();
        git(&self.seed, &["add", "proof.txt"]);
        git(&self.seed, &["commit", "-m", "Change proof"]);
        git(&self.seed, &["push", "origin", "HEAD:master"]);
        git(&self.seed, &["rev-parse", "HEAD"])
    }
}

#[test]
fn successful_run_publishes_only_the_tested_gitlink_to_a_private_remote() {
    let f = Fixture::new();
    nightly::finalize(&f.root, &f.env(), &mut Process).unwrap();
    let updated = git(&f.root, &["rev-parse", "HEAD"]);
    assert_ne!(updated, f.formal);
    assert_eq!(git(&f.remote, &["rev-parse", "refs/heads/master"]), updated);
    assert_eq!(git(&f.root, &["rev-parse", "HEAD^"]), f.formal);
    assert_eq!(
        git(
            &f.root,
            &["diff-tree", "--no-commit-id", "--name-only", "-r", "HEAD"]
        ),
        "veryl"
    );
    assert_eq!(git(&f.root, &["rev-parse", "HEAD:veryl"]), f.candidate);
    assert_eq!(
        fs::read_to_string(f.root.join("proof.txt")).unwrap(),
        "original proof\n"
    );
    assert_eq!(
        git(&f.root, &["log", "-1", "--format=%B"]),
        format!("Update Veryl to {}", &f.candidate[..12])
    );
    assert!(git(&f.root, &["status", "--porcelain"]).is_empty());
}

#[test]
fn a_branch_advanced_before_finalization_keeps_both_pointers_unchanged() {
    let f = Fixture::new();
    let advanced = f.advance_branch();
    nightly::finalize(&f.root, &f.env(), &mut Process).unwrap();
    assert_eq!(git(&f.root, &["rev-parse", "HEAD"]), f.formal);
    assert_eq!(git(&f.root.join("veryl"), &["rev-parse", "HEAD"]), f.pinned);
    assert_eq!(
        git(&f.remote, &["rev-parse", "refs/heads/master"]),
        advanced
    );
}

#[test]
fn extra_staged_changes_cannot_be_committed_or_pushed() {
    let f = Fixture::new();
    fs::write(f.root.join("proof.txt"), "not verified\n").unwrap();
    git(&f.root, &["add", "proof.txt"]);
    assert!(nightly::finalize(&f.root, &f.env(), &mut Process).is_err());
    assert_eq!(git(&f.root, &["rev-parse", "HEAD"]), f.formal);
    assert_eq!(
        git(&f.remote, &["rev-parse", "refs/heads/master"]),
        f.formal
    );
}

#[test]
fn concurrent_push_is_rejected_without_overwriting_a_maintainer_change() {
    struct Race<'a>(&'a Fixture);
    impl Commands for Race<'_> {
        fn run(&mut self, cwd: &Path, program: &str, args: &[&str]) -> Result<String> {
            if program == "git" && args.first() == Some(&"push") {
                self.0.advance_branch();
            }
            Process.run(cwd, program, args)
        }
    }
    let f = Fixture::new();
    assert!(nightly::finalize(&f.root, &f.env(), &mut Race(&f)).is_err());
    assert_eq!(
        git(&f.remote, &["rev-parse", "refs/heads/master"]),
        git(&f.seed, &["rev-parse", "HEAD"])
    );
    assert_eq!(git(&f.remote, &["rev-parse", "master:veryl"]), f.pinned);
}

#[test]
fn dirty_sources_fail_integrity_checks() {
    let f = Fixture::new();
    nightly::verify_clean(&f.root, &f.pinned, &mut Process).unwrap();
    fs::write(f.root.join("veryl/source.rs"), "patched\n").unwrap();
    assert!(nightly::verify_clean(&f.root, &f.pinned, &mut Process).is_err());
}

#[test]
fn binary_resolves_paths_from_its_execution_directory() {
    let f = Fixture::new();
    let event = f._directory.path().join("event.json");
    let output = f._directory.path().join("output");
    for (event_name, inputs, mode, shards) in [
        ("push", Value::Null, "all", 19),
        (
            "workflow_dispatch",
            json!({"target":"simulator", "mode":"positive"}),
            "positive",
            10,
        ),
    ] {
        fs::write(
            &event,
            json!({"repository":{"default_branch":"master"}, "inputs":inputs}).to_string(),
        )
        .unwrap();
        fs::write(&output, "").unwrap();
        let result = Command::new(env!("CARGO_BIN_EXE_formal-ci"))
            .arg("prepare")
            .current_dir(&f.root)
            .env("GITHUB_EVENT_NAME", event_name)
            .env("GITHUB_REF_NAME", "master")
            .env("GITHUB_EVENT_PATH", &event)
            .env("GITHUB_OUTPUT", &output)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let text = fs::read_to_string(&output).unwrap();
        assert!(
            text.lines()
                .any(|line| line == format!("pinned={}", f.pinned))
        );
        assert!(
            text.lines()
                .any(|line| line == format!("candidate={}", f.pinned))
        );
        assert!(
            text.lines()
                .any(|line| line == format!("formal_sha={}", f.formal))
        );
        assert!(text.lines().any(|line| line == "update=false"));
        assert!(text.lines().any(|line| line == format!("mode={mode}")));
        let matrix: Value = serde_json::from_str(
            text.lines()
                .find_map(|line| line.strip_prefix("matrix="))
                .unwrap(),
        )
        .unwrap();
        assert_eq!(matrix["include"].as_array().unwrap().len(), shards);
    }
}

#[test]
fn incomplete_or_unknown_commands_fail_instead_of_certifying_a_step() {
    let directory = tempfile::tempdir().unwrap();
    for args in [
        vec![],
        vec!["unknown"],
        vec!["verify-clean"],
        vec!["verify-clean", "invalid"],
        vec!["prepare", "extra"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_formal-ci"))
            .args(&args)
            .current_dir(directory.path())
            .output()
            .unwrap();
        assert!(!output.status.success(), "accepted {args:?}");
    }
    for args in [
        vec!["--help"],
        vec!["plan", "--help"],
        vec!["run", "--help"],
        vec!["help", "setup"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_formal-ci"))
            .args(&args)
            .output()
            .unwrap();
        assert!(output.status.success(), "{args:?}");
        let help = String::from_utf8(output.stdout).unwrap();
        assert!(help.contains("Usage:"));
        if args[0] == "plan" || args[0] == "run" {
            assert!(help.contains("--mode"));
            assert!(help.contains("positive"));
        }
    }
}
