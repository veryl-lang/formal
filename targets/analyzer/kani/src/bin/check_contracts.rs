//! Run selected analyzer contracts using the shared verification runner.
#[path = "check_contracts/analyzer.rs"]
mod analyzer;
use std::path::PathBuf;
use veryl_formal_runner::{Backend, Mode, Options, Runner, USAGE};

fn main() {
    let options = Options::parse(std::env::args().skip(1)).unwrap_or_else(|error| {
        eprintln!("{error}\n{USAGE}");
        std::process::exit(2)
    });
    if options.help {
        println!("{USAGE}");
        return;
    }
    let contracts = options
        .select("analyzer", &analyzer::CONTRACTS)
        .unwrap_or_else(|error| {
            eprintln!("{error}");
            std::process::exit(2)
        });
    if options.list {
        for contract in contracts {
            println!(
                "analyzer\t{}\t{}\t{}",
                contract.group, contract.name, contract.harness
            );
        }
        return;
    }
    let proof = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo = proof.ancestors().nth(3).unwrap();
    let mutations = if options.mode == Mode::Positive {
        Vec::new()
    } else {
        analyzer::mutations(repo, &contracts)
    };
    assert!(
        options.mode != Mode::Mutation || !mutations.is_empty(),
        "selected contracts have no mutations"
    );
    let runner = Runner {
        target: "analyzer",
        repo,
        manifest: &proof.join("Cargo.toml"),
        files: &[
            "Cargo.toml",
            "Cargo.lock",
            "README.md",
            "LICENSE-MIT",
            "LICENSE-APACHE",
        ],
        directories: &["veryl", "targets", "semantics", "support"],
    };
    // Preserve the legacy diagnostic profile for existing contracts. The new
    // profile requires every declared cover to have a witness in the runner.
    let (explicit, legacy): (Vec<_>, Vec<_>) = contracts
        .into_iter()
        .partition(|c| analyzer::uses_explicit_covers(c));
    let (explicit_mutations, legacy_mutations): (Vec<_>, Vec<_>) = mutations
        .into_iter()
        .partition(|m| analyzer::uses_explicit_covers(m.contract));
    if !legacy.is_empty() && (options.mode != Mode::Mutation || !legacy_mutations.is_empty()) {
        runner.run(options.mode, &legacy, &legacy_mutations);
    }
    if !explicit.is_empty() && (options.mode != Mode::Mutation || !explicit_mutations.is_empty()) {
        runner.run_with_backend(
            Backend::KaniWithExplicitCovers {
                harness_timeout: "60s",
                auxiliary_covers: analyzer::AUXILIARY_NONLINEAR_COVERS,
            },
            options.mode,
            &explicit,
            &explicit_mutations,
        );
    }
}
