//! Finite comb-only array allocation and compiled dynamic reads, with selected
//! unknown indices. Layout coordinates and expected payload/mask planes are
//! independent of production. No arbitrary-rank array theorem.

use super::four_state::{STATES, value};
use super::{Backend, Simulator, Value, analyze_state_config};
use std::process::Command;
use veryl_formal_semantics::logic4::{Logic4, logic_from_planes};

fn source(width: usize, len: usize) -> String {
    let reset: String = (0..len).map(|i| format!("arr[{i}] = 0; ")).collect();
    let ports: String = (0..len)
        .map(|i| format!("o{i}: output logic<{width}>, "))
        .collect();
    let assigns: String = (0..len)
        .map(|i| format!("assign o{i} = arr[{i}]; "))
        .collect();
    format!(
        r#"
    module Top (
        clk: input clock, rst: input reset, en: input logic,
        write_index: input logic<16>, read_index: input logic<16>, data: input logic<{width}>,
        selected: output logic<{width}>, {ports}
    ) {{
        var arr: logic<{width}> [{len}];
        always_ff {{if_reset {{{reset}}} else if en {{arr[write_index] = data;}}}}
        assign selected = arr[read_index]; {assigns}
    }}
    "#
    )
}
fn check(sim: &mut Simulator, port: &str, want: &[Logic4], backend: Backend, step: usize) {
    let got = sim.get(port).unwrap();
    let p = got.payload();
    let m = got.mask_xz();
    let bits: Vec<_> = (0..want.len())
        .map(|i| logic_from_planes(p.bit(i as u64), m.bit(i as u64)))
        .collect();
    assert_eq!(
        bits,
        want,
        "simulator comb-array semantics mismatch: {backend:?} port={port} width={} step={step}",
        want.len()
    );
    assert!(
        p.bits() <= want.len() as u64
            && m.bits() <= want.len() as u64
            && got.width() == want.len()
            && !got.signed(),
        "simulator comb-array semantics mismatch: metadata/padding"
    );
}
fn compare(backend: Backend) {
    for (width, len) in [(129usize, 3usize), (232, 16), (512, 3)] {
        for forced in [false, true] {
            let (ir, _) = analyze_state_config(&source(width, len), backend, forced, true);
            let arr = ir
                .module_variables
                .variables
                .values()
                .find(|v| v.path.to_string() == "arr")
                .unwrap();
            assert_eq!(arr.current_values.len(), len);
            assert_eq!(arr.native_bytes, width.div_ceil(64) * 8);
            let buf = if forced {
                &ir.ff_values
            } else {
                &ir.comb_values
            };
            let lo = buf.as_ptr() as usize;
            let hi = lo + buf.len();
            let span = 2 * arr.native_bytes;
            for &ptr in &arr.current_values {
                let start = ptr as usize;
                assert!(
                    start >= lo && start.checked_add(span).is_some_and(|end| end <= hi),
                    "simulator comb-array semantics mismatch: allocation/placement"
                );
            }
            assert!(
                arr.current_values
                    .windows(2)
                    .all(|p| p[0] as usize + span <= p[1] as usize),
                "simulator comb-array semantics mismatch: overlapping element planes"
            );
            let (cc, ci, ec, ei) = ir.detailed_stats();
            if backend == Backend::CraneliftFallback {
                assert!(
                    cc > 0 && ci == 0,
                    "dynamic comb reader fell back to interpreter"
                );
            }
            eprintln!(
                "comb array route {backend:?} width={width} len={len} forced={forced} comb={cc}/{ci} event={ec}/{ei}"
            );
            let mut sim = Simulator::new(ir, None);
            let clock = sim.get_clock("clk").unwrap();
            let reset = sim.get_reset("rst").unwrap();
            sim.set("en", value(&[Logic4::Zero]));
            sim.set("read_index", Value::new(0, 16, false));
            sim.step_reset(&clock, &reset);
            let mut state = vec![vec![Logic4::Zero; width]; len];
            let mut step = 0;
            for case in 0..len + 3 {
                let index = if case < len {
                    case
                } else {
                    [len, len + 1, 65535][case - len]
                };
                let data: Vec<_> = (0..width)
                    .map(|i| STATES[(i + i / 64 + case) % 4])
                    .collect();
                sim.set("write_index", Value::new(index as u64, 16, false));
                sim.set("data", value(&data));
                sim.set("en", value(&[Logic4::One]));
                for (i, want) in state.iter().enumerate() {
                    check(&mut sim, &format!("o{i}"), want, backend, step);
                }
                sim.step(&clock);
                step += 1;
                if index < len {
                    state[index] = data;
                }
                for (i, want) in state.iter().enumerate() {
                    check(&mut sim, &format!("o{i}"), want, backend, step);
                }
                // Runtime reads change without a clock, including invalid indices.
                for r in (0..len).chain([len, len + 1, 65535]) {
                    sim.set("read_index", Value::new(r as u64, 16, false));
                    let unknown = vec![Logic4::X; width];
                    check(
                        &mut sim,
                        "selected",
                        state.get(r).unwrap_or(&unknown),
                        backend,
                        step,
                    );
                }
                for state_bit in [Logic4::X, Logic4::Z] {
                    let mut bad_index = vec![Logic4::Zero; 16];
                    bad_index[0] = state_bit;
                    sim.set("read_index", value(&bad_index));
                    check(&mut sim, "selected", &vec![Logic4::X; width], backend, step);
                }
                for (i, want) in state.iter().enumerate() {
                    check(&mut sim, &format!("o{i}"), want, backend, step);
                }
            }
            eprintln!("comb array complete width={width} len={len} forced={forced} steps={step}");
        }
    }
}
fn run(backend: Backend, harness: &str) {
    if std::env::var_os("VERYL_FORMAL_COMB_ARRAY_CHILD").is_some() {
        compare(backend);
        return;
    }
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", harness, "--nocapture"])
        .env("VERYL_FORMAL_COMB_ARRAY_CHILD", "1")
        .output()
        .unwrap();
    let log = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.status.success(), "comb array child failed:\n{log}");
    assert!(
        log.contains("1 passed; 0 failed") && log.contains("complete width=512"),
        "comb array child did not complete"
    );
    eprintln!("{log}");
}
#[test]
fn interpreter_comb_array() {
    run(Backend::Interpret, "comb_array::interpreter_comb_array");
}
#[test]
fn cranelift_fallback_comb_array() {
    run(
        Backend::CraneliftFallback,
        "comb_array::cranelift_fallback_comb_array",
    );
}
