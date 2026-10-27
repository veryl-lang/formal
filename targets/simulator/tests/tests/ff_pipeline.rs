//! Real source-to-FF execution with independent state and write-log expectations.
//! Dispatch observers require the stated compiled route and compare committed
//! storage with a replay of actual records. Explicit fallback cases are distinct
//! from strict compiled execution. Fixture modules state their finite domains;
//! no whole-backend, arbitrary-source, or general scheduler theorem follows.

use std::sync::{Arc, Mutex};
use veryl_analyzer::{Analyzer, Context, ir as air};
use veryl_formal_semantics::write_log::{NarrowRecord, WideRecord, committed};
use veryl_metadata::Metadata;
use veryl_parser::Parser;
use veryl_simulator::Simulator;
use veryl_simulator::backend::{CompiledWhole, DispatchOutcome};
use veryl_simulator::ir::{Config, Ir, Value, WriteLogBuffer, build_ir};

mod common;

/// Call after dropping the simulator in an otherwise isolated child process;
/// production publishes its per-instance counts on drop.
fn assert_cc_comb_dispatched() {
    let counts = veryl_simulator::residency::dispatch_counts();
    let comb: Vec<_> = counts
        .iter()
        .filter(|(name, _, _)| name.starts_with("dataflow:"))
        .collect();
    assert!(!comb.is_empty(), "AOT-C combinational dispatch absent");
    assert!(
        comb.iter()
            .all(|(_, ran, fallback)| *ran > 0 && *fallback == 0),
        "AOT-C combinational dispatch fell back: {comb:?}"
    );
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Backend {
    Interpret,
    Cranelift,
    CraneliftFallback,
    Cc,
}

#[derive(Default)]
struct Trace {
    calls: usize,
    narrow: usize,
    wide: usize,
    expected: Option<Vec<u8>>,
}
struct ObservedWhole {
    inner: Arc<dyn CompiledWhole>,
    trace: Arc<Mutex<Trace>>,
    ff_len: usize,
}
impl ObservedWhole {
    fn observe(&self, outcome: DispatchOutcome, ff: *const u8, log: *mut u8) -> DispatchOutcome {
        if matches!(outcome, DispatchOutcome::Done) {
            // The simulator supplies live FF storage and its live WriteLogBuffer.
            let (before, log) = unsafe {
                (
                    std::slice::from_raw_parts(ff, self.ff_len),
                    &*log.cast::<WriteLogBuffer>(),
                )
            };
            let narrow: Vec<_> = log
                .narrow_entries_slice()
                .iter()
                .take(log.narrow_count() as usize)
                .map(|e| NarrowRecord {
                    offset: e.offset,
                    width_class: e.width_class,
                    payload: e.payload,
                })
                .collect();
            let wide: Vec<_> = log
                .wide_entries_slice()
                .iter()
                .take(log.wide_count() as usize)
                .map(|e| WideRecord {
                    offset: e.offset,
                    payload: e.payload[..e.native_bytes as usize].to_vec(),
                })
                .collect();
            let mut trace = self.trace.lock().unwrap();
            trace.calls += 1;
            trace.narrow += narrow.len();
            trace.wide += wide.len();
            trace.expected = Some(committed(before, &narrow, &wide).0);
        }
        outcome
    }
}
impl CompiledWhole for ObservedWhole {
    fn try_dispatch(&self, ff: *const u8, comb: *mut u8, log: *mut u8) -> DispatchOutcome {
        self.observe(self.inner.try_dispatch(ff, comb, log), ff, log)
    }
    fn try_dispatch_delta(
        &self,
        ff: *const u8,
        comb: *mut u8,
        log: *mut u8,
        delta: isize,
    ) -> DispatchOutcome {
        self.observe(self.inner.try_dispatch_delta(ff, comb, log, delta), ff, log)
    }
    fn dataflow_marked(&self) -> bool {
        self.inner.dataflow_marked()
    }
    fn dataflow_ff_marked(&self) -> bool {
        self.inner.dataflow_ff_marked()
    }
}

fn analyze(code: &str, backend: Backend) -> (Ir, Arc<Mutex<Trace>>) {
    analyze_config(code, backend, true)
}

fn analyze_config(code: &str, backend: Backend, disable_ff_opt: bool) -> (Ir, Arc<Mutex<Trace>>) {
    analyze_state_config(code, backend, disable_ff_opt, false)
}

fn analyze_state_config(
    code: &str,
    backend: Backend,
    disable_ff_opt: bool,
    use_4state: bool,
) -> (Ir, Arc<Mutex<Trace>>) {
    assert!(
        !use_4state || backend != Backend::Cc,
        "AOT-C does not support four-state execution"
    );
    let metadata = Metadata::create_default("prj").unwrap();
    let analyzer = Analyzer::new(&metadata);
    analyzer.clear();
    let parser = Parser::parse(code, &"").unwrap();
    let mut context = Context::default();
    let mut air = air::Ir::default();
    let mut errors = analyzer.analyze_pass1("prj", &parser.veryl);
    errors.extend(Analyzer::analyze_post_pass1());
    errors.extend(analyzer.analyze_pass2(&parser.veryl, &mut context, Some(&mut air)));
    errors.extend(Analyzer::analyze_post_pass2(&air));
    assert!(errors.is_empty(), "pipeline source diagnostics: {errors:?}");
    let config = Config {
        use_4state,
        use_jit: backend != Backend::Interpret,
        disable_ff_opt,
        aot_c: backend == Backend::Cc,
        aot_c_event: backend == Backend::Cc,
        aot_c_async: false,
        aot_c_validate: false,
        aot_c_min_stmts: 0,
        ..Default::default()
    };
    let mut ir = build_ir(&air, "Top".into(), &config).unwrap();
    let trace = Arc::new(Mutex::new(Trace::default()));
    let (_, _, compiled, interpreted) = ir.detailed_stats();
    match backend {
        Backend::Interpret => {
            assert_eq!(compiled, 0, "interpreter route changed");
            assert!(interpreted > 0);
            assert!(ir.whole_events.is_empty());
        }
        Backend::Cranelift => {
            assert!(compiled > 0, "Cranelift event route absent");
            assert_eq!(interpreted, 0, "Cranelift event fell back to interpreter");
            assert!(ir.whole_events.is_empty());
        }
        Backend::CraneliftFallback => {
            assert!(
                interpreted > 0,
                "expected interpreter event fallback disappeared"
            );
            assert!(ir.whole_events.is_empty());
            eprintln!(
                "explicit Cranelift fallback route compiled={compiled} interpreted={interpreted}"
            );
        }
        Backend::Cc => {
            common::assert_cc_comb_ready(&ir);
            assert!(
                !ir.whole_events.is_empty(),
                "AOT-C event route absent; a working C compiler and shared-library loader are required"
            );
            let ff_len = ir.ff_values.len();
            for artifact in ir.whole_events.values_mut() {
                *artifact = Arc::new(ObservedWhole {
                    inner: artifact.clone(),
                    trace: trace.clone(),
                    ff_len,
                });
            }
        }
    }
    (ir, trace)
}

fn source(width: usize, dynamic: bool) -> String {
    let writes = if dynamic {
        "q[index+:8] = tag;"
    } else {
        "q[7:0] = tag; q[11:4] = tag2;"
    };
    format!(
        r#"
    module Top (
        clk: input clock, rst: input reset,
        data: input logic<{width}>, tag: input logic<8>, tag2: input logic<8>, index: input logic<16>,
        result: output logic<{width}>, previous: output logic<{width}>, count: output logic<8>,
    ) {{
        var q: logic<{width}>;
        var prev: logic<{width}>;
        var counter: logic<8>;
        always_ff {{
            if_reset {{q = 0; prev = 0; counter = 0;}}
            else {{q = data; {writes} prev = q; counter = counter + 1;}}
        }}
        assign result = q;
        assign previous = prev;
        assign count = counter;
    }}
    "#
    )
}
fn value(bytes: &[u8], width: usize) -> Value {
    let mut payload = vec![0; width.div_ceil(64) * 8];
    payload[..bytes.len()].copy_from_slice(bytes);
    Value::from_le_bytes(&payload, &vec![0; payload.len()], width, false)
}
fn check(
    sim: &mut Simulator,
    port: &str,
    bytes: &[u8],
    width: usize,
    backend: Backend,
    cycle: usize,
) {
    assert_eq!(
        sim.get(port),
        Some(value(bytes, width)),
        "simulator FF pipeline semantics mismatch: {backend:?} width={width} cycle={cycle} port={port}"
    );
}
fn check_commit(
    sim: &Simulator,
    trace: &Arc<Mutex<Trace>>,
    backend: Backend,
    previous_calls: usize,
) {
    if backend == Backend::Cc {
        let trace = trace.lock().unwrap();
        assert!(
            trace.calls > previous_calls,
            "AOT-C fell back without successful dispatch"
        );
        assert_eq!(
            trace.expected.as_deref(),
            Some(&sim.ir.ff_values[..]),
            "simulator FF pipeline semantics mismatch: recorded commit differs"
        );
    }
    assert_eq!(
        sim.ir.write_log_buffer.count(),
        0,
        "simulator FF pipeline semantics mismatch: event did not clear its log"
    );
}

fn compare(backend: Backend) {
    for width in [16usize, 64, 65, 128, 448, 512] {
        for dynamic in [false, true] {
            let (ir, trace) = analyze(&source(width, dynamic), backend);
            let mut sim = Simulator::new(ir, None);
            sim.write_log_diag.enabled = true;
            let clock = sim.get_clock("clk").unwrap();
            let reset = sim.get_reset("rst").unwrap();
            sim.step_reset(&clock, &reset);
            check_commit(&sim, &trace, backend, 0);
            let mut old = vec![0; width.div_ceil(8)];
            check(&mut sim, "result", &old, width, backend, 0);
            check(&mut sim, "previous", &old, width, backend, 0);
            check(&mut sim, "count", &[0], 8, backend, 0);
            for cycle in 1..=18 {
                let mut data: Vec<_> = (0..old.len())
                    .map(|i| match cycle % 6 {
                        0 => 0,
                        1 => 255,
                        2 => 0xaa,
                        3 => 0x55,
                        4 => (i * 37 + cycle) as u8,
                        _ => 1 << (i % 8),
                    })
                    .collect();
                if width % 8 != 0 {
                    *data.last_mut().unwrap() &= (1 << (width % 8)) - 1;
                }
                let tag = (cycle * 13) as u8;
                let tag2 = !tag;
                let index = [
                    0,
                    4,
                    7,
                    width - 8,
                    width - 4,
                    width - 1,
                    width,
                    width + 1,
                    65535,
                ][(cycle - 1) % 9];
                sim.set("data", value(&data, width));
                sim.set("tag", Value::new(tag as u64, 8, false));
                sim.set("tag2", Value::new(tag2 as u64, 8, false));
                sim.set("index", Value::new(index as u64, 16, false));
                let mut expected = data;
                let writes: Vec<_> = if dynamic {
                    vec![(index, tag)]
                } else {
                    vec![(0, tag), (4, tag2)]
                };
                for (start, byte) in writes {
                    for bit in 0..8 {
                        let dst = start + bit;
                        if dst < width {
                            let mask = 1 << (dst % 8);
                            expected[dst / 8] =
                                (expected[dst / 8] & !mask) | (((byte >> bit) & 1) << (dst % 8));
                        }
                    }
                }
                let calls = trace.lock().unwrap().calls;
                sim.step(&clock);
                check_commit(&sim, &trace, backend, calls);
                check(&mut sim, "result", &expected, width, backend, cycle);
                check(&mut sim, "previous", &old, width, backend, cycle);
                check(&mut sim, "count", &[cycle as u8], 8, backend, cycle);
                old = expected;
            }
            assert!(
                sim.write_log_diag.total_entries > 0,
                "FF logs were not exercised"
            );
            if backend == Backend::Cc {
                let trace = trace.lock().unwrap();
                assert!(trace.narrow > 0, "narrow records absent");
                if width > 128 {
                    assert!(trace.wide > 0, "wide records absent");
                } else {
                    assert_eq!(
                        trace.wide, 0,
                        "native FF writes must stay in the narrow pool"
                    );
                }
                eprintln!(
                    "pipeline route {backend:?} width={width} dynamic={dynamic} dispatches={} narrow={} wide={}",
                    trace.calls, trace.narrow, trace.wide
                );
            } else {
                eprintln!(
                    "pipeline route {backend:?} width={width} dynamic={dynamic} entries={}",
                    sim.write_log_diag.total_entries
                );
            }
        }
    }
}
#[test]
fn interpreter_ff_pipeline() {
    compare(Backend::Interpret);
}
#[test]
fn cranelift_ff_pipeline() {
    compare(Backend::Cranelift);
}
#[test]
fn aot_c_ff_pipeline() {
    compare(Backend::Cc);
}

#[test]
fn missing_cc_comb_artifact_is_rejected() {
    let (mut ir, _) = analyze(&source(16, false), Backend::Cc);
    ir.dataflow = None;
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            common::assert_cc_comb_ready(&ir);
        }))
        .is_err()
    );
}

