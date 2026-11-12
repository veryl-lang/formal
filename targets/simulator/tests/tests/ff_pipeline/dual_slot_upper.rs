//! Finite source-to-commit histories with nonzero bits 128/129 in a 130-bit FF.
//! IEEE 1800-2023 4.6/10.4.2 establish ordered nonblocking updates and old-state
//! RHS evaluation. Both destination windows are wholly in range (11.5.1):
//! [118+:12] followed by [122+:8]. RHS aliases select old q[74:63] and q[128:121].
//! Two low-word seeds and upper patterns 1/2/3, both placement modes, four RHS
//! choices, and eight consecutive updates form 384 updates per backend.
//! A byte-vector reference checks values beyond u128, previous-state output,
//! live separate current/next slots, zero storage padding, and actual AOT commits.
//! Payloads and histories are finite, not symbolic or exhaustive. X/Z, initial
//! blocks, arbitrary selectors, out-of-range writes, and general layouts are
//! excluded. The preceding dual_slot fixture retains its independent domains.
use super::{Backend, Simulator, Value, analyze_config, check_commit, value};

const WIDTH: usize = 130;
const MESSAGE: &str = "simulator upper dual-slot state mismatch";

fn source(low: u128, upper: u8) -> String {
    format!(
        r#"module Top(clk:input clock,rst:input reset,
        a:input logic<12>,b:input logic<8>,rhs_choice:input logic<2>,
        ia:input logic<8>,ib:input logic<8>,out:output logic<130>,previous:output logic<130>){{
        var q:logic<130>;var prev:logic<130>;
        always_ff{{if_reset{{q=130'h{upper:x}{low:032x};prev=130'h{upper:x}{low:032x};}}else{{
            if rhs_choice[0]{{q[ia+:12]=q[74:63];}}else{{q[ia+:12]=a;}}
            if rhs_choice[1]{{q[ib+:8]=q[128:121];}}else{{q[ib+:8]=b;}}
            prev=q;
        }}}}
        assign out=q;assign previous=prev;
    }}"#
    )
}
fn bit(bytes: &[u8], index: usize) -> u8 {
    (bytes[index / 8] >> (index % 8)) & 1
}
fn field(bytes: &[u8], start: usize, width: usize) -> u16 {
    (0..width).fold(0, |v, i| v | (u16::from(bit(bytes, start + i)) << i))
}
fn patch(bytes: &mut [u8], start: usize, width: usize, rhs: u16) {
    for i in 0..width {
        let pos = start + i;
        bytes[pos / 8] =
            (bytes[pos / 8] & !(1 << (pos % 8))) | ((((rhs >> i) & 1) as u8) << (pos % 8));
    }
}
fn inspect_slots(sim: &Simulator, expected: &[u8]) {
    let q = sim
        .ir
        .module_variables
        .variables
        .values()
        .find(|v| v.path.to_string() == "q")
        .unwrap();
    assert_eq!(q.width, WIDTH);
    assert_eq!(q.current_values.len(), 1);
    assert_eq!(q.next_values.len(), 1);
    let base = sim.ir.ff_values.as_ptr() as usize;
    let end = base + sim.ir.ff_values.len();
    let nb = q.native_bytes;
    assert_eq!(nb, 24);
    let current = q.current_values[0] as usize;
    let next = q.next_values[0] as usize;
    assert!(
        current >= base && current + nb <= end && next >= base && next + nb <= end,
        "upper dual-slot allocation outside FF arena"
    );
    assert_eq!(
        next,
        current + nb,
        "upper multi-write FF must have a distinct next slot"
    );
    for ptr in [q.current_values[0], q.next_values[0]] {
        // Both complete slot extents were checked against the live arena above.
        let actual = unsafe { std::slice::from_raw_parts(ptr, nb) };
        for i in 0..nb * 8 {
            let want = if i < WIDTH { bit(expected, i) } else { 0 };
            assert_eq!(bit(actual, i), want, "{MESSAGE}: slot bit={i}");
        }
    }
}
fn compare(backend: Backend) {
    let mut vectors = 0;
    let mut upper_changes = 0;
    let mut upper_set = [0usize; 2];
    let mut old_rhs_distinctions = 0;
    let mut overlap_changes = 0;
    for forced in [false, true] {
        for low in [
            0x1555_aaaa_0123_4567_89ab_cdef_aa55_3333u128,
            0xaaaa_5555_fedc_ba98_7654_3210_55aa_cccc,
        ] {
            for upper in [1u8, 2, 3] {
                let (ir, trace) = analyze_config(&source(low, upper), backend, forced);
                let mut sim = Simulator::new(ir, None);
                let clk = sim.get_clock("clk").unwrap();
                let rst = sim.get_reset("rst").unwrap();
                sim.set("ia", Value::new(118, 8, false));
                sim.set("ib", Value::new(122, 8, false));
                let calls = trace.lock().unwrap().calls;
                sim.step_reset(&clk, &rst);
                check_commit(&sim, &trace, backend, calls);
                let mut state = vec![0u8; 17];
                state[..16].copy_from_slice(&low.to_le_bytes());
                state[16] = upper;
                assert_eq!(
                    sim.get("out"),
                    Some(value(&state, WIDTH)),
                    "{MESSAGE}: reset"
                );
                for alias in 0..4 {
                    for cycle in 0..8 {
                        let a = ((cycle * 613 + 0x123) & 0xfff) as u16;
                        let b = ((cycle * 179 + 0x155) & 0xff) as u16;
                        sim.set("a", Value::new(a as u64, 12, false));
                        sim.set("b", Value::new(b as u64, 8, false));
                        sim.set("rhs_choice", Value::new(alias, 2, false));
                        assert_eq!(
                            sim.get("out"),
                            Some(value(&state, WIDTH)),
                            "{MESSAGE}: input drive changed state"
                        );
                        let old = state.clone();
                        let ra = if alias & 1 != 0 {
                            field(&old, 63, 12)
                        } else {
                            a
                        };
                        let rb = if alias & 2 != 0 {
                            field(&old, 121, 8)
                        } else {
                            b
                        };
                        patch(&mut state, 118, 12, ra);
                        let first = state.clone();
                        patch(&mut state, 122, 8, rb);
                        overlap_changes +=
                            usize::from(field(&first, 122, 8) != field(&state, 122, 8));
                        old_rhs_distinctions +=
                            usize::from(alias & 2 != 0 && field(&first, 121, 8) != rb);
                        upper_changes += usize::from(old[16] != state[16]);
                        for (i, count) in upper_set.iter_mut().enumerate() {
                            *count += usize::from(bit(&state, 128 + i) != 0);
                        }
                        let calls = trace.lock().unwrap().calls;
                        sim.step(&clk);
                        assert_eq!(
                            sim.get("out"),
                            Some(value(&state, WIDTH)),
                            "{MESSAGE}: backend={backend:?} forced={forced} upper={upper} alias={alias} cycle={cycle}"
                        );
                        assert_eq!(
                            sim.get("previous"),
                            Some(value(&old, WIDTH)),
                            "{MESSAGE}: RHS did not read old current"
                        );
                        inspect_slots(&sim, &state);
                        check_commit(&sim, &trace, backend, calls);
                        vectors += 1;
                    }
                }
            }
        }
    }
    assert_eq!(vectors, 384);
    assert!(
        upper_changes > 0
            && upper_set.iter().all(|n| *n > 0)
            && old_rhs_distinctions > 0
            && overlap_changes > 0,
        "upper dual-slot distinguishing histories absent"
    );
    eprintln!(
        "upper dual-slot complete backend={backend:?} updates={vectors} upper_changes={upper_changes} upper_set={upper_set:?} overlap={overlap_changes} old_rhs={old_rhs_distinctions}"
    );
}
#[test]
fn interpreter_dual_slot_upper() {
    compare(Backend::Interpret);
}
#[test]
fn cranelift_dual_slot_upper() {
    compare(Backend::Cranelift);
}
#[test]
fn aot_c_dual_slot_upper() {
    compare(Backend::Cc);
}
