use crate::{Commands, Environment, Result, checks, error, locks, required};
use serde_json::Value;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};

const UPSTREAM: &str = "https://github.com/veryl-lang/veryl.git";
pub const ISSUE_TITLE: &str = "Upstream verification failed";
pub const ISSUE_BEGIN: &str = "<!-- formal-upstream-check -->";
pub const ISSUE_END: &str = "<!-- /formal-upstream-check -->";
// GitHub aggregates all matrix shards into this result; partial success cannot advance the pointer.
pub const CHECKS: [&str; 1] = ["contracts"];

pub fn revision(value: &str) -> Result<&str> {
    if value.len() != 40
        || !value
            .bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
    {
        return Err(error("expected a full Git commit ID"));
    }
    Ok(value)
}

#[derive(Debug, PartialEq, Eq)]
pub struct Selection {
    pub target: String,
    pub mode: String,
    pub update: bool,
}

pub fn selection(event: &str, inputs: &Value, branch: &str, default: &str) -> Result<Selection> {
    if !inputs.is_null() && !inputs.is_object() {
        return Err(error("invalid workflow inputs"));
    }
    let input = |name| -> Result<&str> {
        match inputs.get(name) {
            None | Some(Value::Null) => Ok("all"),
            Some(Value::String(s)) if s.is_empty() => Ok("all"),
            Some(Value::String(s)) => Ok(s),
            _ => Err(error(format!("invalid {name} input"))),
        }
    };
    let target = input("target")?;
    let mode = input("mode")?;
    let update = event == "schedule"
        || (event == "workflow_dispatch"
            && (inputs["update_upstream"] == true || inputs["update_upstream"] == "true"));
    if ![
        "all",
        "value",
        "simulator",
        "synthesizer",
        "emitter",
        "frontend",
        "cache",
    ]
    .contains(&target)
    {
        return Err(error("unknown verification target"));
    }
    if !["all", "positive"].contains(&mode) {
        return Err(error("unknown verification mode"));
    }
    if update && (target != "all" || mode != "all" || branch != default) {
        return Err(error(
            "upstream updates require all checks and mutations on the default branch",
        ));
    }
    Ok(Selection {
        target: target.into(),
        mode: mode.into(),
        update,
    })
}

pub fn failed_checks(needs: &Value) -> Vec<&'static str> {
    let mut failed: Vec<_> = CHECKS
        .into_iter()
        .filter(|name| needs[*name]["result"] != "success")
        .collect();
    failed.sort_unstable();
    failed
}

pub fn prepare(root: &Path, env: &Environment, commands: &mut impl Commands) -> Result<()> {
    let event: Value =
        serde_json::from_str(&fs::read_to_string(required(env, "GITHUB_EVENT_PATH")?)?)?;
    let selected = selection(
        required(env, "GITHUB_EVENT_NAME")?,
        &event["inputs"],
        required(env, "GITHUB_REF_NAME")?,
        event["repository"]["default_branch"]
            .as_str()
            .ok_or_else(|| error("missing default branch"))?,
    )?;
    let pinned = commands.run(root, "git", &["rev-parse", "HEAD:veryl"])?;
    revision(&pinned)?;
    let candidate = if selected.update {
        let remote = commands.run(root, "git", &["ls-remote", UPSTREAM, "refs/heads/master"])?;
        revision(
            remote
                .split_whitespace()
                .next()
                .ok_or_else(|| error("upstream master was not found"))?,
        )?
        .to_owned()
    } else {
        pinned.clone()
    };
    let formal_sha = commands.run(root, "git", &["rev-parse", "HEAD"])?;
    revision(&formal_sha)?;
    let mut output = OpenOptions::new()
        .create(true)
        .append(true)
        .open(required(env, "GITHUB_OUTPUT")?)?;
    let matrix = checks::matrix(&selected.target)?.to_string();
    for (key, value) in [
        ("target", selected.target),
        ("matrix", matrix),
        ("mode", selected.mode),
        ("update", selected.update.to_string()),
        ("formal_sha", formal_sha),
        ("pinned", pinned),
        ("candidate", candidate),
    ] {
        writeln!(output, "{key}={value}")?;
    }
    Ok(())
}

