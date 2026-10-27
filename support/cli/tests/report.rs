use std::fs;
use veryl_formal_ci::report::{render, replay};
use veryl_formal_runner::summary::{Phase, Status, Summary};

struct Fixture {
    dir: tempfile::TempDir,
    logs: std::path::PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let logs = dir
            .path()
            .join("targets/simulator/tests/target/contract-logs/run-1");
        fs::create_dir_all(&logs).unwrap();
        Self { dir, logs }
    }
    fn summary(&self, status: Status, phase: Phase) -> Summary {
        let summary = Summary {
            target: "simulator".into(),
            manifest: "targets/simulator/tests/Cargo.toml".into(),
            adapter: "check_backend_contracts".into(),
            adapter_kind: "example".into(),
            active_contract: "event-interpreter".into(),
            active_harness: "events::interpreter_event_state".into(),
            status,
            phase: Some(phase),
            positive_passes: Some(1),
            mutation_passes: Some(0),
            active_mutation: if phase == Phase::Mutation {
                "event-interpreter-drop-edge".into()
            } else {
                String::new()
            },
            ..Default::default()
        };
        summary.write(&self.logs.join("summary.txt")).unwrap();
        summary
    }
    fn render(&self, status: &str) -> String {
        render(self.dir.path(), status, None).unwrap()
    }
}

#[test]
fn expected_mutation_counterexamples_do_not_make_a_failed_run() {
    let f = Fixture::new();
    f.summary(Status::Success, Phase::Mutation);
    fs::write(
        f.logs.join("event-interpreter-drop-edge.log"),
        "VERIFICATION:- FAILED\nsemantic counterexample",
    )
    .unwrap();
    let text = f.render("success");
    assert!(text.contains("| success |"));
    assert!(!text.contains("Replay from"));
}

#[test]
fn timeout_identifies_harness_and_positive_replay() {
    let f = Fixture::new();
    f.summary(Status::Failed, Phase::Positive);
    fs::write(f.logs.join("event-interpreter.log"), "CBMC timed out").unwrap();
    let text = f.render("failure");
    for expected in [
        "timeout observed",
        "events::interpreter_event_state",
        "--mode positive --contract event-interpreter",
        "VERYL_AOT_CACHE_DIR=\"$PWD/target/formal-aot-cache\"",
    ] {
        assert!(text.contains(expected));
    }
}

#[test]
fn mutation_failure_replays_original_before_mutations() {
    let f = Fixture::new();
    f.summary(Status::Failed, Phase::Mutation);
    fs::write(
        f.logs.join("event-interpreter-drop-edge.log"),
        "error: could not compile package",
    )
    .unwrap();
    let text = f.render("failure");
    for expected in [
        "build failure observed",
        "Mutation: `event-interpreter-drop-edge`",
        "--mode all --contract event-interpreter",
    ] {
        assert!(text.contains(expected));
    }
}

#[test]
fn interruptions_and_unknown_states_are_not_verified() {
    let f = Fixture::new();
    for state in [Status::Running, Status::Unknown] {
        f.summary(state, Phase::Positive);
        let text = f.render("cancelled");
        assert!(text.contains("unfinished; not verified"));
        assert!(text.contains("No completed log"));
    }
}

#[test]
fn checker_success_does_not_hide_a_failed_job() {
    let f = Fixture::new();
    f.summary(Status::Success, Phase::Positive);
    assert!(
        f.render("failure")
            .contains("Recorded checker runs passed, but the job did not succeed")
    );
}

#[test]
fn setup_failure_has_no_false_success() {
    let f = Fixture::new();
    assert!(
        f.render("failure")
            .contains("absence of checker logs is not verification success")
    );
}

#[test]
fn cached_summaries_and_private_plans_are_excluded() {
    let f = Fixture::new();
    f.summary(Status::Failed, Phase::Positive);
    let text = render(f.dir.path(), "success", Some(("current-run", "1", "job"))).unwrap();
    assert!(text.contains("No checker summaries"));
    assert!(!text.contains("events::interpreter_event_state"));
    let private = f.dir.path().join(".local/target/contract-logs/run-private");
    fs::create_dir_all(&private).unwrap();
    fs::write(private.join("summary.txt"), "status=success\n").unwrap();
    assert!(!f.render("failure").contains("run-private"));
}

#[test]
fn replay_quotes_shell_arguments_and_rejects_unknown_adapters() {
    let f = Fixture::new();
    let mut summary = f.summary(Status::Failed, Phase::Positive);
    summary.manifest = "path with spaces/a'$(touch marker)/Cargo.toml".into();
    let command = replay(&summary).unwrap();
    assert!(command.contains("'path with spaces/a'\"'\"'$(touch marker)/Cargo.toml'"));
    summary.adapter_kind = "unknown".into();
    assert!(replay(&summary).is_none());
}

#[test]
fn logs_cannot_escape_the_selected_run() {
    let f = Fixture::new();
    let mut summary = f.summary(Status::Failed, Phase::Mutation);
    summary.active_mutation = "../../outside".into();
    summary.write(&f.logs.join("summary.txt")).unwrap();
    let text = f.render("failure");
    assert!(!text.contains("- Log:"));
}

#[test]
fn report_writes_an_artifact_and_appends_to_the_job_summary() {
    let f = Fixture::new();
    f.summary(Status::Success, Phase::Positive);
    let step = f.dir.path().join("step.md");
    fs::write(&step, "Earlier step\n").unwrap();
    let env = [
        ("FORMAL_JOB_STATUS".into(), "success".into()),
        ("GITHUB_STEP_SUMMARY".into(), step.display().to_string()),
    ]
    .into_iter()
    .collect();
    veryl_formal_ci::report::write(f.dir.path(), &env).unwrap();
    let artifact = fs::read_to_string(f.dir.path().join("support/target/ci-report.md")).unwrap();
    assert!(artifact.contains("| success |"));
    assert_eq!(
        fs::read_to_string(step).unwrap(),
        format!("Earlier step\n{artifact}")
    );
}
