//! Finite accepted constant X/Z and lower-overlap selections. Two-state literal
//! mask removal is an execution convention; rejected negative loop expressions
//! are diagnostics, not executed selector cases.

use super::four_state::{STATES, value};
use super::{Backend, Simulator, analyze_state_config};
use std::process::Command;
use veryl_formal_semantics::logic4::Logic4;
#[derive(Debug)]
struct Case {
    expr: String,
    coordinate: Option<i128>,
    payload: i128,
    window: usize,
    form: &'static str,
    unroll: bool,
}
fn cases(width: usize) -> Vec<Case> {
    let mut cases = Vec::new();
    for window in [1, 3, 8, 64, 65, 128, 129]
        .into_iter()
        .filter(|&w| w <= width)
    {
        let forms = if window == 1 {
            vec!["bit"]
        } else {
            vec!["plus", "minus", "step"]
        };
        for form in forms {
            for state in ['x', 'z'] {
                for bit in [0, 31] {
                    for base in [false, true] {
                        let mut bits = [if base { '1' } else { '0' }; 32];
                        bits[31 - bit] = state;
                        let payload = bits
                            .iter()
                            .fold(0i128, |n, b| n * 2 + i128::from(matches!(b, '1' | 'z')));
                        cases.push(Case {
                            expr: format!("32'b{}", bits.iter().collect::<String>()),
                            coordinate: None,
                            payload,
                            window,
                            form,
                            unroll: false,
                        });
                    }
                }
            }
            let mut coordinates = vec![0, 1];
            if form == "plus" {
                coordinates.extend([-1, 1 - window as i128]);
            }
            if form == "step" {
                coordinates.extend([-1, -3, -65]);
            }
            for n in coordinates {
                let expr = if n < 0 {
                    format!("((j as i64) - 64'sd{})", -n)
                } else {
                    format!("((j as i64) + 64'sd{n})")
                };
                cases.push(Case {
                    expr,
                    coordinate: Some(n),
                    payload: n,
                    window,
                    form,
                    unroll: true,
                });
            }
        }
    }
    cases
}
fn source(width: usize, cases: &[Case], write: bool) -> String {
    let mut ports = String::new();
    let mut declarations = String::new();
    for (i, c) in cases.iter().enumerate() {
        let e = &c.expr;
        let w = c.window;
        let select = match c.form {
            "bit" => format!("[{e}]"),
            "plus" => format!("[{e}+:{w}]"),
            "minus" => format!("[{e}-:{w}]"),
            _ => format!("[{e} step {w}]"),
        };
        let statement = if write {
            ports += &format!("patch_{i}: input logic<{w}>, out_{i}: output logic<{width}>, ");
            declarations += &format!("var q_{i}: logic<{width}>; assign out_{i} = q_{i};");
            format!("q_{i}{select} = patch_{i};")
        } else {
            ports += &format!("out_{i}: output logic<{w}>, ");
            format!("out_{i} = data{select};")
        };
        let statement = if c.unroll {
            format!("for j in 0..1 {{{statement}}}")
        } else {
            statement
        };
        let seed = if write {
            format!("q_{i} = data;")
        } else {
            String::new()
        };
        declarations += &format!("always_comb {{{seed}{statement}}}");
    }
    format!(
        "module Top(clk: input clock, rst: input reset, data: input logic<{width}>, mirror: output logic<{width}>, busy: output logic<16>, {ports}) {{var count: logic<16>;always_ff {{if_reset {{count = 0;}} else {{count = count+1;}}}} assign busy = count;assign mirror = data;{declarations}}}"
    )
}
fn pattern(width: usize, mode: usize, four: bool) -> Vec<Logic4> {
    (0..width)
        .map(|b| match mode {
            0 => Logic4::Zero,
            1 => Logic4::One,
            _ => STATES[(b + b / 64 + mode) % if four { 4 } else { 2 }],
        })
        .collect()
}
fn compare(backend: Backend, widths: &[usize], write: bool, fallback: bool) {
    let mut configs = 0;
    let mut comparisons = 0;
    for &width in widths {
        let cases = cases(width);
        for four in [true, false] {
            let (ir, _) = analyze_state_config(&source(width, &cases, write), backend, false, four);
            let (cc, ci, ec, ei) = ir.detailed_stats();
            if backend == Backend::Cranelift {
                if fallback && four {
                    assert!(ci > 0, "constant-edge comb fallback absent");
                } else {
                    assert!(cc > 0 && ci == 0, "constant-edge compiled comb absent");
                }
            }
            eprintln!(
                "constant-edge route {backend:?} width={width} four={four} write={write} fallback={fallback} cases={} comb={cc}/{ci} events={ec}/{ei}",
                cases.len()
            );
            let mut sim = Simulator::new(ir, None);
            let clock = sim.get_clock("clk").unwrap();
            let reset = sim.get_reset("rst").unwrap();
            for epoch in 0..2 {
                sim.step_reset(&clock, &reset);
                for mode in 0..4 {
                    let data = pattern(width, (epoch + mode) % 4, four);
                    sim.set("data", value(&data));
                    for (i, c) in cases.iter().enumerate() {
                        let n = if four { c.coordinate } else { Some(c.payload) };
                        let start = n.map(|n| match c.form {
                            "minus" => n - c.window as i128 + 1,
                            "step" => n * c.window as i128,
                            _ => n,
                        });
                        let patch = pattern(c.window, (epoch + mode + i + 1) % 4, four);
                        if write {
                            sim.set(&format!("patch_{i}"), value(&patch));
                        }
                        let expected = if write {
                            let mut result = data.clone();
                            if let Some(start) = start {
                                for (b, v) in patch.into_iter().enumerate() {
                                    if let Ok(dst) = usize::try_from(start + b as i128)
                                        && let Some(dst) = result.get_mut(dst)
                                    {
                                        *dst = v;
                                    }
                                }
                            }
                            result
                        } else {
                            (0..c.window)
                                .map(|b| {
                                    start
                                        .and_then(|n| usize::try_from(n + b as i128).ok())
                                        .and_then(|n| data.get(n).copied())
                                        .unwrap_or(if four { Logic4::X } else { Logic4::Zero })
                                })
                                .collect()
                        };
                        assert_eq!(
                            sim.get(&format!("out_{i}")),
                            Some(value(&expected)),
                            "simulator constant-edge output mismatch: {backend:?} width={width} four={four} write={write} epoch={epoch} mode={mode} case={c:?}"
                        );
                        comparisons += 1;
                    }
                    assert_eq!(
                        sim.get("mirror"),
                        Some(value(&data)),
                        "simulator constant-edge output mismatch: source changed"
                    );
                    assert_eq!(
                        sim.get("busy"),
                        Some(super::Value::new(mode as u64, 16, false)),
                        "simulator constant-edge output mismatch: clock advanced"
                    );
                    sim.step(&clock);
                }
            }
            configs += 1;
        }
    }
    eprintln!("constant-edge complete configurations={configs} comparisons={comparisons}");
}
fn run(backend: Backend, widths: &[usize], write: bool, fallback: bool, name: &str) {
    if std::env::var_os("VERYL_FORMAL_CONSTANT_EDGE_CHILD").is_some() {
        compare(backend, widths, write, fallback);
        return;
    }
    let out = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", name, "--nocapture"])
        .env("VERYL_FORMAL_CONSTANT_EDGE_CHILD", "1")
        .output()
        .unwrap();
    let log = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(out.status.success(), "constant-edge child failed:\n{log}");
    assert!(log.contains("1 passed; 0 failed") && log.contains("constant-edge complete"));
    eprintln!("{log}");
}
#[test]
fn interpreter_constant_edge_read() {
    run(
        Backend::Interpret,
        &[8, 64, 65, 128, 129, 512],
        false,
        false,
        "constant_edges::interpreter_constant_edge_read",
    );
}
#[test]
fn cranelift_constant_edge_read() {
    run(
        Backend::Cranelift,
        &[8, 64, 65, 128, 129, 512],
        false,
        false,
        "constant_edges::cranelift_constant_edge_read",
    );
}
#[test]
fn interpreter_constant_edge_write() {
    run(
        Backend::Interpret,
        &[8, 64, 65, 128, 129, 512],
        true,
        false,
        "constant_edges::interpreter_constant_edge_write",
    );
}
#[test]
fn cranelift_constant_edge_write() {
    run(
        Backend::Cranelift,
        &[8, 64, 65, 128],
        true,
        false,
        "constant_edges::cranelift_constant_edge_write",
    );
}
#[test]
fn fallback_constant_edge_write() {
    run(
        Backend::Cranelift,
        &[129, 512],
        true,
        true,
        "constant_edges::fallback_constant_edge_write",
    );
}

