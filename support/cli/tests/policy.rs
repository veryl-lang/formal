use serde_json::{Value, json};
use std::{collections::VecDeque, fs, path::Path};
use veryl_formal_ci::{Commands, Environment, Result, nightly::*};

#[derive(Default)]
struct Scripted {
    replies: VecDeque<String>,
    calls: Vec<Vec<String>>,
    issue_bodies: Vec<String>,
}

impl Commands for Scripted {
    fn run(&mut self, _: &Path, program: &str, args: &[&str]) -> Result<String> {
        self.calls.push(
            std::iter::once(program)
                .chain(args.iter().copied())
                .map(str::to_owned)
                .collect(),
        );
        if program == "gh" && args.first() == Some(&"issue") && args.get(1) != Some(&"list") {
            let body = args[args.iter().position(|arg| *arg == "--body-file").unwrap() + 1];
            self.issue_bodies.push(fs::read_to_string(body)?);
        }
        Ok(self.replies.pop_front().expect("unexpected command"))
    }
}

fn sha(c: char) -> String {
    c.to_string().repeat(40)
}

fn needs() -> Value {
    Value::Object(
        CHECKS
            .into_iter()
            .map(|name| (name.into(), json!({"result":"success"})))
            .collect(),
    )
}

fn environment() -> Environment {
    [
        ("VERYL_CANDIDATE", sha('a')),
        ("VERYL_PINNED", sha('b')),
        ("FORMAL_SHA", sha('c')),
        ("DEFAULT_BRANCH", "master".into()),
        ("GITHUB_REPOSITORY", "veryl-lang/formal".into()),
        ("GITHUB_SERVER_URL", "https://github.com".into()),
        ("GITHUB_RUN_ID", "123".into()),
        ("FORMAL_LOCK_ARTIFACT", "formal-candidate-locks-1".into()),
        ("FORMAL_LOCK_DIGEST", sha('d')),
        ("CHECK_RESULTS", needs().to_string()),
    ]
    .into_iter()
    .map(|(k, v)| (k.into(), v))
    .collect()
}

#[test]
fn partial_positive_and_nondefault_runs_cannot_update() {
    for inputs in [
        json!({"target":"value","update_upstream":true}),
        json!({"mode":"positive","update_upstream":true}),
    ] {
        assert!(selection("workflow_dispatch", &inputs, "master", "master").is_err());
    }
    assert!(
        selection(
            "workflow_dispatch",
            &json!({"update_upstream":true}),
            "topic",
            "master"
        )
        .is_err()
    );
    assert!(selection("schedule", &Value::Null, "topic", "master").is_err());
    for inputs in [
        json!({"target":"unknown"}),
        json!({"mode":"mutation"}),
        json!({"target":123}),
        json!({"mode":true}),
        json!([]),
    ] {
        assert!(selection("workflow_dispatch", &inputs, "master", "master").is_err());
    }
}

#[test]
fn only_schedule_or_explicit_full_dispatch_updates() {
    assert!(
        selection("schedule", &Value::Null, "master", "master")
            .unwrap()
            .update
    );
    assert!(
        !selection("push", &Value::Null, "master", "master")
            .unwrap()
            .update
    );
    let manual = selection(
        "workflow_dispatch",
        &json!({"target":"cache"}),
        "master",
        "master",
    )
    .unwrap();
    assert_eq!(manual.target, "cache");
    assert!(!manual.update);
    for value in [json!(true), json!("true")] {
        assert!(
            selection(
                "workflow_dispatch",
                &json!({"update_upstream":value}),
                "master",
                "master"
            )
            .unwrap()
            .update
        );
    }
}

#[test]
fn missing_skipped_cancelled_and_failed_checks_never_certify() {
    assert!(failed_checks(&needs()).is_empty());
    for result in ["failure", "skipped", "cancelled", ""] {
        let mut input = needs();
        input["contracts"]["result"] = json!(result);
        assert_eq!(failed_checks(&input), ["contracts"]);
    }
    let mut input = needs();
    input.as_object_mut().unwrap().remove("contracts");
    assert_eq!(failed_checks(&input), ["contracts"]);
}

