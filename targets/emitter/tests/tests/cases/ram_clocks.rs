//! Finite two-clock masked RAM trace against native, synthesized, and emitted SV.
//! Both words are initialized by actual writes before observation. All three-step
//! clock subsets in both event orders and selected write/data/mask histories
//! are checked from reset.
//! Simultaneous FFs and RAM reads use the old state. No metastability, arbitrary
//! CDC, SRAM technology, derived clock, X/Z, or arbitrary-history guarantee.
use super::sequential::cells;
use super::{
    common,
    sequential::{Observed, analyze_source},
};
use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};
use veryl_parser::resource_table;
use veryl_simulator::{
    Simulator,
    ir::{Config, Value, build_ir},
};
use veryl_synthesizer::{
    RamConfig, build_gate_ir_with,
    ir::{ClockEdge, GateModule, ResetPolarity},
};
const SOURCE: &str = include_str!("../../../../simulator/fixtures/ram_clocks.veryl");
fn word(g: &GateModule, nets: &[u32], values: &HashMap<u32, bool>) -> u8 {
    let mut cache = vec![None; g.nets.len()];
    nets.iter().enumerate().fold(0, |v, (i, &n)| {
        v | (u8::from(cells::eval(g, n, values, &mut cache, 0)) << i)
    })
}
fn port(g: &GateModule, name: &str) -> Vec<u32> {
    g.ports
        .iter()
        .find(|p| resource_table::get_str_value(p.name).unwrap() == name)
        .unwrap()
        .nets
        .clone()
}
struct Machine<'a> {
    g: &'a GateModule,
    ff: Vec<bool>,
    mem: [u8; 2],
}
impl<'a> Machine<'a> {
    fn values(&self, inputs: &[(&str, u8)]) -> HashMap<u32, bool> {
        let mut values = HashMap::new();
        for &(name, value) in inputs {
            for (bit, net) in port(self.g, name).into_iter().enumerate() {
                values.insert(net, value & (1 << bit) != 0);
            }
        }
        for (f, &v) in self.g.ffs.iter().zip(&self.ff) {
            values.insert(f.q, v);
        }
        for r in &self.g.ram_blocks[0].read_ports {
            let address = word(self.g, &r.addr, &values) as usize;
            for (bit, &net) in r.data.iter().enumerate() {
                values.insert(net, self.mem[address] & (1 << bit) != 0);
            }
        }
        values
    }
    fn step(&mut self, inputs: &[(&str, u8)], events: u8) {
        let v = self.values(inputs);
        let ca = port(self.g, "clk_a")[0];
        let cb = port(self.g, "clk_b")[0];
        let active = |clk| {
            assert!(clk == ca || clk == cb, "unexpected derived clock");
            if clk == ca {
                events & 1 != 0
            } else {
                events & 2 != 0
            }
        };
        let ff = self
            .g
            .ffs
            .iter()
            .zip(&self.ff)
            .map(|(f, &old)| {
                assert_eq!(f.clock_edge, ClockEdge::Posedge);
                let reset = f.reset.as_ref().is_some_and(|r| {
                    (word(self.g, &[r.net], &v) != 0) == (r.polarity == ResetPolarity::ActiveHigh)
                });
                if reset && (active(f.clock) || !f.reset.as_ref().unwrap().sync) {
                    f.reset_value
                } else if active(f.clock) {
                    word(self.g, &[f.d], &v) != 0
                } else {
                    old
                }
            })
            .collect();
        let ram = &self.g.ram_blocks[0];
        assert_eq!(ram.clock_edge, ClockEdge::Posedge);
        if active(ram.clock) {
            for p in &ram.write_ports {
                if word(self.g, &[p.enable], &v) != 0 {
                    let address = word(self.g, &p.addr, &v) as usize;
                    let data = word(self.g, &p.data, &v);
                    let mask = p.mask.as_ref().map_or(3, |m| word(self.g, m, &v));
                    self.mem[address] = (self.mem[address] & !mask) | (data & mask);
                }
            }
        }
        self.ff = ff;
    }
    fn check(&self, inputs: &[(&str, u8)], expected: [u8; 3]) {
        let values = self.values(inputs);
        for (name, want) in ["result", "qa", "qb"].into_iter().zip(expected) {
            assert_eq!(
                word(self.g, &port(self.g, name), &values),
                want,
                "RAM clock cross output mismatch: gate {name}"
            );
        }
    }
}
fn compare() {
    let air = analyze_source(SOURCE);
    let gate = build_gate_ir_with(
        &air,
        resource_table::insert_str("Top"),
        RamConfig {
            min_bits: 1,
            max_write_ports: 2,
            ..Default::default()
        },
    )
    .unwrap()
    .module;
    assert_eq!(gate.ram_blocks.len(), 1);
    assert_eq!((gate.ram_blocks[0].width, gate.ram_blocks[0].depth), (2, 2));
    let calls = Arc::new(AtomicUsize::new(0));
    let mut sims = Vec::new();
    for (jit, cc) in [(false, false), (true, false), (true, true)] {
        let cfg = Config {
            use_jit: jit,
            aot_c: cc,
            aot_c_event: cc,
            aot_c_async: false,
            aot_c_min_stmts: 0,
            disable_ff_opt: true,
            ..Default::default()
        };
        let mut ir = build_ir(&air, "Top".into(), &cfg).unwrap();
        let (_, _, compiled, interpreted) = ir.detailed_stats();
        if cc {
            super::sequential::routes::assert_cc_comb_ready(&ir);
            assert!(!ir.whole_events.is_empty(), "RAM C route absent");
            for e in ir.whole_events.values_mut() {
                *e = Arc::new(Observed {
                    inner: e.clone(),
                    calls: calls.clone(),
                });
            }
        } else if jit {
            assert!(compiled > 0 && interpreted == 0, "RAM JIT fallback");
        } else {
            assert!(compiled == 0 && interpreted > 0);
        }
        sims.push(Simulator::new(ir, None));
    }
    let mut bench="module testbench;reg clk_a=0,clk_b=0,rst_a=0,rst_b=0,we=0,wa=0,ra=0;reg [1:0] data=0,mask=0;wire[1:0]result,qa,qb;prj_Top dut(.*);initial begin\n".to_owned();
    let mut outputs = Vec::new();
    for history in 0..128 {
        for pattern in 0..16 {
            let mut machine = Machine {
                g: &gate,
                ff: vec![false; gate.ffs.len()],
                mem: [0; 2],
            };
            let mut input = [
                ("clk_a", 1),
                ("clk_b", 1),
                ("rst_a", 1),
                ("rst_b", 1),
                ("we", 1),
                ("wa", 0),
                ("ra", 0),
                ("data", 0),
                ("mask", 3),
            ];
            bench += "clk_a=0;clk_b=0;rst_a=1;rst_b=1;we=1;mask=3;wa=0;ra=0;data=0;#1;clk_a=1;#1;clk_a=0;wa=1;data=3;#1;clk_a=1;#1;clk_a=0;we=0;rst_a=0;rst_b=0;#1;\n";
            for sim in &mut sims {
                let ca = sim.get_clock("clk_a").unwrap();
                let cb = sim.get_clock("clk_b").unwrap();
                let ra = sim.get_reset("rst_a").unwrap();
                let rb = sim.get_reset("rst_b").unwrap();
                sim.set("we", Value::new(0, 1, false));
                sim.set("ra", Value::new(0, 1, false));
                sim.step_reset(&ca, &ra);
                sim.step_reset(&cb, &rb);
                sim.set_reset_level(&ra.var_id().unwrap(), true);
                sim.set_reset_level(&rb.var_id().unwrap(), true);
                sim.set("we", Value::new(1, 1, false));
                sim.set("mask", Value::new(3, 2, false));
                for (addr, data) in [(0, 0), (1, 3)] {
                    sim.set("wa", Value::new(addr, 1, false));
                    sim.set("data", Value::new(data, 2, false));
                    sim.step(&ca);
                }
                sim.set_reset_level(&ra.var_id().unwrap(), false);
                sim.set_reset_level(&rb.var_id().unwrap(), false);
            }
            machine.step(&input, 1);
            input[5].1 = 1;
            input[7].1 = 3;
            machine.step(&input, 1);
            input[2].1 = 0;
            input[3].1 = 0;
            let mut mem = [0u8, 3];
            let (mut qa, mut qb) = (0, 0);
            let mut code = history % 64;
            for stage in 0..3 {
                let events = code % 4;
                code /= 4;
                let data = ((pattern + stage * 3) & 3) as u8;
                let mask = ((pattern / 4 + stage) & 3) as u8;
                let wa = ((pattern + stage) & 1) as u8;
                let ra = ((pattern / 2 + stage) & 1) as u8;
                let we = ((pattern + stage / 2) & 1) as u8;
                for (name, value) in [
                    ("we", we),
                    ("wa", wa),
                    ("ra", ra),
                    ("data", data),
                    ("mask", mask),
                ] {
                    input.iter_mut().find(|(n, _)| *n == name).unwrap().1 = value;
                }
                let old_a = qa;
                let old_read = mem[ra as usize];
                if events & 1 != 0 {
                    qa = old_read;
                    if we != 0 {
                        mem[wa as usize] = (mem[wa as usize] & !mask) | (data & mask);
                    }
                }
                if events & 2 != 0 {
                    qb = old_a;
                }
                let expected = [mem[ra as usize], qa, qb];
                for (backend, sim) in sims.iter_mut().enumerate() {
                    for &(name, v) in &input[4..] {
                        sim.set(
                            name,
                            Value::new(
                                u64::from(v),
                                if name == "data" || name == "mask" {
                                    2
                                } else {
                                    1
                                },
                                false,
                            ),
                        );
                    }
                    let ca = sim.get_clock("clk_a").unwrap();
                    let cb = sim.get_clock("clk_b").unwrap();
                    let mut clocks = Vec::new();
                    if events & 1 != 0 {
                        clocks.push(ca);
                    }
                    if events & 2 != 0 {
                        clocks.push(cb);
                    }
                    if history >= 64 {
                        clocks.reverse();
                    }
                    sim.step_events(&clocks, &[]);
                    for (name, want) in ["result", "qa", "qb"].into_iter().zip(expected) {
                        let got = sim.get(name).unwrap();
                        assert_eq!(
                            (
                                got.payload().to_u64_digits().first().copied().unwrap_or(0),
                                got.mask_xz().bits()
                            ),
                            (u64::from(want), 0),
                            "RAM clock cross output mismatch: native={backend} {name} history={history} pattern={pattern} stage={stage}"
                        );
                    }
                }
                machine.step(&input, events as u8);
                machine.check(&input, expected);
                bench += &format!(
                    "we={we};wa={wa};ra={ra};data=2'd{data};mask=2'd{mask};#1;clk_a={};clk_b={};#1;$display(\"case %0d %0d %0d\",result,qa,qb);clk_a=0;clk_b=0;#1;\n",
                    events & 1,
                    (events >> 1) & 1
                );
                outputs.push(format!("{} {} {}", expected[0], expected[1], expected[2]));
            }
        }
    }
    assert!(calls.load(Ordering::Relaxed) > 0, "RAM C never dispatched");
    bench += "$display(\"complete\");$finish;end endmodule\n";
    assert_eq!(
        common::execute(SOURCE, &bench),
        outputs,
        "RAM clock cross output mismatch: SV"
    );
}
#[test]
fn masked_ram_and_simultaneous_clocks() {
    if std::env::var_os("VERYL_FORMAL_RAM_CHILD").is_some() {
        compare();
        return;
    }
    let mut command = std::process::Command::new(std::env::current_exe().unwrap());
    command
        .args([
            "--exact",
            "ram_clocks::masked_ram_and_simultaneous_clocks",
            "--nocapture",
        ])
        .env("VERYL_FORMAL_RAM_CHILD", "1")
        .env(
            "VERYL_AOT_CACHE_DIR",
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/aot-cache"),
        );
    let out = command.output().unwrap();
    let log = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(out.status.success(), "RAM clock child failed: {log}");
    assert!(log.contains("1 passed; 0 failed"), "RAM child incomplete");
    eprintln!("{log}");
}
