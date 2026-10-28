use crate::{Commands, Environment, Result, error, nightly, required};
use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    process::Command,
};

pub const KANI_VERSION: &str = "0.68.0";

#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum Mode {
    All,
    Positive,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Invocation {
    pub program: String,
    pub args: Vec<String>,
    pub env: BTreeMap<String, String>,
}
impl Invocation {
    pub fn new(program: &str, args: &[&str]) -> Self {
        Self {
            program: program.into(),
            args: args.iter().map(|s| (*s).into()).collect(),
            env: BTreeMap::new(),
        }
    }
    fn cargo(args: &str) -> Self {
        Self::new("cargo", &args.split_ascii_whitespace().collect::<Vec<_>>())
    }
    fn adapter(args: &str, groups: &[&str], mode: Mode) -> Self {
        Self::selected_adapter(args, "--group", groups, mode)
    }
    fn selected_adapter(args: &str, selector: &str, names: &[&str], mode: Mode) -> Self {
        let mut step = Self::cargo(args);
        for name in names {
            step.args.extend([selector.into(), (*name).into()]);
        }
        if mode == Mode::Positive {
            step.args.extend(["--mode".into(), "positive".into()]);
        }
        step
    }
}

pub trait Executor {
    fn execute(&mut self, root: &Path, invocation: &Invocation) -> Result<()>;
}
pub struct Streaming;
impl Executor for Streaming {
    fn execute(&mut self, root: &Path, invocation: &Invocation) -> Result<()> {
        println!("Running {} {:?}", invocation.program, invocation.args);
        let mut command = Command::new(&invocation.program);
        command.args(&invocation.args).current_dir(root);
        for (key, value) in &invocation.env {
            if let Some(path) = value.strip_prefix("$ROOT/") {
                command.env(key, root.join(path));
            } else {
                command.env(key, value);
            }
        }
        let status = command.status()?;
        if !status.success() {
            return Err(error(format!(
                "{} {:?} failed ({status})",
                invocation.program, invocation.args
            )));
        }
        Ok(())
    }
}

pub struct Plan {
    pub workspaces: &'static str,
    pub kani: bool,
    pub icarus: bool,
    pub steps: Vec<Invocation>,
}

#[derive(Clone, Copy)]
pub struct Shard {
    pub check: &'static str,
    pub suite: &'static str,
    pub timeout_minutes: u16,
    groups: &'static [&'static str],
    contracts: &'static [&'static str],
}

impl Shard {
    const fn new(
        check: &'static str,
        suite: &'static str,
        timeout_minutes: u16,
        groups: &'static [&'static str],
    ) -> Self {
        Self {
            check,
            suite,
            timeout_minutes,
            groups,
            contracts: &[],
        }
    }
    const fn value(suite: &'static str, contracts: &'static [&'static str]) -> Self {
        Self {
            contracts,
            ..Self::new("value-contracts", suite, 90, &[])
        }
    }
    fn target(&self) -> &'static str {
        match self.check {
            "value-contracts" => "value",
            "simulator-contracts" | "simulator-backend-contracts" | "generated-c-contracts" => {
                "simulator"
            }
            "synthesizer-contracts" => "synthesizer",
            "emitter-contracts" => "emitter",
            "frontend-contracts" => "frontend",
            "cache-contracts" => "cache",
            _ => unreachable!(),
        }
    }
}

