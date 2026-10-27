//! Finite source-to-FF comparisons, distinct from symbolic kernel proofs.
//!
//! Expected values come from independent bit/state models, not another backend.
//! Compiled routes require actual dispatch where the fixture supports it; selected
//! wide four-state array stores explicitly permit interpreter fallback. A backend
//! configuration alone does not establish compiled execution. Individual fixture
//! modules define widths, histories, placement, and selector domains. No arbitrary
//! source program, scheduling history, or complete backend correctness is implied.

use std::fs;
use std::path::Path;
use veryl_formal_runner::{Contract, Mutation};

pub static NATIVE: [Contract; 118] = [
    gate_extension_contract(
        "gate-max-interpreter",
        "gate_cycle::interpreter_gate_max",
        false,
    ),
    gate_extension_contract(
        "gate-max-cranelift",
        "gate_cycle::cranelift_gate_max",
        false,
    ),
    gate_extension_contract("gate-max-aot-c", "gate_cycle::aot_c_gate_max", false),
    gate_extension_contract(
        "external-wide-interpreter",
        "external_wide::interpreter_external_wide",
        true,
    ),
    gate_extension_contract(
        "external-wide-cranelift",
        "external_wide::cranelift_external_wide",
        true,
    ),
    gate_extension_contract(
        "external-wide-aot-c",
        "external_wide::aot_c_external_wide",
        true,
    ),
    event_contract("event-interpreter", "events::interpreter_event_state"),
    event_contract("event-cranelift", "events::cranelift_event_state"),
    event_contract("event-aot-c", "events::aot_c_event_state"),
    contract("ff-interpreter", "interpreter_ff_pipeline"),
    contract("ff-cranelift", "cranelift_ff_pipeline"),
    contract("ff-aot-c", "aot_c_ff_pipeline"),
    refinement_contract(
        "refine-interpreter",
        "refinement::interpreter_ff_refinement",
    ),
    refinement_contract("refine-cranelift", "refinement::cranelift_ff_refinement"),
    refinement_contract("refine-aot-c", "refinement::aot_c_ff_refinement"),
    idle_contract("idle-interpreter", "idle::interpreter_idle_gates"),
    idle_contract("idle-cranelift", "idle::cranelift_idle_gates"),
    idle_contract("idle-aot-c", "idle::aot_c_idle_gates"),
    four_state_contract(
        "four-state-interpreter",
        "four_state::interpreter_four_state_ff",
    ),
    four_state_contract(
        "four-state-cranelift",
        "four_state::cranelift_four_state_ff",
    ),
    wide_state_contract(
        "wide-four-interpreter",
        "four_state::interpreter_wide_four_state_ff",
        false,
    ),
    wide_state_contract(
        "wide-four-cranelift",
        "four_state::cranelift_wide_four_state_ff",
        false,
    ),
    wide_state_contract(
        "partial-four-interpreter",
        "four_state_partial::interpreter_four_state_partial_ff",
        true,
    ),
    wide_state_contract(
        "partial-four-cranelift",
        "four_state_partial::cranelift_four_state_partial_ff",
        true,
    ),
    flat_contract(
        "flat-four-interpreter",
        "four_state::interpreter_flat_four_state_ff",
        false,
    ),
    flat_contract(
        "flat-four-cranelift",
        "four_state::cranelift_flat_four_state_ff",
        false,
    ),
    flat_contract(
        "array-four-interpreter-static",
        "four_state_array::interpreter_static_four_state_array",
        true,
    ),
    flat_contract(
        "array-four-cranelift-static",
        "four_state_array::cranelift_static_four_state_array",
        true,
    ),
    flat_contract(
        "array-four-interpreter-dynamic",
        "four_state_array::interpreter_dynamic_four_state_array",
        true,
    ),
    flat_contract(
        "array-four-cranelift-fallback",
        "four_state_array::cranelift_fallback_dynamic_four_state_array",
        true,
    ),
    comb_array_contract(
        "comb-array-interpreter",
        "comb_array::interpreter_comb_array",
    ),
    comb_array_contract(
        "comb-array-cranelift-reader",
        "comb_array::cranelift_fallback_comb_array",
    ),
    gate_cycle_contract(
        "gate-cycle-interpreter",
        "gate_cycle::interpreter_gate_cycle",
    ),
    gate_cycle_contract("gate-cycle-cranelift", "gate_cycle::cranelift_gate_cycle"),
    gate_cycle_contract("gate-cycle-aot-c", "gate_cycle::aot_c_gate_cycle"),
    external_contract(
        "external-interpreter",
        "external_write::interpreter_external_write",
    ),
    external_contract(
        "external-cranelift",
        "external_write::cranelift_external_write",
    ),
    external_contract("external-aot-c", "external_write::aot_c_external_write"),
    unknown_write_contract(
        "unknown-write-interpreter",
        "unknown_write::interpreter_unknown_write",
    ),
    unknown_write_contract(
        "unknown-write-cranelift",
        "unknown_write::cranelift_unknown_write",
    ),
    unknown_write_contract(
        "unknown-write-fallback",
        "unknown_write::fallback_unknown_write",
    ),
    signed_array_contract(
        "signed-interpreter",
        "signed_array::interpreter_signed_array",
    ),
    signed_array_contract("signed-cranelift", "signed_array::cranelift_signed_array"),
    signed_array_contract("signed-aot-c", "signed_array::aot_c_signed_array"),
    signed_array_contract(
        "signed-four-interpreter",
        "signed_array::interpreter_four_signed_array",
    ),
    signed_array_contract(
        "signed-four-cranelift",
        "signed_array::cranelift_four_signed_array",
    ),
    signed_array_contract(
        "signed-four-fallback",
        "signed_array::fallback_four_signed_array",
    ),
    multidim_array_contract(
        "multi-interpreter",
        "multidim_array::interpreter_multidim_array",
    ),
    multidim_array_contract(
        "multi-cranelift",
        "multidim_array::cranelift_multidim_array",
    ),
    multidim_array_contract("multi-aot-c", "multidim_array::aot_c_multidim_array"),
    multidim_array_contract(
        "multi-four-interpreter",
        "multidim_array::interpreter_four_multidim_array",
    ),
    multidim_array_contract(
        "multi-four-cranelift",
        "multidim_array::cranelift_four_multidim_array",
    ),
    multidim_array_contract(
        "multi-four-fallback",
        "multidim_array::fallback_four_multidim_array",
    ),
    unknown_select_contract(
        "select-scalar-interpreter",
        "unknown_select::interpreter_scalar_unknown_select",
    ),
    unknown_select_contract(
        "select-scalar-cranelift",
        "unknown_select::cranelift_scalar_unknown_select",
    ),
    unknown_select_contract(
        "select-scalar-fallback",
        "unknown_select::fallback_scalar_unknown_select",
    ),
    unknown_select_contract(
        "select-array-interpreter",
        "unknown_select::interpreter_array_unknown_select",
    ),
    unknown_select_contract(
        "select-array-cranelift",
        "unknown_select::cranelift_array_unknown_select",
    ),
    unknown_select_contract(
        "select-array-fallback",
        "unknown_select::fallback_array_unknown_select",
    ),
    select_forms_contract(
        "forms-scalar-interpreter",
        "select_forms::interpreter_scalar_select_forms",
    ),
    select_forms_contract(
        "forms-scalar-cranelift",
        "select_forms::cranelift_scalar_select_forms",
    ),
    select_forms_contract(
        "forms-scalar-fallback",
        "select_forms::fallback_scalar_select_forms",
    ),
    select_forms_contract(
        "forms-array-interpreter",
        "select_forms::interpreter_array_select_forms",
    ),
    select_forms_contract(
        "forms-array-cranelift",
        "select_forms::cranelift_array_select_forms",
    ),
    select_forms_contract(
        "forms-array-fallback",
        "select_forms::fallback_array_select_forms",
    ),
    signed_select_contract(
        "signedsel-scalar-interpreter",
        "signed_select::interpreter_scalar_signed_select",
    ),
    signed_select_contract(
        "signedsel-scalar-cranelift",
        "signed_select::cranelift_scalar_signed_select",
    ),
    signed_select_contract(
        "signedsel-scalar-wide-cranelift",
        "signed_select::cranelift_wide_scalar_signed_select",
    ),
    signed_select_contract(
        "signedsel-array-interpreter",
        "signed_select::interpreter_array_signed_select",
    ),
    signed_select_contract(
        "signedsel-array-cranelift",
        "signed_select::cranelift_array_signed_select",
    ),
    signed_select_contract(
        "signedsel-array-fallback",
        "signed_select::fallback_array_signed_select",
    ),
    wide_select_contract(
        "wideidx-scalar-interpreter",
        "wide_select::interpreter_scalar_wide_select",
    ),
    wide_select_contract(
        "wideidx-scalar-cranelift",
        "wide_select::cranelift_scalar_wide_select",
    ),
    wide_select_contract(
        "wideidx-scalar-fallback",
        "wide_select::fallback_scalar_wide_select",
    ),
    wide_select_contract(
        "wideidx-array-interpreter",
        "wide_select::interpreter_array_wide_select",
    ),
    wide_select_contract(
        "wideidx-array-cranelift",
        "wide_select::cranelift_array_wide_select",
    ),
    wide_select_contract(
        "wideidx-array-fallback",
        "wide_select::fallback_array_wide_select",
    ),
    packed_read_contract(
        "read-scalar-interpreter",
        "packed_read::interpreter_scalar_packed_read",
    ),
    packed_read_contract(
        "read-scalar-cranelift",
        "packed_read::cranelift_scalar_packed_read",
    ),
    packed_read_contract(
        "read-array-interpreter",
        "packed_read::interpreter_array_packed_read",
    ),
    packed_read_contract(
        "read-array-cranelift",
        "packed_read::cranelift_array_packed_read",
    ),
    packed_read_contract(
        "read-array-fallback",
        "packed_read::fallback_array_packed_read",
    ),
    wide_packed_read_contract(
        "wide-read-scalar-interpreter",
        "packed_read_wide::interpreter_scalar_wide_packed_read",
    ),
    wide_packed_read_contract(
        "wide-read-scalar-cranelift",
        "packed_read_wide::cranelift_scalar_wide_packed_read",
    ),
    wide_packed_read_contract(
        "wide-read-array-interpreter",
        "packed_read_wide::interpreter_array_wide_packed_read",
    ),
    wide_packed_read_contract(
        "wide-read-array-cranelift",
        "packed_read_wide::cranelift_array_wide_packed_read",
    ),
    wide_packed_read_contract(
        "wide-read-array-fallback",
        "packed_read_wide::fallback_array_wide_packed_read",
    ),
    constant_select_contract(
        "const-read-interpreter",
        "constant_select::interpreter_constant_read",
        false,
    ),
    constant_select_contract(
        "const-read-cranelift",
        "constant_select::cranelift_constant_read",
        false,
    ),
    constant_select_contract(
        "const-write-interpreter",
        "constant_select::interpreter_constant_write",
        false,
    ),
    constant_select_contract(
        "const-write-cranelift",
        "constant_select::cranelift_constant_write",
        false,
    ),
    constant_select_contract(
        "const-write-wide-cranelift",
        "constant_select::cranelift_wide_constant_write",
        false,
    ),
    constant_select_contract(
        "const-write-wide-fallback",
        "constant_select::fallback_wide_constant_write",
        false,
    ),
    constant_select_contract(
        "const-select-diagnostics",
        "constant_select::constant_select_diagnostics",
        true,
    ),
    constant_edge_contract(
        "edge-read-interpreter",
        "constant_edges::interpreter_constant_edge_read",
        false,
    ),
    constant_edge_contract(
        "edge-read-cranelift",
        "constant_edges::cranelift_constant_edge_read",
        false,
    ),
    constant_edge_contract(
        "edge-write-interpreter",
        "constant_edges::interpreter_constant_edge_write",
        false,
    ),
    constant_edge_contract(
        "edge-write-cranelift",
        "constant_edges::cranelift_constant_edge_write",
        false,
    ),
    constant_edge_contract(
        "edge-write-fallback",
        "constant_edges::fallback_constant_edge_write",
        false,
    ),
    constant_edge_contract(
        "edge-negative-diagnostics",
        "constant_edges::negative_constant_edge_diagnostics",
        true,
    ),
    packed_dimension_contract(
        "packdim-read-interpreter",
        "packed_dimensions::interpreter_packed_dimension_read",
    ),
    packed_dimension_contract(
        "packdim-read-cranelift",
        "packed_dimensions::cranelift_packed_dimension_read",
    ),
    packed_dimension_contract(
        "packdim-write-interpreter",
        "packed_dimensions::interpreter_packed_dimension_write",
    ),
    packed_dimension_contract(
        "packdim-write-cranelift",
        "packed_dimensions::cranelift_packed_dimension_write",
    ),
    packed_dimension_contract(
        "packdim-write-wide-two",
        "packed_dimensions::cranelift_wide_packed_dimension_write",
    ),
    packed_dimension_contract(
        "packdim-write-wide-fallback",
        "packed_dimensions::fallback_packed_dimension_write",
    ),
    packed_range_contract(
        "packrange-read-interpreter",
        "packed_ranges::interpreter_packed_range_read",
    ),
    packed_range_contract(
        "packrange-read-cranelift",
        "packed_ranges::cranelift_packed_range_read",
    ),
    packed_range_contract(
        "packrange-write-interpreter",
        "packed_ranges::interpreter_packed_range_write",
    ),
    packed_range_contract(
        "packrange-write-cranelift",
        "packed_ranges::cranelift_packed_range_write",
    ),
    packed_range_contract(
        "packrange-write-wide-two",
        "packed_ranges::cranelift_wide_packed_range_write",
    ),
    packed_range_contract(
        "packrange-write-wide-fallback",
        "packed_ranges::fallback_packed_range_write",
    ),
    packed_stride_contract(
        "packstride-read-interpreter",
        "packed_stride_ranges::interpreter_packed_stride_read",
    ),
    packed_stride_contract(
        "packstride-read-cranelift",
        "packed_stride_ranges::cranelift_packed_stride_read",
    ),
    packed_stride_contract(
        "packstride-write-interpreter",
        "packed_stride_ranges::interpreter_packed_stride_write",
    ),
    packed_stride_contract(
        "packstride-write-cranelift",
        "packed_stride_ranges::cranelift_packed_stride_write",
    ),
    packed_stride_contract(
        "packstride-write-wide-two",
        "packed_stride_ranges::cranelift_wide_packed_stride_write",
    ),
    packed_stride_contract(
        "packstride-write-wide-four-fallback",
        "packed_stride_ranges::fallback_packed_stride_write",
    ),
];
const fn contract(name: &'static str, harness: &'static str) -> Contract {
    Contract {
        name,
        group: "ff-pipeline",
        harness,
        message: "simulator FF pipeline semantics mismatch",
        covers: &[],
    }
}
const fn refinement_contract(name: &'static str, harness: &'static str) -> Contract {
    Contract {
        name,
        group: "ff-refinement",
        harness,
        message: "simulator FF pipeline semantics mismatch",
        covers: &[],
    }
}
const fn idle_contract(name: &'static str, harness: &'static str) -> Contract {
    Contract {
        name,
        group: "event-gate",
        harness,
        message: "simulator FF pipeline semantics mismatch",
        covers: &[],
    }
}
const fn four_state_contract(name: &'static str, harness: &'static str) -> Contract {
    Contract {
        name,
        group: "four-state",
        harness,
        message: "simulator four-state FF semantics mismatch",
        covers: &[],
    }
}
const fn wide_state_contract(name: &'static str, harness: &'static str, partial: bool) -> Contract {
    Contract {
        name,
        group: if partial {
            "four-state-partial"
        } else {
            "wide-four-state"
        },
        harness,
        message: if partial {
            "simulator four-state partial FF semantics mismatch"
        } else {
            "simulator four-state FF semantics mismatch"
        },
        covers: &[],
    }
}
const fn flat_contract(name: &'static str, harness: &'static str, array: bool) -> Contract {
    Contract {
        name,
        group: if array {
            "four-state-array"
        } else {
            "flat-four-state"
        },
        harness,
        message: if array {
            "simulator four-state array semantics mismatch"
        } else {
            "simulator four-state FF semantics mismatch"
        },
        covers: &[],
    }
}
const fn comb_array_contract(name: &'static str, harness: &'static str) -> Contract {
    Contract {
        name,
        group: "comb-four-state-array",
        harness,
        message: "simulator comb-array semantics mismatch",
        covers: &[],
    }
}
const fn gate_cycle_contract(name: &'static str, harness: &'static str) -> Contract {
    Contract {
        name,
        group: "event-gate-cycle",
        harness,
        message: "simulator gate-cycle",
        covers: &[],
    }
}
const fn external_contract(name: &'static str, harness: &'static str) -> Contract {
    Contract {
        name,
        group: "event-gate-external-write",
        harness,
        message: "simulator external-write output mismatch",
        covers: &[],
    }
}
const fn unknown_write_contract(name: &'static str, harness: &'static str) -> Contract {
    Contract {
        name,
        group: "unknown-array-write",
        harness,
        message: "simulator four-state array semantics mismatch",
        covers: &[],
    }
}
const fn signed_array_contract(name: &'static str, harness: &'static str) -> Contract {
    Contract {
        name,
        group: "signed-array-index",
        harness,
        message: "simulator signed-array semantics mismatch",
        covers: &[],
    }
}
const fn multidim_array_contract(name: &'static str, harness: &'static str) -> Contract {
    Contract {
        name,
        group: "multidim-array-index",
        harness,
        message: "simulator multidim-array semantics mismatch",
        covers: &[],
    }
}
const fn unknown_select_contract(name: &'static str, harness: &'static str) -> Contract {
    Contract {
        name,
        group: "unknown-packed-select",
        harness,
        message: "simulator unknown-select output mismatch",
        covers: &[],
    }
}
const fn select_forms_contract(name: &'static str, harness: &'static str) -> Contract {
    Contract {
        name,
        group: "packed-select-forms",
        harness,
        message: "simulator select-forms output mismatch",
        covers: &[],
    }
}
const fn signed_select_contract(name: &'static str, harness: &'static str) -> Contract {
    Contract {
        name,
        group: "signed-packed-select",
        harness,
        message: "simulator signed-select output mismatch",
        covers: &[],
    }
}
const fn wide_select_contract(name: &'static str, harness: &'static str) -> Contract {
    Contract {
        name,
        group: "wide-packed-index",
        harness,
        message: "simulator wide-select output mismatch",
        covers: &[],
    }
}
const fn packed_read_contract(name: &'static str, harness: &'static str) -> Contract {
    Contract {
        name,
        group: "packed-read-boundary",
        harness,
        message: "simulator packed-read output mismatch",
        covers: &[],
    }
}
const fn wide_packed_read_contract(name: &'static str, harness: &'static str) -> Contract {
    Contract {
        name,
        group: "wide-packed-read",
        harness,
        message: "simulator packed-read output mismatch",
        covers: &[],
    }
}
const fn constant_select_contract(
    name: &'static str,
    harness: &'static str,
    diagnostic: bool,
) -> Contract {
    Contract {
        name,
        group: if diagnostic {
            "constant-select-diagnostics"
        } else {
            "constant-packed-select"
        },
        harness,
        message: if diagnostic {
            "simulator constant-select diagnostic mismatch"
        } else {
            "simulator constant-select output mismatch"
        },
        covers: &[],
    }
}
const fn constant_edge_contract(
    name: &'static str,
    harness: &'static str,
    diagnostic: bool,
) -> Contract {
    Contract {
        name,
        group: if diagnostic {
            "constant-edge-diagnostics"
        } else {
            "constant-packed-edge"
        },
        harness,
        message: if diagnostic {
            "simulator constant-edge diagnostic mismatch"
        } else {
            "simulator constant-edge output mismatch"
        },
        covers: &[],
    }
}
const fn packed_dimension_contract(name: &'static str, harness: &'static str) -> Contract {
    Contract {
        name,
        group: "packed-dimension",
        harness,
        message: "simulator packed-dimension output mismatch",
        covers: &[],
    }
}
const fn packed_range_contract(name: &'static str, harness: &'static str) -> Contract {
    Contract {
        name,
        group: "packed-range",
        harness,
        message: "simulator packed-range output mismatch",
        covers: &[],
    }
}
const fn packed_stride_contract(name: &'static str, harness: &'static str) -> Contract {
    Contract {
        name,
        group: "packed-stride-range",
        harness,
        message: "simulator packed-range output mismatch",
        covers: &[],
    }
}
pub fn catalog() -> &'static [Contract] {
    &NATIVE
}
pub fn mutations(repo: &Path, selected: &[&'static Contract]) -> Vec<Mutation> {
    let simulator_path = "veryl/crates/simulator/src/simulator.rs";
    let simulator = fs::read_to_string(repo.join(simulator_path)).unwrap();
    let log_path = "veryl/crates/simulator/src/ir/write_log.rs";
    let log = fs::read_to_string(repo.join(log_path)).unwrap();
    let mut result = Vec::new();
    for contract in selected.iter().filter(|c| c.group == "ff-pipeline") {
        for (suffix, path, original, function, old, new) in [
            (
                "empty-before-commit",
                simulator_path,
                &simulator,
                "fn commit_event_log(&mut self)",
                "self.ir.dataflow_any.set(true);",
                "self.ir.dataflow_any.set(true); self.ir.write_log_buffer.reset();",
            ),
            (
                "retain-live-log",
                simulator_path,
                &simulator,
                "fn commit_event_log(&mut self)",
                "self.ir.write_log_buffer.reset();",
                "/* retain live log */",
            ),
            (
                "narrow-payload",
                log_path,
                &log,
                "fn commit_from_log_impl<const MARKING: bool>(",
                "*p = entry.payload as u8;",
                "*p = (entry.payload as u8) ^ 1;",
            ),
            (
                "wide-payload",
                log_path,
                &log,
                "fn commit_from_log_impl<const MARKING: bool>(",
                "std::ptr::copy_nonoverlapping(s, p, nb);",
                "std::ptr::copy_nonoverlapping(s, p, nb); *p ^= 1;",
            ),
        ] {
            let start = original
                .find(function)
                .expect("pipeline mutation function moved");
            let closing = if path == simulator_path {
                "\n    }"
            } else {
                "\n}"
            };
            let end = start
                + original[start..]
                    .find(closing)
                    .expect("pipeline mutation closing brace moved")
                + closing.len();
            let body = &original[start..end];
            assert_eq!(
                body.matches(old).count(),
                1,
                "pipeline mutation site changed: {old}"
            );
            let name = match (contract.name, suffix) {
                ("ff-interpreter", "empty-before-commit") => "ff-interpreter-empty-before-commit",
                ("ff-interpreter", "retain-live-log") => "ff-interpreter-retain-live-log",
                ("ff-interpreter", "narrow-payload") => "ff-interpreter-narrow-payload",
                ("ff-interpreter", "wide-payload") => "ff-interpreter-wide-payload",
                ("ff-cranelift", "empty-before-commit") => "ff-cranelift-empty-before-commit",
                ("ff-cranelift", "retain-live-log") => "ff-cranelift-retain-live-log",
                ("ff-cranelift", "narrow-payload") => "ff-cranelift-narrow-payload",
                ("ff-cranelift", "wide-payload") => "ff-cranelift-wide-payload",
                ("ff-aot-c", "empty-before-commit") => "ff-aot-c-empty-before-commit",
                ("ff-aot-c", "retain-live-log") => "ff-aot-c-retain-live-log",
                ("ff-aot-c", "narrow-payload") => "ff-aot-c-narrow-payload",
                ("ff-aot-c", "wide-payload") => "ff-aot-c-wide-payload",
                _ => unreachable!(),
            };
            result.push(Mutation {
                name,
                contract,
                path,
                source: format!(
                    "{}{}{}",
                    &original[..start],
                    body.replace(old, new),
                    &original[end..]
                ),
            });
        }
    }
    let classify_path = "veryl/crates/analyzer/src/ir/ff_table.rs";
    let original = fs::read_to_string(repo.join(classify_path)).unwrap();
    let start = original
        .find("fn classify<'a>(")
        .expect("classification function moved");
    let end = start
        + original[start..]
            .find("\n}")
            .expect("classification closing brace moved")
        + 2;
    let body = &original[start..end];
    for contract in selected.iter().filter(|c| c.group == "ff-refinement") {
        for (cross_block, old, new) in [
            (
                true,
                "if *decl != assigned_decl {\n            return true;\n        }",
                "if *decl != assigned_decl {return false;}",
            ),
            (
                false,
                "Some(idx) if Some(*idx) == index => !readable,",
                "Some(idx) if Some(*idx) == index => false,",
            ),
        ] {
            assert_eq!(
                body.matches(old).count(),
                1,
                "classification mutation site changed: {old}"
            );
            let name = match (contract.name, cross_block) {
                ("refine-interpreter", true) => "refine-interpreter-cross-block",
                ("refine-interpreter", false) => "refine-interpreter-repeated-write",
                ("refine-cranelift", true) => "refine-cranelift-cross-block",
                ("refine-cranelift", false) => "refine-cranelift-repeated-write",
                ("refine-aot-c", true) => "refine-aot-c-cross-block",
                ("refine-aot-c", false) => "refine-aot-c-repeated-write",
                _ => unreachable!(),
            };
            result.push(Mutation {
                name,
                contract,
                path: classify_path,
                source: format!(
                    "{}{}{}",
                    &original[..start],
                    body.replace(old, new),
                    &original[end..]
                ),
            });
        }
    }
    for contract in selected.iter().filter(|c| c.group == "event-gate") {
        let c = contract.name == "idle-aot-c";
        let path = if c {
            "veryl/crates/simulator/src/backend/aot_c/emit.rs"
        } else {
            "veryl/crates/simulator/src/ir/opt/event_gate.rs"
        };
        let original = fs::read_to_string(repo.join(path)).unwrap();
        for wake in [true, false] {
            let (declaration, closing, old, new, count) = match (c, wake) {
                (false, true) => (
                    "pub unsafe fn enter(",
                    "\n    }",
                    "                        run = true;",
                    "                        run = false;",
                    1,
                ),
                (false, false) => (
                    "pub unsafe fn exit(",
                    "\n    }",
                    "log_unchanged(ff, log, n0, w0) && self.out_comb_held(comb)",
                    "true",
                    1,
                ),
                (true, true) => (
                    "fn apply_event_gates(",
                    "\n}",
                    "eg_run = 1; }}",
                    "eg_run = 0; }}",
                    2,
                ),
                (true, false) => (
                    "fn apply_event_gates(",
                    "\n}",
                    "int eg_idle = (veryl_evg_unchanged(ff_values, write_log, eg_n0, eg_w0){ocmp});",
                    "int eg_idle = 1;",
                    1,
                ),
            };
            let start = original
                .find(declaration)
                .expect("idle gate mutation function moved");
            let end = start
                + original[start..]
                    .find(closing)
                    .expect("idle gate mutation brace moved")
                + closing.len();
            let body = &original[start..end];
            assert_eq!(
                body.matches(old).count(),
                count,
                "idle mutation site changed: {old}"
            );
            let name = match (contract.name, wake) {
                ("idle-interpreter", true) => "idle-interpreter-misses-wakeup",
                ("idle-interpreter", false) => "idle-interpreter-false-idle",
                ("idle-cranelift", true) => "idle-cranelift-misses-wakeup",
                ("idle-cranelift", false) => "idle-cranelift-false-idle",
                ("idle-aot-c", true) => "idle-aot-c-misses-wakeup",
                ("idle-aot-c", false) => "idle-aot-c-false-idle",
                _ => unreachable!(),
            };
            result.push(Mutation {
                name,
                contract,
                path,
                source: format!(
                    "{}{}{}",
                    &original[..start],
                    body.replace(old, new),
                    &original[end..]
                ),
            });
        }
    }
    for contract in selected.iter().filter(|c| c.group == "four-state") {
        let jit = contract.name == "four-state-cranelift";
        let path = if jit {
            "veryl/crates/simulator/src/backend/cranelift/statement.rs"
        } else {
            "veryl/crates/simulator/src/ir/statement.rs"
        };
        let original = fs::read_to_string(repo.join(path)).unwrap();
        for kind in 0..if jit { 2 } else { 3 } {
            let (declaration, old, new) = match (jit, kind) {
                (false, 0) => (
                    "fn emit_ff_log(",
                    "let payload = value.payload_u64();",
                    "let payload = value.to_u64().unwrap_or(0);",
                ),
                (false, 1) => (
                    "fn emit_ff_log(",
                    "event_write_log_push_static(base_offset + nb_u32, mask, nb_u16);",
                    "event_write_log_push_static(base_offset + nb_u32, 0, nb_u16);",
                ),
                (false, 2) => (
                    "fn value_is_true(",
                    "(x.payload & !x.mask_xz) != 0",
                    "x.payload != 0",
                ),
                (true, 0) => (
                    "fn emit_ff_log_push(",
                    "emit_inline_write_log_push(context, builder, mask_offset_val, m, width_class_val);",
                    "let _ = (mask_offset_val, m);",
                ),
                (true, 1) => (
                    "impl ProtoIfStatement {",
                    "builder.ins().band_not(cond_payload, mask_xz)",
                    "cond_payload",
                ),
                _ => unreachable!(),
            };
            let start = original
                .find(declaration)
                .expect("four-state mutation function moved");
            let end = start
                + original[start..]
                    .find("\n}")
                    .expect("four-state mutation brace moved")
                + 2;
            let body = &original[start..end];
            assert_eq!(
                body.matches(old).count(),
                1,
                "four-state mutation site changed: {old}"
            );
            let name = match (jit, kind) {
                (false, 0) => "four-state-interpreter-numeric-payload-regression",
                (false, 1) => "four-state-interpreter-missing-mask",
                (false, 2) => "four-state-interpreter-unknown-enable",
                (true, 0) => "four-state-cranelift-missing-mask",
                (true, 1) => "four-state-cranelift-unknown-enable",
                _ => unreachable!(),
            };
            result.push(Mutation {
                name,
                contract,
                path,
                source: format!(
                    "{}{}{}",
                    &original[..start],
                    body.replace(old, new),
                    &original[end..]
                ),
            });
        }
    }
    for contract in selected
        .iter()
        .filter(|c| matches!(c.group, "wide-four-state" | "four-state-partial"))
    {
        let jit = contract.name.ends_with("cranelift");
        let partial = contract.group == "four-state-partial";
        let path = if jit {
            "veryl/crates/simulator/src/backend/cranelift/statement.rs"
        } else {
            "veryl/crates/simulator/src/ir/statement.rs"
        };
        let original = fs::read_to_string(repo.join(path)).unwrap();
        for kind in 0..2 {
            let (declaration, old, new, count) = match (jit, partial, kind) {
                (false, false, 0) => (
                    "fn emit_ff_log(",
                    "(payload >> shift) as u64",
                    "payload as u64",
                    1,
                ),
                (false, false, 1) => ("fn emit_ff_log(", "(m >> shift) as u64", "m as u64", 1),
                (true, false, 0) => (
                    "fn emit_ff_log_push(",
                    "builder.ins().ushr_imm_u(payload, (i * 64) as i64)",
                    "builder.ins().ushr_imm_u(payload, 0)",
                    1,
                ),
                (true, false, 1) => (
                    "fn emit_ff_log_push(",
                    "let off = log_current_offset + nb_i32 + (i * 8) as i32;",
                    "let off = log_current_offset + nb_i32;",
                    1,
                ),
                (false, true, 0) => (
                    "",
                    "current.assign(value, beg, end);",
                    "current.assign(value, beg, end.saturating_sub(1));",
                    4,
                ),
                (false, true, 1) => (
                    "fn emit_ff_log(",
                    "base_offset + nb_u32 + start as u32",
                    "base_offset + nb_u32",
                    1,
                ),
                (true, true, 0) => (
                    "impl ProtoAssignStatement {",
                    "let org_m = builder.ins().band(org_m, not_mask);",
                    "let org_m = if wide {context.zero_128} else {context.zero};",
                    2,
                ),
                (true, true, 1) => (
                    "impl ProtoAssignStatement {",
                    "let dyn_mask = mask_out_of_range(context, builder, in_range, dyn_mask, wide);",
                    "let _ = in_range;",
                    1,
                ),
                _ => unreachable!(),
            };
            let (start, end) = if declaration.is_empty() {
                (0, original.len())
            } else {
                let start = original
                    .find(declaration)
                    .expect("wide four-state mutation function moved");
                let end = start
                    + original[start..]
                        .find("\n}")
                        .expect("wide four-state mutation brace moved")
                    + 2;
                (start, end)
            };
            let body = &original[start..end];
            assert_eq!(
                body.matches(old).count(),
                count,
                "wide four-state mutation site changed: {old}"
            );
            let name = match (jit, partial, kind) {
                (false, false, 0) => "wide-four-interpreter-repeats-payload-word",
                (false, false, 1) => "wide-four-interpreter-repeats-mask-word",
                (true, false, 0) => "wide-four-cranelift-repeats-payload-word",
                (true, false, 1) => "wide-four-cranelift-reuses-mask-offset",
                (false, true, 0) => "partial-four-interpreter-wrong-low-bit",
                (false, true, 1) => "partial-four-interpreter-wrong-mask-span",
                (true, true, 0) => "partial-four-cranelift-loses-outside-mask",
                (true, true, 1) => "partial-four-cranelift-ignores-outside-index",
                _ => unreachable!(),
            };
            result.push(Mutation {
                name,
                contract,
                path,
                source: format!(
                    "{}{}{}",
                    &original[..start],
                    body.replace(old, new),
                    &original[end..]
                ),
            });
        }
    }
    for contract in selected.iter().filter(|c| c.group == "flat-four-state") {
        let jit = contract.name == "flat-four-cranelift";
        let path = if jit {
            "veryl/crates/simulator/src/backend/cranelift/expression.rs"
        } else {
            "veryl/crates/simulator/src/ir/statement.rs"
        };
        let original = fs::read_to_string(repo.join(path)).unwrap();
        for kind in 0..if jit { 3 } else { 2 } {
            let (decl, old, new) = match (jit, kind) {
                (false, 0) => (
                    "fn emit_ff_log(",
                    "payload_digits.get(i).copied().unwrap_or(0)",
                    "payload_digits.first().copied().unwrap_or(0)",
                ),
                (false, 1) => (
                    "fn emit_ff_log(",
                    "mask_digits.get(i).copied().unwrap_or(0)",
                    "mask_digits.first().copied().unwrap_or(0)",
                ),
                (true, 0) => (
                    "    fn build_binary_wide_binary(",
                    "Op::BitAnd | Op::BitOr | Op::BitXor | Op::BitXnor",
                    "Op::BitAnd | Op::BitOr",
                ),
                (true, 1) => (
                    "    fn build_binary_wide_unary(",
                    "Op::Add | Op::BitNot => {",
                    "Op::Add => {",
                ),
                (true, 2) => (
                    "    fn build_binary_wide_unary(",
                    "if matches!(op, Op::Sub | Op::BitNot)",
                    "if false",
                ),
                _ => unreachable!(),
            };
            let start = original.find(decl).expect("flat mutation function moved");
            let closing = if jit { "\n    }" } else { "\n}" };
            let end = start
                + original[start..]
                    .find(closing)
                    .expect("flat mutation brace moved")
                + closing.len();
            let body = &original[start..end];
            let source = if jit && kind == 1 {
                assert_eq!(body.matches(old).count(), 1);
                assert_eq!(body.matches("                Op::Sub => {").count(), 1);
                body.replace(old, new).replace(
                    "                Op::Sub => {",
                    "                Op::Sub | Op::BitNot => {",
                )
            } else {
                assert_eq!(
                    body.matches(old).count(),
                    1,
                    "flat mutation site changed: {old}"
                );
                body.replace(old, new)
            };
            let name = match (jit, kind) {
                (false, 0) => "flat-interpreter-repeated-payload-word",
                (false, 1) => "flat-interpreter-repeated-mask-word",
                (true, 0) => "flat-cranelift-xor-payload-regression",
                (true, 1) => "flat-cranelift-not-mask-regression",
                (true, 2) => "flat-cranelift-not-payload-regression",
                _ => unreachable!(),
            };
            result.push(Mutation {
                name,
                contract,
                path,
                source: format!("{}{}{}", &original[..start], source, &original[end..]),
            });
        }
    }
    for contract in selected.iter().filter(|c| c.group == "four-state-array") {
        let dynamic = matches!(
            contract.name,
            "array-four-interpreter-dynamic" | "array-four-cranelift-fallback"
        );
        let path = if dynamic {
            "veryl/crates/simulator/src/ir/statement.rs"
        } else {
            "veryl/crates/simulator/src/ir/write_log.rs"
        };
        let original = fs::read_to_string(repo.join(path)).unwrap();
        for kind in 0..2 {
            let (decl, old, new, count) = match (dynamic, kind) {
                (true, 0) => (
                    "impl AssignDynamicStatement {",
                    "let Some(idx) = idx_val.to_usize() else {",
                    "let Some(idx) = idx_val.to_usize().map(|idx| idx.min(self.dst_num_elements.saturating_sub(1))) else {",
                    1,
                ),
                (true, 1) => (
                    "impl AssignDynamicStatement {",
                    "self.dst_stride * idx as isize",
                    "self.dst_stride * 0",
                    2,
                ),
                (false, 0) => (
                    "fn commit_from_log_impl<const MARKING: bool>(",
                    "std::ptr::copy_nonoverlapping(s, p, nb);",
                    "std::ptr::copy_nonoverlapping(s, p, nb); *p ^= 1;",
                    1,
                ),
                (false, 1) => (
                    "fn commit_from_log_impl<const MARKING: bool>(",
                    "let wide_limit = buffer.wide_count as usize;",
                    "let wide_limit = (buffer.wide_count as usize).saturating_sub(1);",
                    1,
                ),
                _ => unreachable!(),
            };
            let start = original.find(decl).expect("array mutation function moved");
            let end = start
                + original[start..]
                    .find("\n}")
                    .expect("array mutation brace moved")
                + 2;
            let body = &original[start..end];
            assert_eq!(
                body.matches(old).count(),
                count,
                "array mutation site changed: {old}"
            );
            let name = match (contract.name, kind) {
                ("array-four-interpreter-static", 0) => "array-interpreter-static-payload",
                ("array-four-interpreter-static", 1) => "array-interpreter-static-last-record",
                ("array-four-cranelift-static", 0) => "array-cranelift-static-payload",
                ("array-four-cranelift-static", 1) => "array-cranelift-static-last-record",
                ("array-four-interpreter-dynamic", 0) => "array-interpreter-dynamic-clamps-index",
                ("array-four-interpreter-dynamic", 1) => "array-interpreter-dynamic-zero-stride",
                ("array-four-cranelift-fallback", 0) => "array-fallback-dynamic-clamps-index",
                ("array-four-cranelift-fallback", 1) => "array-fallback-dynamic-zero-stride",
                _ => unreachable!(),
            };
            result.push(Mutation {
                name,
                contract,
                path,
                source: format!(
                    "{}{}{}",
                    &original[..start],
                    body.replace(old, new),
                    &original[end..]
                ),
            });
        }
    }
    for contract in selected
        .iter()
        .filter(|c| c.group == "comb-four-state-array")
    {
        let jit = contract.name == "comb-array-cranelift-reader";
        for (compiled, name, path, decl, closing, old, new) in [
            (
                false,
                "comb-array-interpreter-read-stride",
                "veryl/crates/simulator/src/ir/expression.rs",
                "    pub fn eval(&self, mask_cache: &mut MaskCache)",
                "\n    }",
                "(*base_ptr).offset(*stride * idx as isize)",
                "(*base_ptr).offset(0)",
            ),
            (
                false,
                "comb-array-interpreter-clamped-read",
                "veryl/crates/simulator/src/ir/expression.rs",
                "    pub fn eval(&self, mask_cache: &mut MaskCache)",
                "\n    }",
                ".filter(|idx| *idx < *num_elements)",
                ".map(|idx| idx.min(num_elements.saturating_sub(1)))",
            ),
            (
                false,
                "comb-array-interpreter-zero-default",
                "veryl/crates/simulator/src/ir/expression.rs",
                "pub fn out_of_range_read(",
                "\n}",
                "Value::new_x(width, signed)",
                "Value::new(0, width, signed)",
            ),
            (
                true,
                "comb-array-cranelift-unknown-index-regression",
                "veryl/crates/simulator/src/backend/cranelift/expression.rs",
                "    fn build_binary_inner(",
                "\n    }",
                "builder.ins().icmp_imm_s(IntCC::Equal, unknown, 0)",
                "builder.ins().icmp_imm_s(IntCC::UnsignedGreaterThanOrEqual, unknown, 0)",
            ),
            (
                true,
                "comb-array-cranelift-read-stride",
                "veryl/crates/simulator/src/backend/cranelift/expression.rs",
                "    fn build_binary_inner(",
                "\n    }",
                "let byte_offset = builder.ins().imul(clamped, stride_val);",
                "let byte_offset = builder.ins().iconst(I64, 0);",
            ),
            (
                true,
                "comb-array-cranelift-clamped-read",
                "veryl/crates/simulator/src/backend/cranelift/expression.rs",
                "    fn build_binary_inner(",
                "\n    }",
                "builder.ins().select(c, addr, d)",
                "builder.ins().select(c, addr, addr)",
            ),
            (
                true,
                "comb-array-cranelift-zero-default",
                "veryl/crates/simulator/src/backend/cranelift/helpers.rs",
                "pub(crate) fn alloc_wide_out_of_range(",
                "\n}",
                "let ones = builder.ins().iconst(I64, -1);",
                "let ones = builder.ins().iconst(I64, 0);",
            ),
        ] {
            if compiled != jit {
                continue;
            }
            let original = fs::read_to_string(repo.join(path)).unwrap();
            let start = original
                .find(decl)
                .expect("comb array mutation function moved");
            let end = start
                + original[start..]
                    .find(closing)
                    .expect("comb array mutation brace moved")
                + closing.len();
            let body = &original[start..end];
            assert_eq!(
                body.matches(old).count(),
                1,
                "comb array mutation site changed: {old}"
            );
            result.push(Mutation {
                name,
                contract,
                path,
                source: format!(
                    "{}{}{}",
                    &original[..start],
                    body.replace(old, new),
                    &original[end..]
                ),
            });
        }
    }
    for contract in selected.iter().filter(|c| c.group == "event-gate-cycle") {
        let c = contract.name == "gate-cycle-aot-c";
        let original = fs::read_to_string(repo.join(if c {
            "veryl/crates/simulator/src/backend/aot_c/emit.rs"
        } else {
            "veryl/crates/simulator/src/ir/opt/event_gate.rs"
        }))
        .unwrap();
        for (compiled, kind, old, new) in [
            (
                false,
                "off-boundary",
                "if streak >= EVENT_GATE_AUTO_OFF_STREAK {",
                "if streak > EVENT_GATE_AUTO_OFF_STREAK {",
            ),
            (
                false,
                "rearm-boundary",
                "if n >= read_u32(eg.add(GATE_PERIOD)) {",
                "if n > read_u32(eg.add(GATE_PERIOD)) {",
            ),
            (
                false,
                "period-doubling",
                "(off * 2).min(EVENT_GATE_REARM_CAP)",
                "off",
            ),
            (
                false,
                "skip-period-reset",
                "write_u32(eg.add(GATE_PERIOD), 0);",
                "",
            ),
            (
                false,
                "off-runs",
                "return GateEntry::Plain;",
                "return GateEntry::Skip;",
            ),
            (
                true,
                "off-boundary",
                "if (++eg_stk >= {streak}u)",
                "if (++eg_stk > {streak}u)",
            ),
            (
                true,
                "rearm-boundary",
                "if (++eg_n >= eg_b)",
                "if (++eg_n > eg_b)",
            ),
            (
                true,
                "period-doubling",
                "eg_b * 2 : {cap}u",
                "eg_b : {cap}u",
            ),
            (
                true,
                "skip-period-reset",
                "__builtin_memcpy(eg + {per}, &eg_z, 4);",
                "",
            ),
            (
                true,
                "off-runs",
                "{calls}      }} else {{ int eg_run",
                "      }} else {{ int eg_run",
            ),
        ] {
            if compiled != c {
                continue;
            }
            let path = if c {
                "veryl/crates/simulator/src/backend/aot_c/emit.rs"
            } else {
                "veryl/crates/simulator/src/ir/opt/event_gate.rs"
            };
            let decl = if c {
                "fn apply_event_gates("
            } else {
                "pub unsafe fn enter("
            };
            let closing = if c { "\n}" } else { "\n    }" };
            let start = original
                .find(decl)
                .expect("gate-cycle mutation function moved");
            let end = start
                + original[start..]
                    .find(closing)
                    .expect("gate-cycle mutation brace moved")
                + closing.len();
            let body = &original[start..end];
            assert_eq!(
                body.matches(old).count(),
                1,
                "gate-cycle mutation site changed: {old}"
            );
            let name = match (contract.name, kind) {
                ("gate-cycle-interpreter", "off-boundary") => "gate-cycle-interpreter-off-boundary",
                ("gate-cycle-interpreter", "rearm-boundary") => {
                    "gate-cycle-interpreter-rearm-boundary"
                }
                ("gate-cycle-interpreter", "period-doubling") => {
                    "gate-cycle-interpreter-period-doubling"
                }
                ("gate-cycle-interpreter", "skip-period-reset") => {
                    "gate-cycle-interpreter-skip-period-reset"
                }
                ("gate-cycle-interpreter", "off-runs") => "gate-cycle-interpreter-off-runs",
                ("gate-cycle-cranelift", "off-boundary") => "gate-cycle-cranelift-off-boundary",
                ("gate-cycle-cranelift", "rearm-boundary") => "gate-cycle-cranelift-rearm-boundary",
                ("gate-cycle-cranelift", "period-doubling") => {
                    "gate-cycle-cranelift-period-doubling"
                }
                ("gate-cycle-cranelift", "skip-period-reset") => {
                    "gate-cycle-cranelift-skip-period-reset"
                }
                ("gate-cycle-cranelift", "off-runs") => "gate-cycle-cranelift-off-runs",
                ("gate-cycle-aot-c", "off-boundary") => "gate-cycle-aot-c-off-boundary",
                ("gate-cycle-aot-c", "rearm-boundary") => "gate-cycle-aot-c-rearm-boundary",
                ("gate-cycle-aot-c", "period-doubling") => "gate-cycle-aot-c-period-doubling",
                ("gate-cycle-aot-c", "skip-period-reset") => "gate-cycle-aot-c-skip-period-reset",
                ("gate-cycle-aot-c", "off-runs") => "gate-cycle-aot-c-off-runs",
                _ => unreachable!(),
            };
            result.push(Mutation {
                name,
                contract,
                path,
                source: format!(
                    "{}{}{}",
                    &original[..start],
                    body.replace(old, new),
                    &original[end..]
                ),
            });
        }
    }
    for contract in selected
        .iter()
        .filter(|c| c.group == "event-gate-external-write")
    {
        for (raw, function) in [
            (false, "pub fn set_var_by_id("),
            (true, "pub fn mark_ff_written("),
        ] {
            let start = simulator
                .find(function)
                .expect("external-write function moved");
            let end = start
                + simulator[start..]
                    .find("\n    }")
                    .expect("external-write brace moved")
                + "\n    }".len();
            let body = &simulator[start..end];
            let old = "self.invalidate_event_gates();";
            assert_eq!(
                body.matches(old).count(),
                1,
                "external-write invalidation moved"
            );
            let name = match (contract.name, raw) {
                ("external-interpreter", false) => "external-interpreter-set-no-invalidation",
                ("external-interpreter", true) => "external-interpreter-mark-no-invalidation",
                ("external-cranelift", false) => "external-cranelift-set-no-invalidation",
                ("external-cranelift", true) => "external-cranelift-mark-no-invalidation",
                ("external-aot-c", false) => "external-aot-c-set-no-invalidation",
                ("external-aot-c", true) => "external-aot-c-mark-no-invalidation",
                _ => unreachable!(),
            };
            result.push(Mutation {
                name,
                contract,
                path: simulator_path,
                source: format!(
                    "{}{}{}",
                    &simulator[..start],
                    body.replace(old, ""),
                    &simulator[end..]
                ),
            });
        }
    }
    for contract in selected.iter().filter(|c| c.group == "unknown-array-write") {
        let jit = contract.name == "unknown-write-cranelift";
        let path = if jit {
            "veryl/crates/simulator/src/backend/cranelift/statement.rs"
        } else {
            "veryl/crates/simulator/src/ir/statement.rs"
        };
        let original = fs::read_to_string(repo.join(path)).unwrap();
        let decl = if jit {
            "    fn dst_in_range("
        } else {
            "impl AssignDynamicStatement {"
        };
        let closing = if jit { "\n    }" } else { "\n}" };
        let start = original.find(decl).expect("unknown-write function moved");
        let end = start
            + original[start..]
                .find(closing)
                .expect("unknown-write brace moved")
            + closing.len();
        let body = &original[start..end];
        for low_only in [false, true] {
            let (old, new) = match (jit, low_only) {
                (false, false) => (
                    "let Some(idx) = idx_val.to_usize() else {\n            return;\n        };",
                    "let idx = idx_val.to_usize().unwrap_or(0);",
                ),
                (false, true) => (
                    "let Some(idx) = idx_val.to_usize() else {",
                    "let Some(idx) = (if idx_val.mask_xz_u128() & 1 == 0 { Some(idx_val.payload_u64() as usize) } else { None }) else {",
                ),
                (true, false) => (
                    "and_in_range(builder, bounds, known)",
                    "and_in_range(builder, bounds, None)",
                ),
                (true, true) => (
                    "let known = wide_any_xz(",
                    "let idx_mask_xz = idx_mask_xz.map(|mask| builder.ins().band_imm(mask, 1));\n        let known = wide_any_xz(",
                ),
            };
            assert_eq!(
                body.matches(old).count(),
                1,
                "unknown-write mutation site changed"
            );
            let name = match (contract.name, low_only) {
                ("unknown-write-interpreter", false) => {
                    "unknown-write-interpreter-zero-index-regression"
                }
                ("unknown-write-interpreter", true) => "unknown-write-interpreter-low-mask-only",
                ("unknown-write-cranelift", false) => {
                    "unknown-write-cranelift-ignored-mask-regression"
                }
                ("unknown-write-cranelift", true) => "unknown-write-cranelift-low-mask-only",
                ("unknown-write-fallback", false) => "unknown-write-fallback-zero-index-regression",
                ("unknown-write-fallback", true) => "unknown-write-fallback-low-mask-only",
                _ => unreachable!(),
            };
            result.push(Mutation {
                name,
                contract,
                path,
                source: format!(
                    "{}{}{}",
                    &original[..start],
                    body.replace(old, new),
                    &original[end..]
                ),
            });
        }
    }
    for contract in selected.iter().filter(|c| c.group == "signed-array-index") {
        let path = "veryl/crates/simulator/src/ir/expression.rs";
        let original = fs::read_to_string(repo.join(path)).unwrap();
        for kind in 0..3 {
            let (decl, old, new) = match kind {
                0 => (
                    "fn use_site_check_suffices(",
                    "dim_size <= negatives_start",
                    "dim_size <= negatives_start || coordinate.expr_context().signed",
                ),
                1 => (
                    "fn resize_to_width(",
                    "let sign_extend = expr.expr_context().signed;",
                    "let sign_extend = false;",
                ),
                2 => (
                    "pub fn build_linear_index_expr(",
                    "value: Value::new(base as u64, 64, false)",
                    "value: Value::new(base.saturating_sub(1) as u64, 64, false)",
                ),
                _ => unreachable!(),
            };
            let start = original
                .find(decl)
                .expect("signed-array mutation function moved");
            let end = start
                + original[start..]
                    .find("\n}")
                    .expect("signed-array mutation brace moved")
                + 2;
            let body = &original[start..end];
            assert_eq!(
                body.matches(old).count(),
                1,
                "signed-array mutation site changed"
            );
            let name = match (contract.name, kind) {
                ("signed-interpreter", 0) => "signed-interpreter-unsigned-shortcut",
                ("signed-interpreter", 1) => "signed-interpreter-zero-extension",
                ("signed-interpreter", 2) => "signed-interpreter-valid-sentinel",
                ("signed-cranelift", 0) => "signed-cranelift-unsigned-shortcut",
                ("signed-cranelift", 1) => "signed-cranelift-zero-extension",
                ("signed-cranelift", 2) => "signed-cranelift-valid-sentinel",
                ("signed-aot-c", 0) => "signed-aot-c-unsigned-shortcut",
                ("signed-aot-c", 1) => "signed-aot-c-zero-extension",
                ("signed-aot-c", 2) => "signed-aot-c-valid-sentinel",
                ("signed-four-interpreter", 0) => "signed-four-interpreter-unsigned-shortcut",
                ("signed-four-interpreter", 1) => "signed-four-interpreter-zero-extension",
                ("signed-four-interpreter", 2) => "signed-four-interpreter-valid-sentinel",
                ("signed-four-cranelift", 0) => "signed-four-cranelift-unsigned-shortcut",
                ("signed-four-cranelift", 1) => "signed-four-cranelift-zero-extension",
                ("signed-four-cranelift", 2) => "signed-four-cranelift-valid-sentinel",
                ("signed-four-fallback", 0) => "signed-four-fallback-unsigned-shortcut",
                ("signed-four-fallback", 1) => "signed-four-fallback-zero-extension",
                ("signed-four-fallback", 2) => "signed-four-fallback-valid-sentinel",
                _ => unreachable!(),
            };
            result.push(Mutation {
                name,
                contract,
                path,
                source: format!(
                    "{}{}{}",
                    &original[..start],
                    body.replace(old, new),
                    &original[end..]
                ),
            });
        }
    }
    for contract in selected
        .iter()
        .filter(|c| c.group == "multidim-array-index")
    {
        let path = "veryl/crates/simulator/src/ir/expression.rs";
        let original = fs::read_to_string(repo.join(path)).unwrap();
        let start = original
            .find("pub fn build_linear_index_expr(")
            .expect("multidim-array function moved");
        let end = start
            + original[start..]
                .find("\n}")
                .expect("multidim-array brace moved")
            + 2;
        let body = &original[start..end];
        for kind in 0..3 {
            let (old, new) = match kind {
                0 => ("op: Op::LogicAnd,", "op: Op::LogicOr,"),
                1 => ("base *= dim_size;", "base *= 1;"),
                2 => ("op: Op::Less,", "op: Op::LessEq,"),
                _ => unreachable!(),
            };
            assert_eq!(
                body.matches(old).count(),
                1,
                "multidim-array mutation site changed"
            );
            let name = match (contract.name, kind) {
                ("multi-interpreter", 0) => "multi-interpreter-validity-or",
                ("multi-interpreter", 1) => "multi-interpreter-wrong-stride",
                ("multi-interpreter", 2) => "multi-interpreter-inclusive-bound",
                ("multi-cranelift", 0) => "multi-cranelift-validity-or",
                ("multi-cranelift", 1) => "multi-cranelift-wrong-stride",
                ("multi-cranelift", 2) => "multi-cranelift-inclusive-bound",
                ("multi-aot-c", 0) => "multi-aot-c-validity-or",
                ("multi-aot-c", 1) => "multi-aot-c-wrong-stride",
                ("multi-aot-c", 2) => "multi-aot-c-inclusive-bound",
                ("multi-four-interpreter", 0) => "multi-four-interpreter-validity-or",
                ("multi-four-interpreter", 1) => "multi-four-interpreter-wrong-stride",
                ("multi-four-interpreter", 2) => "multi-four-interpreter-inclusive-bound",
                ("multi-four-cranelift", 0) => "multi-four-cranelift-validity-or",
                ("multi-four-cranelift", 1) => "multi-four-cranelift-wrong-stride",
                ("multi-four-cranelift", 2) => "multi-four-cranelift-inclusive-bound",
                ("multi-four-fallback", 0) => "multi-four-fallback-validity-or",
                ("multi-four-fallback", 1) => "multi-four-fallback-wrong-stride",
                ("multi-four-fallback", 2) => "multi-four-fallback-inclusive-bound",
                _ => unreachable!(),
            };
            result.push(Mutation {
                name,
                contract,
                path,
                source: format!(
                    "{}{}{}",
                    &original[..start],
                    body.replace(old, new),
                    &original[end..]
                ),
            });
        }
    }
    for contract in selected
        .iter()
        .filter(|c| c.group == "unknown-packed-select")
    {
        let jit = contract.name.ends_with("cranelift");
        let array = contract.name.starts_with("select-array");
        let path = if jit {
            "veryl/crates/simulator/src/backend/cranelift/helpers.rs"
        } else {
            "veryl/crates/simulator/src/ir/statement.rs"
        };
        let original = fs::read_to_string(repo.join(path)).unwrap();
        let decl = if jit {
            "pub(crate) fn build_dynamic_select_shift_checked("
        } else if array {
            "impl AssignDynamicStatement {"
        } else {
            "impl AssignStatement {"
        };
        let start = original.find(decl).expect("unknown-select function moved");
        let end = start
            + original[start..]
                .find("\n}")
                .expect("unknown-select brace moved")
            + 2;
        let body = &original[start..end];
        for low_only in [false, true] {
            let index_name = if array { "dyn_idx" } else { "idx" };
            let (old, new) = if jit {
                if low_only {
                    ("let in_range = if let Some(unknown) = wide_any_xz(".to_string(),"let idx_mask_xz = idx_mask_xz.map(|mask| builder.ins().band_imm(mask, 1));\n    let in_range = if let Some(unknown) = wide_any_xz(".to_string())
                } else {
                    ("let known = builder.ins().icmp_imm_s(IntCC::Equal, unknown, 0);".to_string(),"let known = builder.ins().icmp_imm_s(IntCC::UnsignedGreaterThanOrEqual, unknown, 0);".to_string())
                }
            } else if low_only {
                (
                    format!(
                        "let Some({index_name}) = dyn_sel.index_expr.eval(mask_cache).to_usize() else {{"
                    ),
                    format!(
                        "let index_value = dyn_sel.index_expr.eval(mask_cache);\n            let Some({index_name}) = (if index_value.mask_xz_u128() & 1 == 0 {{Some(index_value.payload_u64() as usize)}} else {{None}}) else {{"
                    ),
                )
            } else {
                (
                    format!(
                        "let Some({index_name}) = dyn_sel.index_expr.eval(mask_cache).to_usize() else {{\n                return;\n            }};"
                    ),
                    format!(
                        "let {index_name} = dyn_sel.index_expr.eval(mask_cache).to_usize().unwrap_or(0);"
                    ),
                )
            };
            assert_eq!(
                body.matches(&old).count(),
                1,
                "unknown-select mutation site changed"
            );
            let name = match (contract.name, low_only) {
                ("select-scalar-interpreter", false) => {
                    "select-scalar-interpreter-ignored-unknown-regression"
                }
                ("select-scalar-interpreter", true) => "select-scalar-interpreter-low-mask-only",
                ("select-scalar-cranelift", false) => {
                    "select-scalar-cranelift-ignored-unknown-regression"
                }
                ("select-scalar-cranelift", true) => "select-scalar-cranelift-low-mask-only",
                ("select-scalar-fallback", false) => {
                    "select-scalar-fallback-ignored-unknown-regression"
                }
                ("select-scalar-fallback", true) => "select-scalar-fallback-low-mask-only",
                ("select-array-interpreter", false) => {
                    "select-array-interpreter-ignored-unknown-regression"
                }
                ("select-array-interpreter", true) => "select-array-interpreter-low-mask-only",
                ("select-array-cranelift", false) => {
                    "select-array-cranelift-ignored-unknown-regression"
                }
                ("select-array-cranelift", true) => "select-array-cranelift-low-mask-only",
                ("select-array-fallback", false) => {
                    "select-array-fallback-ignored-unknown-regression"
                }
                ("select-array-fallback", true) => "select-array-fallback-low-mask-only",
                _ => unreachable!(),
            };
            result.push(Mutation {
                name,
                contract,
                path,
                source: format!(
                    "{}{}{}",
                    &original[..start],
                    body.replace(&old, &new),
                    &original[end..]
                ),
            });
        }
    }
    for contract in selected.iter().filter(|c| c.group == "packed-select-forms") {
        let jit = contract.name.ends_with("cranelift");
        let array = contract.name.starts_with("forms-array");
        for kind in 0..3 {
            let (path, decl, old, new) = match kind {
                0 => (
                    "veryl/crates/simulator/src/ir/expression.rs",
                    "fn normalize_dynamic_select(",
                    "result.window / result.elem_width - 1",
                    "result.window / result.elem_width",
                ),
                1 => (
                    "veryl/crates/simulator/src/ir/expression.rs",
                    "pub fn build_dynamic_bit_select(",
                    "y: Box::new(const_op(w)),",
                    "y: Box::new(const_op(w.saturating_sub(1))),",
                ),
                2 if jit => (
                    "veryl/crates/simulator/src/backend/cranelift/helpers.rs",
                    "pub(crate) fn build_dynamic_select_shift_checked(",
                    "let skip = builder.ins().select(left, skip, zero);",
                    "let skip = builder.ins().select(left, zero, zero);",
                ),
                // Wide scalar descending writes compile in the mixed fallback cases.
                2 if !array && contract.name.ends_with("fallback") => (
                    "veryl/crates/simulator/src/backend/cranelift/selection.rs",
                    "fn bounds(",
                    "let skip = builder.ins().select(left, skip, zero);",
                    "let skip = builder.ins().select(left, zero, zero);",
                ),
                2 => (
                    "veryl/crates/simulator/src/ir/statement.rs",
                    if array {
                        "impl AssignDynamicStatement {"
                    } else {
                        "impl AssignStatement {"
                    },
                    ".select(dyn_sel.window - 1, skip)",
                    ".select(dyn_sel.window - 1, 0)",
                ),
                _ => unreachable!(),
            };
            let original = fs::read_to_string(repo.join(path)).unwrap();
            let start = original
                .find(decl)
                .expect("select-forms mutation function moved");
            let end = start
                + original[start..]
                    .find("\n}")
                    .expect("select-forms brace moved")
                + 2;
            let body = &original[start..end];
            assert_eq!(
                body.matches(old).count(),
                1,
                "select-forms mutation site changed"
            );
            let name = match (contract.name, kind) {
                ("forms-scalar-interpreter", 0) => "forms-scalar-interpreter-descending-offset",
                ("forms-scalar-interpreter", 1) => "forms-scalar-interpreter-step-stride",
                ("forms-scalar-interpreter", 2) => "forms-scalar-interpreter-source-clip",
                ("forms-scalar-cranelift", 0) => "forms-scalar-cranelift-descending-offset",
                ("forms-scalar-cranelift", 1) => "forms-scalar-cranelift-step-stride",
                ("forms-scalar-cranelift", 2) => "forms-scalar-cranelift-source-clip",
                ("forms-scalar-fallback", 0) => "forms-scalar-fallback-descending-offset",
                ("forms-scalar-fallback", 1) => "forms-scalar-fallback-step-stride",
                ("forms-scalar-fallback", 2) => "forms-scalar-fallback-source-clip",
                ("forms-array-interpreter", 0) => "forms-array-interpreter-descending-offset",
                ("forms-array-interpreter", 1) => "forms-array-interpreter-step-stride",
                ("forms-array-interpreter", 2) => "forms-array-interpreter-source-clip",
                ("forms-array-cranelift", 0) => "forms-array-cranelift-descending-offset",
                ("forms-array-cranelift", 1) => "forms-array-cranelift-step-stride",
                ("forms-array-cranelift", 2) => "forms-array-cranelift-source-clip",
                ("forms-array-fallback", 0) => "forms-array-fallback-descending-offset",
                ("forms-array-fallback", 1) => "forms-array-fallback-step-stride",
                ("forms-array-fallback", 2) => "forms-array-fallback-source-clip",
                _ => unreachable!(),
            };
            result.push(Mutation {
                name,
                contract,
                path,
                source: format!(
                    "{}{}{}",
                    &original[..start],
                    body.replace(old, new),
                    &original[end..]
                ),
            });
        }
    }
    for contract in selected
        .iter()
        .filter(|c| c.group == "signed-packed-select")
    {
        let jit = contract.name.ends_with("cranelift");
        for kind in 0..3 {
            let (path, decl, old, new) = match kind {
                0 => (
                    "veryl/crates/simulator/src/ir/expression.rs",
                    "fn resize_to_width(",
                    "let sign_extend = expr.expr_context().signed;",
                    "let sign_extend = false;",
                ),
                // Preserve compiled dispatch while corrupting the bias.
                1 if contract.name == "signedsel-scalar-wide-cranelift" => (
                    "veryl/crates/simulator/src/ir/expression.rs",
                    "fn normalize_dynamic_select(",
                    "result.low_adjust += bias;",
                    "result.low_adjust += bias.saturating_sub(1).max(1);",
                ),
                1 => (
                    "veryl/crates/simulator/src/ir/expression.rs",
                    "fn normalize_dynamic_select(",
                    "result.low_adjust += bias;",
                    "result.low_adjust += 0;",
                ),
                2 if contract.name == "signedsel-scalar-wide-cranelift" => (
                    "veryl/crates/simulator/src/backend/cranelift/selection.rs",
                    "fn bounds(",
                    "Some((start, skip, builder.ins().select(valid, count, zero)))",
                    "let one = builder.ins().iconst(I64, 1); Some((start, skip, builder.ins().select(valid, count, one)))",
                ),
                2 if jit => (
                    "veryl/crates/simulator/src/backend/cranelift/helpers.rs",
                    "pub(crate) fn build_dynamic_select_shift_checked(",
                    ".icmp_imm_u(IntCC::UnsignedLessThan, rhs_shift, dyn_sel.window as i64)",
                    ".icmp_imm_u(IntCC::UnsignedGreaterThanOrEqual, rhs_shift, 0)",
                ),
                2 => (
                    "veryl/crates/simulator/src/ir/expression.rs",
                    "impl DynamicBitSelect {",
                    "if low >= elements || skip >= self.window {",
                    "if skip >= self.window {return Some((0,0,0));} if low >= elements {",
                ),
                _ => unreachable!(),
            };
            let original = fs::read_to_string(repo.join(path)).unwrap();
            let start = original.find(decl).expect("signed-select function moved");
            let end = start
                + original[start..]
                    .find("\n}")
                    .expect("signed-select brace moved")
                + 2;
            let body = &original[start..end];
            assert_eq!(
                body.matches(old).count(),
                1,
                "signed-select mutation site changed"
            );
            let name = match (contract.name, kind) {
                ("signedsel-scalar-interpreter", 0) => {
                    "signedsel-scalar-interpreter-zero-extension"
                }
                ("signedsel-scalar-interpreter", 1) => {
                    "signedsel-scalar-interpreter-missing-bias-adjustment"
                }
                ("signedsel-scalar-interpreter", 2) => {
                    "signedsel-scalar-interpreter-outside-overlap"
                }
                ("signedsel-scalar-cranelift", 0) => "signedsel-scalar-cranelift-zero-extension",
                ("signedsel-scalar-cranelift", 1) => {
                    "signedsel-scalar-cranelift-missing-bias-adjustment"
                }
                ("signedsel-scalar-cranelift", 2) => "signedsel-scalar-cranelift-outside-overlap",
                ("signedsel-scalar-wide-cranelift", 0) => {
                    "signedsel-scalar-wide-cranelift-zero-extension"
                }
                ("signedsel-scalar-wide-cranelift", 1) => {
                    "signedsel-scalar-wide-cranelift-off-by-one-bias"
                }
                ("signedsel-scalar-wide-cranelift", 2) => {
                    "signedsel-scalar-wide-cranelift-outside-overlap"
                }
                ("signedsel-array-interpreter", 0) => "signedsel-array-interpreter-zero-extension",
                ("signedsel-array-interpreter", 1) => {
                    "signedsel-array-interpreter-missing-bias-adjustment"
                }
                ("signedsel-array-interpreter", 2) => "signedsel-array-interpreter-outside-overlap",
                ("signedsel-array-cranelift", 0) => "signedsel-array-cranelift-zero-extension",
                ("signedsel-array-cranelift", 1) => {
                    "signedsel-array-cranelift-missing-bias-adjustment"
                }
                ("signedsel-array-cranelift", 2) => "signedsel-array-cranelift-outside-overlap",
                ("signedsel-array-fallback", 0) => "signedsel-array-fallback-zero-extension",
                ("signedsel-array-fallback", 1) => {
                    "signedsel-array-fallback-missing-bias-adjustment"
                }
                ("signedsel-array-fallback", 2) => "signedsel-array-fallback-outside-overlap",
                _ => unreachable!(),
            };
            result.push(Mutation {
                name,
                contract,
                path,
                source: format!(
                    "{}{}{}",
                    &original[..start],
                    body.replace(old, new),
                    &original[end..]
                ),
            });
        }
    }
    for contract in selected.iter().filter(|c| c.group == "wide-packed-index") {
        let path = "veryl/crates/simulator/src/ir/expression.rs";
        let original = fs::read_to_string(repo.join(path)).unwrap();
        for kind in 0..3 {
            let (decl, old, new) = match kind {
                0 => (
                    "fn normalize_dynamic_select(",
                    "if let Some(valid) = validity {",
                    "if let Some(valid) = validity.filter(|_| false) {",
                ),
                1 => (
                    "fn packed_write_coordinate_valid(",
                    "op: Op::LogicAnd,",
                    "op: Op::LogicOr,",
                ),
                2 => (
                    "fn normalize_dynamic_select(",
                    "let outside = result.num_elements.saturating_add(result.low_adjust);",
                    "let outside = result.low_adjust;",
                ),
                _ => unreachable!(),
            };
            let start = original.find(decl).expect("wide-select function moved");
            let end = start
                + original[start..]
                    .find("\n}")
                    .expect("wide-select brace moved")
                + 2;
            let body = &original[start..end];
            assert_eq!(
                body.matches(old).count(),
                1,
                "wide-select mutation site changed"
            );
            let name = match (contract.name, kind) {
                ("wideidx-scalar-interpreter", 0) => {
                    "wideidx-scalar-interpreter-omit-coordinate-check"
                }
                ("wideidx-scalar-interpreter", 1) => "wideidx-scalar-interpreter-range-or",
                ("wideidx-scalar-interpreter", 2) => "wideidx-scalar-interpreter-valid-sentinel",
                ("wideidx-scalar-cranelift", 0) => "wideidx-scalar-cranelift-omit-coordinate-check",
                ("wideidx-scalar-cranelift", 1) => "wideidx-scalar-cranelift-range-or",
                ("wideidx-scalar-cranelift", 2) => "wideidx-scalar-cranelift-valid-sentinel",
                ("wideidx-scalar-fallback", 0) => "wideidx-scalar-fallback-omit-coordinate-check",
                ("wideidx-scalar-fallback", 1) => "wideidx-scalar-fallback-range-or",
                ("wideidx-scalar-fallback", 2) => "wideidx-scalar-fallback-valid-sentinel",
                ("wideidx-array-interpreter", 0) => {
                    "wideidx-array-interpreter-omit-coordinate-check"
                }
                ("wideidx-array-interpreter", 1) => "wideidx-array-interpreter-range-or",
                ("wideidx-array-interpreter", 2) => "wideidx-array-interpreter-valid-sentinel",
                ("wideidx-array-cranelift", 0) => "wideidx-array-cranelift-omit-coordinate-check",
                ("wideidx-array-cranelift", 1) => "wideidx-array-cranelift-range-or",
                ("wideidx-array-cranelift", 2) => "wideidx-array-cranelift-valid-sentinel",
                ("wideidx-array-fallback", 0) => "wideidx-array-fallback-omit-coordinate-check",
                ("wideidx-array-fallback", 1) => "wideidx-array-fallback-range-or",
                ("wideidx-array-fallback", 2) => "wideidx-array-fallback-valid-sentinel",
                _ => unreachable!(),
            };
            result.push(Mutation {
                name,
                contract,
                path,
                source: format!(
                    "{}{}{}",
                    &original[..start],
                    body.replace(old, new),
                    &original[end..]
                ),
            });
        }
    }
    for contract in selected
        .iter()
        .filter(|c| matches!(c.group, "packed-read-boundary" | "wide-packed-read"))
    {
        let jit = contract.name.ends_with("cranelift");
        let names: &[&str] = match contract.name {
            "read-scalar-interpreter" => &[
                "read-scalar-interpreter-unknown-index",
                "read-scalar-interpreter-missing-default",
                "read-scalar-interpreter-missing-padding",
            ],
            "read-scalar-cranelift" => &[
                "read-scalar-cranelift-unknown-index",
                "read-scalar-cranelift-missing-default",
                "read-scalar-cranelift-missing-padding",
            ],
            "read-array-interpreter" => &[
                "read-array-interpreter-unknown-index",
                "read-array-interpreter-missing-default",
                "read-array-interpreter-missing-padding",
            ],
            "read-array-cranelift" => &[
                "read-array-cranelift-unknown-index",
                "read-array-cranelift-missing-default",
                "read-array-cranelift-missing-padding",
            ],
            "read-array-fallback" => &[
                "read-array-fallback-unknown-index",
                "read-array-fallback-missing-default",
                "read-array-fallback-missing-padding",
            ],
            "wide-read-scalar-interpreter" => &[
                "wide-read-scalar-interpreter-unknown-index",
                "wide-read-scalar-interpreter-missing-default",
                "wide-read-scalar-interpreter-missing-padding",
                "wide-read-scalar-interpreter-repeat-word",
            ],
            "wide-read-scalar-cranelift" => &[
                "wide-read-scalar-cranelift-unknown-index",
                "wide-read-scalar-cranelift-missing-default",
                "wide-read-scalar-cranelift-missing-padding",
                "wide-read-scalar-cranelift-repeat-word",
            ],
            "wide-read-array-interpreter" => &[
                "wide-read-array-interpreter-unknown-index",
                "wide-read-array-interpreter-missing-default",
                "wide-read-array-interpreter-missing-padding",
                "wide-read-array-interpreter-repeat-word",
            ],
            "wide-read-array-cranelift" => &[
                "wide-read-array-cranelift-unknown-index",
                "wide-read-array-cranelift-missing-default",
                "wide-read-array-cranelift-missing-padding",
                "wide-read-array-cranelift-repeat-word",
            ],
            "wide-read-array-fallback" => &[
                "wide-read-array-fallback-unknown-index",
                "wide-read-array-fallback-missing-default",
                "wide-read-array-fallback-missing-padding",
                "wide-read-array-fallback-repeat-word",
            ],
            _ => unreachable!(),
        };
        for (kind, &name) in names.iter().enumerate() {
            // Adjusted wide reads compile even in the mixed fallback cases.
            let (path, decl, old, new, count) = if kind == 2
                && (contract.name.ends_with("fallback")
                    || (jit && contract.group == "wide-packed-read"))
            {
                (
                    "veryl/crates/simulator/src/backend/cranelift/selection.rs",
                    "fn bounds(",
                    "let skip = builder.ins().select(left, skip, zero);",
                    "let skip = builder.ins().select(left, zero, zero);",
                    1,
                )
            } else if jit {
                match kind {
                    0 => (
                        "veryl/crates/simulator/src/backend/cranelift/helpers.rs",
                        "pub(crate) fn build_dynamic_select_shift_checked(",
                        "let known = builder.ins().icmp_imm_s(IntCC::Equal, unknown, 0);",
                        "let known = builder.ins().icmp_imm_s(IntCC::UnsignedGreaterThanOrEqual, unknown, 0);",
                        1,
                    ),
                    1 => (
                        "veryl/crates/simulator/src/backend/cranelift/expression.rs",
                        "fn finish_packed_read(",
                        "Some(builder.ins().bor(mask, missing))",
                        "let _ = missing; Some(mask)",
                        1,
                    ),
                    2 => (
                        "veryl/crates/simulator/src/backend/cranelift/expression.rs",
                        "fn finish_packed_read(",
                        "let (shift, in_range, padding) = position;",
                        "let (shift, in_range, padding) = position; let padding = padding.map(|_| builder.ins().iconst(I64, 0));",
                        1,
                    ),
                    _ => (
                        "veryl/crates/simulator/src/backend/cranelift/expression.rs",
                        "fn emit_wide_dynsel_read_multi(",
                        "let source_start = builder.ins().iadd(shift, advance);",
                        "let _ = advance; let source_start = shift;",
                        1,
                    ),
                }
            } else {
                match kind {
                    0 => (
                        "veryl/crates/simulator/src/ir/expression.rs",
                        "    pub fn eval(",
                        "let Some(idx) = dyn_sel.index_expr.eval(mask_cache).to_usize() else {\n                        return out_of_range_read(dyn_sel.window, *signed, *use_4state);\n                    };",
                        "let idx = dyn_sel.index_expr.eval(mask_cache).to_usize().unwrap_or(0);",
                        2,
                    ),
                    1 => (
                        "veryl/crates/simulator/src/ir/expression.rs",
                        "    pub fn read_value(",
                        "let mut result = out_of_range_read(self.window, false, use_4state);",
                        "let _ = use_4state; let mut result = Value::new(0, self.window, false);",
                        1,
                    ),
                    2 => (
                        "veryl/crates/simulator/src/ir/expression.rs",
                        "    pub fn read_value(",
                        "result.assign(selected, skip + beg - end, skip);",
                        "let _ = skip; result.assign(selected, beg - end, 0);",
                        1,
                    ),
                    _ => (
                        "veryl/crates/simulator/src/ir/expression.rs",
                        "    pub fn read_value(",
                        "source.select(beg, end)",
                        "source.select(beg.min(63), end.min(63))",
                        1,
                    ),
                }
            };
            let original = fs::read_to_string(repo.join(path)).unwrap();
            let start = original.find(decl).expect("packed-read function moved");
            // The runtime eval method contains both scalar and array handlers.
            let closing = if decl.starts_with("    ") {
                "\n    }"
            } else {
                "\n}"
            };
            let end = start + original[start..].find(closing).unwrap() + closing.len();
            let body = &original[start..end];
            assert_eq!(
                body.matches(old).count(),
                count,
                "packed-read mutation site changed: {name}"
            );
            result.push(Mutation {
                name,
                contract,
                path,
                source: format!(
                    "{}{}{}",
                    &original[..start],
                    body.replace(old, new),
                    &original[end..]
                ),
            });
        }
    }
    for contract in selected.iter().filter(|c| {
        matches!(
            c.group,
            "constant-packed-select" | "constant-select-diagnostics"
        )
    }) {
        let names: &[&str] = match contract.name {
            "const-read-interpreter" => &[
                "const-read-interpreter-missing-default",
                "const-read-interpreter-partial-origin",
                "const-read-interpreter-in-range-origin",
            ],
            "const-read-cranelift" => &[
                "const-read-cranelift-missing-default",
                "const-read-cranelift-partial-origin",
                "const-read-cranelift-in-range-origin",
            ],
            "const-write-interpreter" => &[
                "const-write-interpreter-outside-alias",
                "const-write-interpreter-short-clip",
                "const-write-interpreter-in-range-origin",
            ],
            "const-write-cranelift" => &[
                "const-write-cranelift-outside-alias",
                "const-write-cranelift-short-clip",
                "const-write-cranelift-in-range-origin",
            ],
            "const-write-wide-cranelift" => &[
                "const-write-wide-cranelift-outside-alias",
                "const-write-wide-cranelift-short-clip",
                "const-write-wide-cranelift-in-range-origin",
            ],
            "const-write-wide-fallback" => &[
                "const-write-wide-fallback-outside-alias",
                "const-write-wide-fallback-short-clip",
                "const-write-wide-fallback-in-range-origin",
            ],
            "const-select-diagnostics" => &[
                "const-select-diagnostics-loose-bound",
                "const-select-diagnostics-missing-order",
            ],
            _ => unreachable!(),
        };
        for (kind, &name) in names.iter().enumerate() {
            let (path, decl, old, new) = if contract.name == "const-select-diagnostics" {
                match kind {
                    0 => (
                        "veryl/crates/analyzer/src/ir/variable.rs",
                        "    pub fn eval_comptime(",
                        "if high >= size {",
                        "if high > size {",
                    ),
                    _ => (
                        "veryl/crates/analyzer/src/ir/variable.rs",
                        "    pub fn eval_comptime(",
                        "let wrong_order = if is_array { beg > end } else { beg < end };",
                        "let wrong_order = false;",
                    ),
                }
            } else if contract.name.starts_with("const-read-") {
                match kind {
                    0 => (
                        "veryl/crates/simulator/src/ir/expression.rs",
                        "pub fn storage_bounded_read(",
                        "value: out_of_range_read(lost, false, use_4state),",
                        "value: Value::new(0, lost, false),",
                    ),
                    1 => (
                        "veryl/crates/simulator/src/ir/expression.rs",
                        "pub fn storage_bounded_read(",
                        "let inner = build(Some((clipped_beg, clipped_end)), kept, inner_context);",
                        "let pair = if clipped_end > 0 {(clipped_beg - 1, clipped_end - 1)} else {(clipped_beg, clipped_end)}; let inner = build(Some(pair), kept, inner_context);",
                    ),
                    _ => (
                        "veryl/crates/simulator/src/ir/expression.rs",
                        "pub fn storage_bounded_read(",
                        "return build(select, span, expr_context);",
                        "let select = select.map(|(b,e)| if b + 1 < var_width {(b+1,e+1)} else {(b,e)}); return build(select, span, expr_context);",
                    ),
                }
            } else {
                match kind {
                    0 => (
                        "veryl/crates/simulator/src/ir/statement.rs",
                        "    fn conv_local_assignment(",
                        "let Some(clipped) = clip_select_to_width(beg, end, dst_width) else {\n                    return Ok(ProtoStatement::SequentialBlock(Vec::new()));\n                };",
                        "let clipped = clip_select_to_width(beg, end, dst_width).unwrap_or((0, 0));",
                    ),
                    1 => (
                        "veryl/crates/simulator/src/ir/statement.rs",
                        "    fn conv_local_assignment(",
                        "                Some(clipped)\n",
                        "                Some((clipped.0.saturating_sub(1).max(clipped.1), clipped.1))\n",
                    ),
                    _ => (
                        "veryl/crates/simulator/src/ir/statement.rs",
                        "    fn conv_local_assignment(",
                        "let dynamic_select = if need_dynamic_select {",
                        "let select = select.map(|(b,e)| if b + 1 < dst_width {(b+1,e+1)} else {(b,e)}); let dynamic_select = if need_dynamic_select {",
                    ),
                }
            };
            let original = fs::read_to_string(repo.join(path)).unwrap();
            let start = if decl.contains("eval_comptime") {
                let begin = original.find("impl VarSelect {").unwrap();
                begin + original[begin..].find(decl).unwrap()
            } else {
                original.find(decl).unwrap()
            };
            let closing = if decl.starts_with("    ") {
                "\n    }"
            } else {
                "\n}"
            };
            let end = start + original[start..].find(closing).unwrap() + closing.len();
            let body = &original[start..end];
            assert_eq!(
                body.matches(old).count(),
                1,
                "constant-select mutation site changed: {name}"
            );
            result.push(Mutation {
                name,
                contract,
                path,
                source: format!(
                    "{}{}{}",
                    &original[..start],
                    body.replace(old, new),
                    &original[end..]
                ),
            });
        }
    }
    for contract in selected.iter().filter(|c| {
        matches!(
            c.group,
            "constant-packed-edge" | "constant-edge-diagnostics"
        )
    }) {
        let names: &[&str] = match contract.name {
            "edge-read-interpreter" => &[
                "edge-read-interpreter-unknown-to-zero",
                "edge-read-interpreter-unsigned-negative",
                "edge-read-interpreter-missing-lower-adjust",
                "edge-read-interpreter-mask-or-padding",
            ],
            "edge-read-cranelift" => &[
                "edge-read-cranelift-unknown-to-zero",
                "edge-read-cranelift-unsigned-negative",
                "edge-read-cranelift-missing-lower-adjust",
                "edge-read-cranelift-mask-or-padding",
            ],
            "edge-write-interpreter" => &[
                "edge-write-interpreter-unknown-to-zero",
                "edge-write-interpreter-unsigned-negative",
                "edge-write-interpreter-missing-lower-adjust",
                "edge-write-interpreter-mask-or-padding",
            ],
            "edge-write-cranelift" => &[
                "edge-write-cranelift-unknown-to-zero",
                "edge-write-cranelift-unsigned-negative",
                "edge-write-cranelift-missing-lower-adjust",
                "edge-write-cranelift-mask-or-padding",
            ],
            "edge-write-fallback" => &[
                "edge-write-fallback-unknown-to-zero",
                "edge-write-fallback-unsigned-negative",
                "edge-write-fallback-missing-lower-adjust",
                "edge-write-fallback-mask-or-padding",
            ],
            "edge-negative-diagnostics" => &["edge-negative-diagnostics-missing-size-diagnostic"],
            _ => unreachable!(),
        };
        for (kind, &name) in names.iter().enumerate() {
            let (path, decl, old, new) = if contract.name == "edge-negative-diagnostics" {
                (
                    "veryl/crates/analyzer/src/conv/context.rs",
                    "    pub fn check_size(",
                    "self.insert_error(AnalyzerError::exceed_limit(\n                ExceedLimitKind::EvaluateSize,\n                x,\n                &token,\n            ));",
                    "let _ = token;",
                )
            } else {
                match kind {
                    0 => (
                        "veryl/crates/simulator/src/ir/expression.rs",
                        "pub(crate) fn packed_select_needs_dynamic(",
                        "if value.is_xz()",
                        "if false",
                    ),
                    1 => (
                        "veryl/crates/simulator/src/ir/expression.rs",
                        "pub(crate) fn packed_select_needs_dynamic(",
                        "value.signed()",
                        "false",
                    ),
                    2 => (
                        "veryl/crates/simulator/src/ir/expression.rs",
                        "pub(crate) fn packed_select_needs_dynamic(",
                        "high.checked_sub(width.saturating_sub(1)).is_none()",
                        "{let _ = (high, width); false}",
                    ),
                    _ if contract.name == "edge-write-fallback" => (
                        "veryl/crates/simulator/src/backend/cranelift/selection.rs",
                        "fn bounds(",
                        "let skip = builder.ins().select(left, skip, zero);",
                        "let skip = builder.ins().select(left, zero, zero);",
                    ),
                    _ if contract.name.ends_with("cranelift") => (
                        "veryl/crates/simulator/src/backend/cranelift/helpers.rs",
                        "pub(crate) fn build_dynamic_select_shift_checked(",
                        "Some((shift, in_range, rhs_shift))",
                        "Some((shift, in_range, builder.ins().iconst(I64, 0)))",
                    ),
                    _ => (
                        "veryl/crates/simulator/src/ir/expression.rs",
                        "pub fn build_dynamic_bit_select(",
                        "value.clear_xz();",
                        "let _ = value;",
                    ),
                }
            };
            let original = fs::read_to_string(repo.join(path)).unwrap();
            let start = original.find(decl).unwrap();
            let closing = if decl.starts_with("    ") {
                "\n    }"
            } else {
                "\n}"
            };
            let end = start + original[start..].find(closing).unwrap() + closing.len();
            let body = &original[start..end];
            assert_eq!(
                body.matches(old).count(),
                1,
                "constant-edge mutation site changed: {name}"
            );
            result.push(Mutation {
                name,
                contract,
                path,
                source: format!(
                    "{}{}{}",
                    &original[..start],
                    body.replace(old, new),
                    &original[end..]
                ),
            });
        }
    }
    for contract in selected.iter().filter(|c| c.group == "packed-dimension") {
        let names = match contract.name {
            "packdim-read-interpreter" => [
                "packdim-read-interpreter-loose-axis-bound",
                "packdim-read-interpreter-wrong-stride",
                "packdim-read-interpreter-zero-sentinel",
                "packdim-read-interpreter-static-constant",
            ],
            "packdim-read-cranelift" => [
                "packdim-read-cranelift-loose-axis-bound",
                "packdim-read-cranelift-wrong-stride",
                "packdim-read-cranelift-zero-sentinel",
                "packdim-read-cranelift-static-constant",
            ],
            "packdim-write-interpreter" => [
                "packdim-write-interpreter-loose-axis-bound",
                "packdim-write-interpreter-wrong-stride",
                "packdim-write-interpreter-zero-sentinel",
                "packdim-write-interpreter-static-constant",
            ],
            "packdim-write-cranelift" => [
                "packdim-write-cranelift-loose-axis-bound",
                "packdim-write-cranelift-wrong-stride",
                "packdim-write-cranelift-zero-sentinel",
                "packdim-write-cranelift-static-constant",
            ],
            "packdim-write-wide-two" => [
                "packdim-write-wide-two-loose-axis-bound",
                "packdim-write-wide-two-wrong-stride",
                "packdim-write-wide-two-zero-sentinel",
                "packdim-write-wide-two-static-constant",
            ],
            "packdim-write-wide-fallback" => [
                "packdim-write-wide-fallback-loose-axis-bound",
                "packdim-write-wide-fallback-wrong-stride",
                "packdim-write-wide-fallback-zero-sentinel",
                "packdim-write-wide-fallback-static-constant",
            ],
            _ => unreachable!(),
        };
        for (kind, name) in names.into_iter().enumerate() {
            let (decl, old, new) = match kind {
                0 => (
                    "pub fn build_linear_index_expr(",
                    "op: Op::Less,",
                    "op: Op::LessEq,",
                ),
                1 => (
                    "pub fn build_linear_index_expr(",
                    "value: Value::new(base as u64, index_width, false),",
                    "value: Value::new(base.saturating_sub(1) as u64, index_width, false),",
                ),
                2 => (
                    "pub fn build_linear_index_expr(",
                    "value: Value::new(base as u64, 64, false),",
                    "value: Value::new(0, 64, false),",
                ),
                _ => (
                    "pub(crate) fn packed_select_needs_dynamic(",
                    "if select.dimension() > 1",
                    "if false",
                ),
            };
            let path = "veryl/crates/simulator/src/ir/expression.rs";
            let original = fs::read_to_string(repo.join(path)).unwrap();
            let start = original.find(decl).unwrap();
            let end = start + original[start..].find("\n}").unwrap() + 2;
            let body = &original[start..end];
            assert_eq!(
                body.matches(old).count(),
                1,
                "packed-dimension mutation site changed: {name}"
            );
            result.push(Mutation {
                name,
                contract,
                path,
                source: format!(
                    "{}{}{}",
                    &original[..start],
                    body.replace(old, new),
                    &original[end..]
                ),
            });
        }
    }
    for contract in selected
        .iter()
        .filter(|c| matches!(c.group, "packed-range" | "packed-stride-range"))
    {
        let names: &[&'static str] = match contract.name {
            "packrange-read-interpreter" => &[
                "packrange-read-interpreter-flat-range",
                "packrange-read-interpreter-short-row",
                "packrange-read-interpreter-zero-prefix",
                "packrange-read-interpreter-shifted-local",
            ],
            "packrange-read-cranelift" => &[
                "packrange-read-cranelift-flat-range",
                "packrange-read-cranelift-short-row",
                "packrange-read-cranelift-zero-prefix",
                "packrange-read-cranelift-shifted-local",
            ],
            "packrange-write-interpreter" => &[
                "packrange-write-interpreter-flat-range",
                "packrange-write-interpreter-short-row",
                "packrange-write-interpreter-zero-prefix",
                "packrange-write-interpreter-shifted-local",
            ],
            "packrange-write-cranelift" => &[
                "packrange-write-cranelift-flat-range",
                "packrange-write-cranelift-short-row",
                "packrange-write-cranelift-zero-prefix",
                "packrange-write-cranelift-shifted-local",
            ],
            "packrange-write-wide-two" => &[
                "packrange-write-wide-two-flat-range",
                "packrange-write-wide-two-short-row",
                "packrange-write-wide-two-zero-prefix",
                "packrange-write-wide-two-shifted-local",
            ],
            "packrange-write-wide-fallback" => &[
                "packrange-write-wide-fallback-flat-range",
                "packrange-write-wide-fallback-short-row",
                "packrange-write-wide-fallback-zero-prefix",
                "packrange-write-wide-fallback-shifted-local",
            ],
            "packstride-read-interpreter" => &[
                "packstride-read-interpreter-flat-range",
                "packstride-read-interpreter-short-row",
                "packstride-read-interpreter-zero-prefix",
                "packstride-read-interpreter-shifted-local",
                "packstride-read-interpreter-unit-stride",
            ],
            "packstride-read-cranelift" => &[
                "packstride-read-cranelift-flat-range",
                "packstride-read-cranelift-short-row",
                "packstride-read-cranelift-zero-prefix",
                "packstride-read-cranelift-shifted-local",
                "packstride-read-cranelift-unit-stride",
            ],
            "packstride-write-interpreter" => &[
                "packstride-write-interpreter-flat-range",
                "packstride-write-interpreter-short-row",
                "packstride-write-interpreter-zero-prefix",
                "packstride-write-interpreter-shifted-local",
                "packstride-write-interpreter-unit-stride",
            ],
            "packstride-write-cranelift" => &[
                "packstride-write-cranelift-flat-range",
                "packstride-write-cranelift-short-row",
                "packstride-write-cranelift-zero-prefix",
                "packstride-write-cranelift-shifted-local",
                "packstride-write-cranelift-unit-stride",
            ],
            "packstride-write-wide-two" => &[
                "packstride-write-wide-two-flat-range",
                "packstride-write-wide-two-short-row",
                "packstride-write-wide-two-zero-prefix",
                "packstride-write-wide-two-shifted-local",
                "packstride-write-wide-two-unit-stride",
            ],
            "packstride-write-wide-four-fallback" => &[
                "packstride-write-wide-four-fallback-flat-range",
                "packstride-write-wide-four-fallback-short-row",
                "packstride-write-wide-four-fallback-zero-prefix",
                "packstride-write-wide-four-fallback-shifted-local",
                "packstride-write-wide-four-fallback-unit-stride",
            ],
            _ => unreachable!(),
        };
        for (kind, name) in names.iter().copied().enumerate() {
            let (decl, old, new) = match kind {
                0 => (
                    "pub(crate) fn packed_range_row_type(",
                    "if select.dimension() < 2 || select.1.is_none() {",
                    "if select.dimension() < usize::MAX || select.1.is_none() {",
                ),
                1 => (
                    "pub(crate) fn build_packed_range_select(",
                    "result.row_elements = Some(elements);",
                    "result.row_elements = Some(elements.saturating_sub(1));",
                ),
                2 => (
                    "pub(crate) fn build_packed_range_select(",
                    "let row = resize_to_width(row, 64, ec);",
                    "let _ = row; let row = constant(0);",
                ),
                3 => (
                    "pub(crate) fn build_packed_range_select(",
                    "let local = resize_to_width(*result.index_expr, 64, ec);",
                    "let local = binary(resize_to_width(*result.index_expr, 64, ec), Op::Add, constant(1), 64);",
                ),
                _ => (
                    "pub(crate) fn build_packed_range_select(",
                    "let elements = result.num_elements;",
                    "result.elem_width = 1; let elements = result.num_elements;",
                ),
            };
            let path = "veryl/crates/simulator/src/ir/expression.rs";
            let original = fs::read_to_string(repo.join(path)).unwrap();
            let start = original.find(decl).unwrap();
            let end = start + original[start..].find("\n}").unwrap() + 2;
            let body = &original[start..end];
            assert_eq!(
                body.matches(old).count(),
                1,
                "packed-range mutation site changed: {name}"
            );
            result.push(Mutation {
                name,
                contract,
                path,
                source: format!(
                    "{}{}{}",
                    &original[..start],
                    body.replace(old, new),
                    &original[end..]
                ),
            });
        }
    }
    result.extend(event_contracts::mutations(repo, selected));
    result.extend(gate_extension_contracts::mutations(repo, selected));
    result
}

const fn event_contract(name: &'static str, harness: &'static str) -> Contract {
    Contract {
        name,
        group: "event-state",
        harness,
        message: "simulator event state mismatch",
        covers: &[],
    }
}
#[path = "event_contracts.rs"]
mod event_contracts;

const fn gate_extension_contract(
    name: &'static str,
    harness: &'static str,
    external: bool,
) -> Contract {
    Contract {
        name,
        harness,
        group: if external {
            "external-wide"
        } else {
            "gate-max"
        },
        message: if external {
            "simulator wide external output mismatch"
        } else {
            "simulator gate-cycle policy mismatch"
        },
        covers: &[],
    }
}
#[path = "gate_extension_contracts.rs"]
mod gate_extension_contracts;
