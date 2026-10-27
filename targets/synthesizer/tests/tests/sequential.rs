//! Finite two-state FF/reset and two-entry two-bit masked-RAM sequences, through
//! plain and mapped reconstruction. Explicit initial contents, four reset kinds,
//! and selected input streams define the domain. Reads observe prior FF state and
//! old RAM contents; masked lanes retain values and reset preserves RAM. No general
//! RAM technology, X/Z, arbitrary histories, or FF function-side-effect guarantee.

#[path = "common/clock.rs"]
mod clock;
mod common;
use std::collections::HashMap;
use veryl_analyzer::{Analyzer, Context, ir as air};
use veryl_metadata::Metadata;
use veryl_parser::{Parser, resource_table};
use veryl_synthesizer::{
    GateModule, RamConfig, build_gate_ir_with,
    ir::{ClockEdge, ResetPolarity},
};
fn analyze(reset: &str, registered: bool) -> GateModule {
    let read = if registered {
        "always_ff(clk){result=mem[raddr];}"
    } else {
        "assign result=mem[raddr];"
    };
    let source = format!(
        "module Top(clk:input clock,rst:input {reset},we:input logic,waddr:input logic,raddr:input logic,data:input logic<2>,mask:input logic<2>,result:output logic<2>,q:output logic<2>,previous:output logic<2>){{var mem:logic<2>[2];always_ff(clk){{if we{{if mask[0]{{mem[waddr][0]=data[0];}}if mask[1]{{mem[waddr][1]=data[1];}}}}}}{read}var state:logic<2>;var prev:logic<2>;always_ff(clk,rst){{if_reset{{state=1;prev=0;}}else if we{{state=state^data;prev=state;}}}}assign q=state;assign previous=prev;}}"
    );
    let metadata = Metadata::create_default("prj").unwrap();
    let analyzer = Analyzer::new(&metadata);
    analyzer.clear();
    let parser = Parser::parse(&source, &"").unwrap();
    let mut ctx = Context::default();
    let mut ir = air::Ir::default();
    let mut errors = analyzer.analyze_pass1("prj", &parser.veryl);
    errors.extend(Analyzer::analyze_post_pass1());
    errors.extend(analyzer.analyze_pass2(&parser.veryl, &mut ctx, Some(&mut ir)));
    errors.extend(Analyzer::analyze_post_pass2(&ir));
    assert!(
        errors.is_empty(),
        "sequential synthesis fixture diagnostics: {errors:?}"
    );
    let gate = build_gate_ir_with(
        &ir,
        resource_table::insert_str("Top"),
        RamConfig {
            min_bits: 1,
            max_write_ports: 2,
            ..Default::default()
        },
    )
    .unwrap()
    .module;
    assert_eq!(gate.ram_blocks.len(), 1, "fixture must infer a RAM");
    assert_eq!((gate.ram_blocks[0].depth, gate.ram_blocks[0].width), (2, 2));
    assert!(!gate.ffs.is_empty());
    assert!(gate.ram_blocks[0].read_ports.iter().all(|p| !p.sync));
    gate
}
struct Machine<'a> {
    gate: &'a GateModule,
    ff: Vec<bool>,
    mem: Vec<[u8; 2]>,
    ports: HashMap<String, Vec<u32>>,
}
impl<'a> Machine<'a> {
    fn new(gate: &'a GateModule, seed: u8) -> Self {
        clock::assert_single_clock(gate, "clk", "synthesizer sequential output mismatch");
        Self {
            gate,
            ff: vec![false; gate.ffs.len()],
            mem: vec![[seed & 3, (seed >> 2) & 3]; gate.ram_blocks.len()],
            ports: gate
                .ports
                .iter()
                .map(|p| {
                    (
                        resource_table::get_str_value(p.name).unwrap(),
                        p.nets.clone(),
                    )
                })
                .collect(),
        }
    }
    fn values(&self, inputs: &[(&str, u8)]) -> HashMap<u32, bool> {
        let mut values = HashMap::new();
        for (name, word) in inputs {
            for (bit, &net) in self.ports[*name].iter().enumerate() {
                values.insert(net, (word >> bit) & 1 != 0);
            }
        }
        for (ff, &value) in self.gate.ffs.iter().zip(&self.ff) {
            values.insert(ff.q, value);
        }
        for (i, ram) in self.gate.ram_blocks.iter().enumerate() {
            for port in &ram.read_ports {
                let address = self.word(&port.addr, &values);
                for (bit, &net) in port.data.iter().enumerate() {
                    values.insert(net, (self.mem[i][address as usize] >> bit) & 1 != 0);
                }
            }
        }
        values
    }
    fn bit(&self, net: u32, values: &HashMap<u32, bool>) -> bool {
        common::eval(
            self.gate,
            net,
            values,
            &mut vec![None; self.gate.nets.len()],
            0,
        )
    }
    fn word(&self, nets: &[u32], values: &HashMap<u32, bool>) -> u8 {
        nets.iter().enumerate().fold(0, |word, (bit, &net)| {
            word | (u8::from(self.bit(net, values)) << bit)
        })
    }
    fn step(&mut self, inputs: &[(&str, u8)], clock: bool) {
        let values = self.values(inputs);
        let mut ff = self.ff.clone();
        let mut memory = self.mem.clone();
        for (i, cell) in self.gate.ffs.iter().enumerate() {
            assert_eq!(cell.clock_edge, ClockEdge::Posedge);
            let reset = cell.reset.as_ref().is_some_and(|r| {
                self.bit(r.net, &values) == (r.polarity == ResetPolarity::ActiveHigh)
            });
            if reset && (clock || !cell.reset.as_ref().unwrap().sync) {
                ff[i] = cell.reset_value;
            } else if clock {
                ff[i] = self.bit(cell.d, &values);
            }
        }
        if clock {
            for (i, ram) in self.gate.ram_blocks.iter().enumerate() {
                assert_eq!(ram.clock_edge, ClockEdge::Posedge);
                for port in &ram.write_ports {
                    if self.bit(port.enable, &values) {
                        let addr = self.word(&port.addr, &values) as usize;
                        let data = self.word(&port.data, &values);
                        let mask = port.mask.as_ref().map_or(3, |m| self.word(m, &values));
                        memory[i][addr] = (memory[i][addr] & !mask) | (data & mask);
                    }
                }
            }
        }
        self.ff = ff;
        self.mem = memory;
    }
    fn check(&self, inputs: &[(&str, u8)], expected: [u8; 3], label: &str) {
        let values = self.values(inputs);
        for (name, want) in ["result", "q", "previous"].into_iter().zip(expected) {
            assert_eq!(
                self.word(&self.ports[name], &values),
                want,
                "synthesizer sequential output mismatch: {label} {name}"
            );
        }
    }
}
fn compare() {
    let mut steps = 0;
    for reset in [
        "reset_async_low",
        "reset_async_high",
        "reset_sync_low",
        "reset_sync_high",
    ] {
        for registered in [false, true] {
            let gate = analyze(reset, registered);
            let active = u8::from(reset.ends_with("high"));
            for seed in 0..16 {
                let mut machine = Machine::new(&gate, seed);
                let mut mem = [seed & 3, (seed >> 2) & 3];
                let mut q = 1;
                let mut prev = 0;
                let mut sampled = mem[0];
                let mut inputs = [
                    ("clk", 1),
                    ("rst", active),
                    ("we", 0),
                    ("waddr", 0),
                    ("raddr", 0),
                    ("data", 0),
                    ("mask", 0),
                ];
                machine.step(&inputs, true);
                machine.check(&inputs, [mem[0], q, prev], "initial reset");
                inputs[1].1 = 1 - active;
                for code in 0..128 {
                    let data = ((code >> 3) & 3) as u8;
                    let mask = ((code >> 5) & 3) as u8;
                    let wa = (code >> 1) & 1;
                    let ra = (code >> 2) & 1;
                    let we = code & 1 != 0;
                    inputs[2].1 = u8::from(we);
                    inputs[3].1 = wa as u8;
                    inputs[4].1 = ra as u8;
                    inputs[5].1 = data;
                    inputs[6].1 = mask;
                    machine.check(
                        &inputs,
                        [if registered { sampled } else { mem[ra] }, q, prev],
                        "before clock",
                    );
                    let old = q;
                    sampled = mem[ra];
                    if we {
                        q ^= data;
                        prev = old;
                        mem[wa] = (mem[wa] & !mask) | (data & mask);
                    }
                    machine.step(&inputs, true);
                    steps += 1;
                    machine.check(&inputs,[if registered{sampled}else{mem[ra]},q,prev],&format!("after clock reset={reset} registered={registered} seed={seed} code={code}"));
                    if code % 32 == 31 {
                        inputs[1].1 = active;
                        machine.step(&inputs, false);
                        if reset.contains("async") {
                            q = 1;
                            prev = 0;
                        }
                        machine.check(
                            &inputs,
                            [if registered { sampled } else { mem[ra] }, q, prev],
                            "assertion without clock",
                        );
                        sampled = mem[ra];
                        q = 1;
                        prev = 0;
                        if we {
                            mem[wa] = (mem[wa] & !mask) | (data & mask);
                        }
                        machine.step(&inputs, true);
                        machine.check(
                            &inputs,
                            [if registered { sampled } else { mem[ra] }, q, prev],
                            "held reset RAM persists",
                        );
                        inputs[1].1 = 1 - active;
                    }
                }
            }
        }
    }
    assert_eq!(steps, 16384, "sequential synthesis completion");
    eprintln!("sequential synthesis complete steps={steps}");
}
#[test]
fn sequential_techmap() {
    compare();
}

#[test]
fn sequential_model_rejects_miswired_clocks() {
    let gate = analyze("reset_async_low", false);
    let wrong_clock = gate
        .ports
        .iter()
        .find(|p| resource_table::get_str_value(p.name).unwrap() == "we")
        .unwrap()
        .nets[0];
    for ram in [false, true] {
        let mut broken = gate.clone();
        if ram {
            broken.ram_blocks[0].clock = wrong_clock;
        } else {
            broken.ffs[0].clock = wrong_clock;
        }
        assert!(std::panic::catch_unwind(|| Machine::new(&broken, 0)).is_err());
    }
}
