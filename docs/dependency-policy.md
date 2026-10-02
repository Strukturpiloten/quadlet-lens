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

Renovate's global three-day minimum release age applies to dependency updates. The exact-name
BoxFerry and Lens exception preserves the existing BoxFerry package names and includes
`compose-lens`, `podman-lens`, `quadlet-lens`, and `docker-lens`; it applies only to the Cargo manager
and crates.io (`crate`) datasource. It waives elapsed release age without changing approvals,
grouping, or automerge rules. Other ecosystems and similarly named packages retain the global delay.

Synthetic lock-file maintenance also uses a zero-day Renovate override. Both exceptions remain
subject to the shared, fail-closed lockfile release-age guard and the required aggregate PR gate.
For Cargo's canonical crates.io registry source, the guard's fixed in-house allowlist contains only
`compose-lens`, `podman-lens`, `quadlet-lens`, and `docker-lens`. It waives elapsed age only after a
bounded registry lookup returns a valid publication timestamp; a future timestamp still fails.
Unavailable or malformed registry evidence fails closed. Newly introduced third-party packages,
including transitive dependencies and BoxFerry packages outside that guard allowlist, still require
at least 72 hours. The guard has no CLI or environment override for its allowlist. Its immutable
shared-policy revision retains one Renovate owner. Podman discovery, Dev Container, and
checksum-pinned tool updates remain manual.

## Review checklist

Before merging a dependency change:

1. confirm the package and feature set are necessary;
2. inspect licenses, source, maintenance, and security history;
3. review the lockfile and transitive changes;
4. update an ADR if representation or public policy changes; and
5. run the complete repository gate.
