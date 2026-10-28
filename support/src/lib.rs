//! Host-only orchestration shared by proof and finite comparison adapters.
//!
//! Kani success requires verification and every declared cover. Mutation success
//! requires the selected semantic counterexample; build, unrelated safety, and
//! timeout failures alone never qualify. Native comparisons remain tests, not proofs.
//! Snapshots isolate mutated production sources from originals and reference models;
//! separate build directories prevent Cargo from reusing mutated native libraries.
//! Native mutations reuse incremental compiler state only inside their isolated
//! build directory; original checks retain the caller's compilation settings.
//!
//! Progress metadata is written before each check. Only completed runs become
//! successful; interrupted checks remain unfinished. CI artifacts are diagnostic
//! outputs, not a second registry of verified revisions. Target adapters own their
//! catalogs, assumptions, and mutations.

use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Output};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

pub mod summary;
use summary::{Phase, Status, Summary};

pub struct Contract {
    pub name: &'static str,
    pub group: &'static str,
    pub harness: &'static str,
    pub message: &'static str,
    pub covers: &'static [&'static str],
}

pub struct Mutation {
    pub name: &'static str,
    pub contract: &'static Contract,
    pub path: &'static str,
    pub source: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    All,
    Positive,
    Mutation,
}

#[derive(Debug, Default)]
pub struct Options {
    pub target: Option<String>,
    pub groups: Vec<String>,
    pub contracts: Vec<String>,
    pub mode: Mode,
    pub list: bool,
    pub help: bool,
}

pub const USAGE: &str = "Usage: check_contracts [--target NAME] [--group NAME]... [--contract NAME]... [--mode all|positive|mutation] [--list]\nNo arguments runs all positive checks and mutations. Mutation mode alone does not prove the original implementation.";

impl Options {
    pub fn parse(args: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let mut result = Self::default();
        let mut args = args.into_iter();
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--help" | "-h" => result.help = true,
                "--list" => result.list = true,
                "--target" | "--group" | "--contract" | "--mode" => {
                    let value = args
                        .next()
                        .filter(|v| !v.starts_with('-'))
                        .ok_or_else(|| format!("missing value for {arg}"))?;
                    match arg.as_str() {
                        "--target" => result.target = Some(value),
                        "--group" => result.groups.push(value),
                        "--contract" => result.contracts.push(value),
                        _ => {
                            result.mode = match value.as_str() {
                                "all" => Mode::All,
                                "positive" => Mode::Positive,
                                "mutation" => Mode::Mutation,
                                _ => return Err(format!("unknown mode: {value}")),
                            }
                        }
                    }
                }
                _ => return Err(format!("unknown option: {arg}")),
            }
        }
        Ok(result)
    }

    pub fn select<'a>(
        &self,
        target: &str,
        catalog: &'a [Contract],
    ) -> Result<Vec<&'a Contract>, String> {
        if self.target.as_deref().is_some_and(|name| name != target) {
            return Err(format!("this executable supports target {target}"));
        }
        for group in &self.groups {
            if !catalog.iter().any(|c| c.group == group) {
                return Err(format!("unknown group: {group}"));
            }
        }
        for name in &self.contracts {
            if !catalog.iter().any(|c| c.name == name) {
                return Err(format!("unknown contract: {name}"));
            }
        }
        let selected: Vec<_> = catalog
            .iter()
            .filter(|c| {
                (self.groups.is_empty() || self.groups.iter().any(|group| c.group == group))
                    && (self.contracts.is_empty()
                        || self.contracts.iter().any(|name| name == c.name))
            })
            .collect();
        if selected.is_empty() {
            return Err("selection contains no contracts".into());
        }
        Ok(selected)
    }
}

fn check_has(log: &str, message: &str, status: &str) -> bool {
    log.split("Check ").any(|block| {
        let description = block
            .lines()
            .find_map(|line| line.trim().strip_prefix("- Description: "));
        description.is_some_and(|value| value.trim_matches('"') == message)
            && block
                .lines()
                .any(|line| line.trim() == format!("- Status: {status}"))
    })
}

pub fn positive_result(success: bool, log: &str, contract: &Contract) -> bool {
    success
        && log.contains("VERIFICATION:- SUCCESSFUL")
        && !log.contains("VERIFICATION:- FAILED")
        && contract
            .covers
            .iter()
            .all(|cover| check_has(log, cover, "SATISFIED"))
}