pub const SHARDS: &[Shard] = &[
    Shard::new("value-contracts", "native", 90, &[]),
    // Balance proof and mutation time together. Tests require an exact partition
    // of the analyzer catalog, including full-native multiplication.
    Shard::value(
        "storage",
        &[
            "selection",
            "partial-assignment",
            "truncation",
            "public-resize-one",
            "public-resize-two",
            "public-resize-three",
            "public-resize-four",
            "extension",
            "logical-left-shift",
            "logical-right-shift",
            "arithmetic-left-shift",
            "arithmetic-right-shift",
            "shape-total",
            "shape-index",
            "shape-range",
            "shape-boundary",
            "shape-overflow",
            "selection-endpoints",
            "packed-unpacked-bits",
            "checked-selection-endpoints",
        ],
    ),
    Shard::value(
        "logic",
        &[
            "equality",
            "inequality",
            "wildcard-equality",
            "wildcard-inequality",
            "less-than",
            "less-equal",
            "greater-than",
            "greater-equal",
            "bitwise",
            "bit-not",
            "logical",
            "reductions",
            "condition-truth",
            "conditional",
            "public-conditional",
            "public-conditional-small-12",
            "public-conditional-small-21",
            "public-conditional-small-22",
            "public-conditional-widened",
            "public-conditional-word",
        ],
    ),
    Shard::value(
        "arithmetic",
        &[
            "full-multiply",
            "add-sub",
            "negation",
            "unary-plus",
            "public-arithmetic",
            "context-merge",
            "context-unary",
            "context-binary",
            "context-cast",
            "public-nonlinear",
            "multiplication",
            "division-remainder",
            "power",
            "multiplication-identities",
            "division-exceptions",
            "power-exceptions",
        ],
    ),
    Shard::new(
        "simulator-contracts",
        "core",
        90,
        &[
            "memory",
            "arithmetic",
            "shift",
            "signed-shift",
            "window",
            "commit",
        ],
    ),
    Shard::new("simulator-contracts", "bitwise", 90, &["bitwise"]),
    Shard::new("simulator-contracts", "window-log", 90, &["window-log"]),
    Shard::new(
        "simulator-backend-contracts",
        "core",
        60,
        &[
            "ff-pipeline",
            "ff-refinement",
            "event-state",
            "event-gate",
            "event-gate-cycle",
            "event-gate-external-write",
            "gate-max",
            "external-wide",
        ],
    ),
    Shard::new(
        "simulator-backend-contracts",
        "four-state",
        60,
        &[
            "four-state",
            "wide-four-state",
            "four-state-partial",
            "flat-four-state",
            "four-state-array",
            "comb-four-state-array",
            "unknown-array-write",
        ],
    ),
    Shard::new(
        "simulator-backend-contracts",
        "array-index",
        60,
        &["signed-array-index", "multidim-array-index"],
    ),
    Shard::new(
        "simulator-backend-contracts",
        "packed-write",
        60,
        &[
            "unknown-packed-select",
            "packed-select-forms",
            "signed-packed-select",
            "wide-packed-index",
        ],
    ),
    Shard::new(
        "simulator-backend-contracts",
        "packed-read",
        60,
        &[
            "packed-read-boundary",
            "wide-packed-read",
            "constant-packed-select",
            "constant-packed-edge",
            "constant-select-diagnostics",
            "constant-edge-diagnostics",
        ],
    ),
    Shard::new(
        "simulator-backend-contracts",
        "packed-layout",
        60,
        &["packed-dimension", "packed-range", "packed-stride-range"],
    ),
    Shard::new("synthesizer-contracts", "single", 60, &[]),
    Shard::new("emitter-contracts", "single", 60, &[]),
    Shard::new("generated-c-contracts", "single", 60, &[]),
    Shard::new("frontend-contracts", "single", 60, &[]),
    Shard::new("cache-contracts", "single", 60, &[]),
];

pub fn matrix(target: &str) -> Result<serde_json::Value> {
    let include: Vec<_> = SHARDS
        .iter()
        .filter(|shard| target == "all" || shard.target() == target)
        .map(|shard| {
            serde_json::json!({
                "check": shard.check, "suite": shard.suite, "timeout": shard.timeout_minutes
            })
        })
        .collect();
    if include.is_empty() {
        return Err(error(format!("unknown verification target: {target}")));
    }
    Ok(serde_json::json!({"include": include}))
}

