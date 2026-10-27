//! Finite production store comparisons: individual blob-bit corruption and repair,
//! four-node forward DAG metadata persistence against independent closure, and
//! key-part boundaries. No hash collision, concurrent/crash-consistency, general
//! dependency-discovery, or all-caller correctness theorem.

use std::{
    fs,
    path::PathBuf,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use veryl_cache::{Store, content_hash, global_key};
fn directory() -> PathBuf {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/fixtures")
        .join(format!(
            "{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
    fs::create_dir_all(&path).unwrap();
    path
}
#[test]
fn corrupted_blobs_miss_and_are_repaired() {
    let mut checked = 0;
    for length in [0usize, 1, 7, 8, 9, 64, 65, 129] {
        let path = directory();
        let payload: Vec<_> = (0..length).map(|i| (i * 37 + length) as u8).collect();
        let mut store = Store::open(&path, "key");
        store.put("file".into(), "source-hash".into(), Some(&payload));
        store.set_diagnostics("file", b"cached-warning");
        store.save();
        let entry = store.entry("file").unwrap().clone();
        assert_eq!(
            store.load(&entry),
            Some(payload.clone()),
            "cache integrity output mismatch: original"
        );
        for rel in [
            entry.fragment.clone().unwrap(),
            entry.diagnostics.clone().unwrap(),
        ] {
            let file = path.join(&rel);
            let original = fs::read(&file).unwrap();
            let digest = content_hash(&original);
            for bit in 0..original.len() * 8 {
                let mut changed = original.clone();
                changed[bit / 8] ^= 1 << (bit % 8);
                assert_ne!(
                    content_hash(&changed),
                    digest,
                    "known corruption unexpectedly collided"
                );
                fs::write(&file, changed).unwrap();
                let got = if Some(&rel) == entry.fragment.as_ref() {
                    store.load(&entry)
                } else {
                    store.load_diagnostics(&entry)
                };
                assert!(
                    got.is_none(),
                    "cache integrity output mismatch: corrupt byte length={length} bit={bit}"
                );
                checked += 1;
                fs::write(&file, &original).unwrap();
            }
        }
        let fragment = path.join(entry.fragment.unwrap());
        let mut bad = fs::read(&fragment).unwrap();
        let last = bad.len() - 1;
        bad[last] ^= 1;
        fs::write(&fragment, bad).unwrap();
        store.put("file".into(), "source-hash".into(), Some(&payload));
        store.set_diagnostics("file", b"cached-warning");
        store.save();
        assert_eq!(
            store.load(store.entry("file").unwrap()),
            Some(payload),
            "cache integrity output mismatch: repair"
        );
    }
    eprintln!("cache integrity complete corruptions={checked}");
}
fn dependents(mask: usize, node: usize) -> Vec<String> {
    let mut matrix = [[false; 4]; 4];
    let mut bit = 0;
    for (a, row) in matrix.iter_mut().enumerate() {
        for cell in row.iter_mut().skip(a + 1) {
            *cell = mask & (1 << bit) != 0;
            bit += 1;
        }
    }
    for k in 0..4 {
        for i in 0..4 {
            for j in 0..4 {
                matrix[i][j] |= matrix[i][k] && matrix[k][j];
            }
        }
    }
    (0..4)
        .filter(|&i| matrix[node][i])
        .map(|i| format!("src-{i}"))
        .collect()
}
#[test]
fn metadata_survives_warm_keep_invalidation_and_deletion() {
    for mask in 0..64 {
        let path = directory();
        let key = global_key(&["compiler", "options", "lock"]);
        {
            let mut store = Store::open(&path, &key);
            for node in 0..4 {
                let name = format!("src-{node}");
                let bytes = format!("source-{mask}-{node}").into_bytes();
                store.put(name.clone(), content_hash(&bytes), Some(&bytes));
                store.set_diagnostics(&name, format!("warning-{node}").as_bytes());
                store.set_dependents(&name, dependents(mask, node));
                store.set_tests(&name, vec![format!("test-{node}")], node % 2 == 0);
                store.set_generic_instance_hash(&name, format!("instance-{node}"));
                store.set_updated_time(&name, Some(UNIX_EPOCH + Duration::from_secs(node as u64)));
            }
            store.save();
        }
        let manifest = fs::read(path.join("manifest.toml")).unwrap();
        {
            let mut store = Store::open(&path, &key);
            for node in 0..4 {
                let name = format!("src-{node}");
                let bytes = format!("source-{mask}-{node}").into_bytes();
                let entry = store.entry(&name).unwrap();
                assert_eq!(
                    entry.file_hash,
                    content_hash(&bytes),
                    "cache metadata output mismatch: source hash"
                );
                assert_eq!(
                    store.load(entry),
                    Some(bytes),
                    "cache metadata output mismatch: fragment"
                );
                assert_eq!(
                    store.load_diagnostics(entry),
                    Some(format!("warning-{node}").into_bytes()),
                    "cache metadata output mismatch: diagnostics"
                );
                assert_eq!(
                    entry.dependents,
                    dependents(mask, node),
                    "cache metadata output mismatch: dependents"
                );
                assert_eq!(
                    entry.tests,
                    vec![format!("test-{node}")],
                    "cache metadata output mismatch: tests"
                );
                assert_eq!(
                    entry.has_non_native_test,
                    node % 2 == 0,
                    "cache metadata output mismatch: backend flag"
                );
                assert_eq!(
                    entry.generic_instance_hash,
                    format!("instance-{node}"),
                    "cache metadata output mismatch: generics"
                );
                assert_eq!(
                    entry.updated_time,
                    Some(UNIX_EPOCH + Duration::from_secs(node as u64)),
                    "cache metadata output mismatch: time"
                );
                store.keep(&name);
            }
            store.save();
            assert_eq!(
                fs::read(path.join("manifest.toml")).unwrap(),
                manifest,
                "cache metadata output mismatch: unchanged manifest"
            );
            for node in 0..4 {
                store.keep(&format!("src-{node}"));
            }
            store.invalidate("src-1");
            store.save();
            assert!(
                store.load(store.entry("src-1").unwrap()).is_none(),
                "cache metadata output mismatch: invalidation"
            );
            assert_eq!(
                store.entry("src-1").unwrap().dependents,
                dependents(mask, 1),
                "cache metadata output mismatch: invalidated dependencies"
            );
            store.save();
            for node in 0..4 {
                assert!(
                    store.entry(&format!("src-{node}")).is_none(),
                    "cache metadata output mismatch: deletion"
                );
            }
        }
    }
}
#[test]
fn global_keys_separate_boundaries_and_clear_old_entries() {
    for (a, b) in [
        (vec!["ab", "c"], vec!["a", "bc"]),
        (vec!["", "a"], vec!["a", ""]),
        (vec![], vec![""]),
        (
            vec!["compiler-1", "opts", "lock"],
            vec!["compiler-2", "opts", "lock"],
        ),
        (
            vec!["compiler", "opts-1", "lock"],
            vec!["compiler", "opts-2", "lock"],
        ),
        (
            vec!["compiler", "opts", "lock-1"],
            vec!["compiler", "opts", "lock-2"],
        ),
    ] {
        let first = global_key(&a);
        let second = global_key(&b);
        assert_ne!(first, second, "cache key output mismatch");
        let path = directory();
        {
            let mut store = Store::open(&path, &first);
            store.put("src".into(), "hash".into(), Some(b"fragment"));
            store.save();
        }
        let mut store = Store::open(&path, &second);
        assert!(
            store.entry("src").is_none(),
            "cache key output mismatch: stale entry"
        );
        store.save();
    }
}
