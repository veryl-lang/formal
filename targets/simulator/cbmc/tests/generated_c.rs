//! Symbolic checks of actual emit_expr output for unsigned add/XOR/logical-right-shift
//! at fixed widths 8/32/64. Inputs occupy aligned valid offsets 0/8 in a 16-byte
//! buffer; operands have clean padding and shift counts are arbitrary u64 values.
//! The independent reference uses bit positions or a scalar full adder.
//!
//! The model is 64-bit little-endian GNU C under CBMC's byte-addressed alias model,
//! not a strict-aliasing or host-compiler optimization theorem. ABI declarations
//! are checked against the producer. Safety/unwinding checks and separate reachable
//! covers are required. Only the designated equality failure, without mixed safety
//! failures, qualifies as a semantic mutation rejection.
//!
//! A separate composed signed family checks comparison, addition, XOR and
//! ternary widening at fixed 8->16 and 32->64-bit widths. Other signed/nested
//! expressions, symbolic or unaligned allocations, wide vectors, FF logs,
//! source lowering, and arbitrary AOT functions are outside this domain.
//! A fixed narrowing-cast comb-store family
//! additionally admits arbitrary source/destination padding and checks the whole
//! allocation against per-bit replacement. Other stores remain outside scope.
//! Set CBMC to select a binary; the default also checks Kani's pinned tool bundle.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use veryl_simulator::backend::aot_c::emit_expr;
use veryl_simulator::ir::{ExpressionContext, Op, ProtoExpression, VarOffset};

const LOAD_TYPES: &str = "typedef __uint128_t veryl_u128_ua __attribute__((__aligned__(1)));\ntypedef uint64_t veryl_u64_ua __attribute__((__aligned__(1)));\ntypedef uint32_t veryl_u32_ua __attribute__((__aligned__(1)));\ntypedef uint16_t veryl_u16_ua __attribute__((__aligned__(1)));\n";

fn variable(offset: isize, width: usize) -> ProtoExpression {
    ProtoExpression::Variable {
        var_offset: VarOffset::Comb(offset),
        select: None,
        dynamic_select: None,
        width,
        var_full_width: width,
        expr_context: ExpressionContext {
            width,
            signed: false,
        },
    }
}
fn cbmc() -> PathBuf {
    if let Some(path) = std::env::var_os("CBMC") {
        return path.into();
    }
    let candidate =
        PathBuf::from(std::env::var_os("HOME").unwrap()).join(".kani/kani-0.68.0/bin/cbmc");
    if candidate.exists() {
        candidate
    } else {
        PathBuf::from("cbmc")
    }
}
#[derive(Debug, PartialEq, Eq)]
enum Verdict {
    Proved,
    SemanticMismatch,
    Rejected,
}

fn verification_result(exit_code: Option<i32>, log: &str) -> Verdict {
    let failures: Vec<_> = log
        .lines()
        .filter(|l| l.trim_end().ends_with(": FAILURE"))
        .collect();
    if exit_code == Some(10)
        && log.lines().any(|line| line == "VERIFICATION FAILED")
        && failures.len() == 1
        && failures[0].contains("production-C-reference")
    {
        Verdict::SemanticMismatch
    } else if exit_code == Some(0)
        && log.lines().any(|line| line == "VERIFICATION SUCCESSFUL")
        && failures.is_empty()
        && log
            .lines()
            .any(|l| l.contains("production-C-reference") && l.trim_end().ends_with(": SUCCESS"))
    {
        Verdict::Proved
    } else {
        Verdict::Rejected
    }
}

