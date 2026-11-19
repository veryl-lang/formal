//! Finite 512-bit old-state RHS and ordered partial-write histories.
//! A 510-bit window spans two wide-log records, followed by an overlapping
//! 65-bit write. All end selectors are in range (IEEE 1800-2023 11.5.1).
//! IEEE 1800-2023 4.6/10.4.2 supply statement order and nonblocking RHS snapshots.
//! Two dense seeds, FF optimization enabled/disabled, four RHS choices and
//! sixteen clocks give 256 updates/backend. This is an executable finite
//! comparison, not an exhaustive proof. Other widths/selectors, X/Z and initial
//! blocks are excluded. Postclock q current/next slots and previous current
//! bytes are checked independently of Value transport; actual C commits use
//! the shared trace helper. Next-slot contents before the first write are not
//! an assumption of this fixture.
use super::{Backend, Simulator, Value, analyze_config, check_commit, value};
const WIDTH: usize = 512;
const MESSAGE: &str = "simulator chunked dual-slot state mismatch";
const STARTS: [usize; 9] = [0, 1, 63, 64, 127, 191, 255, 383, 447];
fn source(seed: &[u8]) -> String {
    let init = seed
        .iter()
        .rev()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    format!(
        r#"module Top(clk:input clock,rst:input reset,
        a:input logic<510>,b:input logic<65>,rhs_choice:input logic<2>,
        ia:input logic<10>,sa:input logic<10>,ib:input logic<10>,sb:input logic<10>,
        out:output logic<512>,previous:output logic<512>){{
        var q:logic<512>;var prev:logic<512>;
        always_ff{{if_reset{{q=512'h{init};prev=512'h{init};}}else{{
            if rhs_choice[0]{{q[ia-:510]=q[sa-:510];}}else{{q[ia-:510]=a;}}
            if rhs_choice[1]{{q[ib-:65]=q[sb-:65];}}else{{q[ib-:65]=b;}}
            prev=q;
        }}}}
        assign out=q;assign previous=prev;
    }}"#
    )
}
fn bit(bytes: &[u8], pos: usize) -> u8 {
    bytes[pos / 8] >> (pos % 8) & 1
}
fn patch(dst: &mut [u8], start: usize, width: usize, rhs: &[u8], source_start: usize) {
    for i in 0..width {
        let pos = start + i;
        dst[pos / 8] =
            (dst[pos / 8] & !(1 << (pos % 8))) | (bit(rhs, source_start + i) << (pos % 8));
    }
}
fn different(a: &[u8], b: &[u8], start: usize, width: usize) -> bool {
    (start..start + width).any(|pos| bit(a, pos) != bit(b, pos))
}
fn inspect(sim: &Simulator, name: &str, expected: &[u8], both_slots: bool) {
    let v = sim
        .ir
        .module_variables
        .variables
        .values()
        .find(|v| v.path.to_string() == name)
        .unwrap();
    assert_eq!(v.width, WIDTH);
    assert_eq!(v.native_bytes, 64);
    assert_eq!(v.current_values.len(), 1);
    if both_slots {
        assert_eq!(v.next_values.len(), 1);
    }
    // FF optimization can place the single-write previous value in the
    // combinational arena; validate its actual storage before reading raw bytes.
    let arena = if v.next_values.is_empty() {
        &sim.ir.comb_values
    } else {
        &sim.ir.ff_values
    };
    let base = arena.as_ptr() as usize;
    let end = base + arena.len();
    let pointers = if both_slots {
        vec![v.current_values[0], v.next_values[0]]
    } else {
        vec![v.current_values[0]]
    };
    for p in pointers {
        let address = p as usize;
        assert!(
            address >= base && address.checked_add(64).is_some_and(|last| last <= end),
            "chunked variable allocation outside its live arena: {name}"
        );
        // The complete 64-byte live extent was checked against the arena above.
        let bytes = unsafe { std::slice::from_raw_parts(p, 64) };
        for pos in 0..WIDTH {
            assert_eq!(
                bit(bytes, pos),
                bit(expected, pos),
                "{MESSAGE}: {name} slot bit={pos}"
            );
        }
    }
    // Value assertions precede the chosen layout assertion so a metadata mutant
    // must produce the intended semantic failure, rather than only a route fault.
    if both_slots {
        assert_eq!(
            v.next_values[0] as usize,
            v.current_values[0] as usize + 64,
            "multi-write q requires a separate next slot"
        );
    }
}
fn compare(backend: Backend) {
    let mut updates = 0;
    let mut upper_changes = 0;
    let mut rhs_distinctions = 0;
    let mut forwarding_distinctions = 0;
    let mut overlap_changes = 0;
    for disabled in [false, true] {
        for seed_id in 0..2 {
            let mut state = (0..64)
                .map(|i| ((i * 73 + seed_id * 91 + 0x55) as u8).rotate_left((i % 7) as u32))
                .collect::<Vec<_>>();
            state[63] |= 0x80;
            let (ir, trace) = analyze_config(&source(&state), backend, disabled);
            let mut sim = Simulator::new(ir, None);
            let clk = sim.get_clock("clk").unwrap();
            let rst = sim.get_reset("rst").unwrap();
            let calls = trace.lock().unwrap().calls;
            sim.step_reset(&clk, &rst);
            check_commit(&sim, &trace, backend, calls);
            assert_eq!(
                sim.get("out"),
                Some(value(&state, WIDTH)),
                "{MESSAGE}: reset"
            );
            inspect(&sim, "q", &state, false);
            inspect(&sim, "prev", &state, false);
            for choice in 0u64..4 {
                for cycle in 0usize..16 {
                    let da = cycle % 3;
                    let sa = (cycle + choice as usize + 1) % 3;
                    let db = STARTS[(cycle + choice as usize) % STARTS.len()];
                    let sb = STARTS[(cycle * 3 + choice as usize + 2) % STARTS.len()];
                    let mut a = (0..64)
                        .map(|i| {
                            state[i]
                                .wrapping_add((cycle * 37 + i * 113 + 0x31) as u8)
                                .rotate_left((cycle % 8) as u32)
                        })
                        .collect::<Vec<_>>();
                    a[63] &= 0x3f;
                    let mut b = vec![0u8; 16];
                    for (i, v) in b[..8].iter_mut().enumerate() {
                        *v = (cycle * 41 + i * 67 + seed_id * 11 + 0x93) as u8;
                    }
                    b[8] = (cycle & 1) as u8;
                    sim.set("a", value(&a, 510));
                    sim.set("b", value(&b, 65));
                    sim.set("rhs_choice", Value::new(choice, 2, false));
                    for (name, end) in [
                        ("ia", 509 + da),
                        ("sa", 509 + sa),
                        ("ib", 64 + db),
                        ("sb", 64 + sb),
                    ] {
                        sim.set(name, Value::new(end as u64, 10, false));
                    }
                    assert_eq!(
                        sim.get("out"),
                        Some(value(&state, WIDTH)),
                        "{MESSAGE}: input drive changed state"
                    );
                    let old = state.clone();
                    let (ra, ra_start) = if choice & 1 != 0 {
                        (&old[..], sa)
                    } else {
                        (&a[..], 0)
                    };
                    patch(&mut state, da, 510, ra, ra_start);
                    let first = state.clone();
                    rhs_distinctions +=
                        usize::from(choice & 2 != 0 && different(&first, &old, sb, 65));
                    let word_start = db / 64 * 64;
                    let word_end = (db + 65).div_ceil(64) * 64;
                    forwarding_distinctions +=
                        usize::from((word_start..word_end).any(|p| {
                            !(db..db + 65).contains(&p) && bit(&first, p) != bit(&old, p)
                        }));
                    let (rb, rb_start) = if choice & 2 != 0 {
                        (&old[..], sb)
                    } else {
                        (&b[..], 0)
                    };
                    patch(&mut state, db, 65, rb, rb_start);
                    overlap_changes += usize::from(different(&first, &state, db, 65));
                    upper_changes += usize::from(different(&old, &state, 448, 64));
                    let calls = trace.lock().unwrap().calls;
                    sim.step(&clk);
                    assert_eq!(
                        sim.get("out"),
                        Some(value(&state, WIDTH)),
                        "{MESSAGE}: backend={backend:?} disabled={disabled} seed={seed_id} choice={choice} cycle={cycle}"
                    );
                    assert_eq!(
                        sim.get("previous"),
                        Some(value(&old, WIDTH)),
                        "{MESSAGE}: RHS did not read old current"
                    );
                    inspect(&sim, "q", &state, true);
                    inspect(&sim, "prev", &old, false);
                    check_commit(&sim, &trace, backend, calls);
                    updates += 1;
                }
            }
        }
    }
    assert_eq!(updates, 256);
    assert!(
        upper_changes > 0
            && rhs_distinctions > 0
            && forwarding_distinctions > 0
            && overlap_changes > 0,
        "chunked distinguishing histories absent"
    );
    eprintln!(
        "chunked dual-slot complete backend={backend:?} updates={updates} upper={upper_changes} old_rhs={rhs_distinctions} forwarding={forwarding_distinctions} overlap={overlap_changes}"
    );
}
#[test]
fn interpreter_dual_slot_chunks() {
    compare(Backend::Interpret);
}
#[test]
fn cranelift_dual_slot_chunks() {
    compare(Backend::Cranelift);
}
#[test]
fn aot_c_dual_slot_chunks() {
    compare(Backend::Cc);
}
