//! Finite two-state expression comparisons across analyzer evaluation, three
//! simulator engines, mapped gates, and emitted SV. Operand widths independently
//! range over 1..=4, both signedness flags and every input value are enumerated.
//! Assignment widths are 1, max(input widths), and max+1 (deduplicated).
//! The fixed grammar exercises context propagation and numeric/i32/u32 casts;
//! IEEE 1800-2023 6.24.1, 11.6.1 and 11.8.2 define the independent Expr model.
//! No arbitrary expression, four-state, wide-vector, or sequential proof follows.

use super::{common, sequential};
use std::collections::HashMap;
use veryl_analyzer::{Context, ir as air, value::Value as AnalyzerValue};
use veryl_formal_semantics::{comparison::Operand, expressions::Expr};
use veryl_parser::resource_table;
use veryl_simulator::{
    Simulator,
    ir::{Config, Value, build_ir},
};
use veryl_synthesizer::build_gate_ir;

fn expressions(narrow: usize) -> Vec<Expr> {
    use Expr::*;
    let a = || Box::new(Input(0));
    let b = || Box::new(Input(1));
    let c = || Box::new(Input(2));
    let sum = || Box::new(Add(a(), b()));
    vec![
        Add(a(), b()),
        Sub(a(), b()),
        Less(a(), b()),
        Shift(a(), b()),
        Choose(c(), a(), b()),
        Cast(sum(), narrow),
        Add(Box::new(Cast(a(), narrow)), b()),
        Add(Box::new(Cast(sum(), narrow)), b()),
        Less(Box::new(Cast(sum(), narrow)), b()),
        Choose(c(), Box::new(Cast(sum(), narrow)), b()),
        Shift(a(), Box::new(Reduce(sum()))),
        Add(Box::new(TypedCast(a(), true)), b()),
        Add(Box::new(TypedCast(a(), false)), b()),
        Less(Box::new(TypedCast(a(), true)), b()),
    ]
}

fn render(expr: &Expr) -> String {
    match expr {
        Expr::Input(i) => ["a", "b", "c"][*i].into(),
        Expr::Cast(x, width) => format!("({} as {width})", render(x)),
        Expr::TypedCast(x, signed) => {
            format!("({} as {})", render(x), if *signed { "i32" } else { "u32" })
        }
        Expr::Reduce(x) => format!("(|({}))", render(x)),
        Expr::Choose(c, a, b) => {
            format!("(if {} ? {} : {})", render(c), render(a), render(b))
        }
        _ => {
            let (a, op, b) = match expr {
                Expr::Add(a, b) => (a, "+", b),
                Expr::Sub(a, b) => (a, "-", b),
                Expr::Xor(a, b) => (a, "^", b),
                Expr::Less(a, b) => (a, "<:", b),
                Expr::Shift(a, b) => (a, "<<", b),
                _ => unreachable!(),
            };
            format!("({} {op} {})", render(a), render(b))
        }
    }
}

