# Testing strategy

QuadletLens requires both pure library tests and tests against real released generators. Documentation alone cannot establish parser and generated-command behavior across versions.

## Test layers

### Syntax tests

Cover sections, ordered and repeated keys, comments, blank lines, continuations, quoting, resets, specifiers, malformed lines, Unicode, line endings, and source spans.

### Typed-model tests

Cover every supported unit type, section, key, short/long value form, reference type, unknown entry, and generic systemd section.

### Round-trip and property tests

Verify that parsing never panics, canonical output is deterministic, preservation edits do not rewrite unrelated entries, and supported typed values survive parse-render-parse cycles.

### Capability-schema tests

Every catalogue entry must:

- conform to the schema
- have a unique stable identifier
- use a coherent version range
- link to evidence or state a documented evidence gap
- identify test coverage
- avoid contradictory native/fallback ranges

### Version-boundary tests

For every introduction, change, deprecation, removal, bug range, or fallback, test the nearest supported version below and above the boundary where available.

### Real-generator tests

Run fixtures through exact Podman system-generator versions from 5.4 through the newest supported release. Capture command, version, environment, exit status, generated service, and diagnostics. Separate syntactic acceptance from successful generated-command behavior.

Systemd-dependent fixtures record the systemd version and rootless/rootful context.

### Real-world fixtures

Fixtures require source provenance, redistribution permission, version assumptions, secret review, and an explanation of the protected behavior.

## Regression rule

Every compatibility fix adds a version-specific regression fixture. Catalogue corrections include a test demonstrating why the earlier claim was wrong or an explicit note explaining why automation is not currently possible.

## Canonical commands

The crate uses Rust 2024 with an MSRV of 1.85.0. `rust-toolchain.toml` pins the normal development toolchain; the explicit MSRV command prevents that pin from hiding accidental use of newer language or library features.

```shell
cargo fmt --all -- --check
cargo ci-check
cargo ci-clippy
cargo ci-test
cargo ci-doctest
RUSTDOCFLAGS="-D warnings" cargo ci-doc
cargo +1.85.0 ci-check
cargo deny check
```

The `ci-*` aliases use `--locked`, all workspace features, and all targets where the Cargo command supports them. CI also runs markdownlint and lychee over the documentation. Add exact catalogue-validation, property/fuzz, and real-generator matrix commands here before those harnesses become required checks.
