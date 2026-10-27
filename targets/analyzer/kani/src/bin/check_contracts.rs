//! Run selected analyzer contracts using the shared verification runner.
#[path = "check_contracts/analyzer.rs"]
mod analyzer;
use std::path::PathBuf;
use veryl_formal_runner::{Mode, Options, Runner, USAGE};

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
    Runner {
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
    }
    .run(options.mode, &contracts, &mutations);
}