pub fn mutation_result(success: bool, log: &str, contract: &Contract) -> bool {
    !success
        && log.contains("VERIFICATION:- FAILED")
        && check_has(log, contract.message, "FAILURE")
        && log.split("Check ").skip(1).all(|block| {
            let description = block.lines().find_map(|line| {
                line.trim()
                    .strip_prefix("- Description: ")
                    .map(|value| value.trim_matches('"'))
            });
            let status = block
                .lines()
                .find_map(|line| line.trim().strip_prefix("- Status: "));
            match (description, status) {
                (Some(_), Some("SUCCESS" | "UNREACHABLE")) => true,
                (Some(message), Some("FAILURE")) => message == contract.message,
                (Some(message), Some("SATISFIED" | "UNSATISFIABLE")) => {
                    contract.covers.contains(&message)
                }
                _ => false,
            }
        })
}

#[derive(Clone, Copy)]
pub enum Backend<'a> {
    Kani {
        harness_timeout: &'a str,
    },
    NativeTests {
        package: &'a str,
        test_target: &'a str,
    },
}

pub fn native_positive_result(success: bool, log: &str, contract: &Contract) -> bool {
    success
        && log
            .lines()
            .any(|line| line == format!("test {} ... ok", contract.harness))
        && log.contains("test result: ok. 1 passed; 0 failed;")
}

pub fn native_mutation_result(exit_code: Option<i32>, log: &str, contract: &Contract) -> bool {
    exit_code == Some(101)
        && log
            .lines()
            .any(|line| line == format!("test {} ... FAILED", contract.harness))
        && log.contains("test result: FAILED. 0 passed; 1 failed;")
        && log.contains("panicked at")
        && log.lines().any(|line| line.contains(contract.message))
}

impl Backend<'_> {
    fn name(self) -> &'static str {
        match self {
            Self::Kani { .. } => "kani",
            Self::NativeTests { .. } => "native-tests",
        }
    }
    fn build_targets(self, repo: &Path, target: &Path, unique: &str) -> (PathBuf, PathBuf) {
        match self {
            Self::Kani { .. } => (target.to_owned(), target.to_owned()),
            Self::NativeTests { .. } => (
                repo.join("target"),
                target
                    .join("native-mutations")
                    .join(format!("run-{unique}")),
            ),
        }
    }
    fn run(self, manifest: &Path, target: &Path, contract: &Contract, phase: Phase) -> Output {
        match self {
            Self::Kani { harness_timeout } => {
                run_proof(manifest, target, contract, harness_timeout)
            }
            Self::NativeTests {
                package,
                test_target,
            } => native_test_command(manifest, target, contract, package, test_target, phase)
                .output()
                .expect("failed to launch timeout/cargo test"),
        }
    }
    fn positive(self, success: bool, log: &str, contract: &Contract) -> bool {
        match self {
            Self::Kani { .. } => positive_result(success, log, contract),
            Self::NativeTests { .. } => native_positive_result(success, log, contract),
        }
    }
    fn mutation(self, output: &Output, log: &str, contract: &Contract) -> bool {
        match self {
            Self::Kani { .. } => mutation_result(output.status.success(), log, contract),
            Self::NativeTests { .. } => native_mutation_result(output.status.code(), log, contract),
        }
    }
}

fn native_test_command(
    manifest: &Path,
    target: &Path,
    contract: &Contract,
    package: &str,
    test_target: &str,
    phase: Phase,
) -> Command {
    let mut command = Command::new("timeout");
    command
        .args([
            "--kill-after=5s",
            "600s",
            "cargo",
            "test",
            "--locked",
            "--manifest-path",
        ])
        .arg(manifest)
        .args([
            "--package",
            package,
            "--test",
            test_target,
            contract.harness,
            "--",
            "--exact",
            "--nocapture",
        ])
        .env("CARGO_TARGET_DIR", target)
        .env("CARGO_TERM_COLOR", "never");
    if phase == Phase::Mutation {
        command.env("CARGO_INCREMENTAL", "1");
    }
    command
}

