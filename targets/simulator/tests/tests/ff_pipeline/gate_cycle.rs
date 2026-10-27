//! Finite event-gate transitions including re-arm, period doubling, auto-off, skip
//! reset, and the maximum-period boundary. Selected long runs do not prove arbitrary
//! histories; policy planning and actual gate execution are distinguished.

use super::idle::{N, source};
use super::{Backend, Simulator, Value, analyze_config, check_commit};
use std::process::Command;

#[derive(Debug, PartialEq, Eq)]
struct Header {
    idle: bool,
    off: bool,
    count: u32,
    period: u32,
}
fn header(sim: &Simulator, offset: usize) -> Header {
    let b = &sim.ir.comb_values[offset..offset + 16];
    Header {
        idle: b[0] != 0,
        off: b[2] != 0,
        count: u32::from_le_bytes(b[4..8].try_into().unwrap()),
        period: u32::from_le_bytes(b[8..12].try_into().unwrap()),
    }
}
fn policy(
    sim: &Simulator,
    offset: Option<usize>,
    idle: bool,
    off: bool,
    count: u32,
    period: u32,
    label: &str,
) {
    if let Some(offset) = offset {
        assert_eq!(
            header(sim, offset),
            Header {
                idle,
                off,
                count,
                period
            },
            "simulator gate-cycle policy mismatch: {label}"
        );
    }
}
struct Model {
    held: [u16; N],
    busy: u16,
    cycle: usize,
}
impl Model {
    fn check(&self, sim: &mut Simulator, backend: Backend) {
        for (i, &word) in self.held.iter().enumerate() {
            assert_eq!(
                sim.get(&format!("o{i}")),
                Some(Value::new(word as u64, 16, false)),
                "simulator gate-cycle output mismatch: {backend:?} cycle={} register={i}",
                self.cycle
            );
        }
        assert_eq!(
            sim.get("busy"),
            Some(Value::new(self.busy as u64, 16, false)),
            "simulator gate-cycle output mismatch: busy cycle={}",
            self.cycle
        );
    }
    fn tick(
        &mut self,
        sim: &mut Simulator,
        trace: &std::sync::Arc<std::sync::Mutex<super::Trace>>,
        backend: Backend,
        en: bool,
        data: u16,
    ) {
        let clock = sim.get_clock("clk").unwrap();
        let calls = trace.lock().unwrap().calls;
        sim.step(&clock);
        check_commit(sim, trace, backend, calls);
        if en {
            let old = self.held;
            self.held[0] = old[0].wrapping_add(data);
            for i in 1..N {
                self.held[i] = old[i].wrapping_add(old[i - 1]);
            }
        }
        self.busy = self.busy.wrapping_add(1);
        self.cycle += 1;
        self.check(sim, backend);
    }
}
fn inputs(sim: &mut Simulator, model: &Model, backend: Backend, en: bool, data: u16) {
    sim.set("en", Value::new(u64::from(en), 1, false));
    sim.set("data", Value::new(data as u64, 16, false));
    model.check(sim, backend);
}
fn compare(backend: Backend, enabled: bool) {
    for forced in [false, true] {
        let (ir, trace) = analyze_config(&source(16), backend, forced);
        assert_eq!(
            !ir.event_gate_flags.is_empty(),
            enabled,
            "gate-cycle route mismatch"
        );
        let mut sim = Simulator::new(ir, None);
        let clock = sim.get_clock("clk").unwrap();
        let reset = sim.get_reset("rst").unwrap();
        let busy_reset = sim.get_reset("rst_busy").unwrap();
        sim.set("en", Value::new(0, 1, false));
        sim.set("data", Value::new(1, 16, false));
        sim.step_reset(&clock, &busy_reset);
        sim.step_reset(&clock, &reset);
        let mut model = Model {
            held: [0; N],
            busy: 1,
            cycle: 0,
        };
        model.check(&mut sim, backend);
        for _ in 0..8 {
            model.tick(&mut sim, &trace, backend, false, 1);
        }
        let offset = if enabled {
            Some(
                sim.ir
                    .event_gate_flags
                    .iter()
                    .map(|&v| v as usize)
                    .find(|&o| {
                        header(&sim, o)
                            == Header {
                                idle: true,
                                off: false,
                                count: 0,
                                period: 0,
                            }
                    })
                    .expect("idle gate-cycle target absent"),
            )
        } else {
            None
        };
        inputs(&mut sim, &model, backend, true, 1);
        for i in 1..=64 {
            model.tick(&mut sim, &trace, backend, true, 1);
            if i == 63 {
                policy(&sim, offset, false, false, 63, 0, "first dirty check 63");
            }
        }
        policy(&sim, offset, false, true, 0, 1024, "first auto-off 64");
        for i in 1..=1024 {
            model.tick(&mut sim, &trace, backend, true, 1);
            if i == 1023 {
                policy(&sim, offset, false, true, 1023, 1024, "before first re-arm");
            }
        }
        policy(&sim, offset, false, false, 0, 1024, "first re-arm 1024");
        for i in 1..=64 {
            model.tick(&mut sim, &trace, backend, true, 1);
            if i == 63 {
                policy(
                    &sim,
                    offset,
                    false,
                    false,
                    63,
                    1024,
                    "second dirty check 63",
                );
            }
        }
        policy(
            &sim,
            offset,
            false,
            true,
            0,
            2048,
            "second auto-off doubles period",
        );
        inputs(&mut sim, &model, backend, false, 1);
        for i in 1..=2048 {
            model.tick(&mut sim, &trace, backend, false, 1);
            if i == 2047 {
                policy(
                    &sim,
                    offset,
                    false,
                    true,
                    2047,
                    2048,
                    "before second re-arm",
                );
            }
        }
        policy(&sim, offset, false, false, 0, 2048, "second re-arm 2048");
        model.tick(&mut sim, &trace, backend, false, 1);
        policy(&sim, offset, true, false, 1, 2048, "idle run after re-arm");
        model.tick(&mut sim, &trace, backend, false, 1);
        policy(
            &sim,
            offset,
            true,
            false,
            0,
            0,
            "skip resets streak and period",
        );
        inputs(&mut sim, &model, backend, true, 3);
        for i in 1..=3 {
            model.tick(&mut sim, &trace, backend, true, 3);
            policy(&sim, offset, false, false, i, 0, "wake after long idle");
        }
        inputs(&mut sim, &model, backend, false, 3);
        for _ in 0..8 {
            model.tick(&mut sim, &trace, backend, false, 3);
        }
        policy(&sim, offset, true, false, 0, 0, "second skip reset");
        inputs(&mut sim, &model, backend, true, 3);
        for _ in 0..64 {
            model.tick(&mut sim, &trace, backend, true, 3);
        }
        policy(
            &sim,
            offset,
            false,
            true,
            0,
            1024,
            "new auto-off after skip starts at initial period",
        );
        assert_eq!(model.cycle, 3285);
        eprintln!(
            "gate cycle complete backend={backend:?} gate={enabled} forced={forced} clocks={} tracked={offset:?}",
            model.cycle
        );
    }
}
fn run(backend: Backend, harness: &str) {
    if let Ok(mode) = std::env::var("VERYL_FORMAL_GATE_CYCLE_CHILD") {
        compare(backend, mode == "1");
        return;
    }
    for mode in ["0", "1"] {
        let output = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", harness, "--nocapture"])
            .env("VERYL_FORMAL_GATE_CYCLE_CHILD", mode)
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
            "gate-cycle child failed mode={mode}:\n{log}"
        );
        assert!(
            log.contains("1 passed; 0 failed") && log.contains("clocks=3285"),
            "gate-cycle child did not complete"
        );
        eprintln!("{log}");
    }
}
#[test]
fn interpreter_gate_cycle() {
    run(Backend::Interpret, "gate_cycle::interpreter_gate_cycle");
}
#[test]
fn cranelift_gate_cycle() {
    run(Backend::Cranelift, "gate_cycle::cranelift_gate_cycle");
}
#[test]
fn aot_c_gate_cycle() {
    run(Backend::Cc, "gate_cycle::aot_c_gate_cycle");
}

