//! Finite numeric source-map lookups against a lexicographic predecessor model,
//! including serialized Unicode contents. Numeric coordinates are not LSP UTF-16;
//! arbitrary source maps, URI escaping, and missing files are outside this fixture.

use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use veryl_sourcemap::SourceMap;

#[test]
fn source_map_roundtrip_uses_one_based_predecessor_coordinates() {
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/fixtures")
        .join(format!(
            "map-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
    fs::create_dir_all(&directory).unwrap();
    let source = directory.join("日本.veryl");
    let destination = directory.join("output.sv");
    let map_file = directory.join("output.sv.map");
    let contents = "// α日本😀 abcdefghijklmnop\n".repeat(5);
    fs::write(&source, &contents).unwrap();
    let canonical = fs::canonicalize(&source).unwrap();
    let mut map = SourceMap::new(&source, &destination, &map_file);
    assert!(map.lookup(1, 1).is_none() && map.to_bytes().is_err());
    let points = [
        ((1, 3), (1, 1)),
        ((1, 9), (2, 4)),
        ((3, 2), (4, 2)),
        ((3, 17), (1, 7)),
        ((5, 6), (3, 9)),
    ];
    for &((dl, dc), (sl, sc)) in &points {
        map.add(dl, dc, sl, sc, "日本😀");
    }
    map.set_source_content(&contents);
    map.build();
    let bytes = map.to_bytes().unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(
        json["sourcesContent"][0], contents,
        "source map content mismatch"
    );
    fs::write(&map_file, bytes).unwrap();
    fs::write(
        &destination,
        format!("module output;endmodule\n{}\n", map.get_link()),
    )
    .unwrap();
    let loaded = SourceMap::from_src(&destination).unwrap();
    let mut queries = 0;
    for line in 1..=6 {
        for column in 1..=30 {
            let want = points
                .iter()
                .filter(|(p, _)| *p <= (line, column))
                .max_by_key(|(p, _)| *p)
                .map(|(_, p)| (canonical.clone(), p.0, p.1));
            assert_eq!(
                map.lookup(line, column),
                want,
                "source map coordinate mismatch: line={line} column={column}"
            );
            assert_eq!(
                loaded.lookup(line, column),
                want,
                "source map coordinate mismatch: reloaded line={line} column={column}"
            );
            queries += 2;
        }
    }
    eprintln!("source map complete mappings=5 queries={queries}");
}
