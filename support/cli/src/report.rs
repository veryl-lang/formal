use crate::{Environment, Result};
use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Component, Path, PathBuf},
};
use veryl_formal_runner::summary::{Phase, Status, Summary};

fn directories(path: &Path) -> io::Result<Vec<PathBuf>> {
    match fs::read_dir(path) {
        Ok(entries) => entries
            .filter_map(|entry| match entry {
                Ok(entry) if entry.path().is_dir() => Some(Ok(entry.path())),
                Ok(_) => None,
                Err(error) => Some(Err(error)),
            })
            .collect(),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(error) => Err(error),
    }
}

fn summaries(root: &Path) -> io::Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    // Search only maintained adapters, never private plans or mutation build trees.
    for target in directories(&root.join("targets"))? {
        for adapter in directories(&target)? {
            for run in directories(&adapter.join("target/contract-logs"))? {
                if run
                    .file_name()
                    .is_some_and(|n| n.to_string_lossy().starts_with("run-"))
                {
                    let path = run.join("summary.txt");
                    if path.is_file() {
                        paths.push(path);
                    }
                }
            }
        }
    }
    paths.sort();
    Ok(paths)
}

fn quote(value: &str) -> String {
    if !value.is_empty()
        && value
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"_@%+=:,./-".contains(&c))
    {
        value.into()
    } else {
        format!("'{}'", value.replace('\'', "'\"'\"'"))
    }
}

pub fn replay(summary: &Summary) -> Option<String> {
    if summary.manifest.is_empty()
        || summary.adapter.is_empty()
        || summary.active_contract.is_empty()
        || !["example", "bin"].contains(&summary.adapter_kind.as_str())
    {
        return None;
    }
    let kind = format!("--{}", summary.adapter_kind);
    let mode = if summary.phase == Some(Phase::Mutation) {
        "all"
    } else {
        "positive"
    };
    let command = [
        "cargo",
        "run",
        "--locked",
        "--manifest-path",
        &summary.manifest,
        &kind,
        &summary.adapter,
        "--",
        "--mode",
        mode,
        "--contract",
        &summary.active_contract,
    ]
    .into_iter()
    .map(quote)
    .collect::<Vec<_>>()
    .join(" ");
    Some(if summary.target == "simulator" {
        format!("VERYL_AOT_CACHE_DIR=\"$PWD/target/formal-aot-cache\" {command}")
    } else {
        command
    })
}

fn cell(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', " ")
}

