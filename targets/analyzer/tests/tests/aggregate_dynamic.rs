//! Finite recursive member reads through source analysis and real IR evaluation.
//! Depths 0..=5, four payload patterns, and signed/unsigned four-bit selectors;
//! unknown or locally invalid indices must not alias adjacent fields. Selected
//! effectful calls check evaluation count/order; arbitrary effectful bounds and
//! arbitrary recursive depth remain outside this comparison.

use veryl_analyzer::{
    Analyzer, Context,
    ir::{self, Component, Declaration, Statement},
    value::Value,
};
use veryl_metadata::Metadata;
use veryl_parser::Parser;
fn fixture(
    depth: usize,
    signed: bool,
    pattern: usize,
) -> (
    ir::Module,
    ir::Expression,
    Vec<ir::VarId>,
    Vec<bool>,
    Vec<usize>,
) {
    let mut widths = vec![5usize];
    let mut definitions = "struct Leaf {hi:logic<2>,lo:logic<3>,}\n".to_owned();
    for level in 1..=depth {
        let prev = if level == 1 {
            "Leaf".to_owned()
        } else {
            format!("Node{}", level - 1)
        };
        definitions +=
            &format!("struct Node{level}{{prefix:logic,body:{prev}<2>,suffix:logic<2>,}}\n");
        widths.push(widths[level - 1] * 2 + 3);
    }
    let width = widths[depth] * 2;
    let ty = if depth == 0 {
        "Leaf".to_owned()
    } else {
        format!("Node{depth}")
    };
    let sign = if signed { "signed " } else { "" };
    let ports: String = (0..=depth)
        .map(|i| format!("i{i}:input {sign}logic<4>,"))
        .collect();
    let args: String = (0..=depth).map(|i| format!("i{i},")).collect();
    let mut access = "storage[i0]".to_owned();
    for level in 1..=depth {
        access += &format!(".body[i{level}]");
    }
    access += ".hi";
    let bits: Vec<_> = (0..width)
        .map(|i| match pattern {
            0 => false,
            1 => true,
            2 => i % 2 == 0,
            _ => (i * 37 + i / 7) % 11 < 5,
        })
        .collect();
    let mut hex = String::new();
    for chunk in bits.chunks(4).rev() {
        let mut n = 0;
        for (bit, &v) in chunk.iter().enumerate() {
            n |= usize::from(v) << bit;
        }
        hex.push(char::from_digit(n as u32, 16).unwrap());
    }
    let source = format!(
        "module Top({ports}o:output logic<2>){{{definitions}function probe({ports})->logic<2>{{var storage:{ty}<2>;storage={width}'h{hex};return {access};}}assign o=probe({args});}}"
    );
    let (module, expression) = analyze(&source);
    let ids = (0..=depth)
        .map(|i| {
            *module
                .ports
                .iter()
                .find(|(p, _)| p.to_string() == format!("i{i}"))
                .unwrap()
                .1
        })
        .collect();
    (module, expression, ids, bits, widths)
}
#[test]
fn recursive_member_domains_preserve_dynamic_signed_and_unknown_indices() {
    let mut observations = 0;
    for depth in 0..=5 {
        for signed in [false, true] {
            for pattern in 0..4 {
                let (module, expression, ids, bits, widths) = fixture(depth, signed, pattern);
                let mut ctx = Context::default();
                ctx.variables = module.variables;
                ctx.functions = module.functions;
                let mut cases = Vec::new();
                for combination in 0..(1usize << (depth + 1)) {
                    cases.push(
                        (0..=depth)
                            .map(|i| (((combination >> i) & 1) as u64, 0u64))
                            .collect::<Vec<_>>(),
                    );
                }
                for axis in 0..=depth {
                    for payload in 0..16 {
                        for mask in 0..16 {
                            let mut coords = vec![(0, 0); depth + 1];
                            coords[axis] = (payload, mask);
                            cases.push(coords);
                        }
                    }
                }
                for coords in cases {
                    for (&id, &(p, m)) in ids.iter().zip(&coords) {
                        ctx.variables.get_mut(&id).unwrap().set_value(
                            &[],
                            Value::from_u128(p as u128, m as u128, 4, signed),
                            None,
                        );
                    }
                    let expected = if coords.iter().any(|&(p, m)| p >= 2 || m != 0) {
                        (0, 3)
                    } else {
                        let mut low = coords[0].0 as usize * widths[depth];
                        for level in (1..=depth).rev() {
                            let axis = depth - level + 1;
                            low += 2 + coords[axis].0 as usize * widths[level - 1];
                        }
                        low += 3;
                        let p = u128::from(bits[low]) | (u128::from(bits[low + 1]) << 1);
                        (p, 0)
                    };
                    let got = expression
                        .eval_value(&mut ctx)
                        .expect("dynamic aggregate evaluates");
                    assert_eq!(
                        (got.payload_u128(), got.mask_xz_u128()),
                        expected,
                        "dynamic aggregate output mismatch: depth={depth} signed={signed} pattern={pattern} coords={coords:?}"
                    );
                    assert!(
                        ctx.drain_errors().is_empty(),
                        "dynamic aggregate runtime diagnostics"
                    );
                    observations += 1;
                }
            }
        }
    }
    eprintln!("dynamic aggregate complete observations={observations}");
}

