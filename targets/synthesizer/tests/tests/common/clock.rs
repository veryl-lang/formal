use veryl_synthesizer::ir::GateModule;

/// The caller supplies only posedges of this fixture's single clock
/// (IEEE 1800-2023 9.4.2); validate connections before advancing all cells.
pub fn assert_single_clock(module: &GateModule, port: &str, message: &str) {
    let clock = &module
        .ports
        .iter()
        .find(|p| veryl_parser::resource_table::get_str_value(p.name).unwrap() == port)
        .expect("missing clock port")
        .nets;
    assert_eq!(clock.len(), 1, "{message}: clock port width");
    for (net, edge) in module
        .ffs
        .iter()
        .map(|f| (f.clock, f.clock_edge))
        .chain(module.ram_blocks.iter().map(|r| (r.clock, r.clock_edge)))
    {
        assert_eq!(net, clock[0], "{message}: clock wiring");
        assert_eq!(
            edge,
            veryl_synthesizer::ir::ClockEdge::Posedge,
            "{message}: clock edge"
        );
    }
}