pub fn render(
    root: &Path,
    job_status: &str,
    identity: Option<(&str, &str, &str)>,
) -> Result<String> {
    let mut entries = Vec::new();
    for path in summaries(root)? {
        let summary = Summary::parse(&fs::read_to_string(&path)?);
        if identity.is_some_and(|(run, attempt, job)| {
            (
                summary.ci_run_id.as_str(),
                summary.ci_run_attempt.as_str(),
                summary.ci_job.as_str(),
            ) != (run, attempt, job)
        }) {
            continue;
        }
        entries.push((path, summary));
    }
    let mut out = format!(
        "## Nightly semantic contracts\n\nWorkflow job status: **{}**.\n\n\
        Maintainer triage is required for failures. A failed checker does not by itself\n\
        identify a production bug: inspect the reference/harness, tool/build failures,\n\
        timeouts, and the counterexample before requesting a source fix.\n\n",
        cell(job_status)
    );
    if entries.is_empty() {
        out.push_str("No checker summaries were produced. Inspect setup/build/test steps;\nabsence of checker logs is not verification success.\n");
    } else {
        out.push_str(
            "| Run | State | Positive passes | Mutation passes |\n| --- | --- | ---: | ---: |\n",
        );
        for (path, summary) in &entries {
            let label = match summary.status {
                Status::Success => "success",
                Status::Failed => "failed",
                _ => "unfinished; not verified",
            };
            let count = |n: Option<usize>| n.map_or_else(|| "?".into(), |n| n.to_string());
            out.push_str(&format!(
                "| `{}` | {label} | {} | {} |\n",
                cell(&path.parent().unwrap().strip_prefix(root)?.to_string_lossy()),
                count(summary.positive_passes),
                count(summary.mutation_passes)
            ));
        }
        let pending: Vec<_> = entries
            .iter()
            .filter(|(_, s)| s.status != Status::Success)
            .collect();
        if job_status != "success" && pending.is_empty() {
            out.push_str("\nRecorded checker runs passed, but the job did not succeed.\nInspect other workflow steps; these summaries do not cover every build/test step.\n");
        }
        for (path, summary) in pending {
            let fallback = |s: &str| {
                if s.is_empty() {
                    "?".to_owned()
                } else {
                    cell(s)
                }
            };
            out.push_str(&format!(
                "\n### {}: {}\n\n- Stage: `{}`\n- Harness/test: `{}`\n",
                fallback(&summary.target),
                fallback(&summary.active_contract),
                summary.phase.map_or("?", Phase::as_str),
                fallback(&summary.active_harness)
            ));
            if !summary.exit_code.is_empty() {
                out.push_str(&format!("- Exit code: `{}`\n", cell(&summary.exit_code)));
            }
            if !summary.active_mutation.is_empty() {
                out.push_str(&format!(
                    "- Mutation: `{}`\n",
                    cell(&summary.active_mutation)
                ));
            }
            let name = if summary.active_mutation.is_empty() {
                &summary.active_contract
            } else {
                &summary.active_mutation
            };
            let mut components = Path::new(name).components();
            if matches!(components.next(), Some(Component::Normal(_)))
                && components.next().is_none()
            {
                let log = path.parent().unwrap().join(format!("{name}.log"));
                if log.is_file() {
                    let text = String::from_utf8_lossy(&fs::read(&log)?).to_lowercase();
                    let hint = if text.contains("timed out") {
                        "timeout observed"
                    } else if text.contains("could not compile") {
                        "build failure observed"
                    } else {
                        "inspect checker output"
                    };
                    out.push_str(&format!(
                        "- Log: `{}` ({hint})\n",
                        cell(&log.strip_prefix(root)?.to_string_lossy())
                    ));
                } else {
                    out.push_str("- No completed log for the active check; execution may have been interrupted.\n");
                }
            }
            if let Some(command) = replay(summary) {
                out.push_str(&format!("\nReplay from the repository root with the same tool versions:\n\n```sh\n{command}\n```\n"));
                if summary.phase == Some(Phase::Mutation) {
                    out.push_str("This first checks the original, then regenerates this contract's mutations.\n");
                }
            }
        }
    }
    out.push_str(
        "\nDownload the job's `formal-*` artifact for complete checker logs and metadata.\n\
        Failure fixtures, when present, are saved in a separate `formal-fixtures-*` artifact.\n\
        After confirming a production bug, reduce it to a source/API example and add a fast\n\
        ordinary regression. Proof debugging remains with verification maintainers.\n",
    );
    Ok(out)
}

pub fn write(root: &Path, env: &Environment) -> Result<()> {
    let get = |key| env.get(key).map_or("", String::as_str);
    let identity = (!get("GITHUB_RUN_ID").is_empty()).then(|| {
        (
            get("GITHUB_RUN_ID"),
            get("GITHUB_RUN_ATTEMPT"),
            get("GITHUB_JOB"),
        )
    });
    let report = render(
        root,
        env.get("FORMAL_JOB_STATUS")
            .map_or("unknown", String::as_str),
        identity,
    )?;
    let output = root.join("support/target/ci-report.md");
    fs::create_dir_all(output.parent().unwrap())?;
    fs::write(output, &report)?;
    if !get("GITHUB_STEP_SUMMARY").is_empty() {
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(get("GITHUB_STEP_SUMMARY"))?
            .write_all(report.as_bytes())?;
    }
    print!("{report}");
    Ok(())
}