fn advance_max(
    sim: &mut Simulator,
    model: &mut Model,
    trace: &std::sync::Arc<std::sync::Mutex<super::Trace>>,
    backend: Backend,
    cycles: usize,
) {
    let clock = sim.get_clock("clk").unwrap();
    let calls = trace.lock().unwrap().calls;
    for index in 1..=cycles {
        sim.step(&clock);
        let old = model.held;
        model.held[0] = old[0].wrapping_add(1);
        for i in 1..N {
            model.held[i] = old[i].wrapping_add(old[i - 1]);
        }
        model.busy = model.busy.wrapping_add(1);
        model.cycle += 1;
        if index == cycles || index % 1024 == 0 {
            model.check(sim, backend);
            check_commit(sim, trace, backend, calls);
        }
    }
}
fn compare_max(backend: Backend, enabled: bool) {
    for forced in [false, true] {
        let (ir, trace) = analyze_config(&source(16), backend, forced);
        assert_eq!(!ir.event_gate_flags.is_empty(), enabled, "max gate route");
        let mut sim = Simulator::new(ir, None);
        let clock = sim.get_clock("clk").unwrap();
        let reset = sim.get_reset("rst").unwrap();
        let busy_reset = sim.get_reset("rst_busy").unwrap();
        sim.set("en", Value::new(0, 1, false));
        sim.set("data", Value::new(1, 16, false));
        sim.step_reset(&clock, &busy_reset);
        sim.step_reset(&clock, &reset);
        let mut model = Model {
            held: [0; N],
            busy: 1,
            cycle: 0,
        };
        for _ in 0..8 {
            model.tick(&mut sim, &trace, backend, false, 1);
        }
        let offset = if enabled {
            Some(
                sim.ir
                    .event_gate_flags
                    .iter()
                    .map(|&o| o as usize)
                    .find(|&o| header(&sim, o).idle)
                    .expect("idle max gate"),
            )
        } else {
            None
        };
        inputs(&mut sim, &model, backend, true, 1);
        for period in [1024, 2048, 4096, 8192, 16384, 32768, 65536] {
            advance_max(&mut sim, &mut model, &trace, backend, 63);
            policy(
                &sim,
                offset,
                false,
                false,
                63,
                if period == 1024 { 0 } else { period / 2 },
                "max before off",
            );
            advance_max(&mut sim, &mut model, &trace, backend, 1);
            policy(&sim, offset, false, true, 0, period, "max off");
            advance_max(&mut sim, &mut model, &trace, backend, (period - 1) as usize);
            policy(
                &sim,
                offset,
                false,
                true,
                period - 1,
                period,
                "max before re-arm",
            );
            advance_max(&mut sim, &mut model, &trace, backend, 1);
            policy(&sim, offset, false, false, 0, period, "max re-arm");
        }
        advance_max(&mut sim, &mut model, &trace, backend, 64);
        policy(&sim, offset, false, true, 0, 65536, "cap remains capped");
        eprintln!(
            "gate max complete backend={backend:?} gate={enabled} forced={forced} clocks={}",
            model.cycle
        );
    }
}
fn run_max(backend: Backend, harness: &str) {
    if let Ok(mode) = std::env::var("VERYL_FORMAL_GATE_MAX_CHILD") {
        compare_max(backend, mode == "1");
        return;
    }
    for mode in ["0", "1"] {
        let output = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", harness, "--nocapture"])
            .env("VERYL_FORMAL_GATE_MAX_CHILD", mode)
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
            "max gate child failed mode={mode}: {log}"
        );
        assert!(
            log.contains("1 passed; 0 failed") && log.contains("clocks=130568"),
            "max gate completion absent: {log}"
        );
        eprintln!("{log}");
    }
}
#[test]
fn interpreter_gate_max() {
    run_max(Backend::Interpret, "gate_cycle::interpreter_gate_max");
}
#[test]
fn cranelift_gate_max() {
    run_max(Backend::Cranelift, "gate_cycle::cranelift_gate_max");
}
#[test]
fn aot_c_gate_max() {
    run_max(Backend::Cc, "gate_cycle::aot_c_gate_max");
}
