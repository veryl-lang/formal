#[path = "../../../contracts.rs"]
mod contracts;
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
    let selected = options
        .select("simulator", contracts::catalog(true))
        .unwrap_or_else(|e| {
            eprintln!("{e}");
            std::process::exit(2)
        });
    if options.list {
        for c in selected {
            println!("simulator\t{}\t{}\t{}", c.group, c.name, c.harness);
        }
        return;
    }
    let package = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo = package.ancestors().nth(3).unwrap();
    let mutations = if options.mode == Mode::Positive {
        Vec::new()
    } else {
        contracts::mutations(repo, &selected, true)
    };
    let (history, selected): (Vec<_>, Vec<_>) = selected.into_iter().partition(|c| {
        matches!(
            c.name,
            "log-full-pool-growth"
                | "log-prefix-append"
                | "log-chunk-split"
                | "log-variable"
                | "log-commit-unaligned"
                | "log-history-overlap"
                | "log-from-short"
                | "log-from-native128"
                | "log-from-wide130"
                | "log-from-noop"
                | "log-history-mixed"
                | "log-commit-chunks"
                | "log-two-word"
                | "log-scratch-constructor"
        )
    });
    let (history_mutations, mutations): (Vec<_>, Vec<_>) = mutations.into_iter().partition(|m| {
        matches!(
            m.contract.name,
            "log-full-pool-growth"
                | "log-prefix-append"
                | "log-chunk-split"
                | "log-variable"
                | "log-commit-unaligned"
                | "log-history-overlap"
                | "log-from-short"
                | "log-from-native128"
                | "log-from-wide130"
                | "log-from-noop"
                | "log-history-mixed"
                | "log-commit-chunks"
                | "log-two-word"
                | "log-scratch-constructor"
        )
    });
    let (two_word, history): (Vec<_>, Vec<_>) =
        history.into_iter().partition(|c| c.name == "log-two-word");
    let (two_word_mutations, history_mutations): (Vec<_>, Vec<_>) = history_mutations
        .into_iter()
        .partition(|m| m.contract.name == "log-two-word");
    let (pool, history): (Vec<_>, Vec<_>) = history
        .into_iter()
        .partition(|c| matches!(c.name, "log-full-pool-growth" | "log-prefix-append"));
    let (pool_mutations, history_mutations): (Vec<_>, Vec<_>) =
        history_mutations.into_iter().partition(|m| {
            matches!(
                m.contract.name,
                "log-full-pool-growth" | "log-prefix-append"
            )
        });
    let (full, ordinary): (Vec<_>, Vec<_>) = selected
        .into_iter()
        .partition(|c| c.group == "signed-shift-full");
    let (full_mutations, ordinary_mutations): (Vec<_>, Vec<_>) = mutations
        .into_iter()
        .partition(|m| m.contract.group == "signed-shift-full");
    let (direct_alias, ordinary): (Vec<_>, Vec<_>) = ordinary
        .into_iter()
        .partition(|c| c.name == "log-direct-alias");
    let (direct_alias_mutations, ordinary_mutations): (Vec<_>, Vec<_>) = ordinary_mutations
        .into_iter()
        .partition(|m| m.contract.name == "log-direct-alias");
    let harness_timeout = if ordinary.iter().any(|c| c.group == "window-log") {
        "180s"
    } else {
        "60s"
    };
    let runner = Runner {
        target: "simulator",
        repo,
        manifest: &package.join("Cargo.toml"),
        files: &[
            "Cargo.toml",
            "Cargo.lock",
            "README.md",
            "LICENSE-MIT",
            "LICENSE-APACHE",
        ],
        directories: &["veryl", "targets", "semantics", "support"],
    };
    // The alias contract's eight named covers replace redundant assertion
    // reachability queries; its domain, assertions, timeout and pointer settings stay fixed.
    if !direct_alias.is_empty()
        && (options.mode != Mode::Mutation || !direct_alias_mutations.is_empty())
    {
        runner.run_with_backend(
            Backend::KaniWithExplicitCovers {
                harness_timeout: "180s",
                cache_dereferences: false,
                restrict_vtable: false,
                auxiliary_covers: &[],
            },
            options.mode,
            &direct_alias,
            &direct_alias_mutations,
        );
    }
    if !ordinary.is_empty() && (options.mode != Mode::Mutation || !ordinary_mutations.is_empty()) {
        runner.run_with_backend(
            Backend::Kani { harness_timeout },
            options.mode,
            &ordinary,
            &ordinary_mutations,
        );
    }
    // Explicit covers avoid redundant per-assertion reachability queries.
    // The two-word source includes a third merged word and more cover queries;
    // pool histories preserve up to 64 live records and have their own solver budget.
    for (timeout, contracts, mutations) in [
        ("180s", history, history_mutations),
        ("300s", pool, pool_mutations),
        ("300s", two_word, two_word_mutations),
    ] {
        if contracts.is_empty() || (options.mode == Mode::Mutation && mutations.is_empty()) {
            continue;
        }
        runner.run_with_backend(
            Backend::KaniWithExplicitCovers {
                harness_timeout: timeout,
                cache_dereferences: true,
                restrict_vtable: false,
                auxiliary_covers: &[],
            },
            options.mode,
            &contracts,
            &mutations,
        );
    }
    // The bit-fill loop has up to192 iterations. Explicit covers and pointer
    // caching keep its complete-domain checks within a separate solver budget.
    if !full.is_empty() && (options.mode != Mode::Mutation || !full_mutations.is_empty()) {
        runner.run_with_backend(
            Backend::KaniWithExplicitCovers {
                harness_timeout: "300s",
                cache_dereferences: true,
                restrict_vtable: false,
                auxiliary_covers: &[],
            },
            options.mode,
            &full,
            &full_mutations,
        );
    }
}
