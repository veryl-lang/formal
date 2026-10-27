//! Finite unknown array-write indices must not silently decode to element zero.
//! Independent state comparisons check preservation; source and backend domains
//! are the explicit fixtures below.

use super::four_state::{STATES, value};
use super::four_state_array::{check, layout, source};
use super::{Backend, Simulator, analyze_state_config};
use std::process::Command;
use veryl_formal_semantics::logic4::Logic4;

fn index(base: usize, bad: Option<(usize, Logic4)>) -> Vec<Logic4> {
    let mut bits: Vec<_> = (0..16)
        .map(|i| {
            if base & (1 << i) == 0 {
                Logic4::Zero
            } else {
                Logic4::One
            }
        })
        .collect();
    if let Some((i, bit)) = bad {
        bits[i] = bit;
    }
    bits
}
fn tick(
    sim: &mut Simulator,
    state: &mut [Vec<Logic4>; 4],
    idx: &[Logic4],
    target: Option<usize>,
    backend: Backend,
    step: &mut usize,
) {
    let width = state[0].len();
    let data: Vec<_> = (0..width)
        .map(|i| STATES[(i + i / 64 + *step) % 4])
        .collect();
    sim.set("en", value(&[Logic4::One]));
    sim.set("index", value(idx));
    sim.set("data", value(&data));
    check(sim, state, backend, *step);
    let clock = sim.get_clock("clk").unwrap();
    sim.step(&clock);
    state[3] = state[0].clone();
    if let Some(target) = target {
        state[target] = data;
    }
    *step += 1;
    check(sim, state, backend, *step);
    assert_eq!(
        sim.ir.write_log_buffer.count(),
        0,
        "simulator four-state array semantics mismatch: uncleared log"
    );
}
fn compare(backend: Backend, widths: &[usize]) {
    for &width in widths {
        for forced in [false, true] {
            let (ir, _) = analyze_state_config(&source(width, true), backend, forced, true);
            layout(&ir, width);
            let (_, _, ec, ei) = ir.detailed_stats();
            eprintln!(
                "unknown write route {backend:?} width={width} forced={forced} events={ec}/{ei}"
            );
            let mut sim = Simulator::new(ir, None);
            sim.write_log_diag.enabled = true;
            let clock = sim.get_clock("clk").unwrap();
            let reset = sim.get_reset("rst").unwrap();
            let mut state = std::array::from_fn(|_| vec![Logic4::Zero; width]);
            let mut step = 0;
            for bit in 0..16 {
                if bit == 0 || bit == 8 {
                    sim.step_reset(&clock, &reset);
                    state.iter_mut().for_each(|v| v.fill(Logic4::Zero));
                    check(&mut sim, &state, backend, step);
                    for target in 0..3 {
                        tick(
                            &mut sim,
                            &mut state,
                            &index(target, None),
                            Some(target),
                            backend,
                            &mut step,
                        );
                    }
                }
                for base in [0, 1, 2, 65535] {
                    for unknown in [Logic4::X, Logic4::Z] {
                        tick(
                            &mut sim,
                            &mut state,
                            &index(base, Some((bit, unknown))),
                            None,
                            backend,
                            &mut step,
                        );
                        let target = (base + bit) % 3;
                        tick(
                            &mut sim,
                            &mut state,
                            &index(target, None),
                            Some(target),
                            backend,
                            &mut step,
                        );
                    }
                }
            }
            assert_eq!(step, 262);
            assert!(
                sim.write_log_diag.total_entries > 0,
                "unknown-write logs unused"
            );
            eprintln!(
                "unknown write complete {backend:?} width={width} forced={forced} clocks={step}"
            );
        }
    }
}
fn run(backend: Backend, widths: &[usize], harness: &str) {
    if std::env::var_os("VERYL_FORMAL_UNKNOWN_WRITE_CHILD").is_some() {
        compare(backend, widths);
        return;
    }
    let out = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", harness, "--nocapture"])
        .env("VERYL_FORMAL_UNKNOWN_WRITE_CHILD", "1")
        .output()
        .unwrap();
    let log = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(out.status.success(), "unknown-write child failed:\n{log}");
    assert!(
        log.contains("1 passed; 0 failed")
            && log.matches("unknown write complete").count() == 2 * widths.len(),
        "unknown-write child incomplete"
    );
    eprintln!("{log}");
}
#[test]
fn interpreter_unknown_write() {
    run(
        Backend::Interpret,
        &[8, 64, 65, 128, 129, 256, 512],
        "unknown_write::interpreter_unknown_write",
    );
}
#[test]
fn cranelift_unknown_write() {
    run(
        Backend::Cranelift,
        &[8, 64],
        "unknown_write::cranelift_unknown_write",
    );
}
#[test]
fn fallback_unknown_write() {
    run(
        Backend::CraneliftFallback,
        &[65, 128, 129, 256, 512],
        "unknown_write::fallback_unknown_write",
    );
}