fn compare(aw: usize, bw: usize, sa: bool, sb: bool) -> usize {
    let sign = |s| if s { "signed " } else { "" };
    let mut widths = vec![1, aw.max(bw), aw.max(bw) + 1];
    widths.dedup();
    let outputs: Vec<_> = widths
        .into_iter()
        .flat_map(|width| {
            expressions(aw.min(bw))
                .into_iter()
                .map(move |expr| (width, expr))
        })
        .collect();
    let ports: String = outputs
        .iter()
        .enumerate()
        .map(|(i, (w, _))| format!(",o{i}:output logic<{w}>"))
        .collect();
    let statements: String = outputs
        .iter()
        .enumerate()
        .map(|(i, (_, e))| format!("assign o{i}={};", render(e)))
        .collect();
    let source = format!(
        "module Top(a:input {}logic<{aw}>,b:input {}logic<{bw}>,c:input logic{ports}){{{statements}}}",
        sign(sa),
        sign(sb)
    );
    let ir = sequential::analyze_source(&source);
    let air::Component::Module(module) = &ir.components[0] else {
        panic!("missing module")
    };
    let mut context = Context::default();
    context.variables = module.variables.clone();
    context.functions = module.functions.clone();
    let ids: Vec<_> = ["a", "b", "c"]
        .into_iter()
        .map(|name| {
            *module
                .ports
                .iter()
                .find(|(p, _)| p.to_string() == name)
                .unwrap()
                .1
        })
        .collect();
    let analyzed: Vec<_> = module
        .declarations
        .iter()
        .filter_map(|d| {
            let air::Declaration::Comb(c) = d else {
                return None;
            };
            let air::Statement::Assign(a) = &c.statements[0] else {
                return None;
            };
            Some(a.clone())
        })
        .collect();
    assert_eq!(analyzed.len(), outputs.len());
    let gate = build_gate_ir(&ir, resource_table::insert_str("Top"))
        .unwrap()
        .module;
    assert!(gate.ffs.is_empty() && gate.ram_blocks.is_empty());
    let nets: HashMap<_, _> = gate
        .ports
        .iter()
        .map(|p| {
            (
                resource_table::get_str_value(p.name).unwrap(),
                p.nets.clone(),
            )
        })
        .collect();
    let mut sims = Vec::new();
    for (jit, cc) in [(false, false), (true, false), (true, true)] {
        let config = Config {
            use_jit: jit,
            aot_c: cc,
            aot_c_async: false,
            aot_c_min_stmts: 0,
            aot_c_validate: false,
            ..Default::default()
        };
        let sim_ir = build_ir(&ir, "Top".into(), &config).unwrap();
        let (compiled, interpreted, _, _) = sim_ir.detailed_stats();
        if cc {
            sequential::routes::assert_cc_comb_ready(&sim_ir);
        } else if jit {
            assert!(
                compiled > 0 && interpreted == 0,
                "mixed expression JIT fallback"
            );
        } else {
            assert!(
                compiled == 0 && interpreted > 0,
                "mixed expression interpreter route absent"
            );
        }
        sims.push(Simulator::new(sim_ir, None));
    }
    let wires: String = outputs
        .iter()
        .enumerate()
        .map(|(i, (w, _))| format!("wire [{}:0] o{i};", w - 1))
        .collect();
    let mut bench = format!(
        "module testbench;reg [{}:0] a;reg [{}:0] b;reg c;{wires}prj_Top dut(.*);initial begin\n",
        aw - 1,
        bw - 1
    );
    let format = vec!["%b"; outputs.len()].join(" ");
    let arguments = (0..outputs.len())
        .map(|i| format!("o{i}"))
        .collect::<Vec<_>>()
        .join(",");
    let mut expected_sv = Vec::new();
    let mut comparisons = 0;
    for a in 0..1u64 << aw {
        for b in 0..1u64 << bw {
            for c in 0..2 {
                let inputs = [
                    Operand {
                        planes: (a as u128, 0),
                        width: aw,
                        signed: sa,
                    },
                    Operand {
                        planes: (b as u128, 0),
                        width: bw,
                        signed: sb,
                    },
                    Operand {
                        planes: (c as u128, 0),
                        width: 1,
                        signed: false,
                    },
                ];
                let mut gate_inputs = HashMap::new();
                for ((name, id), input) in ["a", "b", "c"].into_iter().zip(&ids).zip(inputs) {
                    context.variables.get_mut(id).unwrap().set_value(
                        &[],
                        AnalyzerValue::from_u128(input.planes.0, 0, input.width, input.signed),
                        None,
                    );
                    for sim in &mut sims {
                        sim.set(
                            name,
                            Value::new(input.planes.0 as u64, input.width, input.signed),
                        );
                    }
                    for (bit, net) in nets[name].iter().enumerate() {
                        gate_inputs.insert(*net, input.planes.0 & (1 << bit) != 0);
                    }
                }
                let mut cache = vec![None; gate.nets.len()];
                let mut row = Vec::new();
                for (i, ((width, tree), assignment)) in outputs.iter().zip(&analyzed).enumerate() {
                    let mask = (1u128 << width) - 1;
                    let want = tree.evaluate(&inputs, *width).planes;
                    assert_eq!(want.1, 0);
                    let want = want.0 & mask;
                    let label = format!(
                        "aw={aw} bw={bw} sa={sa} sb={sb} a={a} b={b} c={c} o{i}={tree:?} width={width}"
                    );
                    assignment.eval_value(&mut context);
                    let value = context.variables[&assignment.dst[0].id]
                        .get_value(&[])
                        .unwrap();
                    assert_eq!(
                        (value.payload_u128(), value.mask_xz_u128()),
                        (want, 0),
                        "mixed expression output mismatch: analyzer {label}"
                    );
                    for (engine, sim) in sims.iter_mut().enumerate() {
                        let got = sim.get(&format!("o{i}")).unwrap();
                        assert_eq!(
                            (got.payload_u128(), got.mask_xz_u128()),
                            (want, 0),
                            "mixed expression output mismatch: engine={engine} {label}"
                        );
                    }
                    let got =
                        nets[&format!("o{i}")]
                            .iter()
                            .enumerate()
                            .fold(0u128, |v, (bit, &net)| {
                                v | (u128::from(sequential::cells::eval(
                                    &gate,
                                    net,
                                    &gate_inputs,
                                    &mut cache,
                                    0,
                                )) << bit)
                            });
                    assert_eq!(got, want, "mixed expression output mismatch: gates {label}");
                    row.push(format!("{want:0width$b}"));
                    comparisons += 1;
                }
                assert!(context.drain_errors().is_empty());
                bench += &format!(
                    "a={aw}'d{a};b={bw}'d{b};c=1'b{c};#1;$display(\"case {format}\",{arguments});\n"
                );
                expected_sv.push(row.join(" "));
            }
        }
    }
    bench += "$display(\"complete\");$finish;end endmodule\n";
    let got = common::execute(&source, &bench);
    assert_eq!(
        got.len(),
        expected_sv.len(),
        "mixed expression SV vector count changed"
    );
    for (index, (actual, expected)) in got.iter().zip(expected_sv).enumerate() {
        assert_eq!(
            *actual, expected,
            "mixed expression output mismatch: SV aw={aw} bw={bw} sa={sa} sb={sb} vector={index}"
        );
    }
    drop(sims);
    comparisons
}

