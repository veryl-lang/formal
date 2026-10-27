//! Finite packed multidimensional prefixes with independent physical-bit
//! enumeration and per-axis validity. No arbitrary-rank or ranged-selection theorem.

use super::four_state::{STATES, value};
use super::signed_array::known;
use super::{Backend, Simulator, analyze_state_config};
use std::process::Command;
use veryl_formal_semantics::logic4::Logic4;
const SHAPES: &[&[usize]] = &[&[3, 16], &[2, 3, 16], &[4, 2, 8], &[2, 3, 65]];
pub(super) fn positions(shape: &[usize], prefix: &[i128]) -> Vec<usize> {
    if prefix
        .iter()
        .zip(shape)
        .any(|(&n, &size)| n < 0 || n >= size as i128)
    {
        return vec![];
    }
    (0..shape.iter().product())
        .filter(|&bit| {
            let mut left = bit;
            let mut coordinates = vec![0; shape.len()];
            for i in (0..shape.len()).rev() {
                coordinates[i] = left % shape[i];
                left /= shape[i];
            }
            coordinates
                .iter()
                .zip(prefix)
                .all(|(&a, &b)| a as i128 == b)
        })
        .collect()
}
fn source(shape: &[usize], bits: usize, signed: bool, write: bool) -> String {
    let full: usize = shape.iter().product();
    let dimensions = shape
        .iter()
        .map(usize::to_string)
        .collect::<Vec<_>>()
        .join(",");
    let sign = if signed { "signed " } else { "" };
    let mut ports = String::new();
    let mut statements = String::new();
    for axis in 0..shape.len() {
        ports += &format!("idx_{axis}: input {sign}logic<{bits}>, ");
    }
    for depth in 1..=shape.len() {
        let window: usize = shape[depth..].iter().product();
        let select = (0..depth).map(|i| format!("[idx_{i}]")).collect::<String>();
        if write {
            ports += &format!(
                "patch_{depth}: input logic<{window}>, out_{depth}: output logic<{full}>, "
            );
            statements += &format!(
                "var q_{depth}: logic<{dimensions}>;always_comb {{q_{depth} = data;q_{depth}{select} = patch_{depth};}}assign out_{depth} = q_{depth};"
            );
        } else {
            ports += &format!("out_{depth}: output logic<{window}>, ");
            statements += &format!("assign out_{depth} = data{select};");
        }
    }
    format!(
        "module Top(clk: input clock, rst: input reset, data: input logic<{dimensions}>, mirror: output logic<{full}>, busy: output logic<16>, {ports}) {{var count: logic<16>;always_ff {{if_reset {{count = 0;}} else {{count = count+1;}}}}assign busy = count;assign mirror = data;{statements}}}"
    )
}
fn cases(shape: &[usize], bits: usize, signed: bool) -> Vec<Vec<Vec<Logic4>>> {
    let min = if signed { -(1i128 << (bits - 1)) } else { 0 };
    let max = (1i128 << (bits - usize::from(signed))) - 1;
    let mut result = Vec::new();
    fn all(shape: &[usize], prefix: &mut Vec<i128>, output: &mut Vec<Vec<i128>>) {
        if let Some((&size, rest)) = shape.split_first() {
            for n in 0..size {
                prefix.push(n as i128);
                all(rest, prefix, output);
                prefix.pop();
            }
        } else {
            output.push(prefix.clone());
        }
    }
    let mut valid = Vec::new();
    all(shape, &mut vec![], &mut valid);
    for v in valid.into_iter().filter(|v| v.iter().all(|&n| n <= max)) {
        result.push(v.into_iter().map(|n| known(n, bits)).collect());
    }
    for axis in 0..shape.len() {
        let mut numbers = vec![min, -1, shape[axis] as i128, shape[axis] as i128 + 1, max];
        numbers.extend([1i128 << 32, (1i128 << 32) + 1, 1i128 << 63]);
        numbers.retain(|&n| n >= min && n <= max);
        numbers.sort();
        numbers.dedup();
        for base in [
            vec![0; shape.len()],
            shape.iter().map(|&n| (n as i128 - 1).min(max)).collect(),
        ] {
            for &n in &numbers {
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
pub(super) fn decode(bits: &[Logic4], signed: bool, four: bool) -> Option<i128> {
    if four && bits.iter().any(|b| matches!(b, Logic4::X | Logic4::Z)) {
        return None;
    }
    let n = bits.iter().enumerate().fold(0i128, |n, (i, b)| {
        n | if matches!(b, Logic4::One | Logic4::Z) {
            1i128 << i
        } else {
            0
        }
    });
    Some(if signed && n & (1i128 << (bits.len() - 1)) != 0 {
        n - (1i128 << bits.len())
    } else {
        n
    })
}
pub(super) fn pattern(width: usize, n: usize, four: bool) -> Vec<Logic4> {
    (0..width)
        .map(|i| match n {
            0 => Logic4::Zero,
            1 => Logic4::One,
            _ => STATES[(i + i / 64 + n) % if four { 4 } else { 2 }],
        })
        .collect()
}
fn constant_cases(shape: &[usize]) -> Vec<Vec<Vec<Logic4>>> {
    let mut result = Vec::new();
    for base in [
        vec![0; shape.len()],
        shape.iter().map(|&n| n as i128 - 1).collect(),
    ] {
        result.push(base.iter().map(|&n| known(n, 32)).collect());
        for axis in 0..shape.len() {
            for n in [
                0,
                shape[axis] as i128 - 1,
                shape[axis] as i128,
                shape[axis] as i128 + 1,
            ] {
                let mut row: Vec<_> = base.iter().map(|&n| known(n, 32)).collect();
                row[axis] = known(n, 32);
                result.push(row);
            }
            for bit in [0, 31] {
                for state in [Logic4::X, Logic4::Z] {
                    for ones in [false, true] {
                        let mut row: Vec<_> = base.iter().map(|&n| known(n, 32)).collect();
                        row[axis] = known(if ones { -1 } else { 0 }, 32);
                        row[axis][bit] = state;
                        result.push(row);
                    }
                }
            }
        }
    }
    result
}
fn constant_source(shape: &[usize], cases: &[Vec<Vec<Logic4>>], write: bool) -> String {
    let full: usize = shape.iter().product();
    let dimensions = shape
        .iter()
        .map(usize::to_string)
        .collect::<Vec<_>>()
        .join(",");
    let mut ports = String::new();
    let mut body = String::new();
    for depth in 1..=shape.len() {
        let window: usize = shape[depth..].iter().product();
        if write {
            ports += &format!("patch_{depth}: input logic<{window}>, ");
        }
        for (i, c) in cases.iter().enumerate() {
            let mut unroll = false;
            let select: String = c[..depth]
                .iter()
                .map(|bits| {
                    if bits.iter().any(|b| matches!(b, Logic4::X | Logic4::Z)) {
                        let digits: String = bits
                            .iter()
                            .rev()
                            .map(|b| match b {
                                Logic4::Zero => '0',
                                Logic4::One => '1',
                                Logic4::X => 'x',
                                Logic4::Z => 'z',
                            })
                            .collect();
                        format!("[32'b{digits}]")
                    } else {
                        unroll = true;
                        format!("[j+{}]", decode(bits, false, false).unwrap())
                    }
                })
                .collect();
            let output = format!("out_{i}_{depth}");
            let statement = if write {
                ports += &format!("{output}: output logic<{full}>, ");
                body += &format!(
                    "var q_{i}_{depth}: logic<{dimensions}>; assign {output} = q_{i}_{depth};"
                );
                format!("q_{i}_{depth}{select} = patch_{depth};")
            } else {
                ports += &format!("{output}: output logic<{window}>, ");
                format!("{output} = data{select};")
            };
            let statement = if unroll {
                format!("for j in 0..1 {{{statement}}}")
            } else {
                statement
            };
            let seed = if write {
                format!("q_{i}_{depth} = data;")
            } else {
                String::new()
            };
            body += &format!("always_comb {{{seed}{statement}}}");
        }
    }
    format!(
        "module Top(clk: input clock, rst: input reset, data: input logic<{dimensions}>, mirror: output logic<{full}>, busy: output logic<16>, {ports}) {{var count: logic<16>;always_ff {{if_reset {{count = 0;}} else {{count = count+1;}}}}assign busy = count;assign mirror = data;{body}}}"
    )
}
fn compare_constants(
    backend: Backend,
    shapes: &[&[usize]],
    states: &[bool],
    write: bool,
    fallback: bool,
) -> (usize, usize) {
    let mut configs = 0;
    let mut comparisons = 0;
    for &shape in shapes {
        let full: usize = shape.iter().product();
        let cases = constant_cases(shape);
        for &four in states {
            let (ir, _) =
                analyze_state_config(&constant_source(shape, &cases, write), backend, false, four);
            let (cc, ci, _, _) = ir.detailed_stats();
            if backend == Backend::Cranelift {
                if fallback {
                    assert!(ci > 0, "packed-dimension constant fallback absent");
                } else {
                    assert!(
                        cc > 0 && ci == 0,
                        "packed-dimension constant compiled route absent"
                    );
                }
            }
            let mut sim = Simulator::new(ir, None);
            let clock = sim.get_clock("clk").unwrap();
            let reset = sim.get_reset("rst").unwrap();
            sim.step_reset(&clock, &reset);
            for mode in 0..4 {
                let data = pattern(full, mode, four);
                sim.set("data", value(&data));
                for depth in 1..=shape.len() {
                    if write {
                        sim.set(
                            &format!("patch_{depth}"),
                            value(&pattern(
                                shape[depth..].iter().product(),
                                (mode + depth + 1) % 4,
                                four,
                            )),
                        );
                    }
                }
                for (i, c) in cases.iter().enumerate() {
                    for depth in 1..=shape.len() {
                        let window = shape[depth..].iter().product();
                        let prefix: Option<Vec<_>> =
                            c[..depth].iter().map(|b| decode(b, false, four)).collect();
                        let at = prefix.map(|p| positions(shape, &p)).unwrap_or_default();
                        let expected = if write {
                            let mut result = data.clone();
                            for (&dst, v) in
                                at.iter().zip(pattern(window, (mode + depth + 1) % 4, four))
                            {
                                result[dst] = v;
                            }
                            result
                        } else if at.is_empty() {
                            vec![if four { Logic4::X } else { Logic4::Zero }; window]
                        } else {
                            at.iter().map(|&b| data[b]).collect()
                        };
                        assert_eq!(
                            sim.get(&format!("out_{i}_{depth}")),
                            Some(value(&expected)),
                            "simulator packed-dimension output mismatch: constant {backend:?} shape={shape:?} four={four} write={write} mode={mode} depth={depth} index={c:?}"
                        );
                        comparisons += 1;
                    }
                }
                assert_eq!(
                    sim.get("mirror"),
                    Some(value(&data)),
                    "simulator packed-dimension output mismatch: constant source changed"
                );
                assert_eq!(
                    sim.get("busy"),
                    Some(super::Value::new(mode as u64, 16, false)),
                    "simulator packed-dimension output mismatch: constant clock advanced"
                );
                sim.step(&clock);
            }
            configs += 1;
        }
    }
    eprintln!("packed-dimension constants configurations={configs} comparisons={comparisons}");
    (configs, comparisons)
}
fn compare(backend: Backend, shapes: &[&[usize]], states: &[bool], write: bool, fallback: bool) {
    let (mut configs, mut comparisons) =
        compare_constants(backend, shapes, states, write, fallback);
    for &shape in shapes {
        let full: usize = shape.iter().product();
        for bits in [2, 8, 64] {
            for signed in [false, true] {
                for &four in states {
                    let (ir, _) = analyze_state_config(
                        &source(shape, bits, signed, write),
                        backend,
                        false,
                        four,
                    );
                    let (cc, ci, ec, ei) = ir.detailed_stats();
                    if backend == Backend::Cranelift {
                        if fallback {
                            assert!(ci > 0, "packed-dimension comb fallback absent");
                        } else {
                            assert!(cc > 0 && ci == 0, "packed-dimension compiled comb absent");
                        }
                    }
                    eprintln!(
                        "packed-dimension route {backend:?} shape={shape:?} bits={bits} signed={signed} four={four} write={write} fallback={fallback} comb={cc}/{ci} events={ec}/{ei}"
                    );
                    let mut sim = Simulator::new(ir, None);
                    let clock = sim.get_clock("clk").unwrap();
                    let reset = sim.get_reset("rst").unwrap();
                    sim.step_reset(&clock, &reset);
                    let cases = cases(shape, bits, signed);
                    for mode in 0..4 {
                        let data = pattern(full, mode, four);
                        sim.set("data", value(&data));
                        for depth in 1..=shape.len() {
                            let w = shape[depth..].iter().product();
                            if write {
                                sim.set(
                                    &format!("patch_{depth}"),
                                    value(&pattern(w, (mode + depth + 1) % 4, four)),
                                );
                            }
                        }
                        for c in &cases {
                            for (i, v) in c.iter().enumerate() {
                                sim.set(&format!("idx_{i}"), value(v));
                            }
                            for depth in 1..=shape.len() {
                                let window = shape[depth..].iter().product();
                                let decoded: Option<Vec<_>> =
                                    c[..depth].iter().map(|n| decode(n, signed, four)).collect();
                                let at = decoded.map(|c| positions(shape, &c)).unwrap_or_default();
                                let expected = if write {
                                    let mut result = data.clone();
                                    let patch = pattern(window, (mode + depth + 1) % 4, four);
                                    for (&dst, v) in at.iter().zip(patch) {
                                        result[dst] = v;
                                    }
                                    result
                                } else if at.is_empty() {
                                    vec![if four { Logic4::X } else { Logic4::Zero }; window]
                                } else {
                                    at.iter().map(|&bit| data[bit]).collect()
                                };
                                assert_eq!(
                                    sim.get(&format!("out_{depth}")),
                                    Some(value(&expected)),
                                    "simulator packed-dimension output mismatch: {backend:?} shape={shape:?} bits={bits} signed={signed} four={four} write={write} mode={mode} depth={depth} index={c:?}"
                                );
                                comparisons += 1;
                            }
                            assert_eq!(
                                sim.get("mirror"),
                                Some(value(&data)),
                                "simulator packed-dimension output mismatch: source changed"
                            );
                            assert_eq!(
                                sim.get("busy"),
                                Some(super::Value::new(mode as u64, 16, false)),
                                "simulator packed-dimension output mismatch: clock advanced"
                            );
                        }
                        sim.step(&clock);
                    }
                    configs += 1;
                }
            }
        }
    }
    eprintln!("packed-dimension complete configurations={configs} comparisons={comparisons}");
}
fn run(
    backend: Backend,
    shapes: &[&[usize]],
    states: &[bool],
    write: bool,
    fallback: bool,
    name: &str,
) {
    if std::env::var_os("VERYL_FORMAL_PACKED_DIM_CHILD").is_some() {
        compare(backend, shapes, states, write, fallback);
        return;
    }
    let out = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", name, "--nocapture"])
        .env("VERYL_FORMAL_PACKED_DIM_CHILD", "1")
        .output()
        .unwrap();
    let log = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        out.status.success(),
        "packed-dimension child failed:\n{log}"
    );
    assert!(log.contains("1 passed; 0 failed") && log.contains("packed-dimension complete"));
    eprintln!("{log}");
}
#[test]
fn interpreter_packed_dimension_read() {
    run(
        Backend::Interpret,
        SHAPES,
        &[true, false],
        false,
        false,
        "packed_dimensions::interpreter_packed_dimension_read",
    );
}
#[test]
fn cranelift_packed_dimension_read() {
    run(
        Backend::Cranelift,
        SHAPES,
        &[true, false],
        false,
        false,
        "packed_dimensions::cranelift_packed_dimension_read",
    );
}
#[test]
fn interpreter_packed_dimension_write() {
    run(
        Backend::Interpret,
        SHAPES,
        &[true, false],
        true,
        false,
        "packed_dimensions::interpreter_packed_dimension_write",
    );
}
#[test]
fn cranelift_packed_dimension_write() {
    run(
        Backend::Cranelift,
        &SHAPES[..3],
        &[true, false],
        true,
        false,
        "packed_dimensions::cranelift_packed_dimension_write",
    );
}
#[test]
fn cranelift_wide_packed_dimension_write() {
    run(
        Backend::Cranelift,
        &SHAPES[3..],
        &[false],
        true,
        false,
        "packed_dimensions::cranelift_wide_packed_dimension_write",
    );
}
#[test]
fn fallback_packed_dimension_write() {
    run(
        Backend::Cranelift,
        &SHAPES[3..],
        &[true],
        true,
        true,
        "packed_dimensions::fallback_packed_dimension_write",
    );
}