fn output_text(output: &Output) -> String {
    format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn run_proof(manifest: &Path, target: &Path, contract: &Contract, harness_timeout: &str) -> Output {
    Command::new("cargo")
        .args([
            "kani",
            "--lib",
            "--harness",
            contract.harness,
            "--harness-timeout",
            harness_timeout,
            "-Z",
            "unstable-options",
            "--manifest-path",
        ])
        .arg(manifest)
        .env("CARGO_TARGET_DIR", target)
        .output()
        .expect("failed to launch cargo kani")
}

fn copy_file(source: &Path, destination: &Path) {
    let modified = fs::metadata(source).unwrap().modified().unwrap();
    fs::copy(source, destination).unwrap();
    // Parser generation uses mtimes to decide whether generated sources are stale.
    fs::File::options()
        .write(true)
        .open(destination)
        .unwrap()
        .set_times(fs::FileTimes::new().set_modified(modified))
        .unwrap();
}

fn copy_tree(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        if ["target", ".git", ".local"]
            .iter()
            .any(|name| entry.file_name() == *name)
        {
            continue;
        }
        let dst = destination.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &dst);
        } else {
            copy_file(&entry.path(), &dst);
        }
    }
}

struct Scratch(PathBuf);

fn copy_fixtures(source: &Path, destination: &Path) -> std::io::Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let destination = destination.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_fixtures(&entry.path(), &destination)?;
        } else {
            fs::copy(entry.path(), destination)?;
        }
    }
    Ok(())
}

impl Drop for Scratch {
    fn drop(&mut self) {
        // Only remove this run's unique, owned snapshot.
        let _ = fs::remove_dir_all(&self.0);
    }
}

pub struct Runner<'a> {
    pub target: &'a str,
    pub repo: &'a Path,
    pub manifest: &'a Path,
    pub files: &'a [&'a str],
    pub directories: &'a [&'a str],
}