#[test]
fn missing_cc_comb_dispatch_is_rejected() {
    if std::env::var_os("VERYL_FORMAL_MISSING_COMB_CHILD").is_some() {
        let (mut ir, _) = analyze(&source(16, false), Backend::Cc);
        ir.dataflow = None;
        let mut sim = Simulator::new(ir, None);
        let clock = sim.get_clock("clk").unwrap();
        let reset = sim.get_reset("rst").unwrap();
        sim.step_reset(&clock, &reset);
        drop(sim);
        assert_cc_comb_dispatched();
        return;
    }
    let out = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "missing_cc_comb_dispatch_is_rejected",
            "--nocapture",
        ])
        .env("VERYL_FORMAL_MISSING_COMB_CHILD", "1")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(101));
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("AOT-C combinational dispatch absent"),
        "unexpected child failure: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[path = "ff_pipeline/refinement.rs"]
mod refinement;

#[path = "ff_pipeline/idle.rs"]
mod idle;

#[path = "ff_pipeline/four_state.rs"]
mod four_state;

#[path = "ff_pipeline/four_state_partial.rs"]
mod four_state_partial;

#[path = "ff_pipeline/four_state_array.rs"]
mod four_state_array;

#[path = "ff_pipeline/comb_array.rs"]
mod comb_array;

