//! An explicitly built VERYL_BIN is required; this test is intentionally opt-in.
//! A local four-file dependency fixture compares warm and fresh cold output after
//! dependency/options/lock changes and blob corruption. Warning diagnostics are
//! compared separately from informational progress. No arbitrary-project guarantee.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    time::{SystemTime, UNIX_EPOCH},
};
fn binary() -> PathBuf {
    std::env::var_os("VERYL_BIN")
        .map(PathBuf::from)
        .expect("set VERYL_BIN to a built veryl executable")
}
fn run(root: &Path) -> Output {
    let out = Command::new(binary())
        .current_dir(root)
        .args(["build"])
        .env("NO_COLOR", "1")
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "CLI fixture build failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    out
}
fn outputs(root: &Path) -> BTreeMap<String, Vec<u8>> {
    fs::read_dir(root.join("out"))
        .unwrap()
        .map(|x| x.unwrap().path())
        .filter(|p| p.extension().is_some_and(|s| s == "sv"))
        .map(|p| {
            (
                p.file_name().unwrap().to_str().unwrap().to_owned(),
                fs::read(p).unwrap(),
            )
        })
        .collect()
}
fn compare(root: &Path) {
    let check = |root: &Path| {
        let out = Command::new(binary())
            .current_dir(root)
            .args(["check", "--quiet"])
            .env("NO_COLOR", "1")
            .output()
            .unwrap();
        assert!(
            !out.status.success(),
            "warning fixture must make check fail"
        );
        assert!(
            String::from_utf8_lossy(&out.stderr).contains("unused"),
            "expected warning: {out:?}"
        );
        out
    };
    let warm = run(root);
    let diagnostic = check(root);
    let actual = outputs(root);
    assert_eq!(actual.len(), 4, "fixture output count");
    let cache = root.join(".build/cache");
    assert!(cache.exists(), "fixture must exercise cache");
    fs::remove_dir_all(cache).unwrap();
    let cold = run(root);
    let cold_diagnostic = check(root);
    assert_eq!(
        diagnostic.stderr, cold_diagnostic.stderr,
        "cache CLI output mismatch: warning replay"
    );
    assert_eq!(
        actual,
        outputs(root),
        "cache CLI output mismatch: generated SV"
    );
    let diagnostics = |out: &Output| {
        String::from_utf8_lossy(&out.stderr)
            .lines()
            .filter(|line| !line.starts_with("[INFO ]"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    assert_eq!(
        diagnostics(&warm),
        diagnostics(&cold),
        "cache CLI output mismatch: diagnostics"
    );
    assert_eq!(
        warm.stdout, cold.stdout,
        "cache CLI output mismatch: stdout"
    );
}
#[test]
#[ignore = "requires a built veryl executable via VERYL_BIN"]
fn warm_cli_matches_cold_after_dependency_options_lock_and_corruption() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/fixtures")
        .join(format!(
            "cli-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
    fs::create_dir_all(root.join("src")).unwrap();
    let config = r#"[project]
name="cache_contract"
version="0.1.0"
[build]
sources=["src"]
target={type="directory",path="out"}
sourcemap_target={type="none"}
exclude_std=true
incremental=true
"#;
    fs::write(root.join("Veryl.toml"), config).unwrap();
    fs::write(root.join("src/a.veryl"), "package A { const WIDTH:u32=4; }").unwrap();
    fs::write(
        root.join("src/b.veryl"),
        "package B { const WIDTH:u32=A::WIDTH+1; }",
    )
    .unwrap();
    fs::write(
        root.join("src/c.veryl"),
        "module Top(o:output logic<B::WIDTH>){assign o=0;}",
    )
    .unwrap();
    fs::write(
        root.join("src/w.veryl"),
        "module Warn {let abandoned:logic=1;}",
    )
    .unwrap();
    run(&root);
    let restored = run(&root);
    assert!(
        String::from_utf8_lossy(&restored.stderr).contains("Restored 4/4 files"),
        "fixture must restore all fragments: {restored:?}"
    );
    compare(&root);
    for width in [1, 8, 17] {
        fs::write(
            root.join("src/a.veryl"),
            format!("package A {{ const WIDTH:u32={width}; }}"),
        )
        .unwrap();
        compare(&root);
    }
    fs::write(
        root.join("Veryl.toml"),
        format!("{config}strip_comments=true\n"),
    )
    .unwrap();
    compare(&root);
    let lock = root.join("Veryl.lock");
    let mut contents = fs::read_to_string(&lock).unwrap();
    contents.push_str("\n# changed lockfile content\n");
    fs::write(&lock, contents).unwrap();
    compare(&root);
    let mut changed = 0;
    for dir in fs::read_dir(root.join(".build/cache/fragments")).unwrap() {
        for file in fs::read_dir(dir.unwrap().path()).unwrap() {
            let path = file.unwrap().path();
            let mut bytes = fs::read(&path).unwrap();
            let last = bytes.len() - 1;
            bytes[last] ^= 1;
            fs::write(path, bytes).unwrap();
            changed += 1;
        }
    }
    assert!(changed > 0, "fixture needs stored fragments");
    compare(&root);
}
