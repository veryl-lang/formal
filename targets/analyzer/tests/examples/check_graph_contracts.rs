#[path = "../graph_contracts.rs"]
mod contracts;
use std::path::PathBuf;
use veryl_formal_runner::{Backend, Mode, Options, Runner, USAGE};
fn main() {
    let options = Options::parse(std::env::args().skip(1)).unwrap_or_else(|e| {
        eprintln!("{e}\n{USAGE}");
        std::process::exit(2)
    });
    if options.help {
        println!("{USAGE}");
        return;
    }
    let selected = options
        .select("analyzer", contracts::catalog(false))
        .unwrap_or_else(|e| {
            eprintln!("{e}");
            std::process::exit(2)
        });
    if options.list {
        for c in selected {
            println!("analyzer\t{}\t{}\t{}", c.group, c.name, c.harness);
        }
        return;
    }
    let package = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo = package.ancestors().nth(3).unwrap();
    let mutations = if options.mode == Mode::Positive {
        Vec::new()
    } else {
        contracts::mutations(repo, &selected)
    };
    Runner {
        target: "analyzer",
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
    }
    .run_with_backend(
        Backend::NativeTests {
            package: "veryl-formal-contract-tests",
            test_target: "dependency_graph",
        },
        options.mode,
        &selected,
        &mutations,
    );
}
