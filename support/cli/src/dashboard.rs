//! Catalog, provenance, and result export for the static verification dashboard.
use crate::{Commands, Environment, Process, Result, catalog, error, locks, report};
use serde_json::{Value, json};
use std::{fs, path::Path, process::Command};
use veryl_formal_runner::summary::Summary;

fn source_dirty(changes: &str) -> bool {
    changes.lines().any(|line| {
        let path = line
            .trim_start()
            .split_once(' ')
            .map(|(_, path)| path.trim_start());
        !path.is_some_and(|path| locks::FILES.contains(&path))
    })
}

pub fn identity(root: &Path, env: &Environment) -> Result<Value> {
    let mut git = Process;
    let mut lock_hashes = serde_json::Map::new();
    for path in locks::FILES {
        lock_hashes.insert(
            path.into(),
            json!(git.run(root, "git", &["hash-object", "--no-filters", path])?),
        );
    }
    let changes = git.run(
        root,
        "git",
        &[
            "status",
            "--porcelain",
            "--untracked-files=all",
            "--",
            "Cargo.toml",
            ".cargo",
            "targets",
            "semantics",
            "support",
        ],
    )?;
    // Nightly deliberately replaces dependency locks. Their bytes are identified separately.
    let dirty = source_dirty(&changes);
    let version = |program: &str, args: &[&str]| -> String {
        Command::new(program)
            .args(args)
            .current_dir(root)
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| {
                String::from_utf8_lossy(&o.stdout)
                    .lines()
                    .next()
                    .unwrap_or("")
                    .into()
            })
            .unwrap_or_else(|| "unavailable".into())
    };
    let cbmc = env.get("CBMC").cloned().unwrap_or_else(|| {
        env.get("HOME")
            .map(|home| {
                Path::new(home).join(format!(
                    ".kani/kani-{}/bin/cbmc",
                    crate::checks::KANI_VERSION
                ))
            })
            .filter(|p| p.is_file())
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "cbmc".into())
    });
    Ok(json!({
        "schema":1,
        "workflowResult": env.get("FORMAL_CONTRACTS_RESULT"),
        "formal":git.run(root,"git", &["rev-parse","HEAD"] )?,
        "upstream":git.run(root,"git", &["-C","veryl","rev-parse","HEAD"] )?,
        "dirty":dirty,
        "upstreamDirty":!git.run(root,"git", &["-C","veryl","status","--porcelain","--untracked-files=no"] )?.is_empty(),
        "locks":lock_hashes,
        "tools":{"rustc":version("rustc", &["--version"]),"cargo":version("cargo", &["--version"]),"kani":version("cargo", &["kani", "--version"]),"cbmc":version(&cbmc, &["--version"]),"cc":version("cc", &["--version"])},
        "ci":{"run":env.get("GITHUB_RUN_ID"),"attempt":env.get("GITHUB_RUN_ATTEMPT"),"job":env.get("GITHUB_JOB"),"check":env.get("FORMAL_CHECK"),"suite":env.get("FORMAL_SUITE")},
    }))
}

pub fn export(root: &Path, env: &Environment) -> Result<()> {
    let mut runs = Vec::new();
    for path in report::summaries(root)? {
        let s = Summary::parse(&fs::read_to_string(&path)?);
        if env
            .get("GITHUB_RUN_ID")
            .is_some_and(|id| &s.ci_run_id != id)
            || env
                .get("GITHUB_RUN_ATTEMPT")
                .is_some_and(|attempt| &s.ci_run_attempt != attempt)
            || env.get("GITHUB_JOB").is_some_and(|job| &s.ci_job != job)
        {
            continue;
        }
        let dir = path.parent().unwrap();
        if !dir.join("identity.json").is_file() || !dir.join("results.tsv").is_file() {
            continue;
        }
        let identity: Value = serde_json::from_slice(&fs::read(dir.join("identity.json"))?)?;
        let records = fs::read_to_string(dir.join("results.tsv"))?;
        runs.push(
            json!({"identity":identity,"summary":s.encode(),"records":records,
            "path":dir.strip_prefix(root)?.to_string_lossy()}),
        );
    }
    let out = root.join("support/target/dashboard-evidence.json");
    fs::create_dir_all(out.parent().unwrap())?;
    fs::write(
        out,
        serde_json::to_vec_pretty(
            &json!({"schema":1,"jobStatus":env.get("FORMAL_JOB_STATUS"),"runs":runs}),
        )?,
    )?;
    Ok(())
}

pub fn build(root: &Path, env: &Environment, output: &Path, evidence: Option<&Path>) -> Result<()> {
    let output = if output.is_absolute() {
        output.to_owned()
    } else {
        root.join(output)
    };
    fs::create_dir_all(&output)?;
    fs::write(
        output.join("catalog.json"),
        serde_json::to_vec_pretty(&catalog::entries())?,
    )?;
    fs::write(
        output.join("identity.json"),
        serde_json::to_vec_pretty(&identity(root, env)?)?,
    )?;
    let mut command = Command::new("python3");
    command
        .arg(root.join("support/dashboard/build.py"))
        .arg("--root")
        .arg(root)
        .arg("--output")
        .arg(&output)
        .current_dir(root);
    if let Some(evidence) = evidence {
        command.arg("--evidence").arg(evidence);
    }
    if let Some(run) = env.get("GITHUB_RUN_ID") {
        command.arg("--run").arg(run);
    }
    if let Some(attempt) = env.get("GITHUB_RUN_ATTEMPT") {
        command.arg("--attempt").arg(attempt);
    }
    let status = command.status()?;
    if !status.success() {
        return Err(error(format!("dashboard generation failed: {status}")));
    }
    println!("Dashboard: {}", output.join("index.html").display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_known_lock_changes_are_identified_by_separate_hashes() {
        assert!(!source_dirty(""));
        assert!(!source_dirty("M Cargo.lock\n M support/cli/Cargo.lock"));
        for change in [
            " M targets/other/Cargo.lock",
            "?? targets/newCargo.lock",
            " M support/src/lib.rs",
            "?? .cargo/config.toml",
        ] {
            assert!(source_dirty(change));
        }
    }
}
