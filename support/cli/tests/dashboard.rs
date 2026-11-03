use std::collections::BTreeSet;
use veryl_formal_ci::catalog;

#[test]
fn catalogs_keep_target_method_and_adapter_namespaces() {
    let entries = catalog::entries();
    let keys: BTreeSet<_> = entries.iter().map(|c| c["id"].as_str().unwrap()).collect();
    assert_eq!(keys.len(), entries.len());
    let spans: Vec<_> = entries
        .iter()
        .filter(|c| c["target"] == "frontend" && c["name"] == "byte-span")
        .collect();
    assert_eq!(spans.len(), 2);
    assert_ne!(spans[0]["id"], spans[1]["id"]);
    assert!(
        entries
            .iter()
            .filter(|c| c["manifest"] == "targets/simulator/cbmc/Cargo.toml")
            .all(|c| c["method"] == "cbmc")
    );
    let targets: BTreeSet<_> = entries
        .iter()
        .map(|c| c["target"].as_str().unwrap())
        .collect();
    assert_eq!(
        targets,
        [
            "analyzer",
            "cache",
            "emitter",
            "frontend",
            "simulator",
            "synthesizer"
        ]
        .into_iter()
        .collect()
    );
}
