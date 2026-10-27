//! Finite output equivalence for dead-let elimination and bit-lane merging.
//! Each pass runs enabled/disabled in fresh processes; diagnostics must witness
//! a real transformation. Other preprocessing is shared. Eight-bit operands use
//! boundary patterns, not arbitrary symbolic inputs or arbitrary pass sequences.
use super::{Backend, Simulator, Value, analyze_config};
use std::process::Command;
fn source(kind: &str) -> String {
    let rhs = if kind == "lane" {
        let lanes: Vec<_> = (0..8)
            .rev()
            .map(|i| format!("(((a[{i}]^b[{i}])&~s[0])|s[1])"))
            .collect();
        format!("assign o={{{}}};", lanes.join(","))
    } else {
        "let _unused:logic<8> = a ^ b;let live:logic<8> = a+b;assign o=live^8'h5a;".into()
    };
    format!(
        "module Top(clk:input clock,rst:input reset,a:input logic<8>,b:input logic<8>,s:input logic<2>,o:output logic<8>,q:output logic<8>){{{rhs}var r:logic<8>;always_ff{{if_reset{{r=0;}}else{{r=r+o;}}}}assign q=r;}}"
    )
}
fn compare(kind: &str, backend: Backend) {
    let (ir, trace) = analyze_config(&source(kind), backend, false);
    let mut sim = Simulator::new(ir, None);
    let clk = sim.get_clock("clk").unwrap();
    let rst = sim.get_reset("rst").unwrap();
    for (name, width) in [("a", 8), ("b", 8), ("s", 2)] {
        sim.set(name, Value::new(0, width, false));
    }
    sim.step_reset(&clk, &rst);
    let mut q = 0u64;
    for a in [0, 1, 0x55, 0x80, 0xfe, 255u64] {
        for b in [0, 1, 0x55, 0x80, 0xfe, 255u64] {
            for s in 0..4 {
                sim.set("a", Value::new(a, 8, false));
                sim.set("b", Value::new(b, 8, false));
                sim.set("s", Value::new(s, 2, false));
                let expected = if kind == "lane" {
                    ((a ^ b) & if s & 1 == 0 { 255 } else { 0 }) | if s & 2 != 0 { 255 } else { 0 }
                } else {
                    ((a + b) & 255) ^ 0x5a
                };
                for (name, want) in [("o", expected), ("q", q)] {
                    let got = sim.get(name).unwrap();
                    assert_eq!(
                        (
                            got.payload().to_u64_digits().first().copied().unwrap_or(0),
                            got.mask_xz().bits()
                        ),
                        (want, 0),
                        "pass shape output mismatch: {kind} {backend:?} {name} a={a} b={b} s={s}"
                    );
                }
                sim.step(&clk);
                q = (q + expected) & 255;
                assert_eq!(
                    sim.get("q")
                        .unwrap()
                        .payload()
                        .to_u64_digits()
                        .first()
                        .copied()
                        .unwrap_or(0),
                    q,
                    "pass shape output mismatch: FF"
                );
            }
        }
    }
    if backend == Backend::Cc {
        assert!(trace.lock().unwrap().calls > 0, "C dispatch absent");
        drop(sim);
        super::assert_cc_comb_dispatched();
    }
    eprintln!("pass shape complete kind={kind}");
}
fn run(kind: &str, harness: &str) {
    if let Ok(backend) = std::env::var("VERYL_FORMAL_SHAPE_CHILD") {
        compare(
            kind,
            match backend.as_str() {
                "interpret" => Backend::Interpret,
                "jit" => Backend::Cranelift,
                _ => Backend::Cc,
            },
        );
        return;
    }
    for backend in ["interpret", "jit", "cc"] {
        for on in ["0", "1"] {
            let output = Command::new(std::env::current_exe().unwrap())
                .args(["--exact", harness, "--nocapture"])
                .env("VERYL_FORMAL_SHAPE_CHILD", backend)
                .env("VERYL_EVENT_GATE", "0")
                .env("VERYL_VSPLIT", "0")
                .env("VERYL_COMB_FUSION", if kind == "lane" { "1" } else { "0" })
                .env("VERYL_LANE_VECTOR", if kind == "lane" { on } else { "0" })
                .env("VERYL_LANE_FOLD", "0")
                .env("VERYL_LANE_MERGE", "1")
                .env("VERYL_LANE_VECTOR_DIAG", "1")
                .env("VERYL_DEAD_VAR_DCE", if kind == "dce" { on } else { "0" })
                .env("VERYL_DEAD_VAR_DCE_DIAG", "1")
                .output()
                .unwrap();
            let log = format!(
                "{}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(
                output.status.success(),
                "pass shape child failed {kind} {backend} {on}:\n{log}"
            );
            assert!(
                log.contains("pass shape complete") && log.contains("1 passed; 0 failed"),
                "shape child incomplete"
            );
            if on == "1" {
                assert!(
                    if kind == "dce" {
                        log.contains("[DeadVarDce]")
                    } else {
                        log.contains("[lane_vector] lane merge:")
                    },
                    "pass did not transform {kind}:\n{log}"
                );
            }
            eprintln!("{log}");
        }
    }
}
#[test]
fn dead_let_outputs() {
    run("dce", "pass_shapes::dead_let_outputs");
}
#[test]
fn merged_bit_lane_outputs() {
    run("lane", "pass_shapes::merged_bit_lane_outputs");
}
