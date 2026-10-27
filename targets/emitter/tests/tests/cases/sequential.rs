//! Finite cross-target trace of FF prior-state reads and a pure function call.
//! A two-bit source fixture runs through real interpreter/JIT/AOT-C, emitted SV,
//! and synthesized cells. All three-instant en/data histories are enumerated
//! from reset. Function output arguments, global effects, X/Z, RAM, and arbitrary
//! scheduling histories are outside this source-to-hardware comparison.
use super::common;
use std::collections::HashMap;
use veryl_analyzer::{Analyzer, Context, ir as air};
use veryl_metadata::Metadata;
use veryl_parser::{Parser, resource_table};
use veryl_simulator::{
    Simulator,
    ir::{Config, Value, build_ir},
};
use veryl_synthesizer::{
    build_gate_ir,
    ir::{GateModule, NetDriver},
};
#[path = "../../../../synthesizer/tests/tests/common/mod.rs"]
pub(super) mod cells;
#[path = "../../../../synthesizer/tests/tests/common/clock.rs"]
mod clocks;
#[path = "../../../../simulator/tests/tests/common/mod.rs"]
pub(super) mod routes;
const SOURCE: &str = include_str!("../../../../simulator/fixtures/call_ff.veryl");

pub(super) fn analyze_source(source: &str) -> air::Ir {
    let metadata = Metadata::create_default("prj").unwrap();
    let analyzer = Analyzer::new(&metadata);
    analyzer.clear();
    let parser = Parser::parse(source, &"").unwrap();
    let mut context = Context::default();
    let mut ir = air::Ir::default();
    let mut errors = analyzer.analyze_pass1("prj", &parser.veryl);
    errors.extend(Analyzer::analyze_post_pass1());
    errors.extend(analyzer.analyze_pass2(&parser.veryl, &mut context, Some(&mut ir)));
    errors.extend(Analyzer::analyze_post_pass2(&ir));
    assert!(
        errors.is_empty(),
        "sequential fixture diagnostics: {errors:?}"
    );
    ir
}
const PORTS: [&str; 4] = ["q", "previous", "side", "returned"];
fn expected(state: &mut [u8; 4], en: u8, a: u8) {
    if en != 0 {
        let old = state[0];
        *state = [(old + a) & 3, old, (old + a + 1) & 3, ((old + a) & 3) ^ 3];
    }
}
fn word(gate: &GateModule, nets: &[u32], values: &HashMap<u32, bool>) -> u8 {
    let mut cache = vec![None; gate.nets.len()];
    nets.iter().enumerate().fold(0, |v, (i, &n)| {
        v | (u8::from(cells::eval(gate, n, values, &mut cache, 0)) << i)
    })
}
fn gate_values(gate: &GateModule, ff: &[bool], en: u8, a: u8, rst: u8) -> HashMap<u32, bool> {
    let mut values = HashMap::new();
    for port in &gate.ports {
        let name = resource_table::get_str_value(port.name).unwrap();
        let value = match name.as_str() {
            "clk" => 1,
            "rst" => rst,
            "en" => en,
            "a" => a,
            _ => continue,
        };
        for (i, &net) in port.nets.iter().enumerate() {
            if matches!(gate.nets[net as usize].driver, NetDriver::PortInput) {
                values.insert(net, value & (1 << i) != 0);
            }
        }
    }
    for (i, f) in gate.ffs.iter().enumerate() {
        values.insert(f.q, ff[i]);
    }
    values
}
fn compare() {
    let air = analyze_source(SOURCE);
    let gate = build_gate_ir(&air, resource_table::insert_str("Top"))
        .unwrap()
        .module;
    assert!(gate.ram_blocks.is_empty());
    assert!(!gate.ffs.is_empty());
    clocks::assert_single_clock(&gate, "clk", "sequential cross-target output mismatch");
    let mut sims = Vec::new();
    let calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    for (jit, cc) in [(false, false), (true, false), (true, true)] {
        let config = Config {
            use_jit: jit,
            aot_c: cc,
            aot_c_event: cc,
            aot_c_async: false,
            aot_c_min_stmts: 0,
            disable_ff_opt: true,
            ..Default::default()
        };
        let mut ir = build_ir(&air, "Top".into(), &config).unwrap();
        let (_, _, compiled, interpreted) = ir.detailed_stats();
        if cc {
            routes::assert_cc_comb_ready(&ir);
            assert!(!ir.whole_events.is_empty(), "AOT-C route missing");
            for event in ir.whole_events.values_mut() {
                *event = std::sync::Arc::new(Observed {
                    inner: event.clone(),
                    calls: calls.clone(),
                });
            }
        } else if jit {
            assert!(compiled > 0 && interpreted == 0, "JIT route fell back");
        } else {
            assert!(compiled == 0 && interpreted > 0);
        }
        sims.push(Simulator::new(ir, None));
    }
    let mut bench="module testbench;reg clk=0,rst=0,en=0;reg [1:0] a=0;wire [1:0] q,previous,side,returned;prj_Top dut(.*);initial begin\n".to_owned();
    let mut outputs = Vec::new();
    for history in 0..512 {
        let mut state = [0u8; 4];
        let mut ff = vec![false; gate.ffs.len()];
        for sim in &mut sims {
            let clk = sim.get_clock("clk").unwrap();
            let rst = sim.get_reset("rst").unwrap();
            sim.set("en", Value::new(0, 1, false));
            sim.set("a", Value::new(0, 2, false));
            sim.step_reset(&clk, &rst);
        }
        bench += "clk=0;rst=1;#1;clk=1;#1;clk=0;rst=0;#1;\n";
        for f in &gate.ffs {
            assert!(f.reset.is_some(), "fixture FF has no reset");
        }
        for (i, f) in gate.ffs.iter().enumerate() {
            ff[i] = f.reset_value;
        }
        let mut code = history;
        for _ in 0..3 {
            let a = (code % 4) as u8;
            let en = ((code / 4) % 2) as u8;
            code /= 8;
            expected(&mut state, en, a);
            for (backend, sim) in sims.iter_mut().enumerate() {
                sim.set("a", Value::new(u64::from(a), 2, false));
                sim.set("en", Value::new(u64::from(en), 1, false));
                let clk = sim.get_clock("clk").unwrap();
                sim.step(&clk);
                for (name, want) in PORTS.into_iter().zip(state) {
                    let got = sim.get(name).unwrap();
                    assert_eq!(
                        (
                            got.payload().to_u64_digits().first().copied().unwrap_or(0),
                            got.mask_xz().bits()
                        ),
                        (u64::from(want), 0),
                        "sequential cross-target output mismatch: native={backend} history={history} {name}"
                    );
                }
            }
            let values = gate_values(&gate, &ff, en, a, 0);
            ff = gate
                .ffs
                .iter()
                .map(|f| word(&gate, &[f.d], &values) != 0)
                .collect();
            let values = gate_values(&gate, &ff, en, a, 0);
            for (name, want) in PORTS.into_iter().zip(state) {
                let port = gate
                    .ports
                    .iter()
                    .find(|p| resource_table::get_str_value(p.name).unwrap() == name)
                    .unwrap();
                assert_eq!(
                    word(&gate, &port.nets, &values),
                    want,
                    "sequential cross-target output mismatch: gates history={history} {name}"
                );
            }
            bench += &format!(
                "a=2'd{a};en=1'b{en};#1;clk=1;#1;$display(\"case %0d %0d %0d %0d\",q,previous,side,returned);clk=0;#1;\n"
            );
            outputs.push(format!(
                "{} {} {} {}",
                state[0], state[1], state[2], state[3]
            ));
        }
    }
    assert!(
        calls.load(std::sync::atomic::Ordering::Relaxed) > 0,
        "AOT-C did not execute"
    );
    bench += "$display(\"complete\");$finish;end endmodule\n";
    assert_eq!(
        common::execute(SOURCE, &bench),
        outputs,
        "sequential cross-target output mismatch: SV"
    );
}

