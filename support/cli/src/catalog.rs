//! Read the adapters' compiled catalogs rather than parsing Rust source text.
#![allow(dead_code)]
use serde_json::{Value, json};
use veryl_formal_runner::Contract;

#[path = "../../../targets/analyzer/tests/aggregate_contracts.rs"]
mod aggregate;
#[path = "../../../targets/analyzer/kani/src/bin/check_contracts/analyzer.rs"]
mod analyzer;
#[path = "../../../targets/simulator/backend_contracts.rs"]
mod backend;
#[path = "../../../targets/analyzer/tests/big_helper_contracts.rs"]
mod big_helpers;
#[path = "../../../targets/cache/contracts.rs"]
mod cache;
#[path = "../../../targets/analyzer/tests/dynamic_aggregate_contracts.rs"]
mod dynamic;
#[path = "../../../targets/emitter/contracts.rs"]
mod emitter;
#[path = "../../../targets/emitter/cross_contracts.rs"]
mod emitter_cross;
#[path = "../../../targets/analyzer/tests/expression_contracts.rs"]
mod expressions;
#[path = "../../../targets/frontend/contracts.rs"]
mod frontend;
#[path = "../../../targets/simulator/cbmc/contracts.rs"]
mod generated_c;
#[path = "../../../targets/analyzer/tests/graph_contracts.rs"]
mod graph;
#[path = "../../../targets/analyzer/tests/ir_contracts.rs"]
mod ir;
#[path = "../../../targets/analyzer/tests/nonlinear_wide_contracts.rs"]
mod nonlinear;
#[path = "../../../targets/simulator/optimization_contracts.rs"]
mod optimization;
#[path = "../../../targets/analyzer/tests/selection_contracts.rs"]
mod selection;
#[path = "../../../targets/simulator/contracts.rs"]
mod simulator;
#[path = "../../../targets/synthesizer/sequential_contracts.rs"]
mod synth_sequential;
#[path = "../../../targets/synthesizer/source_contracts.rs"]
mod synth_source;

pub fn entries() -> Vec<Value> {
    let mut out = Vec::new();
    let mut add = |target: &str,
                   method: &str,
                   path: &str,
                   manifest: &str,
                   adapter: &str,
                   entries: &[Contract]| {
        for c in entries {
            out.push(json!({
                "id": format!("{target}/{method}/{adapter}/{}", c.name),
                "name": c.name, "target": target, "method": method,
                "group": c.group, "harness": c.harness, "message": c.message,
                "covers": c.covers, "catalog": path, "manifest": manifest,
                "adapter": adapter,
            }));
        }
    };
    add(
        "analyzer",
        "kani",
        "targets/analyzer/kani/src/bin/check_contracts/analyzer.rs",
        "targets/analyzer/kani/Cargo.toml",
        "check_contracts",
        &analyzer::CONTRACTS,
    );
    let manifest = "targets/analyzer/tests/Cargo.toml";
    for (name, path, entries) in [
        (
            "aggregate",
            "aggregate_contracts.rs",
            aggregate::CONTRACTS.as_slice(),
        ),
        (
            "selection",
            "selection_contracts.rs",
            selection::CONTRACTS.as_slice(),
        ),
        (
            "big_helper",
            "big_helper_contracts.rs",
            big_helpers::NATIVE.as_slice(),
        ),
        (
            "expression",
            "expression_contracts.rs",
            expressions::NATIVE.as_slice(),
        ),
        (
            "dynamic_aggregate",
            "dynamic_aggregate_contracts.rs",
            dynamic::NATIVE.as_slice(),
        ),
        ("ir", "ir_contracts.rs", ir::NATIVE.as_slice()),
        ("graph", "graph_contracts.rs", graph::NATIVE.as_slice()),
        (
            "nonlinear_wide",
            "nonlinear_wide_contracts.rs",
            nonlinear::NATIVE.as_slice(),
        ),
    ] {
        add(
            "analyzer",
            "direct",
            &format!("targets/analyzer/tests/{path}"),
            manifest,
            &format!("check_{name}_contracts"),
            entries,
        );
    }
    for (method, entries) in [
        ("kani", frontend::KANI.as_slice()),
        ("direct", frontend::NATIVE.as_slice()),
    ] {
        add(
            "frontend",
            method,
            "targets/frontend/contracts.rs",
            if method == "kani" {
                "targets/frontend/kani/Cargo.toml"
            } else {
                "targets/frontend/tests/Cargo.toml"
            },
            "check_contracts",
            entries,
        );
    }
    for (name, path, entries) in [
        (
            "check_contracts",
            "targets/emitter/contracts.rs",
            emitter::NATIVE.as_slice(),
        ),
        (
            "check_cross_contracts",
            "targets/emitter/cross_contracts.rs",
            emitter_cross::NATIVE.as_slice(),
        ),
    ] {
        add(
            "emitter",
            "direct",
            path,
            "targets/emitter/tests/Cargo.toml",
            name,
            entries,
        );
    }
    for (method, entries) in [
        ("kani", simulator::KANI.as_slice()),
        ("direct", simulator::NATIVE.as_slice()),
    ] {
        add(
            "simulator",
            method,
            "targets/simulator/contracts.rs",
            if method == "kani" {
                "targets/simulator/kani/Cargo.toml"
            } else {
                "targets/simulator/tests/Cargo.toml"
            },
            if method == "kani" {
                "check_contracts"
            } else {
                "check_simulator_contracts"
            },
            entries,
        );
    }
    add(
        "simulator",
        "direct",
        "targets/simulator/backend_contracts.rs",
        "targets/simulator/tests/Cargo.toml",
        "check_backend_contracts",
        &backend::NATIVE,
    );
    add(
        "simulator",
        "direct",
        "targets/simulator/optimization_contracts.rs",
        "targets/simulator/tests/Cargo.toml",
        "check_optimization_contracts",
        &optimization::NATIVE,
    );
    // These Rust tests invoke CBMC over generated C; they are not finite comparisons.
    add(
        "simulator",
        "cbmc",
        "targets/simulator/cbmc/contracts.rs",
        "targets/simulator/cbmc/Cargo.toml",
        "check_contracts",
        &generated_c::NATIVE,
    );
    add(
        "synthesizer",
        "direct",
        "targets/synthesizer/source_contracts.rs",
        "targets/synthesizer/tests/Cargo.toml",
        "check_source_contracts",
        &synth_source::NATIVE,
    );
    add(
        "synthesizer",
        "direct",
        "targets/synthesizer/sequential_contracts.rs",
        "targets/synthesizer/tests/Cargo.toml",
        "check_sequential_contracts",
        &synth_sequential::NATIVE,
    );
    add(
        "cache",
        "direct",
        "targets/cache/contracts.rs",
        "targets/cache/tests/Cargo.toml",
        "check_contracts",
        &cache::NATIVE,
    );
    out
}