#[test]
fn mixed_width_signed_casts_match_all_routes() {
    // Isolate dispatch counters from the other cross-target fixtures.
    if std::env::var_os("VERYL_FORMAL_MIXED_CHILD").is_none() {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "mixed_expressions::mixed_width_signed_casts_match_all_routes",
                "--nocapture",
            ])
            .env("VERYL_FORMAL_MIXED_CHILD", "1")
            .output()
            .unwrap();
        let log = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            output.status.success(),
            "mixed expression child failed:\n{log}"
        );
        assert!(
            log.contains("mixed expression complete configurations=64"),
            "mixed expression child incomplete:\n{log}"
        );
        eprintln!("{log}");
        return;
    }
    let mut comparisons = 0;
    for aw in 1..=4 {
        for bw in 1..=4 {
            for sa in [false, true] {
                for sb in [false, true] {
                    comparisons += compare(aw, bw, sa, sb);
                }
            }
        }
    }
    assert_eq!(comparisons, 301_952, "mixed expression domain changed");
    let dispatch = veryl_simulator::residency::dispatch_counts();
    let comb: Vec<_> = dispatch
        .iter()
        .filter(|(name, _, _)| name.starts_with("dataflow:"))
        .collect();
    assert!(
        !comb.is_empty()
            && comb
                .iter()
                .all(|(_, ran, fallback)| *ran > 0 && *fallback == 0),
        "mixed expression AOT-C dispatch absent or fell back: {comb:?}"
    );
    eprintln!(
        "mixed expression complete configurations=64 comparisons_per_route={comparisons} routes=6"
    );
}
