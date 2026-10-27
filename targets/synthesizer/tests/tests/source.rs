//! Real source-to-gate conversion for widths 1..=4, equal-width signed/unsigned
//! two-state operands, and every input pair, through the default synthesis route.
//! Expected values use independent widened arithmetic. Mixed widths, division,
//! X/Z, arbitrary widths, and sequential behavior are outside these fixtures.

mod common;
use std::collections::HashMap;
use veryl_analyzer::{Analyzer, Context, ir as air};
use veryl_formal_semantics::scalar::{self, Operation};
use veryl_metadata::Metadata;
use veryl_parser::{Parser, resource_table};
use veryl_synthesizer::{GateModule, build_gate_ir};

const OPS: &[(&str, &str)] = &[
    ("sum", "a + b"),
    ("difference", "a - b"),
    ("product", "a * b"),
    ("bit_and", "a & b"),
    ("bit_or", "a | b"),
    ("bit_xor", "a ^ b"),
    ("left_shift", "a << b"),
    ("right_shift", "a >> b"),
    ("arithmetic_right", "a >>> b"),
    ("less", "a <: b"),
    ("less_equal", "a <= b"),
    ("greater", "a >: b"),
    ("greater_equal", "a >= b"),
    ("equal", "a == b"),
    ("unequal", "a != b"),
    ("choose", "if b[0] ? a : b"),
    ("joined", "{a,b}"),
    ("extended", "a"),
    ("wide_sum", "a + b"),
    ("wide_bit_and", "a & b"),
    ("wide_bit_or", "a | b"),
    ("wide_bit_xor", "a ^ b"),
    ("wide_left_shift", "a << b"),
    ("wide_right_shift", "a >> b"),
    ("wide_arithmetic_right", "a >>> b"),
    ("wide_choose", "if b[0] ? a : b"),
    ("top_bit", "a[WIDTH - 1]"),
];
fn operations(signed: bool) -> impl Iterator<Item = &'static (&'static str, &'static str)> {
    OPS.iter().filter(move |(name, _)| {
        signed || !matches!(*name, "arithmetic_right" | "wide_arithmetic_right")
    })
}
fn output_width(name: &str, width: usize) -> usize {
    match name {
        "less" | "less_equal" | "greater" | "greater_equal" | "equal" | "unequal" | "top_bit" => 1,
        "joined"
        | "extended"
        | "wide_sum"
        | "wide_bit_and"
        | "wide_bit_or"
        | "wide_bit_xor"
        | "wide_left_shift"
        | "wide_right_shift"
        | "wide_arithmetic_right"
        | "wide_choose" => width * 2,
        _ => width,
    }
}
fn analyze(width: usize, signed: bool) -> GateModule {
    let sign = if signed { "signed " } else { "" };
    let outputs: String = operations(signed)
        .map(|(name, _)| format!("{name}:output logic<{}>,", output_width(name, width)))
        .collect();
    let statements: String = operations(signed)
        .map(|(name, expr)| {
            format!(
                "assign {name} = {};",
                expr.replace("WIDTH", &width.to_string())
            )
        })
        .collect();
    let source = format!(
        "module Top(a:input {sign}logic<{width}>,b:input {sign}logic<{width}>,{outputs}){{{statements}}}"
    );
    let metadata = Metadata::create_default("prj").unwrap();
    let analyzer = Analyzer::new(&metadata);
    analyzer.clear();
    let parser = Parser::parse(&source, &"").unwrap();
    let mut context = Context::default();
    let mut ir = air::Ir::default();
    let mut errors = analyzer.analyze_pass1("prj", &parser.veryl);
    errors.extend(Analyzer::analyze_post_pass1());
    errors.extend(analyzer.analyze_pass2(&parser.veryl, &mut context, Some(&mut ir)));
    errors.extend(Analyzer::analyze_post_pass2(&ir));
    assert!(
        errors.is_empty(),
        "synthesis fixture diagnostics: {errors:?}"
    );
    let gate = build_gate_ir(&ir, resource_table::insert_str("Top"))
        .unwrap()
        .module;
    assert!(gate.ffs.is_empty() && gate.ram_blocks.is_empty());
    gate
}
fn reference(name: &str, a: u64, b: u64, width: usize, signed: bool) -> u64 {
    let op = match name {
        "sum" | "wide_sum" => Operation::Add,
        "difference" => Operation::Subtract,
        "product" => Operation::Multiply,
        "bit_and" | "wide_bit_and" => Operation::And,
        "bit_or" | "wide_bit_or" => Operation::Or,
        "bit_xor" | "wide_bit_xor" => Operation::Xor,
        "left_shift" | "wide_left_shift" => Operation::ShiftLeft,
        "right_shift" | "wide_right_shift" => Operation::ShiftRight,
        "arithmetic_right" | "wide_arithmetic_right" => Operation::ArithmeticRight,
        "less" => Operation::Less,
        "less_equal" => Operation::LessEqual,
        "greater" => Operation::Greater,
        "greater_equal" => Operation::GreaterEqual,
        "equal" => Operation::Equal,
        "unequal" => Operation::Unequal,
        "choose" | "wide_choose" => Operation::ChooseLowBit,
        "joined" => Operation::Concatenate,
        "extended" => Operation::Extend,
        "top_bit" => Operation::MostSignificantBit,
        _ => unreachable!(),
    };
    scalar::evaluate(op, a, b, width, output_width(name, width), signed)
}

fn compare() {
    let mut configs = 0;
    let mut comparisons = 0;
    for width in 1..=4 {
        for signed in [false, true] {
            let gate = analyze(width, signed);
            let ports: HashMap<_, _> = gate
                .ports
                .iter()
                .map(|p| {
                    (
                        resource_table::get_str_value(p.name).unwrap(),
                        p.nets.clone(),
                    )
                })
                .collect();
            for a in 0..(1u64 << width) {
                for b in 0..(1u64 << width) {
                    let mut inputs = HashMap::new();
                    for (name, value) in [("a", a), ("b", b)] {
                        for (bit, &net) in ports[name].iter().enumerate() {
                            inputs.insert(net, value & (1 << bit) != 0);
                        }
                    }
                    let mut cache = vec![None; gate.nets.len()];
                    for &(name, _) in operations(signed) {
                        let mut got = 0u64;
                        for (bit, &net) in ports[name].iter().enumerate() {
                            if common::eval(&gate, net, &inputs, &mut cache, 0) {
                                got |= 1 << bit;
                            }
                        }
                        assert_eq!(
                            got,
                            reference(name, a, b, width, signed),
                            "synthesizer source output mismatch: name={name} width={width} signed={signed} a={a} b={b}"
                        );
                        comparisons += 1;
                    }
                }
            }
            configs += 1;
        }
    }
    eprintln!("synthesis-source complete configurations={configs} comparisons={comparisons}");
}
#[test]
fn source_to_technology_mapped_gates() {
    compare();
}
