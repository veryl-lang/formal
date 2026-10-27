//! Finite version-split/comb-fusion comparisons against explicit state arithmetic.
//! Four-bit inputs and two-bit selectors are exhaustive along a fixed history.
//! Each process fixes pass switches before OnceLock initialization. Both original
//! and transformed routes must match the oracle; production counters witness
//! actual transformations. Other passes, X/Z, arbitrary circuits, and exhaustive
//! input histories are outside this domain.
use super::{Backend, Simulator, Value, analyze_config};
use std::process::Command;

struct Logger;
impl log::Log for Logger {
    fn enabled(&self, _: &log::Metadata<'_>) -> bool {
        true
    }
    fn log(&self, r: &log::Record<'_>) {
        if r.args().to_string().contains("version_split totals") {
            eprintln!("{}", r.args());
        }
    }
    fn flush(&self) {}
}
static LOGGER: Logger = Logger;
const SOURCE: &str = r#"module Top(clk:input clock,rst:input reset,a:input logic<4>,b:input logic<4>,s:input logic<2>,o:output logic<4>,state:output logic<4>){
var v:logic<4>;var q:logic<4>;
always_comb{v=a;if s[0]{v=b;}if s[1]{v=a^b;}}
let t:logic<4> =v+1;
assign o=t^a;
always_ff{if_reset{q=0;}else{q=q+v;}}
assign state=q;
}"#;
fn compare(backend: Backend) {
    let (ir, trace) = analyze_config(SOURCE, backend, false);
    let mut sim = Simulator::new(ir, None);
    let clk = sim.get_clock("clk").unwrap();
    let rst = sim.get_reset("rst").unwrap();
    sim.set("a", Value::new(0, 4, false));
    sim.set("b", Value::new(0, 4, false));
    sim.set("s", Value::new(0, 2, false));
    sim.step_reset(&clk, &rst);
    let mut state = 0u64;
    let mut count = 0;
    for a in 0..16u64 {
        for b in 0..16u64 {
            for s in 0..4u64 {
                sim.set("a", Value::new(a, 4, false));
                sim.set("b", Value::new(b, 4, false));
                sim.set("s", Value::new(s, 2, false));
                let chosen = if s & 2 != 0 {
                    a ^ b
                } else if s & 1 != 0 {
                    b
                } else {
                    a
                };
                let expected = ((chosen + 1) & 15) ^ a;
                let got = sim.get("o").unwrap();
                assert_eq!(
                    (
                        got.payload().to_u64_digits().first().copied().unwrap_or(0),
                        got.mask_xz().bits()
                    ),
                    (expected, 0),
                    "optimization output mismatch: backend={backend:?} a={a} b={b} s={s}"
                );
                sim.step(&clk);
                state = (state + chosen) & 15;
                assert_eq!(
                    sim.get("state")
                        .unwrap()
                        .payload()
                        .to_u64_digits()
                        .first()
                        .copied()
                        .unwrap_or(0),
                    state,
                    "optimization output mismatch: state backend={backend:?}"
                );
                count += 1;
            }
        }
    }
    if backend == Backend::Cc {
        assert!(trace.lock().unwrap().calls > 0, "compiled dispatch absent");
        drop(sim);
        super::assert_cc_comb_dispatched();
    }
    eprintln!("optimization complete vectors={count}");
}
fn run(backend: Backend, harness: &str) {
    if std::env::var_os("VERYL_FORMAL_OPT_CHILD").is_some() {
        log::set_logger(&LOGGER).unwrap();
        log::set_max_level(log::LevelFilter::Info);
        compare(backend);
        return;
    }
    for (vsplit, fusion) in [("0", "0"), ("1", "0"), ("0", "1"), ("1", "1")] {
        let output = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", harness, "--nocapture"])
            .env("VERYL_FORMAL_OPT_CHILD", "1")
            .env("VERYL_VSPLIT", vsplit)
            .env("VERYL_COMB_FUSION", fusion)
            .env("VERYL_COMB_FUSION_DIAG", "1")
            .env("VERYL_EVENT_GATE", "0")
            .env("VERYL_LANE_VECTOR", "0")
            .output()
            .unwrap();
        let log = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            output.status.success(),
            "optimization child failed: vsplit={vsplit} fusion={fusion}\n{log}"
        );
        assert!(
            log.contains("optimization complete vectors=1024")
                && log.contains("1 passed; 0 failed"),
            "optimization child did not finish"
        );
        if vsplit == "1" {
            assert!(
                log.split_whitespace().any(|t| t
                    .strip_prefix("fused_vars=")
                    .and_then(|n| n.parse::<usize>().ok())
                    .is_some_and(|n| n > 0)),
                "version split did not transform the fixture:\n{log}"
            );
        }
        if fusion == "1" {
            assert!(
                log.split_whitespace().any(|t| t
                    .strip_prefix("inlined=")
                    .and_then(|n| n.parse::<usize>().ok())
                    .is_some_and(|n| n > 0)),
                "comb fusion did not transform the fixture:\n{log}"
            );
        }
        eprintln!("{log}");
    }
}
#[test]
fn interpreter_optimization() {
    run(Backend::Interpret, "optimization::interpreter_optimization");
}
#[test]
fn cranelift_optimization() {
    run(Backend::Cranelift, "optimization::cranelift_optimization");
}
#[test]
fn aot_c_optimization() {
    run(Backend::Cc, "optimization::aot_c_optimization");
}
