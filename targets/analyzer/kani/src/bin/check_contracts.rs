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
    let (expressions, explicit): (Vec<_>, Vec<_>) = explicit
        .into_iter()
        .partition(|c| c.group == "expression-context");
    let (expression_mutations, explicit_mutations): (Vec<_>, Vec<_>) = explicit_mutations
        .into_iter()
        .partition(|m| m.contract.group == "expression-context");
    let (big, explicit): (Vec<_>, Vec<_>) = explicit
        .into_iter()
        .partition(|c| c.group == "public-big-values");
    let (big_mutations, explicit_mutations): (Vec<_>, Vec<_>) = explicit_mutations
        .into_iter()
        .partition(|m| m.contract.group == "public-big-values");
    let (large_contexts, big): (Vec<_>, Vec<_>) = big
        .into_iter()
        .partition(|c| analyzer::uses_large_context_timeout(c));
    let (large_context_mutations, big_mutations): (Vec<_>, Vec<_>) = big_mutations
        .into_iter()
        .partition(|m| analyzer::uses_large_context_timeout(m.contract));
    let (extended, explicit): (Vec<_>, Vec<_>) = explicit
        .into_iter()
        .partition(|c| analyzer::uses_extended_logical_timeout(c));
    let (extended_mutations, explicit_mutations): (Vec<_>, Vec<_>) = explicit_mutations
        .into_iter()
        .partition(|m| analyzer::uses_extended_logical_timeout(m.contract));
    let (big_extended, big): (Vec<_>, Vec<_>) = big
        .into_iter()
        .partition(|c| analyzer::uses_extended_logical_timeout(c));
    let (big_extended_mutations, big_mutations): (Vec<_>, Vec<_>) = big_mutations
        .into_iter()
        .partition(|m| analyzer::uses_extended_logical_timeout(m.contract));
    for (timeout, cache_dereferences, restrict_vtable, contracts, mutations) in [
        ("60s", false, false, explicit, explicit_mutations),
        ("180s", false, false, extended, extended_mutations),
        ("60s", true, false, big, big_mutations),
        ("180s", true, false, big_extended, big_extended_mutations),
        ("300s", true, false, large_contexts, large_context_mutations),
        ("180s", true, true, expressions, expression_mutations),
    ] {
        if contracts.is_empty() || (options.mode == Mode::Mutation && mutations.is_empty()) {
            continue;
        }
        runner.run_with_backend(
            Backend::KaniWithExplicitCovers {
                harness_timeout: timeout,
                cache_dereferences,
                restrict_vtable,
                auxiliary_covers: analyzer::AUXILIARY_CONDITIONAL_COVERS,
            },
            options.mode,
            &contracts,
            &mutations,
        );
    }
}