pub(super) struct Observed {
    pub(super) inner: std::sync::Arc<dyn veryl_simulator::backend::CompiledWhole>,
    pub(super) calls: std::sync::Arc<std::sync::atomic::AtomicUsize>,
}
impl Observed {
    fn record(
        &self,
        outcome: veryl_simulator::backend::DispatchOutcome,
    ) -> veryl_simulator::backend::DispatchOutcome {
        if matches!(outcome, veryl_simulator::backend::DispatchOutcome::Done) {
            self.calls
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
        outcome
    }
}
impl veryl_simulator::backend::CompiledWhole for Observed {
    fn try_dispatch(
        &self,
        ff: *const u8,
        comb: *mut u8,
        log: *mut u8,
    ) -> veryl_simulator::backend::DispatchOutcome {
        self.record(self.inner.try_dispatch(ff, comb, log))
    }
    fn try_dispatch_delta(
        &self,
        ff: *const u8,
        comb: *mut u8,
        log: *mut u8,
        delta: isize,
    ) -> veryl_simulator::backend::DispatchOutcome {
        self.record(self.inner.try_dispatch_delta(ff, comb, log, delta))
    }
    fn dataflow_marked(&self) -> bool {
        self.inner.dataflow_marked()
    }
    fn dataflow_ff_marked(&self) -> bool {
        self.inner.dataflow_ff_marked()
    }
}
#[test]
fn pure_calls_and_ff_reads_match_native_gate_and_sv() {
    if std::env::var_os("VERYL_FORMAL_CROSS_CHILD").is_some() {
        compare();
        return;
    }
    let mut command = std::process::Command::new(std::env::current_exe().unwrap());
    command
        .args([
            "--exact",
            "sequential::pure_calls_and_ff_reads_match_native_gate_and_sv",
            "--nocapture",
        ])
        .env("VERYL_FORMAL_CROSS_CHILD", "1")
        .env(
            "VERYL_AOT_CACHE_DIR",
            std::env::var_os("VERYL_AOT_CACHE_DIR")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| {
                    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/aot-cache")
                }),
        );
    let output = command.output().unwrap();
    let log = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.status.success(), "cross-target child failed:\n{log}");
    assert!(
        log.contains("1 passed; 0 failed"),
        "cross-target child did not complete"
    );
    eprintln!("{log}");
}