#[test]
fn negative_constant_edge_diagnostics() {
    use veryl_analyzer::{Analyzer, AnalyzerError, Context, ir as air};
    use veryl_metadata::Metadata;
    use veryl_parser::Parser;
    if std::env::var_os("VERYL_FORMAL_CONSTANT_EDGE_DIAG_CHILD").is_none() {
        let out = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "constant_edges::negative_constant_edge_diagnostics",
                "--nocapture",
            ])
            .env("VERYL_FORMAL_CONSTANT_EDGE_DIAG_CHILD", "1")
            .output()
            .unwrap();
        let log = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(out.status.success(), "negative edge child failed:\n{log}");
        assert!(
            log.contains("1 passed; 0 failed")
                && log.contains("constant-edge diagnostics complete cases=48")
        );
        eprintln!("{log}");
        return;
    }
    for width in [8, 129] {
        for write in [false, true] {
            for form in ["bit", "plus", "minus", "step"] {
                for n in [1, 3, 65] {
                    let c = Case {
                        expr: format!("((j as i64) - 64'sd{n})"),
                        coordinate: Some(-n),
                        payload: -n,
                        window: if form == "bit" { 1 } else { 3 },
                        form,
                        unroll: true,
                    };
                    let text = source(width, &[c], write);
                    let analyzer = Analyzer::new(&Metadata::create_default("prj").unwrap());
                    analyzer.clear();
                    let parser = Parser::parse(&text, &"").unwrap();
                    let mut context = Context::default();
                    let mut ir = air::Ir::default();
                    let mut errors = analyzer.analyze_pass1("prj", &parser.veryl);
                    errors.extend(Analyzer::analyze_post_pass1());
                    errors.extend(analyzer.analyze_pass2(
                        &parser.veryl,
                        &mut context,
                        Some(&mut ir),
                    ));
                    errors.extend(Analyzer::analyze_post_pass2(&ir));
                    let kinds: Vec<_> = errors
                        .iter()
                        .map(|e| match e {
                            AnalyzerError::ExceedLimit { .. } => "limit",
                            AnalyzerError::InvalidSelect { .. } => "select",
                            _ => "other",
                        })
                        .collect();
                    let expected = if form == "step" || (form == "plus" && n == 1) {
                        vec![]
                    } else {
                        vec!["limit"]
                    };
                    assert_eq!(
                        kinds, expected,
                        "simulator constant-edge diagnostic mismatch: width={width} write={write} form={form} n=-{n}"
                    );
                    eprintln!(
                        "negative-edge diagnostics width={width} write={write} form={form} n=-{n} kinds={kinds:?}"
                    );
                }
            }
        }
    }
    eprintln!("constant-edge diagnostics complete cases=48");
}