impl Runner<'_> {
    pub fn run(&self, mode: Mode, contracts: &[&Contract], mutations: &[Mutation]) {
        self.run_with_backend(
            Backend::Kani {
                harness_timeout: "60s",
            },
            mode,
            contracts,
            mutations,
        );
    }

    pub fn run_with_backend(
        &self,
        backend: Backend<'_>,
        mode: Mode,
        contracts: &[&Contract],
        mutations: &[Mutation],
    ) {
        assert!(!contracts.is_empty(), "selection contains no contracts");
        assert!(
            mode != Mode::Mutation || !mutations.is_empty(),
            "selected contracts have no mutations"
        );
        if matches!(backend, Backend::NativeTests { .. }) {
            assert!(
                contracts.iter().all(|c| c.covers.is_empty()),
                "native tests cannot satisfy Kani covers"
            );
        }
        if matches!(backend, Backend::Kani { .. }) {
            let version = Command::new("cargo")
                .args(["kani", "--version"])
                .output()
                .expect("install kani-verifier 0.68.0 first");
            assert!(
                version.status.success()
                    && String::from_utf8_lossy(&version.stdout)
                        .lines()
                        .any(|line| line == "Kani Rust Verifier 0.68.0 (cargo plugin)"),
                "the output parser requires Kani 0.68.0"
            );
        }
        assert!(
            Command::new("cargo")
                .args([
                    "metadata",
                    "--no-deps",
                    "--format-version",
                    "1",
                    "--locked",
                    "--manifest-path"
                ])
                .arg(self.manifest)
                .output()
                .unwrap()
                .status
                .success()
        );
        let unique = format!(
            "{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let target = self.manifest.parent().unwrap().join("target");
        let (build_target, mutation_target) = backend.build_targets(self.repo, &target, &unique);
        let logs = target.join("contract-logs").join(format!("run-{unique}"));
        fs::create_dir_all(&logs).unwrap();
        let selection = contracts
            .iter()
            .map(|c| c.name)
            .collect::<Vec<_>>()
            .join(",");
        let executable = std::env::current_exe().unwrap();
        let adapter = executable.file_stem().unwrap().to_string_lossy();
        let adapter_kind = if executable.parent().is_some_and(|p| p.ends_with("examples")) {
            "example"
        } else {
            "bin"
        };
        let ci_run_id = std::env::var("GITHUB_RUN_ID").unwrap_or_default();
        let ci_run_attempt = std::env::var("GITHUB_RUN_ATTEMPT").unwrap_or_default();
        let ci_job = std::env::var("GITHUB_JOB").unwrap_or_default();
        let mut summary = Summary {
            ci_run_id,
            ci_run_attempt,
            ci_job,
            target: self.target.into(),
            backend: backend.name().into(),
            mode: format!("{mode:?}"),
            contracts: selection,
            manifest: self
                .manifest
                .strip_prefix(self.repo)
                .unwrap()
                .display()
                .to_string(),
            adapter: adapter.into_owned(),
            adapter_kind: adapter_kind.into(),
            build_target: build_target.display().to_string(),
            mutation_build_target: mutation_target.display().to_string(),
            status: Status::Running,
            positive_passes: Some(0),
            mutation_passes: Some(0),
            ..Summary::default()
        };
        let summary_path = logs.join("summary.txt");
        summary.write(&summary_path).unwrap();
        let mut positives = 0;
        let mut negatives = 0;
        if mode != Mode::Mutation {
            for contract in contracts {
                summary.phase = Some(Phase::Positive);
                summary.active_contract = contract.name.into();
                summary.active_harness = contract.harness.into();
                summary.positive_passes = Some(positives);
                summary.write(&summary_path).unwrap();
                let start = Instant::now();
                let output = backend.run(self.manifest, &build_target, contract, Phase::Positive);
                let log = output_text(&output);
                fs::write(logs.join(format!("{}.log", contract.name)), &log).unwrap();
                let accepted = backend.positive(output.status.success(), &log, contract);
                if !accepted {
                    summary.status = Status::Failed;
                    summary.exit_code = format!("{:?}", output.status.code());
                    summary.write(&summary_path).unwrap();
                }
                assert!(
                    accepted,
                    "{} check or required cover failed; see {}",
                    contract.name,
                    logs.display()
                );
                positives += 1;
                match backend {
                    Backend::Kani { .. } => println!(
                        "PASS {} and {} reachability checks ({:.2}s wall, including build)",
                        contract.name,
                        contract.covers.len(),
                        start.elapsed().as_secs_f64()
                    ),
                    Backend::NativeTests { .. } => println!(
                        "PASS tested {} ({:.2}s wall, including build)",
                        contract.name,
                        start.elapsed().as_secs_f64()
                    ),
                }
            }
        }
        if mode != Mode::Positive && !mutations.is_empty() {
            summary.phase = Some(Phase::MutationPreparation);
            summary.active_contract.clear();
            summary.active_harness.clear();
            summary.positive_passes = Some(positives);
            summary.write(&summary_path).unwrap();
            let scratch = Scratch(
                std::env::temp_dir().join(format!("veryl-formal-{}-{unique}", self.target)),
            );
            fs::create_dir(&scratch.0).unwrap();
            for file in self.files {
                copy_file(&self.repo.join(file), &scratch.0.join(file));
            }
            for directory in self.directories {
                copy_tree(&self.repo.join(directory), &scratch.0.join(directory));
            }
            let relative_manifest = self.manifest.strip_prefix(self.repo).unwrap();
            let fixtures = scratch
                .0
                .join(relative_manifest.parent().unwrap())
                .join("target/fixtures");
            for mutant in mutations {
                assert!(
                    Path::new(mutant.path)
                        .components()
                        .all(|c| matches!(c, Component::Normal(_))),
                    "mutation path must stay within the snapshot"
                );
                assert!(
                    contracts.iter().any(|c| c.name == mutant.contract.name),
                    "mutation belongs to an unselected contract"
                );
                let original = fs::read(self.repo.join(mutant.path)).unwrap();
                if fixtures.exists() {
                    fs::remove_dir_all(&fixtures).unwrap();
                }
                fs::write(scratch.0.join(mutant.path), &mutant.source).unwrap();
                summary.phase = Some(Phase::Mutation);
                summary.active_contract = mutant.contract.name.into();
                summary.active_harness = mutant.contract.harness.into();
                summary.active_mutation = mutant.name.into();
                summary.mutation_passes = Some(negatives);
                summary.write(&summary_path).unwrap();
                let start = Instant::now();
                let output = backend.run(
                    &scratch.0.join(relative_manifest),
                    &mutation_target,
                    mutant.contract,
                    Phase::Mutation,
                );
                let mut log = output_text(&output);
                fs::write(logs.join(format!("{}.log", mutant.name)), &log).unwrap();
                fs::write(scratch.0.join(mutant.path), original).unwrap();
                let accepted = backend.mutation(&output, &log, mutant.contract);
                if !accepted {
                    summary.status = Status::Failed;
                    summary.exit_code = format!("{:?}", output.status.code());
                    summary.write(&summary_path).unwrap();
                    if fixtures.exists() {
                        let destination = logs.join("fixtures").join(mutant.name);
                        copy_fixtures(&fixtures, &destination).unwrap();
                        log.push_str(&format!(
                            "\nMutation fixtures preserved at {}\n",
                            destination.display()
                        ));
                        fs::write(logs.join(format!("{}.log", mutant.name)), &log).unwrap();
                    }
                }
                assert!(
                    accepted,
                    "mutation was not rejected by the semantic assertion: {}; see {}",
                    mutant.name,
                    logs.display()
                );
                negatives += 1;
                println!(
                    "PASS rejected {} with a {} ({:.2}s wall)",
                    mutant.name,
                    match backend {
                        Backend::Kani { .. } => "semantic counterexample",
                        Backend::NativeTests { .. } => "reference comparison failure",
                    },
                    start.elapsed().as_secs_f64()
                );
            }
        }
        summary.status = Status::Success;
        summary.positive_passes = Some(positives);
        summary.mutation_passes = Some(negatives);
        summary.write(&summary_path).unwrap();
        println!(
            "PASS selected contracts: {positives} positive, {negatives} mutations; detailed logs: {}",
            logs.display()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    static CONTRACTS: [Contract; 2] = [
        Contract {
            name: "one",
            group: "values",
            harness: "h1",
            message: "one mismatch",
            covers: &["case one"],
        },
        Contract {
            name: "two",
            group: "layout",
            harness: "h2",
            message: "two mismatch",
            covers: &[],
        },
    ];
    fn parse(args: &[&str]) -> Result<Options, String> {
        Options::parse(args.iter().map(|s| s.to_string()))
    }
    #[test]
    fn defaults_select_every_contract() {
        let options = parse(&[]).unwrap();
        assert_eq!(options.mode, Mode::All);
        assert_eq!(options.select("analyzer", &CONTRACTS).unwrap().len(), 2);
    }
    #[test]
    fn filters_intersect_and_repeated_contracts_form_a_set() {
        let options = parse(&[
            "--target",
            "analyzer",
            "--group",
            "layout",
            "--contract",
            "one",
            "--contract",
            "two",
            "--contract",
            "two",
            "--mode",
            "positive",
        ])
        .unwrap();
        let selected = options.select("analyzer", &CONTRACTS).unwrap();
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].name, "two");
        assert_eq!(options.mode, Mode::Positive);
    }
    #[test]
    fn repeated_groups_form_a_union_without_duplicates() {
        let options = parse(&[
            "--group", "values", "--group", "layout", "--group", "values",
        ])
        .unwrap();
        let selected = options.select("analyzer", &CONTRACTS).unwrap();
        assert_eq!(
            selected.iter().map(|c| c.name).collect::<Vec<_>>(),
            vec!["one", "two"]
        );
    }
    #[test]
    fn group_union_intersects_contracts_and_rejects_unknown_members() {
        let options = parse(&[
            "--group",
            "values",
            "--group",
            "layout",
            "--contract",
            "one",
        ])
        .unwrap();
        let selected = options.select("analyzer", &CONTRACTS).unwrap();
        assert_eq!(
            selected.iter().map(|c| c.name).collect::<Vec<_>>(),
            vec!["one"]
        );
        assert!(
            parse(&["--group", "values", "--group", "missing"])
                .unwrap()
                .select("analyzer", &CONTRACTS)
                .is_err()
        );
    }
    #[test]
    fn rejects_typos_and_empty_selections() {
        assert!(parse(&["--mode", "typo"]).is_err());
        assert!(parse(&["--contract"]).is_err());
        assert!(parse(&["--typo"]).is_err());
        for args in [
            vec!["--target", "simulator"],
            vec!["--group", "missing"],
            vec!["--contract", "missing"],
            vec!["--group", "layout", "--contract", "one"],
        ] {
            assert!(
                parse(&args)
                    .unwrap()
                    .select("analyzer", &CONTRACTS)
                    .is_err()
            );
        }
    }
    #[test]
    fn positive_requires_exit_status_and_every_exact_cover() {
        let log = "Check 1:\n - Status: SATISFIED\n - Description: \"case one\"\nVERIFICATION:- SUCCESSFUL";
        assert!(positive_result(true, log, &CONTRACTS[0]));
        assert!(!positive_result(false, log, &CONTRACTS[0]));
        assert!(!positive_result(
            true,
            &log.replace("case one", "case one extra"),
            &CONTRACTS[0]
        ));
        assert!(!positive_result(
            true,
            &log.replace("SATISFIED", "UNSATISFIABLE"),
            &CONTRACTS[0]
        ));
        assert!(!positive_result(
            true,
            &format!("{log}\nVERIFICATION:- FAILED"),
            &CONTRACTS[0]
        ));
    }
    #[test]
    fn negative_requires_failure_at_the_selected_assertion() {
        let log = "Check 1:\n - Status: FAILURE\n - Description: \"\"one mismatch\"\"\nVERIFICATION:- FAILED";
        assert!(mutation_result(false, log, &CONTRACTS[0]));
        assert!(!mutation_result(true, log, &CONTRACTS[0]));
        assert!(!mutation_result(false, log, &CONTRACTS[1]));
        assert!(!mutation_result(false, "compile error", &CONTRACTS[0]));
        assert!(!mutation_result(false, "CBMC timed out", &CONTRACTS[0]));
    }
    #[test]
    fn negative_rejects_mixed_safety_and_incomplete_checks() {
        let semantic = "Check 1:\n - Status: FAILURE\n - Description: \"one mismatch\"\n";
        for status in ["FAILURE", "UNDETERMINED", "UNKNOWN"] {
            let log = format!(
                "{semantic}Check 2:\n - Status: {status}\n - Description: \"pointer outside bounds\"\nVERIFICATION:- FAILED"
            );
            assert!(!mutation_result(false, &log, &CONTRACTS[0]));
        }
        for status in ["SUCCESS", "UNREACHABLE", "SATISFIED", "UNSATISFIABLE"] {
            let description = if matches!(status, "SUCCESS" | "UNREACHABLE") {
                "pointer outside bounds"
            } else {
                "case one"
            };
            let log = format!(
                "{semantic}Check 2:\n - Status: {status}\n - Description: \"{description}\"\nVERIFICATION:- FAILED"
            );
            assert!(mutation_result(false, &log, &CONTRACTS[0]));
        }
        let truncated = format!("{semantic}Check 2:\nVERIFICATION:- FAILED");
        assert!(!mutation_result(false, &truncated, &CONTRACTS[0]));
    }
    #[test]
    fn does_not_join_status_and_description_from_different_checks() {
        let log = "Check 1:\n - Status: FAILURE\n - Description: \"bounds check\"\nCheck 2:\n - Status: SUCCESS\n - Description: \"one mismatch\"\nVERIFICATION:- FAILED";
        assert!(!mutation_result(false, log, &CONTRACTS[0]));
    }
    #[test]
    fn native_mutation_artifacts_are_isolated_from_originals_and_other_runs() {
        let backend = Backend::NativeTests {
            package: "p",
            test_target: "t",
        };
        let (original, mutated) = backend.build_targets(
            Path::new("repo"),
            Path::new("targets/example/tests/target"),
            "one",
        );
        let (other_original, other_mutated) = backend.build_targets(
            Path::new("repo"),
            Path::new("targets/example/tests/target"),
            "two",
        );
        assert_eq!(original, Path::new("repo/target"));
        assert_eq!(original, other_original);
        assert_ne!(original, mutated);
        assert_ne!(mutated, other_mutated);
    }
    #[test]
    #[should_panic(expected = "native tests cannot satisfy Kani covers")]
    fn native_backend_rejects_kani_covers_before_launching_tools() {
        Runner {
            target: "analyzer",
            repo: Path::new("missing"),
            manifest: Path::new("missing"),
            files: &[],
            directories: &[],
        }
        .run_with_backend(
            Backend::NativeTests {
                package: "p",
                test_target: "t",
            },
            Mode::Positive,
            &[&CONTRACTS[0]],
            &[],
        );
    }
    #[test]
    fn native_positive_requires_exactly_one_named_test() {
        let log = "test h1 ... ok\n\ntest result: ok. 1 passed; 0 failed; 2 filtered out;";
        assert!(native_positive_result(true, log, &CONTRACTS[0]));
        assert!(!native_positive_result(false, log, &CONTRACTS[0]));
        assert!(!native_positive_result(
            true,
            &log.replace("h1", "h2"),
            &CONTRACTS[0]
        ));
        assert!(!native_positive_result(
            true,
            "test result: ok. 0 passed; 0 failed;",
            &CONTRACTS[0]
        ));
    }
    #[test]
    fn native_mutation_requires_the_named_test_and_semantic_failure() {
        let log = "test h1 ... FAILED\ntest result: FAILED. 0 passed; 1 failed;\nthread 'h1' panicked at source.rs:1:\nassertion failed: one mismatch";
        assert!(native_mutation_result(Some(101), log, &CONTRACTS[0]));
        for code in [Some(0), Some(124), Some(137), None] {
            assert!(!native_mutation_result(code, log, &CONTRACTS[0]));
        }
        assert!(!native_mutation_result(
            Some(101),
            &log.replace("one mismatch", "bounds check"),
            &CONTRACTS[0]
        ));
        assert!(!native_mutation_result(
            Some(101),
            &log.replace("h1", "h2"),
            &CONTRACTS[0]
        ));
        assert!(!native_mutation_result(
            Some(101),
            "compile error: one mismatch",
            &CONTRACTS[0]
        ));
    }
    #[test]
    fn snapshot_excludes_local_plans_and_artifacts_and_preserves_mtime() {
        let unique = format!(
            "veryl-runner-test-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let scratch = Scratch(std::env::temp_dir().join(unique));
        let source = scratch.0.join("source");
        let destination = scratch.0.join("copy");
        for dir in [".local", "target", ".git", "nested"] {
            fs::create_dir_all(source.join(dir)).unwrap();
            fs::write(source.join(dir).join("data"), "content").unwrap();
        }
        copy_tree(&source, &destination);
        assert!(destination.join("nested/data").is_file());
        for dir in [".local", "target", ".git"] {
            assert!(!destination.join(dir).exists());
        }
        assert_eq!(
            fs::metadata(source.join("nested/data"))
                .unwrap()
                .modified()
                .unwrap(),
            fs::metadata(destination.join("nested/data"))
                .unwrap()
                .modified()
                .unwrap()
        );
    }
    #[test]
    fn incremental_mutations_rebuild_changes_and_reject_an_original_control() {
        let directory = Scratch(std::env::temp_dir().join(format!(
            "veryl-incremental-test-{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
        )));
        let package = directory.0.join("toy");
        fs::create_dir_all(package.join("src")).unwrap();
        fs::create_dir_all(package.join("tests")).unwrap();
        fs::write(
            package.join("Cargo.toml"),
            "[package]\nname=\"incremental-probe\"\nversion=\"0.1.0\"\nedition=\"2024\"\n[workspace]\n",
        )
        .unwrap();
        fs::write(
            package.join("Cargo.lock"),
            "version = 4\n[[package]]\nname = \"incremental-probe\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        let original = "pub const VALUE: u8 = 0;";
        fs::write(package.join("src/lib.rs"), original).unwrap();
        fs::write(
            package.join("tests/probe.rs"),
            r#"
#[test]
fn check() {
    println!("INCREMENTAL={}", std::env::var("CARGO_INCREMENTAL").unwrap_or_default());
    assert_eq!(incremental_probe::VALUE, 0, "semantic mismatch");
}
"#,
        )
        .unwrap();
        static CONTRACT: Contract = Contract {
            name: "incremental-check",
            group: "incremental",
            harness: "check",
            message: "semantic mismatch",
            covers: &[],
        };
        let mutations = [("changed", 1), ("original-control", 0)].map(|(name, value)| Mutation {
            name,
            contract: &CONTRACT,
            path: "toy/src/lib.rs",
            source: format!("pub const VALUE: u8 = {value};"),
        });
        let result = std::panic::catch_unwind(|| {
            Runner {
                target: "incremental-test",
                repo: &directory.0,
                manifest: &package.join("Cargo.toml"),
                files: &[],
                directories: &["toy"],
            }
            .run_with_backend(
                Backend::NativeTests {
                    package: "incremental-probe",
                    test_target: "probe",
                },
                Mode::All,
                &[&CONTRACT],
                &mutations,
            );
        });
        assert!(
            result.is_err(),
            "an original control must not count as a rejected mutation"
        );
        let runs: Vec<_> = fs::read_dir(package.join("target/contract-logs"))
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect();
        assert_eq!(runs.len(), 1);
        let run = &runs[0];
        let summary = Summary::parse(&fs::read_to_string(run.join("summary.txt")).unwrap());
        assert_eq!(summary.status, Status::Failed);
        assert_eq!(summary.positive_passes, Some(1));
        assert_eq!(summary.mutation_passes, Some(1));
        assert_eq!(summary.active_mutation, "original-control");
        assert_eq!(summary.exit_code, "Some(0)");
        assert_ne!(summary.build_target, summary.mutation_build_target);
        assert!(
            Path::new(&summary.mutation_build_target)
                .join("debug/incremental")
                .is_dir()
        );
        let inherited = std::env::var("CARGO_INCREMENTAL").unwrap_or_default();
        let positive = fs::read_to_string(run.join("incremental-check.log")).unwrap();
        assert!(
            positive
                .lines()
                .any(|line| line == format!("INCREMENTAL={inherited}"))
        );
        for name in ["changed", "original-control"] {
            let log = fs::read_to_string(run.join(format!("{name}.log"))).unwrap();
            assert!(log.lines().any(|line| line == "INCREMENTAL=1"));
        }
        assert_eq!(
            fs::read_to_string(package.join("src/lib.rs")).unwrap(),
            original
        );
    }
    #[test]
    fn failed_mutation_retains_only_its_fixtures_after_snapshot_cleanup() {
        let directory = Scratch(std::env::temp_dir().join(format!(
            "veryl-fixture-test-{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
        )));
        let package = directory.0.join("toy");
        fs::create_dir_all(package.join("src")).unwrap();
        fs::create_dir_all(package.join("tests")).unwrap();
        fs::write(
            package.join("Cargo.toml"),
            "[package]\nname=\"fixture-probe\"\nversion=\"0.1.0\"\nedition=\"2024\"\n[workspace]\n",
        )
        .unwrap();
        fs::write(
            package.join("Cargo.lock"),
            "version = 4\n[[package]]\nname = \"fixture-probe\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        let original = "pub const MODE: u8 = 0;";
        fs::write(package.join("src/lib.rs"), original).unwrap();
        fs::write(
            package.join("tests/probe.rs"),
            r#"
#[test]
fn check() {
    let mode = fixture_probe::MODE;
    if mode == 0 { return; }
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target/fixtures");
    let file = root.join(if mode == 1 { "old.txt" } else { "target/witness.txt" });
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(&file, "counterexample").unwrap();
    eprintln!("FIXTURE={}", file.display());
    if mode == 1 { panic!("semantic mismatch"); }
    panic!("unrelated failure");
}
"#,
        )
        .unwrap();
        static CONTRACT: Contract = Contract {
            name: "fixture-check",
            group: "fixtures",
            harness: "check",
            message: "semantic mismatch",
            covers: &[],
        };
        let mutations = [1, 2].map(|mode| Mutation {
            name: if mode == 1 { "accepted" } else { "failed" },
            contract: &CONTRACT,
            path: "toy/src/lib.rs",
            source: format!("pub const MODE: u8 = {mode};"),
        });
        let result = std::panic::catch_unwind(|| {
            Runner {
                target: "fixture-test",
                repo: &directory.0,
                manifest: &package.join("Cargo.toml"),
                files: &[],
                directories: &["toy"],
            }
            .run_with_backend(
                Backend::NativeTests {
                    package: "fixture-probe",
                    test_target: "probe",
                },
                Mode::All,
                &[&CONTRACT],
                &mutations,
            );
        });
        assert!(result.is_err());
        let runs: Vec<_> = fs::read_dir(package.join("target/contract-logs"))
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect();
        assert_eq!(runs.len(), 1);
        let run = &runs[0];
        assert_eq!(
            Summary::parse(&fs::read_to_string(run.join("summary.txt")).unwrap()).status,
            Status::Failed
        );
        assert_eq!(
            fs::read_to_string(run.join("fixtures/failed/target/witness.txt")).unwrap(),
            "counterexample"
        );
        assert!(!run.join("fixtures/failed/old.txt").exists());
        assert!(!run.join("fixtures/accepted").exists());
        let log = fs::read_to_string(run.join("failed.log")).unwrap();
        assert!(log.contains("Mutation fixtures preserved at"));
        let removed = log
            .lines()
            .find_map(|line| line.strip_prefix("FIXTURE="))
            .unwrap();
        assert!(!Path::new(removed).exists());
        assert_eq!(
            fs::read_to_string(package.join("src/lib.rs")).unwrap(),
            original
        );
    }
    #[test]
    #[should_panic(expected = "selected contracts have no mutations")]
    fn rejects_an_empty_mutation_run_before_launching_tools() {
        Runner {
            target: "analyzer",
            repo: Path::new("missing"),
            manifest: Path::new("missing"),
            files: &[],
            directories: &[],
        }
        .run(Mode::Mutation, &[&CONTRACTS[0]], &[]);
    }
}
