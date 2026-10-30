use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use veryl_formal_ci::{
    Commands, Environment, Process, Result, locks,
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
        fs::write(seed.join(".gitignore"), "**/target/\n/support/ci-inputs/\n").unwrap();
        for path in locks::FILES {
            fs::create_dir_all(seed.join(path).parent().unwrap()).unwrap();
            fs::write(seed.join(path), "committed lock\n").unwrap();
            git(&seed, &["add", path]);
        }
        git(
            &seed,
            &["add", "veryl", ".gitmodules", "proof.txt", ".gitignore"],
        );
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
        let mut files: std::collections::BTreeMap<_, _> = locks::FILES
            .into_iter()
            .map(|p| (p, "committed lock\n"))
            .collect();
        files.insert("Cargo.lock", "verified candidate lock\n");
        let path = self.root.join(locks::BUNDLE);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(
            &path,
            json!({"formal":self.formal, "pinned":self.pinned,
            "candidate":self.candidate, "files":files})
            .to_string(),
        )
        .unwrap();
        let digest = git(&self.root, &["hash-object", locks::BUNDLE]);
        [
            ("FORMAL_LOCK_DIGEST", digest),
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
fn successful_run_publishes_the_tested_gitlink_and_exact_locks_to_a_private_remote() {
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
        "Cargo.lock\nveryl"
    );
    assert_eq!(
        git(&f.root, &["show", "HEAD:Cargo.lock"]),
        "verified candidate lock"
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
        ("push", Value::Null, "all", 20),
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

#[test]
fn artifact_corruption_wrong_revisions_and_incomplete_file_sets_are_rejected_before_apply() {
    for change in ["bytes", "formal", "pinned", "candidate", "missing", "extra"] {
        let f = Fixture::new();
        let env = f.env();
        nightly::checkout(&f.root, &f.candidate, &mut Process).unwrap();
        let path = f.root.join(locks::BUNDLE);
        let mut bundle: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        match change {
            "bytes" => bundle["files"]["Cargo.lock"] = json!("corrupted"),
            "formal" | "pinned" | "candidate" => bundle[change] = json!("a".repeat(40)),
            "missing" => {
                bundle["files"]
                    .as_object_mut()
                    .unwrap()
                    .remove("Cargo.lock");
            }
            "extra" => bundle["files"]["../proof.txt"] = json!("unverified"),
            _ => unreachable!(),
        }
        fs::write(&path, bundle.to_string()).unwrap();
        let digest = if change == "bytes" {
            env["FORMAL_LOCK_DIGEST"].clone()
        } else {
            git(&f.root, &["hash-object", locks::BUNDLE])
        };
        assert!(
            locks::apply(&f.root, &f.candidate, &digest, &mut Process).is_err(),
            "{change}"
        );
        for lock in locks::FILES {
            assert_eq!(
                fs::read_to_string(f.root.join(lock)).unwrap(),
                "committed lock\n"
            );
        }
    }
}

#[test]
fn missing_or_changed_artifacts_cannot_be_published() {
    for missing in [true, false] {
        let f = Fixture::new();
        let env = f.env();
        let path = f.root.join(locks::BUNDLE);
        if missing {
            fs::remove_file(path).unwrap();
        } else {
            fs::write(path, "tampered").unwrap();
        }
        assert!(nightly::finalize(&f.root, &env, &mut Process).is_err());
        assert_eq!(git(&f.remote, &["rev-parse", "master"]), f.formal);
    }
}

#[test]
fn staged_lock_changes_cannot_be_overwritten_by_finalization() {
    let f = Fixture::new();
    let env = f.env();
    fs::write(f.root.join("Cargo.lock"), "unverified staged lock").unwrap();
    git(&f.root, &["add", "Cargo.lock"]);
    assert!(nightly::finalize(&f.root, &env, &mut Process).is_err());
    assert_eq!(
        git(&f.root, &["show", ":Cargo.lock"]),
        "unverified staged lock"
    );
    assert_eq!(git(&f.remote, &["rev-parse", "master"]), f.formal);
}

#[test]
fn lock_integrity_is_required_before_and_after_even_a_failed_checker() {
    use veryl_formal_ci::checks::{self, Executor, Invocation, Plan};
    struct Checker {
        calls: usize,
        fail: bool,
    }
    impl Executor for Checker {
        fn execute(&mut self, root: &Path, _: &Invocation) -> Result<()> {
            self.calls += 1;
            fs::write(root.join("Cargo.lock"), "changed during verification")?;
            if self.fail {
                Err(veryl_formal_ci::error("checker failed"))
            } else {
                Ok(())
            }
        }
    }
    let f = Fixture::new();
    let env = f.env();
    let digest = &env["FORMAL_LOCK_DIGEST"];
    nightly::checkout(&f.root, &f.candidate, &mut Process).unwrap();
    let plan = Plan {
        workspaces: "",
        kani: false,
        icarus: false,
        steps: vec![Invocation::new("checker", &[])],
    };
    for fail in [false, true] {
        locks::apply(&f.root, &f.candidate, digest, &mut Process).unwrap();
        let mut checker = Checker { calls: 0, fail };
        assert!(
            checks::verify(
                &f.root,
                &f.candidate,
                Some(digest),
                &plan,
                &mut checker,
                &mut Process
            )
            .is_err()
        );
        assert_eq!(checker.calls, 1);
        let mut checker = Checker {
            calls: 0,
            fail: false,
        };
        assert!(
            checks::verify(
                &f.root,
                &f.candidate,
                Some(digest),
                &plan,
                &mut checker,
                &mut Process
            )
            .is_err()
        );
        assert_eq!(checker.calls, 0);
    }
}

#[test]
fn one_resolution_is_replayed_in_another_checkout_with_locked_dependencies() {
    struct Count {
        updates: usize,
    }
    impl Commands for Count {
        fn run(&mut self, root: &Path, program: &str, args: &[&str]) -> Result<String> {
            if program == "cargo" && args.first() == Some(&"update") {
                self.updates += 1;
            }
            Process.run(root, program, args)
        }
    }
    let dir = tempfile::tempdir().unwrap();
    let upstream = dir.path().join("upstream");
    fs::create_dir_all(upstream.join("extra/src")).unwrap();
    git(&upstream, &["init", "--initial-branch=master"]);
    identity(&upstream);
    let manifest = "[package]\nname='upstream'\nversion='0.1.0'\nedition='2024'\n[workspace]\n[lib]\npath='source.rs'\n";
    fs::write(upstream.join("Cargo.toml"), manifest).unwrap();
    fs::write(upstream.join("source.rs"), "").unwrap();
    fs::write(
        upstream.join("extra/Cargo.toml"),
        "[package]\nname='extra'\nversion='0.1.0'\nedition='2024'\n",
    )
    .unwrap();
    fs::write(upstream.join("extra/src/lib.rs"), "").unwrap();
    git(&upstream, &["add", "Cargo.toml", "source.rs", "extra"]);
    git(&upstream, &["commit", "-m", "Initial dependency graph"]);
    let pinned = git(&upstream, &["rev-parse", "HEAD"]);
    let root = dir.path().join("formal");
    fs::create_dir(&root).unwrap();
    git(&root, &["init", "--initial-branch=master"]);
    identity(&root);
    git(
        &root,
        &[
            "-c",
            "protocol.file.allow=always",
            "submodule",
            "add",
            upstream.to_str().unwrap(),
            "veryl",
        ],
    );
    for (i, lock) in locks::FILES.into_iter().enumerate() {
        let manifest = Path::new(lock).with_file_name("Cargo.toml");
        let parent = manifest.parent().unwrap();
        fs::create_dir_all(root.join(parent)).unwrap();
        let dependency = "../".repeat(parent.components().count()) + "veryl";
        fs::write(root.join(&manifest), format!("[package]\nname='fixture_{i}'\nversion='0.1.0'\nedition='2024'\n[workspace]\nexclude=['veryl']\n[lib]\npath='fixture.rs'\n[dependencies]\nupstream={{path='{dependency}'}}\n")).unwrap();
        fs::write(root.join(parent).join("fixture.rs"), "").unwrap();
        Process
            .run(
                &root,
                "cargo",
                &[
                    "update",
                    "--offline",
                    "--workspace",
                    "--manifest-path",
                    manifest.to_str().unwrap(),
                ],
            )
            .unwrap();
        git(
            &root,
            &[
                "add",
                manifest.to_str().unwrap(),
                lock,
                parent.join("fixture.rs").to_str().unwrap(),
            ],
        );
    }
    fs::write(root.join(".gitignore"), "**/target/\n/support/ci-inputs/\n").unwrap();
    git(&root, &["add", ".gitmodules", ".gitignore", "veryl"]);
    git(&root, &["commit", "-m", "Pin dependency graph"]);
    let mut counted = Count { updates: 0 };
    let pinned_digest = locks::create(&root, &pinned, &mut counted).unwrap();
    assert_eq!(counted.updates, 0);
    locks::verify(&root, &pinned, &pinned_digest, &mut Process).unwrap();
    fs::write(
        upstream.join("Cargo.toml"),
        format!("{manifest}[dependencies]\nextra={{path='extra'}}\n"),
    )
    .unwrap();
    git(&upstream, &["add", "Cargo.toml"]);
    git(&upstream, &["commit", "-m", "Add upstream dependency"]);
    let candidate = git(&upstream, &["rev-parse", "HEAD"]);
    nightly::checkout(&root, &candidate, &mut Process).unwrap();
    assert!(
        Process
            .run(
                &root,
                "cargo",
                &["metadata", "--offline", "--locked", "--format-version", "1"]
            )
            .is_err()
    );
    let digest = locks::create(&root, &candidate, &mut counted).unwrap();
    assert_eq!(counted.updates, locks::FILES.len());
    locks::verify(&root, &candidate, &digest, &mut Process).unwrap();
    let worker = dir.path().join("worker");
    git(
        dir.path(),
        &[
            "-c",
            "protocol.file.allow=always",
            "clone",
            "--recurse-submodules",
            root.to_str().unwrap(),
            worker.to_str().unwrap(),
        ],
    );
    assert_eq!(git(&worker.join("veryl"), &["rev-parse", "HEAD"]), pinned);
    nightly::checkout(&worker, &candidate, &mut Process).unwrap();
    fs::create_dir_all(worker.join(locks::BUNDLE).parent().unwrap()).unwrap();
    fs::copy(root.join(locks::BUNDLE), worker.join(locks::BUNDLE)).unwrap();
    locks::apply(&worker, &candidate, &digest, &mut counted).unwrap();
    assert_eq!(counted.updates, locks::FILES.len());
    for lock in locks::FILES {
        assert_eq!(
            fs::read(root.join(lock)).unwrap(),
            fs::read(worker.join(lock)).unwrap()
        );
        let manifest = Path::new(lock).with_file_name("Cargo.toml");
        Process
            .run(
                &worker,
                "cargo",
                &[
                    "metadata",
                    "--offline",
                    "--locked",
                    "--format-version",
                    "1",
                    "--manifest-path",
                    manifest.to_str().unwrap(),
                ],
            )
            .unwrap();
    }
    locks::verify(&worker, &candidate, &digest, &mut Process).unwrap();
    nightly::verify_clean(&worker, &candidate, &mut Process).unwrap();
}

#[test]
fn candidate_artifact_covers_every_committed_formal_lock_file() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let listed = git(repo, &["ls-files", "--", "Cargo.lock", "**/Cargo.lock"]);
    assert_eq!(listed.lines().collect::<Vec<_>>(), locks::FILES);
}
