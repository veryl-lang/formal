#[path = "../../optimization_contracts.rs"]
mod contracts;
use std::path::PathBuf;
use veryl_formal_runner::{Backend, Mode, Options, Runner, USAGE};
fn main() {
    if std::env::var_os("VERYL_AOT_CACHE_DIR").is_none() {
        let cache = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/aot-cache");
        // No other threads exist before parsing options and starting the runner.
        unsafe {
            std::env::set_var("VERYL_AOT_CACHE_DIR", cache);
        }
    }
    let options = Options::parse(std::env::args().skip(1)).unwrap_or_else(|error| {
        eprintln!("{error}\n{USAGE}");
        std::process::exit(2)
    });
    if options.help {
        println!("{USAGE}\nFinite native comparisons, not Kani proofs.");
        return;
    }
    let selected = options
        .select("simulator", contracts::catalog(false))
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
        contracts::mutations(repo, &selected)
    };
    Runner {
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
    }
    .run_with_backend(
        Backend::NativeTests {
            package: "veryl-simulator-contract-tests",
            test_target: "ff_pipeline",
        },
        options.mode,
        &selected,
        &mutations,
    );
}
