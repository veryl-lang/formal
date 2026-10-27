//! Finite wide four-state array allocation and stores. Selected dynamic >128-bit
//! stores explicitly use interpreter fallback under Cranelift configuration; those
//! cases establish fallback behavior, not compiled execution of the store.

use super::four_state::{STATES, value};
use super::{Backend, Simulator, Value, analyze_state_config};
use std::process::Command;
use veryl_formal_semantics::logic4::{Logic4, logic_from_planes};

pub(super) fn source(width: usize, dynamic: bool) -> String {
    let store = if dynamic {
        "arr[index] = data;"
    } else {
        "if index == 0 {arr[0] = data;} else if index == 1 {arr[1] = data;} else if index == 2 {arr[2] = data;}"
    };
    format!(
        r#"
    module Top (
        clk: input clock, rst: input reset, en: input logic,
        index: input logic<16>, data: input logic<{width}>,
        out0: output logic<{width}>, out1: output logic<{width}>,
        out2: output logic<{width}>, previous: output logic<{width}>,
    ) {{
        var arr: logic<{width}> [3]; var prev: logic<{width}>;
        always_ff {{
            if_reset {{arr[0] = 0; arr[1] = 0; arr[2] = 0; prev = 0;}}
            else if en {{{store} prev = arr[0];}}
        }}
        assign out0 = arr[0]; assign out1 = arr[1]; assign out2 = arr[2];
        assign previous = prev;
    }}
    "#
    )
}
pub(super) fn layout(ir: &veryl_simulator::ir::Ir, width: usize) {
    let arr = ir
        .module_variables
        .variables
        .values()
        .find(|v| v.path.to_string() == "arr")
        .expect("array variable absent");
    assert_eq!(arr.current_values.len(), 3);
    assert_eq!(arr.width, width);
    let span = 2 * arr.native_bytes;
    let ff = ir.ff_values.as_ptr() as usize..ir.ff_values.as_ptr() as usize + ir.ff_values.len();
    let comb =
        ir.comb_values.as_ptr() as usize..ir.comb_values.as_ptr() as usize + ir.comb_values.len();
    for &p in arr.current_values.iter().chain(&arr.next_values) {
        let start = p as usize;
        assert!(
            [&ff, &comb].iter().any(
                |r| start >= r.start && start.checked_add(span).is_some_and(|end| end <= r.end)
            ),
            "simulator four-state array semantics mismatch: payload/mask allocation too small"
        );
    }
    let mut intervals: Vec<_> = arr
        .current_values
        .iter()
        .map(|&p| (p as usize, p as usize + span))
        .collect();
    intervals.sort_unstable();
    assert!(
        intervals.windows(2).all(|pair| pair[0].1 <= pair[1].0),
        "simulator four-state array semantics mismatch: overlapping element storage"
    );
}
pub(super) fn check(
    sim: &mut Simulator,
    expected: &[Vec<Logic4>; 4],
    backend: Backend,
    step: usize,
) {
    for (port, want) in ["out0", "out1", "out2", "previous"].iter().zip(expected) {
        let got = sim.get(port).unwrap();
        let p = got.payload();
        let m = got.mask_xz();
        let bits: Vec<_> = (0..want.len())
            .map(|i| logic_from_planes(p.bit(i as u64), m.bit(i as u64)))
            .collect();
        assert_eq!(
            bits,
            *want,
            "simulator four-state array semantics mismatch: {backend:?} width={} step={step} port={port}",
            want.len()
        );
        assert!(
            p.bits() <= want.len() as u64
                && m.bits() <= want.len() as u64
                && got.width() == want.len()
                && !got.signed(),
            "simulator four-state array semantics mismatch: metadata or padding"
        );
    }
}
fn compare(backend: Backend, dynamic: bool) {
    for width in [129usize, 256, 512] {
        for forced in [false, true] {
            let (ir, _) = analyze_state_config(&source(width, dynamic), backend, forced, true);
            layout(&ir, width);
            let mut sim = Simulator::new(ir, None);
            let clock = sim.get_clock("clk").unwrap();
            let reset = sim.get_reset("rst").unwrap();
            let mut state = std::array::from_fn(|_| vec![Logic4::Zero; width]);
            let mut step = 0;
            for case in 0..12 {
                if case == 0 || case == 6 {
                    sim.step_reset(&clock, &reset);
                    state.iter_mut().for_each(|v| v.fill(Logic4::Zero));
                    check(&mut sim, &state, backend, step);
                }
                let index = [0, 1, 2, 3, 4, 65535][case % 6];
                for (hold, en) in [Logic4::One, Logic4::Zero, Logic4::X, Logic4::Z]
                    .into_iter()
                    .enumerate()
                {
                    let data: Vec<_> = (0..width)
                        .map(|i| STATES[(i + i / 64 + case + hold) % 4])
                        .collect();
                    sim.set("data", value(&data));
                    sim.set("index", Value::new(index as u64, 16, false));
                    sim.set("en", value(&[en]));
                    check(&mut sim, &state, backend, step);
                    sim.step(&clock);
                    if en == Logic4::One {
                        state[3] = state[0].clone();
                        if index < 3 {
                            state[index] = data;
                        }
                    }
                    step += 1;
                    check(&mut sim, &state, backend, step);
                    assert_eq!(
                        sim.ir.write_log_buffer.count(),
                        0,
                        "simulator four-state array semantics mismatch: live log after commit"
                    );
                }
            }
            eprintln!(
                "array route backend={backend:?} dynamic={dynamic} width={width} forced={forced} steps={step}"
            );
        }
    }
}
fn run(backend: Backend, dynamic: bool, harness: &str) {
    if std::env::var_os("VERYL_FORMAL_ARRAY_CHILD").is_some() {
        compare(backend, dynamic);
        return;
    }
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", harness, "--nocapture"])
        .env("VERYL_FORMAL_ARRAY_CHILD", "1")
        .output()
        .unwrap();
    let log = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.status.success(), "array child failed:\n{log}");
    assert!(
        log.contains("1 passed; 0 failed") && log.contains("steps=48"),
        "array child did not complete"
    );
    eprintln!("{log}");
}
#[test]
fn interpreter_static_four_state_array() {
    run(
        Backend::Interpret,
        false,
        "four_state_array::interpreter_static_four_state_array",
    );
}
#[test]
fn cranelift_static_four_state_array() {
    run(
        Backend::Cranelift,
        false,
        "four_state_array::cranelift_static_four_state_array",
    );
}
#[test]
fn interpreter_dynamic_four_state_array() {
    run(
        Backend::Interpret,
        true,
        "four_state_array::interpreter_dynamic_four_state_array",
    );
}
#[test]
fn cranelift_fallback_dynamic_four_state_array() {
    run(
        Backend::CraneliftFallback,
        true,
        "four_state_array::cranelift_fallback_dynamic_four_state_array",
    );
}