pub fn plan(check: &str, suite: &str, mode: Mode) -> Result<Plan> {
    let shard = SHARDS
        .iter()
        .find(|shard| shard.check == check && shard.suite == suite)
        .ok_or_else(|| error(format!("unknown check/suite: {check}/{suite}")))?;
    let groups = shard.groups;
    let mut steps = Vec::new();
    match check {
        "value-contracts" if suite == "native" => {
            steps.push(Invocation::cargo(
                "fmt --manifest-path targets/analyzer/kani/Cargo.toml --check",
            ));
            steps.push(Invocation::cargo(
                "fmt --manifest-path semantics/Cargo.toml --check",
            ));
            steps.push(Invocation::cargo(
                "fmt --manifest-path support/Cargo.toml --check",
            ));
            steps.push(Invocation::cargo(
                "fmt -p veryl-formal-contract-tests --check",
            ));
            steps.push(Invocation::cargo(
                "test --locked --manifest-path support/Cargo.toml",
            ));
            steps.push(Invocation::cargo(
                "test --locked -p veryl-formal-contract-tests",
            ));
            steps.push(Invocation::cargo(
                "test --locked --manifest-path veryl/Cargo.toml -p veryl-analyzer --lib _contract",
            ));
            steps.push(Invocation::adapter("run --locked -p veryl-formal-contract-tests --example check_aggregate_contracts --", &[], mode));
            steps.push(Invocation::adapter("run --locked -p veryl-formal-contract-tests --example check_selection_contracts --", &[], mode));
            steps.push(Invocation::adapter(
                "run --locked -p veryl-formal-contract-tests --example check_ir_contracts --",
                &[],
                mode,
            ));
            steps.push(Invocation::adapter("run --locked -p veryl-formal-contract-tests --example check_expression_contracts --", &[], mode));
            steps.push(Invocation::adapter(
                "run --locked -p veryl-formal-contract-tests --example check_graph_contracts --",
                &[],
                mode,
            ));
            steps.push(Invocation::adapter("run --locked -p veryl-formal-contract-tests --example check_dynamic_aggregate_contracts --", &[], mode));
            steps.push(Invocation::adapter("run --locked -p veryl-formal-contract-tests --example check_nonlinear_wide_contracts --", &[], mode));
        }
        "value-contracts" => {
            steps.push(Invocation::selected_adapter(
                "run --locked --manifest-path targets/analyzer/kani/Cargo.toml --bin check_contracts --",
                "--contract",
                shard.contracts,
                mode,
            ));
        }
        "simulator-contracts" => {
            steps.push(Invocation::cargo(
                "fmt --manifest-path targets/simulator/kani/Cargo.toml --check",
            ));
            steps.push(Invocation::cargo(
                "fmt --manifest-path semantics/Cargo.toml --check",
            ));
            steps.push(Invocation::cargo(
                "fmt --manifest-path support/Cargo.toml --check",
            ));
            steps.push(Invocation::cargo(
                "fmt -p veryl-simulator-contract-tests --check",
            ));
            steps.push(Invocation::cargo(
                "fmt --manifest-path veryl/Cargo.toml -p veryl-simulator --check",
            ));
            steps.push(Invocation::cargo(
                "test --locked --manifest-path support/Cargo.toml",
            ));
            steps.push(Invocation::cargo(
                "test --locked --manifest-path semantics/Cargo.toml",
            ));
            steps.push(Invocation::cargo(
                "test --locked -p veryl-simulator-contract-tests --test wide_memory",
            ));
            steps.push(Invocation::cargo(
                "test --locked --manifest-path veryl/Cargo.toml -p veryl-simulator --lib wide_ops",
            ));
            steps.push(Invocation::cargo(
                "test --locked --manifest-path veryl/Cargo.toml -p veryl-simulator --lib write_log",
            ));
            steps.push(Invocation::adapter("run --locked -p veryl-simulator-contract-tests --example check_simulator_contracts --", groups, mode));
            steps.push(Invocation::adapter("run --locked --manifest-path targets/simulator/kani/Cargo.toml --bin check_contracts --", groups, mode));
        }
        "simulator-backend-contracts" => {
            steps.push(Invocation::cargo(
                "fmt -p veryl-simulator-contract-tests --check",
            ));
            if suite == "core" {
                steps.push(Invocation::cargo("test --locked --manifest-path veryl/Cargo.toml -p veryl-simulator --lib interpreted_narrow_ff_log_preserves_four_state_payload"));
                steps.push(Invocation::cargo("test --locked --manifest-path veryl/Cargo.toml -p veryl-simulator --lib wide_four_state_bitwise_results_are_canonical"));
                steps.push(Invocation::cargo("test --locked --manifest-path veryl/Cargo.toml -p veryl-simulator --lib dynamic_array_read_rejects_unknown_index"));
                steps.push(Invocation::cargo("test --locked --manifest-path veryl/Cargo.toml -p veryl-simulator --lib dynamic_array_write_rejects_unknown_index"));
                steps.push(Invocation::cargo("test --locked --manifest-path veryl/Cargo.toml -p veryl-simulator --lib packed_select_write_rejects_unknown_index"));
                steps.push(Invocation::cargo("test --locked --manifest-path veryl/Cargo.toml -p veryl-simulator --lib descending_packed_write_clips_both_boundaries"));
                steps.push(Invocation::cargo("test --locked --manifest-path veryl/Cargo.toml -p veryl-simulator --lib signed_packed_write_preserves_negative_positions"));
                steps.push(Invocation::cargo("test --locked --manifest-path veryl/Cargo.toml -p veryl-simulator --lib wide_packed_write_does_not_wrap"));
                steps.push(Invocation::cargo("test --locked --manifest-path veryl/Cargo.toml -p veryl-simulator --lib packed_read_preserves_partial_boundaries"));
                steps.push(Invocation::cargo("test --locked --manifest-path veryl/Cargo.toml -p veryl-simulator --lib wide_packed_read_preserves_partial_boundaries"));
                steps.push(Invocation::cargo("test --locked --manifest-path veryl/Cargo.toml -p veryl-simulator --lib constant_packed_read_uses_selected_width"));
                steps.push(Invocation::cargo("test --locked --manifest-path veryl/Cargo.toml -p veryl-simulator --lib constant_packed_edges_preserve_unknowns_and_lower_overlap"));
                steps.push(Invocation::cargo("test --locked --manifest-path veryl/Cargo.toml -p veryl-simulator --lib packed_multidimensional_select_checks_each_coordinate"));
                steps.push(Invocation::cargo("test --locked --manifest-path veryl/Cargo.toml -p veryl-simulator --lib packed_multidimensional_ranges_stay_in_the_selected_row"));
                steps.push(Invocation::cargo("test --locked --manifest-path veryl/Cargo.toml -p veryl-simulator --lib packed_multidimensional_ranges_preserve_element_strides"));
                steps.push(Invocation::cargo("test --locked --manifest-path veryl/Cargo.toml -p veryl-simulator --lib store_past_the_variable_from_a_loop"));
            }
            if suite == "core" {
                steps.push(Invocation::adapter("run --locked -p veryl-simulator-contract-tests --example check_optimization_contracts --", &[], mode));
            }
            steps.push(Invocation::adapter("run --locked -p veryl-simulator-contract-tests --example check_backend_contracts --", groups, mode));
        }
        "synthesizer-contracts" => {
            steps.push(Invocation::cargo(
                "fmt --manifest-path semantics/Cargo.toml --check",
            ));
            steps.push(Invocation::cargo(
                "fmt -p veryl-synthesizer-contract-tests --check",
            ));
            steps.push(Invocation::cargo(
                "fmt --manifest-path veryl/Cargo.toml -p veryl-synthesizer --check",
            ));
            steps.push(Invocation::cargo(
                "test --locked -p veryl-synthesizer-contract-tests",
            ));
            steps.push(Invocation::cargo(
                "test --locked --manifest-path veryl/Cargo.toml -p veryl-synthesizer --lib",
            ));
            steps.push(Invocation::adapter("run --locked -p veryl-synthesizer-contract-tests --example check_source_contracts --", &[], mode));
            steps.push(Invocation::adapter("run --locked -p veryl-synthesizer-contract-tests --example check_sequential_contracts --", &[], mode));
        }
        "emitter-contracts" => {
            steps.push(Invocation::cargo(
                "fmt --manifest-path targets/emitter/tests/Cargo.toml --check",
            ));
            steps.push(Invocation::cargo(
                "test --locked --manifest-path targets/emitter/tests/Cargo.toml",
            ));
            steps.push(Invocation::adapter("run --locked --manifest-path targets/emitter/tests/Cargo.toml --example check_contracts --", &[], mode));
            steps.push(Invocation::adapter("run --locked --manifest-path targets/emitter/tests/Cargo.toml --example check_cross_contracts --", &[], mode));
        }
        "generated-c-contracts" => {
            steps.push(Invocation::cargo(
                "fmt --manifest-path targets/simulator/cbmc/Cargo.toml --check",
            ));
            steps.push(Invocation::cargo(
                "test --locked --manifest-path targets/simulator/cbmc/Cargo.toml",
            ));
            steps.push(Invocation::adapter("run --locked --manifest-path targets/simulator/cbmc/Cargo.toml --example check_contracts --", &[], mode));
        }
        "frontend-contracts" => {
            steps.push(Invocation::cargo(
                "fmt -p veryl-frontend-contract-tests --check",
            ));
            steps.push(Invocation::cargo("fmt --manifest-path veryl/Cargo.toml -p veryl-parser -p veryl-formatter -p veryl-sourcemap -p veryl-ls --check"));
            steps.push(Invocation::cargo(
                "fmt --manifest-path targets/frontend/kani/Cargo.toml --check",
            ));
            steps.push(Invocation::cargo(
                "test --locked -p veryl-frontend-contract-tests",
            ));
            steps.push(Invocation::cargo(
                "test --locked --manifest-path veryl/Cargo.toml -p veryl-ls --lib",
            ));
            steps.push(Invocation::cargo(
                "test --locked --manifest-path veryl/Cargo.toml -p veryl-parser --lib",
            ));
            steps.push(Invocation::cargo(
                "test --locked --manifest-path veryl/Cargo.toml -p veryl-formatter --lib",
            ));
            steps.push(Invocation::adapter(
                "run --locked -p veryl-frontend-contract-tests --example check_contracts --",
                &[],
                mode,
            ));
            steps.push(Invocation::adapter("run --locked --manifest-path targets/frontend/kani/Cargo.toml --bin check_contracts --", &[], mode));
        }
        "cache-contracts" => {
            steps.push(Invocation::cargo(
                "fmt -p veryl-cache-contract-tests --check",
            ));
            steps.push(Invocation::cargo(
                "fmt --manifest-path veryl/Cargo.toml -p veryl-cache --check",
            ));
            steps.push(Invocation::cargo(
                "test --locked -p veryl-cache-contract-tests",
            ));
            steps.push(Invocation::cargo(
                "test --locked --manifest-path veryl/Cargo.toml -p veryl-cache",
            ));
            steps.push(Invocation::adapter(
                "run --locked -p veryl-cache-contract-tests --example check_contracts --",
                &[],
                mode,
            ));
            steps.push(Invocation::cargo(
                "build --locked --manifest-path veryl/Cargo.toml -p veryl --bin veryl",
            ));
            let mut step = Invocation::cargo(
                "test --locked -p veryl-cache-contract-tests --test cli -- --ignored",
            );
            step.env
                .insert("VERYL_BIN".into(), "$ROOT/target/debug/veryl".into());
            steps.push(step);
        }
        _ => unreachable!(),
    }
    for step in &mut steps {
        step.env
            .insert("CARGO_TARGET_DIR".into(), "$ROOT/target".into());
        if matches!(check, "simulator-backend-contracts" | "emitter-contracts") {
            step.env.insert(
                "VERYL_AOT_CACHE_DIR".into(),
                "$ROOT/target/formal-aot-cache".into(),
            );
        }
    }
    let workspaces = match check {
        "value-contracts" if suite == "native" => {
            ". -> target\nsupport -> target\nsemantics -> target"
        }
        "value-contracts" => {
            ". -> target\ntargets/analyzer/kani -> target\nsupport -> target\nsemantics -> target"
        }
        "simulator-contracts" => {
            ". -> target\ntargets/simulator/kani -> target\nsupport -> target\nsemantics -> target"
        }
        "simulator-backend-contracts" => ". -> target",
        "synthesizer-contracts" => ". -> target\nsupport -> target\nsemantics -> target",
        "emitter-contracts" => "targets/emitter/tests -> target",
        "generated-c-contracts" => "targets/simulator/cbmc -> target",
        "frontend-contracts" => ". -> target\ntargets/frontend/kani -> target\nsupport -> target",
        "cache-contracts" => ". -> target",
        _ => unreachable!(),
    };
    Ok(Plan {
        workspaces,
        kani: (check == "value-contracts" && suite != "native")
            || matches!(
                check,
                "simulator-contracts" | "generated-c-contracts" | "frontend-contracts"
            ),
        icarus: check == "emitter-contracts",
        steps,
    })
}

