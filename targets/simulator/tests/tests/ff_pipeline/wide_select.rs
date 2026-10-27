//! Finite wide packed-index arithmetic and clipping with selected large values.
//! Independent positions must not wrap or truncate into a valid write.

use super::four_state::{STATES, value};
use super::signed_array::known;
use super::{Backend, Simulator, analyze_state_config};
use std::process::Command;
use veryl_formal_semantics::logic4::Logic4;

fn source(
    width: usize,
    bits: usize,
    signed: bool,
    window: usize,
    form: &str,
    array: bool,
) -> String {
    let sign = if signed { "signed " } else { "" };
    let count = if array { 3 } else { 1 };
    let name = if array { "arr[element]" } else { "q" };
    let select = match form {
        "bit" => "[index]".to_string(),
        "plus" => format!("[index+:{window}]"),
        "minus" => format!("[index-:{window}]"),
        _ => format!("[index step {window}]"),
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
        data: input logic<{width}>, patch: input logic<{window}>, index: input {sign}logic<{bits}>,
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
pub(super) fn offset(n: i128, window: usize, form: &str) -> i128 {
    match form {
        "minus" => n - (window as i128 - 1),
        "step" => n * window as i128,
        _ => n,
    }
}
pub(super) fn numbers(bits: usize, signed: bool, width: usize, window: usize) -> Vec<i128> {
    let min = if signed { -(1i128 << (bits - 1)) } else { 0 };
    let max = (1i128 << (bits - usize::from(signed))) - 1;
    let w = window as i128;
    let mut values = vec![
        min,
        min + 1,
        -w - 1,
        -w,
        -1,
        0,
        1,
        2,
        3,
        7,
        8,
        63,
        64,
        (width as i128 - 1) / w,
        width as i128 / w,
        width as i128 - 1,
        width as i128,
        width as i128 + 1,
        max - 1,
        max,
    ];
    for shift in [31, 32, 33, 58, 61, 62, 63] {
        let p = 1i128 << shift;
        values.extend([p - 1, p, p + 1, -p - 1, -p, -p + 1]);
    }
    for shift in [32, 64] {
        let q = (1i128 << shift) / w;
        values.extend([q - 1, q, q + 1, -q - 1, -q, -q + 1]);
    }
    values.retain(|n| *n >= min && *n <= max);
    values.sort();
    values.dedup();
    values
}
pub(super) fn forms(width: usize) -> Vec<(&'static str, usize)> {
    let mut result = vec![
        ("bit", 1),
        ("plus", 3),
        ("minus", 3),
        ("step", 3),
        ("plus", 8),
        ("minus", 8),
        ("step", 8),
    ];
    if width >= 64 {
        result.extend([("plus", 64), ("minus", 64), ("step", 64)]);
    }
    result
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
                "simulator wide-select output mismatch: {:?} width={} step={} element={i}",
                self.backend,
                want.len(),
                self.step
            );
        }
        assert_eq!(
            self.sim.get("previous"),
            Some(value(&self.previous)),
            "simulator wide-select output mismatch: previous step={}",
            self.step
        );
    }
    fn tick(
        &mut self,
        idx: &[Logic4],
        start: Option<i128>,
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
                if let Some(dst) = usize::try_from(start + bit as i128)
                    .ok()
                    .and_then(|i| self.state[element].get_mut(i))
                {
                    *dst = v;
                }
            }
        }
        self.step += 1;
        self.check();
        assert_eq!(
            self.sim.ir.write_log_buffer.count(),
            0,
            "simulator wide-select output mismatch: uncleared log"
        );
    }
}
fn compare(backend: Backend, widths: &[usize], array: bool) {
    let mut configurations = 0;
    let mut clocks = 0;
    for &width in widths {
        for bits in [32, 64] {
            for signed in [false, true] {
                for (form, window) in forms(width) {
                    for forced in [false, true] {
                        let route = if backend == Backend::CraneliftFallback
                            && !array
                            && (signed || form == "minus")
                        {
                            Backend::Cranelift
                        } else {
                            backend
                        };
                        let (ir, _) = analyze_state_config(
                            &source(width, bits, signed, window, form, array),
                            route,
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
                                    "wide-select allocation invalid"
                                );
                            }
                        }
                        let (_, _, ec, ei) = ir.detailed_stats();
                        eprintln!(
                            "wide-select route {backend:?} width={width} bits={bits} signed={signed} window={window} form={form} array={array} forced={forced} events={ec}/{ei}"
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
                            let starts = numbers(bits, signed, width, window);
                            for &n in &starts {
                                let start = offset(n, window, form);
                                let element = t.step % count;
                                t.tick(&known(n, bits), Some(start), element, window, false);
                            }
                            for bit in [0, bits / 2, bits - 1] {
                                for base in [0, -1] {
                                    for unknown in [Logic4::X, Logic4::Z] {
                                        let mut idx = known(base, bits);
                                        idx[bit] = unknown;
                                        let element = t.step % count;
                                        t.tick(&idx, None, element, window, false);
                                        let n = if form == "step" {
                                            (bit % width) / window
                                        } else {
                                            bit % width
                                        };
                                        let n = n as i128;
                                        let start = offset(n, window, form);
                                        t.tick(
                                            &known(n, bits),
                                            Some(start),
                                            element,
                                            window,
                                            false,
                                        );
                                    }
                                }
                            }
                        }
                        assert_eq!(
                            t.step,
                            2 * (count + numbers(bits, signed, width, window).len() + 24)
                        );
                        assert!(
                            t.sim.write_log_diag.total_entries > 0,
                            "wide-select logs unused"
                        );
                        clocks += t.step;
                        configurations += 1;
                    }
                }
            }
        }
    }
    assert_eq!(
        configurations,
        widths.iter().map(|&w| forms(w).len() * 8).sum::<usize>()
    );
    eprintln!(
        "wide-select complete {backend:?} array={array} configurations={configurations} clocks={clocks}"
    );
}
fn run(backend: Backend, widths: &[usize], array: bool, harness: &str) {
    if std::env::var_os("VERYL_FORMAL_WIDE_SELECT_CHILD").is_some() {
        compare(backend, widths, array);
        return;
    }
    let out = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", harness, "--nocapture"])
        .env("VERYL_FORMAL_WIDE_SELECT_CHILD", "1")
        .output()
        .unwrap();
    let log = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(out.status.success(), "wide-select child failed:\n{log}");
    assert!(
        log.contains("1 passed; 0 failed") && log.contains("wide-select complete"),
        "wide-select child incomplete"
    );
    eprintln!("{log}");
}
#[test]
fn interpreter_scalar_wide_select() {
    run(
        Backend::Interpret,
        &[8, 64, 65, 128, 129, 512],
        false,
        "wide_select::interpreter_scalar_wide_select",
    );
}
#[test]
fn cranelift_scalar_wide_select() {
    run(
        Backend::Cranelift,
        &[8, 64, 65, 128],
        false,
        "wide_select::cranelift_scalar_wide_select",
    );
}
#[test]
fn fallback_scalar_wide_select() {
    run(
        Backend::CraneliftFallback,
        &[129, 512],
        false,
        "wide_select::fallback_scalar_wide_select",
    );
}
#[test]
fn interpreter_array_wide_select() {
    run(
        Backend::Interpret,
        &[8, 64, 65, 128, 129, 512],
        true,
        "wide_select::interpreter_array_wide_select",
    );
}
#[test]
fn cranelift_array_wide_select() {
    run(
        Backend::Cranelift,
        &[8, 64],
        true,
        "wide_select::cranelift_array_wide_select",
    );
}
#[test]
fn fallback_array_wide_select() {
    run(
        Backend::CraneliftFallback,
        &[65, 128, 129, 512],
        true,
        "wide_select::fallback_array_wide_select",
    );
}