fn analyze(source: &str) -> (ir::Module, ir::Expression) {
    let metadata = Metadata::create_default("prj").unwrap();
    let analyzer = Analyzer::new(&metadata);
    analyzer.clear();
    let parser = Parser::parse(source, &"").unwrap();
    let mut ctx = Context::default();
    let mut air = ir::Ir::default();
    let mut errors = analyzer.analyze_pass1("prj", &parser.veryl);
    errors.extend(Analyzer::analyze_post_pass1());
    errors.extend(analyzer.analyze_pass2(&parser.veryl, &mut ctx, Some(&mut air)));
    errors.extend(Analyzer::analyze_post_pass2(&air));
    assert!(
        errors.is_empty(),
        "dynamic aggregate source diagnostics: {errors:?}"
    );
    let Component::Module(module) = &air.components[0] else {
        panic!("missing module")
    };
    let expression = module
        .declarations
        .iter()
        .find_map(|d| {
            let Declaration::Comb(c) = d else { return None };
            c.statements.iter().find_map(|s| {
                let Statement::Assign(a) = s else { return None };
                Some(a.expr.clone())
            })
        })
        .unwrap();
    (module.clone(), expression)
}

#[test]
fn member_selector_calls_are_evaluated_once() {
    let (module, expr) = analyze(
        "module Top(i0:input logic<4>,o:output logic<6>){struct Leaf{hi:logic<2>,lo:logic<3>,}var counter:logic<4>;function next()->logic<4>{counter+=1;return counter;}function probe(seed:input logic<4>)->logic<6>{counter=seed;var storage:Leaf<2>;storage=10'h2ab;var picked:logic<2>;picked=storage[next()].hi;return {counter,picked};}assign o=probe(i0);}",
    );
    let id = *module
        .ports
        .iter()
        .find(|(p, _)| p.to_string() == "i0")
        .unwrap()
        .1;
    let mut ctx = Context::default();
    ctx.variables = module.variables;
    ctx.functions = module.functions;
    for (seed, payload, mask) in [(0, 6, 0), (15, 1, 0), (1, 8, 3)] {
        ctx.variables
            .get_mut(&id)
            .unwrap()
            .set_value(&[], Value::new(seed, 4, false), None);
        let got = expr.eval_value(&mut ctx).unwrap();
        assert_eq!(
            (got.payload_u128(), got.mask_xz_u128()),
            (payload, mask),
            "dynamic aggregate call output mismatch: seed={seed}"
        );
    }
}

