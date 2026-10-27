//! Finite event-gate enabled/disabled execution with idle holds and wakeups. Child
//! processes isolate policy configuration. No general event-scheduler theorem.

use super::{Backend, Simulator, Value, analyze_config, check, check_commit, value};
use std::process::Command;
use veryl_formal_semantics::words;

pub(super) const N: usize = 64;

pub(super) fn source(width: usize) -> String {
    let mut ports = String::new();
    let mut declarations = String::new();
    let mut resets = String::new();
    let mut updates = String::new();
    let mut assigns = String::new();
    let mut connections = String::new();
    for i in 0..N {
        ports.push_str(&format!("o{i}: output logic<{width}>,\n"));
        declarations.push_str(&format!("var r{i}: logic<{width}>;\n"));
        resets.push_str(&format!("r{i} = 0;\n"));
        updates.push_str(&if i == 0 {
            "r0 = r0 + data;\n".into()
        } else {
            format!("r{i} = r{i} + r{};\n", i - 1)
        });
        assigns.push_str(&format!("assign o{i} = r{i};\n"));
        connections.push_str(&format!("o{i}: o{i},\n"));
    }
    format!(
        r#"
    module Held (
        clk: input clock, rst: input reset, en: input logic, data: input logic<{width}>,
        {ports}
    ) {{
        {declarations}
        always_ff {{if_reset {{{resets}}} else if en {{{updates}}}}}
        {assigns}
    }}
    module Busy (clk: input clock, rst: input reset, out: output logic<16>) {{
        var count: logic<16>;
        always_ff {{if_reset {{count = 0;}} else {{count = count + 1;}}}}
        assign out = count;
    }}
    module Top (
        clk: input clock, rst: input reset, rst_busy: input reset,
        en: input logic, data: input logic<{width}>, busy: output logic<16>,
        {ports}
    ) {{
        inst held: Held (clk, rst, en, data, {connections});
        inst active: Busy (clk, rst: rst_busy, out: busy);
    }}
    "#
    )
}

fn add(a: &[u8], b: &[u8], width: usize) -> Vec<u8> {
    let mut result = words::add_sub(a, b, false);
    if !width.is_multiple_of(8) {
        *result.last_mut().unwrap() &= (1 << (width % 8)) - 1;
    }
    result
}

fn check_state(
    sim: &mut Simulator,
    state: &[Vec<u8>],
    busy: u16,
    width: usize,
    backend: Backend,
    cycle: usize,
) {
    for (i, bytes) in state.iter().enumerate() {
        check(sim, &format!("o{i}"), bytes, width, backend, cycle);
    }
    check(sim, "busy", &busy.to_le_bytes(), 16, backend, cycle);
}

fn skip_state(sim: &Simulator) -> bool {
    // Gate state ABI: idle at 0, off at 2, streak at 4. An idle, enabled
    // gate with streak zero distinguishes Skip from the incrementing Run.
    sim.ir.event_gate_flags.iter().any(|&off| {
        let off = off as usize;
        let header = &sim.ir.comb_values[off..off + 16];
        header[0] == 1 && header[2] == 0 && header[4..8] == [0; 4]
    })
}

fn compare(backend: Backend, enabled: bool) {
    for width in [16usize, 65] {
        for forced in [false, true] {
            let (ir, trace) = analyze_config(&source(width), backend, forced);
            assert_eq!(
                !ir.event_gate_flags.is_empty(),
                enabled,
                "idle gate route missing or unexpectedly enabled"
            );
            let mut sim = Simulator::new(ir, None);
            let clock = sim.get_clock("clk").unwrap();
            let reset = sim.get_reset("rst").unwrap();
            let busy_reset = sim.get_reset("rst_busy").unwrap();
            sim.set("en", Value::new(0, 1, false));
            sim.set("data", Value::new(0, width, false));
            sim.step_reset(&clock, &busy_reset);
            sim.step_reset(&clock, &reset);
            let mut state = vec![vec![0; width.div_ceil(8)]; N];
            let mut busy = 1u16;
            let mut cycle = 0;
            let mut witnesses = 0;
            check_state(&mut sim, &state, busy, width, backend, cycle);
            for epoch in 0..2 {
                for (en, input, clocks) in [
                    (false, 0u8, 8),
                    (false, 0x55, 8),
                    (true, 0x55, 4),
                    (false, 0xaa, 8),
                    (true, 0xf0, 3),
                ] {
                    let mut data = vec![input; width.div_ceil(8)];
                    if !width.is_multiple_of(8) {
                        *data.last_mut().unwrap() &= (1 << (width % 8)) - 1;
                    }
                    sim.set("en", Value::new(u64::from(en), 1, false));
                    sim.set("data", value(&data, width));
                    check_state(&mut sim, &state, busy, width, backend, cycle);
                    for _ in 0..clocks {
                        let calls = trace.lock().unwrap().calls;
                        sim.step(&clock);
                        check_commit(&sim, &trace, backend, calls);
                        if en {
                            let old = state.clone();
                            state[0] = add(&old[0], &data, width);
                            for i in 1..N {
                                state[i] = add(&old[i], &old[i - 1], width);
                            }
                        }
                        busy = busy.wrapping_add(1);
                        cycle += 1;
                        check_state(&mut sim, &state, busy, width, backend, cycle);
                    }
                    if !en && enabled {
                        assert!(skip_state(&sim), "idle gate did not reach Skip state");
                        witnesses += 1;
                    }
                }
                // Reset must reach a previously idle subtree while the busy
                // sibling keeps receiving clock edges.
                sim.set("en", Value::new(0, 1, false));
                for _ in 0..4 {
                    let calls = trace.lock().unwrap().calls;
                    sim.step(&clock);
                    check_commit(&sim, &trace, backend, calls);
                    busy = busy.wrapping_add(1);
                    cycle += 1;
                    check_state(&mut sim, &state, busy, width, backend, cycle);
                }
                if enabled {
                    assert!(skip_state(&sim), "pre-reset gate was not idle");
                }
                let calls = trace.lock().unwrap().calls;
                sim.step_reset(&clock, &reset);
                check_commit(&sim, &trace, backend, calls);
                state.iter_mut().for_each(|r| r.fill(0));
                busy = busy.wrapping_add(1);
                check_state(&mut sim, &state, busy, width, backend, cycle);
                eprintln!(
                    "idle route backend={backend:?} gate={enabled} width={width} forced={forced} epoch={epoch} witnesses={witnesses}"
                );
            }
        }
    }
}

fn run(backend: Backend, harness: &str) {
    if let Ok(mode) = std::env::var("VERYL_FORMAL_IDLE_CHILD") {
        compare(backend, mode == "1");
        return;
    }
    for mode in ["0", "1"] {
        let output = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", harness, "--nocapture"])
            .env("VERYL_FORMAL_IDLE_CHILD", mode)
            .env("VERYL_EVENT_GATE", mode)
            .env("VERYL_EVENT_GATE_CHECK", "0")
            .output()
            .unwrap();
        let log = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            output.status.success(),
            "idle gate child failed mode={mode}:\n{log}"
        );
        assert!(
            log.contains("1 passed; 0 failed") && log.contains("epoch=1"),
            "idle child did not complete its comparisons"
        );
        eprintln!("{log}");
    }
}

#[test]
fn interpreter_idle_gates() {
    run(Backend::Interpret, "idle::interpreter_idle_gates");
}
#[test]
fn cranelift_idle_gates() {
    run(Backend::Cranelift, "idle::cranelift_idle_gates");
}
#[test]
fn aot_c_idle_gates() {
    run(Backend::Cc, "idle::aot_c_idle_gates");
}
