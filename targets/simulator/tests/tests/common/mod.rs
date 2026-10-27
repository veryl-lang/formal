use veryl_simulator::ir::Ir;

pub fn assert_cc_comb_ready(ir: &Ir) {
    assert!(
        ir.dataflow.as_ref().is_some_and(|df| {
            !df.jit_only && !df.plan.cells.is_empty() && df.cell.get().is_some()
        }),
        "AOT-C combinational route absent or not loaded"
    );
}