fn verify(source: &str) -> (Verdict, PathBuf, String) {
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/fixtures")
        .join(format!(
            "{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
    fs::create_dir_all(&directory).unwrap();
    let file = directory.join("harness.c");
    fs::write(&file, source).unwrap();
    let out = Command::new("timeout")
        .args(["--kill-after=5s", "60s"])
        .arg(cbmc())
        .arg(&file)
        .args([
            "--function",
            "main",
            "--64",
            "--little-endian",
            "--bounds-check",
            "--pointer-check",
            "--pointer-overflow-check",
            "--div-by-zero-check",
            "--signed-overflow-check",
            "--undefined-shift-check",
            "--unwind",
            "65",
            "--unwinding-assertions",
        ])
        .output()
        .expect("CBMC must be installed");
    let log = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    fs::write(directory.join("cbmc.log"), &log).unwrap();
    let verdict = verification_result(out.status.code(), &log);
    (verdict, directory, log)
}

fn cover(source: &str, directory: &std::path::Path, mask: u64, ymask: u64, width: usize) {
    let needle = "__CPROVER_assert(actual==expected,";
    assert_eq!(source.matches(needle).count(), 1);
    let instrumented=source.replace(needle,&format!("__CPROVER_cover(x==0 && y==0);__CPROVER_cover(x==0x{mask:x}ULL && y==0x{ymask:x}ULL);__CPROVER_cover(x==0x{mask:x}ULL && y=={}ULL);{needle}",width-1));
    let file = directory.join("harness-cover.c");
    fs::write(&file, instrumented).unwrap();
    let out = Command::new("timeout")
        .args(["--kill-after=5s", "60s"])
        .arg(cbmc())
        .arg(file)
        .args([
            "--function",
            "main",
            "--64",
            "--little-endian",
            "--unwind",
            "65",
            "--cover",
            "cover",
        ])
        .output()
        .unwrap();
    let log = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    fs::write(directory.join("covers.log"), &log).unwrap();
    let goals: Vec<_> = log
        .lines()
        .filter(|l| l.starts_with("[main.coverage."))
        .collect();
    assert!(
        out.status.success()
            && goals.len() == 3
            && goals.iter().all(|l| l.trim_end().ends_with(": SATISFIED")),
        "CBMC input reachability failed: {}\n{log}",
        directory.display()
    );
}

fn prove(op: Op, width: usize) {
    let shift = matches!(op, Op::LogicShiftR);
    let expression = ProtoExpression::Binary {
        x: Box::new(variable(0, width)),
        op,
        y: Box::new(variable(8, if shift { 64 } else { width })),
        width,
        expr_context: ExpressionContext {
            width,
            signed: false,
        },
    };
    let generated = emit_expr(&expression).expect("supported scalar expression was declined");
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .unwrap()
        .to_path_buf();
    let producer =
        fs::read_to_string(repo.join("veryl/crates/simulator/src/backend/aot_c/emit.rs")).unwrap();
    for declaration in LOAD_TYPES.lines() {
        assert_eq!(
            producer.matches(declaration).count(),
            4,
            "generated-C ABI declaration changed"
        );
    }

    let mask = if width == 64 {
        u64::MAX
    } else {
        (1u64 << width) - 1
    };
    let reference = match op {
        Op::Add => format!(
            "bool carry=false; for(unsigned i=0;i<{width};i++){{bool a=((x>>i)&1)!=0,b=((y>>i)&1)!=0;bool sum=(a!=b)!=carry;bool next=(a&&b)||((a!=b)&&carry);if(sum)expected|=((uint64_t)1)<<i;carry=next;}}"
        ),
        Op::BitXor => format!(
            "for(unsigned i=0;i<{width};i++){{bool a=((x>>i)&1)!=0,b=((y>>i)&1)!=0;if(a!=b)expected|=((uint64_t)1)<<i;}}"
        ),
        Op::LogicShiftR => format!(
            "if(y<{width})for(unsigned i=0;i<{width};i++){{if(i+y<{width} && ((x>>(i+y))&1)!=0)expected|=((uint64_t)1)<<i;}}"
        ),
        _ => unreachable!(),
    };
    let ymask = if shift { u64::MAX } else { mask };
    let source = format!(
        "#include <stdint.h>\n#include <stdbool.h>\n{LOAD_TYPES}extern uint64_t nondet_u64(void);\nint main(void){{uint64_t storage[2];uint64_t x=nondet_u64()&0x{mask:x}ULL,y=nondet_u64()&0x{ymask:x}ULL;storage[0]=x;storage[1]=y;uint8_t *comb_values=(uint8_t*)storage;uint8_t ff_values[8]={{0}};uint64_t actual=({generated});uint64_t expected=0;{reference}__CPROVER_assert(actual==expected,\"production-C-reference\");return 0;}}\n"
    );
    let (verdict, directory, log) = verify(&source);
    if verdict == Verdict::SemanticMismatch {
        panic!(
            "generated C expression mismatch: op={op:?} width={width}; {}",
            directory.display()
        );
    }
    assert_eq!(
        verdict,
        Verdict::Proved,
        "CBMC did not prove a safe harness: op={op:?} width={width}; {}\n{log}",
        directory.display()
    );
    cover(&source, &directory, mask, ymask, width);
    eprintln!(
        "generated-C complete op={op:?} width={width} covers=3; {}",
        directory.display()
    );
}

#[test]
fn emitted_addition_8() {
    prove(Op::Add, 8);
}
#[test]
fn emitted_addition_32() {
    prove(Op::Add, 32);
}
#[test]
fn emitted_addition_64() {
    prove(Op::Add, 64);
}
#[test]
fn emitted_xor_8() {
    prove(Op::BitXor, 8);
}
#[test]
fn emitted_xor_32() {
    prove(Op::BitXor, 32);
}
#[test]
fn emitted_xor_64() {
    prove(Op::BitXor, 64);
}
#[test]
fn emitted_shift_8() {
    prove(Op::LogicShiftR, 8);
}
#[test]
fn emitted_shift_32() {
    prove(Op::LogicShiftR, 32);
}
#[test]
fn emitted_shift_64() {
    prove(Op::LogicShiftR, 64);
}

#[test]
fn cbmc_incomplete_and_abnormal_results_are_rejected() {
    let failure = "[main.assertion.1] production-C-reference: FAILURE\n";
    let completed = format!("{failure}VERIFICATION FAILED\n");
    assert_eq!(
        verification_result(Some(10), &completed),
        Verdict::SemanticMismatch
    );
    assert_eq!(verification_result(Some(10), failure), Verdict::Rejected);
    for code in [Some(0), Some(1), Some(124), Some(137), None] {
        assert_eq!(verification_result(code, &completed), Verdict::Rejected);
    }
    let mixed = format!("[main.pointer.1] pointer outside bounds: FAILURE\n{completed}");
    assert_eq!(verification_result(Some(10), &mixed), Verdict::Rejected);
}

#[test]
fn cbmc_safety_and_parse_failures_are_distinct_from_semantic_mutations() {
    let cases = [
        (
            "int main(void){__CPROVER_assert(1,\"production-C-reference\");}",
            Verdict::Proved,
        ),
        (
            "int main(void){__CPROVER_assert(0,\"production-C-reference\");}",
            Verdict::SemanticMismatch,
        ),
        ("int main(void){THIS_IS_NOT_C}", Verdict::Rejected),
        (
            "extern unsigned nondet(void);int main(void){int a[1]={0};unsigned i=nondet();int x=a[i];__CPROVER_assert(x==x,\"production-C-reference\");}",
            Verdict::Rejected,
        ),
        (
            "extern unsigned nondet(void);int main(void){int a[1]={0};unsigned i=nondet();int x=a[i];__CPROVER_assert(x==1,\"production-C-reference\");}",
            Verdict::Rejected,
        ),
    ];
    for (source, want) in cases {
        let (got, _, log) = verify(source);
        assert_eq!(got, want, "CBMC failure classification changed: {log}");
    }
}

/// Fixed composed IR: signed comparison chooses a widened signed sum or an
/// eight-/32-bit XOR sign-extended by the ternary. Arbitrary clean inputs;
/// fixed valid ABI locations. Does not establish source lowering or all casts.
fn prove_signed_tree(width: usize) {
    let wide = width * 2;
    let signed_var = |offset| {
        let mut v = variable(offset, width);
        if let ProtoExpression::Variable { expr_context, .. } = &mut v {
            expr_context.signed = true;
        }
        v
    };
    let binary = |op, result_width, ctx_width, signed| ProtoExpression::Binary {
        x: Box::new(signed_var(0)),
        op,
        y: Box::new(signed_var(8)),
        width: result_width,
        expr_context: ExpressionContext {
            width: ctx_width,
            signed,
        },
    };
    let expression = ProtoExpression::Ternary {
        cond: Box::new(binary(Op::Less, 1, 1, false)),
        true_expr: Box::new(binary(Op::Add, wide, wide, true)),
        false_expr: Box::new(binary(Op::BitXor, width, width, true)),
        width: wide,
        expr_context: ExpressionContext {
            width: wide,
            signed: true,
        },
    };
    let generated = emit_expr(&expression).expect("signed composed expression declined");
    let mask = (1u64 << width) - 1;
    let source = format!(
        "#include <stdint.h>\n#include <stdbool.h>\n{LOAD_TYPES}extern uint64_t nondet_u64(void);\nint main(void){{uint64_t storage[2];uint64_t x=nondet_u64()&0x{mask:x}ULL,y=nondet_u64()&0x{mask:x}ULL;storage[0]=x;storage[1]=y;uint8_t *comb_values=(uint8_t*)storage;uint8_t ff_values[8]={{0}};uint64_t actual=({generated});uint64_t expected=0;bool nx=((x>>{})&1)!=0,ny=((y>>{})&1)!=0;bool less=nx!=ny?nx:x<y;uint64_t sx=x,sy=y;for(unsigned i={width};i<{wide};i++){{if(nx)sx|=((uint64_t)1)<<i;if(ny)sy|=((uint64_t)1)<<i;}}if(less){{bool carry=false;for(unsigned i=0;i<{wide};i++){{bool a=((sx>>i)&1)!=0,b=((sy>>i)&1)!=0;bool sum=(a!=b)!=carry;carry=(a&&b)||((a!=b)&&carry);if(sum)expected|=((uint64_t)1)<<i;}}}}else{{for(unsigned i=0;i<{wide};i++){{unsigned at=i<{width}?i:{};if((((x>>at)&1)!=0)!=(((y>>at)&1)!=0))expected|=((uint64_t)1)<<i;}}}}__CPROVER_assert(actual==expected,\"production-C-reference\");return 0;}}",
        width - 1,
        width - 1,
        width - 1
    );
    let (verdict, directory, log) = verify(&source);
    if verdict == Verdict::SemanticMismatch {
        panic!(
            "generated C expression mismatch: signed tree width={width}; {}",
            directory.display()
        );
    }
    assert_eq!(
        verdict,
        Verdict::Proved,
        "unsafe/unproved signed tree: {}\n{log}",
        directory.display()
    );
    cover(&source, &directory, mask, mask, width);
}
#[test]
fn emitted_signed_tree_8() {
    prove_signed_tree(8);
}
#[test]
fn emitted_signed_tree_32() {
    prove_signed_tree(32);
}

/// Actual comb-store producer with an explicit narrowing As node and a fixed
/// eight-bit window in a sixteen-bit destination. Source padding, destination
/// padding, and all surrounding storage are arbitrary. The bit oracle compares
/// the entire allocation. No FF/log, symbolic address, or C compiler theorem.
#[test]
fn emitted_cast_store_preserves_dirty_padding_and_guards() {
    use veryl_simulator::ir::{ProtoAssignStatement, ProtoStatement};
    let rhs = ProtoExpression::Binary {
        x: Box::new(variable(0, 12)),
        op: Op::As,
        y: Box::new(ProtoExpression::Value {
            value: veryl_simulator::ir::Value::new(8, 32, false),
            width: 32,
            expr_context: ExpressionContext {
                width: 32,
                signed: false,
            },
        }),
        width: 8,
        expr_context: ExpressionContext {
            width: 8,
            signed: false,
        },
    };
    let statement = ProtoStatement::Assign(ProtoAssignStatement {
        dst: VarOffset::Comb(8),
        dst_width: 16,
        select: Some((11, 4)),
        dynamic_select: None,
        rhs_select: None,
        expr: rhs,
        dst_ff_current_offset: 0,
        comb_direct: false,
        token: Default::default(),
    });
    let generated =
        veryl_simulator::backend::aot_c::emit_stmt(&statement).expect("cast/store declined");
    let source = format!(
        "#include <stdint.h>\n#include <stdbool.h>\n#include <string.h>\n{LOAD_TYPES}extern uint64_t nondet_u64(void);int main(void){{uint64_t storage[4],before[4];for(unsigned i=0;i<4;i++)before[i]=storage[i]=nondet_u64();uint64_t x=storage[0],y=storage[1];uint8_t *comb_values=(uint8_t*)storage;uint8_t ff_values[8]={{0}};{generated}bool actual=true,expected=true;for(unsigned word=0;word<4;word++)for(unsigned bit=0;bit<64;bit++){{bool want=((before[word]>>bit)&1)!=0;if(word==1 && bit>=4 && bit<12)want=((x>>(bit-4))&1)!=0;if((((storage[word]>>bit)&1)!=0)!=want)actual=false;}}__CPROVER_assert(actual==expected,\"production-C-reference\");return 0;}}"
    );
    let (verdict, directory, log) = verify(&source);
    if verdict == Verdict::SemanticMismatch {
        panic!(
            "generated C expression mismatch: cast/store {};",
            directory.display()
        );
    }
    assert_eq!(
        verdict,
        Verdict::Proved,
        "cast/store safety/proof failed: {}\n{log}",
        directory.display()
    );
    cover(&source, &directory, u64::MAX, u64::MAX, 12);
}
