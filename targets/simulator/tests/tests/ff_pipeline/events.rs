//! Finite two-clock four-register four-bit histories, both event orders and FF
//! placements, and synchronous/asynchronous high/low resets. Cross-domain reads
//! observe the prior event state. No arbitrary history, concurrency, or CDC theorem.

use super::{Backend, Simulator, analyze_config};
use veryl_simulator::ir::Value;
use veryl_simulator::simulator::ResetWindow;
fn source(reset: &str) -> String {
    format!(
        r#"
module Top(
    clk_a:input 'domain_a clock, rst_a:input 'domain_a {reset}, data_a:input 'domain_a logic<4>,
    clk_b:input 'domain_b clock, rst_b:input 'domain_b {reset}, data_b:input 'domain_b logic<4>,
    out_a:output 'domain_a logic<4>,out_b:output 'domain_b logic<4>, prev_a:output 'domain_a logic<4>,prev_b:output 'domain_b logic<4>,
){{
    var a:'domain_a logic<4>;var b:'domain_b logic<4>;var pa:'domain_a logic<4>;var pb:'domain_b logic<4>;
    unsafe(cdc){{
        always_ff(clk_a,rst_a){{if_reset{{a=1;pa=0;}}else{{a=data_a;a=a+1;pa=b;}}}}
        always_ff(clk_b,rst_b){{if_reset{{b=2;pb=0;}}else{{b=a^data_b;pb=a;}}}}
    }}
    assign out_a=a;assign out_b=b;assign prev_a=pa;assign prev_b=pb;
}}
"#
    )
}
fn compare(backend: Backend) {
    let mut steps = 0;
    for reset in [
        "reset_async_low",
        "reset_async_high",
        "reset_sync_low",
        "reset_sync_high",
    ] {
        for forced in [false, true] {
            for reverse in [false, true] {
                let (ir, trace) = analyze_config(&source(reset), backend, forced);
                let mut sim = Simulator::new(ir, None);
                let ca = sim.get_clock("clk_a").unwrap();
                let cb = sim.get_clock("clk_b").unwrap();
                let ra = sim.get_reset("rst_a").unwrap();
                let rb = sim.get_reset("rst_b").unwrap();
                let clocks = if reverse {
                    [cb.clone(), ca.clone()]
                } else {
                    [ca.clone(), cb.clone()]
                };
                let mut state = [0u8; 4];
                for history in 0..64 {
                    sim.set_reset_level(&ra.var_id().unwrap(), true);
                    sim.set_reset_level(&rb.var_id().unwrap(), true);
                    let windows = [
                        ResetWindow {
                            clock: ca.clone(),
                            reset: ra.clone(),
                            assertion_edge: true,
                        },
                        ResetWindow {
                            clock: cb.clone(),
                            reset: rb.clone(),
                            assertion_edge: true,
                        },
                    ];
                    sim.step_events(&[], &windows);
                    if reset.contains("async") {
                        state = [1, 2, 0, 0];
                    }
                    // The initial synchronous state is intentionally not assumed.
                    if history > 0 || reset.contains("async") {
                        observe(&mut sim, state, backend, steps);
                    }
                    sim.step_events(&clocks, &[]);
                    state = [1, 2, 0, 0];
                    observe(&mut sim, state, backend, steps);
                    sim.step(&ca);
                    observe(&mut sim, state, backend, steps);
                    sim.set_reset_level(&ra.var_id().unwrap(), false);
                    sim.set_reset_level(&rb.var_id().unwrap(), false);
                    let mut encoded = history;
                    for stage in 0..3 {
                        let active = encoded % 4;
                        encoded /= 4;
                        let data = ((history * 7 + stage * 3) & 15) as u8;
                        sim.set("data_a", Value::new((data ^ 15) as u64, 4, false));
                        sim.set("data_b", Value::new(data as u64, 4, false));
                        observe(&mut sim, state, backend, steps);
                        let events: Vec<_> = clocks
                            .iter()
                            .filter(|e| {
                                if **e == ca {
                                    active & 1 != 0
                                } else {
                                    active & 2 != 0
                                }
                            })
                            .cloned()
                            .collect();
                        let before = state;
                        if active & 1 != 0 {
                            state[0] = (before[0] + 1) & 15;
                            state[2] = before[1];
                        }
                        if active & 2 != 0 {
                            state[1] = before[0] ^ data;
                            state[3] = before[0];
                        }
                        sim.step_events(&events, &[]);
                        steps += 1;
                        observe(&mut sim, state, backend, steps);
                        assert_eq!(
                            sim.ir.write_log_buffer.count(),
                            0,
                            "simulator event state mismatch: live log"
                        );
                    }
                }
                if backend == Backend::Cc {
                    assert!(
                        trace.lock().unwrap().calls > 0,
                        "AOT-C event dispatch absent"
                    );
                }
            }
        }
    }
    eprintln!("event state complete backend={backend:?} steps={steps}");
}
fn observe(sim: &mut Simulator, state: [u8; 4], backend: Backend, step: usize) {
    for (port, byte) in ["out_a", "out_b", "prev_a", "prev_b"]
        .into_iter()
        .zip(state)
    {
        assert_eq!(
            sim.get(port),
            Some(Value::new(byte as u64, 4, false)),
            "simulator event state mismatch: {backend:?} {step} {port}"
        );
    }
}
#[test]
fn interpreter_event_state() {
    compare(Backend::Interpret);
}
#[test]
fn cranelift_event_state() {
    compare(Backend::Cranelift);
}
#[test]
fn aot_c_event_state() {
    compare(Backend::Cc);
}