#[test]
fn revisions_are_full_lowercase_commit_ids() {
    assert!(revision(&sha('a')).is_ok());
    for invalid in ["master", "abc123", "--help", &sha('G'), &(sha('a') + "\n")] {
        assert!(revision(invalid).is_err());
    }
}

#[test]
fn dirty_or_changed_revision_is_rejected_without_checkout() {
    for replies in [vec![sha('b')], vec![sha('a'), " M generated.rs".into()]] {
        let mut commands = Scripted {
            replies: replies.into(),
            ..Default::default()
        };
        assert!(verify_clean(Path::new("repo"), &sha('a'), &mut commands).is_err());
        assert!(commands.calls.iter().all(|call| call[1] != "checkout"));
    }
}

#[test]
fn replay_and_repeated_issues_preserve_maintainer_notes() {
    let old = issue_body(
        &sha('a'),
        &sha('b'),
        &sha('c'),
        &["value-contracts"],
        "old-run",
        &sha('e'),
        "formal-candidate-locks-1",
    );
    let new = issue_body(
        &sha('d'),
        &sha('b'),
        &sha('c'),
        &["cache-contracts"],
        "new-run",
        &sha('f'),
        "formal-candidate-locks-2",
    );
    let merged = merge_issue_body(&format!("Diagnosis\n\n{old}\nLocal notes\n"), &new);
    assert!(merged.starts_with("Diagnosis"));
    assert!(merged.ends_with("Local notes\n"));
    assert!(!merged.contains("old-run"));
    assert!(merged.contains("new-run"));
    assert_eq!(merged.matches(ISSUE_BEGIN).count(), 1);
    assert!(merged.contains("--name formal-candidate-locks-2 --dir support/ci-inputs"));
    assert!(merged.contains(&format!("apply-locks {} {}", sha('d'), sha('f'))));
    assert!(merged.contains("git submodule update --init --recursive"));
}

#[test]
fn failed_checks_only_create_or_update_an_issue() {
    for existing in [
        json!([]),
        json!([{"number":7,"title":ISSUE_TITLE,"body":"Maintainer notes"}]),
    ] {
        let mut env = environment();
        env.insert("CHECK_RESULTS".into(), "{}".into());
        let mut commands = Scripted {
            replies: vec![existing.to_string(), String::new()].into(),
            ..Default::default()
        };
        finalize(Path::new("repo"), &env, &mut commands).unwrap();
        assert!(commands.calls.iter().all(|call| call[0] == "gh"));
        assert_eq!(
            commands.calls[1][2],
            if existing.as_array().unwrap().is_empty() {
                "create"
            } else {
                "edit"
            }
        );
        assert_eq!(commands.issue_bodies.len(), 1);
        let body = &commands.issue_bodies[0];
        assert!(body.contains("contracts"));
        assert!(body.contains("Committed Veryl revision remains"));
        if !existing.as_array().unwrap().is_empty() {
            assert!(body.contains("Maintainer notes"));
        }
    }
}

#[test]
fn unchanged_pointer_or_advanced_branch_creates_no_commit() {
    let mut env = environment();
    env.insert("VERYL_PINNED".into(), sha('a'));
    let mut commands = Scripted::default();
    finalize(Path::new("repo"), &env, &mut commands).unwrap();
    assert!(commands.calls.is_empty());
    let mut commands = Scripted {
        replies: vec![String::new(), sha('d')].into(),
        ..Default::default()
    };
    finalize(Path::new("repo"), &environment(), &mut commands).unwrap();
    assert_eq!(commands.calls.len(), 2);
    assert_eq!(commands.calls[0][1], "fetch");
}