pub fn describe(plan: &Plan, env: &Environment) -> Result<()> {
    for step in &plan.steps {
        println!("{} {:?}", step.program, step.args);
    }
    if let Some(path) = env.get("GITHUB_OUTPUT") {
        let mut out = OpenOptions::new().create(true).append(true).open(path)?;
        writeln!(out, "kani={}\nkani-version={KANI_VERSION}", plan.kani)?;
        writeln!(
            out,
            "workspaces<<FORMAL_WORKSPACES\n{}\nFORMAL_WORKSPACES",
            plan.workspaces
        )?;
    }
    Ok(())
}

pub fn execute(root: &Path, steps: &[Invocation], executor: &mut impl Executor) -> Result<()> {
    for step in steps {
        executor.execute(root, step)?;
    }
    Ok(())
}

pub fn verify(
    root: &Path,
    candidate: &str,
    plan: &Plan,
    executor: &mut impl Executor,
    commands: &mut impl Commands,
) -> Result<()> {
    nightly::verify_clean(root, candidate, commands)?;
    let result = execute(root, &plan.steps, executor);
    let clean = nightly::verify_clean(root, candidate, commands);
    if let Err(error) = &clean {
        eprintln!("Veryl integrity check failed: {error}");
    }
    result.and(clean)
}

