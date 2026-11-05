use crate::{Commands, Environment, Result, error, locks, nightly, required};
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
    Shard::value(
        "expression-context",
        &[
            "resolved-type-copy-update",
            "literal-metadata-copy",
            "literal-leaf-context",
        ],
    ),
    Shard::new("value-contracts", "native", 90, &[]),
    // Balance proof and mutation time together. Tests require an exact partition
    // of the analyzer catalog, including full-native multiplication.
    Shard::value(
        "storage",
        &[
            "selection",
            "public-selection-native",
            "partial-assignment",
            "public-assignment-native",
            "truncation",
            "public-resize-native",
            "public-native-zero-extension-wide",
            "public-native-zero-fill-extension-wide",
            "public-big-getters-128",
            "extension",
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
    // Keep the numeric API proofs and mutations together, independently of
    // storage geometry and real-library getter contracts.
    Shard::value(
        "big-signed-division-power-quotient",
        &[
            "signedq256-byte-api",
            "signedq256-quotient-word-api",
            "signedq256-domain-api",
            "signedq256-div-positive",
            "signedq256-div-negative",
            "signedq256-div-positive-positive-context",
            "signedq256-div-positive-negative-context",
            "signedq256-div-negative-positive-context",
            "signedq256-div-negative-negative-context",
        ],
    ),
    Shard::value(
        "big-signed-remainder-power-quotient",
        &[
            "signedq256-remainder-word-api",
            "signedq256-rem-positive",
            "signedq256-rem-negative",
            "signedq256-rem-positive-positive-context",
            "signedq256-rem-positive-negative-context",
            "signedq256-rem-negative-positive-context",
            "signedq256-rem-negative-negative-context",
        ],
    ),
    Shard::value(
        "big-power-quotient",
        &[
            "powerq256-byte-api",
            "powerq256-product-api",
            "powerq256-euclidean-api",
            "powerq256-domain-api",
            "powerq256-div",
            "powerq256-div-context",
            "powerq256-rem",
            "powerq256-rem-context",
        ],
    ),
    Shard::value(
        "big-unit-quotient",
        &[
            "unitq256-byte-api",
            "unitq256-euclidean-api",
            "unitq256-domain-api",
            "unitq256-div",
            "unitq256-div-context",
            "unitq256-rem",
            "unitq256-rem-context",
        ],
    ),
    Shard::value(
        "big-single-hot-power",
        &[
            "powerhot256-expansion-api",
            "powerhot256-byte-api",
            "powerhot256-coordinates-api",
            "powerhot256-successor-api",
            "powerhot256-domain-api",
            "powerhot256-prepared-signed",
            "powerhot256-prepared-unsigned",
            "powerhot256-context-signed",
            "powerhot256-context-unsigned",
        ],
    ),
    Shard::value(
        "big-all-ones-power",
        &[
            "powerones256-byte-api",
            "powerones256-successor-api",
            "powerones256-domain-api",
            "powerones256-prepared",
            "powerones256-context",
            "powerones256-prepared-witness",
            "powerones256-context-witness",
        ],
    ),
    Shard::value(
        "big-all-ones-multiplication",
        &[
            "ones256-byte-api",
            "ones256-inverse-api",
            "ones256-word-api",
            "ones256-right-domain-api",
            "ones256-left-domain-api",
            "ones256-right",
            "ones256-right-context",
            "ones256-left",
            "ones256-left-context",
        ],
    ),
    Shard::value(
        "big-signed-power-two-division",
        &[
            "signedpow256-byte-api",
            "signedpow256-coordinates-api",
            "signedpow256-word-api",
            "signedpow256-domain-api",
            "signedpow256-div",
            "signedpow256-div-nonnegative-context",
            "signedpow256-div-negative-context",
            "signedpow256-rem",
            "signedpow256-rem-nonnegative-context",
            "signedpow256-rem-negative-context",
        ],
    ),
    Shard::value(
        "big-signed-smaller-division",
        &[
            "signedsmall256-conversion-api",
            "signedsmall256-byte-api",
            "signedsmall256-word-api",
            "signedsmall256-domain-api",
            "signedsmall256-div",
            "signedsmall256-div-nonnegative-context",
            "signedsmall256-div-negative-context",
            "signedsmall256-rem",
            "signedsmall256-rem-nonnegative-context",
            "signedsmall256-rem-negative-context",
        ],
    ),
    Shard::value(
        "big-opposite-division",
        &[
            "opposite256-conversion-api",
            "opposite256-byte-api",
            "opposite256-context-domain-api",
            "opposite256-div",
            "opposite256-div-positive-context",
            "opposite256-div-negative-context",
            "opposite256-rem",
            "opposite256-rem-positive-context",
            "opposite256-rem-negative-context",
        ],
    ),
    Shard::value(
        "big-power-two-multiplication",
        &[
            "multiply256-byte-api",
            "multiply256-coordinates-api",
            "multiply256-right",
            "multiply256-right-context",
            "multiply256-left",
            "multiply256-left-context",
        ],
    ),
    Shard::value(
        "big-self-division",
        &[
            "division256-byte-api",
            "division256-self-signed-domain-api",
            "division256-self-div-unsigned",
            "division256-self-div-unsigned-context",
            "division256-self-div-signed",
            "division256-self-div-signed-context",
            "division256-self-rem-unsigned",
            "division256-self-rem-unsigned-context",
            "division256-self-rem-signed",
            "division256-self-rem-signed-context",
        ],
    ),
    Shard::value(
        "big-unsigned-division-classes",
        &[
            "division256-pow2-api",
            "division256-smaller-div",
            "division256-smaller-div-context",
            "division256-smaller-rem",
            "division256-smaller-rem-context",
            "division256-pow2-div",
            "division256-pow2-div-context",
            "division256-pow2-rem",
            "division256-pow2-rem-context",
        ],
    ),
    Shard::value(
        "big-division-identities",
        &[
            "special256-signed-api",
            "special256-signed-div-unit",
            "special256-signed-div-unit-context",
            "special256-signed-rem-unit",
            "special256-signed-rem-unit-context",
            "special256-zero-div",
            "special256-zero-div-context",
            "special256-zero-rem",
            "special256-zero-rem-context",
        ],
    ),
    Shard::value(
        "big-positive-power",
        &[
            "special256-power-api",
            "special256-power-one",
            "special256-power-one-context",
            "special256-power-unit-base",
            "special256-power-unit-base-context",
        ],
    ),
    Shard::value(
        "big-unit-identities",
        &[
            "unit256-api",
            "unit256-div-one-reference",
            "identity256-mul-right",
            "identity256-mul-right-context",
            "identity256-mul-left",
            "identity256-mul-left-context",
            "identity256-div-one",
            "identity256-div-one-context",
            "identity256-rem-one",
            "identity256-rem-one-context",
        ],
    ),
    Shard::value(
        "big-power-special",
        &[
            "power256-zero-api",
            "power256-unknown",
            "power256-unknown-context",
            "power256-negative",
            "power256-negative-context",
            "power256-zero",
            "power256-zero-context",
        ],
    ),
    Shard::value(
        "big-nonlinear-exceptions",
        &[
            "exception256-mul",
            "exception256-mul-context",
            "exception256-div",
            "exception256-div-context",
            "exception256-rem",
            "exception256-rem-context",
        ],
    ),
    Shard::value(
        "big-logical",
        &[
            "logical256-truth-reference",
            "logical256-and-native-big",
            "logical256-and-big-native",
            "logical256-and-big-big",
            "logical256-or-native-big",
            "logical256-or-big-native",
            "logical256-or-big-big",
        ],
    ),
    Shard::value(
        "big-ordering",
        &[
            "conversion256-bigint",
            "numeric256-order-api",
            "ordering256-less-unsigned",
            "ordering256-less-signed",
            "ordering256-less-unknown",
            "ordering256-less-eq-unsigned",
            "ordering256-less-eq-signed",
            "ordering256-less-eq-unknown",
            "ordering256-greater-unsigned",
            "ordering256-greater-signed",
            "ordering256-greater-unknown",
            "ordering256-greater-eq-unsigned",
            "ordering256-greater-eq-signed",
            "ordering256-greater-eq-unknown",
            "wiring256-less",
            "wiring256-less-eq",
            "wiring256-greater",
            "wiring256-greater-eq",
        ],
    ),
    Shard::value(
        "big-context-arithmetic",
        &[
            "numeric256-add",
            "numeric256-add-api",
            "equality256-eq",
            "equality256-ne",
            "equality256-wild-eq",
            "equality256-wild-ne",
            "arithmetic256-add",
            "arithmetic256-sub",
            "arithmetic256-neg",
            "arithmetic256-plus",
            "wiring256-plus",
            "wiring256-and",
            "wiring256-or",
            "wiring256-xor",
            "wiring256-xnor",
            "wiring256-not",
            "wiring256-left",
            "wiring256-right",
            "wiring256-arith-left",
            "wiring256-arith-right",
            "wiring256-eq",
            "wiring256-ne",
            "wiring256-wild-eq",
            "wiring256-wild-ne",
            "wiring256-add",
            "wiring256-sub",
            "wiring256-neg",
        ],
    ),
    Shard::value(
        "big-operators",
        &[
            "numeric256-shifts",
            "assignment256-native-native",
            "assignment256-native-big",
            "assignment256-big-native",
            "assignment256-big-big",
            "assignment256-high",
            "bitwise256-and",
            "bitwise256-or",
            "bitwise256-xor",
            "bitwise256-xnor",
            "not256",
            "shift256-left",
            "shift256-right",
            "shift256-arith-left",
            "shift256-arith-right-signed",
            "shift256-arith-right-unsigned",
            "reduction256-and",
            "reduction256-nand",
            "reduction256-or",
            "reduction256-nor",
            "reduction256-xor",
            "reduction256-xnor",
            "reduction256-logic-not",
            "numeric256-api",
        ],
    ),
    Shard::value(
        "big-values",
        &[
            "conditional-expansion-value-summary",
            "conditional-expansion-summary",
            "big-numeric-wide-api",
            "public-big-truncation",
            "public-big-expansion-zero",
            "public-big-expansion",
            "public-big-selection-wide",
            "public-conditional-wide-native-native",
            "public-conditional-wide-native-big",
            "public-conditional-wide-big-native",
            "public-conditional-wide-big-big",
            "big-numeric-api",
            "big-numeric-bridge",
            "big-truth-reference",
            "public-big-truth",
            "public-big-conditional-native",
            "public-native-sign-extension-wide",
            "public-big-selection-native",
        ],
    ),
    Shard::value(
        "shifts",
        &[
            "public-big-amount-decoding-128",
            "public-word64-big-amount-small",
            "public-word64-big-amount-high",
            "public-word64-big-amount-upper-x",
            "public-word64-big-amount-upper-z",
            "public-word32-big-amount-small",
            "public-word32-big-amount-high",
            "public-word32-big-amount-upper-x",
            "public-word32-big-amount-upper-z",
            "public-shift-big-amount-width-1-small",
            "public-shift-big-amount-width-1-high",
            "public-shift-big-amount-width-1-upper-x",
            "public-shift-big-amount-width-1-upper-z",
            "public-shift-big-amount-width-2-small",
            "public-shift-big-amount-width-2-high",
            "public-shift-big-amount-width-2-upper-x",
            "public-shift-big-amount-width-2-upper-z",
            "public-shift-big-amount-width-31-small",
            "public-shift-big-amount-width-31-high",
            "public-shift-big-amount-width-31-upper-x",
            "public-shift-big-amount-width-31-upper-z",
            "public-shift-big-amount-width-63-small",
            "public-shift-big-amount-width-63-high",
            "public-shift-big-amount-width-63-upper-x",
            "public-shift-big-amount-width-63-upper-z",
            "logical-left-shift",
            "public-shift-unsigned-native",
            "public-small-shift-continuation-width-1",
            "public-small-shift-continuation-width-2",
            "public-small-shift-continuation-width-3",
            "public-small-shift-continuation-width-4",
            "public-shift-boundary-width-31",
            "public-shift-boundary-width-32",
            "public-shift-boundary-width-63",
            "public-shift-boundary-width-64",
            "public-shift-widening-4-to-8",
            "public-shift-widening-4-to-64",
            "logical-right-shift",
            "arithmetic-left-shift",
            "arithmetic-right-shift",
        ],
    ),
    Shard::value(
        "nonlinear",
        &[
            "public-small-remainder-width-1-unsigned",
            "public-small-remainder-width-1-signed",
            "public-small-remainder-width-2-unsigned",
            "public-small-remainder-width-2-signed",
            "public-small-remainder-width-3-unsigned",
            "public-small-remainder-width-3-signed",
            "public-small-remainder-width-4-unsigned",
            "public-small-remainder-width-4-signed",
        ],
    ),
    // Keep comparison families separate from gates and conditionals as the proof
    // catalog grows. Tests require a complete proof and mutation partition.
    Shard::value(
        "comparison-equality",
        &[
            "equality",
            "public-equality-native",
            "public-inequality-native-unsigned-continuation",
            "public-inequality-native-signed-continuation",
            "public-small-wildcard-not-equal-width-1-by-4",
            "public-small-wildcard-not-equal-width-2-by-4",
            "public-small-wildcard-not-equal-width-3-by-4",
            "public-small-wildcard-not-equal-width-4-by-1",
            "public-small-wildcard-not-equal-width-4-by-2",
            "public-small-wildcard-not-equal-width-4-by-3",
            "public-small-wildcard-not-equal-width-4-by-4",
            "public-small-wildcard-inequality-unsigned-continuation",
            "public-small-wildcard-inequality-signed-continuation",
            "public-small-wildcard-equal-width-1-by-4",
            "public-small-wildcard-equal-width-2-by-4",
            "public-small-wildcard-equal-width-3-by-4",
            "public-small-wildcard-equal-width-4-by-1",
            "public-small-wildcard-equal-width-4-by-2",
            "public-small-wildcard-equal-width-4-by-3",
            "public-small-wildcard-equal-width-4-by-4",
            "public-small-wildcard-equality-continuation",
            "inequality",
            "wildcard-equality",
            "wildcard-inequality",
        ],
    ),
    Shard::value(
        "comparison-less",
        &[
            "public-less-equal-left-width4",
            "public-less-equal-right-width4",
            "public-small-less-equal-width-1-by-1",
            "public-small-less-equal-width-1-by-2",
            "public-small-less-equal-width-1-by-3",
            "public-small-less-equal-width-2-by-1",
            "public-small-less-equal-width-2-by-2",
            "public-small-less-equal-width-2-by-3",
            "public-small-less-equal-width-3-by-1",
            "public-small-less-equal-width-3-by-2",
            "public-small-less-equal-width-3-by-3",
            "public-small-less-width-1-by-4",
            "public-small-less-width-2-by-4",
            "public-small-less-width-3-by-4",
            "public-small-less-width-4-by-1",
            "public-small-less-width-4-by-2",
            "public-small-less-width-4-by-3",
            "public-small-less-width-4-by-4",
            "public-small-less-unsigned-continuation",
            "public-small-less-signed-continuation",
            "less-than",
            "less-equal",
        ],
    ),
    Shard::value(
        "comparison-greater",
        &[
            "public-small-greater-equal-width-1-by-4",
            "public-small-greater-equal-width-2-by-4",
            "public-small-greater-equal-width-3-by-4",
            "public-small-greater-equal-width-4-by-1",
            "public-small-greater-equal-width-4-by-2",
            "public-small-greater-equal-width-4-by-3",
            "public-small-greater-equal-width-4-by-4",
            "public-small-greater-equal-width-1-by-1",
            "public-small-greater-equal-width-1-by-2",
            "public-small-greater-equal-width-1-by-3",
            "public-small-greater-equal-width-2-by-1",
            "public-small-greater-equal-width-2-by-2",
            "public-small-greater-equal-width-2-by-3",
            "public-small-greater-equal-width-3-by-1",
            "public-small-greater-equal-width-3-by-2",
            "public-small-greater-equal-width-3-by-3",
            "public-small-greater-width-1-by-4",
            "public-small-greater-width-2-by-4",
            "public-small-greater-width-3-by-4",
            "public-small-greater-width-4-by-1",
            "public-small-greater-width-4-by-2",
            "public-small-greater-width-4-by-3",
            "public-small-greater-width-4-by-4",
            "public-small-greater-width-1-by-1",
            "public-small-greater-width-1-by-2",
            "public-small-greater-width-1-by-3",
            "public-small-greater-width-2-by-1",
            "public-small-greater-width-2-by-2",
            "public-small-greater-width-2-by-3",
            "public-small-greater-width-3-by-1",
            "public-small-greater-width-3-by-2",
            "public-small-greater-width-3-by-3",
            "greater-than",
            "greater-equal",
        ],
    ),
    // Split unary reductions so proof and mutation batches fit the job budget.
    Shard::value(
        "reductions",
        &[
            "reductions",
            "public-reduction-and-native-continuation",
            "public-reduction-and-width65",
            "public-reduction-and-width128",
            "reduction-and-reference",
            "public-reduction-nand-native-continuation",
            "public-reduction-nand-width65",
            "public-reduction-nand-width128",
        ],
    ),
    Shard::value(
        "reductions-or-xor",
        &[
            "reduction-or-reference",
            "public-reduction-or-native-continuation",
            "public-reduction-or-width65",
            "public-reduction-or-width128",
            "public-reduction-nor-native-continuation",
            "public-reduction-nor-width65",
            "public-reduction-nor-width128",
            "reduction-xor-reference",
            "public-reduction-xor-native-continuation",
            "public-reduction-xor-width65",
            "public-reduction-xor-width128",
            "public-reduction-xnor-native-continuation",
            "public-reduction-xnor-width65",
            "public-reduction-xnor-width128",
            "public-logical-not-native-continuation",
            "public-logical-not-width65",
            "public-logical-not-width128",
        ],
    ),
    Shard::value(
        "logic",
        &[
            "bitwise",
            "public-bitwise-native",
            "public-bitwise-widened-native-trace",
            "public-bitwise-widened-mutation-witness",
            "bit-not",
            "public-not-native",
            "logical",
            "public-logical-and-native-continuation",
            "logical-truth-reference",
            "public-logical-and-mutation-witness",
            "logical-truth-unrolled-reference",
            "public-logical-binary-domain",
            "public-logical-and-width65",
            "public-logical-or-native-continuation",
            "public-logical-or-width65",
            "public-logical-and-width65-witness",
            "public-logical-or-native-witness",
            "public-logical-or-width65-witness",
            "condition-truth",
            "conditional",
            "public-conditional-native",
        ],
    ),
    Shard::value(
        "arithmetic",
        &[
            "full-multiply",
            "add-sub",
            "negation",
            "unary-plus",
            "public-arithmetic-add",
            "public-arithmetic-subtract",
            "public-arithmetic-initial-pair",
            "public-binary-natural-signed-add",
            "public-binary-natural-signed-subtract",
            "public-binary-widened-unsigned-add",
            "public-binary-widened-unsigned-subtract",
            "public-binary-widened-signed-add",
            "public-binary-widened-signed-subtract",
            "public-binary-word64",
            "public-small-multiply",
            "public-small-divide-width-1-unsigned",
            "public-small-divide-width-1-signed",
            "public-small-divide-width-2-unsigned",
            "public-small-divide-width-2-signed",
            "public-small-divide-width-3-unsigned",
            "public-small-divide-width-3-signed",
            "public-small-divide-width-4-unsigned",
            "public-small-divide-width-4-signed",
            "public-arithmetic-unary-native",
            "public-arithmetic-unsigned-growth-plus",
            "public-arithmetic-unsigned-growth-negate",
            "public-arithmetic-signed-growth-plus",
            "public-arithmetic-signed-growth-negate",
            "public-arithmetic",
            "public-arithmetic-plus",
            "public-arithmetic-negate",
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
    Shard::new(
        "simulator-contracts",
        "signed-shift-full",
        45,
        &["signed-shift-full"],
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
            steps.push(Invocation::adapter("run --locked -p veryl-formal-contract-tests --example check_big_helper_contracts --", &[], mode));
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
            if suite != "signed-shift-full" {
                steps.push(Invocation::adapter("run --locked -p veryl-simulator-contract-tests --example check_simulator_contracts --", groups, mode));
            }
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
    lock_digest: Option<&str>,
    plan: &Plan,
    executor: &mut impl Executor,
    commands: &mut impl Commands,
) -> Result<()> {
    nightly::verify_clean(root, candidate, commands)?;
    if let Some(digest) = lock_digest {
        locks::verify(root, candidate, digest, commands)?;
    }
    let result = execute(root, &plan.steps, executor);
    let clean = nightly::verify_clean(root, candidate, commands);
    if let Err(error) = &clean {
        eprintln!("Veryl integrity check failed: {error}");
    }
    let unchanged = match lock_digest {
        Some(digest) => locks::verify(root, candidate, digest, commands),
        None => Ok(()),
    };
    if let Err(error) = &unchanged {
        eprintln!("Candidate lock integrity check failed: {error}");
    }
    result.and(clean).and(unchanged)
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
            // Rust caches can restore the launchers without Cargo's install metadata.
            executor.execute(
                root,
                &Invocation::new(
                    "cargo",
                    &[
                        "install",
                        "--locked",
                        "--force",
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
            Invocation::new(
                "python3",
                &[
                    "-m",
                    "unittest",
                    "discover",
                    "-s",
                    "support/dashboard/tests",
                ],
            ),
        ],
        executor,
    )
}
