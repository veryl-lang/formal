//! Selected native and 65/128/>128-bit four-state storage, bitwise operations,
//! procedural enables, and FF holds. Expected bits use scalar Logic4 tables, not
//! another backend. Selected patterns do not imply arbitrary four-state execution.

use super::{Backend, Simulator, Value, analyze_state_config};
use veryl_formal_semantics::{
    gates::{self, BitOp},
    logic4::{Logic4, logic_from_planes},
};

const PORTS: [&str; 6] = ["copied", "previous", "anded", "ored", "xored", "inverted"];
pub(super) const STATES: [Logic4; 4] = [Logic4::Zero, Logic4::One, Logic4::X, Logic4::Z];

fn source(width: usize) -> String {
    format!(
        r#"
    module Top (
        clk: input clock, rst: input reset, en: input logic,
        a: input logic<{width}>, b: input logic<{width}>,
        copied: output logic<{width}>, previous: output logic<{width}>,
        anded: output logic<{width}>, ored: output logic<{width}>,
        xored: output logic<{width}>, inverted: output logic<{width}>,
    ) {{
        var q: logic<{width}>; var prev: logic<{width}>;
        var qa: logic<{width}>; var qo: logic<{width}>;
        var qx: logic<{width}>; var qn: logic<{width}>;
        always_ff {{
            if_reset {{q = 0; prev = 0; qa = 0; qo = 0; qx = 0; qn = 0;}}
            else if en {{q = a; prev = q; qa = a & b; qo = a | b; qx = a ^ b; qn = ~a;}}
        }}
        assign copied = q; assign previous = prev;
        assign anded = qa; assign ored = qo; assign xored = qx; assign inverted = qn;
    }}
    "#
    )
}
pub(super) fn value(bits: &[Logic4]) -> Value {
    let mut payload = vec![0u8; bits.len().div_ceil(64) * 8];
    let mut mask = payload.clone();
    for (i, bit) in bits.iter().enumerate() {
        if matches!(bit, Logic4::One | Logic4::Z) {
            payload[i / 8] |= 1 << (i % 8);
        }
        if matches!(bit, Logic4::X | Logic4::Z) {
            mask[i / 8] |= 1 << (i % 8);
        }
    }
    Value::from_le_bytes(&payload, &mask, bits.len(), false)
}
fn check(
    sim: &mut Simulator,
    state: &[Vec<Logic4>; 6],
    width: usize,
    backend: Backend,
    cycle: usize,
) {
    for (port, expected) in PORTS.iter().zip(state) {
        let actual = sim.get(port).unwrap();
        let payload = actual.payload();
        let mask = actual.mask_xz();
        let decoded: Vec<_> = (0..width)
            .map(|i| logic_from_planes(payload.bit(i as u64), mask.bit(i as u64)))
            .collect();
        assert_eq!(
            decoded, *expected,
            "simulator four-state FF semantics mismatch: {backend:?} width={width} cycle={cycle} port={port}"
        );
        assert!(
            payload.bits() <= width as u64
                && mask.bits() <= width as u64
                && actual.width() == width
                && !actual.signed(),
            "simulator four-state FF semantics mismatch: output metadata or padding"
        );
    }
}
fn compare(backend: Backend, widths: &[usize]) {
    for &width in widths {
        for forced in [false, true] {
            let (ir, _) = analyze_state_config(&source(width), backend, forced, true);
            assert!(ir.use_4state);
            let mut sim = Simulator::new(ir, None);
            sim.write_log_diag.enabled = true;
            let clock = sim.get_clock("clk").unwrap();
            let reset = sim.get_reset("rst").unwrap();
            let mut state = std::array::from_fn(|_| vec![Logic4::Zero; width]);
            let mut cycle = 0;
            for pair in 0..16 {
                if pair == 0 || pair == 8 {
                    sim.step_reset(&clock, &reset);
                    state.iter_mut().for_each(|v| v.fill(Logic4::Zero));
                    check(&mut sim, &state, width, backend, cycle);
                }
                for (hold, enable) in [Logic4::One, Logic4::Zero, Logic4::X, Logic4::Z]
                    .into_iter()
                    .enumerate()
                {
                    let a: Vec<_> = (0..width)
                        .map(|i| STATES[(pair % 4 + i + i / 64 + hold) % 4])
                        .collect();
                    let b: Vec<_> = (0..width)
                        .map(|i| STATES[(pair / 4 + i / 4 + 2 * (i / 64) + hold) % 4])
                        .collect();
                    sim.set("a", value(&a));
                    sim.set("b", value(&b));
                    sim.set("en", value(&[enable]));
                    check(&mut sim, &state, width, backend, cycle);
                    sim.step(&clock);
                    if enable == Logic4::One {
                        state = [
                            a.clone(),
                            state[0].clone(),
                            a.iter()
                                .zip(&b)
                                .map(|(&x, &y)| gates::binary(x, y, BitOp::And))
                                .collect(),
                            a.iter()
                                .zip(&b)
                                .map(|(&x, &y)| gates::binary(x, y, BitOp::Or))
                                .collect(),
                            a.iter()
                                .zip(&b)
                                .map(|(&x, &y)| gates::binary(x, y, BitOp::Xor))
                                .collect(),
                            a.iter().map(|&x| gates::not(x)).collect(),
                        ];
                    }
                    cycle += 1;
                    check(&mut sim, &state, width, backend, cycle);
                    assert_eq!(
                        sim.ir.write_log_buffer.count(),
                        0,
                        "simulator four-state FF semantics mismatch: uncleared event log"
                    );
                }
            }
            assert!(
                sim.write_log_diag.total_entries > 0,
                "four-state FF logs were not exercised"
            );
            eprintln!(
                "four-state route backend={backend:?} width={width} forced={forced} clocks={cycle} entries={}",
                sim.write_log_diag.total_entries
            );
        }
    }
}
#[test]
fn interpreter_four_state_ff() {
    compare(Backend::Interpret, &[1, 8, 16, 32, 64]);
}
#[test]
fn cranelift_four_state_ff() {
    compare(Backend::Cranelift, &[1, 8, 16, 32, 64]);
}

#[test]
fn interpreter_wide_four_state_ff() {
    compare(Backend::Interpret, &[65, 128]);
}
#[test]
fn cranelift_wide_four_state_ff() {
    compare(Backend::Cranelift, &[65, 128]);
}

#[test]
fn interpreter_flat_four_state_ff() {
    compare(Backend::Interpret, &[129, 256, 512]);
}
#[test]
fn cranelift_flat_four_state_ff() {
    compare(Backend::Cranelift, &[129, 256, 512]);
}
