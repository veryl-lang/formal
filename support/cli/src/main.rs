use clap::{Args, Parser, Subcommand};
use std::{env, path::PathBuf};
use veryl_formal_ci::{Commands, Environment, Process, Result, checks, locks, nightly, report};

#[derive(Parser)]
#[command(
    name = "formal-ci",
    about = "Run and maintain independent Veryl checks"
)]
struct Cli {
    #[arg(long, global = true, default_value = ".", value_name = "PATH")]
    root: PathBuf,
    #[command(subcommand)]
    command: Operation,
}

#[derive(Args)]
struct Selection {
    check: String,
    suite: String,
}

#[derive(Args)]
struct Verification {
    #[command(flatten)]
    selection: Selection,
    #[arg(long, value_enum, default_value_t = checks::Mode::All)]
    mode: checks::Mode,
}

#[derive(Subcommand)]
enum Operation {
    /// Resolve the pinned or upstream revision and CI matrix.
    Prepare,
    /// Resolve and export one lock-file artifact for an upstream candidate.
    PrepareLocks { candidate: String },
    /// Install the prepared lock files after checking out their candidate.
    ApplyLocks { candidate: String, digest: String },
    /// Check out the selected Veryl commit.
    Checkout {
        #[arg(value_name = "SHA")]
        candidate: String,
    },
    /// Require the selected Veryl revision with no source changes.
    VerifyClean {
        #[arg(value_name = "SHA")]
        candidate: String,
    },
    /// Summarize checker results and failure diagnostics.
    Report,
    /// Advance a verified pointer or report an upstream failure.
    Finalize,
    /// Check formatting and test the CI host tools.
    Bootstrap,
    /// List commands and tools for a check suite.
    Plan(Verification),
    /// Install verification tools on Ubuntu.
    Setup(Selection),
    /// Execute a check suite and verify source integrity.
    Run(Verification),
}

fn run(cli: Cli) -> Result<()> {
    let root = cli.root.canonicalize()?;
    let env: Environment = env::vars().collect();
    let mut commands = Process;
    let mut executor = checks::Streaming;
    match cli.command {
        Operation::Prepare => nightly::prepare(&root, &env, &mut commands),
        Operation::PrepareLocks { candidate } => {
            let digest = locks::create(&root, &candidate, &mut commands)?;
            if let Some(path) = env.get("GITHUB_OUTPUT") {
                use std::io::Write;
                let mut output = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)?;
                writeln!(output, "digest={digest}")?;
            }
            println!("Candidate lock artifact: {} ({digest})", locks::BUNDLE);
            Ok(())
        }
        Operation::ApplyLocks { candidate, digest } => {
            locks::apply(&root, &candidate, &digest, &mut commands)
        }
        Operation::Checkout { candidate } => nightly::checkout(&root, &candidate, &mut commands),
        Operation::VerifyClean { candidate } => {
            nightly::verify_clean(&root, &candidate, &mut commands)
        }
        Operation::Report => report::write(&root, &env),
        Operation::Finalize => nightly::finalize(&root, &env, &mut commands),
        Operation::Bootstrap => checks::bootstrap(&root, &mut executor),
        Operation::Plan(Verification { selection, mode }) => checks::describe(
            &checks::plan(&selection.check, &selection.suite, mode)?,
            &env,
        ),
        Operation::Setup(selection) => checks::setup(
            &root,
            &checks::plan(&selection.check, &selection.suite, checks::Mode::All)?,
            &env,
            &mut executor,
        ),
        Operation::Run(Verification { selection, mode }) => {
            let plan = checks::plan(&selection.check, &selection.suite, mode)?;
            let candidate = match env.get("VERYL_CANDIDATE") {
                Some(candidate) => candidate.clone(),
                None => commands.run(&root, "git", &["rev-parse", "HEAD:veryl"])?,
            };
            let digest = env.get("FORMAL_LOCK_DIGEST").filter(|s| !s.is_empty());
            checks::verify(
                &root,
                &candidate,
                digest.map(String::as_str),
                &plan,
                &mut executor,
                &mut commands,
            )
        }
    }
}

fn main() {
    if let Err(error) = run(Cli::parse()) {
        eprintln!("formal-ci: {error}");
        std::process::exit(1);
    }
}
