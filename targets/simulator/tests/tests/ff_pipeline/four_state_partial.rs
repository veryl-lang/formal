//! Finite static/dynamic partial FF writes with independent per-bit replacement
//! and logical-end clipping. Exact fixtures define widths and backend routes;
//! no general write-lowering or scheduling theorem.

use super::four_state::{STATES, value};
use super::{Backend, Simulator, Value, analyze_state_config};
use veryl_formal_semantics::logic4::{Logic4, logic_from_planes};

fn source(width: usize, start: Option<usize>) -> String {
    let store = start.map_or_else(
        || "q[index+:8] = patch;".to_string(),
        |i| format!("q[{}:{i}] = patch;", i + 7),
    );
    format!(
        r#"
    module Top (
        clk: input clock, rst: input reset, en: input logic, load: input logic,
        data: input logic<{width}>, patch: input logic<8>, index: input logic<16>,
        out: output logic<{width}>, previous: output logic<{width}>,
    ) {{
        var q: logic<{width}>; var prev: logic<{width}>;
        always_ff {{
            if_reset {{q = 0; prev = 0;}}
            else if en {{if load {{q = data;}} else {{{store}}} prev = q;}}
        }}
        assign out = q; assign previous = prev;
    }}
    "#
    )
}
fn check(sim: &mut Simulator, port: &str, expected: &[Logic4], backend: Backend, step: usize) {
    let actual = sim.get(port).unwrap();
    let p = actual.payload();
    let m = actual.mask_xz();
    let bits: Vec<_> = (0..expected.len())
        .map(|i| logic_from_planes(p.bit(i as u64), m.bit(i as u64)))
        .collect();
    assert_eq!(
        bits,
        expected,
        "simulator four-state partial FF semantics mismatch: backend={backend:?} width={} step={step} port={port}",
        expected.len()
    );
    assert!(
        actual.width() == expected.len()
            && !actual.signed()
            && p.bits() <= expected.len() as u64
            && m.bits() <= expected.len() as u64,
        "simulator four-state partial FF semantics mismatch: metadata or padding"
    );
}
fn compare(backend: Backend) {
    for width in [65usize, 128] {
        let mut starts = vec![Some(0), Some(57), Some(width - 8), None];
        starts.dedup();
        for start in starts {
            for forced in [false, true] {
                let (ir, _) = analyze_state_config(&source(width, start), backend, forced, true);
                let mut sim = Simulator::new(ir, None);
                sim.write_log_diag.enabled = true;
                let clock = sim.get_clock("clk").unwrap();
                let reset = sim.get_reset("rst").unwrap();
                let mut state = vec![Logic4::Zero; width];
                let mut previous = state.clone();
                let mut step = 0;
                for pair in 0..16 {
                    if pair == 0 || pair == 8 {
                        sim.step_reset(&clock, &reset);
                        state.fill(Logic4::Zero);
                        previous.fill(Logic4::Zero);
                        check(&mut sim, "out", &state, backend, step);
                        check(&mut sim, "previous", &previous, backend, step);
                    }
                    let data: Vec<_> = (0..width)
                        .map(|i| STATES[(i + pair + i / 64) % 4])
                        .collect();
                    let patch: Vec<_> = (0..8).map(|i| STATES[(i / 2 + pair) % 4]).collect();
                    let index = [
                        0,
                        1,
                        7,
                        57,
                        60,
                        63,
                        64,
                        width - 8,
                        width - 1,
                        width,
                        width + 1,
                        65535,
                    ][pair % 12];
                    for (load, enable) in [
                        (true, Logic4::One),
                        (false, Logic4::One),
                        (false, Logic4::Zero),
                        (false, Logic4::X),
                        (false, Logic4::Z),
                    ] {
                        sim.set("data", value(&data));
                        sim.set("patch", value(&patch));
                        sim.set("index", Value::new(index as u64, 16, false));
                        sim.set("load", Value::new(u64::from(load), 1, false));
                        sim.set("en", value(&[enable]));
                        check(&mut sim, "out", &state, backend, step);
                        check(&mut sim, "previous", &previous, backend, step);
                        sim.step(&clock);
                        if enable == Logic4::One {
                            previous = state.clone();
                            if load {
                                state = data.clone();
                            } else {
                                for (bit, &v) in patch.iter().enumerate() {
                                    let dst = start.unwrap_or(index) + bit;
                                    if dst < width {
                                        state[dst] = v;
                                    }
                                }
                            }
                        }
                        step += 1;
                        check(&mut sim, "out", &state, backend, step);
                        check(&mut sim, "previous", &previous, backend, step);
                        assert_eq!(
                            sim.ir.write_log_buffer.count(),
                            0,
                            "simulator four-state partial FF semantics mismatch: uncleared event log"
                        );
                    }
                }
                assert!(sim.write_log_diag.total_entries > 0);
                eprintln!(
                    "partial four-state route backend={backend:?} width={width} start={start:?} forced={forced} clocks={step} entries={}",
                    sim.write_log_diag.total_entries
                );
            }
        }
    }
}
#[test]
fn interpreter_four_state_partial_ff() {
    compare(Backend::Interpret);
}
#[test]
fn cranelift_four_state_partial_ff() {
    compare(Backend::Cranelift);
}
