//! Selected four-bit source functions and repeated calls compared with independent
//! state/trace models, with and without constant-function caching. Checks write
//! order and argument capture for these fixtures, not arbitrary IR execution.

use veryl_analyzer::value::Value;
use veryl_analyzer::{
    Analyzer, Context,
    ir::{self, Component, Declaration, Statement},
};
use veryl_metadata::Metadata;
use veryl_parser::Parser;

fn fixture(body: &str, actual: &str) -> (ir::Module, ir::Expression, veryl_analyzer::ir::VarId) {
    let source = format!(
        "module Top(i:input logic<4>,o:output logic<48>){{var counter:logic<4>;var trace:logic<32>;function stamp(delta:input logic<4>)->logic<4>{{counter+=delta;trace={{trace[27:0],counter}};return counter;}}function pack(x:input logic<4>,y:input logic<4>)->logic<8>{{return {{x,y}};}}{body}assign o=run({actual});}}"
    );
    let metadata = Metadata::create_default("prj").unwrap();
    let analyzer = Analyzer::new(&metadata);
    analyzer.clear();
    let parser = Parser::parse(&source, &"").unwrap();
    let mut ctx = Context::default();
    let mut air = ir::Ir::default();
    let mut errors = analyzer.analyze_pass1("prj", &parser.veryl);
    errors.extend(Analyzer::analyze_post_pass1());
    errors.extend(analyzer.analyze_pass2(&parser.veryl, &mut ctx, Some(&mut air)));
    errors.extend(Analyzer::analyze_post_pass2(&air));
    assert!(errors.is_empty(), "IR fixture diagnostics: {errors:?}");
    let Component::Module(module) = &air.components[0] else {
        panic!("missing module")
    };
    let input = module
        .ports
        .iter()
        .find(|(path, _)| path.to_string() == "i")
        .unwrap()
        .1;
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
    (module.clone(), expression, *input)
}
fn compare(body: &str, model: fn(u64) -> u64) {
    let (module, expression, input) = fixture(body, "i");
    for cached in [false, true] {
        let mut ctx = Context::default();
        ctx.variables = module.variables.clone();
        ctx.functions = module.functions.clone();
        if !cached {
            for function in ctx.functions.values_mut() {
                function.is_const = false;
            }
        }
        for seed in (0..16).chain([15, 0, 7, 1, 15, 0]) {
            ctx.variables
                .get_mut(&input)
                .unwrap()
                .set_value(&[], Value::new(seed, 4, false), None);
            let got = expression.eval_value(&mut ctx).unwrap();
            assert_eq!(
                got.to_u64(),
                Some(model(seed)),
                "IR execution trace mismatch: seed={seed} cached={cached}"
            );
            assert_eq!(
                got.mask_xz_u128(),
                0,
                "IR execution trace mismatch: unexpected unknown"
            );
            assert!(ctx.drain_errors().is_empty());
        }
    }
}
#[test]
fn loop_and_branch_preserve_each_write() {
    compare(
        r#"function run(seed:input logic<4>)->logic<48>{counter=seed;trace=0;var value:logic<4>;for k in 0..4 {if k%2==0 {value=stamp((k+1) as 4);}else{value=stamp(3);}if value[0] {value=stamp(2);}else{value=stamp(1);}}return {trace,12'b0,value};}"#,
        |seed| {
            let mut count = seed;
            let mut trace = 0;
            for k in 0..4 {
                let delta = if k % 2 == 0 { k + 1 } else { 3 };
                count = (count + delta) & 15;
                trace = ((trace << 4) | count) & 0xffffffff;
                count = (count + if count & 1 != 0 { 2 } else { 1 }) & 15;
                trace = ((trace << 4) | count) & 0xffffffff;
            }
            (trace << 16) | count
        },
    );
}
#[test]
fn nested_calls_capture_actuals_before_binding() {
    compare(
        r#"function run(seed:input logic<4>)->logic<48>{counter=seed;trace=0;var encoded:logic<8>;var last:logic<4>;encoded=pack(stamp(1),pack(stamp(2),stamp(3)) as 4);last=stamp(4);return {trace,4'b0,encoded,last};}"#,
        |seed| {
            let mut count = seed;
            let mut trace = 0;
            let mut states = [0; 4];
            for (i, slot) in states.iter_mut().enumerate() {
                count = (count + i as u64 + 1) & 15;
                trace = (trace << 4) | count;
                *slot = count;
            }
            (trace << 16) | (states[0] << 8) | (states[2] << 4) | states[3]
        },
    );
}

#[test]
fn constant_function_folding_preserves_control_flow() {
    let body = r#"function run(seed:input logic<4>)->logic<48>{if seed[0] {return seed+5;}var accum:logic<8>;accum=seed;for k in 0..4 {if k%2==0 {accum=accum*3+(k as 8);}else{accum=accum^((seed+k) as 8);}}return accum;}"#;
    for seed in 0..16u64 {
        let (_, expression, _) = fixture(body, &format!("4'd{seed}"));
        let mut want = seed;
        if seed & 1 != 0 {
            want += 5;
        } else {
            for k in 0..4 {
                want = if k % 2 == 0 {
                    (want * 3 + k) & 255
                } else {
                    (want ^ (seed + k)) & 255
                };
            }
        }
        let got = expression
            .eval_value(&mut Context::default())
            .unwrap()
            .to_u64();
        assert_eq!(got, Some(want), "IR folded control mismatch: seed={seed}");
    }
}