/// Side-effecting packed-read base is evaluated exactly once, including
/// partially overlapping and entirely missing windows. Width is constant;
/// effectful widths, multidimensional member flattening, and LHS order differ.
#[test]
fn packed_selector_calls_preserve_count_and_missing_bits() {
    let (module, expr) = analyze(
        "module Top(i0:input logic<4>,o:output logic<6>){var counter:logic<4>;function next()->logic<4>{counter+=1;return counter;}function probe(seed:input logic<4>)->logic<6>{counter=seed;var storage:logic<8>;storage=8'hb6;var picked:logic<2>;picked=storage[next() +: 2];return {counter,picked};}assign o=probe(i0);}",
    );
    let id = *module
        .ports
        .iter()
        .find(|(p, _)| p.to_string() == "i0")
        .unwrap()
        .1;
    let mut ctx = Context::default();
    ctx.variables = module.variables;
    ctx.functions = module.functions;
    for seed in 0..16u64 {
        ctx.variables
            .get_mut(&id)
            .unwrap()
            .set_value(&[], Value::new(seed, 4, false), None);
        let got = expr.eval_value(&mut ctx).unwrap();
        let next = (seed + 1) & 15;
        let mut payload = u128::from(next) << 2;
        let mut mask = 0;
        for bit in 0..2 {
            let pos = next + bit;
            if pos < 8 {
                payload |= u128::from((0xb6u64 >> pos) & 1) << bit;
            } else {
                mask |= 1 << bit;
            }
        }
        assert_eq!(
            (got.payload_u128(), got.mask_xz_u128()),
            (payload, mask),
            "packed selector call output mismatch: seed={seed}"
        );
    }
}

/// Function output copyback into FF storage is rejected by the language;
/// successful simulation of a patched/accepted variant would not verify it.
#[test]
fn function_output_arguments_in_ff_are_rejected() {
    let source = include_str!("../../../simulator/fixtures/call_ff_output_rejected.veryl");
    let metadata = Metadata::create_default("prj").unwrap();
    let analyzer = Analyzer::new(&metadata);
    analyzer.clear();
    let parser = Parser::parse(source, &"").unwrap();
    let mut ctx = Context::default();
    let mut ir = ir::Ir::default();
    let mut errors = analyzer.analyze_pass1("prj", &parser.veryl);
    errors.extend(Analyzer::analyze_post_pass1());
    errors.extend(analyzer.analyze_pass2(&parser.veryl, &mut ctx, Some(&mut ir)));
    errors.extend(Analyzer::analyze_post_pass2(&ir));
    assert!(
        errors.iter().any(|e| matches!(
            e,
            veryl_analyzer::AnalyzerError::FunctionOutputInAlwaysFf { .. }
        )),
        "expected function output-in-FF diagnostic: {errors:?}"
    );
}

/// A member array is surrounded by live fields. Invalid side-effecting indices
/// must not turn those fields into selected data, and must execute only once.
/// All unsigned four-bit seeds are checked for one dynamic coordinate after a
/// static member prefix; arbitrary effectful multidimensional paths differ.
#[test]
fn effectful_member_indices_stay_inside_the_static_prefix() {
    let (module, expr) = analyze(
        "module Top(i0:input logic<4>,o:output logic<6>){struct Leaf{hi:logic<2>,lo:logic<3>,}struct Packet{head:logic<4>,body:Leaf<2>,tail:logic<4>,}var counter:logic<4>;function next()->logic<4>{counter+=1;return counter;}function probe(seed:input logic<4>)->logic<6>{counter=seed;var storage:Packet;storage=18'h3ffff;var picked:logic<2>;picked=storage.body[next()].hi;return {counter,picked};}assign o=probe(i0);}",
    );
    let id = *module
        .ports
        .iter()
        .find(|(p, _)| p.to_string() == "i0")
        .unwrap()
        .1;
    let mut ctx = Context::default();
    ctx.variables = module.variables;
    ctx.functions = module.functions;
    for seed in 0..16u64 {
        ctx.variables
            .get_mut(&id)
            .unwrap()
            .set_value(&[], Value::new(seed, 4, false), None);
        let got = expr.eval_value(&mut ctx).unwrap();
        let next = (seed + 1) & 15;
        let expected = if next < 2 {
            ((next << 2) | 3, 0u64)
        } else {
            (next << 2, 3u64)
        };
        assert_eq!(
            (got.payload_u128(), got.mask_xz_u128()),
            (u128::from(expected.0), u128::from(expected.1)),
            "effectful member output mismatch: seed={seed}"
        );
    }
}
