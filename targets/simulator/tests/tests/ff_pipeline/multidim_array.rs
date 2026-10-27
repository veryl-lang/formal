//! Finite multidimensional array coordinates with per-axis validity independent
//! of flattened bounds. No arbitrary-rank or arbitrary-extent theorem.

use super::four_state::{STATES, value};
use super::signed_array::known;
use super::{Backend, Simulator, analyze_state_config, check_commit};
use std::process::Command;
use veryl_formal_semantics::logic4::Logic4;

struct Shape {
    dims: &'static [usize],
    bits: &'static [usize],
    signed: &'static [bool],
}
const SHAPES: [Shape; 4] = [
    Shape {
        dims: &[2, 3],
        bits: &[4, 4],
        signed: &[true, true],
    },
    Shape {
        dims: &[2, 2, 3],
        bits: &[4, 4, 4],
        signed: &[true, false, true],
    },
    Shape {
        dims: &[4, 2],
        bits: &[2, 1],
        signed: &[false, false],
    },
    Shape {
        dims: &[2, 3],
        bits: &[64, 64],
        signed: &[true, true],
    },
];
fn coordinates(dims: &[usize]) -> Vec<Vec<usize>> {
    if let Some((&first, rest)) = dims.split_first() {
        let tails = coordinates(rest);
        (0..first)
            .flat_map(|i| {
                tails.iter().map(move |tail| {
                    let mut v = vec![i];
                    v.extend(tail);
                    v
                })
            })
            .collect()
    } else {
        vec![vec![]]
    }
}
fn path(coords: &[usize]) -> String {
    coords.iter().map(|i| format!("[{i}]")).collect()
}
fn source(width: usize, shape: &Shape) -> String {
    let coords = coordinates(shape.dims);
    let ports: String = (0..coords.len())
        .map(|i| format!("o{i}: output logic<{width}>, "))
        .collect();
    let reset: String = coords
        .iter()
        .enumerate()
        .map(|(i, c)| format!("arr{} = {}; ", path(c), i + 1))
        .collect();
    let assigns: String = coords
        .iter()
        .enumerate()
        .map(|(i, c)| format!("assign o{i} = arr{}; ", path(c)))
        .collect();
    let indices: String = shape
        .bits
        .iter()
        .zip(shape.signed)
        .enumerate()
        .map(|(i, (bits, signed))| {
            let sign = if *signed { "signed " } else { "" };
            format!("w{i}: input {sign}logic<{bits}>, r{i}: input {sign}logic<{bits}>, ")
        })
        .collect();
    let dims = shape
        .dims
        .iter()
        .map(usize::to_string)
        .collect::<Vec<_>>()
        .join(",");
    let wi: String = (0..shape.dims.len()).map(|i| format!("[w{i}]")).collect();
    let ri: String = (0..shape.dims.len()).map(|i| format!("[r{i}]")).collect();
    let zero = path(&vec![0; shape.dims.len()]);
    format!(
        r#"
    module Top (
        clk: input clock, rst: input reset, data: input logic<{width}>, {indices}
        selected: output logic<{width}>, previous: output logic<{width}>, {ports}
    ) {{
        var arr: logic<{width}> [{dims}]; var prev: logic<{width}>;
        always_ff {{
            if_reset {{{reset} prev = 0;}}
            else {{arr{wi} = data; prev = arr{zero};}}
        }}
        assign selected = arr{ri}; assign previous = prev; {assigns}
    }}
    "#
    )
}
struct Case {
    values: Vec<Vec<Logic4>>,
    target: Option<usize>,
}
fn cases(shape: &Shape, four: bool) -> Vec<Case> {
    let coords = coordinates(shape.dims);
    let encode = |c: &[usize]| {
        c.iter()
            .zip(shape.bits)
            .map(|(&n, &bits)| known(n as i128, bits))
            .collect::<Vec<_>>()
    };
    let mut result: Vec<_> = coords
        .iter()
        .enumerate()
        .map(|(i, c)| Case {
            values: encode(c),
            target: Some(i),
        })
        .collect();
    for axis in 0..shape.dims.len() {
        let bits = shape.bits[axis];
        let min = if shape.signed[axis] {
            -(1i128 << (bits - 1))
        } else {
            0
        };
        let max = (1i128 << (bits - usize::from(shape.signed[axis]))) - 1;
        let mut invalid = vec![
            min,
            -1,
            shape.dims[axis] as i128,
            shape.dims[axis] as i128 + 1,
            max,
        ];
        if bits == 64 {
            invalid.extend([1i128 << 32, (1i128 << 32) + 1]);
        }
        invalid.retain(|&n| n >= min && n <= max && !(0..shape.dims[axis] as i128).contains(&n));
        invalid.sort();
        invalid.dedup();
        for base in [
            vec![0; shape.dims.len()],
            shape.dims.iter().map(|d| d - 1).collect(),
        ] {
            for &n in &invalid {
                let mut values = encode(&base);
                values[axis] = known(n, bits);
                result.push(Case {
                    values,
                    target: None,
                });
            }
            if four {
                for bit in [0, bits / 2, bits - 1] {
                    for unknown in [Logic4::X, Logic4::Z] {
                        let mut values = encode(&base);
                        values[axis][bit] = unknown;
                        result.push(Case {
                            values,
                            target: None,
                        });
                    }
                }
            }
        }
    }
    result
}
fn check(sim: &mut Simulator, port: &str, want: &[Logic4], backend: Backend, step: usize) {
    assert_eq!(
        sim.get(port),
        Some(value(want)),
        "simulator multidim-array semantics mismatch: {backend:?} port={port} width={} step={step}",
        want.len()
    );
}
fn state_check(
    sim: &mut Simulator,
    state: &[Vec<Logic4>],
    previous: &[Logic4],
    backend: Backend,
    step: usize,
) {
    for (i, want) in state.iter().enumerate() {
        check(sim, &format!("o{i}"), want, backend, step);
    }
    check(sim, "previous", previous, backend, step);
}
fn compare(backend: Backend, four: bool, widths: &[usize]) {
    let mut total = 0;
    let mut reads = 0;
    let mut configurations = 0;
    for &width in widths {
        for shape in &SHAPES {
            let len = coordinates(shape.dims).len();
            for forced in [false, true] {
                let (ir, trace) =
                    analyze_state_config(&source(width, shape), backend, forced, four);
                let arr = ir
                    .module_variables
                    .variables
                    .values()
                    .find(|v| v.path.to_string() == "arr")
                    .unwrap();
                let span = arr.native_bytes * if four { 2 } else { 1 };
                assert_eq!(arr.current_values.len(), len);
                let ff = ir.ff_values.as_ptr() as usize
                    ..ir.ff_values.as_ptr() as usize + ir.ff_values.len();
                let comb = ir.comb_values.as_ptr() as usize
                    ..ir.comb_values.as_ptr() as usize + ir.comb_values.len();
                for &ptr in arr.current_values.iter().chain(&arr.next_values) {
                    assert!(
                        [&ff, &comb].iter().any(|r| ptr as usize >= r.start
                            && (ptr as usize)
                                .checked_add(span)
                                .is_some_and(|end| end <= r.end)),
                        "multidim-array allocation invalid"
                    );
                }
                let mut intervals: Vec<_> = arr
                    .current_values
                    .iter()
                    .map(|&p| (p as usize, p as usize + span))
                    .collect();
                intervals.sort_unstable();
                assert!(
                    intervals.windows(2).all(|p| p[0].1 <= p[1].0),
                    "multidim-array elements overlap"
                );
                let (cc, ci, ec, ei) = ir.detailed_stats();
                if backend != Backend::Interpret {
                    assert!(cc > 0 && ci == 0, "signed dynamic reader fell back");
                }
                eprintln!(
                    "multidim-array route {backend:?} four={four} width={width} shape={:?} len={len} forced={forced} comb={cc}/{ci} events={ec}/{ei}",
                    shape.dims
                );
                let mut sim = Simulator::new(ir, None);
                sim.write_log_diag.enabled = true;
                let clock = sim.get_clock("clk").unwrap();
                let reset = sim.get_reset("rst").unwrap();
                let schedule = cases(shape, four);
                let mut step = 0;
                for epoch in 0..2 {
                    let calls = trace.lock().unwrap().calls;
                    sim.step_reset(&clock, &reset);
                    check_commit(&sim, &trace, backend, calls);
                    let mut state: Vec<_> =
                        (0..len).map(|i| known((i + 1) as i128, width)).collect();
                    let mut prev = vec![Logic4::Zero; width];
                    state_check(&mut sim, &state, &prev, backend, step);
                    for case in &schedule {
                        let data: Vec<_> = (0..width)
                            .map(|i| {
                                if four {
                                    STATES[(i + i / 64 + step + epoch) % 4]
                                } else if (i + i / 64 + step + epoch) % 2 == 0 {
                                    Logic4::Zero
                                } else {
                                    Logic4::One
                                }
                            })
                            .collect();
                        for (axis, idx) in case.values.iter().enumerate() {
                            sim.set(&format!("w{axis}"), value(idx));
                        }
                        sim.set("data", value(&data));
                        state_check(&mut sim, &state, &prev, backend, step);
                        let calls = trace.lock().unwrap().calls;
                        sim.step(&clock);
                        check_commit(&sim, &trace, backend, calls);
                        prev = state[0].clone();
                        if let Some(target) = case.target {
                            state[target] = data;
                        }
                        step += 1;
                        state_check(&mut sim, &state, &prev, backend, step);
                        for read in &schedule {
                            reads += 1;
                            for (axis, idx) in read.values.iter().enumerate() {
                                sim.set(&format!("r{axis}"), value(idx));
                            }
                            let invalid = vec![if four { Logic4::X } else { Logic4::Zero }; width];
                            check(
                                &mut sim,
                                "selected",
                                read.target.and_then(|i| state.get(i)).unwrap_or(&invalid),
                                backend,
                                step,
                            );
                        }
                        state_check(&mut sim, &state, &prev, backend, step);
                        assert_eq!(
                            sim.ir.write_log_buffer.count(),
                            0,
                            "simulator multidim-array semantics mismatch: uncleared log"
                        );
                    }
                }
                assert!(
                    sim.write_log_diag.total_entries > 0,
                    "multidim-array logs unused"
                );
                total += step;
                configurations += 1;
            }
        }
    }
    assert_eq!(configurations, widths.len() * SHAPES.len() * 2);
    eprintln!(
        "multidim-array complete {backend:?} four={four} configurations={configurations} clocks={total} reads={reads}"
    );
}
fn run(backend: Backend, four: bool, widths: &[usize], harness: &str) {
    if std::env::var_os("VERYL_FORMAL_MULTIDIM_ARRAY_CHILD").is_some() {
        compare(backend, four, widths);
        return;
    }
    let out = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", harness, "--nocapture"])
        .env("VERYL_FORMAL_MULTIDIM_ARRAY_CHILD", "1")
        .output()
        .unwrap();
    let log = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(out.status.success(), "multidim-array child failed:\n{log}");
    assert!(
        log.contains("1 passed; 0 failed") && log.contains("multidim-array complete"),
        "multidim-array child incomplete"
    );
    eprintln!("{log}");
}
#[test]
fn interpreter_multidim_array() {
    run(
        Backend::Interpret,
        false,
        &[16, 129],
        "multidim_array::interpreter_multidim_array",
    );
}
#[test]
fn cranelift_multidim_array() {
    run(
        Backend::Cranelift,
        false,
        &[16, 129],
        "multidim_array::cranelift_multidim_array",
    );
}
#[test]
fn aot_c_multidim_array() {
    run(
        Backend::Cc,
        false,
        &[16, 129],
        "multidim_array::aot_c_multidim_array",
    );
}
#[test]
fn interpreter_four_multidim_array() {
    run(
        Backend::Interpret,
        true,
        &[8, 64, 65, 129, 512],
        "multidim_array::interpreter_four_multidim_array",
    );
}
#[test]
fn cranelift_four_multidim_array() {
    run(
        Backend::Cranelift,
        true,
        &[8, 64],
        "multidim_array::cranelift_four_multidim_array",
    );
}
#[test]
fn fallback_four_multidim_array() {
    run(
        Backend::CraneliftFallback,
        true,
        &[65, 129, 512],
        "multidim_array::fallback_four_multidim_array",
    );
}