#[test]
fn prepare_resolves_one_candidate_and_exports_the_verified_pair() {
    let dir = tempfile::tempdir().unwrap();
    let event = dir.path().join("event.json");
    let output = dir.path().join("output");
    fs::write(&event, r#"{"repository":{"default_branch":"master"}}"#).unwrap();
    let env: Environment = [
        ("GITHUB_EVENT_PATH", event.display().to_string()),
        ("GITHUB_OUTPUT", output.display().to_string()),
        ("GITHUB_EVENT_NAME", "schedule".into()),
        ("GITHUB_REF_NAME", "master".into()),
    ]
    .into_iter()
    .map(|(k, v)| (k.into(), v))
    .collect();
    let mut commands = Scripted {
        replies: vec![
            sha('b'),
            format!("{}\trefs/heads/master", sha('a')),
            sha('c'),
        ]
        .into(),
        ..Default::default()
    };
    prepare(dir.path(), &env, &mut commands).unwrap();
    assert_eq!(
        commands
            .calls
            .iter()
            .filter(|call| call[1] == "ls-remote")
            .count(),
        1
    );
    let text = fs::read_to_string(output).unwrap();
    for expected in [
        format!("candidate={}", sha('a')),
        format!("pinned={}", sha('b')),
        format!("formal_sha={}", sha('c')),
        "update=true".into(),
    ] {
        assert!(text.lines().any(|line| line == expected));
    }
}

#[test]
fn checkout_mismatch_cannot_be_published() {
    let mut commands = Scripted {
        replies: vec![String::new(), sha('c'), sha('d')].into(),
        ..Default::default()
    };
    assert!(finalize(Path::new("repo"), &environment(), &mut commands).is_err());
    assert!(
        !commands
            .calls
            .iter()
            .any(|call| call.iter().any(|arg| arg == "commit" || arg == "push"))
    );
}

#[test]
fn workflow_requires_all_checks_and_post_verification_integrity() {
    let workflow = include_str!("../../../.github/workflows/verify.yml");
    let jobs = workflow
        .split_once("\njobs:\n")
        .unwrap()
        .1
        .split_once("\nconcurrency:")
        .unwrap()
        .0;
    let names: std::collections::BTreeSet<_> = jobs
        .lines()
        .filter_map(|line| {
            line.strip_prefix("  ")
                .filter(|s| !s.starts_with(' '))
                .and_then(|s| s.strip_suffix(':'))
        })
        .filter(|name| !matches!(*name, "prepare" | "update-upstream"))
        .collect();
    assert_eq!(names, CHECKS.into_iter().collect());
    assert!(workflow.contains("matrix: ${{ fromJSON(needs.prepare.outputs.matrix) }}"));
    assert!(workflow.contains("fail-fast: false"));
    assert!(workflow.contains("needs.prepare.outputs.update == 'true'"));
    assert!(workflow.contains("timeout-minutes: ${{ matrix.timeout }}"));
    assert!(workflow.contains("--bin formal-ci -- run"));
    assert!(workflow.contains("--bin formal-ci -- report"));
    assert!(workflow.contains("if: always() && steps.ci-cli.outcome == 'success'"));
    assert!(workflow.contains("if: failure()"));
    assert!(workflow.contains("--bin formal-ci -- finalize"));
    assert!(!workflow.contains("python3"));
    assert_eq!(
        workflow
            .matches("uses: actions/download-artifact@v4")
            .count(),
        2
    );
    assert!(workflow.contains("-- prepare-locks"));
    assert!(workflow.contains("-- apply-locks"));
    assert_eq!(
        workflow
            .matches("name: ${{ needs.prepare.outputs.lock_artifact }}")
            .count(),
        2
    );
    assert!(workflow.contains("lock_artifact: formal-candidate-locks-${{ github.run_attempt }}"));
    assert!(workflow.contains("FORMAL_LOCK_DIGEST: ${{ needs.prepare.outputs.lock_digest }}"));
    assert!(workflow.contains("if: needs.contracts.result == 'success'"));
    assert!(!workflow.contains("workflow_call"));
    assert!(!workflow.contains("cargo test"));
}
