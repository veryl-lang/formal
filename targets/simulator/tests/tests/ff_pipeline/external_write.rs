//! Finite external FF writes must invalidate selected gated consumers. Actual
//! execution traces distinguish compiled dispatch from fallback.

use super::idle::{N, source};
use super::{Backend, Simulator, Value, analyze_config, check_commit};
use std::process::Command;
use veryl_simulator::ir::write_native_value;

fn compare(backend: Backend, enabled: bool) {
    let full = source(16);
    let code = full
        .split("module Busy")
        .next()
        .unwrap()
        .replace("module Held", "module Top");
    for forced in [false, true] {
        let (ir, trace) = analyze_config(&code, backend, forced);
        assert_eq!(
            ir.event_gate_flags.len(),
            usize::from(enabled),
            "external-write gate route absent"
        );
        let mut sim = Simulator::new(ir, None);
        let clock = sim.get_clock("clk").unwrap();
        let reset = sim.get_reset("rst").unwrap();
        sim.set("en", Value::new(1, 1, false));
        sim.set("data", Value::new(0, 16, false));
        for raw in [false, true] {
            for target in [0, 31] {
                sim.step_reset(&clock, &reset);
                let mut state = [0u16; N];
                for _ in 0..8 {
                    sim.step(&clock);
                }
                if enabled {
                    assert!(
                        sim.ir.event_gate_flags.iter().any(|&o| {
                            let h = &sim.ir.comb_values[o as usize..o as usize + 16];
                            h[0] == 1 && h[2] == 0 && h[4..8] == [0; 4]
                        }),
                        "external-write target did not reach Skip"
                    );
                }
                let path = format!("r{target}");
                let (&id, var) = sim
                    .ir
                    .module_variables
                    .variables
                    .iter()
                    .find(|(_, v)| v.path.to_string() == path)
                    .expect("root FF absent");
                let ptr = var.current_values[0];
                let nb = var.native_bytes;
                let base = sim.ir.ff_values.as_ptr() as usize;
                assert!(
                    (base..base + sim.ir.ff_values.len()).contains(&(ptr as usize))
                        && (ptr as usize + nb) <= base + sim.ir.ff_values.len(),
                    "external-write target must be FF storage"
                );
                assert_eq!(nb, 2);
                let seed = Value::new(0xfffe, 16, false);
                if raw {
                    unsafe {
                        write_native_value(ptr, nb, false, &seed);
                    }
                    sim.mark_ff_written();
                } else {
                    sim.set_var_by_id(&id, seed);
                }
                state[target] = 0xfffe;
                for cycle in 0..=5 {
                    for (i, &v) in state.iter().enumerate() {
                        assert_eq!(
                            sim.get(&format!("o{i}")),
                            Some(Value::new(v as u64, 16, false)),
                            "simulator external-write output mismatch: {backend:?} forced={forced} raw={raw} target={target} cycle={cycle} register={i}"
                        );
                    }
                    if cycle == 5 {
                        break;
                    }
                    let calls = trace.lock().unwrap().calls;
                    sim.step(&clock);
                    check_commit(&sim, &trace, backend, calls);
                    let old = state;
                    for i in 1..N {
                        state[i] = old[i].wrapping_add(old[i - 1]);
                    }
                }
                eprintln!(
                    "external-write complete backend={backend:?} forced={forced} raw={raw} target={target}"
                );
            }
        }
    }
}
fn run(backend: Backend, harness: &str) {
    if let Ok(mode) = std::env::var("VERYL_FORMAL_EXTERNAL_CHILD") {
        compare(backend, mode == "1");
        return;
    }
    for mode in ["0", "1"] {
        let out = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", harness, "--nocapture"])
            .env("VERYL_FORMAL_EXTERNAL_CHILD", mode)
            .env("VERYL_EVENT_GATE", mode)
            .env("VERYL_EVENT_GATE_CHECK", "0")
            .output()
            .unwrap();
        let log = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(
            out.status.success(),
            "external-write child failed mode={mode}:\n{log}"
        );
        assert!(
            log.contains("1 passed; 0 failed")
                && log.matches("external-write complete").count() == 8,
            "external-write child incomplete"
        );
        eprintln!("{log}");
    }
}
#[test]
fn interpreter_external_write() {
    run(
        Backend::Interpret,
        "external_write::interpreter_external_write",
    );
}
#[test]
fn cranelift_external_write() {
    run(
        Backend::Cranelift,
        "external_write::cranelift_external_write",
    );
}
#[test]
fn aot_c_external_write() {
    run(Backend::Cc, "external_write::aot_c_external_write");
}
