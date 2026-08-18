# Repository guidance for coding agents

This file applies to the entire QuadletLens repository.

## Read before changing code

Read these documents in order:

1. `README.md`
2. `docs/implementation-plan.md`
3. `docs/architecture.md`
4. `docs/project-structure.md`
5. `docs/capability-model.md`
6. `docs/testing.md`
7. `docs/dependency-policy.md`
8. `docs/decisions/README.md` and all accepted ADRs

Architectural changes require documentation and an ADR update in the same change.

## Scope

QuadletLens owns native Quadlet syntax, native typed models, document-set relationships, rendering, diagnostics, and capability evidence for supported Podman/systemd targets.

QuadletLens does not own Compose, Kubernetes, runtime inspection, BoxFerry's application model, or cross-format conversion decisions. It must not depend on BoxFerry.

## Origin policy

QuadletLens is implemented from scratch. Do not copy or mechanically translate source code from Podlet, Podman, systemd, or another Quadlet parser. Public documentation, released source behavior, and commands from identified versions may inform independent implementation and differential tests.

External fixtures and behavior-oracle results must record source, version, command, environment, license, and expected result.

## Non-negotiable behavior

- Unknown keys and generic systemd sections are never silently discarded.
- Preserve repeated keys and their ordering; a map is not a sufficient syntax representation.
- Keep systemd specifiers such as `%h` distinct from shell or Compose substitutions.
- Keep source syntax, typed values, and target-version validation as separate layers.
- A capability claim requires evidence and a test or an explicitly documented test gap.
- Distinguish a recognized key from a key that behaves correctly in a particular patch release.
- Invalid user input returns structured diagnostics and never panics.
- Rendering is deterministic for the same document, target, and options.
- Parsing and rendering never install, enable, start, or otherwise mutate units.

## Development rules

- Do not hard-code a closed Podman version enum in application logic; use the data-driven capability catalogue.
- Do not attempt to model all Podman features. Model Quadlet features and fallbacks relevant to the library contract.
- Keep target range evaluation independent of the parser.
- Add parser, renderer, capability-boundary, and real-generator tests with behavior changes.
- Record Podman tag/commit, documentation source, and observed generator behavior for capability updates.
- Update documentation and compatibility claims in the same change.
- Keep release notes and changelog entries concise: summarize user-visible feature families and
  link to canonical technical documentation instead of repeating model, fixture, and test details.
  Follow `docs/releasing.md`.
- Pin every GitHub Action to its full commit SHA and append its exact release tag as a comment. Verify new pins upstream; Renovate must preserve and update both values.

## Canonical development commands

The crate uses Rust 2024, supports Rust 1.85.0 and newer, and pins the normal development toolchain in `rust-toolchain.toml`.

```shell
./scripts/check-all.sh
./scripts/check-files.sh --check
cargo fmt --all -- --check
cargo ci-check
cargo ci-catalogue
cargo ci-generators
cargo ci-model
cargo ci-policy
cargo ci-clippy
cargo ci-test
cargo ci-doctest
RUSTDOCFLAGS="-D warnings" cargo ci-doc
cargo +1.85.0 ci-check
cargo +1.85.0 ci-policy
cargo deny check
```

The `ci-*` aliases in `.cargo/config.toml` use locked resolution and all workspace features and
targets where applicable. `cargo ci-catalogue` is the focused strict-schema and version-boundary
gate, `cargo ci-model` is the native typed-model and document-set gate, and `cargo ci-test` also includes their
non-network contract tests. `cargo ci-generators` runs the pinned minimum/image-boundary/current
smoke lane; `QUADLET_LENS_GENERATOR_LANE=full cargo ci-generators` is scheduled/manual and runs
every recorded official-image or exact-source generator.

## GitHub issue-to-PR workflow

When the user authorizes creating an issue, branch, commit, push, and pull request, follow this
sequence:

1. Inspect `git status` and the complete diff. Identify the exact pull-request scope and preserve
   unrelated changes.
2. Search for an existing issue, then create a focused GitHub issue when no duplicate exists.
3. Fetch `origin/main`, verify local `main` is synchronized, and create
   `TheRealBecks/issue<NUMBER>` from it.
4. Complete and review the change without staging unrelated files.
5. Run `./scripts/check-all.sh` from the repository root. Do not commit, push, or create the pull
   request unless every step passes. Any source, test, configuration, or documentation change made
   after the successful run invalidates it and requires the complete task to run again.
6. Stage only explicit in-scope paths, run `git diff --cached --check`, review the staged diff, and
   create one intentional commit.
7. Push the issue branch and open a ready-for-review pull request containing `Closes #<NUMBER>`.
8. Read the pull request back from GitHub and report its issue, branch, commit, validation result,
   URL, and current check state.

The issue may be created before local validation so failed work remains traceable, but a failed or
incomplete `./scripts/check-all.sh` run is a hard gate against commit, push, and pull-request
creation.

The primary Sol agent runs this workflow with high reasoning effort. Sol owns the issue and branch,
final integration and diff review, complete local gate, explicit staging, commit, push, pull-request
creation, and GitHub readback. Terra subagents may perform bounded research, implementation,
read-only review, or non-mutating verification assigned by Sol, but never execute the Git or GitHub
write steps. Because `./scripts/check-all.sh` formats repository files, its mandatory final run
remains Sol's responsibility and is not replaced by read-only Terra verification.

## Multi-agent coordination

- Delegate only concrete, bounded tasks with an independently verifiable result.
- Never run two source-writing agents in this repository checkout concurrently.
- Agents may write concurrently in separate repository checkouts only after the public contract is
  defined by the primary agent.
- Specification research and review agents remain read-only.
- Run a verifier only after this repository's writing agent has finished.
- Verification agents report failures but do not modify source, tests, configuration, or
  documentation.
- The primary agent reviews every diff and owns architectural and cross-repository API decisions.
- Subagents never commit, push, publish crates, create tags, or create releases.
- Prefer subagents for specification research, focused implementation, review, test execution, and
  log analysis when those tasks would otherwise pollute the primary thread or can proceed
  independently.