pub fn verify_clean(root: &Path, candidate: &str, commands: &mut impl Commands) -> Result<()> {
    revision(candidate)?;
    let veryl = root.join("veryl");
    if commands.run(&veryl, "git", &["rev-parse", "HEAD"])? != candidate
        || !commands
            .run(&veryl, "git", &["status", "--porcelain"])?
            .is_empty()
    {
        return Err(error(
            "verification requires the unmodified Veryl candidate",
        ));
    }
    Ok(())
}

pub fn checkout(root: &Path, candidate: &str, commands: &mut impl Commands) -> Result<()> {
    revision(candidate)?;
    let veryl = root.join("veryl");
    if commands.run(&veryl, "git", &["rev-parse", "HEAD"])? != candidate {
        commands.run(
            &veryl,
            "git",
            &["fetch", "--no-tags", "--depth=1", "origin", candidate],
        )?;
        commands.run(&veryl, "git", &["checkout", "--detach", candidate])?;
    }
    verify_clean(root, candidate, commands)
}

pub fn issue_body(
    candidate: &str,
    pinned: &str,
    formal_sha: &str,
    failed: &[&str],
    url: &str,
    lock_digest: &str,
    artifact: &str,
) -> String {
    format!(
        "{ISSUE_BEGIN}\nCandidate Veryl revision: `{candidate}`\n\n\
        Verification revision: `{formal_sha}`.\n\nCommitted Veryl revision remains `{pinned}`.\n\n\
        Failed or incomplete jobs: {}.\n\n[Workflow run and diagnostic artifacts]({url})\n\n\
        Diagnose the reference, proof, build/tool failure, timeout, or implementation locally.\n\
        The workflow does not change proofs or send upstream patches.\n\n\
        Replay from a clone with the same verification revision:\n\n\
        ```sh\ngit checkout --detach {formal_sha}\ngit submodule update --init --recursive\n\
        git -C veryl fetch origin {candidate}\ngit -C veryl checkout --detach {candidate}\n\
        gh run download {} --name {artifact} --dir support/ci-inputs\n\
        cargo run --locked --manifest-path support/cli/Cargo.toml -- apply-locks {candidate} {lock_digest}\n\
        # Run the failed jobs' commands from .github/workflows/verify.yml.\n```\n{ISSUE_END}\n",
        failed.join(", "),
        url.rsplit('/').next().unwrap_or(url),
    )
}

pub fn merge_issue_body(previous: &str, generated: &str) -> String {
    if let Some(start) = previous.find(ISSUE_BEGIN)
        && let Some(end) = previous[start..].find(ISSUE_END).map(|n| start + n)
    {
        return format!(
            "{}{}{}",
            &previous[..start],
            generated.trim_end(),
            &previous[end + ISSUE_END.len()..]
        );
    }
    format!("{}\n\n{generated}", previous.trim_end())
}

