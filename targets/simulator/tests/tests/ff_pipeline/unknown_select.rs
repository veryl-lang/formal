//! Finite unknown packed-write selectors must preserve unrelated storage.
//! Expected writes use explicit per-bit positions, not production index decoding.

use super::four_state::{STATES, value};
use super::signed_array::known;
use super::{Backend, Simulator, analyze_state_config};
use std::process::Command;
use veryl_formal_semantics::logic4::Logic4;

fn source(width: usize, bits: usize, window: usize, array: bool) -> String {
    let count = if array { 3 } else { 1 };
    let name = if array { "arr[element]" } else { "q" };
    let select = if window == 1 {
        "[index]".to_string()
    } else {
        format!("[index+:{window}]")
    };
    let ports: String = (0..count)
        .map(|i| format!("o{i}: output logic<{width}>, "))
        .collect();
    let destination = |i: usize| {
        if array {
            format!("arr[{i}]")
        } else {
            "q".into()
        }
    };
    let reset: String = (0..count)
        .map(|i| format!("{} = 0;", destination(i)))
        .collect();
    let assigns: String = (0..count)
        .map(|i| format!("assign o{i} = {};", destination(i)))
        .collect();
    let declaration = if array {
        format!("var arr: logic<{width}> [3];")
    } else {
        format!("var q: logic<{width}>;")
    };
    let first = destination(0);
    format!(
        r#"
    module Top (
        clk: input clock, rst: input reset, load: input logic, element: input logic<2>,
        data: input logic<{width}>, patch: input logic<{window}>, index: input logic<{bits}>,
        previous: output logic<{width}>, {ports}
    ) {{
        {declaration} var prev: logic<{width}>;
        always_ff {{
            if_reset {{{reset} prev = 0;}}
            else {{if load {{{name} = data;}} else {{{name}{select} = patch;}} prev = {first};}}
        }}
        {assigns} assign previous = prev;
    }}
    "#
    )
}
struct Trial {
    sim: Simulator,
    state: Vec<Vec<Logic4>>,
    previous: Vec<Logic4>,
    step: usize,
    backend: Backend,
}
impl Trial {
    fn check(&mut self) {
        for (i, want) in self.state.iter().enumerate() {
            assert_eq!(
                self.sim.get(&format!("o{i}")),
                Some(value(want)),
                "simulator unknown-select output mismatch: {:?} width={} step={} element={i}",
                self.backend,
                want.len(),
                self.step
            );
        }
        assert_eq!(
            self.sim.get("previous"),
            Some(value(&self.previous)),
            "simulator unknown-select output mismatch: previous step={}",
            self.step
        );
    }
    fn tick(
        &mut self,
        idx: &[Logic4],
        start: Option<usize>,
        element: usize,
        window: usize,
        load: bool,
    ) {
        let width = self.previous.len();
        let data: Vec<_> = (0..width)
            .map(|i| STATES[(i + i / 64 + self.step) % 4])
            .collect();
        let patch: Vec<_> = (0..window)
            .map(|i| STATES[(i + self.step + 2) % 4])
            .collect();
        self.sim.set("data", value(&data));
        self.sim.set("patch", value(&patch));
        self.sim.set("index", value(idx));
        self.sim.set("element", value(&known(element as i128, 2)));
        self.sim.set(
            "load",
            value(&[if load { Logic4::One } else { Logic4::Zero }]),
        );
        self.check();
        let clock = self.sim.get_clock("clk").unwrap();
        self.sim.step(&clock);
        self.previous = self.state[0].clone();
        if load {
            self.state[element] = data;
        } else if let Some(start) = start {
            for (bit, &v) in patch.iter().enumerate() {
                if let Some(dst) = self.state[element].get_mut(start + bit) {
                    *dst = v;
                }
            }
        }
        self.step += 1;
        self.check();
        assert_eq!(
            self.sim.ir.write_log_buffer.count(),
            0,
            "simulator unknown-select output mismatch: uncleared log"
        );
    }
}
fn compare(backend: Backend, widths: &[usize], array: bool) {
    let mut configurations = 0;
    let mut clocks = 0;
    for &width in widths {
        for bits in [1, 16] {
            for window in [1, 8] {
                for forced in [false, true] {
                    let (ir, _) = analyze_state_config(
                        &source(width, bits, window, array),
                        backend,
                        forced,
                        true,
                    );
                    if array {
                        super::four_state_array::layout(&ir, width);
                    } else {
                        let q = ir
                            .module_variables
                            .variables
                            .values()
                            .find(|v| v.path.to_string() == "q")
                            .unwrap();
                        let ff = ir.ff_values.as_ptr() as usize
                            ..ir.ff_values.as_ptr() as usize + ir.ff_values.len();
                        let comb = ir.comb_values.as_ptr() as usize
                            ..ir.comb_values.as_ptr() as usize + ir.comb_values.len();
                        for &p in q.current_values.iter().chain(&q.next_values) {
                            assert!(
                                [&ff, &comb].iter().any(|r| p as usize >= r.start
                                    && (p as usize)
                                        .checked_add(2 * q.native_bytes)
                                        .is_some_and(|end| end <= r.end)),
                                "unknown-select allocation invalid"
                            );
                        }
                    }
                    let (_, _, ec, ei) = ir.detailed_stats();
                    eprintln!(
                        "unknown-select route {backend:?} width={width} bits={bits} window={window} array={array} forced={forced} events={ec}/{ei}"
                    );
                    let count = if array { 3 } else { 1 };
                    let mut t = Trial {
                        sim: Simulator::new(ir, None),
                        state: vec![vec![Logic4::Zero; width]; count],
                        previous: vec![Logic4::Zero; width],
                        step: 0,
                        backend,
                    };
                    t.sim.write_log_diag.enabled = true;
                    let clock = t.sim.get_clock("clk").unwrap();
                    let reset = t.sim.get_reset("rst").unwrap();
                    for _ in 0..2 {
                        t.sim.step_reset(&clock, &reset);
                        t.state.iter_mut().for_each(|v| v.fill(Logic4::Zero));
                        t.previous.fill(Logic4::Zero);
                        t.check();
                        for element in 0..count {
                            t.tick(&known(0, bits), Some(0), element, window, true);
                        }
                        for bit in 0..bits {
                            for base in [0, (1usize << bits) - 1] {
                                for unknown in [Logic4::X, Logic4::Z] {
                                    let mut idx = known(base as i128, bits);
                                    idx[bit] = unknown;
                                    let element = t.step % count;
                                    t.tick(&idx, None, element, window, false);
                                    let start = if bits == 1 {
                                        base
                                    } else {
                                        [0, 1, 7, 57, 63, 64, width - 1, width, width + 1, 65535]
                                            [t.step % 10]
                                    };
                                    t.tick(
                                        &known(start as i128, bits),
                                        Some(start),
                                        element,
                                        window,
                                        false,
                                    );
                                }
                            }
                        }
                    }
                    assert_eq!(t.step, 2 * count + 16 * bits);
                    assert!(
                        t.sim.write_log_diag.total_entries > 0,
                        "unknown-select logs unused"
                    );
                    clocks += t.step;
                    configurations += 1;
                }
            }
        }
    }
    assert_eq!(configurations, widths.len() * 8);
    eprintln!(
        "unknown-select complete {backend:?} array={array} configurations={configurations} clocks={clocks}"
    );
}
fn run(backend: Backend, widths: &[usize], array: bool, harness: &str) {
    if std::env::var_os("VERYL_FORMAL_UNKNOWN_SELECT_CHILD").is_some() {
        compare(backend, widths, array);
        return;
    }
    let out = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", harness, "--nocapture"])
        .env("VERYL_FORMAL_UNKNOWN_SELECT_CHILD", "1")
        .output()
        .unwrap();
    let log = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(out.status.success(), "unknown-select child failed:\n{log}");
    assert!(
        log.contains("1 passed; 0 failed") && log.contains("unknown-select complete"),
        "unknown-select child incomplete"
    );
    eprintln!("{log}");
}
#[test]
fn interpreter_scalar_unknown_select() {
    run(
        Backend::Interpret,
        &[8, 64, 65, 128, 129, 512],
        false,
        "unknown_select::interpreter_scalar_unknown_select",
    );
}
#[test]
fn cranelift_scalar_unknown_select() {
    run(
        Backend::Cranelift,
        &[8, 64, 65, 128],
        false,
        "unknown_select::cranelift_scalar_unknown_select",
    );
}
#[test]
fn fallback_scalar_unknown_select() {
    run(
        Backend::CraneliftFallback,
        &[129, 512],
        false,
        "unknown_select::fallback_scalar_unknown_select",
    );
}
#[test]
fn interpreter_array_unknown_select() {
    run(
        Backend::Interpret,
        &[8, 64, 65, 128, 129, 512],
        true,
        "unknown_select::interpreter_array_unknown_select",
    );
}
#[test]
fn cranelift_array_unknown_select() {
    run(
        Backend::Cranelift,
        &[8, 64],
        true,
        "unknown_select::cranelift_array_unknown_select",
    );
}
#[test]
fn fallback_array_unknown_select() {
    run(
        Backend::CraneliftFallback,
        &[65, 128, 129, 512],
        true,
        "unknown_select::fallback_array_unknown_select",
    );
}
