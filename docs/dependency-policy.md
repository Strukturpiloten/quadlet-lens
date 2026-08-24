# Dependency policy

Dependencies are part of the library contract. Prefer the standard library and focused crates with
active maintenance, clear licensing, and APIs that preserve Quadlet's ordered and source-aware
representation.

## Rust dependencies

- Use explicit compatible requirements; wildcard requirements are denied.
- Use crates.io releases unless an approved exception requires another source.
- Understand default features before enabling them.
- Avoid overlapping crates for the same concern.
- Commit `Cargo.lock` and use locked resolution in automation.
- Record a dependency that shapes public representation, catalogue data, or version policy in an
  ADR.

The capability catalogue uses Serde for its closed schema and TOML for decoding. They do not
participate in Quadlet syntax parsing or native value interpretation. Exact versions belong in
`Cargo.toml` and `Cargo.lock`.

## Licenses and advisories

[`deny.toml`](../deny.toml) is the source of truth for allowed licenses, advisories, bans, and
sources. Adding a license or source is a distribution decision and requires review of its
obligations.

Do not silence an advisory, allow a Git source, clarify a license, or skip a duplicate merely to
make CI pass. An exception must be narrow, versioned, justified in `deny.toml`, and explained in
the change. Use an ADR for a lasting exception.

Run:

```console
cargo deny --all-features check
```

## Repository tools

Formatting, linting, link checking, workflow analysis, coverage, API compatibility, and release
preparation are development tools rather than published Rust dependencies.

Their exact sources are:

- `package-lock.json` for Node tools;
- `scripts/install-file-tools.sh` for checksum-pinned native tools;
- `.devcontainer/Dockerfile` for container tooling;
- full commit pins and release comments in GitHub workflows; and
- `release-plz.toml` plus the release workflows for release preparation.

The Dev Container and CI run the same repository file checks. Renovate may propose version changes,
but every update receives the normal tests and review.

## Review checklist

Before merging a dependency change:

1. confirm the package and feature set are necessary;
2. inspect licenses, source, maintenance, and security history;
3. review the lockfile and transitive changes;
4. update an ADR if representation or public policy changes; and
5. run the complete repository gate.
