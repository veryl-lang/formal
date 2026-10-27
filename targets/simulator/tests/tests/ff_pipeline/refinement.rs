//! Finite default/forced FF placement and old-value/reset comparisons. Independent
//! state expectations cover selected declarations and histories, not arbitrary lowering.

use super::{Backend, Simulator, analyze_config, check, check_commit, value};
use veryl_formal_semantics::words;
use veryl_simulator::ir::Ir;

fn source(width: usize) -> String {
    format!(
        r#"
    module Top (
        clk: input clock, rst: input reset, data: input logic<{width}>,
        safe_out: output logic<{width}>, retained_out: output logic<{width}>,
        previous: output logic<{width}>, repeated_out: output logic<{width}>,
    ) {{
        var safe: logic<{width}>;
        var retained: logic<{width}>;
        var prev: logic<{width}>;
        var repeated: logic<{width}>;
        always_ff {{
            if_reset {{safe = 0;}} else {{safe = safe + 1;}}
        }}
        always_ff {{
            if_reset {{retained = 0;}} else {{retained = data;}}
        }}
        always_ff {{
            if_reset {{prev = 0;}} else {{prev = retained;}}
        }}
        always_ff {{
            if_reset {{repeated = 0;}}
            else {{repeated = repeated + 1; repeated = repeated + 1;}}
        }}
        assign safe_out = safe;
        assign retained_out = retained;
        assign previous = prev;
        assign repeated_out = repeated;
    }}
    "#
    )
}

fn check_layout(ir: &Ir, forced: bool, width: usize, backend: Backend) {
    let ff = ir.ff_values.as_ptr() as usize..ir.ff_values.as_ptr() as usize + ir.ff_values.len();
    let comb =
        ir.comb_values.as_ptr() as usize..ir.comb_values.as_ptr() as usize + ir.comb_values.len();
    for (name, retained) in [
        ("safe", false),
        ("retained", true),
        ("prev", false),
        ("repeated", true),
    ] {
        let variable = ir.module_variables.variables.values().find(|v| {
            let path = v.path.to_string();
            path == name || path.ends_with(&format!(".{name}"))
        });
        assert!(
            variable.is_some(),
            "simulator FF pipeline semantics mismatch: missing layout variable {name}"
        );
        let variable = variable.unwrap();
        assert_eq!(variable.width, width);
        assert_eq!(variable.current_values.len(), 1);
        let pointer = variable.current_values[0] as usize;
        let expected_ff = forced || retained;
        assert!(
            if expected_ff {
                ff.contains(&pointer)
            } else {
                comb.contains(&pointer)
            },
            "simulator FF pipeline semantics mismatch: layout backend={backend:?} width={width} forced={forced} variable={name} expected_ff={expected_ff}"
        );
        eprintln!(
            "refinement layout backend={backend:?} width={width} forced={forced} variable={name} ff={expected_ff}"
        );
    }
}

fn increment(bytes: &[u8], width: usize) -> Vec<u8> {
    let mut one = vec![0; bytes.len()];
    one[0] = 1;
    let mut result = words::add_sub(bytes, &one, false);
    if !width.is_multiple_of(8) {
        *result.last_mut().unwrap() &= (1 << (width % 8)) - 1;
    }
    result
}

fn compare(backend: Backend) {
    for width in [8usize, 64, 65, 128, 448, 512] {
        for forced in [false, true] {
            let (ir, trace) = analyze_config(&source(width), backend, forced);
            check_layout(&ir, forced, width, backend);
            let mut sim = Simulator::new(ir, None);
            sim.write_log_diag.enabled = true;
            let clock = sim.get_clock("clk").unwrap();
            let reset = sim.get_reset("rst").unwrap();
            let mut retained = vec![0; width.div_ceil(8)];
            let mut counter = retained.clone();
            for epoch in 0..2 {
                let calls = trace.lock().unwrap().calls;
                sim.step_reset(&clock, &reset);
                check_commit(&sim, &trace, backend, calls);
                retained.fill(0);
                counter.fill(0);
                for port in ["safe_out", "retained_out", "previous", "repeated_out"] {
                    check(&mut sim, port, &retained, width, backend, epoch * 9);
                }
                for cycle in 1..=9 {
                    let mut data: Vec<_> = (0..retained.len())
                        .map(|i| match (cycle + epoch) % 6 {
                            0 => 0,
                            1 => 255,
                            2 => 0xaa,
                            3 => 0x55,
                            4 => (i * 37 + cycle * 13) as u8,
                            _ => 1 << (i % 8),
                        })
                        .collect();
                    if !width.is_multiple_of(8) {
                        *data.last_mut().unwrap() &= (1 << (width % 8)) - 1;
                    }
                    sim.set("data", value(&data, width));
                    // Driving a new input must not fire an always_ff block.
                    check(&mut sim, "retained_out", &retained, width, backend, cycle);
                    check(&mut sim, "safe_out", &counter, width, backend, cycle);
                    let calls = trace.lock().unwrap().calls;
                    sim.step(&clock);
                    check_commit(&sim, &trace, backend, calls);
                    counter = increment(&counter, width);
                    check(&mut sim, "retained_out", &data, width, backend, cycle);
                    check(&mut sim, "previous", &retained, width, backend, cycle);
                    check(&mut sim, "safe_out", &counter, width, backend, cycle);
                    check(&mut sim, "repeated_out", &counter, width, backend, cycle);
                    retained = data;
                }
            }
            assert!(sim.write_log_diag.total_entries > 0);
            if backend == Backend::Cc {
                let trace = trace.lock().unwrap();
                assert!(trace.narrow + trace.wide > 0);
                eprintln!(
                    "refinement route width={width} forced={forced} dispatches={} narrow={} wide={}",
                    trace.calls, trace.narrow, trace.wide
                );
            }
        }
    }
}

#[test]
fn interpreter_ff_refinement() {
    compare(Backend::Interpret);
}
#[test]
fn cranelift_ff_refinement() {
    compare(Backend::Cranelift);
}
#[test]
fn aot_c_ff_refinement() {
    compare(Backend::Cc);
}
