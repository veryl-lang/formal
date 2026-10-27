use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use veryl_analyzer::{Analyzer, Context, ir};
use veryl_emitter::Emitter;
use veryl_metadata::Metadata;
use veryl_parser::Parser;

pub fn execute(source: &str, testbench: &str) -> Vec<String> {
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/fixtures")
        .join(format!(
            "{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
    fs::create_dir_all(&directory).unwrap();
    let src = directory.join("source.veryl");
    let sv = directory.join("source.sv");
    let tb = directory.join("testbench.sv");
    let executable = directory.join("simulation");
    fs::write(&src, source).unwrap();
    let metadata = Metadata::create_default("prj").unwrap();
    let analyzer = Analyzer::new(&metadata);
    analyzer.clear();
    let parser = Parser::parse(source, &src).unwrap();
    let mut context = Context::default();
    let mut air = ir::Ir::default();
    let mut errors = analyzer.analyze_pass1("prj", &parser.veryl);
    errors.extend(Analyzer::analyze_post_pass1());
    errors.extend(analyzer.analyze_pass2(&parser.veryl, &mut context, Some(&mut air)));
    errors.extend(Analyzer::analyze_post_pass2(&air));
    assert!(errors.is_empty(), "emitter fixture diagnostics: {errors:?}");
    let mut emitter = Emitter::new(
        &metadata,
        "prj",
        &src,
        &sv,
        &directory.join("source.sv.map"),
    );
    emitter.emit(&parser.veryl, source);
    fs::write(&sv, emitter.as_str()).unwrap();
    fs::write(&tb, testbench).unwrap();
    let output = Command::new("iverilog")
        .args(["-g2012", "-s", "testbench", "-o"])
        .arg(&executable)
        .arg(&sv)
        .arg(&tb)
        .output()
        .expect("Icarus Verilog must be installed");
    assert!(
        output.status.success(),
        "SV compilation failed in {}:\n{}",
        directory.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    let output = Command::new("vvp")
        .arg(&executable)
        .output()
        .expect("vvp must be installed");
    assert!(
        output.status.success(),
        "SV simulation failed in {}:\n{}",
        directory.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(
        text.lines().any(|s| s == "complete"),
        "SV completion handshake absent: {text}"
    );
    text.lines()
        .filter_map(|line| line.strip_prefix("case ").map(str::to_owned))
        .collect()
}
