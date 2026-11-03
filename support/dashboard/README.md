# Verification dashboard

Build the self-contained HTML dashboard from the checkout:

```sh
cargo run --locked --manifest-path support/cli/Cargo.toml -- dashboard
```

Open `dist/index.html` in a browser. No server or JavaScript dependencies are
required. The official logo and fonts are embedded, including font licenses.
Generated files are ignored; `bootstrap` builds and validates them.

The host CLI imports the adapters' Rust catalogs, including factory-generated
contracts. IDs include target, method, adapter, and contract name. Generated-C
contracts are classified as CBMC, despite using a native Rust test adapter.
Ordinary Rust test definitions and registered contracts are different units.

Display groups and scope summaries live in `targets/<target>/coverage.json`.
`bootstrap` rejects missing, stale, or duplicate contract assignments for every
target. When adding/removing a contract, update its mapping and review the scope
summary against the harness. For non-Analyzer targets, `scopeDocs` reads the
maintained `//!` headers directly: changes to their assumptions and limits appear
in the next build. A missing header fails the build. The summary's short bounds
remain reviewed metadata, not a claim inferred from a contract name.

`run` records provenance once and passes it to each runner. The runner appends
per-check outcomes to `results.tsv` using its existing positive/mutation
classifiers. Started checks without a terminal record remain unfinished. Tool
failures, timeouts, and unrelated assertions do not count as mutation detection.

`report` exports `support/target/dashboard-evidence.json`. To import it, or a
directory of downloaded evidence artifacts:

```sh
cargo run --locked --manifest-path support/cli/Cargo.toml -- dashboard \
  --evidence support/target/dashboard-input
```

Only records with matching formal/Veryl SHAs and all dependency-lock hashes,
from clean source trees, contribute results. CI imports only its own run and
attempt. Legacy summaries and private `.local/` logs do not certify results.
Positive checks and mutation detection are separate; a passed checker does not
imply a successful CI job or a complete verification suite. A dirty checkout can
still generate the catalog, but its evidence remains unattributed.

CI uploads an inventory dashboard on PRs and an aggregated result dashboard after
verification jobs, including failed jobs. Download the `formal-dashboard-*`
artifact and open `index.html`. This workflow does not publish a hosted website.
