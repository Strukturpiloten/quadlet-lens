# Dependency and license policy

Dependencies are design decisions, not incidental implementation details. Prefer the standard library and focused crates with active maintenance, a clear security history, and APIs that preserve Quadlet syntax, ordered entries, and target-version evidence.

## Baseline rules

- Use explicit, compatible Cargo version requirements by default; wildcard requirements are denied.
- Use an exact pin only for a documented compatibility or representation reason.
- Use crates.io releases by default. Unapproved registries and Git dependencies are denied.
- Keep default features only when they are understood and useful.
- Avoid overlapping crates that solve the same problem without a documented reason.
- Record dependencies that constrain unit representation, capability data, version ranges, or public APIs in an ADR.
- Commit `Cargo.lock` and use locked dependency resolution in CI.

## Catalogue dependencies

ADR 0004 approves `serde` 1.0 with derives and `toml` 1.1 for the strict versioned capability
catalogue. `serde` defines the closed schema boundary and `toml` decodes authored catalogue data;
neither participates in syntax parsing or native Quadlet value interpretation. Renovate and locked
CI resolution maintain their compatible versions.

## License allowlist

`deny.toml` is the machine-readable source of truth. The initial allowlist is deliberately narrow: Apache-2.0, Apache-2.0 with LLVM exception, BSD-2-Clause, BSD-3-Clause, ISC, MIT, MPL-2.0, Unicode-3.0, Unicode-DFS-2016, and Zlib.

Adding a license is a compatibility and distribution decision. Review its obligations before changing the allowlist. This policy records project intent and is not legal advice.

## Exceptions

Do not silence an advisory, allow a Git source, clarify a license, or skip a duplicate merely to make CI pass. An exception must be narrowly versioned, include a reason in `deny.toml`, and be explained in the change that introduces it. Use an ADR when the exception has lasting architectural or distribution consequences.

## Automation

Release preparation uses release-plz Action 0.5.131 at immutable commit
`2eb1d8bcb770b4c48ccfaad919734b38b51958c9`, release-plz CLI 0.3.160, and SHA-pinned GitHub App
token Action 3.2.0. These are repository-only workflow dependencies. The configuration disables
publication, tags, and GitHub releases; the existing protected workflow retains those
responsibilities. Renovate's GitHub Actions manager tracks the Action SHA and release comment, and
a custom workflow-tool manager tracks the release-plz CLI input. Review the Action, CLI,
least-privilege App token inputs, and preparation-only policy together.

Run `cargo deny check` after installing `cargo-deny`. CI checks advisories, licenses, bans, and
sources. Renovate proposes Cargo, npm development-tool, lockfile, Rust toolchain, Dev Container,
GitHub Actions, directly pinned workflow-tool, checksum-pinned file-tool, base-image, GitHub CLI,
documented Dev Container CLI, and Podman/generator-matrix updates. Updates still require the same
tests and review as human-authored dependency changes.

Repository-only file quality uses tools outside the published Rust dependency graph:
markdownlint-cli2 and Prettier for Markdown, Prettier for JSON and YAML, Tombi for TOML, shfmt and
ShellCheck for shell, and Hadolint for Dockerfiles. `package-lock.json` fixes the complete Node
tool graph, while `scripts/install-file-tools.sh` pins native Linux release assets and SHA-256
checksums. The Dev Container provides them, and CI plus release validation run the same
`scripts/check-files.sh --check` boundary. These tools do not affect the library package or MSRV.
The release-plz-owned root `CHANGELOG.md` remains Markdownlint-checked and release-structure-
validated but is the sole Markdown file excluded from Prettier through `.prettierignore`.
