//! Finite source-to-commit dual-slot histories, complementing the symbolic kernel
//! checks in dual_slot_window.rs. IEEE 1800-2023 4.6 and 10.4.2 establish ordered
//! nonblocking updates with RHS values evaluated before those updates commit.
//! Widths 61/128/130, default/forced FF placement, two nonzero reset seeds, four
//! independent RHS-alias choices, and eight consecutive updates per choice are
//! checked in each backend. Dynamic destination indices are fixed at 5/11 or
//! 60/68; source windows are fixed at 17/3 or 7/57. Payloads are selected patterns,
//! not symbolic or exhaustive; bits 128/129 remain zero at width 130.
//! X/Z, initial blocks, arbitrary selectors/programs,
//! asynchronous reset timing, and general allocation correctness are excluded.
//! Layout and current/next equality are observed only inside the live FF arena.
use super::{Backend, Simulator, Value, analyze_config, check_commit};

fn source(width: usize, seed: u128) -> String {
    let (wa, wb, sa, sb) = if width == 61 {
        (13, 9, 17, 3)
    } else {
        (12, 10, 7, 57)
    };
    format!(
        r#"module Top(clk:input clock,rst:input reset,
        a:input logic<{wa}>,b:input logic<{wb}>,rhs_choice:input logic<2>,
        ia:input logic<8>,ib:input logic<8>,out:output logic<{width}>,previous:output logic<{width}>){{
        var q:logic<{width}>;var prev:logic<{width}>;
        always_ff{{if_reset{{q={width}'h{seed:x};prev={width}'h{seed:x};}}else{{
            if rhs_choice[0]{{q[ia+:{wa}]=q[{ahi}:{sa}];}}else{{q[ia+:{wa}]=a;}}
            if rhs_choice[1]{{q[ib+:{wb}]=q[{bhi}:{sb}];}}else{{q[ib+:{wb}]=b;}}
            prev=q;
        }}}}
        assign out=q;assign previous=prev;
    }}"#,
        ahi = sa + wa - 1,
        bhi = sb + wb - 1
    )
}
fn inspect_slots(sim: &Simulator, expected: u128, width: usize) {
    let q = sim
        .ir
        .module_variables
        .variables
        .values()
        .find(|v| v.path.to_string() == "q")
        .unwrap();
    assert_eq!(q.width, width);
    assert_eq!(q.current_values.len(), 1);
    assert_eq!(q.next_values.len(), 1);
    let base = sim.ir.ff_values.as_ptr() as usize;
    let end = base + sim.ir.ff_values.len();
    let nb = q.native_bytes;
    let current = q.current_values[0] as usize;
    let next = q.next_values[0] as usize;
    assert!(
        current >= base && current + nb <= end && next >= base && next + nb <= end,
        "dual-slot allocation outside FF arena"
    );
    assert_eq!(
        next,
        current + nb,
        "multi-write FF must have a distinct next slot"
    );
    let mask = if width >= 128 {
        u128::MAX
    } else {
        (1u128 << width) - 1
    };
    for ptr in [q.current_values[0], q.next_values[0]] {
        // The bounds above cover the entire live native allocation. Check all
        // logical bits, including bits 128/129 separately for the wide case.
        let bytes = unsafe { std::slice::from_raw_parts(ptr, nb) };
        for bit in 0..width {
            let want = if bit < 128 { (expected >> bit) & 1 } else { 0 };
            assert_eq!(
                u128::from((bytes[bit / 8] >> (bit % 8)) & 1),
                want,
                "simulator dual-slot state mismatch: slot bit={bit} width={width} expected={:x}",
                expected & mask
            );
        }
    }
}
fn value(bits: u128, width: usize) -> Value {
    Value::from_u128(bits, 0, width, false)
}
fn compare(backend: Backend) {
    let mut vectors = 0;
    let mut overlap_changes = 0;
    let mut old_rhs_distinctions = 0;
    for width in [61, 128, 130] {
        let (da, db, wa, wb, sa, sb) = if width == 61 {
            (5, 11, 13, 9, 17, 3)
        } else {
            (60, 68, 12, 10, 7, 57)
        };
        let mask = if width >= 128 {
            u128::MAX
        } else {
            (1u128 << width) - 1
        };
        for forced in [false, true] {
            for seed in [
                0x1555_aaaa_0123_4567_89ab_cdef_aa55_3333u128,
                0xaaaa_5555_fedc_ba98_7654_3210_55aa_cccc,
            ] {
                let (ir, trace) = analyze_config(&source(width, seed & mask), backend, forced);
                let mut sim = Simulator::new(ir, None);
                let clk = sim.get_clock("clk").unwrap();
                let rst = sim.get_reset("rst").unwrap();
                sim.set("ia", Value::new(da, 8, false));
                sim.set("ib", Value::new(db, 8, false));
                let calls = trace.lock().unwrap().calls;
                sim.step_reset(&clk, &rst);
                check_commit(&sim, &trace, backend, calls);
                let mut state = seed & mask;
                for alias in 0..4 {
                    for cycle in 0..8 {
                        let a = ((cycle * 613 + 0x123) as u128) & ((1 << wa) - 1);
                        let b = ((cycle * 179 + 0x155) as u128) & ((1 << wb) - 1);
                        sim.set("a", value(a, wa));
                        sim.set("b", value(b, wb));
                        sim.set("rhs_choice", Value::new(alias, 2, false));
                        assert_eq!(
                            sim.get("out"),
                            Some(value(state, width)),
                            "simulator dual-slot state mismatch: input drive changed state"
                        );
                        let old = state;
                        let ra = if alias & 1 != 0 {
                            (old >> sa) & ((1 << wa) - 1)
                        } else {
                            a
                        };
                        let rb = if alias & 2 != 0 {
                            (old >> sb) & ((1 << wb) - 1)
                        } else {
                            b
                        };
                        // A scalar masked replacement is independent of the production
                        // byte/word store, write-log representation, and commit path.
                        let first = (old & !(((1 << wa) - 1) << da)) | (ra << da);
                        state = (first & !(((1 << wb) - 1) << db)) | (rb << db);
                        overlap_changes +=
                            usize::from((first ^ state) & (((1 << wa) - 1) << da) != 0);
                        old_rhs_distinctions +=
                            usize::from(alias & 2 != 0 && ((first >> sb) & ((1 << wb) - 1)) != rb);
                        let calls = trace.lock().unwrap().calls;
                        sim.step(&clk);
                        assert_eq!(
                            sim.get("out"),
                            Some(value(state, width)),
                            "simulator dual-slot state mismatch: backend={backend:?} width={width} forced={forced} alias={alias} cycle={cycle}"
                        );
                        assert_eq!(
                            sim.get("previous"),
                            Some(value(old, width)),
                            "simulator dual-slot state mismatch: RHS did not read old current"
                        );
                        inspect_slots(&sim, state, width);
                        check_commit(&sim, &trace, backend, calls);
                        vectors += 1;
                    }
                }
            }
        }
    }
    assert_eq!(vectors, 384);
    assert!(
        overlap_changes > 0 && old_rhs_distinctions > 0,
        "dual-slot distinguishing histories absent"
    );
    eprintln!(
        "dual-slot complete backend={backend:?} updates={vectors} overlap={overlap_changes} old_rhs={old_rhs_distinctions}"
    );
}
#[test]
fn interpreter_dual_slot() {
    compare(Backend::Interpret);
}
#[test]
fn cranelift_dual_slot() {
    compare(Backend::Cranelift);
}
#[test]
fn aot_c_dual_slot() {
    compare(Backend::Cc);
}
