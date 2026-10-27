//! Shared progress format. Unknown or interrupted states never certify a run.
use std::{collections::BTreeMap, fmt::Write, fs, io, path::Path};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Status {
    Running,
    Success,
    Failed,
    #[default]
    Unknown,
}

impl Status {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Success => "success",
            Self::Failed => "failed",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Positive,
    MutationPreparation,
    Mutation,
}

impl Phase {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Positive => "positive",
            Self::MutationPreparation => "mutation-preparation",
            Self::Mutation => "mutation",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Summary {
    pub ci_run_id: String,
    pub ci_run_attempt: String,
    pub ci_job: String,
    pub target: String,
    pub backend: String,
    pub mode: String,
    pub contracts: String,
    pub manifest: String,
    pub adapter: String,
    pub adapter_kind: String,
    pub build_target: String,
    pub mutation_build_target: String,
    pub status: Status,
    pub phase: Option<Phase>,
    pub active_contract: String,
    pub active_harness: String,
    pub active_mutation: String,
    pub positive_passes: Option<usize>,
    pub mutation_passes: Option<usize>,
    pub exit_code: String,
}

impl Summary {
    // Keep the existing wire format so earlier diagnostic logs remain readable.
    pub fn parse(text: &str) -> Self {
        let fields: BTreeMap<_, _> = text
            .lines()
            .filter_map(|line| line.split_once('='))
            .collect();
        let get = |key| fields.get(key).copied().unwrap_or("");
        Self {
            ci_run_id: get("ci_run_id").into(),
            ci_run_attempt: get("ci_run_attempt").into(),
            ci_job: get("ci_job").into(),
            target: get("target").into(),
            backend: get("backend").into(),
            mode: get("mode").into(),
            contracts: get("contracts").into(),
            manifest: get("manifest").into(),
            adapter: get("adapter").into(),
            adapter_kind: get("adapter_kind").into(),
            build_target: get("build_target").into(),
            mutation_build_target: get("mutation_build_target").into(),
            status: match get("status") {
                "running" => Status::Running,
                "success" => Status::Success,
                "failed" => Status::Failed,
                _ => Status::Unknown,
            },
            phase: match get("phase") {
                "positive" => Some(Phase::Positive),
                "mutation-preparation" => Some(Phase::MutationPreparation),
                "mutation" => Some(Phase::Mutation),
                _ => None,
            },
            active_contract: get("active_contract").into(),
            active_harness: get("active_harness").into(),
            active_mutation: get("active_mutation").into(),
            positive_passes: get("positive_passes").parse().ok(),
            mutation_passes: get("mutation_passes").parse().ok(),
            exit_code: get("exit_code").into(),
        }
    }

    pub fn encode(&self) -> String {
        let mut text = String::new();
        for (key, value) in [
            ("ci_run_id", &self.ci_run_id),
            ("ci_run_attempt", &self.ci_run_attempt),
            ("ci_job", &self.ci_job),
            ("target", &self.target),
            ("backend", &self.backend),
            ("mode", &self.mode),
            ("contracts", &self.contracts),
            ("manifest", &self.manifest),
            ("adapter", &self.adapter),
            ("adapter_kind", &self.adapter_kind),
            ("build_target", &self.build_target),
            ("mutation_build_target", &self.mutation_build_target),
            ("active_contract", &self.active_contract),
            ("active_harness", &self.active_harness),
            ("active_mutation", &self.active_mutation),
            ("exit_code", &self.exit_code),
        ] {
            writeln!(text, "{key}={value}").unwrap();
        }
        writeln!(text, "status={}", self.status.as_str()).unwrap();
        if let Some(phase) = self.phase {
            writeln!(text, "phase={}", phase.as_str()).unwrap();
        }
        for (key, value) in [
            ("positive_passes", self.positive_passes),
            ("mutation_passes", self.mutation_passes),
        ] {
            if let Some(value) = value {
                writeln!(text, "{key}={value}").unwrap();
            }
        }
        text
    }

    pub fn write(&self, path: &Path) -> io::Result<()> {
        fs::write(path, self.encode())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_round_trip_and_legacy_logs() {
        let text = "ci_run_id=123\nstatus=failed\nphase=mutation\nactive_contract=carry\npositive_passes=2\nmutation_passes=1\nmanifest=path with spaces/a=b/Cargo.toml\nexit_code=Some(101)\n";
        let summary = Summary::parse(text);
        assert_eq!(summary.status, Status::Failed);
        assert_eq!(summary.phase, Some(Phase::Mutation));
        assert_eq!(summary.manifest, "path with spaces/a=b/Cargo.toml");
        assert_eq!(Summary::parse(&summary.encode()), summary);
    }

    #[test]
    fn missing_or_unknown_state_is_not_success() {
        for text in [
            "",
            "status=running",
            "status=invalid",
            "phase=mutation-preparation",
        ] {
            assert_ne!(Summary::parse(text).status, Status::Success);
        }
        assert_eq!(Summary::parse("positive_passes=bad").positive_passes, None);
    }
}
