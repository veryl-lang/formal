//! Finite 65-bit external FF API/raw stores, known/X/Z patterns, and selected
//! register indices. Two-state cases exercise gate wakeup; four-state statement-gate
//! configuration checks planning, not executed gate skipping.

use super::idle::{N, source};
use super::{Backend, Simulator, Value, analyze_state_config};
use std::process::Command;
use veryl_simulator::ir::write_native_value;
fn compare(backend: Backend, enabled: bool) {
    let code = source(65)
        .split("module Busy")
        .next()
        .unwrap()
        .replace("module Held", "module Top");
    for four in [false, true] {
        if four && backend == Backend::Cc {
            continue;
        }
        for forced in [false, true] {
            let (ir, trace) = analyze_state_config(&code, backend, forced, four);
            assert_eq!(
                ir.event_gate_flags.len(),
                usize::from(enabled),
                "wide external gate route"
            );
            let mut sim = Simulator::new(ir, None);
            let clock = sim.get_clock("clk").unwrap();
            let reset = sim.get_reset("rst").unwrap();
            sim.set("en", Value::new(1, 1, false));
            sim.set("data", Value::from_u128(0, 0, 65, false));
            for raw in [false, true] {
                for target in [0, 31] {
                    for (payload, mask) in [
                        (1u128 << 64, 0),
                        (0, (1u128 << 64) * u128::from(four)),
                        (1, (1u128 << 64) * u128::from(four)),
                        (1u128 << 64, (1u128 << 64) * u128::from(four)),
                        (1, ((1u128 << 64) | 1) * u128::from(four)),
                    ] {
                        if payload == 0 && mask == 0 {
                            continue;
                        }
                        sim.step_reset(&clock, &reset);
                        for _ in 0..8 {
                            sim.step(&clock);
                        }
                        if enabled && !four {
                            assert!(
                                sim.ir
                                    .event_gate_flags
                                    .iter()
                                    .any(|&o| sim.ir.comb_values[o as usize] == 1),
                                "wide external must reach idle backend={backend:?} four={four} forced={forced} raw={raw} target={target} payload={payload} mask={mask}, header={:?}",
                                sim.ir
                                    .event_gate_flags
                                    .iter()
                                    .map(|&o| &sim.ir.comb_values[o as usize..o as usize + 16])
                                    .collect::<Vec<_>>()
                            );
                        }
                        let (&id, var) = sim
                            .ir
                            .module_variables
                            .variables
                            .iter()
                            .find(|(_, v)| v.path.to_string() == format!("r{target}"))
                            .unwrap();
                        let ptr = var.current_values[0];
                        let bytes = var.native_bytes;
                        assert_eq!(bytes, 16);
                        let base = sim.ir.ff_values.as_ptr() as usize;
                        let length = bytes * if four { 2 } else { 1 };
                        assert!(
                            (base..base + sim.ir.ff_values.len()).contains(&(ptr as usize))
                                && ptr as usize + length <= base + sim.ir.ff_values.len(),
                            "wide external FF allocation"
                        );
                        let seed = Value::from_u128(payload, mask, 65, false);
                        if raw {
                            unsafe {
                                write_native_value(ptr, bytes, four, &seed);
                            }
                            sim.mark_ff_written();
                        } else {
                            sim.set_var_by_id(&id, seed);
                        }
                        let mut state = [(0u128, 0u128); N];
                        state[target] = (payload, mask);
                        let logical = (1u128 << 65) - 1;
                        for cycle in 0..=5 {
                            for (i, &(p, m)) in state.iter().enumerate() {
                                assert_eq!(
                                    sim.get(&format!("o{i}")),
                                    Some(Value::from_u128(p, m, 65, false)),
                                    "simulator wide external output mismatch: {backend:?} four={four} gate={enabled} forced={forced} raw={raw} target={target} cycle={cycle} reg={i}"
                                );
                            }
                            if cycle == 5 {
                                break;
                            }
                            let old = state;
                            for i in 0..N {
                                let rhs = if i == 0 { (0, 0) } else { old[i - 1] };
                                state[i] = if old[i].1 != 0 || rhs.1 != 0 {
                                    (0, logical)
                                } else {
                                    ((old[i].0 + rhs.0) & logical, 0)
                                };
                            }
                            sim.step(&clock);
                        }
                    }
                }
            }
            if backend == Backend::Cc {
                assert!(
                    trace.lock().unwrap().calls > 0,
                    "wide external AOT dispatch"
                );
            }
        }
    }
    eprintln!("wide external complete backend={backend:?} gate={enabled}");
}
fn run(backend: Backend, harness: &str) {
    if let Ok(mode) = std::env::var("VERYL_FORMAL_EXTERNAL_WIDE_CHILD") {
        compare(backend, mode == "1");
        return;
    }
    for mode in ["0", "1"] {
        let output = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", harness, "--nocapture"])
            .env("VERYL_FORMAL_EXTERNAL_WIDE_CHILD", mode)
            .env("VERYL_EVENT_GATE", mode)
            .env("VERYL_EVENT_GATE_CHECK", "0")
            .output()
            .unwrap();
        let log = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.status.success(), "wide external child failed: {log}");
        assert!(
            log.contains("wide external complete") && log.contains("1 passed; 0 failed"),
            "wide external completion"
        );
        eprintln!("{log}");
    }
}
#[test]
fn interpreter_external_wide() {
    run(
        Backend::Interpret,
        "external_wide::interpreter_external_wide",
    );
}
#[test]
fn cranelift_external_wide() {
    run(Backend::Cranelift, "external_wide::cranelift_external_wide");
}
#[test]
fn aot_c_external_wide() {
    run(Backend::Cc, "external_wide::aot_c_external_wide");
}
