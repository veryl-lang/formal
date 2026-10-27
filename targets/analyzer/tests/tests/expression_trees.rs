//! Generated finite source expressions with independent recursive sizing/evaluation.
//! Small two-state inputs are exhaustive; four-state and 63/64/65-bit boundaries
//! use explicit patterns. Grammar depth is bounded by the constructors below.
//! No arbitrary grammar, effectful expression, float, or general AST theorem.

use veryl_analyzer::{
    Analyzer, Context,
    ir::{self, Component, Declaration, Statement},
    value::Value,
};
use veryl_formal_semantics::{comparison::Operand, expressions::Expr};
use veryl_metadata::Metadata;
use veryl_parser::Parser;

fn render(expr: &Expr) -> String {
    match expr {
        Expr::Input(i) => ["a", "b", "c"][*i].into(),
        Expr::TypedCast(a, signed) => {
            format!("({} as {})", render(a), if *signed { "i32" } else { "u32" })
        }
        Expr::Cast(a, width) => format!("({} as {width})", render(a)),
        Expr::Reduce(a) => format!("(|({}))", render(a)),
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

fn trees(width: usize) -> Vec<Expr> {
    use Expr::*;
    let a = || Box::new(Input(0));
    let b = || Box::new(Input(1));
    let c = || Box::new(Input(2));
    let mut result = vec![TypedCast(a(), false), TypedCast(a(), true)];
    for inner in [
        Add(a(), b()),
        Sub(a(), b()),
        Xor(a(), b()),
        Choose(c(), a(), b()),
    ] {
        for boundary in [
            inner.clone(),
            Cast(Box::new(inner.clone()), 1),
            Cast(Box::new(inner.clone()), width),
            TypedCast(Box::new(inner.clone()), false),
            TypedCast(Box::new(inner.clone()), true),
        ] {
            result.push(Add(Box::new(boundary.clone()), a()));
            result.push(Choose(c(), Box::new(boundary.clone()), b()));
            result.push(Less(Box::new(boundary.clone()), b()));
            result.push(Shift(a(), Box::new(Reduce(Box::new(boundary)))));
        }
    }
    result
}
fn patterns(width: usize, exhaustive: bool) -> Vec<(u128, u128)> {
    let mask = (1u128 << width) - 1;
    if exhaustive {
        return (0..=mask).map(|p| (p, 0)).collect();
    }
    vec![
        (0, 0),
        (mask, 0),
        (1u128 << (width - 1), 0),
        (0, mask),
        (mask, mask),
        (0, 1u128 << (width - 1)),
    ]
}
fn compare(exhaustive: bool) {
    let mut count = 0;
    let widths: &[(usize, usize)] = if exhaustive {
        &[(1, 2), (2, 3), (3, 1)]
    } else {
        &[(1, 2), (2, 3), (63, 65), (64, 65), (65, 63)]
    };
    for &(aw, bw) in widths {
        for sa in [false, true] {
            for sb in [false, true] {
                for extra in [0, 1] {
                    let width = aw.max(bw) + extra;
                    let trees = trees(aw);
                    let sign = |s| if s { "signed " } else { "" };
                    let outputs: String = (0..trees.len())
                        .map(|i| format!(",o{i}:output logic<{width}>"))
                        .collect();
                    let body: String = trees
                        .iter()
                        .enumerate()
                        .map(|(i, t)| format!("assign o{i}={};", render(t)))
                        .collect();
                    let source = format!(
                        "module Top(a:input {}logic<{aw}>,b:input {}logic<{bw}>,c:input logic{outputs}){{{body}}}",
                        sign(sa),
                        sign(sb)
                    );
                    let metadata = Metadata::create_default("prj").unwrap();
                    let analyzer = Analyzer::new(&metadata);
                    analyzer.clear();
                    let parser = Parser::parse(&source, &"").unwrap();
                    let mut ctx = Context::default();
                    let mut ir = ir::Ir::default();
                    let mut errors = analyzer.analyze_pass1("prj", &parser.veryl);
                    errors.extend(Analyzer::analyze_post_pass1());
                    errors.extend(analyzer.analyze_pass2(&parser.veryl, &mut ctx, Some(&mut ir)));
                    errors.extend(Analyzer::analyze_post_pass2(&ir));
                    assert!(
                        errors.is_empty(),
                        "expression fixture diagnostics: {errors:?}\n{source}"
                    );
                    let Component::Module(module) = &ir.components[0] else {
                        panic!("missing module")
                    };
                    let ids: Vec<_> = ["a", "b", "c"]
                        .iter()
                        .map(|name| {
                            *module
                                .ports
                                .iter()
                                .find(|(p, _)| p.to_string() == *name)
                                .unwrap()
                                .1
                        })
                        .collect();
                    let expressions: Vec<_> = module
                        .declarations
                        .iter()
                        .filter_map(|d| {
                            let Declaration::Comb(c) = d else { return None };
                            let Statement::Assign(a) = &c.statements[0] else {
                                return None;
                            };
                            Some(a.expr.clone())
                        })
                        .collect();
                    assert_eq!(expressions.len(), trees.len());
                    ctx.variables = module.variables.clone();
                    ctx.functions = module.functions.clone();
                    for av in patterns(aw, exhaustive) {
                        for bv in patterns(bw, exhaustive) {
                            for cv in if exhaustive {
                                vec![(0, 0), (1, 0)]
                            } else {
                                vec![(0, 0), (1, 0), (0, 1), (1, 1)]
                            } {
                                let inputs = [
                                    Operand {
                                        planes: av,
                                        width: aw,
                                        signed: sa,
                                    },
                                    Operand {
                                        planes: bv,
                                        width: bw,
                                        signed: sb,
                                    },
                                    Operand {
                                        planes: cv,
                                        width: 1,
                                        signed: false,
                                    },
                                ];
                                for (id, input) in ids.iter().zip(inputs) {
                                    ctx.variables.get_mut(id).unwrap().set_value(
                                        &[],
                                        Value::from_u128(
                                            input.planes.0,
                                            input.planes.1,
                                            input.width,
                                            input.signed,
                                        ),
                                        None,
                                    );
                                }
                                for (tree, expr) in trees.iter().zip(&expressions) {
                                    let actual = expr.eval_value(&mut ctx).unwrap();
                                    // A root cast returns its own width; assignment sizing is external.
                                    let result_width = if matches!(tree, Expr::TypedCast(..)) {
                                        32
                                    } else {
                                        width
                                    };
                                    let expected = tree.evaluate(&inputs, result_width);
                                    assert_eq!(
                                        (
                                            actual.payload_u128(),
                                            actual.mask_xz_u128(),
                                            actual.width()
                                        ),
                                        (expected.planes.0, expected.planes.1, expected.width),
                                        "expression tree output mismatch: expr={} inputs={inputs:?} width={width}",
                                        render(tree)
                                    );
                                    assert!(ctx.drain_errors().is_empty());
                                    count += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    eprintln!("expression tree comparisons={count} exhaustive={exhaustive}");
}
#[test]
fn source_expression_trees_small_two_state() {
    compare(true);
}
#[test]
fn source_expression_trees_four_state_and_word_boundaries() {
    compare(false);
}
