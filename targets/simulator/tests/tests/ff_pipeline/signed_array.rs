//! Finite signed array-index boundaries; negative and outside indices must not
//! alias valid storage. Selected widths and payloads are ordinary comparisons.

use super::four_state::{STATES, value};
use super::{Backend, Simulator, analyze_state_config, check_commit};
use std::process::Command;
use veryl_formal_semantics::logic4::Logic4;

const SHAPES: [(usize, usize); 6] = [(1, 3), (2, 3), (4, 8), (4, 12), (16, 3), (64, 3)];

fn source(width: usize, bits: usize, len: usize) -> String {
    let ports: String = (0..len)
        .map(|i| format!("o{i}: output logic<{width}>, "))
        .collect();
    let reset: String = (0..len)
        .map(|i| format!("arr[{i}] = {}; ", i + 1))
        .collect();
    let assigns: String = (0..len)
        .map(|i| format!("assign o{i} = arr[{i}]; "))
        .collect();
    format!(
        r#"
    module Top (
        clk: input clock, rst: input reset, data: input logic<{width}>,
        write_index: input signed logic<{bits}>, read_index: input signed logic<{bits}>,
        selected: output logic<{width}>, previous: output logic<{width}>, {ports}
    ) {{
        var arr: logic<{width}> [{len}]; var prev: logic<{width}>;
        always_ff {{
            if_reset {{{reset} prev = 0;}}
            else {{arr[write_index] = data; prev = arr[0];}}
        }}
        assign selected = arr[read_index]; assign previous = prev; {assigns}
    }}
    "#
    )
}
pub(super) fn known(n: i128, bits: usize) -> Vec<Logic4> {
    (0..bits)
        .map(|i| {
            if i >= 128 || (n as u128) & (1u128 << i) == 0 {
                Logic4::Zero
            } else {
                Logic4::One
            }
        })
        .collect()
}
fn cases(bits: usize, len: usize, four: bool) -> Vec<(Vec<Logic4>, Option<usize>)> {
    let min = -(1i128 << (bits - 1));
    let max = (1i128 << (bits - 1)) - 1;
    let mut numbers: Vec<_> = if bits <= 4 {
        (min..=max).collect()
    } else {
        [
            min,
            min + 1,
            -(len as i128),
            -2,
            -1,
            0,
            1,
            len as i128 - 1,
            len as i128,
            len as i128 + 1,
            max - 1,
            max,
        ]
        .into_iter()
        .collect()
    };
    if bits == 64 {
        numbers.extend([1i128 << 32, (1i128 << 32) + 1]);
    }
    numbers.sort();
    numbers.dedup();
    let mut result: Vec<_> = numbers
        .into_iter()
        .map(|n| {
            (
                known(n, bits),
                (n >= 0 && n < len as i128).then_some(n as usize),
            )
        })
        .collect();
    if four {
        for bit in [0, bits / 2, bits - 1] {
            for unknown in [Logic4::X, Logic4::Z] {
                for n in [0, -1] {
                    let mut idx = known(n, bits);
                    idx[bit] = unknown;
                    result.push((idx, None));
                }
            }
        }
    }
    result
}
fn check(sim: &mut Simulator, port: &str, want: &[Logic4], backend: Backend, step: usize) {
    assert_eq!(
        sim.get(port),
        Some(value(want)),
        "simulator signed-array semantics mismatch: {backend:?} port={port} width={} step={step}",
        want.len()
    );
}
fn state_check(
    sim: &mut Simulator,
    state: &[Vec<Logic4>],
    previous: &[Logic4],
    backend: Backend,
    step: usize,
) {
    for (i, want) in state.iter().enumerate() {
        check(sim, &format!("o{i}"), want, backend, step);
    }
    check(sim, "previous", previous, backend, step);
}
fn compare(backend: Backend, four: bool, widths: &[usize]) {
    let mut total = 0;
    let mut configurations = 0;
    for &width in widths {
        for (bits, len) in SHAPES {
            for forced in [false, true] {
                let (ir, trace) =
                    analyze_state_config(&source(width, bits, len), backend, forced, four);
                let arr = ir
                    .module_variables
                    .variables
                    .values()
                    .find(|v| v.path.to_string() == "arr")
                    .unwrap();
                let span = arr.native_bytes * if four { 2 } else { 1 };
                assert_eq!(arr.current_values.len(), len);
                let ff = ir.ff_values.as_ptr() as usize
                    ..ir.ff_values.as_ptr() as usize + ir.ff_values.len();
                let comb = ir.comb_values.as_ptr() as usize
                    ..ir.comb_values.as_ptr() as usize + ir.comb_values.len();
                for &ptr in arr.current_values.iter().chain(&arr.next_values) {
                    assert!(
                        [&ff, &comb].iter().any(|r| ptr as usize >= r.start
                            && (ptr as usize)
                                .checked_add(span)
                                .is_some_and(|end| end <= r.end)),
                        "signed-array allocation invalid"
                    );
                }
                let mut intervals: Vec<_> = arr
                    .current_values
                    .iter()
                    .map(|&p| (p as usize, p as usize + span))
                    .collect();
                intervals.sort_unstable();
                assert!(
                    intervals.windows(2).all(|p| p[0].1 <= p[1].0),
                    "signed-array elements overlap"
                );
                let (cc, ci, ec, ei) = ir.detailed_stats();
                if backend != Backend::Interpret {
                    assert!(cc > 0 && ci == 0, "signed dynamic reader fell back");
                }
                eprintln!(
                    "signed-array route {backend:?} four={four} width={width} bits={bits} len={len} forced={forced} comb={cc}/{ci} events={ec}/{ei}"
                );
                let mut sim = Simulator::new(ir, None);
                sim.write_log_diag.enabled = true;
                let clock = sim.get_clock("clk").unwrap();
                let reset = sim.get_reset("rst").unwrap();
                let schedule = cases(bits, len, four);
                let mut step = 0;
                for epoch in 0..2 {
                    let calls = trace.lock().unwrap().calls;
                    sim.step_reset(&clock, &reset);
                    check_commit(&sim, &trace, backend, calls);
                    let mut state: Vec<_> =
                        (0..len).map(|i| known((i + 1) as i128, width)).collect();
                    let mut prev = vec![Logic4::Zero; width];
                    state_check(&mut sim, &state, &prev, backend, step);
                    for (idx, target) in &schedule {
                        let data: Vec<_> = (0..width)
                            .map(|i| {
                                if four {
                                    STATES[(i + i / 64 + step + epoch) % 4]
                                } else if (i + i / 64 + step + epoch) % 2 == 0 {
                                    Logic4::Zero
                                } else {
                                    Logic4::One
                                }
                            })
                            .collect();
                        sim.set("write_index", value(idx));
                        sim.set("data", value(&data));
                        state_check(&mut sim, &state, &prev, backend, step);
                        let calls = trace.lock().unwrap().calls;
                        sim.step(&clock);
                        check_commit(&sim, &trace, backend, calls);
                        prev = state[0].clone();
                        if let Some(target) = *target {
                            state[target] = data;
                        }
                        step += 1;
                        state_check(&mut sim, &state, &prev, backend, step);
                        for (read, want) in &schedule {
                            sim.set("read_index", value(read));
                            let invalid = vec![if four { Logic4::X } else { Logic4::Zero }; width];
                            check(
                                &mut sim,
                                "selected",
                                want.and_then(|i| state.get(i)).unwrap_or(&invalid),
                                backend,
                                step,
                            );
                        }
                        state_check(&mut sim, &state, &prev, backend, step);
                        assert_eq!(
                            sim.ir.write_log_buffer.count(),
                            0,
                            "simulator signed-array semantics mismatch: uncleared log"
                        );
                    }
                }
                assert!(
                    sim.write_log_diag.total_entries > 0,
                    "signed-array logs unused"
                );
                total += step;
                configurations += 1;
            }
        }
    }
    assert_eq!(configurations, widths.len() * SHAPES.len() * 2);
    eprintln!(
        "signed-array complete {backend:?} four={four} configurations={configurations} clocks={total}"
    );
}
fn run(backend: Backend, four: bool, widths: &[usize], harness: &str) {
    if std::env::var_os("VERYL_FORMAL_SIGNED_ARRAY_CHILD").is_some() {
        compare(backend, four, widths);
        return;
    }
    let out = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", harness, "--nocapture"])
        .env("VERYL_FORMAL_SIGNED_ARRAY_CHILD", "1")
        .output()
        .unwrap();
    let log = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(out.status.success(), "signed-array child failed:\n{log}");
    assert!(
        log.contains("1 passed; 0 failed") && log.contains("signed-array complete"),
        "signed-array child incomplete"
    );
    eprintln!("{log}");
}
#[test]
fn interpreter_signed_array() {
    run(
        Backend::Interpret,
        false,
        &[16, 129],
        "signed_array::interpreter_signed_array",
    );
}
#[test]
fn cranelift_signed_array() {
    run(
        Backend::Cranelift,
        false,
        &[16, 129],
        "signed_array::cranelift_signed_array",
    );
}
#[test]
fn aot_c_signed_array() {
    run(
        Backend::Cc,
        false,
        &[16, 129],
        "signed_array::aot_c_signed_array",
    );
}
#[test]
fn interpreter_four_signed_array() {
    run(
        Backend::Interpret,
        true,
        &[8, 64, 65, 129, 512],
        "signed_array::interpreter_four_signed_array",
    );
}
#[test]
fn cranelift_four_signed_array() {
    run(
        Backend::Cranelift,
        true,
        &[8, 64],
        "signed_array::cranelift_four_signed_array",
    );
}
#[test]
fn fallback_four_signed_array() {
    run(
        Backend::CraneliftFallback,
        true,
        &[65, 129, 512],
        "signed_array::fallback_four_signed_array",
    );
}
