//! Append-only per-check outcomes. A running record cannot certify completion.
use crate::{Contract, summary::Phase};
use std::{
    fs::OpenOptions,
    io::{self, Write},
    path::Path,
};

pub const HEADER: &str =
    "phase\tcontract\tharness\tmutation\tstatus\telapsed_ms\texit_code\tcovers\n";

pub fn record(
    path: &Path,
    contract: &Contract,
    phase: Phase,
    mutation: &str,
    status: &str,
    elapsed_ms: u128,
    exit_code: Option<i32>,
) -> io::Result<()> {
    let fields = [
        phase.as_str(),
        contract.name,
        contract.harness,
        mutation,
        status,
    ];
    if fields.iter().any(|s| s.contains(['\t', '\n', '\r'])) {
        return Err(io::Error::other("invalid evidence field"));
    }
    writeln!(
        OpenOptions::new().append(true).open(path)?,
        "{}\t{}\t{}\t{}\t{}\t{elapsed_ms}\t{}\t{}",
        fields[0],
        fields[1],
        fields[2],
        fields[3],
        fields[4],
        exit_code.map_or_else(|| "unknown".into(), |code| code.to_string()),
        contract.covers.len()
    )
}

pub fn outcome(accepted: bool, code: Option<i32>, log: &str) -> &'static str {
    if accepted {
        "passed"
    } else if code == Some(124) || log.to_ascii_lowercase().contains("timed out") {
        "timeout"
    } else {
        "failed"
    }
}
