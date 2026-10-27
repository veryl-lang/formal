//! Finite constant selections after loop expansion and separate direct-source
//! diagnostics. Rejected sources are not evidence about runtime selector behavior.

use super::four_state::{STATES, value};
use super::{Backend, Simulator, analyze_state_config};
use std::process::Command;
use veryl_formal_semantics::logic4::Logic4;

#[derive(Clone, Debug)]
struct Selection {
    form: &'static str,
    coordinate: usize,
    window: usize,
    start: usize,
}
fn selections(width: usize) -> Vec<Selection> {
    let mut result = Vec::new();
    let mut bits = vec![0, 1, width - 1, width, width + 1, 63, 64];
    bits.sort();
    bits.dedup();
    for n in bits {
        result.push(Selection {
            form: "bit",
            coordinate: n,
            window: 1,
            start: n,
        });
    }
    for w in [3, 8, 64, 65, 128, 129, 512]
        .into_iter()
        .filter(|&w| w <= width)
    {
        for n in [0, width - 1, width] {
            result.push(Selection {
                form: "plus",
                coordinate: n,
                window: w,
                start: n,
            });
        }
        for n in [0, width - w] {
            result.push(Selection {
                form: "colon",
                coordinate: n,
                window: w,
                start: n,
            });
        }
        for n in [w - 1, width - 1, width + w - 1] {
            result.push(Selection {
                form: "minus",
                coordinate: n,
                window: w,
                start: n + 1 - w,
            });
        }
        for n in [0, (width - 1) / w, width.div_ceil(w)] {
            result.push(Selection {
                form: "step",
                coordinate: n,
                window: w,
                start: n * w,
            });
        }
    }
    result
}
fn source(width: usize, cases: &[Selection], write: bool) -> String {
    let mut ports = String::new();
    let mut declarations = String::new();
    let mut stores = String::new();
    for (i, c) in cases.iter().enumerate() {
        let w = c.window;
        let sel = match c.form {
            "bit" => "[j]".into(),
            "plus" => format!("[j+:{w}]"),
            "minus" => format!("[j-:{w}]"),
            "step" => format!("[j step {w}]"),
            _ => format!("[{}:{}]", c.start + w - 1, c.start),
        };
        let n = c.coordinate;
        let end = n + 1;
        if write {
            ports += &format!("patch_{i}: input logic<{w}>, o{i}: output logic<{width}>, ");
            declarations += &format!("var q{i}: logic<{width}>; assign o{i} = q{i};");
            if c.form == "colon" {
                stores += &format!("always_comb {{q{i} = data; q{i}{sel} = patch_{i};}}");
            } else {
                stores += &format!(
                    "always_comb {{q{i} = data; for j in {n}..{end} {{q{i}{sel} = patch_{i};}}}}"
                );
            }
        } else {
            ports += &format!("o{i}: output logic<{w}>, ");
            if c.form == "colon" {
                declarations += &format!("assign o{i} = data{sel};");
            } else {
                declarations +=
                    &format!("always_comb {{ for j in {n}..{end} {{o{i} = data{sel};}} }}");
            }
        }
    }
    declarations += &stores;
    declarations += "assign guard = data;";
    format!(
        r#"module Top(clk: input clock, rst: input reset, data: input logic<{width}>,
        mirror: output logic<{width}>, busy: output logic<16>, {ports}) {{
        var guard: logic<{width}>; var count: logic<16>;
        always_ff {{if_reset {{count = 0;}} else {{count = count + 1;}}}}
        assign busy = count; assign mirror = guard; {declarations}
    }}"#
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
fn assert_static(stmts: &[veryl_simulator::ir::Statement]) {
    use veryl_simulator::ir::{Expression as E, Statement as S};
    fn expr(e: &E) {
        match e {
            E::Variable { dynamic_select, .. } => {
                assert!(dynamic_select.is_none(), "constant-select remained dynamic")
            }
            E::Value { .. } => {}
            E::Unary { x, .. } | E::Resize { x, .. } => expr(x),
            E::Binary { x, y, .. } => {
                expr(x);
                expr(y);
            }
            E::Concatenation { elements, .. } => {
                for (x, _, _) in elements {
                    expr(x);
                }
            }
            E::Ternary {
                cond,
                true_expr,
                false_expr,
                ..
            } => {
                expr(cond);
                expr(true_expr);
                expr(false_expr);
            }
            E::DynamicVariable { .. } => panic!("constant-select unexpectedly used an array index"),
        }
    }
    for s in stmts {
        match s {
            S::Assign(a) => {
                assert!(
                    a.dynamic_select.is_none(),
                    "constant-select store remained dynamic"
                );
                expr(&a.expr);
            }
            S::SequentialBlock(body) => assert_static(body),
            _ => panic!("constant-select fixture retained a loop or unexpected statement"),
        }
    }
}
fn compare(backend: Backend, widths: &[usize], states: &[bool], write: bool) {
    let mut configurations = 0;
    let mut comparisons = 0;
    let mut clocks = 0;
    for &width in widths {
        let cases = selections(width);
        for &four in states {
            {
                let forced = false;
                let (ir, _) = analyze_state_config(
                    &source(width, &cases, write),
                    if backend == Backend::CraneliftFallback {
                        Backend::Cranelift
                    } else {
                        backend
                    },
                    forced,
                    four,
                );
                if backend == Backend::Interpret {
                    assert_static(&ir.comb_statements);
                }
                let (cc, ci, ec, ei) = ir.detailed_stats();
                if backend == Backend::Cranelift {
                    assert!(
                        cc > 0 && ci == 0,
                        "constant-select compiled comb route missing"
                    );
                }
                if backend == Backend::CraneliftFallback {
                    assert!(ci > 0, "constant-select comb fallback missing");
                }
                eprintln!(
                    "constant-select route {backend:?} width={width} four={four} write={write} forced={forced} cases={} comb={cc}/{ci} events={ec}/{ei}",
                    cases.len()
                );
                let mut sim = Simulator::new(ir, None);
                let clock = sim.get_clock("clk").unwrap();
                let reset = sim.get_reset("rst").unwrap();
                for epoch in 0..2 {
                    sim.step_reset(&clock, &reset);
                    for mode in 0..4 {
                        let data = pattern(width, (mode + epoch) % 4, four);
                        sim.set("data", value(&data));
                        let mut expected = Vec::new();
                        if write {
                            for (i, c) in cases.iter().enumerate() {
                                let patch = pattern(c.window, (mode + epoch + i + 1) % 4, four);
                                sim.set(&format!("patch_{i}"), value(&patch));
                                let mut after = data.clone();
                                for (b, v) in patch.into_iter().enumerate() {
                                    if let Some(dst) = after.get_mut(c.start + b) {
                                        *dst = v;
                                    }
                                }
                                expected.push(after);
                            }
                        } else {
                            let default = if four { Logic4::X } else { Logic4::Zero };
                            expected = cases
                                .iter()
                                .map(|c| {
                                    (0..c.window)
                                        .map(|b| data.get(c.start + b).copied().unwrap_or(default))
                                        .collect()
                                })
                                .collect();
                        }
                        for (i, exp) in expected.iter().enumerate() {
                            assert_eq!(
                                sim.get(&format!("o{i}")),
                                Some(value(exp)),
                                "simulator constant-select output mismatch: {backend:?} width={width} four={four} write={write} forced={forced} epoch={epoch} mode={mode} case={:?}",
                                cases[i]
                            );
                            comparisons += 1;
                        }
                        assert_eq!(
                            sim.get("mirror"),
                            Some(value(&data)),
                            "simulator constant-select output mismatch: guard"
                        );
                        assert_eq!(
                            sim.get("busy"),
                            Some(super::Value::new(mode as u64, 16, false)),
                            "simulator constant-select output mismatch: clock"
                        );
                        sim.step(&clock);
                        clocks += 1;
                    }
                }
                configurations += 1;
            }
        }
    }
    eprintln!(
        "constant-select complete configurations={configurations} comparisons={comparisons} clocks={clocks}"
    );
}
fn run(backend: Backend, widths: &[usize], states: &[bool], write: bool, harness: &str) {
    if std::env::var_os("VERYL_FORMAL_CONSTANT_SELECT_CHILD").is_some() {
        compare(backend, widths, states, write);
        return;
    }
    let out = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", harness, "--nocapture"])
        .env("VERYL_FORMAL_CONSTANT_SELECT_CHILD", "1")
        .output()
        .unwrap();
    let log = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(out.status.success(), "constant-select child failed:\n{log}");
    assert!(
        log.contains("1 passed; 0 failed") && log.contains("constant-select complete"),
        "constant-select child incomplete"
    );
    eprintln!("{log}");
}
#[test]
fn interpreter_constant_read() {
    run(
        Backend::Interpret,
        &[8, 64, 65, 128, 129, 512],
        &[true, false],
        false,
        "constant_select::interpreter_constant_read",
    );
}
#[test]
fn cranelift_constant_read() {
    run(
        Backend::Cranelift,
        &[8, 64, 65, 128, 129, 512],
        &[true, false],
        false,
        "constant_select::cranelift_constant_read",
    );
}
#[test]
fn interpreter_constant_write() {
    run(
        Backend::Interpret,
        &[8, 64, 65, 128, 129, 512],
        &[true, false],
        true,
        "constant_select::interpreter_constant_write",
    );
}
#[test]
fn cranelift_constant_write() {
    run(
        Backend::Cranelift,
        &[8, 64, 65, 128],
        &[true, false],
        true,
        "constant_select::cranelift_constant_write",
    );
}
#[test]
fn cranelift_wide_constant_write() {
    run(
        Backend::Cranelift,
        &[129, 512],
        &[false],
        true,
        "constant_select::cranelift_wide_constant_write",
    );
}
#[test]
fn fallback_wide_constant_write() {
    run(
        Backend::CraneliftFallback,
        &[129, 512],
        &[true],
        true,
        "constant_select::fallback_wide_constant_write",
    );
}

fn diagnostics() {
    use veryl_analyzer::{
        Analyzer, AnalyzerError, Context, analyzer_error::InvalidSelectKind, ir as air,
    };
    use veryl_metadata::Metadata;
    use veryl_parser::Parser;
    let mut checks = 0;
    for (select, expected) in [
        ("[0]", "valid"),
        ("[7]", "valid"),
        ("[8]", "range"),
        ("[9]", "range"),
        ("[-1]", "negative"),
        ("[0+:8]", "valid"),
        ("[1+:8]", "range"),
        ("[7-:8]", "valid"),
        ("[8-:8]", "range"),
        ("[0 step 8]", "valid"),
        ("[1 step 8]", "range"),
        ("[7:0]", "valid"),
        ("[8:1]", "range"),
        ("[0:7]", "order"),
        ("[0+:0]", "zero"),
    ] {
        for write in [false, true] {
            let statements = if write {
                format!(
                    "var q: logic<8>; always_comb {{q = data; q{select} = patch;}} assign out = q;"
                )
            } else {
                format!("assign out = data{select};")
            };
            let source = format!(
                "module Top(data: input logic<8>, patch: input logic<8>, out: output logic<8>) {{{statements}}}"
            );
            let analyzer = Analyzer::new(&Metadata::create_default("prj").unwrap());
            analyzer.clear();
            let parser = Parser::parse(&source, &"").unwrap();
            let mut context = Context::default();
            let mut ir = air::Ir::default();
            let mut errors = analyzer.analyze_pass1("prj", &parser.veryl);
            errors.extend(Analyzer::analyze_post_pass1());
            errors.extend(analyzer.analyze_pass2(&parser.veryl, &mut context, Some(&mut ir)));
            errors.extend(Analyzer::analyze_post_pass2(&ir));
            let kinds: Vec<_> = errors
                .iter()
                .map(|e| match e {
                    AnalyzerError::InvalidSelect {
                        kind: InvalidSelectKind::OutOfRange { .. },
                        ..
                    } => "range",
                    AnalyzerError::InvalidSelect {
                        kind: InvalidSelectKind::WrongOrder { .. },
                        ..
                    } => "order",
                    AnalyzerError::ZeroSize { .. } => "zero",
                    AnalyzerError::ExceedLimit { .. } => "limit",
                    _ => "other",
                })
                .collect();
            let want = if expected == "valid" {
                vec![]
            } else if expected == "negative" {
                vec!["range", "limit"]
            } else {
                vec![expected]
            };
            assert_eq!(
                kinds, want,
                "simulator constant-select diagnostic mismatch: {select} write={write}: {errors:?}"
            );
            checks += 1;
        }
    }
    eprintln!("constant-select complete diagnostics={checks}");
}
#[test]
fn constant_select_diagnostics() {
    let name = "constant_select::constant_select_diagnostics";
    if std::env::var_os("VERYL_FORMAL_CONSTANT_SELECT_CHILD").is_some() {
        diagnostics();
        return;
    }
    let out = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", name, "--nocapture"])
        .env("VERYL_FORMAL_CONSTANT_SELECT_CHILD", "1")
        .output()
        .unwrap();
    let log = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(out.status.success(), "constant-select child failed:\n{log}");
    assert!(
        log.contains("1 passed; 0 failed")
            && log.contains("constant-select complete diagnostics=30"),
        "constant-select diagnostics incomplete"
    );
    eprintln!("{log}");
}
