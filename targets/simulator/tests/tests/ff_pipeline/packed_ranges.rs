//! Finite innermost row-local packed ranges, clipped before physical mapping.
//! Includes selected aliased-RHS sequential blocking writes. FF/NBA updates and
//! non-innermost ranges are separate domains.

use super::four_state::value;
use super::packed_dimensions::{decode, pattern, positions};
use super::signed_array::known;
use super::{Backend, Simulator, analyze_state_config};
use std::process::Command;
use veryl_formal_semantics::logic4::Logic4;
const SHAPES: &[(&[usize], usize)] = &[(&[3, 8], 1), (&[2, 3, 16], 2), (&[2, 3, 65], 2)];
fn forms(row: usize, stride: usize) -> Vec<(&'static str, usize)> {
    let windows: &[usize] = if stride == 1 {
        &[3, 8, 64, 65]
    } else {
        &[1, 2, 3]
    };
    windows
        .iter()
        .copied()
        .filter(|&w| w <= row)
        .flat_map(|w| {
            [
                ("plus", w),
                ("minus", w),
                ("step", w),
                ("colon-low", w),
                ("colon-high", w),
            ]
        })
        .collect()
}
fn constant_case(shape: &[usize], case: usize) -> (Vec<i128>, usize) {
    let rank = shape.len() - 1;
    let mut prefix: Vec<_> = if case == 1 {
        shape[..rank].iter().map(|&n| n as i128 - 1).collect()
    } else {
        vec![0; rank]
    };
    if case == 2 {
        prefix[rank - 1] = shape[rank - 1] as i128;
    }
    (
        prefix,
        match case {
            0 => 0,
            1 => shape[rank] - 1,
            _ => 1,
        },
    )
}
fn expected(
    data: &[Logic4],
    patch: &[Logic4],
    at: &[usize],
    start: Option<i128>,
    four: bool,
    write: bool,
    alias: bool,
) -> Vec<Logic4> {
    if write {
        let mut result = data.to_vec();
        if let Some(start) = start {
            for (b, &v) in patch.iter().enumerate() {
                if let Ok(local) = usize::try_from(start + b as i128)
                    && let Some(&dst) = at.get(local)
                {
                    result[dst] = v;
                }
            }
        }
        if alias {
            let copy = result[..patch.len()].to_vec();
            for (&dst, v) in at.iter().take(patch.len()).zip(copy) {
                result[dst] = v;
            }
        }
        result
    } else {
        (0..patch.len())
            .map(|b| {
                start
                    .and_then(|n| usize::try_from(n + b as i128).ok())
                    .and_then(|i| at.get(i))
                    .map(|&i| data[i])
                    .unwrap_or(if four { Logic4::X } else { Logic4::Zero })
            })
            .collect()
    }
}
fn source(
    shape: &[usize],
    axis: usize,
    bits: usize,
    signed: bool,
    form: &str,
    window: usize,
    write: bool,
) -> String {
    let full: usize = shape.iter().product();
    let dims = shape
        .iter()
        .map(usize::to_string)
        .collect::<Vec<_>>()
        .join(",");
    let sign = if signed { "signed " } else { "" };
    let mut ports = (0..=axis)
        .map(|i| format!("idx_{i}: input {sign}logic<{bits}>, "))
        .collect::<String>();
    let prefix = (0..axis).map(|i| format!("[idx_{i}]")).collect::<String>();
    let index = format!("idx_{axis}");
    let row = shape[axis];
    let stride: usize = shape[axis + 1..].iter().product();
    let patch_bits = window * stride;
    let range = match form {
        "plus" => format!("[{index}+:{window}]"),
        "minus" => format!("[{index}-:{window}]"),
        "step" => format!("[{index} step {window}]"),
        "colon-low" => format!("[{}:0]", window - 1),
        _ => format!("[{}:{}]", row - 1, row - window),
    };
    let (output, mut body) = if write {
        (
            full,
            format!(
                "var q: logic<{dims}>;always_comb {{q = data;q{prefix}{range} = patch;if alias_en {{q{prefix}[0+:{window}] = q;}}}}assign out = q;"
            ),
        )
    } else {
        (patch_bits, format!("assign out = data{prefix}{range};"))
    };
    for case in 0..3 {
        let (coordinates, column) = constant_case(&shape[..=axis], case);
        let last_prefix = coordinates.len() - 1;
        let constant_prefix: String = coordinates
            .iter()
            .enumerate()
            .map(|(i, n)| {
                if i == last_prefix {
                    "[k]".into()
                } else {
                    format!("[{n}]")
                }
            })
            .collect();
        let constant_range = range.replace(&index, "j");
        let statement = if write {
            body += &format!(
                "var q_const_{case}: logic<{dims}>;assign const_out_{case} = q_const_{case};"
            );
            let alias_write = if case == 2 {
                String::new()
            } else {
                format!(
                    "if alias_en {{q_const_{case}{constant_prefix}[0+:{window}] = q_const_{case};}}"
                )
            };
            format!("q_const_{case}{constant_prefix}{constant_range} = patch;{alias_write}")
        } else {
            format!("const_out_{case} = data{constant_prefix}{constant_range};")
        };
        let statement = if form.starts_with("colon") {
            statement
        } else {
            format!("for j in {column}..{} {{{statement}}}", column + 1)
        };
        let row = coordinates[last_prefix];
        let seed = if write {
            format!("q_const_{case} = data;")
        } else {
            String::new()
        };
        body += &format!(
            "always_comb {{{seed}for k in {row}..{} {{{statement}}}}}",
            row + 1
        );
        ports += &format!("const_out_{case}: output logic<{output}>, ");
    }
    format!(
        "module Top(clk: input clock, rst: input reset, data: input logic<{dims}>, patch: input logic<{patch_bits}>, alias_en: input logic, out: output logic<{output}>, mirror: output logic<{full}>, busy: output logic<16>, {ports}) {{var count:logic<16>;always_ff {{if_reset {{count = 0;}}else{{count = count+1;}}}}assign busy = count;assign mirror = data;{body}}}"
    )
}
fn indices(shape: &[usize], bits: usize, signed: bool, window: usize) -> Vec<Vec<Vec<Logic4>>> {
    let min = if signed { -(1i128 << (bits - 1)) } else { 0 };
    let max = (1i128 << (bits - usize::from(signed))) - 1;
    let mut result = Vec::new();
    let bases = [
        vec![0i128; shape.len()],
        shape.iter().map(|&s| s as i128 - 1).collect::<Vec<_>>(),
    ];
    for base in bases {
        result.push(base.iter().map(|&n| known(n, bits)).collect());
        for axis in 0..shape.len() {
            let mut ns = vec![
                min,
                -(window as i128),
                -1,
                0,
                1,
                shape[axis] as i128 - 1,
                shape[axis] as i128,
                shape[axis] as i128 + 1,
                max,
                1i128 << 32,
                (1i128 << 64) / window as i128,
            ];
            ns.retain(|&n| n >= min && n <= max);
            ns.sort();
            ns.dedup();
            for n in ns {
                let mut c = base.iter().map(|&n| known(n, bits)).collect::<Vec<_>>();
                c[axis] = known(n, bits);
                result.push(c);
            }
            for bit in [0, bits / 2, bits - 1] {
                for state in [Logic4::X, Logic4::Z] {
                    let mut c = base.iter().map(|&n| known(n, bits)).collect::<Vec<_>>();
                    c[axis][bit] = state;
                    result.push(c);
                }
            }
        }
    }
    result
}
fn compare(backend: Backend, shapes: &[(&[usize], usize)], states: &[bool], write: bool) {
    let mut configs = 0;
    let mut checks = 0;
    for &(shape, axis) in shapes {
        let full = shape.iter().product();
        let row = shape[axis];
        let stride: usize = shape[axis + 1..].iter().product();
        for (form, window) in forms(row, stride) {
            for bits in [8, 64] {
                for signed in [false, true] {
                    for &four in states {
                        let (ir, _) = analyze_state_config(
                            &source(shape, axis, bits, signed, form, window, write),
                            backend,
                            false,
                            four,
                        );
                        let (cc, ci, ec, ei) = ir.detailed_stats();
                        eprintln!(
                            "packed-range route {backend:?} shape={shape:?} axis={axis} bits={bits} signed={signed} four={four} form={form} window={window} write={write} comb={cc}/{ci} events={ec}/{ei}"
                        );
                        let mut sim = Simulator::new(ir, None);
                        let clock = sim.get_clock("clk").unwrap();
                        let reset = sim.get_reset("rst").unwrap();
                        sim.step_reset(&clock, &reset);
                        for mode in 0..4 {
                            let data = pattern(full, mode, four);
                            let patch = pattern(window * stride, (mode + 1) % 4, four);
                            sim.set("data", value(&data));
                            sim.set("patch", value(&patch));
                            sim.set("alias_en", super::Value::new((mode % 2) as u64, 1, false));
                            for case in 0..3 {
                                let (prefix, column) = constant_case(&shape[..=axis], case);
                                let at = positions(shape, &prefix);
                                let start = Some(
                                    (match form {
                                        "minus" => column as i128 - window as i128 + 1,
                                        "step" => column as i128 * window as i128,
                                        "colon-low" => 0,
                                        "colon-high" => (row - window) as i128,
                                        _ => column as i128,
                                    }) * stride as i128,
                                );
                                let want =
                                    expected(&data, &patch, &at, start, four, write, mode % 2 == 1);
                                assert_eq!(
                                    sim.get(&format!("const_out_{case}")),
                                    Some(value(&want)),
                                    "simulator packed-range output mismatch: constant {backend:?} shape={shape:?} axis={axis} form={form} window={window} mode={mode} case={case} four={four} write={write}"
                                );
                                checks += 1;
                            }
                            for c in indices(&shape[..=axis], bits, signed, window) {
                                for (i, n) in c.iter().enumerate() {
                                    sim.set(&format!("idx_{i}"), value(n));
                                }
                                let coords: Option<Vec<_>> = c[..c.len() - 1]
                                    .iter()
                                    .map(|b| decode(b, signed, four))
                                    .collect();
                                let at = coords.map(|p| positions(shape, &p)).unwrap_or_default();
                                let start = match form {
                                    "colon-low" => Some(0),
                                    "colon-high" => Some((row - window) as i128),
                                    _ => decode(c.last().unwrap(), signed, four).map(
                                        |n| match form {
                                            "minus" => n - window as i128 + 1,
                                            "step" => n * window as i128,
                                            _ => n,
                                        },
                                    ),
                                };
                                let start = start.map(|n| n * stride as i128);
                                let expected =
                                    expected(&data, &patch, &at, start, four, write, mode % 2 == 1);
                                assert_eq!(
                                    sim.get("out"),
                                    Some(value(&expected)),
                                    "simulator packed-range output mismatch: {backend:?} shape={shape:?} axis={axis} bits={bits} signed={signed} four={four} write={write} form={form} window={window} mode={mode} index={c:?}"
                                );
                                checks += 1;
                                assert_eq!(
                                    sim.get("mirror"),
                                    Some(value(&data)),
                                    "simulator packed-range output mismatch: source changed"
                                );
                                assert_eq!(
                                    sim.get("busy"),
                                    Some(super::Value::new(mode as u64, 16, false)),
                                    "simulator packed-range output mismatch: clock advanced"
                                );
                            }
                            sim.step(&clock);
                        }
                        if backend == Backend::Cranelift {
                            if write && four && full > 128 {
                                assert!(ci > 0, "packed-range wide four-state fallback absent");
                            } else {
                                assert!(cc > 0 && ci == 0, "packed-range compiled route absent");
                            }
                        }
                        configs += 1;
                    }
                }
            }
        }
    }
    eprintln!("packed-range complete configurations={configs} comparisons={checks}");
}
pub(super) fn run(
    backend: Backend,
    shapes: &[(&[usize], usize)],
    states: &[bool],
    write: bool,
    name: &str,
) {
    if std::env::var_os("VERYL_FORMAL_PACKED_RANGE_CHILD").is_some() {
        compare(backend, shapes, states, write);
        return;
    }
    let out = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", name, "--nocapture"])
        .env("VERYL_FORMAL_PACKED_RANGE_CHILD", "1")
        .output()
        .unwrap();
    let log = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(out.status.success(), "packed-range child failed:\n{log}");
    assert!(log.contains("1 passed; 0 failed") && log.contains("packed-range complete"));
    eprintln!("{log}");
}
#[test]
fn interpreter_packed_range_read() {
    run(
        Backend::Interpret,
        SHAPES,
        &[true, false],
        false,
        "packed_ranges::interpreter_packed_range_read",
    );
}
#[test]
fn cranelift_packed_range_read() {
    run(
        Backend::Cranelift,
        SHAPES,
        &[true, false],
        false,
        "packed_ranges::cranelift_packed_range_read",
    );
}
#[test]
fn interpreter_packed_range_write() {
    run(
        Backend::Interpret,
        SHAPES,
        &[true, false],
        true,
        "packed_ranges::interpreter_packed_range_write",
    );
}
#[test]
fn cranelift_packed_range_write() {
    run(
        Backend::Cranelift,
        &SHAPES[..2],
        &[true, false],
        true,
        "packed_ranges::cranelift_packed_range_write",
    );
}
#[test]
fn cranelift_wide_packed_range_write() {
    run(
        Backend::Cranelift,
        &SHAPES[2..],
        &[false],
        true,
        "packed_ranges::cranelift_wide_packed_range_write",
    );
}
#[test]
fn fallback_packed_range_write() {
    run(
        Backend::Cranelift,
        &SHAPES[2..],
        &[true],
        true,
        "packed_ranges::fallback_packed_range_write",
    );
}