pub fn finalize(root: &Path, env: &Environment, commands: &mut impl Commands) -> Result<()> {
    let candidate = revision(required(env, "VERYL_CANDIDATE")?)?;
    let pinned = revision(required(env, "VERYL_PINNED")?)?;
    let formal_sha = revision(required(env, "FORMAL_SHA")?)?;
    let branch = required(env, "DEFAULT_BRANCH")?;
    let repository = required(env, "GITHUB_REPOSITORY")?;
    let needs: Value = serde_json::from_str(required(env, "CHECK_RESULTS")?)?;
    let failed = failed_checks(&needs);
    if !failed.is_empty() {
        let url = format!(
            "{}/{repository}/actions/runs/{}",
            required(env, "GITHUB_SERVER_URL")?,
            required(env, "GITHUB_RUN_ID")?
        );
        let issues: Value = serde_json::from_str(&commands.run(
            root,
            "gh",
            &[
                "issue",
                "list",
                "--repo",
                repository,
                "--state",
                "open",
                "--search",
                &format!("\"{ISSUE_TITLE}\" in:title"),
                "--limit",
                "100",
                "--json",
                "number,title,body",
            ],
        )?)?;
        let issues = issues
            .as_array()
            .ok_or_else(|| error("invalid issue list"))?;
        let existing = issues.iter().find(|item| item["title"] == ISSUE_TITLE);
        let generated = issue_body(
            candidate,
            pinned,
            formal_sha,
            &failed,
            &url,
            revision(required(env, "FORMAL_LOCK_DIGEST")?)?,
            required(env, "FORMAL_LOCK_ARTIFACT")?,
        );
        let body = match existing {
            Some(item) => merge_issue_body(
                item["body"]
                    .as_str()
                    .ok_or_else(|| error("invalid issue body"))?,
                &generated,
            ),
            None => generated,
        };
        let temporary = tempfile::tempdir()?;
        let path = temporary.path().join("issue.md");
        fs::write(&path, body)?;
        let path = path
            .to_str()
            .ok_or_else(|| error("issue path is not UTF-8"))?;
        if let Some(existing) = existing {
            let number = existing["number"]
                .as_u64()
                .ok_or_else(|| error("invalid issue number"))?
                .to_string();
            commands.run(
                root,
                "gh",
                &[
                    "issue",
                    "edit",
                    &number,
                    "--repo",
                    repository,
                    "--body-file",
                    path,
                ],
            )?;
        } else {
            commands.run(
                root,
                "gh",
                &[
                    "issue",
                    "create",
                    "--title",
                    ISSUE_TITLE,
                    "--repo",
                    repository,
                    "--body-file",
                    path,
                ],
            )?;
        }
        return Ok(());
    }
    if candidate == pinned {
        println!("The committed pointer already matches the verified upstream candidate.");
        return Ok(());
    }
    commands.run(
        root,
        "git",
        &[
            "fetch",
            "--no-tags",
            "origin",
            &format!("refs/heads/{branch}"),
        ],
    )?;
    let latest = commands.run(root, "git", &["rev-parse", "FETCH_HEAD"])?;
    if revision(&latest)? != formal_sha {
        println!("The verification branch advanced during the run; leaving its pointer unchanged.");
        return Ok(());
    }
    if commands.run(root, "git", &["rev-parse", "HEAD"])? != formal_sha {
        return Err(error(
            "finalizer checkout differs from the verified formal revision",
        ));
    }
    commands.run(root, "git", &["diff", "--cached", "--exit-code"])?;
    checkout(root, candidate, commands)?;
    let digest = required(env, "FORMAL_LOCK_DIGEST")?;
    locks::apply(root, candidate, digest, commands)?;
    let mut paths = vec!["add", "veryl"];
    paths.extend(locks::FILES);
    commands.run(root, "git", &paths)?;
    let changed = commands.run(root, "git", &["diff", "--cached", "--name-only"])?;
    if !changed.lines().any(|path| path == "veryl")
        || changed
            .lines()
            .any(|path| path != "veryl" && !locks::FILES.contains(&path))
    {
        return Err(error(
            "the update may contain only the verified submodule pointer and lock files",
        ));
    }
    commands.run(root, "git", &["config", "user.name", "github-actions[bot]"])?;
    commands.run(
        root,
        "git",
        &[
            "config",
            "user.email",
            "41898282+github-actions[bot]@users.noreply.github.com",
        ],
    )?;
    commands.run(
        root,
        "git",
        &[
            "-c",
            "commit.gpgsign=false",
            "commit",
            "-m",
            &format!("Update Veryl to {}", &candidate[..12]),
        ],
    )?;
    // An ordinary fast-forward push rejects a concurrent branch update.
    commands.run(
        root,
        "git",
        &["push", "origin", &format!("HEAD:refs/heads/{branch}")],
    )?;
    Ok(())
}