#[path = "ff_pipeline/gate_cycle.rs"]
mod gate_cycle;

#[path = "ff_pipeline/external_write.rs"]
mod external_write;

#[path = "ff_pipeline/unknown_write.rs"]
mod unknown_write;

#[path = "ff_pipeline/signed_array.rs"]
mod signed_array;

#[path = "ff_pipeline/multidim_array.rs"]
mod multidim_array;

#[path = "ff_pipeline/unknown_select.rs"]
mod unknown_select;

#[path = "ff_pipeline/select_forms.rs"]
mod select_forms;

#[path = "ff_pipeline/signed_select.rs"]
mod signed_select;

#[path = "ff_pipeline/wide_select.rs"]
mod wide_select;

#[path = "ff_pipeline/packed_read.rs"]
mod packed_read;

#[path = "ff_pipeline/packed_read_wide.rs"]
mod packed_read_wide;

#[path = "ff_pipeline/constant_select.rs"]
mod constant_select;

#[path = "ff_pipeline/constant_edges.rs"]
mod constant_edges;

#[path = "ff_pipeline/packed_dimensions.rs"]
mod packed_dimensions;

#[path = "ff_pipeline/packed_ranges.rs"]
mod packed_ranges;

#[path = "ff_pipeline/packed_stride_ranges.rs"]
mod packed_stride_ranges;

#[path = "ff_pipeline/events.rs"]
mod events;

#[path = "ff_pipeline/external_wide.rs"]
mod external_wide;

#[path = "ff_pipeline/optimization.rs"]
mod optimization;

#[path = "ff_pipeline/pass_shapes.rs"]
mod pass_shapes;
