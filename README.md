# Formal checks for Veryl

Checks selected behavior of the Veryl compiler against independent reference
models using Kani, generated-C/CBMC proofs, and executable comparisons. Scope,
assumptions, and specification references are documented beside the checks and
models.

```text
veryl/       Pinned Veryl submodule
targets/     Checks organized by compiler component
semantics/   Independent reference models
support/     Shared runners and CI CLI
```

## Running

Install Rust stable, a C compiler, and GNU `timeout`, then run from the repository
root:

```sh
git clone --recurse-submodules https://github.com/veryl-lang/formal.git
cd formal
cargo run --locked --manifest-path support/cli/Cargo.toml -- --help
cargo run --locked --manifest-path support/cli/Cargo.toml -- plan cache-contracts single
cargo run --locked --manifest-path support/cli/Cargo.toml -- run cache-contracts single --mode positive
```

`plan CHECK SUITE` lists the commands; `run CHECK SUITE` executes them. Available
check/suite pairs are defined in the [catalog](support/cli/src/checks.rs).
The value target uses `native`, `storage`, `logic`, `arithmetic`, and `nonlinear`
suites.
Omit `--mode positive` to include mutation checks. On Ubuntu, `setup CHECK SUITE`
installs additional verification tools required by the suite.

The Veryl checkout must match the pinned revision and have no source changes.
Logs and failure fixtures are written under `target/` directories; `report`
generates a verification summary.

## CI

The committed Veryl revision must pass the complete verification suite.
[CI](.github/workflows/verify.yml) runs formatting and host CLI/runner tests on
pull requests. Pushes to master run the complete suite, including proofs and
mutation checks.

Nightly resolves dependency locks once for upstream master and shares them across
all verification jobs. It updates the submodule pointer and those lock files
together only when all required checks pass. Verification failures create or
update an issue for local investigation. Manual workflow runs can select individual
targets.