pub fn setup(
    root: &Path,
    plan: &Plan,
    env: &Environment,
    executor: &mut impl Executor,
) -> Result<()> {
    if plan.kani {
        if env.get("KANI_CACHE_HIT").is_some_and(|s| s == "true") {
            let home = Path::new(required(env, "HOME")?);
            let bundle = home.join(format!(".kani/kani-{KANI_VERSION}"));
            let toolchain = fs::read_to_string(bundle.join("rust-toolchain-version"))?;
            let toolchain = toolchain.trim();
            if toolchain.is_empty()
                || !toolchain
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'.' || b == b'_')
            {
                return Err(error("invalid cached Kani toolchain"));
            }
            executor.execute(
                root,
                &Invocation::new(
                    "rustup",
                    &["toolchain", "install", toolchain, "--profile", "minimal"],
                ),
            )?;
            let installed = home.join(".rustup/toolchains").join(toolchain);
            executor.execute(
                root,
                &Invocation::new(
                    "ln",
                    &[
                        "-sfn",
                        installed
                            .to_str()
                            .ok_or_else(|| error("non-UTF8 toolchain path"))?,
                        bundle
                            .join("toolchain")
                            .to_str()
                            .ok_or_else(|| error("non-UTF8 bundle path"))?,
                    ],
                ),
            )?;
        } else {
            executor.execute(
                root,
                &Invocation::new(
                    "cargo",
                    &[
                        "install",
                        "--locked",
                        "kani-verifier",
                        "--version",
                        KANI_VERSION,
                    ],
                ),
            )?;
            let mut setup = Invocation::cargo("kani setup");
            setup
                .env
                .insert("TAR_OPTIONS".into(), "--no-same-owner".into());
            executor.execute(root, &setup)?;
        }
        executor.execute(root, &Invocation::cargo("kani --version"))?;
    }
    if plan.icarus {
        executor.execute(root, &Invocation::new("sudo", &["apt-get", "update"]))?;
        executor.execute(
            root,
            &Invocation::new("sudo", &["apt-get", "install", "-y", "iverilog"]),
        )?;
    }
    Ok(())
}

pub fn bootstrap(root: &Path, executor: &mut impl Executor) -> Result<()> {
    execute(
        root,
        &[
            Invocation::cargo("fmt --manifest-path support/cli/Cargo.toml --check"),
            Invocation::cargo("fmt --manifest-path support/Cargo.toml --check"),
            Invocation::cargo("test --locked --manifest-path support/cli/Cargo.toml"),
            Invocation::cargo("test --locked --manifest-path support/Cargo.toml"),
        ],
        executor,
    )
}
