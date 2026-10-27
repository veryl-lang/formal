//! Finite dynamic packed-read boundaries. Missing four-state source bits yield X;
//! two-state output uses zero defaults. Valid bits retain X/Z. This is not a
//! general selector-decoding or source-to-result theorem.

use super::four_state::{STATES, value};
use super::signed_array::known;
use super::wide_select::{forms, numbers, offset};
use super::{Backend, Simulator, analyze_state_config};
use std::process::Command;
use veryl_formal_semantics::logic4::Logic4;

fn source(
    width: usize,
    bits: usize,
    signed: bool,
    form: &str,
    window: usize,
    array: bool,
) -> String {
    let sign = if signed { "signed " } else { "" };
    let select = match form {
        "bit" => "[index]".to_string(),
        "plus" => format!("[index+:{window}]"),
        "minus" => format!("[index-:{window}]"),
        _ => format!("[index step {window}]"),
    };
    let count = if array { 3 } else { 1 };
    let ports: String = (0..count)
        .map(|i| format!("d{i}: input logic<{width}>, m{i}: output logic<{width}>, "))
        .collect();
    let declaration = if array {
        format!("var arr: logic<{width}> [3];")
    } else {
        format!("var q: logic<{width}>;")
    };
    let assignments: String = (0..count)
        .map(|i| {
            if array {
                format!("assign arr[{i}] = d{i}; assign m{i} = arr[{i}];")
            } else {
                "assign q = d0; assign m0 = q;".into()
            }
        })
        .collect();
    let target = if array { "arr[element]" } else { "q" };
    format!(
        r#"
        module Top (
            clk: input clock, rst: input reset, index: input {sign}logic<{bits}>, element: input logic<2>,
            out: output logic<{window}>, busy: output logic<16>, {ports}
        ) {{
            {declaration} var count: logic<16>;
            always_ff {{if_reset {{count = 0;}} else {{count = count + 1;}}}}
            assign busy = count; {assignments} assign out = {target}{select};
        }}
    "#
    )
}
fn read_forms(width: usize, wide: bool) -> Vec<(&'static str, usize)> {
    if wide {
        [65, 128, 129, 255, 512]
            .into_iter()
            .filter(|&w| w <= width)
            .flat_map(|w| [("plus", w), ("minus", w), ("step", w)])
            .collect()
    } else {
        forms(width)
    }
}
fn compare(backend: Backend, widths: &[usize], array: bool, fallback: bool, wide: bool) {
    let mut configurations = 0;
    let mut reads = 0;
    for &width in widths {
        for bits in [1, 4, 64] {
            for signed in [false, true] {
                for four in [true, false] {
                    for (form, window) in read_forms(width, wide) {
                        let (ir, _) = analyze_state_config(
                            &source(width, bits, signed, form, window, array),
                            backend,
                            false,
                            four,
                        );
                        if array {
                            let arr = ir
                                .module_variables
                                .variables
                                .values()
                                .find(|v| v.path.to_string() == "arr")
                                .unwrap();
                            let span = arr.native_bytes * if four { 2 } else { 1 };
                            let ff = ir.ff_values.as_ptr() as usize
                                ..ir.ff_values.as_ptr() as usize + ir.ff_values.len();
                            let comb = ir.comb_values.as_ptr() as usize
                                ..ir.comb_values.as_ptr() as usize + ir.comb_values.len();
                            assert_eq!(arr.current_values.len(), 3);
                            for &p in arr.current_values.iter().chain(&arr.next_values) {
                                assert!(
                                    [&ff, &comb].iter().any(|r| p as usize >= r.start
                                        && (p as usize)
                                            .checked_add(span)
                                            .is_some_and(|e| e <= r.end)),
                                    "packed-read allocation invalid"
                                );
                            }
                            let mut spans: Vec<_> = arr
                                .current_values
                                .iter()
                                .map(|&p| (p as usize, p as usize + span))
                                .collect();
                            spans.sort();
                            assert!(
                                spans.windows(2).all(|p| p[0].1 <= p[1].0),
                                "packed-read elements overlap"
                            );
                        }
                        let (cc, ci, ec, ei) = ir.detailed_stats();
                        eprintln!(
                            "packed-read route {backend:?} width={width} bits={bits} signed={signed} four={four} form={form} window={window} array={array} fallback={fallback} comb={cc}/{ci} events={ec}/{ei}"
                        );
                        if backend == Backend::Cranelift {
                            if fallback && !signed && form != "minus" {
                                assert!(ci > 0, "packed-read expected comb fallback missing");
                            } else {
                                assert!(
                                    cc > 0 && ci == 0,
                                    "packed-read compiled comb route missing"
                                );
                            }
                        }
                        let mut sim = Simulator::new(ir, None);
                        let clock = sim.get_clock("clk").unwrap();
                        let reset = sim.get_reset("rst").unwrap();
                        sim.step_reset(&clock, &reset);
                        let count = if array { 3 } else { 1 };
                        for pattern in 0..4 {
                            let data: Vec<Vec<_>> = (0..count)
                                .map(|e| {
                                    (0..width)
                                        .map(|b| match pattern {
                                            0 => Logic4::Zero,
                                            1 => Logic4::One,
                                            _ => {
                                                STATES[(b + b / 64 + e + pattern)
                                                    % if four { 4 } else { 2 }]
                                            }
                                        })
                                        .collect()
                                })
                                .collect();
                            let values: Vec<_> = data.iter().map(|v| value(v)).collect();
                            for (i, v) in values.iter().enumerate() {
                                sim.set(&format!("d{i}"), v.clone());
                            }
                            let element = pattern % count;
                            sim.set("element", value(&known(element as i128, 2)));
                            let check =
                                |sim: &mut Simulator, start: Option<i128>, reads: &mut usize| {
                                    let default = if four { Logic4::X } else { Logic4::Zero };
                                    let expected: Vec<_> = (0..window)
                                        .map(|b| {
                                            start
                                                .and_then(|n| usize::try_from(n + b as i128).ok())
                                                .and_then(|i| data[element].get(i).copied())
                                                .unwrap_or(default)
                                        })
                                        .collect();
                                    assert_eq!(
                                        sim.get("out"),
                                        Some(value(&expected)),
                                        "simulator packed-read output mismatch: {backend:?} width={width} bits={bits} signed={signed} four={four} form={form} window={window} pattern={pattern} start={start:?}"
                                    );
                                    for (i, v) in values.iter().enumerate() {
                                        assert_eq!(
                                            sim.get(&format!("m{i}")),
                                            Some(v.clone()),
                                            "simulator packed-read output mismatch: source changed"
                                        );
                                    }
                                    assert_eq!(
                                        sim.get("busy"),
                                        Some(super::Value::new(pattern as u64, 16, false)),
                                        "simulator packed-read output mismatch: unexpected clock"
                                    );
                                    *reads += 1;
                                };
                            for n in numbers(bits, signed, width, window) {
                                sim.set("index", value(&known(n, bits)));
                                check(&mut sim, Some(offset(n, window, form)), &mut reads);
                            }
                            for bit in [0, bits / 2, bits - 1] {
                                for base in [0, -1] {
                                    for unknown in [Logic4::X, Logic4::Z] {
                                        let mut idx = known(base, bits);
                                        idx[bit] = unknown;
                                        sim.set("index", value(&idx));
                                        let normalized = if four {
                                            None
                                        } else {
                                            let n =
                                                idx.iter().enumerate().fold(0u128, |v, (i, b)| {
                                                    v | if matches!(b, Logic4::One | Logic4::Z) {
                                                        1u128 << i
                                                    } else {
                                                        0
                                                    }
                                                });
                                            let n = if signed && n & (1u128 << (bits - 1)) != 0 {
                                                n as i128 - (1i128 << bits)
                                            } else {
                                                n as i128
                                            };
                                            Some(offset(n, window, form))
                                        };
                                        check(&mut sim, normalized, &mut reads);
                                    }
                                }
                            }
                            sim.step(&clock);
                        }
                        assert_eq!(
                            sim.get("busy"),
                            Some(super::Value::new(4, 16, false)),
                            "packed-read clock witness missing"
                        );
                        configurations += 1;
                    }
                }
            }
        }
    }
    assert_eq!(
        configurations,
        widths
            .iter()
            .map(|&w| read_forms(w, wide).len() * 12)
            .sum::<usize>()
    );
    eprintln!(
        "packed-read complete {backend:?} array={array} fallback={fallback} configurations={configurations} reads={reads}"
    );
}
pub(super) fn run(
    backend: Backend,
    widths: &[usize],
    array: bool,
    fallback: bool,
    wide: bool,
    harness: &str,
) {
    if std::env::var_os("VERYL_FORMAL_PACKED_READ_CHILD").is_some() {
        compare(backend, widths, array, fallback, wide);
        return;
    }
    let out = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", harness, "--nocapture"])
        .env("VERYL_FORMAL_PACKED_READ_CHILD", "1")
        .output()
        .unwrap();
    let log = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(out.status.success(), "packed-read child failed:\n{log}");
    assert!(
        log.contains("1 passed; 0 failed") && log.contains("packed-read complete"),
        "packed-read child incomplete"
    );
    eprintln!("{log}");
}
#[test]
fn interpreter_scalar_packed_read() {
    run(
        Backend::Interpret,
        &[8, 64, 65, 128, 129, 512],
        false,
        false,
        false,
        "packed_read::interpreter_scalar_packed_read",
    );
}
#[test]
fn cranelift_scalar_packed_read() {
    run(
        Backend::Cranelift,
        &[8, 64, 65, 128, 129, 512],
        false,
        false,
        false,
        "packed_read::cranelift_scalar_packed_read",
    );
}
#[test]
fn interpreter_array_packed_read() {
    run(
        Backend::Interpret,
        &[8, 64, 65, 128, 129, 512],
        true,
        false,
        false,
        "packed_read::interpreter_array_packed_read",
    );
}
#[test]
fn cranelift_array_packed_read() {
    run(
        Backend::Cranelift,
        &[8, 64, 65, 128],
        true,
        false,
        false,
        "packed_read::cranelift_array_packed_read",
    );
}
#[test]
fn fallback_array_packed_read() {
    run(
        Backend::Cranelift,
        &[129, 512],
        true,
        true,
        false,
        "packed_read::fallback_array_packed_read",
    );
}
