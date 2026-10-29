//! One dependency resolution for each upstream candidate, shared by every shard
//! and the final commit. Ordinary pinned runs retain their committed lock files.
use crate::{Commands, Result, error, nightly};
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, path::Path};

// Keep the input artifact outside cached build directories.
pub const BUNDLE: &str = "support/ci-inputs/candidate-locks.json";
// Exclude veryl/Cargo.lock: it belongs to the clean upstream revision.
pub const FILES: [&str; 9] = [
    "Cargo.lock",
    "semantics/Cargo.lock",
    "support/Cargo.lock",
    "support/cli/Cargo.lock",
    "targets/analyzer/kani/Cargo.lock",
    "targets/emitter/tests/Cargo.lock",
    "targets/frontend/kani/Cargo.lock",
    "targets/simulator/cbmc/Cargo.lock",
    "targets/simulator/kani/Cargo.lock",
];

fn digest(root: &Path, commands: &mut impl Commands) -> Result<String> {
    // Git's blob hash identifies the complete artifact bytes, including metadata.
    let hash = commands.run(root, "git", &["hash-object", "--no-filters", BUNDLE])?;
    nightly::revision(&hash)?;
    Ok(hash)
}

pub fn create(root: &Path, candidate: &str, commands: &mut impl Commands) -> Result<String> {
    nightly::revision(candidate)?;
    let formal = commands.run(root, "git", &["rev-parse", "HEAD"])?;
    let pinned = commands.run(root, "git", &["rev-parse", "HEAD:veryl"])?;
    nightly::revision(&formal)?;
    nightly::revision(&pinned)?;
    let mut clean = vec!["diff", "--exit-code", "HEAD", "--"];
    clean.extend(FILES);
    commands.run(root, "git", &clean)?;
    nightly::checkout(root, candidate, commands)?;
    if candidate != pinned {
        for path in FILES {
            let manifest = Path::new(path).with_file_name("Cargo.toml");
            // Preserve locked registry versions unless the candidate requires a change.
            commands.run(
                root,
                "cargo",
                &[
                    "update",
                    "--workspace",
                    "--manifest-path",
                    manifest.to_str().unwrap(),
                ],
            )?;
        }
    }
    nightly::verify_clean(root, candidate, commands)?;
    let files: BTreeMap<_, _> = FILES
        .into_iter()
        .map(|path| Ok((path, fs::read_to_string(root.join(path))?)))
        .collect::<Result<_>>()?;
    let bundle = json!({
        "formal": formal,
        "pinned": pinned,
        "candidate": candidate,
        "files": files,
    });
    let path = root.join(BUNDLE);
    fs::create_dir_all(path.parent().unwrap())?;
    fs::write(path, serde_json::to_string_pretty(&bundle)? + "\n")?;
    digest(root, commands)
}

fn load(
    root: &Path,
    candidate: &str,
    expected_digest: &str,
    commands: &mut impl Commands,
) -> Result<BTreeMap<String, String>> {
    nightly::revision(candidate)?;
    nightly::revision(expected_digest)?;
    if digest(root, commands)? != expected_digest {
        return Err(error("candidate lock artifact digest mismatch"));
    }
    let bundle: Value = serde_json::from_slice(&fs::read(root.join(BUNDLE))?)?;
    if bundle.as_object().map(|x| x.len()) != Some(4)
        || bundle["candidate"].as_str() != Some(candidate)
        || bundle["formal"].as_str()
            != Some(commands.run(root, "git", &["rev-parse", "HEAD"])?.as_str())
        || bundle["pinned"].as_str()
            != Some(
                commands
                    .run(root, "git", &["rev-parse", "HEAD:veryl"])?
                    .as_str(),
            )
    {
        return Err(error("candidate lock artifact revision mismatch"));
    }
    let files: BTreeMap<String, String> = serde_json::from_value(bundle["files"].clone())?;
    if files.keys().map(String::as_str).ne(FILES) {
        return Err(error(
            "candidate lock artifact must contain exactly the formal lock files",
        ));
    }
    Ok(files)
}

pub fn apply(
    root: &Path,
    candidate: &str,
    expected_digest: &str,
    commands: &mut impl Commands,
) -> Result<()> {
    nightly::verify_clean(root, candidate, commands)?;
    let files = load(root, candidate, expected_digest, commands)?;
    // Validate the whole artifact before writing any of its files.
    for (path, contents) in files {
        fs::write(root.join(path), contents)?;
    }
    Ok(())
}

pub fn verify(
    root: &Path,
    candidate: &str,
    expected_digest: &str,
    commands: &mut impl Commands,
) -> Result<()> {
    let files = load(root, candidate, expected_digest, commands)?;
    for (path, contents) in files {
        if fs::read(root.join(&path))? != contents.as_bytes() {
            return Err(error(format!("candidate lock file changed: {path}")));
        }
    }
    Ok(())
}
